#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod format_tests {
    use deep_time::{Dt, Lang, Scale};

    #[test]
    fn test_leap_second_gotcha_2016_12_31() {
        let leap = Dt::from_ymd(
            2016,
            12,
            31,
            Scale::UTC,
            23,
            59,
            60,
            123_456_789_000_000_000,
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

        // Formatting must output "60"
        let s = leap.to_str_lite("%Y-%m-%d %H:%M:%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2016-12-31 23:59:60.123456789000000000");

        // Trimmed fractional on leap second
        let s = leap
            .to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "2016-12-31T23:59:60.123456789Z");

        // leap second Unix timestamp (POSIX convention)
        let unix = leap.to_unix().to_sec64();
        assert_eq!(unix, 1483228799);
    }

    #[test]
    fn test_basic_formatting() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::TAI, 14, 30, 45, 123_456_789_000_000_000);

        let s = t.to_str_lite("%Y-%m-%d %H:%M:%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45.123456789000000000");

        let s = t.to_str_lite("%F", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-04-16");

        let s = t.to_str_lite("%T", Lang::En).unwrap();
        assert_eq!(s.as_str(), "14:30:45");

        let s = t.to_str_lite("%R", Lang::En).unwrap();
        assert_eq!(s.as_str(), "14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 0, 0, 0, 123_456_789_000_000_000);

        let s = t.to_str_lite("%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456789000000000");

        let s = t.to_str_lite("%N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456789000000000");

        let s = t.to_str_lite("%.3f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123");

        let s = t.to_str_lite("%.6N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456");
    }

    #[test]
    fn test_iso_week_fix() {
        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        let s = t2000.to_str_lite("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = Dt::from_ymd(2000, 1, 3, Scale::UTC, 12, 0, 0, 0);
        let s = t2000_monday.to_str_lite("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2000-W01-1");

        // Year with 53 weeks
        let t_week53 = Dt::from_ymd(2015, 12, 28, Scale::UTC, 12, 0, 0, 0);
        let s = t_week53.to_str_lite("%G-W%V", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 14, 30, 45, 0);

        let s = t.to_str_lite("%z", Lang::En).unwrap();
        assert_eq!(s.as_str(), "+0000");

        let s = t.to_str_lite_in_offset("%:z", -5 * 3600, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-05:00");

        let s = t
            .to_str_lite_in_offset("%::z", -8 * 3600, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "-08:00:00");

        let s = t
            .to_str_lite_in_offset("%z", 2 * 3600 + 30 * 60, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "+0230");

        let s = t.to_str_lite("%Q", Lang::En).unwrap();
        assert_eq!(s.as_str(), "UTC");

        let s = t.to_str_lite_in_offset("%z", -5 * 3600, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = Dt::from_ymd(2025, 4, 5, Scale::TAI, 3, 9, 7, 0);

        let s = t.to_str_lite("%d %H %M %S", Lang::En).unwrap();
        assert_eq!(s.as_str(), "05 03 09 07");

        let s = t.to_str_lite("%_d %_H", Lang::En).unwrap();
        assert_eq!(s.as_str(), " 5  3");

        let s = t.to_str_lite("%-d %-H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "5 3");

        let s = t.to_str_lite("%0d %0H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 0, 0, 0, 0); // Wednesday

        let s = t.to_str_lite("%A, %B %d, %Y", Lang::En).unwrap();
        assert_eq!(s.as_str(), "Wednesday, April 16, 2025");

        let s = t.to_str_lite("%a %b %d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "Wed Apr 16");
    }

    #[test]
    fn test_weekday_and_week_number_directives() {
        // 2023-12-31 was a Sunday
        let sun = Dt::from_ymd(2023, 12, 31, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(sun.to_str_lite("%A", Lang::En).unwrap().as_str(), "Sunday");
        assert_eq!(sun.to_str_lite("%a", Lang::En).unwrap().as_str(), "Sun");
        assert_eq!(sun.to_str_lite("%w", Lang::En).unwrap().as_str(), "0");
        assert_eq!(sun.to_str_lite("%u", Lang::En).unwrap().as_str(), "7");

        // 2024-01-01 was a Monday (ISO week 1 of 2024)
        let mon = Dt::from_ymd(2024, 1, 1, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(mon.to_str_lite("%A", Lang::En).unwrap().as_str(), "Monday");
        assert_eq!(mon.to_str_lite("%w", Lang::En).unwrap().as_str(), "1");
        assert_eq!(mon.to_str_lite("%u", Lang::En).unwrap().as_str(), "1");
        assert_eq!(mon.to_str_lite("%V", Lang::En).unwrap().as_str(), "01");
        assert_eq!(mon.to_str_lite("%G", Lang::En).unwrap().as_str(), "2024");
        assert_eq!(mon.to_str_lite("%g", Lang::En).unwrap().as_str(), "24"); // ← added

        // 2000-01-01 was a Saturday
        let sat = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(sat.to_str_lite("%w", Lang::En).unwrap().as_str(), "6");
        assert_eq!(sat.to_str_lite("%U", Lang::En).unwrap().as_str(), "00");
        assert_eq!(sat.to_str_lite("%W", Lang::En).unwrap().as_str(), "00");

        // 2015-12-28 → ISO week 53 of 2015
        let w53 = Dt::from_ymd(2015, 12, 28, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(w53.to_str_lite("%V", Lang::En).unwrap().as_str(), "53");
        assert_eq!(w53.to_str_lite("%G", Lang::En).unwrap().as_str(), "2015");

        // 2024-12-30 → ISO week 1 of 2025
        let dec30 = Dt::from_ymd(2024, 12, 30, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(dec30.to_str_lite("%V", Lang::En).unwrap().as_str(), "01");
        assert_eq!(dec30.to_str_lite("%G", Lang::En).unwrap().as_str(), "2025");
        assert_eq!(dec30.to_str_lite("%g", Lang::En).unwrap().as_str(), "25"); // ← added
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);

        let s = t.to_str_lite("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0");

        let s = t.to_str_lite("%j", Lang::En).unwrap();
        assert_eq!(s.as_str(), "001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        // Negative & zero years
        let t_neg = Dt::from_ymd(-123, 6, 15, Scale::TAI, 9, 30, 45, 0);
        let s = t_neg.to_str_lite("%Y-%m-%d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0123-06-15");

        let s = t_neg.to_str_lite("%C", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-2");

        let t_zero = Dt::from_ymd(0, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let s = t_zero.to_str_lite("%Y", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0000");

        // ISO week year-boundary cases
        let t_2024_dec30 = Dt::from_ymd(2024, 12, 30, Scale::TAI, 12, 0, 0, 0);
        let s = t_2024_dec30.to_str_lite("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-1");

        let t_2024_dec31 = Dt::from_ymd(2024, 12, 31, Scale::TAI, 12, 0, 0, 0);
        let s = t_2024_dec31.to_str_lite("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-2");

        let t_2025_jan1 = Dt::from_ymd(2025, 1, 1, Scale::TAI, 12, 0, 0, 0);
        let s = t_2025_jan1.to_str_lite("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-3");

        let t_2015_dec28 = Dt::from_ymd(2015, 12, 28, Scale::TAI, 12, 0, 0, 0);
        let s = t_2015_dec28.to_str_lite("%G-W%V", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2015-W53");

        // Week numbers %U / %W edge cases
        let t2000 = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
        let s = t2000.to_str_lite("%U", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00");

        let s = t2000.to_str_lite("%W", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00");

        let t_sun = Dt::from_ymd(2023, 12, 31, Scale::TAI, 12, 0, 0, 0);
        let s = t_sun.to_str_lite("%U", Lang::En).unwrap();
        assert_eq!(s.as_str(), "53");

        // Fractional seconds extremes
        let t_frac = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 0);
        let s = t_frac.to_str_lite("%.0f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        let s = t_frac.to_str_lite("%.9N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "000000000");

        let s = t_frac.to_str_lite("%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00.000000000000000000");

        // Timezone offsets with seconds & different colon counts
        let ny = -5 * 3600;
        let la = -8 * 3600;
        let weird = 3600 + 23 * 60 + 45;

        let s = t_frac.to_str_lite_in_offset("%::z", ny, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-05:00:00");

        let s = t_frac.to_str_lite_in_offset("%:z", la, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-08:00");

        let s = t_frac
            .to_str_lite_in_offset("%::z", weird, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "+01:23:45");

        // Padding + explicit width + flags combined
        let t_small = Dt::from_ymd(2025, 4, 5, Scale::TAI, 3, 9, 7, 0);

        let s = t_small.to_str_lite("%03d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "005");

        let s = t_small.to_str_lite("%-5H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "3");

        let s = t_small.to_str_lite("%_3M", Lang::En).unwrap();
        assert_eq!(s.as_str(), "  9");

        // Negative Unix timestamp
        let t_neg_unix = Dt::from_ymd(1969, 12, 31, Scale::TAI, 23, 59, 59, 0);
        let s = t_neg_unix.to_str_lite("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-1");

        // Large positive
        let t_large = Dt::from_ymd(2038, 1, 19, Scale::TAI, 3, 14, 7, 0);
        let s = t_large.to_str_lite("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2147483647");
    }

    #[test]
    fn test_fractional_trim_flag() {
        // Value with trailing zeros in fractional part
        let t = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123_456_789_000_000_000);

        let s = t.to_str_lite("%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        let s = t.to_str_lite("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        let s = t.to_str_lite("%.18~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        // Value that becomes all zeros after trimming
        let t_zero = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 0);
        let s = t_zero.to_str_lite("%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        let s = t_zero.to_str_lite("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        // Without ~ it should NOT trim
        let t_trailing = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123_000_000_000_000_000);
        let s = t_trailing.to_str_lite("%.9f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123000000");

        let s = t_trailing.to_str_lite("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123");

        let s = t.to_str_lite("%.0~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        // Negative years + fractional trim
        let t_neg = Dt::from_ymd(-123, 6, 15, Scale::TAI, 9, 30, 45, 123_456_789_000_000_000);
        let s = t_neg
            .to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "-0123-06-15T09:30:45.123456789Z");

        let t_neg_zero = Dt::from_ymd(-1, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let s = t_neg_zero.to_str_lite("%Y-%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0001-");

        let t_year0 = Dt::from_ymd(0, 1, 1, Scale::TAI, 0, 0, 0, 500_000_000_000_000_000);
        let s = t_year0.to_str_lite("%Y%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0000.5");

        // Long years + fractional
        let t_long_year = Dt::from_ymd(123456, 7, 4, Scale::TAI, 12, 0, 0, 987654321987654321);
        let s = t_long_year
            .to_str_lite("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "123456-07-04T12:00:00.987654321987654321Z");

        let t_long_neg_year =
            Dt::from_ymd(-100000, 12, 31, Scale::TAI, 23, 59, 59, 111111111111111111);
        let s = t_long_neg_year.to_str_lite("%Y-%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-100000-.111111111111111111");

        // 18-digit attos with NO trailing zeros
        let t_full_attos = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123456789012345678);

        let s = t_full_attos.to_str_lite("%.18~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789012345678");

        let s = t_full_attos.to_str_lite("%.18f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789012345678");
    }

    #[cfg(feature = "jiff-tz")]
    #[test]
    fn test_format_label_only_no_time_shift() {
        // Base time: 2025-04-16 14:30:45 UTC
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 14, 30, 45, 0);

        let s = t
            .to_str_lite_with_offset_label("%Y-%m-%d %H:%M:%S %:z", -5 * 3600, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45 -05:00");

        let s = t
            .to_str_lite_with_tz_label("%Y-%m-%d %H:%M:%S %Z", "America/New_York", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45 EDT");
    }

    #[cfg(all(feature = "alloc", feature = "jiff-tz"))]
    #[test]
    fn test_format_label_only_no_time_shift_alloc() {
        // Base time: 2025-04-16 14:30:45 UTC
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 14, 30, 45, 0);

        let s = t
            .to_str_with_offset_label("%Y-%m-%d %H:%M:%S %:z", -5 * 3600, Lang::En)
            .unwrap();
        assert_eq!(s, "2025-04-16 14:30:45 -05:00");

        let s = t
            .to_str_with_tz_label("%Y-%m-%d %H:%M:%S %Z", "America/New_York", Lang::En)
            .unwrap();
        assert_eq!(s, "2025-04-16 14:30:45 EDT");
    }
}
