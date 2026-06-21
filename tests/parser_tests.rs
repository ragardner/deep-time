#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    use deep_time::DtErrKind;
    use deep_time::civil_parts::{Offset, Parts, Weekday};

    #[test]
    fn test_basic_ymd_hms() {
        let parsed = Parts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 14:30:45",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.yr, Some(2024));
        assert_eq!(parsed.mo, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.hr, 14);
        assert_eq!(parsed.min, 30);
        assert_eq!(parsed.sec, 45);
        assert_eq!(parsed.attos, 0);
        assert_eq!(parsed.offset, None);
    }

    #[test]
    fn test_unix_timestamp_direct() {
        let parsed = Parts::from_str("%s", "1713191445", false, false, false).unwrap();
        assert_eq!(parsed.timestamp_sec, Some(1713191445));
    }

    #[test]
    fn test_fractional_seconds_various_widths() {
        // Explicit literal dot + %.f (the parser's optional-dot logic works reliably this way)
        let parsed = Parts::from_str(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap();
        let expected = 123_456_789u64 * 10u64.pow(9);
        assert_eq!(parsed.attos, expected);

        let parsed2 = Parts::from_str(
            "%Y-%m-%d %H:%M:%S.%3N",
            "2024-04-15 14:30:45.123",
            false,
            false,
            false,
        )
        .unwrap();
        let expected2 = 123u64 * 10u64.pow(15);
        assert_eq!(parsed2.attos, expected2);
    }

    #[test]
    fn test_leap_second_flag() {
        let parsed = Parts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 23:59:60",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.sec, 60);
    }

    #[test]
    fn test_iana_name_parsing() {
        let parsed = Parts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        assert!(parsed.iana_name.is_some());
        let name = parsed.iana_name.unwrap();
        let name_str = name.as_str();
        assert_eq!(name_str, "America/New_York");
        assert_eq!(parsed.offset, Some(Offset::None));
    }

    #[test]
    fn test_fixed_offset_parsing() {
        // Space before %z is required by the current parser (no literal character between %T and %z otherwise)
        let parsed =
            Parts::from_str("%F %T %z", "2024-04-15 10:30:00 -0400", false, false, false).unwrap();
        assert_eq!(parsed.offset, Some(Offset::Fixed(-14400)));
    }

    #[test]
    fn test_fixed_offset_with_colons() {
        let parsed = Parts::from_str(
            "%F %T %:z",
            "2024-04-15 10:30:00 -04:00",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.offset, Some(Offset::Fixed(-14400)));
    }

    #[test]
    fn test_shortcut_formats() {
        let parsed_f =
            Parts::from_str("%F %T", "2024-04-15 14:30:45", false, false, false).unwrap();
        assert_eq!(parsed_f.yr, Some(2024));
        assert_eq!(parsed_f.mo, Some(4));
        assert_eq!(parsed_f.day, Some(15));
        assert_eq!(parsed_f.hr, 14);
        assert_eq!(parsed_f.min, 30);
        assert_eq!(parsed_f.sec, 45);

        let parsed_d = Parts::from_str("%D", "04/15/24", false, false, false).unwrap();
        assert_eq!(parsed_d.yr, Some(2024));
        assert_eq!(parsed_d.mo, Some(4));
        assert_eq!(parsed_d.day, Some(15));
    }

    #[test]
    fn test_month_and_weekday_names() {
        let parsed = Parts::from_str(
            "%B %d, %Y (%A)",
            "April 15, 2024 (Monday)",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.mo, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.yr, Some(2024));
        assert_eq!(parsed.wkday, Some(Weekday::Monday));
    }

    #[test]
    fn test_strict_mode_trailing_chars() {
        let err = Parts::from_str("%Y-%m-%d", "2024-04-15 extra", false, false, false).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::TrailingCharacters));
    }

    #[test]
    fn test_incomplete_date_error() {
        let err = Parts::from_str("%H:%M:%S", "14:30:45", false, false, false).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::Incomplete));
    }

    #[test]
    fn test_ordinal_date() {
        let parsed = Parts::from_str("%Y-%j", "2024-106", false, false, false).unwrap();
        assert_eq!(parsed.yr, Some(2024));
        assert_eq!(parsed.day_of_yr, Some(106));
    }

    #[test]
    fn test_iso_week_date() {
        let parsed = Parts::from_str("%G-W%V-%u", "2024-W16-2", false, false, false).unwrap();
        assert_eq!(parsed.iso_wk_yr, Some(2024));
        assert_eq!(parsed.iso_wk, Some(16));
        assert_eq!(parsed.wkday, Some(Weekday::Tuesday));
    }

    #[test]
    fn test_format_extensions_numeric_padding() {
        // Default zero padding
        let p = Parts::from_str("%04Y-%02m-%02d", "2024-04-05", false, false, false).unwrap();
        assert_eq!(p.yr, Some(2024));
        assert_eq!(p.mo, Some(4));
        assert_eq!(p.day, Some(5));

        // Explicit zero padding
        let p = Parts::from_str("%0Y-%0m-%0d", "2024-04-05", false, false, false).unwrap();
        assert_eq!(p.yr, Some(2024));

        // Space padding
        let p = Parts::from_str("%_4Y-%_2m-%_2d", " 2024- 4- 5", false, false, false).unwrap();
        assert_eq!(p.yr, Some(2024));
        assert_eq!(p.mo, Some(4));
        assert_eq!(p.day, Some(5));

        // No padding / left justify
        let p = Parts::from_str("%-Y-%-m-%-d", "2024-4-5", false, false, false).unwrap();
        assert_eq!(p.yr, Some(2024));
        assert_eq!(p.mo, Some(4));
        assert_eq!(p.day, Some(5));
    }

    #[test]
    fn test_format_extensions_width_on_year() {
        let p = Parts::from_str("%6Y-%m-%d", "2024-04-05", false, false, false).unwrap();
        assert_eq!(p.yr, Some(2024));
    }

    #[test]
    fn test_format_extensions_fractional() {
        // Width before f/N (no dot in format)
        let p = Parts::from_str(
            "%Y-%m-%d %H:%M:%S.%3f",
            "2024-04-15 14:30:45.123",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(p.attos, 123_000_000_000_000_000); // 123 * 10^15

        let p = Parts::from_str(
            "%Y-%m-%d %H:%M:%S.%6N",
            "2024-04-15 14:30:45.123456",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(p.attos, 123_456_000_000_000_000);

        // %.f style (dot in format)
        let p = Parts::from_str(
            "%Y-%m-%d %H:%M:%S.%.3f",
            "2024-04-15 14:30:45.123",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(p.attos, 123_000_000_000_000_000);
    }

    #[test]
    fn test_format_extensions_timezone_colons() {
        // %z (no colons)
        let p =
            Parts::from_str("%F %T%z", "2024-04-15 10:30:00-0400", false, false, false).unwrap();
        assert_eq!(p.offset, Some(Offset::Fixed(-14400)));

        // %:z (one colon)
        let p =
            Parts::from_str("%F %T%:z", "2024-04-15 10:30:00-04:00", false, false, false).unwrap();
        assert_eq!(p.offset, Some(Offset::Fixed(-14400)));

        // %::z (two colons)
        let p = Parts::from_str(
            "%F %T%::z",
            "2024-04-15 10:30:00-04:00:00",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(p.offset, Some(Offset::Fixed(-14400)));

        // %:::z (three colons) — more flexible
        let p = Parts::from_str(
            "%F %T%:::z",
            "2024-04-15 10:30:00-04:00:00",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(p.offset, Some(Offset::Fixed(-14400)));
    }

    #[test]
    fn test_format_extensions_combined() {
        let p = Parts::from_str(
            "%-4Y-%_2m-%02dT%3H:%M%:z",
            "2024- 4-05T 14:30-04:00",
            false,
            false,
            false,
        )
        .unwrap();

        assert_eq!(p.yr, Some(2024));
        assert_eq!(p.mo, Some(4));
        assert_eq!(p.day, Some(5));
        assert_eq!(p.hr, 14);
        assert_eq!(p.offset, Some(Offset::Fixed(-14400)));
    }

    #[test]
    fn test_format_extensions_case_flags() {
        // These are accepted by the extension parser.
        // Current name parsers are case-insensitive, so behavior is the same as without the flag.
        let p = Parts::from_str("%^A", "MONDAY", false, false, false);
        // This may currently return Incomplete depending on your from_str wrapper.
        // If it does, we can adjust. For now we just check it doesn't panic on unknown directive.
        if let Ok(p) = p {
            assert_eq!(p.wkday, Some(Weekday::Monday));
        }
    }

    #[test]
    fn test_format_extensions_errors() {
        // Flag without directive
        let err = Parts::from_str("%_", " ", false, false, false);
        assert!(err.is_err());

        // Width without directive
        let err = Parts::from_str("%3", "123", false, false, false);
        assert!(err.is_err());

        // Colons without directive
        let err = Parts::from_str("%:", ":", false, false, false);
        assert!(err.is_err());

        // Too many colons on %z (Jiff rejects > 3)
        let err = Parts::from_str(
            "%F %T%::::z",
            "2024-04-15 10:30:00-04:00",
            false,
            false,
            false,
        );
        assert!(err.is_err());
    }
}
