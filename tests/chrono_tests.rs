#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "chrono")]
mod tests {
    use chrono::{
        DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as ChronoTimeZone,
    };
    use deep_time::TimeParts;

    #[test]
    fn test_to_chrono_naive_datetime_basic_ymd_hms() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 14:30:45",
            false,
            false,
            false,
        )
        .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_ordinal_date() {
        let parsed =
            TimeParts::from_str("%Y-%j %H:%M:%S", "2024-106 14:30:45", false, false, false)
                .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_yo_opt(2024, 106).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_iso_week_date() {
        use chrono::Weekday as ChronoWeekday;

        let parsed = TimeParts::from_str(
            "%G-W%V-%u %H:%M:%S",
            "2024-W16-2 14:30:45",
            false,
            false,
            false,
        )
        .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // 2024-W16-2 = Tuesday 2024-04-16
        let expected_date = NaiveDate::from_isoywd_opt(2024, 16, ChronoWeekday::Tue).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_fractional_seconds() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%N",
            "2024-04-15 14:30:45.123456789012345678901234567890",
            false,
            false,
            false,
        )
        .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_nano_opt(14, 30, 45, 123_456_789).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_leap_second() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 23:59:60",
            false,
            false,
            false,
        )
        .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // Chrono represents leap second as 23:59:59 + 1_000_000_000 ns
        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_nano_opt(23, 59, 59, 1_000_000_000).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_datetime_fixed_offset() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 14:30:45 -0400", false, false, false)
                .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
        );
        let offset = FixedOffset::west_opt(4 * 3600).unwrap();
        let expected = offset
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap();

        assert_eq!(dt, expected);
        assert_eq!(dt.offset(), &offset);
    }

    #[test]
    fn test_to_chrono_datetime_colon_z_offset() {
        let parsed = TimeParts::from_str(
            "%F %T %:z",
            "2024-04-15 14:30:45 -04:00",
            false,
            false,
            false,
        )
        .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
        );
        let offset = FixedOffset::west_opt(4 * 3600).unwrap();
        let expected = offset
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap();

        assert_eq!(dt, expected);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_direct() {
        let parsed = TimeParts::from_str("%s", "1713191445", false, false, false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        // 1713191445 = 2024-04-15 14:30:45 UTC
        let expected_utc = DateTime::from_timestamp(1713191445, 0).unwrap();
        let offset = FixedOffset::east_opt(0).unwrap();
        let expected = expected_utc.with_timezone(&offset);

        assert_eq!(dt, expected);
        assert_eq!(dt.timestamp(), 1713191445);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_with_fraction() {
        let parsed =
            TimeParts::from_str("%s.%N", "1713191445.123456789", false, false, false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_utc = DateTime::from_timestamp(1713191445, 123_456_789).unwrap();
        let offset = FixedOffset::east_opt(0).unwrap();
        let expected = expected_utc.with_timezone(&offset);

        assert_eq!(dt, expected);
    }

    #[test]
    fn test_to_chrono_timestamp_basic() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S",
            "2024-04-15 14:30:45",
            false,
            false,
            false,
        )
        .unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_unix_direct() {
        let parsed = TimeParts::from_str("%s", "1713191445", false, false, false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_with_offset() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 10:30:45 -0400", false, false, false)
                .unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        // 10:30:45 EDT = 14:30:45 UTC → same as above
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_naive_datetime_incomplete_date_fails_in_finish_but_assembly_fails_here() {
        // Parser already rejects incomplete date in finish(), but we test the assembly path too
        let parsed = TimeParts::from_str("%H:%M:%S", "14:30:45", false, false, false);
        assert!(parsed.is_err()); // finish() already fails with IncompleteDate
    }

    #[test]
    fn test_to_chrono_datetime_utc_explicit() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 14:30:45 +0000", false, false, false)
                .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected = DateTime::from_timestamp(1713191445, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        assert_eq!(dt, expected);
    }

    //

    #[test]
    fn test_to_chrono_datetime_civil_with_fixed_positive_offset() {
        // 2024-04-15 14:30:45 +05:00  → local time in +5 zone
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 14:30:45 +0500", false, false, false)
                .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
        );
        let offset = FixedOffset::east_opt(5 * 3600).unwrap();
        let expected = offset
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap();

        assert_eq!(dt, expected);
        assert_eq!(dt.offset(), &offset);
        assert_eq!(dt.timestamp(), 1713173445); // ← corrected: 09:30:45 UTC
    }

    #[cfg(feature = "chrono-tz")]
    #[test]
    fn test_to_chrono_datetime_civil_with_iana_america_new_york_edt() {
        // 2024-04-15 10:30:00 America/New_York  (EDT = UTC-4)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap());
        assert_eq!(dt.timestamp(), 1713191400); // 14:30:00 UTC
    }

    // #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_civil_with_iana_fallback_offset_at() {
        // Same as above, but using your tzdb::offset_at fallback
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap());
        assert_eq!(dt.timestamp(), 1713191400);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_ignores_iana_name() {
        // %s + IANA name → must still be pure UTC (+0000)
        let parsed =
            TimeParts::from_str("%s %Q", "1713191400 America/New_York", false, false, false)
                .unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::east_opt(0).unwrap());
        assert_eq!(dt.timestamp(), 1713191400);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_ignores_fixed_offset() {
        // %s + %z → must still be pure UTC (+0000)
        let parsed = TimeParts::from_str("%s %z", "1713191400 -0400", false, false, false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::east_opt(0).unwrap());
        assert_eq!(dt.timestamp(), 1713191400);
    }

    #[cfg(feature = "chrono-tz")]
    #[test]
    fn test_to_chrono_timestamp_with_iana_name() {
        // Civil time in IANA zone → correct UTC unix timestamp
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191400); // 14:30 UTC
    }

    #[cfg(feature = "chrono-tz")]
    #[test]
    fn test_to_chrono_datetime_iana_ambiguous_time_fails() {
        // 2024-11-03 01:30:00 America/New_York is ambiguous (fall-back)
        // Should error (same as chrono-tz behaviour)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-11-03 01:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        let err = parsed.to_chrono_datetime().unwrap_err();
        assert!(matches!(
            err.kind().unwrap(),
            deep_time::error::DtErrKind::InvalidTimezoneOffset
        ));
    }

    #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_iana_spring_forward_gap() {
        // 2023-03-12 02:30:00 America/New_York is inside the DST spring-forward gap (non-existent)
        // Our code must shift it forward and succeed with the post-gap offset (EDT = -4h)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2023-03-12 02:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();

        let dt = parsed.to_chrono_datetime().unwrap();

        // After shift: becomes 03:30:00 EDT → 07:30:00 UTC
        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap());
        assert_eq!(dt.timestamp(), 1678606200); // 2023-03-12 07:30:00 UTC
    }

    #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_iana_exact_spring_forward_boundary() {
        // Exact transition moment: 2023-03-12 02:00:00 America/New_York (start of gap)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2023-03-12 02:00:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();

        let dt = parsed.to_chrono_datetime().unwrap();

        // Starts the gap → shifts to 03:00:00 EDT → 07:00:00 UTC
        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap());
        assert_eq!(dt.timestamp(), 1678604400);
    }

    #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_iana_fall_back_overlap() {
        // 2023-11-05 01:00:00 America/New_York is ambiguous (fall-back overlap)
        // We follow Jiff behavior: pick earlier occurrence (still on EDT, -04:00)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2023-11-05 01:00:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();

        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap()); // EDT (earlier occurrence)
        assert_eq!(dt.timestamp(), 1699160400); // 2023-11-05 05:00:00 UTC
    }

    #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_iana_exact_fall_back_boundary() {
        // Exact transition moment: 2023-11-05 01:00:00 America/New_York (overlap boundary)
        // We follow Jiff behavior: pick earlier occurrence (EDT, -04:00)
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2023-11-05 01:00:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();

        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::west_opt(4 * 3600).unwrap()); // EDT (earlier occurrence)
        assert_eq!(dt.timestamp(), 1699160400); // 2023-11-05 05:00:00 UTC
    }

    #[cfg(not(feature = "chrono-tz"))]
    #[test]
    fn test_to_chrono_datetime_iana_southern_hemisphere_gap() {
        // Southern hemisphere spring-forward gap (Australia/Sydney)
        // 02:30 is in the gap → shifts to 03:30 AEDT
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-10-06 02:30:00 Australia/Sydney",
            false,
            false,
            false,
        )
        .unwrap();

        let dt = parsed.to_chrono_datetime().unwrap();

        assert_eq!(dt.offset(), &FixedOffset::east_opt(11 * 3600).unwrap()); // AEDT
        assert_eq!(dt.timestamp(), 1728145800); // 2024-10-05 16:30:00 UTC (correct shifted time)
    }
}
