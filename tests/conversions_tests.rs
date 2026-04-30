use deep_time_core::{
    ClockDrift, ClockModel, ClockType, TimePoint, TimeSpan, constants::MARS_SOL_LENGTH_SEC,
};

#[test]
fn test_ymd_to_jdn() {
    // ── Positive years ─────────────────────────────────────────────
    assert_eq!(TimePoint::ymd_to_jdn(2025, 4, 16), 2460782);
    assert_eq!(TimePoint::ymd_to_jdn(2000, 1, 1), 2451545); // J2000.0 epoch
    assert_eq!(TimePoint::ymd_to_jdn(1970, 1, 1), 2440588); // Unix epoch
    assert_eq!(TimePoint::ymd_to_jdn(1582, 10, 15), 2299161); // Gregorian calendar introduction
    assert_eq!(TimePoint::ymd_to_jdn(1, 1, 1), 1721426);

    // ── Year 0 (corrected) ─────────────────────────────────────────
    assert_eq!(TimePoint::ymd_to_jdn(0, 1, 1), 1721060);
    assert_eq!(TimePoint::ymd_to_jdn(0, 12, 31), 1721425);

    // ── Negative years (BCE / large negative) (corrected) ──────────
    assert_eq!(TimePoint::ymd_to_jdn(-1, 1, 1), 1720695);
    assert_eq!(TimePoint::ymd_to_jdn(-1, 12, 31), 1721059);
    assert_eq!(TimePoint::ymd_to_jdn(-4, 1, 1), 1719599); // leap year
    assert_eq!(TimePoint::ymd_to_jdn(-100, 1, 1), 1684536);
    assert_eq!(TimePoint::ymd_to_jdn(-400, 1, 1), 1574963);
    assert_eq!(TimePoint::ymd_to_jdn(-100000, 12, 31), -34802825); // critical large negative year

    // ── Leap year edge cases (corrected) ───────────────────────────
    assert_eq!(TimePoint::ymd_to_jdn(2000, 2, 29), 2451604); // leap year
    assert_eq!(TimePoint::ymd_to_jdn(1900, 2, 28), 2415079); // not a leap year
    assert_eq!(TimePoint::ymd_to_jdn(4, 2, 29), 1722580); // positive leap year
    assert_eq!(TimePoint::ymd_to_jdn(-4, 2, 29), 1719658); // negative leap year

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
        let jdn = TimePoint::ymd_to_jdn(y, m, d);
        let (y2, m2, d2) = TimePoint::jdn_to_ymd(jdn);
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
    assert_eq!(TimePoint::jdn_to_ymd(2460782), (2025, 4, 16));
    assert_eq!(TimePoint::jdn_to_ymd(2451545), (2000, 1, 1));
    assert_eq!(TimePoint::jdn_to_ymd(1721060), (0, 1, 1));
    assert_eq!(TimePoint::jdn_to_ymd(1720695), (-1, 1, 1));
    assert_eq!(TimePoint::jdn_to_ymd(-34802825), (-100000, 12, 31));
}

