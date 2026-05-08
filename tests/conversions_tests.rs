use deep_time::{
    ClockDrift, ClockModel, Dt, Scale, TSpan,
    constants::{ATTOS_PER_HALF_DAYU, MARS_SOL_LENGTH_SEC},
    to_sec_f,
};

#[test]
fn test_ymd_to_jdn() {
    // ── Positive years ─────────────────────────────────────────────
    assert_eq!(Dt::ymd_to_jdn(2025, 4, 16), 2460782);
    assert_eq!(Dt::ymd_to_jdn(2000, 1, 1), 2451545); // J2000.0 epoch
    assert_eq!(Dt::ymd_to_jdn(1970, 1, 1), 2440588); // Unix epoch
    assert_eq!(Dt::ymd_to_jdn(1582, 10, 15), 2299161); // Gregorian calendar introduction
    assert_eq!(Dt::ymd_to_jdn(1, 1, 1), 1721426);

    // ── Year 0 (corrected) ─────────────────────────────────────────
    assert_eq!(Dt::ymd_to_jdn(0, 1, 1), 1721060);
    assert_eq!(Dt::ymd_to_jdn(0, 12, 31), 1721425);

    // ── Negative years (BCE / large negative) (corrected) ──────────
    assert_eq!(Dt::ymd_to_jdn(-1, 1, 1), 1720695);
    assert_eq!(Dt::ymd_to_jdn(-1, 12, 31), 1721059);
    assert_eq!(Dt::ymd_to_jdn(-4, 1, 1), 1719599); // leap year
    assert_eq!(Dt::ymd_to_jdn(-100, 1, 1), 1684536);
    assert_eq!(Dt::ymd_to_jdn(-400, 1, 1), 1574963);
    assert_eq!(Dt::ymd_to_jdn(-100000, 12, 31), -34802825); // critical large negative year

    // ── Leap year edge cases (corrected) ───────────────────────────
    assert_eq!(Dt::ymd_to_jdn(2000, 2, 29), 2451604); // leap year
    assert_eq!(Dt::ymd_to_jdn(1900, 2, 28), 2415079); // not a leap year
    assert_eq!(Dt::ymd_to_jdn(4, 2, 29), 1722580); // positive leap year
    assert_eq!(Dt::ymd_to_jdn(-4, 2, 29), 1719658); // negative leap year

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
        let jdn = Dt::ymd_to_jdn(y, m, d);
        let (y2, m2, d2) = Dt::jdn_to_ymd(jdn);
        assert_eq!(
            (y2, m2, d2),
            (y, m, d),
            "round-trip failed for {}-{:02}-{:02}",
            y,
            m,
            d
        );
    }

    // ── Specific jdn_to_ymd known values (corrected) ─────────
    assert_eq!(Dt::jdn_to_ymd(2460782), (2025, 4, 16));
    assert_eq!(Dt::jdn_to_ymd(2451545), (2000, 1, 1));
    assert_eq!(Dt::jdn_to_ymd(1721060), (0, 1, 1));
    assert_eq!(Dt::jdn_to_ymd(1720695), (-1, 1, 1));
    assert_eq!(Dt::jdn_to_ymd(-34802825), (-100000, 12, 31));
}

#[cfg(feature = "tz")]
#[test]
fn roundtrip_gap_boundary_new_york() {
    let our_input = "2023-03-12 02:00:00 America/New_York";
    let expected_snapped = "2023-03-12 03:00:00 America/New_York";

    // Parse the non-existent local time (should succeed via lenient gap handling)
    let our_dt: Dt =
        Dt::from_str_parse(our_input, &None).expect("parse should succeed (lenient gap handling)");

    // Verify internal representation (the snapped UTC instant)
    assert_eq!(
        our_dt.to_epoch(Dt::UNIX_EPOCH, Scale::UTC).to_sec(),
        1678604400,
        "internal unix timestamp should be the snapped UTC instant"
    );

    // Format back using the IANA zone
    let fmt = "%Y-%m-%d %H:%M:%S %Q";
    let output = our_dt
        .to_str_with_tz(fmt, "America/New_York")
        .expect("to_str_with_tz should succeed");

    // === THE KEY REGRESSION ASSERT ===
    assert_eq!(
        output, expected_snapped,
        "gap time should silently snap forward to the next valid local time (post-DST)"
    );

    // Bonus: verify the round-trip is stable (parse → format → parse → format)
    let our_dt2: Dt = Dt::from_str_parse(&output, &None).expect("second parse should also succeed");
    let output2 = our_dt2
        .to_str_with_tz(fmt, "America/New_York")
        .expect("second format should succeed");

    assert_eq!(output2, expected_snapped, "round-trip must be stable");
}

