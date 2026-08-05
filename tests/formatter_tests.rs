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
        let s = leap.to_str_b("%Y-%m-%d %H:%M:%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2016-12-31 23:59:60.123456789000000000");

        // Trimmed fractional on leap second
        let s = leap.to_str_b("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2016-12-31T23:59:60.123456789Z");

        // leap second Unix timestamp (POSIX convention)
        let unix = leap.to_unix().to_sec64();
        assert_eq!(unix, 1483228799);
    }

    #[test]
    fn test_basic_formatting() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::TAI, 14, 30, 45, 123_456_789_000_000_000);

        let s = t.to_str_b("%Y-%m-%d %H:%M:%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45.123456789000000000");

        let s = t.to_str_b("%F", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-04-16");

        let s = t.to_str_b("%T", Lang::En).unwrap();
        assert_eq!(s.as_str(), "14:30:45");

        let s = t.to_str_b("%R", Lang::En).unwrap();
        assert_eq!(s.as_str(), "14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 0, 0, 0, 123_456_789_000_000_000);

        let s = t.to_str_b("%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456789000000000");

        let s = t.to_str_b("%N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456789000000000");

        let s = t.to_str_b("%.3f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123");

        let s = t.to_str_b("%.6N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "123456");
    }

    #[test]
    fn test_iso_week_fix() {
        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        let s = t2000.to_str_b("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = Dt::from_ymd(2000, 1, 3, Scale::UTC, 12, 0, 0, 0);
        let s = t2000_monday.to_str_b("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2000-W01-1");

        // Year with 53 weeks
        let t_week53 = Dt::from_ymd(2015, 12, 28, Scale::UTC, 12, 0, 0, 0);
        let s = t_week53.to_str_b("%G-W%V", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 14, 30, 45, 0);

        let s = t.to_str_b("%z", Lang::En).unwrap();
        assert_eq!(s.as_str(), "+0000");

        let s = t.to_str_b_in_offset("%:z", -5 * 3600, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-05:00");

        let s = t.to_str_b_in_offset("%::z", -8 * 3600, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-08:00:00");

        let s = t
            .to_str_b_in_offset("%z", 2 * 3600 + 30 * 60, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "+0230");

        let s = t.to_str_b("%Q", Lang::En).unwrap();
        assert_eq!(s.as_str(), "UTC");

        let s = t.to_str_b_in_offset("%z", -5 * 3600, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = Dt::from_ymd(2025, 4, 5, Scale::TAI, 3, 9, 7, 0);

        let s = t.to_str_b("%d %H %M %S", Lang::En).unwrap();
        assert_eq!(s.as_str(), "05 03 09 07");

        let s = t.to_str_b("%_d %_H", Lang::En).unwrap();
        assert_eq!(s.as_str(), " 5  3");

        let s = t.to_str_b("%-d %-H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "5 3");

        let s = t.to_str_b("%0d %0H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 0, 0, 0, 0); // Wednesday

        let s = t.to_str_b("%A, %B %d, %Y", Lang::En).unwrap();
        assert_eq!(s.as_str(), "Wednesday, April 16, 2025");

        let s = t.to_str_b("%a %b %d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "Wed Apr 16");
    }

    #[test]
    fn test_weekday_and_week_number_directives() {
        // 2023-12-31 was a Sunday
        let sun = Dt::from_ymd(2023, 12, 31, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(sun.to_str_b("%A", Lang::En).unwrap().as_str(), "Sunday");
        assert_eq!(sun.to_str_b("%a", Lang::En).unwrap().as_str(), "Sun");
        assert_eq!(sun.to_str_b("%w", Lang::En).unwrap().as_str(), "0");
        assert_eq!(sun.to_str_b("%u", Lang::En).unwrap().as_str(), "7");

        // 2024-01-01 was a Monday (ISO week 1 of 2024)
        let mon = Dt::from_ymd(2024, 1, 1, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(mon.to_str_b("%A", Lang::En).unwrap().as_str(), "Monday");
        assert_eq!(mon.to_str_b("%w", Lang::En).unwrap().as_str(), "1");
        assert_eq!(mon.to_str_b("%u", Lang::En).unwrap().as_str(), "1");
        assert_eq!(mon.to_str_b("%V", Lang::En).unwrap().as_str(), "01");
        assert_eq!(mon.to_str_b("%G", Lang::En).unwrap().as_str(), "2024");
        assert_eq!(mon.to_str_b("%g", Lang::En).unwrap().as_str(), "24"); // ← added

        // 2000-01-01 was a Saturday
        let sat = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(sat.to_str_b("%w", Lang::En).unwrap().as_str(), "6");
        assert_eq!(sat.to_str_b("%U", Lang::En).unwrap().as_str(), "00");
        assert_eq!(sat.to_str_b("%W", Lang::En).unwrap().as_str(), "00");

        // 2015-12-28 → ISO week 53 of 2015
        let w53 = Dt::from_ymd(2015, 12, 28, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(w53.to_str_b("%V", Lang::En).unwrap().as_str(), "53");
        assert_eq!(w53.to_str_b("%G", Lang::En).unwrap().as_str(), "2015");

        // 2024-12-30 → ISO week 1 of 2025
        let dec30 = Dt::from_ymd(2024, 12, 30, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(dec30.to_str_b("%V", Lang::En).unwrap().as_str(), "01");
        assert_eq!(dec30.to_str_b("%G", Lang::En).unwrap().as_str(), "2025");
        assert_eq!(dec30.to_str_b("%g", Lang::En).unwrap().as_str(), "25"); // ← added
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);

        let s = t.to_str_b("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0");

        let s = t.to_str_b("%j", Lang::En).unwrap();
        assert_eq!(s.as_str(), "001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        // Negative & zero years
        let t_neg = Dt::from_ymd(-123, 6, 15, Scale::TAI, 9, 30, 45, 0);
        let s = t_neg.to_str_b("%Y-%m-%d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0123-06-15");

        let s = t_neg.to_str_b("%C", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-1");

        let t_zero = Dt::from_ymd(0, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let s = t_zero.to_str_b("%Y", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0000");

        // ISO week year-boundary cases
        let t_2024_dec30 = Dt::from_ymd(2024, 12, 30, Scale::TAI, 12, 0, 0, 0);
        let s = t_2024_dec30.to_str_b("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-1");

        let t_2024_dec31 = Dt::from_ymd(2024, 12, 31, Scale::TAI, 12, 0, 0, 0);
        let s = t_2024_dec31.to_str_b("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-2");

        let t_2025_jan1 = Dt::from_ymd(2025, 1, 1, Scale::TAI, 12, 0, 0, 0);
        let s = t_2025_jan1.to_str_b("%G-W%V-%u", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2025-W01-3");

        let t_2015_dec28 = Dt::from_ymd(2015, 12, 28, Scale::TAI, 12, 0, 0, 0);
        let s = t_2015_dec28.to_str_b("%G-W%V", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2015-W53");

        // Week numbers %U / %W edge cases
        let t2000 = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
        let s = t2000.to_str_b("%U", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00");

        let s = t2000.to_str_b("%W", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00");

        let t_sun = Dt::from_ymd(2023, 12, 31, Scale::TAI, 12, 0, 0, 0);
        let s = t_sun.to_str_b("%U", Lang::En).unwrap();
        assert_eq!(s.as_str(), "53");

        // Fractional seconds extremes
        let t_frac = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 0);
        let s = t_frac.to_str_b("%.0f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        let s = t_frac.to_str_b("%.9N", Lang::En).unwrap();
        assert_eq!(s.as_str(), "000000000");

        let s = t_frac.to_str_b("%S.%f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "00.000000000000000000");

        // Timezone offsets with seconds & different colon counts
        let ny = -5 * 3600;
        let la = -8 * 3600;
        let weird = 3600 + 23 * 60 + 45;

        let s = t_frac.to_str_b_in_offset("%::z", ny, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-05:00:00");

        let s = t_frac.to_str_b_in_offset("%:z", la, Lang::En).unwrap();
        assert_eq!(s.as_str(), "-08:00");

        let s = t_frac.to_str_b_in_offset("%::z", weird, Lang::En).unwrap();
        assert_eq!(s.as_str(), "+01:23:45");

        // Padding + explicit width + flags combined
        let t_small = Dt::from_ymd(2025, 4, 5, Scale::TAI, 3, 9, 7, 0);

        let s = t_small.to_str_b("%03d", Lang::En).unwrap();
        assert_eq!(s.as_str(), "005");

        let s = t_small.to_str_b("%-5H", Lang::En).unwrap();
        assert_eq!(s.as_str(), "3");

        let s = t_small.to_str_b("%_3M", Lang::En).unwrap();
        assert_eq!(s.as_str(), "  9");

        // Negative Unix timestamp
        let t_neg_unix = Dt::from_ymd(1969, 12, 31, Scale::TAI, 23, 59, 59, 0);
        let s = t_neg_unix.to_str_b("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-1");

        // Large positive
        let t_large = Dt::from_ymd(2038, 1, 19, Scale::TAI, 3, 14, 7, 0);
        let s = t_large.to_str_b("%s", Lang::En).unwrap();
        assert_eq!(s.as_str(), "2147483647");
    }

    /// Full `Dt` range: year and both timestamps print (and parse back) for MAX/MIN.
    #[test]
    fn test_dt_extremes_year_and_timestamps() {
        use deep_time::StrPTimeFmt;
        use deep_time::civil_parts::Parts;

        // printer
        assert_eq!(
            Dt::MAX.to_str_b("%*", Lang::En).unwrap().as_str(),
            "5391559473918"
        );
        assert_eq!(
            Dt::MAX.to_str_b("%J", Lang::En).unwrap().as_str(),
            "170141183460469231731.687303715884105727"
        );
        // to_unix saturates at MAX, so %s matches %J here
        assert_eq!(
            Dt::MAX.to_str_b("%s", Lang::En).unwrap().as_str(),
            "170141183460469231731.687303715884105727"
        );

        assert_eq!(
            Dt::MIN.to_str_b("%*", Lang::En).unwrap().as_str(),
            "-5391559469919"
        );
        assert_eq!(
            Dt::MIN.to_str_b("%J", Lang::En).unwrap().as_str(),
            "-170141183460469231731.687303715884105728"
        );
        assert_eq!(
            Dt::MIN.to_str_b("%s", Lang::En).unwrap().as_str(),
            "-170141183459522503731.687303715884105728"
        );

        // strptime: %J + TAI (avoid default UTC leap conversion)
        let fmt_j = StrPTimeFmt::new("%J %L").unwrap();
        assert_eq!(
            fmt_j
                .to_dt(
                    "170141183460469231731.687303715884105727 TAI",
                    false,
                    false,
                    false
                )
                .unwrap()
                .to_attos(),
            Dt::MAX.to_attos()
        );
        assert_eq!(
            fmt_j
                .to_dt(
                    "-170141183460469231731.687303715884105728 TAI",
                    false,
                    false,
                    false
                )
                .unwrap()
                .to_attos(),
            Dt::MIN.to_attos()
        );

        // year-only parse needs allow_partial_date
        let p = Parts::from_strptime("%*", "5391559473918", false, false, true).unwrap();
        assert_eq!(p.yr, Some(5_391_559_473_918));
        let p = Parts::from_strptime("%*", "-5391559469919", false, false, true).unwrap();
        assert_eq!(p.yr, Some(-5_391_559_469_919));
    }

    #[test]
    fn test_fractional_trim_flag() {
        // Value with trailing zeros in fractional part
        let t = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123_456_789_000_000_000);

        let s = t.to_str_b("%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        let s = t.to_str_b("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        let s = t.to_str_b("%.18~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789");

        // Value that becomes all zeros after trimming
        let t_zero = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 0);
        let s = t_zero.to_str_b("%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        let s = t_zero.to_str_b("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        // Without ~ it should NOT trim
        let t_trailing = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123_000_000_000_000_000);
        let s = t_trailing.to_str_b("%.9f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123000000");

        let s = t_trailing.to_str_b("%.9~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123");

        let s = t.to_str_b("%.0~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "");

        // Negative years + fractional trim
        let t_neg = Dt::from_ymd(-123, 6, 15, Scale::TAI, 9, 30, 45, 123_456_789_000_000_000);
        let s = t_neg.to_str_b("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0123-06-15T09:30:45.123456789Z");

        let t_neg_zero = Dt::from_ymd(-1, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let s = t_neg_zero.to_str_b("%Y-%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-0001-");

        let t_year0 = Dt::from_ymd(0, 1, 1, Scale::TAI, 0, 0, 0, 500_000_000_000_000_000);
        let s = t_year0.to_str_b("%Y%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "0000.5");

        // Long years + fractional
        let t_long_year = Dt::from_ymd(123456, 7, 4, Scale::TAI, 12, 0, 0, 987654321987654321);
        let s = t_long_year
            .to_str_b("%Y-%m-%dT%H:%M:%S%.~fZ", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "123456-07-04T12:00:00.987654321987654321Z");

        let t_long_neg_year =
            Dt::from_ymd(-100000, 12, 31, Scale::TAI, 23, 59, 59, 111111111111111111);
        let s = t_long_neg_year.to_str_b("%Y-%.~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), "-100000-.111111111111111111");

        // 18-digit attos with NO trailing zeros
        let t_full_attos = Dt::from_ymd(2025, 4, 16, Scale::TAI, 0, 0, 0, 123456789012345678);

        let s = t_full_attos.to_str_b("%.18~f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789012345678");

        let s = t_full_attos.to_str_b("%.18f", Lang::En).unwrap();
        assert_eq!(s.as_str(), ".123456789012345678");
    }

    /// Without jiff-tz*: real IANA names → MissingFeature; UTC aliases format ok
    #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
    #[test]
    fn test_format_in_tz_without_jiff_tz() {
        use deep_time::DtErrKind;
        use deep_time::tz::UTC_ALIASES;

        let t = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);

        let err = t
            .to_str_b_in_tz("%Y-%m-%d %H:%M:%S %Z", "America/New_York", Lang::En)
            .unwrap_err();
        assert!(
            matches!(err.kind(), DtErrKind::MissingFeature),
            "expected MissingFeature, got {:?}",
            err.kind()
        );

        #[cfg(feature = "alloc")]
        {
            let err = t
                .to_str_in_tz("%Y-%m-%d %H:%M:%S %Z", "Europe/London", Lang::En)
                .unwrap_err();
            assert!(matches!(err.kind(), DtErrKind::MissingFeature));
        }

        for &alias in UTC_ALIASES {
            let s = t
                .to_str_b_in_tz("%Y-%m-%d %H:%M:%S %Z", alias, Lang::En)
                .unwrap_or_else(|e| panic!("UTC alias {alias:?} should format: {e}"));
            // UTC alias: no civil shift, abbrev is "UTC"
            assert!(
                s.as_str().starts_with("2000-01-01 12:00:00"),
                "alias {alias}: {s}"
            );
            assert!(
                s.as_str().ends_with(" UTC") || s.as_str().contains(" UTC"),
                "alias {alias}: expected UTC abbrev in {s}"
            );
        }
    }

    #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
    #[test]
    fn test_format_label_only_no_time_shift() {
        // Base time: 2025-04-16 14:30:45 UTC
        let t = Dt::from_ymd(2025, 4, 16, Scale::UTC, 14, 30, 45, 0);

        let s = t
            .to_str_b_with_offset_label("%Y-%m-%d %H:%M:%S %:z", -5 * 3600, Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45 -05:00");

        let s = t
            .to_str_b_with_tz_label("%Y-%m-%d %H:%M:%S %Z", "America/New_York", Lang::En)
            .unwrap();
        assert_eq!(s.as_str(), "2025-04-16 14:30:45 EDT");
    }

    #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
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

    #[test]
    fn test_display_precision() {
        use core::fmt::Write;
        use deep_time::BufStr;

        let dt = Dt::new(
            -1_500_000_000_000_000_000, // -1.5s
            Scale::TT,
            Scale::GPS,
        );
        assert_eq!(format!("{dt}"), "[-1.5s TT>GPS]");
        assert_eq!(format!("{dt:.0}"), "[-1s TT>GPS]"); // truncate, not round
        assert_eq!(format!("{dt:.1}"), "[-1.5s TT>GPS]");
        assert_eq!(format!("{dt:.3}"), "[-1.5s TT>GPS]"); // trims trailing zeros
        assert_eq!(format!("{dt:+.1}"), "[-1.5s TT>GPS]");

        let pos = Dt::new(1_500_000_000_000_000_000, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{pos:+.1}"), "[+1.5s TAI>TAI]");
        assert_eq!(format!("{pos:.0}"), "[1s TAI>TAI]");

        // Truncation keeps the kept prefix; never bumps the digit
        let almost = Dt::new(1_234_499_999_999_999_999, Scale::TAI, Scale::UTC);
        assert_eq!(format!("{almost:.3}"), "[1.234s TAI>UTC]");

        let half = Dt::new(1_234_500_000_000_000_000, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{half:.3}"), "[1.234s TAI>TAI]"); // not 1.235

        // Would have rounded up under half-up — truncate keeps 1.999…
        let nines = Dt::new(1_999_500_000_000_000_000, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{nines:.3}"), "[1.999s TAI>TAI]");

        // Full attosecond still works; precision > 18 clamps
        let one_atto = Dt::new(1, Scale::TDB, Scale::TDB);
        assert_eq!(format!("{one_atto}"), "[0.000000000000000001s TDB>TDB]");
        assert_eq!(format!("{one_atto:.18}"), "[0.000000000000000001s TDB>TDB]");
        assert_eq!(format!("{one_atto:.20}"), "[0.000000000000000001s TDB>TDB]");
        assert_eq!(format!("{one_atto:.0}"), "[0s TDB>TDB]");

        // no_std-friendly path via BufStr (same Display impl)
        let mut s = BufStr::<64>::default();
        write!(&mut s, "{:.2}", pos).unwrap();
        assert_eq!(s.as_str(), "[1.5s TAI>TAI]");
    }

    #[test]
    fn test_display_precision_edge_cases() {
        use deep_time::consts::ATTOS_PER_SEC_I128 as APS;

        // Zero / integer-only (no decimal even when precision requested)
        assert_eq!(format!("{:.0}", Dt::ZERO), "[0s TAI>TAI]");
        assert_eq!(format!("{:.9}", Dt::ZERO), "[0s TAI>TAI]");
        assert_eq!(format!("{:+.3}", Dt::ZERO), "[+0s TAI>TAI]");
        let whole = Dt::new(42 * APS, Scale::UTC, Scale::GPS);
        assert_eq!(format!("{whole:.5}"), "[42s UTC>GPS]");
        assert_eq!(format!("{whole:+.0}"), "[+42s UTC>GPS]");

        // Exact ±0.5s → {:.0} truncates to 0 whole seconds
        let half = Dt::new(APS / 2, Scale::TAI, Scale::TAI);
        let neg_half = Dt::new(-APS / 2, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{half:.0}"), "[0s TAI>TAI]");
        assert_eq!(format!("{neg_half:.0}"), "[0s TAI>TAI]"); // not signed zero
        assert_eq!(format!("{half:.1}"), "[0.5s TAI>TAI]");
        assert_eq!(format!("{neg_half:.1}"), "[-0.5s TAI>TAI]");

        let two_fifths = Dt::new(APS * 2 / 5, Scale::TAI, Scale::TAI); // 0.4s
        assert_eq!(format!("{two_fifths:.0}"), "[0s TAI>TAI]");
        assert_eq!(format!("{two_fifths:.1}"), "[0.4s TAI>TAI]");

        // ±1 attosecond
        let one_atto = Dt::new(1, Scale::TAI, Scale::TAI);
        let neg_atto = Dt::new(-1, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{one_atto:.17}"), "[0s TAI>TAI]"); // truncated away
        assert_eq!(format!("{one_atto:.18}"), "[0.000000000000000001s TAI>TAI]");
        assert_eq!(
            format!("{neg_atto:.18}"),
            "[-0.000000000000000001s TAI>TAI]"
        );
        assert_eq!(format!("{neg_atto:.0}"), "[0s TAI>TAI]");
        assert_eq!(format!("{neg_atto:.17}"), "[0s TAI>TAI]");
        assert_eq!(format!("{neg_atto:+.0}"), "[+0s TAI>TAI]");

        // 0.9995 → truncate, never carry into whole seconds
        let nines = Dt::new(9995 * 10i128.pow(14), Scale::TAI, Scale::TAI);
        assert_eq!(format!("{nines}"), "[0.9995s TAI>TAI]");
        assert_eq!(format!("{nines:.3}"), "[0.999s TAI>TAI]");
        assert_eq!(format!("{nines:.1}"), "[0.9s TAI>TAI]");
        assert_eq!(format!("{nines:.0}"), "[0s TAI>TAI]");
        assert_eq!(format!("{nines:.4}"), "[0.9995s TAI>TAI]");

        // Max sub-second fraction (1s − 1 atto)
        let almost_one = Dt::new(APS - 1, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{almost_one}"), "[0.999999999999999999s TAI>TAI]");
        assert_eq!(format!("{almost_one:.0}"), "[0s TAI>TAI]");
        assert_eq!(
            format!("{almost_one:.18}"),
            "[0.999999999999999999s TAI>TAI]"
        );
        assert_eq!(format!("{almost_one:.3}"), "[0.999s TAI>TAI]");

        let x = Dt::new(1_234_500_000_000_000_000, Scale::TAI, Scale::TAI);
        assert_eq!(format!("{x:.2}"), "[1.23s TAI>TAI]");
        assert_eq!(format!("{x:.3}"), "[1.234s TAI>TAI]");
        assert_eq!(format!("{x:.1}"), "[1.2s TAI>TAI]");
        assert_eq!(format!("{x:.0}"), "[1s TAI>TAI]");

        // Extremes: truncate whole seconds only; MIN uses wrapping_neg for abs
        assert!(format!("{}", Dt::MAX).starts_with('['));
        assert!(format!("{}", Dt::MIN).starts_with("[-"));
        assert_eq!(
            format!("{:.0}", Dt::MAX),
            "[170141183460469231731s TAI>TAI]"
        );
        assert_eq!(
            format!("{:.0}", Dt::MIN),
            "[-170141183460469231731s TAI>TAI]"
        );
        assert_eq!(
            format!("{:.3}", Dt::MAX),
            "[170141183460469231731.687s TAI>TAI]"
        );
        assert_eq!(
            format!("{:.3}", Dt::MIN),
            "[-170141183460469231731.687s TAI>TAI]"
        );
    }

    #[test]
    fn test_ymdhms_display_precision() {
        let ymd = Dt::from_ymd(2000, 1, 2, Scale::UTC, 3, 4, 5, 123_456_789_000_000_000).to_ymd();
        assert_eq!(format!("{ymd}"), "2000-01-02T03:04:05.123456789 UTC");
        assert_eq!(format!("{ymd:.3}"), "2000-01-02T03:04:05.123 UTC");
        assert_eq!(format!("{ymd:.0}"), "2000-01-02T03:04:05 UTC");
        assert_eq!(format!("{ymd:.20}"), "2000-01-02T03:04:05.123456789 UTC");

        // Truncate: must not advance the second
        let almost =
            Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 999_999_999_999_999_999).to_ymd();
        assert_eq!(format!("{almost:.3}"), "2000-01-01T12:00:00.999 TAI");
        assert_eq!(format!("{almost:.0}"), "2000-01-01T12:00:00 TAI");

        // Trailing zeros after truncate still trimmed
        let padded =
            Dt::from_ymd(2024, 3, 14, Scale::GPS, 15, 30, 45, 120_000_000_000_000_000).to_ymd();
        assert_eq!(format!("{padded:.6}"), "2024-03-14T15:30:45.12 GPS");
        assert_eq!(format!("{padded:.1}"), "2024-03-14T15:30:45.1 GPS");
    }

    #[test]
    fn test_dt_display_alternate() {
        let noon = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        assert_eq!(format!("{noon}"), "[32s TAI>UTC]");
        assert_eq!(format!("{noon:#}"), "2000-01-01T12:00:00 UTC");
        assert_eq!(format!("{noon:#}"), format!("{}", noon.to_ymd()));

        let frac = Dt::from_ymd(2024, 3, 14, Scale::UTC, 15, 30, 45, 123_456_789_000_000_000);
        assert_eq!(format!("{frac:#}"), "2024-03-14T15:30:45.123456789 UTC");
        assert_eq!(format!("{frac:#.3}"), "2024-03-14T15:30:45.123 UTC");
        assert_eq!(format!("{frac:#.0}"), "2024-03-14T15:30:45 UTC");
        assert_eq!(format!("{frac:#.3}"), format!("{:.3}", frac.to_ymd()));

        let day = Dt::new(
            86400 * deep_time::consts::ATTOS_PER_SEC_I128,
            Scale::TAI,
            Scale::UTC,
        );
        assert_eq!(format!("{day}"), "[86400s TAI>UTC]");
        assert_eq!(format!("{day:#}"), format!("{}", day.to_ymd()));

        assert_eq!(format!("{:#}", Dt::ZERO), "2000-01-01T12:00:00 TAI");
    }
}