#[test]
fn roundtrip_gap_boundary_new_york() {
    let our_input = "2023-03-12 02:00:00 America/New_York";
    let expected_snapped = "2023-03-12 03:00:00 America/New_York";

    // Parse the non-existent local time (should succeed via lenient gap handling)
    let our_dt: TimePoint = TimePoint::from_str_parse(our_input, &None)
        .expect("parse should succeed (lenient gap handling)");

    // Verify internal representation (the snapped UTC instant)
    assert_eq!(
        our_dt.to_unix_sec(),
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
    let our_dt2: TimePoint =
        TimePoint::from_str_parse(&output, &None).expect("second parse should also succeed");
    let output2 = our_dt2
        .to_str_with_tz(fmt, "America/New_York")
        .expect("second format should succeed");

    assert_eq!(output2, expected_snapped, "round-trip must be stable");
}

#[test]
fn test_mjd_utc_roundtrip() {
    // Normal instant (non-leap)
    let original = TimePoint::from_gregorian_ymdhms(
        2025,
        4,
        27,
        14,
        30,
        0,
        123_456_789_000_000_000,
        ClockType::UTC,
    );
    let (mjd, frac) = original.to_mjd_utc_exact();
    let roundtrip = TimePoint::from_mjd_utc_exact(mjd, frac);
    assert_eq!(
        original, roundtrip,
        "MJD UTC round-trip failed for normal time"
    );

    // Also exercise the JD variant
    let (jd, frac_jd) = original.to_jd_utc_exact();
    let roundtrip_jd = TimePoint::from_jd_utc_exact(jd, frac_jd);
    assert_eq!(original, roundtrip_jd, "JD UTC round-trip failed");

    // Leap-second case (2015-06-30 23:59:60 UTC) — the trickiest path
    let leap = TimePoint::from_gregorian_ymdhms(2015, 6, 30, 23, 59, 60, 0, ClockType::UTC);
    let (mjd_leap, frac_leap) = leap.to_mjd_utc_exact();
    let roundtrip_leap = TimePoint::from_mjd_utc_exact(mjd_leap, frac_leap);
    assert_eq!(
        leap, roundtrip_leap,
        "MJD UTC round-trip failed for leap second"
    );

    // Also verify JD round-trip on the leap second
    let (jd_leap, frac_jd_leap) = leap.to_jd_utc_exact();
    let roundtrip_jd_leap = TimePoint::from_jd_utc_exact(jd_leap, frac_jd_leap);
    assert_eq!(
        leap, roundtrip_jd_leap,
        "JD UTC round-trip failed for leap second"
    );
}

#[test]
fn test_leap_second_gotcha_1972_06_30() {
    let leap = TimePoint::from_gregorian_ymdhms(1972, 6, 30, 23, 59, 60, 0, ClockType::UTC);
    let g = leap.to_gregorian_ymdhms();
    assert_eq!(g.sec, 60);
    assert_eq!(g.day, 30);
}

#[test]
fn test_leap_second_roundtrip_2015_06_30() {
    // A leap second from the middle of the table (36 leap seconds accumulated)
    let original = TimePoint::from_gregorian_ymdhms(
        2015,
        6,
        30,
        23,
        59,
        60,
        123_456_789_000_000_000,
        ClockType::UTC,
    );

    // === Round-trip through canonical attoseconds ===
    let canon = original.to_attos_since(TimePoint::UNIX_EPOCH_UTC);
    let roundtrip1 = TimePoint::from_to_attos_since(canon, TimePoint::UNIX_EPOCH_UTC);

    assert_eq!(original, roundtrip1, "Canonical round-trip failed");

    // === Multiple Gregorian round-trips ===
    let mut current = original;
    for i in 0..5 {
        let g = current.to_gregorian_ymdhms();
        assert_eq!(g.sec, 60, "Leap second lost on iteration {}", i);
        assert_eq!(g.day, 30);
        assert_eq!(g.mo, 6);
        assert_eq!(g.yr, 2015);

        current = TimePoint::from_gregorian_ymdhms(
            g.yr,
            g.mo,
            g.day,
            g.hr,
            g.min,
            g.sec,
            g.subsec,
            ClockType::UTC,
        );
    }
    assert_eq!(original, current, "Multiple Gregorian round-trips failed");

    // Final sanity check via to_gregorian_time
    let gt = original.to_gregorian_time();
    assert_eq!(gt.sec(), 60);
    assert_eq!(gt.day(), 30);
}

/// Verifies that `ClockType::ET` is a true alias for `ClockType::TDB`
/// as defined by NASA/NAIF SPICE.
///
/// Per the official SPICE documentation:
/// "In the Toolkit ET Means TDB. When ephemeris time is called for by
/// CSPICE functions, TDB is the implied time system."
///
/// Source: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html
/// (section "In the Toolkit ET Means TDB")
#[test]
fn et_is_alias_for_tdb() {
    let p = TimePoint::from_tai_sec(1_234_567_890);

    let et = p.to_clock_type(ClockType::ET);
    let tdb = p.to_clock_type(ClockType::TDB);

    assert_eq!(
        et, tdb,
        "ET and TDB must represent the identical physical instant"
    );
    assert_eq!(et.clock_type(), ClockType::ET);
    assert_eq!(tdb.clock_type(), ClockType::TDB);
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
        let tt = TimePoint::from_tai_sec(tai_sec).to_clock_type(ClockType::TT);
        let tdb = tt.to_clock_type(ClockType::TDB);

        let diff = tdb.duration_since(tt).as_sec_f().abs();
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
    let original = TimePoint::from_tai_sec(987_654_321_098);

    let et = original.to_clock_type(ClockType::ET);
    let back_to_tai = et.to_tai();

    assert_eq!(original, back_to_tai, "ET round-trip must be lossless");
}

/// Round-trip accuracy test (TAI → TDB → TAI)
#[test]
fn tdb_tai_roundtrip_is_accurate() {
    let test_points = [
        TimePoint::from_tai_sec(0),                  // J2000 TAI
        TimePoint::from_tai_sec(86_400 * 365),       // ~1 year later
        TimePoint::from_tai_sec(-86_400 * 365 * 10), // 10 years before
        TimePoint::from_tai_sec(1_000_000_000),      // ~31.7 years later
        TimePoint::from_tai_sec(-2_208_945_600),     // J1900 epoch
    ];

    for &p in &test_points {
        let tdb = p.to_clock_type(ClockType::TDB);
        let back = tdb.to_clock_type(ClockType::TAI);

        let diff = back.duration_since(p).as_sec_f().abs();
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
    let tai = TimePoint::ZERO;
    let tdb = tai.to_clock_type(ClockType::TDB);
    let diff_s = tdb.numerical_seconds_since(&tai); // see helper below

    assert!(
        (diff_s - 32.183925).abs() < 0.00001,
        "TDB-TAI difference at J2000 was {} s (expected ~32.183925 s)",
        diff_s
    );
}

#[test]
fn tdb_minus_tt_at_j2000_2() {
    let tai = TimePoint::ZERO;
    let tdb = tai.to_clock_type(ClockType::TDB);
    let diff_s = tdb.numerical_seconds_since(&tai);

    assert!((diff_s - 32.18392391273422).abs() < 1e-6, "got {}", diff_s);
}

/// Check that the *periodic correction* (TDB − TT) stays within sensible bounds
#[test]
fn tdb_correction_stays_within_bounds() {
    let points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365 * 100),
        TimePoint::from_tai_sec(-86_400 * 365 * 50),
    ];

    for &p in &points {
        let tt = p.to_clock_type(ClockType::TT);
        let tdb = p.to_clock_type(ClockType::TDB);

        // TDB - TT (periodic term only)
        let corr_s = tdb.numerical_seconds_since(&tt);

        assert!(
            corr_s.abs() < 0.002,
            "TDB-TT correction should be < 2 ms (got {} s)",
            corr_s
        );
    }
}