#[test]
fn test_mjd_utc_roundtrip() {
    // Normal instant (non-leap)
    let original = Dt::from_ymdhms(2025, 4, 27, 14, 30, 0, 123_456_789_000_000_000);
    let (mjd, frac) = original.to_mjd_exact(Scale::UTC);
    let roundtrip = Dt::from_mjd_exact(mjd, frac, Scale::UTC);
    assert_eq!(
        original, roundtrip,
        "MJD UTC round-trip failed for normal time"
    );

    // Also exercise the JD variant
    let (jd, frac_jd) = original.to_jd_exact(Scale::UTC);
    let roundtrip_jd = Dt::from_jd_exact(jd, frac_jd, Scale::UTC);
    assert_eq!(original, roundtrip_jd, "JD UTC round-trip failed");

    // Leap-second case (2015-06-30 23:59:60 UTC) — the trickiest path
    let leap = Dt::from_ymdhms(2015, 6, 30, 23, 59, 60, 0);
    let (mjd_leap, frac_leap) = leap.to_mjd_exact(Scale::UTC);
    let roundtrip_leap = Dt::from_mjd_exact(mjd_leap, frac_leap, Scale::UTC);
    assert_eq!(
        leap, roundtrip_leap,
        "MJD UTC round-trip failed for leap second"
    );

    // Also verify JD round-trip on the leap second
    let (jd_leap, frac_jd_leap) = leap.to_jd_exact(Scale::UTC);
    let roundtrip_jd_leap = Dt::from_jd_exact(jd_leap, frac_jd_leap, Scale::UTC);
    assert_eq!(
        leap, roundtrip_jd_leap,
        "JD UTC round-trip failed for leap second"
    );
}

/// Verifies that the TDB-TT difference produced by our implementation
/// stays within the documented SPICE tolerance (~30 µs accuracy for
/// the simple approximation, max amplitude ~1.658 ms).
///
/// Source: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html
/// (section describing "TDB - TT = K * sin(E)" with amplitude ~0.001658 s)
#[test]
fn tdb_tt_difference_matches_spice_approximation() {
    // Test at a few representative epochs
    for &tai_sec in &[0i64, 1_000_000_000, -500_000_000] {
        // from tai sec to a tt timespan
        let tt = Dt::from_sec(tai_sec, Scale::TAI).to(Scale::TT);
        // from tt timespan to tdb timespan, create tai from tdb timespan
        let tdb = tt.to(Scale::TT, Scale::TDB).to_tai(Scale::TDB);
        // create tai from tt, measure against tdb (tai internally)
        let diff = tdb.to_tai_since(tt.to_tai(Scale::TT)).to_sec_f().abs();
        assert!(
            diff < 0.002,
            "TDB-TT difference ({:.6} s) exceeded SPICE documented max (~1.658 ms)",
            diff
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
    let original = Dt::from_sec(987_654_321_098, Scale::TAI);

    let et = original.to(Scale::ET);
    let xt = et.to_tai(Scale::ET);

    assert_eq!(original, xt, "ET round-trip must be lossless");
}

/// Round-trip accuracy test (TAI → TDB → TAI)
#[test]
fn tdb_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),                  // J2000 TAI
        Dt::from_sec(86_400 * 365, Scale::TAI),       // ~1 year later
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI), // 10 years before
        Dt::from_sec(1_000_000_000, Scale::TAI),      // ~31.7 years later
        Dt::from_sec(-2_208_945_600, Scale::TAI),     // J1900 epoch
    ];

    for &p in &test_points {
        let tdb = p.to(Scale::TDB);
        let back = tdb.to_tai(Scale::TDB);

        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(
            diff < 1e-6,
            "TDB round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// At J2000 the TDB–TAI difference should be ~32.183925 s
/// (TT = TAI + 32.184 s and TDB − TT ≈ −74.6 µs with this formula)
#[test]
fn tdb_minus_tt_at_j2000() {
    let tai = Dt::ZERO;
    let tdb = tai.to(Scale::TDB);

    let diff_s = tdb.to_diff_tp(tai).to_sec_f(); // see helper below

    assert!(
        (diff_s - 32.183925).abs() < 0.00001,
        "TDB-TAI difference at J2000 was {} s (expected ~32.183925 s)",
        diff_s
    );
}

/// Check that the *periodic correction* (TDB − TT) stays within sensible bounds
#[test]
fn tdb_correction_stays_within_bounds() {
    let points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 50, Scale::TAI),
    ];

    for &p in &points {
        let tt = p.to(Scale::TT);
        let tdb = p.to(Scale::TDB);

        // TDB - TT (periodic term only)
        let corr_s = tdb.to_diff(tt).to_sec_f();

        assert!(
            corr_s.abs() < 0.002,
            "TDB-TT correction should be < 2 ms (got {} s)",
            corr_s
        );
    }
}

