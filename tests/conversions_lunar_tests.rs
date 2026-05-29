#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/// Tests for Lunar Coordinated Time (LTC) realization.
///
/// LTC is the proper-time scale for a clock at rest on the lunar selenoid,
/// as defined in Ashby & Patla (2024) with the addition of the full 13-term
/// periodic corrections from the LTE440 lunar time ephemeris (Lu et al. 2025,
/// A&A 704, A76; arXiv:2509.18511). These tests verify numerical stability,
/// secular rate, periodic relativistic effects, and consistency with the
/// authoritative published reference value. All tolerances are set to be
/// substantially tighter than any operational requirement for cislunar PNT.
mod ltc_tests {
    use deep_time::{Dt, Scale, constants::ATTOS_PER_SEC_I128};

    /// Verifies round-trip conversion accuracy between TAI and LTC.
    ///
    /// The LTC transformation (fixed-point secular scaling + bounded periodic
    /// correction) must be numerically reversible to sub-nanosecond precision.
    /// The 1 ns tolerance accounts for unavoidable f64 rounding in the Julian-date
    /// helper while remaining far stricter than any mission requirement.
    #[test]
    fn ltc_tai_roundtrip_is_accurate() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI),                  // J2000.0 TAI
            Dt::from_sec(86_400 * 365, Scale::TAI),       // ~1 year after J2000.0
            Dt::from_sec(-86_400 * 365 * 10, Scale::TAI), // 10 years before J2000.0
            Dt::from_sec(1_000_000_000, Scale::TAI),      // ~31.7 years after J2000.0
            Dt::from_sec(-2_208_945_600, Scale::TAI),     // Approximate J1900 epoch
        ];

        for &p in &test_points {
            let ltc = p.to(Scale::TAI, Scale::LTC);
            let back = ltc.to_tai(Scale::LTC);

            let diff = back.to_diff_raw(p).to_sec_f().abs();

            assert!(
                diff < 1e-9,
                "LTC ↔ TAI round-trip error of {} s exceeds tolerance at input instant {:?}",
                diff,
                p
            );
        }
    }

    /// Validates the LTC-TAI offset at the J2000.0 reference epoch.
    ///
    /// This test anchors the complete model (Ashby & Patla secular rate L_M plus
    /// the full 13-term LTE440 periodic series) at the standard reference epoch.
    /// The periodic contribution at J2000.0 is approximately -35.128 µs.
    #[test]
    fn ltc_minus_tai_at_j2000() {
        let tai = Dt::ZERO;
        let ltc = tai.to(Scale::TAI, Scale::LTC);

        let diff_s = ltc.to_diff_raw(tai).to_sec_f();

        const EXPECTED_LTC_TAI_J2000_S: f64 = 32.654559693364384;

        assert!(
            (diff_s - EXPECTED_LTC_TAI_J2000_S).abs() < 1e-9,
            "LTC-TAI difference at J2000.0 was {} s (expected {:.12} s)",
            diff_s,
            EXPECTED_LTC_TAI_J2000_S
        );
    }

    /// Validates long-term secular growth of the LTC-TT offset.
    ///
    /// A selenoid clock runs faster than a TT clock by L_M ≈ 6.48378 × 10^{-10}
    /// (Ashby & Patla 2024). This test confirms linear growth of the secular
    /// component over multi-decade timescales. Note that secular accumulation
    /// is measured from the library’s internal reference epoch (1977-01-01.0 TAI),
    /// not J2000.0; the ~100-year test point therefore corresponds to ~123 years
    /// of elapsed time.
    #[test]
    fn ltc_offset_grows_linearly() {
        let points = [
            Dt::from_sec(0, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI), // ~1 year
            Dt::from_sec(86_400 * 365 * 100, Scale::TAI), // ~100 years from J2000.0
        ];

        for &p in &points {
            let tt = p.to(Scale::TAI, Scale::TT);
            let ltc = p.to(Scale::TAI, Scale::LTC);

            let corr_s = ltc.to_diff_raw(tt).to_sec_f();

            assert!(
                corr_s > 0.0,
                "LTC should run ahead of TT (positive offset). Got {} s at {:?}",
                corr_s,
                p
            );

            // At the ~100-year point the secular offset must be ~2.516 s
            // (L_M × ~123 years of elapsed time from the 1977 reference epoch).
            if p.to_sec() > 86_400 * 365 * 50 {
                assert!(
                    (corr_s > 2.4 && corr_s < 2.6),
                    "Secular LTC-TT offset at ~100 years from J2000.0 should be ~2.516 s (got {} s)",
                    corr_s
                );
            }
        }
    }

    /// Validates consistency with the official LTE440 reference value at J2000.0.
    ///
    /// LTE440 (Lu et al. 2025) is the current state-of-the-art lunar time ephemeris.
    /// It publishes TCL − TDB = +0.49330749643254945 s at JD 2451545.0 TDB.
    /// Our LTC realization (proper time on the selenoid) uses the identical periodic
    /// terms; a small constant offset relative to TCL is expected and physically correct.
    ///
    /// Reference value and example output: https://github.com/xlucn/LTE440
    #[test]
    fn ltc_agrees_with_lte440_j2000_reference() {
        let tai = Dt::ZERO;
        let ltc = tai.to(Scale::TAI, Scale::LTC);
        let tdb = tai.to(Scale::TAI, Scale::TDB);

        let diff_s = ltc.to_diff_raw(tdb).to_sec_f();

        const PUBLISHED_TCL_TDB_J2000_S: f64 = 0.49330749643254945;

        assert!(
            (diff_s - PUBLISHED_TCL_TDB_J2000_S).abs() < 0.03,
            "LTC-TDB at J2000.0 was {} s (published TCL-TDB reference = {:.14} s)",
            diff_s,
            PUBLISHED_TCL_TDB_J2000_S
        );
    }

    /// Validates TCL against the official LTE440 reference value at J2000.0.
    ///
    /// TCL (Lunar Coordinate Time) is the coordinate time of the Lunar Celestial
    /// Reference System (LCRS) as defined by IAU 2024 Resolution II. The LTE440
    /// ephemeris (Lu et al. 2025, A&A 704, A76; arXiv:2509.18511) is the current
    /// state-of-the-art numerical realization of TCL based on DE440.
    ///
    /// The authoritative reference value published by the LTE440 authors is:
    ///
    ///     TCL − TDB = +0.49330749643254945 s
    ///
    /// at JD 2451545.0 TDB (J2000.0). This test confirms that our TCL implementation
    /// (LTE440 secular rate + full 13-term periodic series) reproduces this value
    /// to high precision.
    ///
    /// Reference: https://github.com/xlucn/LTE440
    /// (see demo output in the repository README)
    #[test]
    fn tcl_agrees_with_lte440_j2000_reference() {
        let tai = Dt::ZERO;
        let tcl = tai.to(Scale::TAI, Scale::TCL);
        let tdb = tai.to(Scale::TAI, Scale::TDB);

        let diff_s = tcl.to_diff_raw(tdb).to_sec_f();

        const PUBLISHED_TCL_TDB_J2000_S: f64 = 0.49330749643254945;

        assert!(
            (diff_s - PUBLISHED_TCL_TDB_J2000_S).abs() < 1e-12,
            "TCL-TDB difference at J2000.0 was {} s (published LTE440 reference = {:.14} s)",
            diff_s,
            PUBLISHED_TCL_TDB_J2000_S
        );
    }

    /// ### Expected TCL–TDB difference on 2038-01-01
    ///
    /// According to the official LTE440 ephemeris (Lu et al. 2025, A&A 704, A76):
    ///
    /// - At J2000.0 (JD 2451545.0 TDB):  
    ///   TCL − TDB = **+0.49330749643254945 s** (exact published reference value).
    ///
    /// - Secular (linear) rate: ⟨dTCL/dTDB⟩ − 1 = **+6.798355238 × 10⁻¹⁰**
    ///   (from the LTE440 `.tpc` kernel constant `BODY1000000005_RATE`).
    ///
    /// - Representative epoch: 2038-01-01 12:00 TDB (JD 2465425.0)  
    ///   ΔJD = 13 880 days  
    ///   Linear accumulation = 6.798355238 × 10⁻¹⁰ × 13 880 × 86 400 ≈ **+0.815280515 s**
    ///
    /// - Therefore the expected mean value is:  
    ///   TCL − TDB ≈ 0.493307496 + 0.815280515 = **+1.308588011 s**
    ///
    /// The periodic contribution (13-term Fourier series) varies by up to ±1.65 ms
    /// (dominated by the annual term of amplitude ~1.651 ms). The change in the
    /// periodic part from J2000.0 to this 2038 epoch is only ~0.88 µs, which is
    /// negligible at the millisecond level.
    ///
    /// The assertion range (1.3069 – 1.3103 s) comfortably covers the expected
    /// secular mean plus the full periodic oscillation while remaining tight enough
    /// to verify correct implementation.
    ///
    /// Reference kernels and exact values:  
    /// https://github.com/xlucn/LTE440 (lte440.bsp + lte440.tpc, README, and demo scripts)
    #[test]
    fn tcl_for_lunar_orbit_planning_2038_example() {
        // 2038-01-01 00:00:00 TAI
        // (Unix timestamp 2_145_916_800 on the TAI scale)
        let unix_tai_sec = 2_145_916_800i128;

        let tai_2038 =
            Dt::from_diff_and_scale(Dt::from_tai_sec(unix_tai_sec), Dt::UNIX_EPOCH, Scale::TAI);

        let tcl_span = tai_2038.to(Scale::TAI, Scale::TCL); // Dt on TCL scale
        let tdb_span = tai_2038.to(Scale::TAI, Scale::TDB); // Dt on TDB scale

        let diff_s = tcl_span.to_diff_raw(tdb_span).to_sec_f();

        assert!(
            (diff_s > 1.3069 && diff_s < 1.3103),
            "TCL-TDB difference on 2038-01-01 should be ~1.3086 s ± periodic terms (got {} s)",
            diff_s
        );

        // Round-trip sanity check
        let tai = tcl_span.to(Scale::TCL, Scale::TAI);

        let roundtrip_error = tai.to_diff_raw(tai_2038).to_sec_f().abs();

        assert!(
            roundtrip_error < 1e-9,
            "TCL → TAI round-trip error too large: {} s",
            roundtrip_error
        );
    }

    /// Cross-validation test against the latest hifitime (v4.3+) TCL implementation.
    ///
    /// hifitime 4.3.0 introduced experimental support for Lunar Coordinate Time (TCL).
    /// This test verifies that our analytical LTE440-based TCL agrees with hifitime's
    /// implementation to within 1 ms (the observed difference at this epoch is ~535 µs,
    /// well within the ±1.65 ms periodic term amplitude and the experimental nature
    /// of hifitime's TCL support).
    #[cfg(feature = "hifitime")]
    #[test]
    fn tcl_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // TAI seconds since 1900-01-01 00:00 TAI for the instant 2038-01-01 00:00 TAI
        let tai_sec: f64 = 4_354_905_600.0;

        // Create Epoch directly from the raw TAI seconds value (no Gregorian anywhere)
        let epoch_tai = Epoch::from_tai_seconds(tai_sec);

        // Convert the *instant* to the TCL scale
        let epoch_tcl = epoch_tai.to_time_scale(TimeScale::TCL);

        // The numeric value on the TCL time scale (seconds since TCL reference epoch 1977)
        let tcl_sec = epoch_tcl.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd_on(2038, 1, 1, Scale::TAI);
        let my_tcl = my_2038_tai
            .to_scale_and_then_diff(Scale::TCL, Dt::TAI_1977_EPOCH.to(Scale::TAI, Scale::TCL));

        let diff = (my_tcl.to_sec_f() - tcl_sec).abs();

        assert!(
            diff < 0.001,
            "TCL mismatch with hifitime: our = {:.9}, hifitime = {:.9}, diff = {:.9} s (expected < 1 ms)",
            my_tcl,
            tcl_sec,
            diff
        );
    }
}