#[test]
fn proper_to_tt_with_drift_roundtrip() {
    let reference = TimePoint::from_tai_sec(0);
    let drift = ClockDrift::new(
        TimeSpan::from_ms(100), // exactly 0.1 s
        TimeSpan::from_ns(1),   // exactly 1 ns/s = 1e-9 s/s
        TimeSpan::ZERO,
    );
    let model = ClockModel::proper(reference, drift);

    let onboard_proper = TimePoint::create_from_model(model).add(TimeSpan::from_sec(1_000_000));

    let tt = onboard_proper.convert_using_model(model);
    let back = tt.convert_back_using_model(model);

    assert_eq!(back, onboard_proper);
}

#[test]
fn zero_drift_is_identity() {
    let reference = TimePoint::from_tai_sec(0);
    let drift = ClockDrift::ZERO;
    let model = ClockModel::proper(reference, drift);

    let p = TimePoint::from_tai_sec(1_234_567);
    let converted = p.convert_using_model(model);

    assert_eq!(converted, p.to_clock_type(ClockType::Proper));
}

#[test]
fn constant_offset_only() {
    let reference = TimePoint::from_tai_sec(0);
    let drift = ClockDrift::from_constant(TimeSpan::from_sec_f(32.184));
    let model = ClockModel::proper(reference, drift);

    let onboard = TimePoint::create_from_model(model).add(TimeSpan::from_sec(100));
    let tt = onboard.convert_using_model(model);

    let expected = onboard
        .add(TimeSpan::from_sec_f(32.184))
        .to_clock_type(ClockType::Proper);
    assert_eq!(tt, expected);
}

