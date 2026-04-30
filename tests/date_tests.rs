use std::mem;

use deep_time_core::{ClockType, DateOrder, DateParseMode, Lang, ParseCfg, TimePoint};

#[cfg(feature = "jiff-tz")]
use jiff::{Zoned, civil::DateTime};

#[cfg(feature = "jiff-tz")]
#[test]
fn test_historical_iana_with_jiff() {
    // Historical / interesting IANA timezone cases.
    // Jiff is used as the ground-truth (full tzdb historical rules).
    // Your parser gets the exact same input string.

    let cases = vec![
        (
            "2020-06-15 14:30:00",
            "America/New_York",
            "NY summer (EDT = UTC-4)",
        ),
        (
            "2020-12-15 14:30:00",
            "America/New_York",
            "NY winter (EST = UTC-5)",
        ),
        (
            "2023-11-05 01:30:00",
            "America/New_York",
            "NY DST fall-back (ambiguous hour)",
        ),
        (
            "2023-03-12 02:30:00",
            "America/New_York",
            "NY DST spring-forward (gap)",
        ),
        ("1999-12-31 23:59:59", "Europe/London", "London historical"),
        (
            "1969-07-20 20:17:00",
            "America/New_York",
            "Moon landing (very old TZ rules)",
        ),
        ("1970-01-01 00:00:00", "UTC", "UTC edge case"),
    ];

    for (civil_str, iana_name, description) in cases {
        // ─── Jiff ground truth ─────────────────────────────────────────────────────
        let civil_dt: DateTime = civil_str
            .parse()
            .unwrap_or_else(|e| panic!("Jiff civil parse failed for '{}': {}", civil_str, e));

        let jiff_zoned: Zoned = civil_dt
            .in_tz(iana_name)
            .unwrap_or_else(|e| panic!("Jiff in_tz('{}') failed: {}", iana_name, e));

        // CHANGED: Convert to Timestamp so Jiff prints the *same* pure UTC RFC 3339
        // string with Z that deep_time_core::to_rfc3339() produces.
        let jiff_rfc = jiff_zoned.timestamp().to_string();

        // ─── Your library ──────────────────────────────────────────────────────────
        let our_input = format!("{} {}", civil_str, iana_name);

        let our_dt = TimePoint::from_str_parse(&our_input, &None)
            .unwrap_or_else(|e| panic!("deep_time_core failed on '{}': {}", our_input, e));

        let our_rfc = our_dt.to_str_rfc3339().unwrap();

        // ─── Assert (no more manual prints) ────────────────────────────────────────
        assert_eq!(
            our_rfc, jiff_rfc,
            "\n=== IANA Historical Test FAILED: {} ===\n\
             Input string       : {}\n\
             Jiff               : {}\n\
             deep_time_core     : {}\n",
            description, our_input, jiff_rfc, our_rfc
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────
fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
    let dt = TimePoint::from_str_parse(input.trim(), &opts)
        .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
    let actual = dt.to_str_rfc3339().unwrap();

    assert_eq!(actual, expected_rfc3339, "Input: {}", input);
}

fn assert_millis(input: &str, expected_millis: i128, opts: Option<ParseCfg>) {
    let millis = TimePoint::str_to_unix_ms(input, &opts)
        .unwrap_or_else(|| panic!("Failed millis parse: {}", input));
    assert_eq!(millis, expected_millis, "Input: {}", input);
}

fn assert_fails(input: &str, opts: Option<ParseCfg>) {
    assert!(
        TimePoint::from_str_parse(input, &opts).is_err(),
        "Expected failure: {}",
        input
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Comprehensive suite
// ─────────────────────────────────────────────────────────────────────────────

fn generate_date_test_cases() -> Vec<(String, String, Option<ParseCfg>)> {
    let mut cases = Vec::new();

    // ================================================================
    // 1. Core components – add new formats here for instant 10x coverage
    // ================================================================

    let date_only_bases = [
        "2024-03-14",
        "2024.03.14",
        "2024/03/14",
        "2024 03 14",
        "20240314",
        "14Mar24",
        "14-Mar-24",
        "14 Mar 2024",
        "Mar 14 2024",
        "March 14, 2024",
        "14 March 2024",
        "14-March-2024",
        "14/Mar/2024",
        "2024-W11-4",
        "2024-074",
        "2024/074",
        "2024.074",
        "240314",
        // "202403",
    ];

    let dt_separators = [" ", "T", "", ":"]; // ← ":" added (per request)

    let time_variants = [
        ("", "T00:00:00Z"), // date-only
        ("15:30", "T15:30:00Z"),
        ("15:30:45", "T15:30:45Z"),
        ("15:30:45.123", "T15:30:45.123Z"),
        ("15:30:45.123456", "T15:30:45.123456Z"),
        ("15:30:45.123456789", "T15:30:45.123456789Z"),
        ("03:30:45 PM", "T15:30:45Z"),
        ("03:30:45.123456789 PM", "T15:30:45.123456789Z"),
    ];

    let tz_variants = ["", "+0000", " +0000", "+00:00", " +00:00", "Z", "-0000"];

    let prefixes = ["", " ", "Thu ", "Thu. ", "Thursday, ", "Thu, "];

    let opts = ParseCfg {
        order: DateOrder::YearFirst,
        ..Default::default()
    };

    // ================================================================
    // 2. Generate massive combinatorial coverage
    // ================================================================

    for date in date_only_bases {
        for prefix in prefixes {
            for dt_sep in dt_separators {
                for (time_in, time_expected) in time_variants {
                    for tz in tz_variants {
                        // === FIXED: Prevent invalid date-only + timezone (no time) ===
                        if time_in.is_empty()
                            && !tz.is_empty()
                            && (dt_sep.is_empty() || dt_sep == " ")
                        {
                            continue;
                        }

                        // === UPDATED: Prevent malformed date-only with T or : separator ===
                        //     (now covers both "2024-03-14T" and "2024-03-14:")
                        if time_in.is_empty() && (dt_sep == "T" || dt_sep == ":") {
                            continue;
                        }

                        // === NEW: Prevent malformed 12-hour AM/PM + compact timezone suffix ===
                        //     e.g. "03:30:45 PMZ", "PM+0000", "PM-0000", etc.
                        //     (still allows valid spaced variants like "PM +0000")
                        if (time_in.contains("PM") || time_in.contains("AM"))
                            && !tz.is_empty()
                            && !tz.starts_with(' ')
                        {
                            continue;
                        }

                        // === Prevent compact NAMED date formats glued directly to time ===
                        //     e.g. "14Mar2415:30", "14-Mar-2415:30", "March 14, 202415:30"
                        //     (named = contains letters; numeric compacts like "2024031415:30" are untouched)
                        if dt_sep.is_empty()
                            && !time_in.is_empty()
                            && date.chars().any(|c| c.is_alphabetic())
                        {
                            continue;
                        }

                        // === NEW FIX: Prevent "Thu 2024 03 14:15:30" and similar bad cases ===
                        //     Day name + purely numeric spaced date (YYYY MM DD) + time glued with ":"
                        //     (or empty separator). These produce only "2 date digit groups" after the
                        //     day name (2024 03) before the time starts, which violates the rule you
                        //     described ("if there's only 2 date digit groups and a named then the
                        //     named should be the month, not day"). We keep weekday prefixes only
                        //     with named-month dates like "Thu Mar 14 2024".
                        if !prefix.trim().is_empty()
                            && (prefix.contains("Thu") || prefix.contains("Thursday"))
                            && date.contains(' ')
                            && !date.chars().any(|c| c.is_alphabetic())
                            && (dt_sep == ":" || dt_sep.is_empty())
                            && !time_in.is_empty()
                        {
                            continue;
                        }

                        // === NEW FIX: Prevent "2024 03 14:15:30" (the specific invalid format you asked for) ===
                        //     Purely numeric spaced date (YYYY MM DD) glued directly to time with ":"
                        //     This is the exact case you flagged. It is now completely excluded from
                        //     the generated test cases while leaving every other valid combination intact.
                        if date.contains(' ')
                            && !date.chars().any(|c| c.is_alphabetic())
                            && dt_sep == ":"
                            && !time_in.is_empty()
                        {
                            continue;
                        }

                        // === NEW: Prevent julians from being produced with times that dont have a time connector ===
                        //     e.g. "2024-07415:30" - should not be produced.
                        //     Only blocks the empty (glued) case; T, space, and colon are still allowed.
                        if date.len() == 8
                            && matches!(date.chars().nth(4), Some('-') | Some('/') | Some('.'))
                            && date[5..].chars().all(|c| c.is_ascii_digit())
                            && !time_in.is_empty()
                            && dt_sep.is_empty()
                        {
                            continue;
                        }

                        // === Prevent all pure numeric YYYYMMDDHHMM* format tests ===
                        //     (ONLY REMOVE THE PURE NUMERIC CASES — separators, connectors, T, :, ., space, etc. are OK!)
                        //     This blocks only fully-glued digit-only cases like "202403141530" or "240314153045".
                        //     Cases with any separator/connector (including the . in millis) are left untouched.
                        if dt_sep.is_empty()
                            && !time_in.is_empty()
                            && date.chars().all(|c| c.is_ascii_digit())
                            && time_in.chars().all(|c| c.is_ascii_digit())
                        {
                            continue;
                        }

                        // === Prevent 6-digit pure numeric date (e.g. 240314) glued directly to time
                        //     without any connector
                        //     Bad examples that are now blocked:
                        //       "24031415:30:00", "24031415:30:45", "24031415:30:45.123"
                        //     Allowed (they contain a connector):
                        //       "240314T15:30+0000", "240314 15:30", "240314:15:30:00", etc.
                        if date.len() == 6
                            && date.chars().all(|c| c.is_ascii_digit())
                            && dt_sep.is_empty()
                            && !time_in.is_empty()
                        {
                            continue;
                        }

                        let input = format!("{}{}{}{}{}", prefix, date, dt_sep, time_in, tz)
                            .trim()
                            .to_string();

                        let expected = if time_in.is_empty() {
                            "2024-03-14T00:00:00Z".to_string()
                        } else {
                            format!("2024-03-14{}", time_expected)
                        };

                        cases.push((input, expected, Some(opts.clone())));
                    }
                }
            }
        }
    }

    // ================================================================
    // 3. Truly special / one-off cases (unchanged)
    // ================================================================

    let special_cases: Vec<(String, String, Option<ParseCfg>)> = vec![
        (
            "2024-W11".to_string(),
            "2024-03-11T00:00:00Z".to_string(),
            None,
        ),
        (
            "2024-074T15:30:45.123456789Z".to_string(),
            "2024-03-14T15:30:45.123456789Z".to_string(),
            None,
        ),
        (
            "20240314T153045Z".to_string(),
            "2024-03-14T15:30:45Z".to_string(),
            None,
        ),
        (
            "2024-03-14T15:30:45Z".to_string(),
            "2024-03-14T15:30:45Z".to_string(),
            None,
        ),
        (
            "Thu Mar 14 15:30:45 2024".to_string(),
            "2024-03-14T15:30:45Z".to_string(),
            None,
        ),
        (
            "14/Mar/2024:15:30:45 +0000".to_string(),
            "2024-03-14T15:30:45Z".to_string(),
            None,
        ),
        // REMOVED pure-numeric YYYYMMDDHHMM* cases per user request:
        //   - "20240314153045"
        //   - "240314153045"
        // (the .millis version below is kept because it contains a separator)
        (
            "20240314153045.123456789".to_string(),
            "2024-03-14T15:30:45.123456789Z".to_string(),
            None,
        ),
        // Parse-mode / DateOrder / explicit format tests
        (
            "60400".to_string(),
            "2024-03-31T00:00:00Z".to_string(),
            Some(ParseCfg {
                mode: DateParseMode::Scientific,
                ..Default::default()
            }),
        ),
        (
            "05/06/2024".to_string(),
            "2024-06-05T00:00:00Z".to_string(),
            Some(ParseCfg {
                order: DateOrder::DayFirst,
                ..Default::default()
            }),
        ),
        (
            "14/03/2024 15:30".to_string(),
            "2024-03-14T15:30:00Z".to_string(),
            Some(ParseCfg {
                parse: Some(vec!["%d/%m/%Y %H:%M".to_string()]),
                ..Default::default()
            }),
        ),
        ("2024".to_string(), "2024-01-01T00:00:00Z".to_string(), None),
        (
            "-2024-03-14".to_string(),
            "-2024-03-14T00:00:00Z".to_string(),
            None,
        ),
        (
            "Dec 31 23:59:59".to_string(),
            "2025-12-31T23:59:59Z".to_string(),
            Some(ParseCfg {
                ref_time: Some(TimePoint::from_gregorian_ymdhms(
                    2025,
                    12,
                    31,
                    23,
                    59,
                    59,
                    0,
                    ClockType::UTC,
                )),
                ..Default::default()
            }),
        ),
    ];

    cases.extend(special_cases);
    cases
}

#[test]
fn date_parser_keeps_clock_type() {
    let tp1 = TimePoint::new(5, 0, ClockType::LTC);
    let tp2 = TimePoint::new(5, 0, ClockType::GPS);
    let xp1 = tp1.to_str("%Y-%m-%dT%H:%M:%S%.f %L").unwrap();
    let xp2 = tp2.to_str("%Y-%m-%dT%H:%M:%S%.f %L").unwrap();
    let res_tp1 = TimePoint::from_str(&xp1, "%Y-%m-%dT%H:%M:%S%.f %L", true, true, false).unwrap();
    let res_tp2 = TimePoint::from_str(&xp2, "%Y-%m-%dT%H:%M:%S%.f %L", true, true, false).unwrap();
    assert!(tp1 == res_tp1 && tp1.clock_type() == res_tp1.clock_type());
    assert!(tp2 == res_tp2 && tp2.clock_type() == res_tp2.clock_type());
}

#[test]
fn round_trip_fixed_offsets() {
    for tp in [
        TimePoint::new(5, 0, ClockType::TAI),
        TimePoint::new(5, 0, ClockType::UTC),
    ] {
        let xp1 = tp
            .to_str_with_offset("%Y-%m-%dT%H:%M:%S%.~f %:z %L", 3600)
            .unwrap();
        let tp2 = TimePoint::from_str_parse(&xp1, &None).unwrap();
        let xp2 = tp2
            .to_str_with_offset("%Y-%m-%dT%H:%M:%S%.~f %:z %L", 3600)
            .unwrap();
        let tp3 = TimePoint::from_str_parse(&xp2, &None).unwrap();
        assert_eq!(tp, tp3);
    }
}

#[test]
fn test_date_error() {
    let dt = TimePoint::from_str_parse("bad date", &None);
    match dt {
        Ok(_) => {}
        Err(e) => {
            eprintln!("EzError size = {} bytes", mem::size_of_val(&e));
            eprintln!("{}", e);
        }
    }
}

#[test]
fn date_parser_comprehensive() {
    let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![
        (
            "2024-03-14 03:30:45.123 PM",
            "2024-03-14T15:30:45.123Z",
            None,
        ),
        ("2024.03.14", "2024-03-14T00:00:00Z", None),
        ("The 14th of March, 2024", "2024-03-14T00:00:00Z", None),
        ("2024-03-14", "2024-03-14T00:00:00Z", None),
        ("2024/03/14", "2024-03-14T00:00:00Z", None),
        ("2024 03 14", "2024-03-14T00:00:00Z", None),
        ("2024.03.14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        (
            "2024.03.14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14T15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14T15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14 15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14 15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("14Mar24", "2024-03-14T00:00:00Z", None),
        ("20240314", "2024-03-14T00:00:00Z", None),
        ("2024-W11", "2024-03-11T00:00:00Z", None),
        ("2024-W11", "2024-03-11T00:00:00Z", None),
        ("2024-074", "2024-03-14T00:00:00Z", None),
        ("2024/074", "2024-03-14T00:00:00Z", None),
        ("2024.074", "2024-03-14T00:00:00Z", None),
        ("14Mar2024", "2024-03-14T00:00:00Z", None),
        ("14-Mar-24", "2024-03-14T00:00:00Z", None),
        (" 14-Mar-24", "2024-03-14T00:00:00Z", None),
        // REMOVED pure-numeric YYYYMMDDHHMM* cases (no separators at all) per user request:
        //   - "2403141530"
        //   - "202403141530"
        //   - "20240314153045" (both occurrences)
        //   - "240314153045"
        // (cases with separators/T/:/space/. remain)
        ("2024-W11-4", "2024-03-14T00:00:00Z", None),
        ("2024-W11-4", "2024-03-14T00:00:00Z", None),
        ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
        ("14-Mar-2024", "2024-03-14T00:00:00Z", None),
        ("Mar 14 2024", "2024-03-14T00:00:00Z", None),
        ("2024 Mar 14", "2024-03-14T00:00:00Z", None),
        ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
        ("14-Mar-2024", "2024-03-14T00:00:00Z", None),
        ("20240314 15:30", "2024-03-14T15:30:00Z", None),
        ("2024-074 15:30", "2024-03-14T15:30:00Z", None),
        ("2024.074 15:30", "2024-03-14T15:30:00Z", None),
        ("2024/074 15:30", "2024-03-14T15:30:00Z", None),
        ("20240314T153045", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30", "2024-03-14T15:30:00Z", None),
        ("2024-03-14T15:30", "2024-03-14T15:30:00Z", None),
        ("14-03-2024 15:30", "2024-03-14T15:30:00Z", None),
        ("14.03.2024 15:30", "2024-03-14T15:30:00Z", None),
        ("2024 03 14 15:30", "2024-03-14T15:30:00Z", None),
        ("2024-074T15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-074 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024/074T15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024/074 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024.074T15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024.074 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024 Mar 14 15:30", "2024-03-14T15:30:00Z", None),
        ("14 Mar 2024 15:30", "2024-03-14T15:30:00Z", None),
        ("2024 Mar 14 15:30", "2024-03-14T15:30:00Z", None),
        ("20240314 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-03-1415:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024/03/14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024 03 14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14-03-2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14.03.2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-W11-4 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-W11-4 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14/Mar/2024:15:30:45", "2024-03-14T15:30:45Z", None),
        ("14-Mar-2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("Mar 14 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14 Mar 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024 Mar 14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024 Mar 14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14 Mar 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14-Mar-2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14 Mar, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("Mar 14, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024.03.14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        (
            "2024-03-14T15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14T15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14 15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14 15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("2024-03-1415:30:45.123", "2024-03-14T15:30:45.123Z", None),
        ("14 Mar 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("Mar 14 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("Mar 14, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("2024-03-14 15:30:45+0000", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45+0000", "2024-03-14T15:30:45Z", None),
        ("Thu Mar 14 15:30:45 2024", "2024-03-14T15:30:45Z", None),
        ("14 Mar, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45+00:00", "2024-03-14T15:30:45Z", None),
        (
            "2024-03-1415:30:45.123456",
            "2024-03-14T15:30:45.123456Z",
            None,
        ),
        ("14/Mar/2024:15:30:45 +0000", "2024-03-14T15:30:45Z", None),
        (
            "Thu Mar 14 15:30:45 +00:00 2024",
            "2024-03-14T15:30:45Z",
            None,
        ),
        ("Mar 14, 2024 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
        ("Mar 14, 2024 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        (
            "Thu Mar 14 15:30:45 +0000 2024",
            "2024-03-14T15:30:45Z",
            None,
        ),
        (
            "Thu, 14 Mar 2024 15:30:45 +0000",
            "2024-03-14T15:30:45Z",
            None,
        ),
        (
            "Thu Mar 14 15:30:45 +00:00 2024",
            "2024-03-14T15:30:45Z",
            None,
        ),
        ("20240314T153045Z", "2024-03-14T15:30:45Z", None),
        ("2024-03-14T15:30:45Z", "2024-03-14T15:30:45Z", None),
        ("March 14, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("March 14, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        (
            "March 14, 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "March 14, 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("March 14, 2024", "2024-03-14T00:00:00Z", None),
        (
            "March 14, 2024 15:30:45 +00:00",
            "2024-03-14T15:30:45Z",
            None,
        ),
        (
            "March 14, 2024 15:30:45 +0000",
            "2024-03-14T15:30:45Z",
            None,
        ),
        (
            "March 14 2024 15:30:45 +00:00",
            "2024-03-14T15:30:45Z",
            None,
        ),
        (
            "Mar 14, 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Mar 14, 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Mar 14 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Mar 14 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("Thursday, 14 March 2024", "2024-03-14T00:00:00Z", None),
        ("Thursday 14 March 2024", "2024-03-14T00:00:00Z", None),
        ("Thursday, 14 Mar 2024", "2024-03-14T00:00:00Z", None),
        ("Thursday 14 Mar 2024", "2024-03-14T00:00:00Z", None),
        (
            "Thursday, 14 Mar 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Thursday 14 Mar 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Thu Mar 14 15:30:45.123456789 2024",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Thu Mar 14 15:30:45.123456789 +00:00 2024",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Thu Mar 14 15:30:45.123456789 +0000 2024",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("14 March 2024", "2024-03-14T00:00:00Z", None),
        ("14 March, 2024", "2024-03-14T00:00:00Z", None),
        ("14 March 2024", "2024-03-14T00:00:00Z", None),
        ("14 March, 2024", "2024-03-14T00:00:00Z", None),
        ("14-March-2024", "2024-03-14T00:00:00Z", None),
        ("14 March 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14 March, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14 March 2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("14-March-2024 15:30:45", "2024-03-14T15:30:45Z", None),
        ("Mar 14, 2024 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
        (
            "Mar 14, 2024 15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "Mar 14, 2024 15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar, 2024 15:30:45.123456789 +00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar, 2024 15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 March 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 March, 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 March 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14-March-2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("14 March 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        ("14 March, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
        (
            "14 March 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 March, 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar, 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar, 2024 03:30:45.123456789 PM",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14 Mar 2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14-Mar-2024 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "14/Mar/2024:15:30:45.123456789 +0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "20240314T153045.123456789Z",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074T15:30:45.123456789Z", // here
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024.074T15:30:45.123456789Z",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/074T15:30:45.123456789Z",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14T15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14T15:30:45.123456789+0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-03-1415:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/03/14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "20240314T153045.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "20240314153045.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "20240314 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024 March 14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024 Mar 14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        ("2024 March 14 15:30:45", "2024-03-14T15:30:45Z", None),
        ("2024 March 14", "2024-03-14T00:00:00Z", None),
        (
            "2024 Mar 14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024 03 14 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074T15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074T15:30:45.123456789+0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074 15:30:45.123456789+0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074T15:30:45.123456789+00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024-074 15:30:45.123456789+00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024.074T15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024.074 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024.074T15:30:45.123456789+0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024.074T15:30:45.123456789+00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/074T15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/074 15:30:45.123456789",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/074T15:30:45.123456789+0000",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        (
            "2024/074T15:30:45.123456789+00:00",
            "2024-03-14T15:30:45.123456789Z",
            None,
        ),
        // Common formats
        ("2024-03-14", "2024-03-14T00:00:00Z", None),
        ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
        ("15 Mar 2024 14:30", "2024-03-15T14:30:00Z", None),
        ("Mar 14, 2024", "2024-03-14T00:00:00Z", None),
        ("2024/03/14", "2024-03-14T00:00:00Z", None),
        ("14.03.2024", "2024-03-14T00:00:00Z", None),
        ("14/03.2024", "2024-03-14T00:00:00Z", None),
        // Pure-numeric special cases (date-only or epoch — kept; only full datetime smashed cases were removed)
        ("240314", "2024-03-14T00:00:00Z", None),
        ("202403", "2024-03-01T00:00:00Z", None),
        ("2024073", "2024-03-13T00:00:00Z", None),
        ("24073", "2024-03-13T00:00:00Z", None),
        ("60400", "2024-03-31T00:00:00Z", None),
        ("2460000", "2023-02-25T00:00:00Z", None),
        ("1700000000", "2023-11-14T22:13:20Z", None),
        ("1700000000000", "2023-11-14T22:13:20Z", None),
        // Parse modes
        (
            "60400",
            "2024-03-31T00:00:00Z",
            Some(ParseCfg {
                parse: None,
                mode: DateParseMode::Scientific,
                order: DateOrder::default(),
                ..Default::default()
            }),
        ),
        (
            "24073",
            "2024-03-13T00:00:00Z",
            Some(ParseCfg {
                parse: None,
                mode: DateParseMode::Legacy,
                order: DateOrder::default(),
                ..Default::default()
            }),
        ),
        // DateOrder
        (
            "05/06/2024",
            "2024-06-05T00:00:00Z",
            Some(ParseCfg {
                parse: None,
                mode: DateParseMode::Scientific,
                order: DateOrder::DayFirst,
                ..Default::default()
            }),
        ),
        (
            "05/06/2024",
            "2024-05-06T00:00:00Z",
            Some(ParseCfg {
                order: DateOrder::MonthFirst,
                ..Default::default()
            }),
        ),
        // Year-only
        ("2024", "2024-01-01T00:00:00Z", None),
        ("24", "2024-01-01T00:00:00Z", None),
        ("2024.074", "2024-03-14T00:00:00Z", None),
        // Negative years — Jiff pads negative years to 6 digits (Temporal/ISO rule)
        ("-2024-03-14", "-2024-03-14T00:00:00Z", None),
        ("-2025/01/01", "-2025-01-01T00:00:00Z", None),
        ("-0001-01-01", "-0001-01-01T00:00:00Z", None),
        // Syslog no-year
        (
            "Dec 31 23:59:59",
            "2025-12-31T23:59:59Z",
            Some(ParseCfg {
                ref_time: Some(TimePoint::from_gregorian_ymdhms(
                    2025,
                    12,
                    31,
                    23,
                    59,
                    59,
                    0,
                    ClockType::UTC,
                )),
                ..Default::default()
            }),
        ),
        // Explicit format
        (
            "14/03/2024 15:30",
            "2024-03-14T15:30:00Z",
            Some(ParseCfg {
                parse: Some(vec!["%d/%m/%Y %H:%M".to_string()]),
                ..Default::default()
            }),
        ),
        (
            "２０２６年４月５日",
            "2026-04-05T00:00:00Z",
            Some(ParseCfg {
                order: DateOrder::YearFirst,
                ..Default::default()
            }),
        ),
        (
            "߂߀߂߄-߀߄-߀߅",
            "2024-04-05T00:00:00Z",
            Some(ParseCfg {
                order: DateOrder::YearFirst,
                ..Default::default()
            }),
        ),
        (
            "1er janvier 2024",
            "2024-01-01T00:00:00Z",
            Some(ParseCfg {
                lang: Lang::Fr,
                ..Default::default()
            }),
        ),
    ];

    for (input, expected, opts) in cases {
        assert_date(input, expected, opts);
    }
    let cases = generate_date_test_cases();
    for (input, expected, opts) in cases {
        assert_date(&input, &expected, opts);
    }
}

fn generate_relative_date_test_cases() -> Vec<String> {
    let mut cases: Vec<String> = Vec::new();

    let core_phrases = ["now", "today", "tomorrow", "yesterday"];
    cases.extend(core_phrases.iter().map(|&s| s.to_string()));

    let numbers = [
        "1", "2", "3", "5", "10", "42", "0.5", "1.5", "2,5", "3.75", "1_000",
    ];
    let units = [
        "sec", "second", "seconds", "min", "minute", "minutes", "hr", "hour", "hours", "day",
        "days", "wk", "week", "weeks", "mo", "month", "months", "yr", "year", "years",
    ];

    let past_suffixes = [" ago"];
    let future_prefixes = ["in "];

    for num in numbers {
        for unit in units {
            // past forms
            for suffix in past_suffixes {
                cases.push(format!("{} {}{}", num, unit, suffix));
            }
            for prefix in future_prefixes {
                if !prefix.is_empty() || num.parse::<f64>().unwrap_or(0.0) != 1.0 {
                    cases.push(format!("{}{} {}", prefix, num, unit).trim().to_string());
                }
            }
        }
    }

    let multi_unit_cases = [
        "2 hours and 30 minutes ago",
        "1 day 12 hours from now",
        "3 weeks 4 days later",
        "1day ,5hr ago",
        "2hrs30min from now",
        "45min 15sec later",
        "1 day, 2 hours, and 30 minutes ago",
        "the 1.5 days ago",
        "in 2day 3hr 45min",
        "1 week and 2 days from now",
        "2 weeks 3 days 4 hours ago",
        "in 1 week and 2 days",
        "3 days 5 hours later",
    ];
    cases.extend(multi_unit_cases.iter().map(|&s| s.to_string()));

    cases
}

#[test]
fn relative_date_parser_comprehensive() {
    let cases = generate_relative_date_test_cases();
    let opts = Some(ParseCfg {
        ref_time: Some(TimePoint::new(5_000_000, 0, ClockType::UTC)),
        ..Default::default()
    });

    for input in cases {
        let result = TimePoint::from_str_parse(input.trim(), &opts);
        // eprintln!("Tried: {}, got result: {:?}", &input, result);
        assert!(result.is_ok(), "Failed to parse relative date: '{}'", input);
    }
}

#[test]
fn date_millis_and_errors() {
    assert_millis("2024-03-14", 1710374400000, None);
    assert_millis("1700000000000", 1700000000000, None);

    assert_fails("not-a-date", None);
    assert_fails("2024-13-01", None);
}