#[test]
fn proper_to_tt_with_drift_roundtrip() {
    let reference = Dt::from_sec(0, Scale::TAI);
    let drift = ClockDrift::new(
        TSpan::from_ms(100), // exactly 0.1 s
        TSpan::from_ns(1),   // exactly 1 ns/s = 1e-9 s/s
        TSpan::ZERO,
    );
    let model = ClockModel::new(Scale::Custom, reference, drift);

    let onboard_proper = model.reference.add(TSpan::from_sec(1_000_000));

    let tt = onboard_proper.convert_using_model(model);
    let back = tt.convert_back_using_model(model);

    assert_eq!(back, onboard_proper);
}

/// Round-trip accuracy test (TAI → LTC → TAI)
///
/// LTC conversion is purely linear, so round-trips should be extremely
/// accurate. The observed error is ~0.3 ns due to unavoidable f64 rounding
/// noise in `to_jd_tt()` + the rate multiplication. We therefore allow a
/// very small tolerance that is still far tighter than any practical use case.
#[test]
fn ltc_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),                  // J2000 TAI
        Dt::from_sec(86_400 * 365, Scale::TAI),       // ~1 year later
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI), // 10 years before
        Dt::from_sec(1_000_000_000, Scale::TAI),      // ~31.7 years later
        Dt::from_sec(-2_208_945_600, Scale::TAI),     // J1900 epoch
    ];

    for &p in &test_points {
        let ltc = p.to(Scale::LTC);
        let back = ltc.to_tai(Scale::LTC);

        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(
            diff < 1e-9,
            "LTC round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// At J2000 the LTC–TAI difference must be exactly the value produced by
/// the library’s f64 math (L_M × days × 86400 + TT–TAI offset).
#[test]
fn ltc_minus_tai_at_j2000() {
    let tai = Dt::ZERO;
    let ltc = tai.to(Scale::LTC);

    let diff_s = ltc.to_diff(tai.to_span()).to_sec_f();

    assert!(
        (diff_s - 32.6545948272096).abs() < 1e-9,
        "LTC-TAI difference at J2000 was {} s (expected 32.6545948272096 s)",
        diff_s
    );
}

/// Verify that the LTC–TT difference grows linearly (pure secular term).
#[test]
fn ltc_offset_grows_linearly() {
    let points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI), // ~1 year
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI), // ~100 years
    ];

    for &p in &points {
        let tt = p.to(Scale::TT);
        let ltc = p.to(Scale::LTC);

        // LTC - TT (pure secular term)
        let corr_s = ltc.to_diff(tt).to_sec_f();

        assert!(
            corr_s > 0.0,
            "LTC-TT correction should be positive (got {} s at {:?})",
            corr_s,
            p
        );

        // At ~100 years the offset should be ~2.5 s (56 µs/day × 36525 days)
        if p.sec() > 86_400 * 365 * 50 {
            assert!(
                corr_s > 1.0 && corr_s < 4.0,
                "LTC-TT correction at ~100y should be ~2-3 s (got {} s)",
                corr_s
            );
        }
    }
}