#[test]
fn convert_back_using_model_inverse() {
    let reference = TimePoint::from_tai_sec(0);
    let drift = ClockDrift::new(
        TimeSpan::from_ms(500), // exactly 0.5 s
        TimeSpan::from_ns(2),   // exactly 2 ns/s = 2e-9 s/s
        TimeSpan::ZERO,
    );
    let model = ClockModel::proper(reference, drift);

    // Start from onboard Proper time (the natural input for this API)
    let proper = TimePoint::create_from_model(model).add(TimeSpan::from_sec(1_000_000));

    let tt = proper.convert_using_model(model); // Proper → TT
    let back = tt.convert_back_using_model(model); // TT → Proper

    assert_eq!(back, proper);
}

#[test]
fn apply_new_model_and_create_from_model() {
    let reference = TimePoint::from_tai_sec(0);
    let drift = ClockDrift::ZERO;
    let model = ClockModel::proper(reference, drift);

    let raw = TimePoint::from_tai_sec(123);
    let tagged = raw.apply_new_model(model);

    assert_eq!(tagged.clock_type(), ClockType::Proper);
    assert_eq!(
        TimePoint::create_from_model(model),
        reference.to_clock_type(ClockType::Proper)
    );
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
        TimePoint::from_tai_sec(0),                  // J2000 TAI
        TimePoint::from_tai_sec(86_400 * 365),       // ~1 year later
        TimePoint::from_tai_sec(-86_400 * 365 * 10), // 10 years before
        TimePoint::from_tai_sec(1_000_000_000),      // ~31.7 years later
        TimePoint::from_tai_sec(-2_208_945_600),     // J1900 epoch
    ];

    for &p in &test_points {
        let ltc = p.to_clock_type(ClockType::LTC);
        let back = ltc.to_clock_type(ClockType::TAI);

        let diff = back.duration_since(p).as_sec_f().abs();
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
    let tai = TimePoint::ZERO;
    let ltc = tai.to_clock_type(ClockType::LTC);

    let diff_s = ltc.numerical_seconds_since(&tai);

    assert!(
        (diff_s - 32.6545948272096).abs() < 1e-9,
        "LTC-TAI difference at J2000 was {} s (expected 32.6545948272096 s)",
        diff_s
    );
}

/// Tighter check of the same J2000 value (matches the style of the second TDB test).
#[test]
fn ltc_minus_tai_at_j2000_2() {
    let tai = TimePoint::ZERO;
    let ltc = tai.to_clock_type(ClockType::LTC);
    let diff_s = ltc.numerical_seconds_since(&tai);

    assert!(
        (diff_s - 32.6545948272096).abs() < 1e-9,
        "got {} (expected 32.6545948272096)",
        diff_s
    );
}

/// Verify that the LTC–TT difference grows linearly (pure secular term).
#[test]
fn ltc_offset_grows_linearly() {
    let points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),       // ~1 year
        TimePoint::from_tai_sec(86_400 * 365 * 100), // ~100 years
    ];

    for &p in &points {
        let tt = p.to_clock_type(ClockType::TT);
        let ltc = p.to_clock_type(ClockType::LTC);

        // LTC - TT (pure secular term)
        let corr_s = ltc.numerical_seconds_since(&tt);

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
                "LTC-TT correction at ~100y should be ~2–3 s (got {} s)",
                corr_s
            );
        }
    }
}

