#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::macros::days_f;
use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY};

#[test]
fn mjd_truncating_split_from_mjd_f() {
    let cases = [
        (100.75, 100_i128, days_f!(0.75)),
        (-100.75, -100, -days_f!(0.75)),
        (60_961.25, 60_961, days_f!(0.25)),
        (-1_000.25, -1_000, -days_f!(0.25)),
    ];

    for (mjd_f, whole, frac) in cases {
        let dt = Dt::from_mjd_f(mjd_f, Scale::TAI);
        assert_eq!(dt.to_mjd_f(), mjd_f);
        assert_eq!(dt.to_mjd(), (whole, frac), "to_mjd failed for {mjd_f}");
        assert_eq!(
            dt.to_mjd_raw(),
            (whole, frac),
            "to_mjd_raw failed for {mjd_f}"
        );
    }
}

#[test]
fn mjd_floor_split_from_mjd_f() {
    let cases = [
        (100.75, 100_i128, days_f!(0.75)),
        (-100.75, -101, days_f!(0.25)),
        (60_961.25, 60_961, days_f!(0.25)),
        (-1_000.25, -1_001, days_f!(0.75)),
    ];

    for (mjd_f, whole, frac) in cases {
        let dt = Dt::from_mjd_f(mjd_f, Scale::TAI);
        assert_eq!(dt.to_mjd_f(), mjd_f);
        assert_eq!(
            dt.to_mjd_floor(),
            (whole, frac),
            "to_mjd_floor failed for {mjd_f}"
        );
        assert_eq!(
            dt.to_mjd_floor_raw(),
            (whole, frac),
            "to_mjd_floor_raw failed for {mjd_f}"
        );

        let (days, frac_attos) = dt.to_mjd_floor();
        let back = Dt::from_mjd(days, frac_attos, Scale::TAI);
        assert_eq!(back, dt, "floor round-trip failed for {mjd_f}");
    }
}

#[test]
fn j2000_tt_is_jd_2451545() {
    let j2000_tt = Dt::from_jd(2451545, 0, Scale::TAI);

    let (jd, frac) = j2000_tt.to_jd();
    assert_eq!(jd, 2451545);
    assert_eq!(frac, 0);

    let (mjd, mjd_frac) = j2000_tt.to_mjd();

    // Standard MJD = JD − 2400000.5
    // At J2000.0 (JD 2451545.0) → MJD 51544.5
    assert_eq!(mjd, 51544, "MJD integer part (standard convention)");
    assert_eq!(
        mjd_frac, ATTOS_PER_HALF_DAY,
        "MJD fractional part should be 0.5 day"
    );
}

/// Exact JD ↔ Dt round-trip (full attosecond precision).
#[test]
fn jd_tt_exact_roundtrip() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
    ];

    for &p in &test_points {
        let (jd, frac) = p.target(Scale::TT).to_jd();
        let back = Dt::from_jd(jd, frac, Scale::TT);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(diff < 1e-10, "JD round-trip error {} s at {:?}", diff, p);
    }
}

/// Exact MJD ↔ Dt round-trip.
#[test]
fn mjd_tt_exact_roundtrip() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI, Scale::TAI),
    ];

    for &p in &test_points {
        let (mjd, frac) = p.target(Scale::TT).to_mjd();
        let back = Dt::from_mjd(mjd, frac, Scale::TT);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(diff < 1e-10, "MJD round-trip error {} s at {:?}", diff, p);
    }
}

#[test]
fn test_mjd_utc_roundtrip() {
    // Normal instant (non-leap)
    let original = Dt::from_ymd(2025, 4, 27, Scale::UTC, 14, 30, 0, 123_456_789_000_000_000);
    let (mjd, frac) = original.to_mjd();
    let roundtrip = Dt::from_mjd(mjd, frac, Scale::UTC);
    assert_eq!(
        original, roundtrip,
        "MJD UTC round-trip failed for normal time"
    );

    // Also exercise the JD variant
    let (jd, frac_jd) = original.to_jd();
    let roundtrip_jd = Dt::from_jd(jd, frac_jd, Scale::UTC);
    assert_eq!(original, roundtrip_jd, "JD UTC round-trip failed");

    // How to round trip a leap-second case (2015-06-30 23:59:60 UTC)
    // Have to use TAI, no conversion at the mjd/jd functions
    let leap = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    let (mjd_leap, frac_leap) = leap.target(Scale::TAI).to_mjd();
    let roundtrip_leap = Dt::from_mjd(mjd_leap, frac_leap, Scale::TAI);
    assert_eq!(
        leap, roundtrip_leap,
        "MJD UTC round-trip failed for leap second"
    );

    // Also verify JD round-trip on the leap second
    let (jd_leap, frac_jd_leap) = leap.target(Scale::TAI).to_jd();
    let roundtrip_jd_leap = Dt::from_jd(jd_leap, frac_jd_leap, Scale::TAI);
    assert_eq!(
        leap, roundtrip_jd_leap,
        "JD UTC round-trip failed for leap second"
    );
}

#[test]
fn ymd_jd_safety() {
    let test_points = [
        (i64::MIN, 1, 1),
        (0_i64, 1, 1),
        (i64::MAX, 1, 1),
        (i64::MIN, 12, 31),
        (0_i64, 12, 31),
        (i64::MAX, 12, 31),
    ];
    for (y, m, d) in &test_points {
        let jd = Dt::ymd_to_jd(*y, *m, *d);
        let ymd = Dt::jd_to_ymd(jd);
        assert_eq!(ymd, (ymd), "round trip extreme ymd jd failed");
    }

    let test_points = [i64::MIN, 0_i64, 1721060_i64, i64::MAX];
    for jd1 in &test_points {
        let (y, m, d) = Dt::jd_to_ymd(*jd1);
        let jd2 = Dt::ymd_to_jd(y, m, d);
        assert_eq!(*jd1, jd2, "round trip extreme jd ymd failed");
    }
}

#[test]
fn ymd_jd() {
    let test_points = [
        (0000, 1, 1, 1721060),
        (2000, 1, 1, 2451545),
        (2023, 1, 1, 2459946),
        (2024, 1, 1, 2460311),
        // end of year
        (0000, 12, 31, 1721425),
        (2000, 12, 31, 2451910),
        (2023, 12, 31, 2460310),
        (2024, 12, 31, 2460676),
    ];
    for (y, m, d, expected_jd) in &test_points {
        let jd = Dt::ymd_to_jd(*y, *m, *d);
        assert_eq!(jd, *expected_jd, "expected jd failed");

        let (yr, mo, day) = Dt::jd_to_ymd(*expected_jd);
        assert_eq!((yr, mo, day), (*y, *m, *d), "expected yr mo day failed");
    }
}
