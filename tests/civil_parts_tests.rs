#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    use deep_time::civil_parts::{Offset, Parts, Weekday};
    use deep_time::consts::ATTOS_PER_SEC_I128;
    use deep_time::{Dt, DtErrKind, Scale};

    /// Small helper for readable JD assertions (matches how the rest of the crate uses `to_jd_f()`).
    fn jd_tt(tp: &Dt) -> f64 {
        tp.to(Scale::TT).to_jd_f_raw()
    }

    #[test]
    fn test_unix_epoch_1970() {
        let parsed = Parts::from_strptime("%s", "0", false, false, false).unwrap();
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
        let parsed = Parts::from_strptime("%s", "946728000", false, false, false).unwrap();
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
        let ymd = Parts::from_strptime(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap()
        .to_dt()
        .unwrap();

        let ordinal = Parts::from_strptime(
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
        let parsed = Parts::from_strptime(
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
        let parsed = Parts::from_strptime(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
            false,
            false,
        )
        .unwrap();

        let tp = parsed.to_dt().unwrap();
        let (_, frac_attos) = tp.target(Scale::TT).to_jd();

        // Convert attoseconds → seconds
        let seconds_past_noon = frac_attos as f64 / ATTOS_PER_SEC_I128 as f64;

        const EXPECTED: f64 = 43269.307456789;

        assert!(
            (seconds_past_noon - EXPECTED).abs() < 1e-9,
            "JD TT fractional seconds not preserved.\n\
         Expected ~{EXPECTED} s past noon (TT), got {seconds_past_noon}"
        );
    }

    #[test]
    fn test_incomplete_date_error() {
        // Default Parts has no year → early failure in to_time_point.
        let pd = Parts::default();
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind(), DtErrKind::ExpectedYear));
    }

    #[test]
    fn test_day_of_year_out_of_range_non_leap_year() {
        // 2023 is not a leap year. We build a Parts manually because the parser
        // rejects day 366 (u8 limit in parse_u8_padded), so we never reach to_time_point
        // with a parser-constructed value. This test directly exercises the leap-year check.
        let mut pd = Parts::default();
        pd.yr = Some(2023);
        pd.day_of_yr = Some(366);
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind(), DtErrKind::DayOfYearOutOfRange));
    }

    #[test]
    fn test_iso_week_out_of_range() {
        // Parser rejects week 54, so we build manually to hit the to_time_point check.
        let mut pd = Parts::default();
        pd.iso_wk_yr = Some(2024);
        pd.iso_wk = Some(54);
        pd.wkday = Some(Weekday::Monday); // required for the ISO path
        let err = pd.to_dt().unwrap_err();
        assert!(matches!(err.kind(), DtErrKind::IsoWeekOutOfRange));
    }

    #[test]
    fn test_pure_iso_week_date() {
        // Pure ISO week date (%G/%V/%u) is now fully supported in to_time_point
        // via the iso_week_year + iso_week + weekday path (no regular .year required).
        let parsed = Parts::from_strptime("%G-W%V-%u", "2024-W16-1", false, false, false).unwrap();
        let tp_iso = parsed.to_dt().unwrap();

        // 2024-W16-1 is Monday, April 15, 2024
        let ymd = Parts::from_strptime("%Y-%m-%d", "2024-04-15", false, false, false)
            .unwrap()
            .to_dt()
            .unwrap();

        assert_eq!(jd_tt(&tp_iso), jd_tt(&ymd));
        assert_eq!(tp_iso.to_jd(), ymd.to_jd());
    }

    /// Civil `to_dt` must copy `Parts::target` (not force it to equal `scale`).
    #[test]
    fn test_civil_to_dt_preserves_distinct_target() {
        let parts = Parts {
            yr: Some(2024),
            mo: Some(6),
            day: Some(20),
            hr: 12,
            scale: Scale::UTC,
            target: Scale::GPS,
            ..Parts::default()
        };
        let dt = parts.to_dt().unwrap();
        assert_eq!(dt.scale, Scale::TAI);
        assert_eq!(dt.target, Scale::GPS);
    }

    /// Civil `to_dt` without IANA covers the full range of `Dt`
    #[test]
    fn test_civil_to_dt_full_i128_range() {
        // Round-trip extremes via to_ymd civil fields (no zone)
        for attos in [i128::MAX, i128::MIN, i128::MAX - 1, i128::MIN + 1] {
            let dt = Dt::new(attos, Scale::TAI, Scale::TAI);
            let ymd = dt.to_ymd();
            let parts = Parts {
                yr: Some(ymd.yr()),
                mo: Some(ymd.mo()),
                day: Some(ymd.day()),
                hr: ymd.hr(),
                min: ymd.min(),
                sec: ymd.sec(),
                attos: ymd.attos(),
                scale: Scale::TAI,
                target: Scale::TAI,
                ..Parts::default()
            };
            let back = parts.to_dt().unwrap();
            assert_eq!(back.to_attos(), attos, "civil to_dt lost attos for {attos}");
        }

        // larger than i64 seconds - must agree with from_ymd
        let past = Dt::from_sec((i64::MAX as i128) + 1, Scale::TAI, Scale::TAI);
        let y = past.to_ymd();
        let parts = Parts {
            yr: Some(y.yr()),
            mo: Some(y.mo()),
            day: Some(y.day()),
            hr: y.hr(),
            min: y.min(),
            sec: y.sec(),
            attos: y.attos(),
            scale: Scale::TAI,
            target: Scale::TAI,
            ..Parts::default()
        };
        assert_eq!(parts.to_dt().unwrap(), past);

        // fixed offset (no jiff)
        let parts = Parts {
            yr: Some(2000),
            mo: Some(1),
            day: Some(1),
            hr: 12,
            offset: Some(Offset::Fixed(3600)),
            scale: Scale::TAI,
            target: Scale::TAI,
            ..Parts::default()
        };
        // local 12:00+01:00 → 11:00 library noon = -3600 s
        assert_eq!(parts.to_dt().unwrap().to_sec(), -3600);
    }

    /// Without jiff-tz*: real IANA names → MissingFeature; UTC aliases still work
    #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
    #[test]
    fn test_to_dt_iana_without_jiff_tz() {
        use deep_time::tz::UTC_ALIASES;

        let base = || Parts {
            yr: Some(2000),
            mo: Some(1),
            day: Some(1),
            hr: 12,
            scale: Scale::TAI,
            target: Scale::TAI,
            ..Parts::default()
        };

        let mut real = base();
        real.set_iana_name(Some("America/New_York"));
        let err = real.to_dt().unwrap_err();
        assert!(
            matches!(err.kind(), DtErrKind::MissingFeature),
            "expected MissingFeature, got {:?}",
            err.kind()
        );

        let without = base().to_dt().unwrap();
        for &alias in UTC_ALIASES {
            let mut p = base();
            p.set_iana_name(Some(alias));
            let dt = p.to_dt().unwrap_or_else(|e| {
                panic!("UTC alias {alias:?} should succeed without jiff-tz: {e}")
            });
            // Alias is treated as UTC; no local shift relative to no zone
            assert_eq!(dt, without, "alias {alias}");
        }
    }
}