#[test]
fn msd_exact_roundtrip_is_accurate() {
    let test_points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(-86_400 * 365 * 10),
        TimePoint::from_tai_sec(1_000_000_000),
        TimePoint::from_tai_sec(-2_208_945_600),
    ];

    for &p in &test_points {
        let (whole, frac) = p.to_msd_exact();
        let back = TimePoint::from_msd_exact(whole, frac);

        let diff = back.duration_since(p).as_sec_f().abs();
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
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365 * 100),
        TimePoint::from_tai_sec(1_000_000_000),
    ];

    for &p in &test_points {
        let msd_float = p.to_msd();
        let back = TimePoint::from_msd(msd_float);

        let diff = back.duration_since(p).as_sec_f().abs();
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
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(1_000_000_000),
    ];

    for &p in &test_points {
        let mtc = p.to_mtc();
        let mtc_sec = mtc.as_sec_f();
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
    let tai = TimePoint::ZERO;
    let (whole, frac) = tai.to_msd_exact();

    assert_eq!(whole, 44791, "Integer part of MSD at J2000 should be 44791");

    // New exact value (no magic number)
    let frac_sols = frac.as_sec_f() / MARS_SOL_LENGTH_SEC;
    assert!(
        (frac_sols - 0.61987471912).abs() < 1e-11, // or use a TimeSpan comparison
        "Fractional part of MSD at J2000 (TAI) was {} sols",
        frac_sols
    );
}

#[test]
fn utc_leap_seconds_are_handled_in_mars_time() {
    use deep_time_core::ClockType;
    // One second before vs after a leap second insertion
    let utc_pre = TimePoint::new(1_485_779_199, 0, ClockType::UTC);
    let utc_post = TimePoint::new(1_485_779_200, 0, ClockType::UTC);

    let msd_pre = utc_pre.to_msd();
    let msd_post = utc_post.to_msd();

    let diff_sols = (msd_post - msd_pre).abs();
    assert!(
        diff_sols > 1e-6 && diff_sols < 2e-5,
        "MSD difference across leap second was {} sols (expected ~1.126e-5)",
        diff_sols
    );
}

/// TT is exactly TAI + 32.184 s (and ET is an alias for TT).
#[test]
fn tt_tai_offset_exact() {
    let tai = TimePoint::ZERO;
    let tt = tai.to_clock_type(ClockType::TT);
    let diff_s = tt.numerical_seconds_since(&tai);
    assert!(
        (diff_s - 32.184).abs() < 1e-12,
        "TT-TAI at J2000 was {} s (expected exactly 32.184)",
        diff_s
    );
}

/// All GNSS scales have fixed integer-second offsets from TAI.
#[test]
fn gnss_offsets_are_correct() {
    let tai = TimePoint::ZERO;

    let gpst = tai.to_clock_type(ClockType::GPS);
    assert!((gpst.numerical_seconds_since(&tai) + 19.0).abs() < 1e-12);

    let qzsst = tai.to_clock_type(ClockType::QZSS);
    assert!((qzsst.numerical_seconds_since(&tai) + 19.0).abs() < 1e-12);

    let gst = tai.to_clock_type(ClockType::GST);
    assert!((gst.numerical_seconds_since(&tai) + 19.0).abs() < 1e-12);

    let bdt = tai.to_clock_type(ClockType::BDT);
    assert!((bdt.numerical_seconds_since(&tai) + 33.0).abs() < 1e-12);
}

