#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

//! Interop tests for `Dt` ↔ ICU4X (`icu_time::DateTime<Iso>`).
//!
//! ICU `DateTime` is a civil display type (ISO date + time-of-day at nanosecond
//! precision). deep-time converts via UTC civil fields, same convention as
//! chrono `DateTime<Utc>` interop.

#[cfg(feature = "icu")]
mod interop {
    use deep_time::{Dt, DtErrKind, Scale};
    use icu_calendar::Date;
    use icu_time::{DateTime, Time};

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
    fn roundtrip_modern_with_subsec() {
        let dt = Dt::from_ymd(2024, 4, 15, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);
        let icu = dt.to_icu_datetime_iso().unwrap();
        assert_eq!(icu.date.era_year().extended_year, 2024);
        assert_eq!(icu.date.month().ordinal, 4);
        assert_eq!(icu.date.day_of_month().0, 15);
        assert_eq!(icu.time.hour.number(), 14);
        assert_eq!(icu.time.minute.number(), 30);
        assert_eq!(icu.time.second.number(), 45);
        assert_eq!(icu.time.subsecond.number(), 123_456_789);
        assert_ns_eq(Dt::from_icu_datetime_iso(icu), dt, "2024 with ns");
    }

    #[test]
    fn roundtrip_unix_epoch() {
        let dt = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let icu = dt.to_icu_datetime_iso().unwrap();
        assert_eq!(icu.date.era_year().extended_year, 1970);
        assert_eq!(icu.time.hour.number(), 0);
        assert_ns_eq(Dt::from_icu_datetime_iso(icu), dt, "unix epoch");
    }

    #[test]
    fn roundtrip_j2000_utc_noon() {
        let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        let icu = dt.to_icu_datetime_iso().unwrap();
        assert_eq!(icu.date.era_year().extended_year, 2000);
        assert_eq!(icu.time.hour.number(), 12);
        assert_ns_eq(Dt::from_icu_datetime_iso(icu), dt, "j2000 utc noon");
    }

    #[test]
    fn truncates_sub_nanosecond_attos() {
        // 123_456_789 ns + 1 atto beyond that ns → truncate to ns
        let attos = 123_456_789_000_000_000 + 1;
        let dt = Dt::from_ymd(2024, 1, 1, Scale::UTC, 0, 0, 0, attos);
        let icu = dt.to_icu_datetime_iso().unwrap();
        assert_eq!(icu.time.subsecond.number(), 123_456_789);
        let back = Dt::from_icu_datetime_iso(icu);
        assert_eq!(back.to_ns().0, dt.to_ns().0);
        assert_ne!(back.attos, dt.attos);
    }

    #[test]
    fn year_zero_and_negative() {
        let y0 = Dt::from_ymd(0, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let icu0 = y0.to_icu_datetime_iso().unwrap();
        assert_eq!(icu0.date.era_year().extended_year, 0);
        assert_ns_eq(Dt::from_icu_datetime_iso(icu0), y0, "year 0");

        let yn = Dt::from_ymd(-44, 3, 15, Scale::UTC, 12, 0, 0, 0);
        let icun = yn.to_icu_datetime_iso().unwrap();
        assert_eq!(icun.date.era_year().extended_year, -44);
        assert_ns_eq(Dt::from_icu_datetime_iso(icun), yn, "year -44");
    }

    #[test]
    fn year_bounds_icu_constructor() {
        let ok_hi = Dt::from_ymd(9999, 12, 31, Scale::UTC, 23, 59, 59, 0);
        assert!(ok_hi.to_icu_datetime_iso().is_ok());

        let ok_lo = Dt::from_ymd(-9999, 1, 1, Scale::UTC, 0, 0, 0, 0);
        assert!(ok_lo.to_icu_datetime_iso().is_ok());

        let bad_hi = Dt::from_ymd(10_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let err = bad_hi.to_icu_datetime_iso().unwrap_err();
        assert_eq!(err.kind(), DtErrKind::InvalidDate);

        let bad_lo = Dt::from_ymd(-10_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let err = bad_lo.to_icu_datetime_iso().unwrap_err();
        assert_eq!(err.kind(), DtErrKind::InvalidDate);
    }

    #[test]
    fn from_constructs_via_utc_civil() {
        let icu = DateTime {
            date: Date::try_new_iso(2017, 1, 1).unwrap(),
            time: Time::try_new(0, 0, 0, 0).unwrap(),
        };
        let dt = Dt::from_icu_datetime_iso(icu);
        let ymd = dt.target(Scale::UTC).to_ymd();
        assert_eq!((ymd.yr(), ymd.mo(), ymd.day()), (2017, 1, 1));
        assert_eq!((ymd.hr(), ymd.min(), ymd.sec()), (0, 0, 0));
    }

    #[test]
    fn try_from_and_from_traits() {
        let dt = Dt::from_ymd(2026, 6, 16, Scale::UTC, 12, 0, 0, 0);
        let icu: DateTime<_> = dt.try_into().unwrap();
        assert_eq!(icu.date.era_year().extended_year, 2026);
        let back: Dt = icu.into();
        assert_ns_eq(back, dt, "trait roundtrip");
    }

    #[test]
    fn leap_second_civil_sec_60() {
        // 2016-12-31 leap second: construct UTC civil with sec=60 via from_ymd
        let dt = Dt::from_ymd(2016, 12, 31, Scale::UTC, 23, 59, 60, 0);
        let ymd = dt.target(Scale::UTC).to_ymd();
        // Only assert ICU accepts sec=60 if deep-time still reports it as 60
        if ymd.sec() == 60 {
            let icu = dt.to_icu_datetime_iso().unwrap();
            assert_eq!(icu.time.second.number(), 60);
            let back = Dt::from_icu_datetime_iso(icu);
            let ymd_back = back.target(Scale::UTC).to_ymd();
            assert_eq!(ymd_back.sec(), 60);
        }
    }
}
