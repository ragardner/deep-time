#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale};

#[test]
fn test_ymd_to_jd() {
    // ── Positive years ─────────────────────────────────────────────
    assert_eq!(Dt::ymd_to_jd(2025, 4, 16), 2460782);
    assert_eq!(Dt::ymd_to_jd(2000, 1, 1), 2451545); // J2000.0 epoch
    assert_eq!(Dt::ymd_to_jd(1970, 1, 1), 2440588); // Unix epoch
    assert_eq!(Dt::ymd_to_jd(1582, 10, 15), 2299161); // Gregorian calendar introduction
    assert_eq!(Dt::ymd_to_jd(1, 1, 1), 1721426);

    // ── Year 0 (corrected) ─────────────────────────────────────────
    assert_eq!(Dt::ymd_to_jd(0, 1, 1), 1721060);
    assert_eq!(Dt::ymd_to_jd(0, 12, 31), 1721425);

    // ── Negative years (BCE / large negative) (corrected) ──────────
    assert_eq!(Dt::ymd_to_jd(-1, 1, 1), 1720695);
    assert_eq!(Dt::ymd_to_jd(-1, 12, 31), 1721059);
    assert_eq!(Dt::ymd_to_jd(-4, 1, 1), 1719599); // leap year
    assert_eq!(Dt::ymd_to_jd(-100, 1, 1), 1684536);
    assert_eq!(Dt::ymd_to_jd(-400, 1, 1), 1574963);
    assert_eq!(Dt::ymd_to_jd(-100000, 12, 31), -34802825); // critical large negative year

    // ── Leap year edge cases (corrected) ───────────────────────────
    assert_eq!(Dt::ymd_to_jd(2000, 2, 29), 2451604); // leap year
    assert_eq!(Dt::ymd_to_jd(1900, 2, 28), 2415079); // not a leap year
    assert_eq!(Dt::ymd_to_jd(4, 2, 29), 1722580); // positive leap year
    assert_eq!(Dt::ymd_to_jd(-4, 2, 29), 1719658); // negative leap year

    // ── Round-trip tests ───────────────────────────────────────────
    let test_dates = [
        (2025, 4, 16),
        (2000, 1, 1),
        (1970, 1, 1),
        (1582, 10, 15),
        (1, 1, 1),
        (0, 1, 1),
        (0, 12, 31),
        (-1, 1, 1),
        (-1, 12, 31),
        (-4, 1, 1),
        (-100, 1, 1),
        (-400, 1, 1),
        (-100000, 12, 31),
        (123456, 7, 4),
        (-123456, 12, 31),
    ];
    for (y, m, d) in test_dates {
        let jd = Dt::ymd_to_jd(y, m, d);
        let (y2, m2, d2) = Dt::jd_to_ymd(jd);
        assert_eq!(
            (y2, m2, d2),
            (y, m, d),
            "round-trip failed for {}-{:02}-{:02}",
            y,
            m,
            d
        );
    }

    // ── Specific jd_to_ymd known values ─────────
    assert_eq!(Dt::jd_to_ymd(2460782), (2025, 4, 16));
    assert_eq!(Dt::jd_to_ymd(2451545), (2000, 1, 1));
    assert_eq!(Dt::jd_to_ymd(1721060), (0, 1, 1));
    assert_eq!(Dt::jd_to_ymd(1720695), (-1, 1, 1));
    assert_eq!(Dt::jd_to_ymd(-34802825), (-100000, 12, 31));
}

/// According to NASA/SPICE documentation:
///   TDB − TT ≈ K ⋅ sin(E)  (simple approximation)
///   amplitude ≈ 0.001658 s
///   this simple model is accurate to ~30 µs (it ignores small periodic terms)
///
/// Our implementation uses the more accurate 4-term Fairhead & Bretagnon model
/// (SOFA/ERFA `eraDtdb`), so the difference must still be < 2 ms and the
/// round-trip must be extremely tight.
#[test]
fn tdb_tt_difference_matches_spice_approximation() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI), // J2000.0
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI), // ~31.7 y after J2000
        Dt::from_sec(-500_000_000, Scale::TAI, Scale::TAI), // ~15.85 y before J2000
        Dt::from_sec(86_400 * 365 * 50, Scale::TAI, Scale::TAI), // +50 years
        Dt::from_sec(-86_400 * 365 * 100, Scale::TAI, Scale::TAI), // -100 years
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI), // ≈ J1900
    ];

    for &tai in &test_points {
        // These give the *numerical* values on each scale (correct usage of .to)
        let tt_num = tai.to(Scale::TT);
        let tdb_num = tai.to(Scale::TDB);

        // This is exactly TDB − TT in seconds (the quantity SPICE approximates)
        let diff = tdb_num.to_diff_raw(tt_num).to_sec_f().abs();

        assert!(
            diff < 0.002,
            "TDB-TT difference of {:.9} s exceeds expected max amplitude (~1.658 ms) at TAI = {:?}",
            diff,
            tai
        );
    }
}