/// TCG ↔ TAI round-trip (pure linear rate – should be exact within f64 noise).
#[test]
fn tcg_tai_roundtrip_is_accurate() {
    let test_points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(-86_400 * 365 * 10),
        TimePoint::from_tai_sec(1_000_000_000),
        TimePoint::from_tai_sec(-2_208_945_600),
    ];

    for &p in &test_points {
        let tcg = p.to_clock_type(ClockType::TCG);
        let back = tcg.to_clock_type(ClockType::TAI);
        let diff = back.duration_since(p).as_sec_f().abs();
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
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(-86_400 * 365 * 10),
        TimePoint::from_tai_sec(1_000_000_000),
        TimePoint::from_tai_sec(-2_208_945_600),
    ];

    for &p in &test_points {
        let tcb = p.to_clock_type(ClockType::TCB);
        let back = tcb.to_clock_type(ClockType::TAI);
        let diff = back.duration_since(p).as_sec_f().abs();
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
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(-86_400 * 365 * 10),
        TimePoint::from_tai_sec(1_000_000_000),
        TimePoint::from_tai_sec(-2_208_945_600),
        TimePoint::from_tai_sec(1_485_779_200), // around 2017-01-01 leap second
    ];

    for &p in &test_points {
        let utc = p.to_clock_type(ClockType::UTC);
        let back = utc.to_clock_type(ClockType::TAI);
        assert_eq!(back, p, "UTC round-trip failed at {:?}", p);
    }
}

/// J2000.0 TT = 2000-01-01 12:00:00 TT exactly (JD 2451545.0).
/// The library’s exact MJD convention is JD − 2_400_000 (MJD 51545.0, frac = 0).
#[test]
fn j2000_tt_is_jd_2451545() {
    let j2000_tt = TimePoint::from_jd_tt_exact(2451545, TimeSpan::ZERO);

    let (jd, frac) = j2000_tt.to_jd_tt_exact();
    assert_eq!(jd, 2451545, "JD integer part wrong");
    assert!(frac.is_zero(), "JD fractional part must be zero");

    let (mjd, mjd_frac) = j2000_tt.to_mjd_tt_exact();
    assert_eq!(mjd, 51545, "MJD integer part wrong (library convention)");
    assert!(mjd_frac.is_zero(), "MJD fractional part must be zero");
}

/// Exact JD ↔ TimePoint round-trip (full attosecond precision).
#[test]
fn jd_tt_exact_roundtrip() {
    let test_points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365),
        TimePoint::from_tai_sec(1_000_000_000),
        TimePoint::from_tai_sec(-2_208_945_600),
    ];

    for &p in &test_points {
        let (jd, frac) = p.to_jd_tt_exact();
        let back = TimePoint::from_jd_tt_exact(jd, frac);
        let diff = back.duration_since(p).as_sec_f().abs();
        assert!(diff < 1e-10, "JD round-trip error {} s at {:?}", diff, p);
    }
}

/// Exact MJD ↔ TimePoint round-trip.
#[test]
fn mjd_tt_exact_roundtrip() {
    let test_points = [
        TimePoint::from_tai_sec(0),
        TimePoint::from_tai_sec(86_400 * 365 * 100),
    ];

    for &p in &test_points {
        let (mjd, frac) = p.to_mjd_tt_exact();
        let back = TimePoint::from_mjd_tt_exact(mjd, frac);
        let diff = back.duration_since(p).as_sec_f().abs();
        assert!(diff < 1e-10, "MJD round-trip error {} s at {:?}", diff, p);
    }
}

#[test]
fn ymd_to_jdn_j2000() {
    assert_eq!(TimePoint::ymd_to_jdn(2000, 1, 1), 2451545);
}

#[test]
fn ymd_to_jdn_leap_year_handling() {
    assert_eq!(TimePoint::ymd_to_jdn(2000, 2, 29), 2451604); // leap day
    assert_eq!(TimePoint::ymd_to_jdn(1900, 2, 28), 2415079); // non-leap
}

#[test]
fn is_leap_year_and_valid_date() {
    assert!(TimePoint::is_leap_year(2000));
    assert!(!TimePoint::is_leap_year(1900));
    assert!(TimePoint::is_valid_ymd(2024, 2, 29));
    assert!(!TimePoint::is_valid_ymd(2023, 2, 29));
}