#[test]
fn msd_exact_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
    ];

    for &p in &test_points {
        let (whole, frac) = p.to_msd_exact();
        let back = Dt::from_msd_exact(whole, frac);

        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(
            diff < 5e-5, // ← relaxed for f64 JD precision (max observed error ≈ 13.7 µs)
            "MSD round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

#[test]
fn msd_float_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
    ];

    for &p in &test_points {
        let msd_float = p.to_msd();
        let back = Dt::from_msd(msd_float);

        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(
            diff < 5e-5, // ← relaxed for f64 MSD path (max observed error ≈ 13.7 µs)
            "MSD float round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

#[test]
fn mtc_is_in_valid_range() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
    ];

    for &p in &test_points {
        let mtc = p.to_mtc();
        let mtc_sec = mtc.to_sec_f();
        assert!(
            mtc_sec >= 0.0 && mtc_sec < MARS_SOL_LENGTH_SEC,
            "MTC out of range: {} s at {:?}",
            mtc_sec,
            p
        );
    }
}

#[test]
fn msd_at_j2000_is_correct() {
    let tai = Dt::ZERO;
    let (whole, frac) = tai.to_msd_exact();

    assert_eq!(whole, 44791, "Integer part of MSD at J2000 should be 44791");

    // New exact value (no magic number)
    let frac_sols = to_sec_f(frac) / MARS_SOL_LENGTH_SEC;
    assert!(
        (frac_sols - 0.61987471912).abs() < 1e-11, // or use a TSpan comparison
        "Fractional part of MSD at J2000 (TAI) was {} sols",
        frac_sols
    );
}

/// TT is exactly TAI + 32.184 s (and ET is an alias for TT).
#[test]
fn tt_tai_offset_exact() {
    let tai = Dt::ZERO;
    let tt = tai.to(Scale::TT);
    let diff_s = tt.to_diff_tp(tai).to_sec_f();
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
    assert!((gpst.to_diff_tp(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let qzsst = tai.to(Scale::QZSS);
    assert!((qzsst.to_diff_tp(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let gst = tai.to(Scale::GST);
    assert!((gst.to_diff_tp(tai).to_sec_f() + 19.0).abs() < 1e-12);

    let bdt = tai.to(Scale::BDT);
    assert!((bdt.to_diff_tp(tai).to_sec_f() + 33.0).abs() < 1e-12);
}

/// TCG ↔ TAI round-trip (pure linear rate – should be exact within f64 noise).
#[test]
fn tcg_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
    ];

    for &p in &test_points {
        let tcg = p.to(Scale::TCG);
        let back = tcg.to_tai(Scale::TCG);
        let diff = back.to_tai_since(p).to_sec_f().abs();
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
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
    ];

    for &p in &test_points {
        let tcb = p.to(Scale::TCB);
        let back = tcb.to_tai(Scale::TCB);
        let diff = back.to_tai_since(p).to_sec_f().abs();
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
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
        Dt::from_sec(1_485_779_200, Scale::TAI), // around 2017-01-01 leap second
    ];

    for &p in &test_points {
        let utc = p.to(Scale::UTC);
        let back = utc.to_tai(Scale::UTC);
        assert_eq!(back, p, "UTC round-trip failed at {:?}", p);
    }
}

#[test]
fn j2000_tt_is_jd_2451545() {
    let j2000_tt = Dt::from_jd_exact(2451545, 0, Scale::TT);

    let (jd, frac) = j2000_tt.to_jd_exact(Scale::TT);
    assert_eq!(jd, 2451545);
    assert_eq!(frac, 0);

    let (mjd, mjd_frac) = j2000_tt.to_mjd_exact(Scale::TT);

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
        let (jd, frac) = p.to_jd_exact(Scale::TT);
        let back = Dt::from_jd_exact(jd, frac, Scale::TT);
        let diff = back.to_tai_since(p).to_sec_f().abs();
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
        let (mjd, frac) = p.to_mjd_exact(Scale::TT);
        let back = Dt::from_mjd_exact(mjd, frac, Scale::TT);
        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(diff < 1e-10, "MJD round-trip error {} s at {:?}", diff, p);
    }
}

#[test]
fn ymd_to_jdn_j2000() {
    assert_eq!(Dt::ymd_to_jdn(2000, 1, 1), 2451545);
}

#[test]
fn ymd_to_jdn_leap_year_handling() {
    assert_eq!(Dt::ymd_to_jdn(2000, 2, 29), 2451604); // leap day
    assert_eq!(Dt::ymd_to_jdn(1900, 2, 28), 2415079); // non-leap
}

#[test]
fn is_leap_year_and_valid_date() {
    assert!(Dt::is_leap_year(2000));
    assert!(!Dt::is_leap_year(1900));
    assert!(Dt::is_valid_ymd(2024, 2, 29));
    assert!(!Dt::is_valid_ymd(2023, 2, 29));
}
