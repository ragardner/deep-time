#[cfg(test)]
mod tests {
    use deep_time::{DtErrKind, Offset, TimeParts, Weekday};

    #[test]
    fn test_basic_ymd_hms() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 14:30:45",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.hour, Some(14));
        assert_eq!(parsed.minute, Some(30));
        assert_eq!(parsed.second, Some(45));
        assert_eq!(parsed.attos, Some(0));
        assert_eq!(parsed.offset, Some(Offset::Utc));
    }

    #[test]
    fn test_unix_timestamp_direct() {
        let parsed = TimeParts::from_str("%s", "1713191445", false, false, false).unwrap();
        assert_eq!(parsed.unix_timestamp_seconds, Some(1713191445));
    }

    #[test]
    fn test_fractional_seconds_various_widths() {
        // Explicit literal dot + %.f (the parser's optional-dot logic works reliably this way)
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap();
        let expected = 123_456_789u64 * 10u64.pow(9);
        assert_eq!(parsed.attos, Some(expected));

        let parsed2 = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%3N",
            "2024-04-15 14:30:45.123",
            false,
            false,
            false,
        )
        .unwrap();
        let expected2 = 123u64 * 10u64.pow(15);
        assert_eq!(parsed2.attos, Some(expected2));
    }

    #[test]
    fn test_leap_second_flag() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 23:59:60",
            false,
            false,
            false,
        )
        .unwrap();
        assert!(parsed.is_leap_second);
        assert_eq!(parsed.second, Some(60));
    }

    #[test]
    fn test_iana_name_parsing() {
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        assert!(parsed.iana_name.is_some());
        let name = parsed.iana_name.unwrap();
        let name_str = name.as_str().expect("IANA name should be valid ASCII");
        assert_eq!(name_str, "America/New_York");
        assert_eq!(parsed.offset, Some(Offset::None));
    }

    #[test]
    fn test_fixed_offset_parsing() {
        // Space before %z is required by the current parser (no literal character between %T and %z otherwise)
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 10:30:00 -0400", false, false, false)
                .unwrap();
        assert_eq!(parsed.offset, Some(Offset::Fixed(-14400)));
    }

    #[test]
    fn test_fixed_offset_with_colons() {
        let parsed = TimeParts::from_str(
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
            TimeParts::from_str("%F %T", "2024-04-15 14:30:45", false, false, false).unwrap();
        assert_eq!(parsed_f.year, Some(2024));
        assert_eq!(parsed_f.month, Some(4));
        assert_eq!(parsed_f.day, Some(15));
        assert_eq!(parsed_f.hour, Some(14));
        assert_eq!(parsed_f.minute, Some(30));
        assert_eq!(parsed_f.second, Some(45));

        let parsed_d = TimeParts::from_str("%D", "04/15/24", false, false, false).unwrap();
        assert_eq!(parsed_d.year, Some(2024));
        assert_eq!(parsed_d.month, Some(4));
        assert_eq!(parsed_d.day, Some(15));
    }

    #[test]
    fn test_month_and_weekday_names() {
        let parsed = TimeParts::from_str(
            "%B %d, %Y (%A)",
            "April 15, 2024 (Monday)",
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.weekday, Some(Weekday::Monday));
    }

    #[test]
    fn test_strict_mode_trailing_chars() {
        let err =
            TimeParts::from_str("%Y-%m-%d", "2024-04-15 extra", false, false, false).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::TrailingCharacters));
    }

    #[test]
    fn test_incomplete_date_error() {
        let err = TimeParts::from_str("%H:%M:%S", "14:30:45", false, false, false).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::Incomplete));
    }

    #[test]
    fn test_ordinal_date() {
        let parsed = TimeParts::from_str("%Y-%j", "2024-106", false, false, false).unwrap();
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.day_of_year, Some(106));
    }

    #[test]
    fn test_iso_week_date() {
        let parsed = TimeParts::from_str("%G-W%V-%u", "2024-W16-2", false, false, false).unwrap();
        assert_eq!(parsed.iso_week_year, Some(2024));
        assert_eq!(parsed.iso_week, Some(16));
        assert_eq!(parsed.weekday, Some(Weekday::Tuesday));
    }
}
