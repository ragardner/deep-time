#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use deep_time::{DtErr, TimeParts};
    use jiff::{SignedDuration, Timestamp};

    fn parse_ts(fmt: &str, input: &str, strict: bool) -> Result<Timestamp, DtErr> {
        let parsed = TimeParts::from_str(fmt, input, strict, false, false)?;
        parsed.to_jiff_timestamp()
    }

    #[test]
    fn test_basic_ymd_hms_utc() {
        let ts = parse_ts("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191445).unwrap());
    }

    #[test]
    fn test_unix_timestamp_direct() {
        let ts = parse_ts("%s", "1713191445", false).unwrap();
        assert_eq!(ts.as_second(), 1713191445);
    }

    #[test]
    fn test_fixed_offset() {
        let ts = parse_ts("%F %T%z", "2024-04-15 10:30:00-0400", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_fixed_offset_with_colons() {
        let ts = parse_ts("%F %T%:z", "2024-04-15 10:30:00-04:00", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_iana_timezone() {
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        assert!(parsed.iana_name.is_some());
        let ts = parsed.to_jiff_timestamp().unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_fractional_seconds_various_widths() {
        let base = Timestamp::from_second(1713191445).unwrap();

        let ts = parse_ts(
            "%Y-%m-%d %H:%M:%S%.f",
            "2024-04-15 14:30:45.123456789",
            false,
        )
        .unwrap();
        assert_eq!(
            ts,
            base.checked_add(SignedDuration::from_nanos(123_456_789))
                .unwrap()
        );

        let ts2 = parse_ts("%Y-%m-%d %H:%M:%S%3N", "2024-04-15 14:30:45.123", false).unwrap();
        assert_eq!(
            ts2,
            base.checked_add(SignedDuration::from_millis(123)).unwrap()
        );
    }

    #[test]
    fn test_ordinal_date_fallback_path() {
        let ts = parse_ts("%Y-%j %H:%M:%S", "2024-106 14:30:45", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191445).unwrap());
    }

    #[test]
    fn test_iso_week_date_fallback_path() {
        let ts = parse_ts("%G-W%V-%u %H:%M:%S", "2024-W16-2 14:30:45", false).unwrap();
        // 2024-W16-2 is Tuesday April 16 (not April 15)
        assert_eq!(ts, Timestamp::from_second(1713277845).unwrap());
    }

    #[test]
    fn test_shortcut_formats() {
        let ts_f = parse_ts("%F %T", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(ts_f, Timestamp::from_second(1713191445).unwrap());

        let ts_d = parse_ts("%D", "04/15/24", false).unwrap();
        // %D is date-only → defaults to 00:00:00 UTC
        assert_eq!(ts_d, Timestamp::from_second(1713139200).unwrap());
    }

    #[test]
    fn test_broken_down_time_assembly() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S %z",
            "2024-04-15 14:30:45 +0200",
            false,
            false,
            false,
        )
        .unwrap();
        let bdt = parsed.to_jiff_broken_down_time().unwrap();

        assert_eq!(bdt.year(), Some(2024));
        assert_eq!(bdt.month(), Some(4));
        assert_eq!(bdt.day(), Some(15));
        assert_eq!(bdt.hour(), Some(14));
        assert_eq!(bdt.minute(), Some(30));
        assert_eq!(bdt.second(), Some(45));
        assert_eq!(
            bdt.offset(),
            Some(jiff::tz::Offset::from_seconds(7200).unwrap())
        );
    }
}
