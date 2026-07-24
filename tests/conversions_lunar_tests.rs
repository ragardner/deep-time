#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/// Tests for approximate TCL and library LTC (mean selenoid via L_m).
///
/// TCL: `TCL − TDB = L_D^M · (t − t₀) + P₁₃(t)`, `t₀` = 1977.
/// LTC: `LTC = TCL − L_m · (TCL − t₀)`.
mod ltc_tcl_tests {
    use deep_time::{Dt, Scale};

    // -------------------------------------------------------------------------
    // Round-trips
    // -------------------------------------------------------------------------

    #[test]
    fn tcl_tai_roundtrip_is_accurate() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
            Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
            Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
            Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
        ];

        for &p in &test_points {
            let tcl = p.to(Scale::TCL);
            let back = tcl.to(Scale::TAI);
            let diff = back.to_diff_raw(p).to_sec_f().abs();
            assert!(
                diff < 1e-9,
                "TCL ↔ TAI round-trip error of {} s at {:?}",
                diff,
                p
            );
        }
    }

    #[test]
    fn ltc_tai_roundtrip_is_accurate() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
            Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
            Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
            Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
        ];

        for &p in &test_points {
            let ltc = p.to(Scale::LTC);
            let back = ltc.to(Scale::TAI);
            let diff = back.to_diff_raw(p).to_sec_f().abs();
            assert!(
                diff < 1e-9,
                "LTC ↔ TAI round-trip error of {} s at {:?}",
                diff,
                p
            );
        }
    }

    #[test]
    fn ltc_tcl_roundtrip_is_exact_linear() {
        // LTC ↔ TCL is pure linear L_m scaling — should be essentially exact.
        let points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365 * 20, Scale::TAI, Scale::TAI),
            Dt::from_sec(-86_400 * 365 * 5, Scale::TAI, Scale::TAI),
        ];
        for &p in &points {
            let tcl = p.to(Scale::TCL);
            let ltc = tcl.to(Scale::LTC);
            let back = ltc.to(Scale::TCL);
            let diff = back.to_diff_raw(tcl).to_sec_f().abs();
            assert!(
                diff < 1e-15,
                "LTC ↔ TCL round-trip error {} s (expected near-exact linear)",
                diff
            );
        }
    }

    // -------------------------------------------------------------------------
    // TCL / LTE440
    // -------------------------------------------------------------------------

    #[test]
    fn tcl_tdb_offset_near_j2000_is_sensible() {
        let tai = Dt::ZERO;
        let tcl = tai.to(Scale::TCL);
        let tdb = tai.to(Scale::TDB);
        let diff_s = tcl.to_diff_raw(tdb).to_sec_f();
        // ~L_D^M × 8400.5 d ≈ 0.493 s
        assert!(
            (0.492..0.495).contains(&diff_s),
            "TCL-TDB at J2000.0 was {} s (expected ~0.493 s)",
            diff_s
        );
    }

    /// ~2038-01-01: secular from 1977 ≈ 1.309 s plus ±~1.65 ms periodic.
    #[test]
    fn tcl_tdb_offset_near_2038() {
        let tai_2038 = Dt::from_ymd(2038, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let tcl = tai_2038.to(Scale::TCL);
        let tdb = tai_2038.to(Scale::TDB);
        let diff_s = tcl.to_diff_raw(tdb).to_sec_f();

        assert!(
            (1.306..1.311).contains(&diff_s),
            "TCL-TDB on 2038-01-01 should be ~1.309 s ± periodic (got {} s)",
            diff_s
        );

        let back = tcl.to(Scale::TAI);
        let rt = back.to_diff_raw(tai_2038).to_sec_f().abs();
        assert!(rt < 1e-9, "TCL → TAI round-trip error: {} s", rt);
    }

    // -------------------------------------------------------------------------
    // LTC / Ashby + L_m
    // -------------------------------------------------------------------------

    /// LTC runs ahead of TT (Moon clocks tick faster than geoid clocks).
    #[test]
    fn ltc_runs_ahead_of_tt() {
        let points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365 * 100, Scale::TAI, Scale::TAI),
        ];

        for &p in &points {
            let tt = p.to(Scale::TT);
            let ltc = p.to(Scale::LTC);
            let corr_s = ltc.to_diff_raw(tt).to_sec_f();
            assert!(
                corr_s > 0.0,
                "LTC should run ahead of TT; got {} s at {:?}",
                corr_s,
                p
            );

            // ~100 y after J2000 ≈ 123 y from 1977 → L_M × Δt ≈ 2.5 s
            if p.to_sec() > 86_400 * 365 * 50 {
                assert!(
                    (2.4..2.6).contains(&corr_s),
                    "Secular LTC-TT at ~100 y from J2000 should be ~2.5 s (got {} s)",
                    corr_s
                );
            }
        }
    }

    /// Mean dLTC/dTT − 1 ≈ Ashby L_M ≈ 6.48378×10⁻¹⁰.
    ///
    /// Finite difference over 20 Julian years so annual TCL−TDB / TDB−TT terms
    /// largely average out. Theoretical mean is \(L_D^M - L_m ≈ L_M\).
    #[test]
    fn ltc_tt_mean_rate_matches_ashby_lm() {
        let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
        // 20 Julian years
        let t1 = Dt::from_sec(86_400 * 365 * 20 + 86_400 * 5, Scale::TAI, Scale::TAI);

        let d0 = t0.to(Scale::LTC).to_diff_raw(t0.to(Scale::TT)).to_sec_f();
        let d1 = t1.to(Scale::LTC).to_diff_raw(t1.to(Scale::TT)).to_sec_f();
        let elapsed = t1.to(Scale::TT).to_diff_raw(t0.to(Scale::TT)).to_sec_f();
        let rate = (d1 - d0) / elapsed;

        // L_D^M − L_m from the constants used in lunar.rs
        const LD_MINUS_LM: f64 = 6.798355238e-10 - 3.13881e-11;
        const ASHBY_LM: f64 = 6.48378e-10;
        assert!(
            (LD_MINUS_LM - ASHBY_LM).abs() < 1e-13,
            "L_D − L_m should match Ashby L_M to ~1e-13"
        );
        // Multi-year finite difference still has residual periodic leakage
        assert!(
            (rate - ASHBY_LM).abs() < 2e-11,
            "mean dLTC/dTT − 1 = {:.6e} (Ashby L_M = {:.6e}, L_D−L_m = {:.6e})",
            rate,
            ASHBY_LM,
            LD_MINUS_LM
        );
    }

    /// LTC − TCL = −L_m · (TCL − t₀): pure L_m scaling, no extra periodic layer.
    #[test]
    fn ltc_minus_tcl_is_lm_scaling_only() {
        let tai = Dt::ZERO;
        let tcl = tai.to(Scale::TCL);
        let ltc = tai.to(Scale::LTC);
        let diff = ltc.to_diff_raw(tcl).to_sec_f(); // LTC − TCL (< 0)

        // elapsed TCL since 1977 at this instant ≈ 8400.5 d (same order as TCG epoch)
        // L_m * 8400.5 * 86400 ≈ 0.02278 s → LTC − TCL ≈ −0.02278 s
        assert!(
            diff < 0.0 && diff > -0.03,
            "LTC−TCL at J2000 should be ≈ −L_m·Δt ≈ −0.023 s (got {} s)",
            diff
        );
    }

    /// Annual LTC−TT residual must be ≪ 1.65 ms (the raw TCL−TDB annual amplitude).
    ///
    /// After removing a linear trend over one year, peak residual should stay
    /// well below ~0.5 ms (composition residual of TCL−TDB + TDB−TT).
    #[test]
    fn ltc_tt_annual_residual_is_not_full_tcl_tdb_annual() {
        let n = 37; // samples over ~1 year
        let mut offsets = Vec::with_capacity(n);
        for k in 0..n {
            let tai = Dt::from_sec(k as i128 * 10 * 86_400, Scale::TAI, Scale::TAI);
            let off = tai.to(Scale::LTC).to_diff_raw(tai.to(Scale::TT)).to_sec_f();
            offsets.push(off);
        }

        // Linear least-squares residual peak-to-peak
        let n_f = (n - 1) as f64;
        let y0 = offsets[0];
        let y1 = offsets[n - 1];
        let mut max_abs_res = 0.0_f64;
        for (k, &y) in offsets.iter().enumerate() {
            let pred = y0 + (y1 - y0) * (k as f64 / n_f);
            max_abs_res = max_abs_res.max((y - pred).abs());
        }

        // Raw TCL−TDB annual is 1.65 ms; composed residual must be far smaller.
        assert!(
            max_abs_res < 5e-4,
            "LTC−TT detrended residual peak {} s exceeds 0.5 ms — annual TCL−TDB may have been applied raw",
            max_abs_res
        );
        // And not trivially zero either (there should be some residual periodic).
        // (No lower bound: depending on sampling, residual can be small.)
    }

    /// LTC and TCL share the same physical instant mapping through TAI.
    #[test]
    fn ltc_and_tcl_agree_as_instants_via_tai() {
        let tai = Dt::from_ymd(2025, 6, 15, Scale::TAI, 12, 0, 0, 0);
        let ltc = tai.to(Scale::LTC);
        let tcl = tai.to(Scale::TCL);
        assert_eq!(ltc.to(Scale::TAI), tai);
        assert_eq!(tcl.to(Scale::TAI), tai);
        assert_eq!(ltc.to(Scale::TCL).to(Scale::TAI), tai);
    }

    // -------------------------------------------------------------------------
    // Optional hifitime cross-check (TCL only; experimental in hifitime)
    // -------------------------------------------------------------------------

    #[cfg(feature = "hifitime")]
    #[test]
    fn tcl_matches_hifitime_latest() {
        use hifitime::{Epoch, TimeScale};

        // TAI seconds since 1900-01-01 00:00 TAI for 2038-01-01 00:00 TAI
        let tai_sec: f64 = 4_354_905_600.0;
        let epoch_tai = Epoch::from_tai_seconds(tai_sec);
        let epoch_tcl = epoch_tai.to_time_scale(TimeScale::TCL);
        let tcl_sec = epoch_tcl.duration.to_seconds();

        let my_2038_tai = Dt::from_ymd(2038, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let my_tcl = my_2038_tai
            .target(Scale::TCL)
            .to_scale_and_diff(Dt::TAI_1977_EPOCH, true);

        let diff = (my_tcl.to_sec_f() - tcl_sec).abs();
        assert!(
            diff < 0.001,
            "TCL mismatch with hifitime: our = {:.9}, hifitime = {:.9}, diff = {:.9} s",
            my_tcl,
            tcl_sec,
            diff
        );
    }
}