/// Verifies lossless round-trip conversion through ET (which is TDB)
/// and back to the original TAI instant. This is the core safety
/// property required for all SPICE-based ephemeris work.
///
/// Source: SPICE time conversion routines (str2et, et2utc, unitim, etc.)
/// https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html
#[test]
fn et_tai_roundtrip_is_lossless() {
    let original = Dt::from_sec(987_654_321_098, Scale::TAI, Scale::TAI);

    let et = original.to(Scale::ET);
    let xt = et.to(Scale::TAI);

    assert_eq!(original, xt, "ET round-trip must be lossless");
}

/// Round-trip accuracy test (TAI → TDB → TAI)
#[test]
fn tdb_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI), // J2000 TAI
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI), // ~1 year later
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI), // 10 years before
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI), // ~31.7 years later
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI), // J1900 epoch
    ];

    for &p in &test_points {
        let tdb = p.to(Scale::TDB);
        let back = tdb.to(Scale::TAI);

        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(
            diff == 0.0,
            "TDB round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// Round-trip accuracy test (TAI → ET → TAI)
#[test]
fn et_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI), // J2000 TAI
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI), // ~1 year later
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI), // 10 years before
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI), // ~31.7 years later
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI), // J1900 epoch
    ];

    for &p in &test_points {
        let et = p.to(Scale::ET);
        let back = et.to(Scale::TAI);

        let diff = back.to_diff_raw(p).to_sec_f().abs();

        assert!(
            diff == 0.0,
            "ET round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// Check that the *periodic correction* (TDB − TT) stays within sensible bounds
#[test]
fn tdb_correction_stays_within_bounds() {
    let points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 50, Scale::TAI, Scale::TAI),
    ];

    for &p in &points {
        let tt = p.to(Scale::TT);
        let tdb = p.to(Scale::TDB);

        // TDB - TT (periodic term only)
        let corr_s = tdb.to_diff_raw(tt).to_sec_f();

        assert!(
            corr_s.abs() < 0.002,
            "TDB-TT correction should be < 2 ms (got {} s)",
            corr_s
        );
    }
}

#[cfg(feature = "physics")]
#[test]
fn proper_to_tt_with_drift_roundtrip() {
    use deep_time::Drift;

    let epoch = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    let drift = Drift::new(
        Dt::from_ms(100, 0, Scale::TAI, Scale::TAI), // exactly 0.1 s
        Dt::from_ns(1, 0, Scale::TAI, Scale::TAI),   // exactly 1 ns/s = 1e-9 s/s
        Dt::ZERO,
    );
    let onboard_proper = epoch.add(Dt::from_sec(1_000_000, Scale::TAI, Scale::TAI));
    let tt = onboard_proper.convert_using_drift(epoch, &drift);
    let back = tt.convert_back_using_drift(epoch, &drift);

    assert_eq!(back, onboard_proper);
}

/// TT is exactly TAI + 32.184 s (and ET is an alias for TT).
#[test]
fn tt_tai_offset_exact() {
    let tai = Dt::ZERO;
    let tt = tai.to(Scale::TT);
    let diff_s = tt.to_diff_raw(tai).to_sec_f();
    assert!(
        (diff_s - 32.184).abs() < 1e-12,
        "TT-TAI at J2000 was {} s (expected exactly 32.184)",
        diff_s
    );
}

