// tests/duration_tests.rs
use deep_time::{Lang, TSpan};

fn assert_duration(input: &str, expected_millis: i64) {
    let trimmed = input.trim();
    let dur = TSpan::from_str(trimmed, Lang::default())
        .unwrap_or_else(|e| panic!("Failed '{}': {}", input, e));

    let actual_millis = dur.to_ms() as i64;

    assert_eq!(actual_millis, expected_millis, "Input: '{}'", input);
}

#[test]
fn duration_comprehensive() {
    let cases: Vec<(&str, i64)> = vec![
        // === Legacy / basic ===
        ("0", 0),
        ("5000", 5000),
        ("86400000", 86400000),
        ("-250", -250),
        // === ISO 8601 ===
        ("PT2H30M", 9000000),
        ("P1DT12H", 129600000),
        ("P3D", 259200000),
        ("P2W", 1209600000),
        // ("PT1.5H", 5400000),
        // ("P1Y", 31_557_600_000),
        // ("P6M", 15778800000),
        ("-PT2H30M", -9000000),
        ("PT1H30M45S", 5445000),
        ("PT999.999999999S", 999999),
        // === Common formats ===
        ("5sec", 5000),
        ("1hr", 3600000),
        ("30min", 1800000),
        ("2days", 172800000),
        ("1wk", 604800000),
        ("1 hour", 3600000),
        ("2 days", 172800000),
        ("1hr 30min", 5400000),
        ("2 hours 30 minutes", 9000000),
        ("1 day and 2 hours", 93600000),
        ("1.5 hours", 5400000),
        (".5hr", 1800000),
        ("0hr", 0),
        ("0 days", 0),
        ("and 5 minutes", 300000),
        ("1 day, 2 hours", 93600000),
        ("1mo", 2629800000),
        // === Comma as thousands separator (English style) ===
        ("1,000 days", 86400000000),       // 1000 days
        ("2,500,000 seconds", 2500000000), // 2,500,000 s = 2.5 billion ms
        ("10,000 minutes", 600000000),     // 10,000 min
        ("1,234,567 milliseconds", 1234567),
        ("100,000,000 nanoseconds", 100),
        ("-1,500 seconds", -1500000),
        // === Decimal comma still works exactly as before (no regression) ===
        ("1.5 hours", 5400000),      // 1.5 hours
        ("123.45 minutes", 7407000), // 123.45 min
        ("0.5 seconds", 500),
        ("2.75 days", 237600000),
        ("999.999999999 seconds", 999999),
        // === Mixed thousands + decimal ===
        ("1,234.5 seconds", 1234500),
        ("10,000.75 hours", 36002700000),
        ("1,234,567.89 milliseconds", 1234567),
        // === Edge cases for the thousands-comma heuristic ===
        ("1.23 seconds", 1230),     // comma = decimal (1.23 s)
        ("1,234 seconds", 1234000), // comma = thousands (1,234 s)
        ("1,000,000,000 seconds", 1000000000000),
        ("1,000.0secs", 1000000),
        ("0.0005 hours", 1800),
        (".5hr", 1800000), // accepted as 0.5 h (consistent with .5h)
        // === Core "1d ,5h" style quirks (comma decimal + spaces in every possible spot) ===
        ("1d .5hr", 88_200_000),
        ("1d 5hr", 104_400_000),
        ("1d 5hr", 104_400_000),
        ("1d, 5hr", 104_400_000),
        ("1 day,5 hours", 104_400_000),
        ("1 day .5 hours", 88_200_000),
        ("1 day, 5 hours", 104_400_000),
        ("1 day , 5 hours", 104_400_000),
        ("  1d  ,   5hr  ", 104_400_000),
        ("1d  .5 hours", 88_200_000),
        // === Decimal comma with "and" and mixed separators ===
        ("1 day and .5hr", 88_200_000),
        ("1d and 0.5 hours", 88_200_000),
        ("1 day, and .75hr", 89_100_000),
        // === Other quirky decimal commas ===
        (".5hr", 1_800_000),
        (".75 days", 64_800_000),
        ("0.5days", 43_200_000),
        ("2.5 hours", 9_000_000),
        ("30.5 minutes", 1_830_000),
        ("5min,30.5sec", 330_500),
        ("1hr,30.5min", 5_430_000),
        // === Thousands comma vs decimal comma ===
        ("1,234 seconds", 1_234_000),
        ("1,234,567 milliseconds", 1_234_567), // should be treated as integer + ms unit
        // === Mixed separators, full words, abbreviations ===
        ("1 day, 2 hours, 30 minutes", 95_400_000),
        ("2 hours, 15 minutes and 30 seconds", 8_130_000),
        ("1day and 2hr 30min", 95_400_000),
        // === Negative, positive sign, leading zero quirks ===
        ("-1day .5hr", -88_200_000),
        ("+1d,5hr", 104_400_000),
        ("-1d,5hr", -104_400_000),
        ("-0.5 hours", -1_800_000),
        // === Edge-case whitespace and punctuation combos ===
        ("1day and  .5hr", 88_200_000),
        ("  .25day  and  3 hours  ", 32_400_000),
        ("1.5 days, and 30.5m", 131_430_000),
        ("1hr 30.5min 20sec", 5_450_000),
        // === ADDED: More comprehensive edge cases (concatenated components, heavy whitespace, multiple fractions, etc.) ===
        ("2day5hr", 190_800_000),       // concatenated shorthand: 2d + 5h
        ("1hr30min", 5_400_000),        // concatenated shorthand: 1h + 30m
        (" .5 seconds", 500),           // leading decimal comma + whitespace
        ("- .25day", -21_600_000),      // negative leading decimal comma
        ("+ .75 hours", 2_700_000),     // positive leading decimal comma
        ("1.5day 2.75hr", 139_500_000), // 1.5d + 2.75h
        ("1.5hr 30.25min", 7_215_000),  // 1.5h + 30.25m
        ("0.5day, 0.25hr, 15.5min", 45_030_000), // multiple fractional components anywhere
        ("1,234.567 seconds and .5 minutes", 1_264_567), // thousands + decimal comma + "and"
        ("   .5   days   and   10.25   minutes   ", 43_815_000), // heavy whitespace around leading decimal
        ("- .8day", -69_120_000), // negative leading decimal comma (different fraction)
        ("and .75 hours", 2_700_000), // "and" + leading decimal comma
        ("999,999,999 milliseconds", 999_999_999), // large thousands + unit
        ("1,000,000,000,000 nanoseconds", 1_000_000), // 1 trillion ns = exactly 1 second
        ("1day ,and 30min", 88_200_000), // decimal comma right before "and"
        ("2hr,30min", 9_000_000), // list-separator comma (no space before)
        (" .999 days", 86_313_600), // leading decimal close to 1.0
        ("1 day and .25 hours", 87_300_000), // 1d + 0.25h
        ("1day , .5hr", 88_200_000),
        ("1day 5hr ,", 104_400_000),
        (".5hr ,", 1_800_000),
        ("1hr2min3sec", 3_723_000),
        ("1 day and", 86_400_000),
        // ("P1Y2M3DT4.5H", 37_092_600_000),
        ("1_000_000_000_000 nanoseconds", 1_000_000),
        (
            "1 day, 5.5 hours, 30 minutes, 30 seconds, and 500 microseconds",
            108_030_000,
        ),
        ("5 seconds", 5000),
        ("2 minutes", 120000),
        ("1.5 hours", 5400000),
        ("3 days", 259200000),
        // Unit-first (new foreign-language support)
        ("hours 3", 10800000),
        ("days 5", 432000000),
        ("minutes 45", 2700000),
        // Scientific notation (e) – positive exponent
        ("1e3 seconds", 1000000),
        ("2.5e3 ms", 2500), // 2500 milliseconds
        ("1.5e4 milliseconds", 15000),
        // Scientific notation (e) – negative exponent + decimal
        ("1.5e-2 seconds", 15), // 0.015 s = 15 ms
        ("5e-3 hours", 18000),  // 0.005 h = 18 s = 18000 ms
        // Negative durations with e
        ("-2e3 milliseconds", -2000),
        ("-4e1 minutes", -2400000), // -40 minutes
        // Mixed: unit-first + e notation
        ("seconds 1.23e4", 12300000), // 12300 seconds
        ("hours 2.5e1", 90000000),    // 25 hours
        // Combined units (still works with new logic)
        ("1 hour 30 minutes", 5400000),
        ("2.5e2 seconds", 250000),
    ];

    for (input, expected) in cases {
        assert_duration(input, expected);
    }
}
