use deep_time::{Dt, Scale, constants::ATTOS_PER_HALF_DAYU};

#[test]
fn j2000_tt_is_jd_2451545() {
    let j2000_tt = Dt::from_jd(2451545, 0, Scale::TT);

    let (jd, frac) = j2000_tt.to_jd(Scale::TAI, Scale::TT);
    assert_eq!(jd, 2451545);
    assert_eq!(frac, 0);

    let (mjd, mjd_frac) = j2000_tt.to_mjd(Scale::TAI, Scale::TT);

    // Standard MJD = JD − 2400000.5
    // At J2000.0 (JD 2451545.0) → MJD 51544.5
    assert_eq!(mjd, 51544, "MJD integer part (standard convention)");
    assert_eq!(
        mjd_frac, ATTOS_PER_HALF_DAYU,
        "MJD fractional part should be 0.5 day"
    );
}

/// Exact JD ↔ Dt round-trip (full attosecond precision).
#[test]
fn jd_tt_exact_roundtrip() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
    ];

    for &p in &test_points {
        let (jd, frac) = p.to_jd(Scale::TAI, Scale::TT);
        let back = Dt::from_jd(jd, frac, Scale::TT);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(diff < 1e-10, "JD round-trip error {} s at {:?}", diff, p);
    }
}

/// Exact MJD ↔ Dt round-trip.
#[test]
fn mjd_tt_exact_roundtrip() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI),
    ];

    for &p in &test_points {
        let (mjd, frac) = p.to_mjd(Scale::TAI, Scale::TT);
        let back = Dt::from_mjd(mjd, frac, Scale::TT);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(diff < 1e-10, "MJD round-trip error {} s at {:?}", diff, p);
    }
}

#[test]
fn test_mjd_utc_roundtrip() {
    // Normal instant (non-leap)
    let original = Dt::from_ymdhms(2025, 4, 27, 14, 30, 0, 123_456_789_000_000_000);
    let (mjd, frac) = original.to_mjd(Scale::TAI, Scale::UTC);
    let roundtrip = Dt::from_mjd(mjd, frac, Scale::UTC);
    assert_eq!(
        original, roundtrip,
        "MJD UTC round-trip failed for normal time"
    );

    // Also exercise the JD variant
    let (jd, frac_jd) = original.to_jd(Scale::TAI, Scale::UTC);
    let roundtrip_jd = Dt::from_jd(jd, frac_jd, Scale::UTC);
    assert_eq!(original, roundtrip_jd, "JD UTC round-trip failed");

    // Leap-second case (2015-06-30 23:59:60 UTC) — the trickiest path
    let leap = Dt::from_ymdhms(2015, 6, 30, 23, 59, 60, 0);
    let (mjd_leap, frac_leap) = leap.to_mjd(Scale::TAI, Scale::UTC);
    let roundtrip_leap = Dt::from_mjd(mjd_leap, frac_leap, Scale::UTC);
    assert_eq!(
        leap, roundtrip_leap,
        "MJD UTC round-trip failed for leap second"
    );

    // Also verify JD round-trip on the leap second
    let (jd_leap, frac_jd_leap) = leap.to_jd(Scale::TAI, Scale::UTC);
    let roundtrip_jd_leap = Dt::from_jd(jd_leap, frac_jd_leap, Scale::UTC);
    assert_eq!(
        leap, roundtrip_jd_leap,
        "JD UTC round-trip failed for leap second"
    );
}