/// All GNSS scales have fixed integer-second offsets from TAI.
#[test]
fn gnss_offsets_are_correct() {
    let tai = Dt::ZERO;

    let gpst = tai.to(Scale::GPS);
    assert!((gpst.to_diff_raw(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let qzsst = tai.to(Scale::QZSS);
    assert!((qzsst.to_diff_raw(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let gst = tai.to(Scale::GST);
    assert!((gst.to_diff_raw(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let bdt = tai.to(Scale::BDT);
    assert!((bdt.to_diff_raw(tai).to_sec_f() + 33.0).abs() < 1e-12);
}

/// TCG ↔ TAI round-trip (pure linear rate – should be exact within f64 noise).
#[test]
fn tcg_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
    ];

    for &p in &test_points {
        let tcg = p.to(Scale::TCG);
        let back = tcg.to(Scale::TAI);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(
            diff < 1e-9,
            "TCG round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// TCB ↔ TAI round-trip (linear + constant TDB0 term).
#[test]
fn tcb_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
    ];

    for &p in &test_points {
        let tcb = p.to(Scale::TCB);
        let back = tcb.to(Scale::TAI);
        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(
            diff < 1e-9,
            "TCB round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// UTC ↔ TAI round-trip must be exact (leap-second table is bijective).
#[test]
fn utc_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
        Dt::from_sec(1_485_779_200, Scale::TAI, Scale::TAI), // around 2017-01-01 leap second
    ];

    for &p in &test_points {
        let utc = p.to(Scale::UTC);
        let back = utc.to(Scale::TAI);
        assert_eq!(back, p, "UTC round-trip failed at {:?}", p);
    }
}

#[test]
fn ymd_to_jd_j2000() {
    assert_eq!(Dt::ymd_to_jd(2000, 1, 1), 2451545);
}

#[test]
fn ymd_to_jd_leap_year_handling() {
    assert_eq!(Dt::ymd_to_jd(2000, 2, 29), 2451604); // leap day
    assert_eq!(Dt::ymd_to_jd(1900, 2, 28), 2415079); // non-leap
}

#[test]
fn is_leap_year_and_valid_date() {
    assert!(Dt::is_leap_yr(2000));
    assert!(!Dt::is_leap_yr(1900));
    assert!(Dt::is_valid_ymd(2024, 2, 29));
    assert!(!Dt::is_valid_ymd(2023, 2, 29));
}

#[test]
fn unix_days_roundtrip() {
    use deep_time::consts::ATTOS_PER_HALF_DAY_U128;

    let epoch = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert_eq!(epoch.to_unix_days(), (0, 0));

    let noon_2000 = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    let (days, attos) = noon_2000.to_unix_days();
    assert_eq!(days, 10_957);
    assert_eq!(attos, ATTOS_PER_HALF_DAY_U128);

    let roundtrip = Dt::from_unix_days(days, attos, Scale::UTC);
    assert_eq!(roundtrip, noon_2000);

    let days_f = noon_2000.to_unix_days_f();
    let roundtrip_f = Dt::from_unix_days_f(days_f, Scale::UTC);
    assert_eq!(roundtrip_f.to_unix_days_f(), days_f);
}

#[test]
fn ntp_timestamp() {
    // 2698012800
    let dt = Dt::from_ymd(1985, 7, 1, Scale::UTC, 0, 0, 0, 0);
    let ntp = dt.to_ntp();
    assert_eq!(
        ntp.to_sec(),
        2698012800_i128,
        "ntp sec for 1985 is wrong, got: {}, expected: {}",
        ntp.to_sec(),
        2698012800_i128
    );
    let dt2 = Dt::from_ntp(ntp);
    assert_eq!(
        dt.to_sec(),
        dt2.to_sec(),
        "round trip to Dt got wrong sec, old: {}, new: {}",
        dt.to_sec(),
        dt2.to_sec()
    );
    let ymd = dt2.to_ymd();
    assert_eq!(ymd.yr(), 1985_i64);
    assert_eq!(ymd.mo(), 7);
    assert_eq!(ymd.day(), 1);
    assert_eq!(ymd.hr(), 0);
    assert_eq!(ymd.min(), 0);
    assert_eq!(ymd.sec(), 0);
    assert_eq!(ymd.attos(), 0);
}

#[test]
fn unit_split_roundtrip_positive() {
    let dt = Dt::from_ns(5, 123_456_789, Scale::TAI, Scale::TAI);
    let (whole, frac_attos) = dt.to_ns_floor();
    assert_eq!(whole, 5);
    assert_eq!(frac_attos, 123_456_789);
    assert_eq!(
        Dt::from_ns(whole, frac_attos as i128, Scale::TAI, Scale::TAI),
        dt
    );

    let ms_dt = Dt::span(1_300_000_000_000_000_000);
    let (whole, frac_attos) = ms_dt.to_ms_floor();
    assert_eq!(whole, 1300);
    assert_eq!(frac_attos, 0);
    assert_eq!(
        Dt::from_ms(whole, frac_attos as i128, Scale::TAI, Scale::TAI),
        ms_dt
    );

    let fs_dt = Dt::from_fs(42, 0, Scale::TAI, Scale::TAI);
    let (_, frac_attos) = fs_dt.to_fs_floor();
    assert_eq!(frac_attos, 0);
}

#[test]
fn unit_split_negative_whole_plus_fraction() {
    let dt = Dt::span(-1_000_500_000_000_000_000);
    let (whole, frac_attos) = dt.to_ms_floor();
    assert_eq!(whole, -1001);
    assert_eq!(frac_attos, 500_000_000_000_000);
    assert_eq!(
        Dt::from_ms(whole, frac_attos as i128, Scale::TAI, Scale::TAI),
        dt
    );

    // Truncation toward zero differs for negative non-whole values
    let (whole, frac_attos) = dt.to_ms();
    assert_eq!(whole, -1000);
    assert_eq!(frac_attos, -500_000_000_000_000);
    assert_eq!(Dt::from_ms(whole, frac_attos, Scale::TAI, Scale::TAI), dt);
}

#[test]
fn unit_split_trunc_semantics() {
    let positive = Dt::span(1_300_500_000_000_000_000);
    let (whole, frac_attos) = positive.to_ms();
    assert_eq!(whole, 1300);
    assert_eq!(frac_attos, 500_000_000_000_000);
    assert_eq!(
        Dt::from_ms(whole, frac_attos, Scale::TAI, Scale::TAI),
        positive
    );

    let negative_whole = Dt::span(-1_300_000_000_000_000_000);
    let (whole, frac_attos) = negative_whole.to_ms();
    assert_eq!(whole, -1300);
    assert_eq!(frac_attos, 0);
    assert_eq!(
        Dt::from_ms(whole, frac_attos, Scale::TAI, Scale::TAI),
        negative_whole
    );

    let small_negative = Dt::span(-500_000_000_000_000);
    let (whole, frac_attos) = small_negative.to_ms();
    assert_eq!(whole, 0);
    assert_eq!(frac_attos, -500_000_000_000_000);
    assert_eq!(
        Dt::from_ms(whole, frac_attos, Scale::TAI, Scale::TAI),
        small_negative
    );
}

#[test]
fn unit_split_trunc_roundtrip_all_units() {
    use deep_time::consts::{ATTOS_PER_DAY, ATTOS_PER_SEC_I128};

    let dt = Dt::span(-1_000_500_000_000_000_000);

    let (whole, frac) = dt.to_ms();
    assert_eq!(Dt::from_ms(whole, frac, Scale::TAI, Scale::TAI), dt);

    let (whole, frac) = dt.to_us();
    assert_eq!(Dt::from_us(whole, frac, Scale::TAI, Scale::TAI), dt);

    let (whole, frac) = dt.to_ns();
    assert_eq!(Dt::from_ns(whole, frac, Scale::TAI, Scale::TAI), dt);

    let (whole, frac) = dt.to_ps();
    assert_eq!(Dt::from_ps(whole, frac, Scale::TAI, Scale::TAI), dt);

    let (whole, frac) = dt.to_fs();
    assert_eq!(Dt::from_fs(whole, frac, Scale::TAI, Scale::TAI), dt);

    // mins / hours / days: construct from whole + signed attos fraction
    let mins = Dt::from_mins(-1, -30 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);
    assert_eq!(mins.to_attos(), -90 * ATTOS_PER_SEC_I128);

    let hours = Dt::from_hours(-2, -1800 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);
    assert_eq!(hours.to_attos(), -9000 * ATTOS_PER_SEC_I128);

    let days = Dt::from_days(-1, -ATTOS_PER_DAY / 4, Scale::TAI, Scale::TAI);
    assert_eq!(days.to_attos(), -5 * ATTOS_PER_DAY / 4);
}
