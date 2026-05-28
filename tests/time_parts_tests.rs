#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    use deep_time::constants::ATTOS_PER_SEC_I128;
    use deep_time::error::DtErrKind;
    use deep_time::{Dt, Scale, TimeParts, Weekday};

    /// Small helper for readable JD assertions (matches how the rest of the crate uses `to_jd_f()`).
    fn jd_tt(tp: &Dt) -> f64 {
        tp.to(Scale::TAI, Scale::TT).to_jd_f()
    }

    #[test]
    fn test_unix_epoch_1970() {
        let parsed = TimeParts::from_str("%s", "0", false, false, false).unwrap();
        let tp = parsed.to_dt().unwrap();

        let jd = jd_tt(&tp);
        // Unix epoch (1970-01-01 00:00:00 UTC) in TT scale:
        // 2440587.5 + 32.184 / 86400 = 2440587.5003725 exactly.
        assert!(
            (jd - 2440587.5003725).abs() == 0.0,
            "Expected ~2440587.5003725 (Unix epoch in TT), got {}",
            jd
        );
    }

    #[test]
    fn test_j2000_noon_via_unix_timestamp() {
        let parsed = TimeParts::from_str("%s", "946728000", false, false, false).unwrap();
        let tp = parsed.to_dt().unwrap();

        let jd = jd_tt(&tp);
        // J2000.0 = JD 2451545.0 in TT. Tiny deviation expected due to leap seconds + TAI→TT.
        assert!(
            (jd - 2451545.0).abs() < 0.01,
            "Expected ~2451545.0, got {}",
            jd
        );
    }

    #[test]
    fn test_ymd_and_ordinal_produce_identical_time_point() {
        // YMD and ordinal (%j) paths both set `.year` and produce the exact same instant.
        let ymd = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap()
        .to_dt()
        .unwrap();

        let ordinal = TimeParts::from_str(
            "%Y-%j %H:%M:%S.%.f",
            "2024-106 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap()
        .to_dt()
        .unwrap();

        assert_eq!(jd_tt(&ymd), jd_tt(&ordinal));
        assert_eq!(ymd.to_jd(), ordinal.to_jd());
    }

    #[test]
    fn test_fractional_seconds_are_preserved() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
            false,
            false,
        )
        .unwrap();
        let tp = parsed.to_dt().unwrap();

        let expected = 123_456_789u64 * 1_000_000_000;
        assert_eq!(
            tp.to_sec_ufrac(),
            expected,
            "fractional seconds were not preserved"
        );
    }

    #[test]
    fn test_jd_tt_fractional_seconds_preserved() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
            false,
            false,
        )
        .unwrap();

        let tp = parsed.to_dt().unwrap();
        let (_, frac_attos) = tp.to(Scale::UTC, Scale::TT).to_jd();

        // Convert attoseconds → seconds
        let seconds_past_noon = (frac_attos as f64) / (ATTOS_PER_SEC_I128 as f64);

        const EXPECTED: f64 = 43269.307456789;

        assert!(
            (seconds_past_noon - EXPECTED).abs() < 1e-9,
            "JD TT fractional seconds not preserved.\n\
         Expected ~{EXPECTED} s past noon (TT), got {seconds_past_noon}"
        );
    }

    #[test]
    fn test_incomplete_date_error() {
        // Default TimeParts has no year → early failure in to_time_point.
        let pd = TimeParts::default();
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::Incomplete));
    }

    #[test]
    fn test_day_of_year_out_of_range_non_leap_year() {
        // 2023 is not a leap year. We build a TimeParts manually because the parser
        // rejects day 366 (u8 limit in parse_u8_padded), so we never reach to_time_point
        // with a parser-constructed value. This test directly exercises the leap-year check.
        let mut pd = TimeParts::default();
        pd.yr = Some(2023);
        pd.day_of_yr = Some(366);
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::OutOfRange));
    }

    #[test]
    fn test_iso_week_out_of_range() {
        // Parser rejects week 54, so we build manually to hit the to_time_point check.
        let mut pd = TimeParts::default();
        pd.iso_wk_yr = Some(2024);
        pd.iso_wk = Some(54);
        pd.wkday = Some(Weekday::Monday); // required for the ISO path
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::OutOfRange));
    }

    #[test]
    fn test_pure_iso_week_date() {
        // Pure ISO week date (%G/%V/%u) is now fully supported in to_time_point
        // via the iso_week_year + iso_week + weekday path (no regular .year required).
        let parsed = TimeParts::from_str("%G-W%V-%u", "2024-W16-1", false, false, false).unwrap();
        let tp_iso = parsed.to_dt().unwrap();

        // 2024-W16-1 is Monday, April 15, 2024
        let ymd = TimeParts::from_str("%Y-%m-%d", "2024-04-15", false, false, false)
            .unwrap()
            .to_dt()
            .unwrap();

        assert_eq!(jd_tt(&tp_iso), jd_tt(&ymd));
        assert_eq!(tp_iso.to_jd(), ymd.to_jd());
    }
}
