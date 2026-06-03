#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod format_tests {
    use deep_time::{Dt, Scale};

    #[test]
    fn test_leap_second_gotcha_2016_12_31() {
        let leap = Dt::from_ymd(
            2016,
            12,
            31,
            23,
            59,
            60,
            123_456_789_000_000_000,
            Scale::UTC,
        );

        // Civil time must show sec=60
        let g = leap.to_ymd();
        assert_eq!(g.yr(), 2016);
        assert_eq!(g.mo(), 12);
        assert_eq!(g.day(), 31);
        assert_eq!(g.hr(), 23);
        assert_eq!(g.min(), 59);
        assert_eq!(g.sec(), 60);
        assert_eq!(g.attos(), 123_456_789_000_000_000);

        // Formatting must output "60" correctly
        let s = leap.to_str_lite("%Y-%m-%d %H:%M:%S.%f").unwrap();
        assert_eq!(
            s.as_str().unwrap(),
            "2016-12-31 23:59:60.123456789000000000"
        );

        // Trimmed fractional on leap second
        let s = leap.to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ").unwrap();
        assert_eq!(s.as_str().unwrap(), "2016-12-31T23:59:60.123456789Z");

        // leap second Unix timestamp (POSIX convention)
        let unix = leap.to_unix().to_sec64();
        assert_eq!(unix, 1483228799);
    }

    #[test]
    fn test_basic_formatting() {
        let t = Dt::from_ymd(2025, 4, 16, 14, 30, 45, 123_456_789_000_000_000, Scale::TAI);

        let s = t.to_str_lite("%Y-%m-%d %H:%M:%S.%f").unwrap();
        assert_eq!(
            s.as_str().unwrap(),
            "2025-04-16 14:30:45.123456789000000000"
        );

        let s = t.to_str_lite("%F").unwrap();
        assert_eq!(s.as_str().unwrap(), "2025-04-16");

        let s = t.to_str_lite("%T").unwrap();
        assert_eq!(s.as_str().unwrap(), "14:30:45");

        let s = t.to_str_lite("%R").unwrap();
        assert_eq!(s.as_str().unwrap(), "14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000, Scale::UTC);

        let s = t.to_str_lite("%f").unwrap();
        assert_eq!(s.as_str().unwrap(), "123456789000000000");

        let s = t.to_str_lite("%N").unwrap();
        assert_eq!(s.as_str().unwrap(), "123456789000000000");

        let s = t.to_str_lite("%.3f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123");

        let s = t.to_str_lite("%.6N").unwrap();
        assert_eq!(s.as_str().unwrap(), "123456");
    }

    #[test]
    fn test_iso_week_fix() {
        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
        let s = t2000.to_str_lite("%G-W%V-%u").unwrap();
        assert_eq!(s.as_str().unwrap(), "1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = Dt::from_ymd(2000, 1, 3, 12, 0, 0, 0, Scale::UTC);
        let s = t2000_monday.to_str_lite("%G-W%V-%u").unwrap();
        assert_eq!(s.as_str().unwrap(), "2000-W01-1");

        // Year with 53 weeks
        let t_week53 = Dt::from_ymd(2015, 12, 28, 12, 0, 0, 0, Scale::UTC);
        let s = t_week53.to_str_lite("%G-W%V").unwrap();
        assert_eq!(s.as_str().unwrap(), "2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = Dt::from_ymd(2025, 4, 16, 14, 30, 45, 0, Scale::UTC);

        let s = t.to_str_lite("%z").unwrap();
        assert_eq!(s.as_str().unwrap(), "+0000");

        let s = t.to_str_lite_with_offset("%:z", -5 * 3600).unwrap();
        assert_eq!(s.as_str().unwrap(), "-05:00");

        let s = t.to_str_lite_with_offset("%::z", -8 * 3600).unwrap();
        assert_eq!(s.as_str().unwrap(), "-08:00:00");

        let s = t.to_str_lite_with_offset("%z", 2 * 3600 + 30 * 60).unwrap();
        assert_eq!(s.as_str().unwrap(), "+0230");

        let s = t.to_str_lite("%Q").unwrap();
        assert_eq!(s.as_str().unwrap(), "UTC");

        let s = t.to_str_lite_with_offset("%z", -5 * 3600).unwrap();
        assert_eq!(s.as_str().unwrap(), "-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = Dt::from_ymd(2025, 4, 5, 3, 9, 7, 0, Scale::TAI);

        let s = t.to_str_lite("%d %H %M %S").unwrap();
        assert_eq!(s.as_str().unwrap(), "05 03 09 07");

        let s = t.to_str_lite("%_d %_H").unwrap();
        assert_eq!(s.as_str().unwrap(), " 5  3");

        let s = t.to_str_lite("%-d %-H").unwrap();
        assert_eq!(s.as_str().unwrap(), "5 3");

        let s = t.to_str_lite("%0d %0H").unwrap();
        assert_eq!(s.as_str().unwrap(), "05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 0, Scale::UTC); // Wednesday

        let s = t.to_str_lite("%A, %B %d, %Y").unwrap();
        assert_eq!(s.as_str().unwrap(), "Wednesday, April 16, 2025");

        let s = t.to_str_lite("%a %b %d").unwrap();
        assert_eq!(s.as_str().unwrap(), "Wed Apr 16");
    }

    #[test]
    fn test_weekday_and_week_number_directives() {
        // 2023-12-31 was a Sunday
        let sun = Dt::from_ymd(2023, 12, 31, 12, 0, 0, 0, Scale::UTC);
        assert_eq!(sun.to_str_lite("%A").unwrap().as_str().unwrap(), "Sunday");
        assert_eq!(sun.to_str_lite("%a").unwrap().as_str().unwrap(), "Sun");
        assert_eq!(sun.to_str_lite("%w").unwrap().as_str().unwrap(), "0");
        assert_eq!(sun.to_str_lite("%u").unwrap().as_str().unwrap(), "7");

        // 2024-01-01 was a Monday (ISO week 1 of 2024)
        let mon = Dt::from_ymd(2024, 1, 1, 12, 0, 0, 0, Scale::UTC);
        assert_eq!(mon.to_str_lite("%A").unwrap().as_str().unwrap(), "Monday");
        assert_eq!(mon.to_str_lite("%w").unwrap().as_str().unwrap(), "1");
        assert_eq!(mon.to_str_lite("%u").unwrap().as_str().unwrap(), "1");
        assert_eq!(mon.to_str_lite("%V").unwrap().as_str().unwrap(), "01");
        assert_eq!(mon.to_str_lite("%G").unwrap().as_str().unwrap(), "2024");
        assert_eq!(mon.to_str_lite("%g").unwrap().as_str().unwrap(), "24"); // ← added

        // 2000-01-01 was a Saturday
        let sat = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
        assert_eq!(sat.to_str_lite("%w").unwrap().as_str().unwrap(), "6");
        assert_eq!(sat.to_str_lite("%U").unwrap().as_str().unwrap(), "00");
        assert_eq!(sat.to_str_lite("%W").unwrap().as_str().unwrap(), "00");

        // 2015-12-28 → ISO week 53 of 2015
        let w53 = Dt::from_ymd(2015, 12, 28, 12, 0, 0, 0, Scale::UTC);
        assert_eq!(w53.to_str_lite("%V").unwrap().as_str().unwrap(), "53");
        assert_eq!(w53.to_str_lite("%G").unwrap().as_str().unwrap(), "2015");

        // 2024-12-30 → ISO week 1 of 2025
        let dec30 = Dt::from_ymd(2024, 12, 30, 12, 0, 0, 0, Scale::UTC);
        assert_eq!(dec30.to_str_lite("%V").unwrap().as_str().unwrap(), "01");
        assert_eq!(dec30.to_str_lite("%G").unwrap().as_str().unwrap(), "2025");
        assert_eq!(dec30.to_str_lite("%g").unwrap().as_str().unwrap(), "25"); // ← added
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = Dt::from_ymd(1970, 1, 1, 0, 0, 0, 0, Scale::UTC);

        let s = t.to_str_lite("%s").unwrap();
        assert_eq!(s.as_str().unwrap(), "0");

        let s = t.to_str_lite("%j").unwrap();
        assert_eq!(s.as_str().unwrap(), "001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        // Negative & zero years
        let t_neg = Dt::from_ymd(-123, 6, 15, 9, 30, 45, 0, Scale::TAI);
        let s = t_neg.to_str_lite("%Y-%m-%d").unwrap();
        assert_eq!(s.as_str().unwrap(), "-0123-06-15");

        let s = t_neg.to_str_lite("%C").unwrap();
        assert_eq!(s.as_str().unwrap(), "-2");

        let t_zero = Dt::from_ymd(0, 1, 1, 0, 0, 0, 0, Scale::TAI);
        let s = t_zero.to_str_lite("%Y").unwrap();
        assert_eq!(s.as_str().unwrap(), "0000");

        // ISO week year-boundary cases
        let t_2024_dec30 = Dt::from_ymd(2024, 12, 30, 12, 0, 0, 0, Scale::TAI);
        let s = t_2024_dec30.to_str_lite("%G-W%V-%u").unwrap();
        assert_eq!(s.as_str().unwrap(), "2025-W01-1");

        let t_2024_dec31 = Dt::from_ymd(2024, 12, 31, 12, 0, 0, 0, Scale::TAI);
        let s = t_2024_dec31.to_str_lite("%G-W%V-%u").unwrap();
        assert_eq!(s.as_str().unwrap(), "2025-W01-2");

        let t_2025_jan1 = Dt::from_ymd(2025, 1, 1, 12, 0, 0, 0, Scale::TAI);
        let s = t_2025_jan1.to_str_lite("%G-W%V-%u").unwrap();
        assert_eq!(s.as_str().unwrap(), "2025-W01-3");

        let t_2015_dec28 = Dt::from_ymd(2015, 12, 28, 12, 0, 0, 0, Scale::TAI);
        let s = t_2015_dec28.to_str_lite("%G-W%V").unwrap();
        assert_eq!(s.as_str().unwrap(), "2015-W53");

        // Week numbers %U / %W edge cases
        let t2000 = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::TAI);
        let s = t2000.to_str_lite("%U").unwrap();
        assert_eq!(s.as_str().unwrap(), "00");

        let s = t2000.to_str_lite("%W").unwrap();
        assert_eq!(s.as_str().unwrap(), "00");

        let t_sun = Dt::from_ymd(2023, 12, 31, 12, 0, 0, 0, Scale::TAI);
        let s = t_sun.to_str_lite("%U").unwrap();
        assert_eq!(s.as_str().unwrap(), "53");

        // Fractional seconds extremes
        let t_frac = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 0, Scale::TAI);
        let s = t_frac.to_str_lite("%.0f").unwrap();
        assert_eq!(s.as_str().unwrap(), "");

        let s = t_frac.to_str_lite("%.9N").unwrap();
        assert_eq!(s.as_str().unwrap(), "000000000");

        let s = t_frac.to_str_lite("%S.%f").unwrap();
        assert_eq!(s.as_str().unwrap(), "00.000000000000000000");

        // Timezone offsets with seconds & different colon counts
        let ny = -5 * 3600;
        let la = -8 * 3600;
        let weird = 3600 + 23 * 60 + 45;

        let s = t_frac.to_str_lite_with_offset("%::z", ny).unwrap();
        assert_eq!(s.as_str().unwrap(), "-05:00:00");

        let s = t_frac.to_str_lite_with_offset("%:z", la).unwrap();
        assert_eq!(s.as_str().unwrap(), "-08:00");

        let s = t_frac.to_str_lite_with_offset("%::z", weird).unwrap();
        assert_eq!(s.as_str().unwrap(), "+01:23:45");

        // Padding + explicit width + flags combined
        let t_small = Dt::from_ymd(2025, 4, 5, 3, 9, 7, 0, Scale::TAI);

        let s = t_small.to_str_lite("%03d").unwrap();
        assert_eq!(s.as_str().unwrap(), "005");

        let s = t_small.to_str_lite("%-5H").unwrap();
        assert_eq!(s.as_str().unwrap(), "3");

        let s = t_small.to_str_lite("%_3M").unwrap();
        assert_eq!(s.as_str().unwrap(), "  9");

        // Negative Unix timestamp
        let t_neg_unix = Dt::from_ymd(1969, 12, 31, 23, 59, 59, 0, Scale::TAI);
        let s = t_neg_unix.to_str_lite("%s").unwrap();
        assert_eq!(s.as_str().unwrap(), "-1");

        // Large positive
        let t_large = Dt::from_ymd(2038, 1, 19, 3, 14, 7, 0, Scale::TAI);
        let s = t_large.to_str_lite("%s").unwrap();
        assert_eq!(s.as_str().unwrap(), "2147483647");
    }

    #[test]
    fn test_fractional_trim_flag() {
        // Value with trailing zeros in fractional part
        let t = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000, Scale::TAI);

        let s = t.to_str_lite("%.~f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123456789");

        let s = t.to_str_lite("%.9~f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123456789");

        let s = t.to_str_lite("%.18~f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123456789");

        // Value that becomes all zeros after trimming
        let t_zero = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 0, Scale::TAI);
        let s = t_zero.to_str_lite("%.~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "");

        let s = t_zero.to_str_lite("%.9~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "");

        // Without ~ it should NOT trim
        let t_trailing = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 123_000_000_000_000_000, Scale::TAI);
        let s = t_trailing.to_str_lite("%.9f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123000000");

        let s = t_trailing.to_str_lite("%.9~f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123");

        let s = t.to_str_lite("%.0~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "");

        // Negative years + fractional trim
        let t_neg = Dt::from_ymd(-123, 6, 15, 9, 30, 45, 123_456_789_000_000_000, Scale::TAI);
        let s = t_neg.to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ").unwrap();
        assert_eq!(s.as_str().unwrap(), "-0123-06-15T09:30:45.123456789Z");

        let t_neg_zero = Dt::from_ymd(-1, 1, 1, 0, 0, 0, 0, Scale::TAI);
        let s = t_neg_zero.to_str_lite("%Y-%.~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "-0001-");

        let t_year0 = Dt::from_ymd(0, 1, 1, 0, 0, 0, 500_000_000_000_000_000, Scale::TAI);
        let s = t_year0.to_str_lite("%Y%.~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "0000.5");

        // Long years + fractional
        let t_long_year = Dt::from_ymd(123456, 7, 4, 12, 0, 0, 987654321987654321, Scale::TAI);
        let s = t_long_year.to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ").unwrap();
        assert_eq!(
            s.as_str().unwrap(),
            "123456-07-04T12:00:00.987654321987654321Z"
        );

        let t_long_neg_year =
            Dt::from_ymd(-100000, 12, 31, 23, 59, 59, 111111111111111111, Scale::TAI);
        let s = t_long_neg_year.to_str_lite("%Y-%.~f").unwrap();
        assert_eq!(s.as_str().unwrap(), "-100000-.111111111111111111");

        // 18-digit attos with NO trailing zeros
        let t_full_attos = Dt::from_ymd(2025, 4, 16, 0, 0, 0, 123456789012345678, Scale::TAI);

        let s = t_full_attos.to_str_lite("%.18~f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123456789012345678");

        let s = t_full_attos.to_str_lite("%.18f").unwrap();
        assert_eq!(s.as_str().unwrap(), ".123456789012345678");
    }
}
