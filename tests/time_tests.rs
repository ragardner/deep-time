#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "time")]
mod tests {
    use deep_time::{Dt, Scale};
    use time::{Duration, OffsetDateTime, Timestamp, UtcDateTime, UtcOffset};

    fn assert_ns_eq(a: Dt, b: Dt, label: &str) {
        assert_eq!(
            a.to_ns().0,
            b.to_ns().0,
            "{label}: ns mismatch (attos {} vs {})",
            a.attos,
            b.attos
        );
    }

    #[test]
    fn timestamp_roundtrip_unix_epoch() {
        let dt = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = dt.to_time_timestamp();
        assert_eq!(ts.as_seconds(), 0);
        assert_eq!(ts.as_nanoseconds(), 0);
        assert_ns_eq(Dt::from_time_timestamp(ts), dt, "unix epoch");
    }

    #[test]
    fn timestamp_roundtrip_j2000_utc_noon() {
        let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        let ts = dt.to_time_timestamp();
        assert_eq!(ts.as_seconds(), 946_728_000);
        assert_ns_eq(Dt::from_time_timestamp(ts), dt, "j2000 utc noon");
    }

    #[test]
    fn timestamp_roundtrip_modern_with_subsec() {
        // 123_456_789 ns → 123_456_789_000_000_000 attos
        let dt = Dt::from_ymd(2024, 4, 15, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);
        let ts = dt.to_time_timestamp();
        assert_eq!(ts.as_seconds(), 1_713_191_445);
        assert_eq!(ts.as_nanoseconds(), 1_713_191_445_123_456_789);
        assert_ns_eq(Dt::from_time_timestamp(ts), dt, "2024 with ns");
    }

    #[test]
    fn timestamp_roundtrip_near_leap_seconds() {
        // Just after the 2016-12-31 leap second
        let dt = Dt::from_ymd(2017, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = dt.to_time_timestamp();
        assert_eq!(ts.as_seconds(), 1_483_228_800);
        assert_ns_eq(Dt::from_time_timestamp(ts), dt, "2017-01-01 UTC");
    }

    #[test]
    fn timestamp_from_tai_uses_utc_unix() {
        // Same civil UTC instant should map to the same Unix timestamp whether
        // the Dt was built as UTC or as the equivalent TAI instant.
        let utc = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let tai = utc.to_tai();
        assert_eq!(
            utc.to_time_timestamp().as_nanoseconds(),
            tai.to_time_timestamp().as_nanoseconds()
        );
    }

    #[test]
    fn timestamp_is_posix_not_tai() {
        // At 2020-01-01, TAI-UTC is 37 s. A mistaken TAI Unix conversion would
        // shift the second count by that amount.
        let utc = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = utc.to_time_timestamp();
        assert_eq!(ts.as_seconds(), 1_577_836_800);

        // Reconstructing from that Unix second and reading civil UTC fields
        // must yield 2020-01-01, not 2019-12-31 23:59:23.
        let back = Dt::from_time_timestamp(ts);
        let ymd = back.target(Scale::UTC).to_ymd();
        assert_eq!(ymd.yr(), 2020);
        assert_eq!(ymd.mo(), 1);
        assert_eq!(ymd.day(), 1);
        assert_eq!(ymd.hr(), 0);
        assert_eq!(ymd.min(), 0);
        assert_eq!(ymd.sec(), 0);
    }

    #[test]
    fn offset_datetime_utc_roundtrip() {
        let dt = Dt::from_ymd(2019, 6, 15, Scale::UTC, 12, 0, 0, 0);
        let odt = dt.to_time_offset_datetime_utc();
        assert_eq!(odt.offset(), UtcOffset::UTC);
        assert_eq!(odt.unix_timestamp(), 1_560_600_000);
        assert_ns_eq(Dt::from_time_offset_datetime(odt), dt, "offset utc");
    }

    #[test]
    fn offset_datetime_non_utc_offset_uses_instant() {
        // 2024-04-15 10:30:00-04:00 == 2024-04-15 14:30:00 UTC
        let odt = OffsetDateTime::from_unix_timestamp(1_713_191_400)
            .unwrap()
            .to_offset(UtcOffset::from_hms(-4, 0, 0).unwrap());
        assert_eq!(odt.hour(), 10);
        assert_eq!(odt.minute(), 30);

        let dt = Dt::from_time_offset_datetime(odt);
        let ymd = dt.target(Scale::UTC).to_ymd();
        assert_eq!(ymd.yr(), 2024);
        assert_eq!(ymd.mo(), 4);
        assert_eq!(ymd.day(), 15);
        assert_eq!(ymd.hr(), 14);
        assert_eq!(ymd.min(), 30);
        assert_eq!(ymd.sec(), 0);
    }

    #[test]
    fn utc_datetime_roundtrip() {
        let dt = Dt::from_ymd(1999, 12, 31, Scale::UTC, 23, 59, 59, 0);
        let udt = dt.to_time_utc_datetime();
        assert_eq!(udt.unix_timestamp(), 946_684_799);
        assert_ns_eq(Dt::from_time_utc_datetime(udt), dt, "utc datetime");
    }

    #[test]
    fn utc_datetime_from_timestamp_path() {
        let ts = Timestamp::from_seconds(1_000_000_000).unwrap();
        let udt: UtcDateTime = ts.to_utc();
        let dt = Dt::from_time_utc_datetime(udt);
        assert_eq!(dt.to_time_timestamp().as_seconds(), 1_000_000_000);
    }

    #[test]
    fn duration_roundtrip() {
        let span = Dt::from_ns(3_600_000_000_000 + 123, 0, Scale::TAI, Scale::TAI); // 1 hour + 123 ns
        let dur = span.to_time_duration();
        assert_eq!(dur.whole_seconds(), 3_600);
        assert_eq!(dur.subsec_nanoseconds(), 123);
        assert_ns_eq(Dt::from_time_duration(dur), span, "duration");
    }

    #[test]
    fn duration_negative_roundtrip() {
        let span = Dt::from_ns(-5_000_000_001, 0, Scale::TAI, Scale::TAI);
        let dur = span.to_time_duration();
        assert!(dur.is_negative());
        assert_eq!(dur.whole_nanoseconds(), -5_000_000_001);
        assert_ns_eq(Dt::from_time_duration(dur), span, "neg duration");
    }

    #[test]
    fn duration_truncates_sub_nanosecond() {
        // 1.5 ns worth of attoseconds → truncates toward zero to 1 ns
        let span = Dt::from_attos(1_500_000_000, Scale::TAI);
        let dur = span.to_time_duration();
        assert_eq!(dur.whole_nanoseconds(), 1);
    }

    #[test]
    fn duration_saturates_at_extremes() {
        // Far beyond Duration's i64-second range
        let huge = Dt::from_attos(i128::MAX / 2, Scale::TAI);
        assert_eq!(huge.to_time_duration(), Duration::MAX);

        let tiny = Dt::from_attos(i128::MIN / 2, Scale::TAI);
        assert_eq!(tiny.to_time_duration(), Duration::MIN);
    }

    #[test]
    fn timestamp_saturates_out_of_range() {
        // Far future / past relative to time crate's default ±9999 year range
        let far_future = Dt::from_ymd(50_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(far_future.to_time_timestamp(), Timestamp::MAX);

        let far_past = Dt::from_ymd(-50_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(far_past.to_time_timestamp(), Timestamp::MIN);
    }
}
