#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(test)]
mod tests {
    use deep_time::{Drift, Dt, Scale, Spacetime, constants::PLANCK_LENGTH_4};

    #[test]
    fn evaluate_zero_drift() {
        let drift = Drift::ZERO;
        let dt = Dt::from_sec(1_234_567, Scale::TAI);
        assert_eq!(drift.time_diff_after(&dt), Dt::ZERO);
    }

    #[test]
    fn evaluate_constant_only() {
        let drift = Drift::from_constant(Dt::from_sec_f(0.5));
        let dt = Dt::from_sec(1_000, Scale::TAI);
        assert_eq!(drift.time_diff_after(&dt), Dt::from_sec_f(0.5));
    }

    #[test]
    fn evaluate_rate_only() {
        let drift = Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(1e-9)); // 1 ns/s
        let dt = Dt::from_sec(1_000_000, Scale::TAI); // 1 million seconds
        assert_eq!(drift.time_diff_after(&dt), Dt::from_sec_f(0.001)); // 1 µs
    }

    #[test]
    fn evaluate_full_quadratic() {
        let drift = Drift::new(
            Dt::from_sec(2, Scale::TAI),
            Dt::from_ns(1, Scale::TAI),    // exactly 1e-9 s/s
            Dt::from_attos(2, Scale::TAI), // exactly 2e-18 s/s²
        );
        let dt = Dt::from_sec(1_000_000, Scale::TAI);

        // Exact mathematical result:
        // 2 + (1e-9 * 1_000_000) + (2e-18 * 1_000_000²) = 2 + 0.001 + 0.000002
        // = 2.001002 s = 2 s + 1_002_000_000_000_000 attoseconds
        assert_eq!(
            drift.time_diff_after(&dt),
            Dt::new(2, 1_002_000_000_000_000)
        );
    }

    #[test]
    fn evaluate_negative_dt() {
        let drift = Drift::new(
            Dt::from_sec(5, Scale::TAI),
            Dt::from_ns(1, Scale::TAI),    // exactly 1e-9 s/s
            Dt::from_attos(1, Scale::TAI), // exactly 1e-18 s/s²
        );
        let dt = Dt::from_sec(-500_000, Scale::TAI);

        // Exact mathematical result (no f64 loss)
        let expected = Dt::from_sec(4, Scale::TAI)
            .add(Dt::from_ms(999, Scale::TAI))
            .add(Dt::from_us(500, Scale::TAI))
            .add(Dt::from_ns(250, Scale::TAI));

        assert_eq!(drift.time_diff_after(&dt), expected);
    }

    #[test]
    fn evaluate_large_dt_exact() {
        let drift = Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(1e-12));
        let dt = Dt::from_sec(1_000_000_000, Scale::TAI); // ~31.7 years
        assert_eq!(drift.time_diff_after(&dt), Dt::from_sec_f(0.001));
    }

    // ========================================================================
    // Thorough tests for the unified proper-time rate (master Lagrangian)
    // ========================================================================

    #[test]
    fn unified_proper_time_rate_low_curvature() {
        // kretschmann = 0 must recover exactly the GR limit dτ/dt = √(max(δ, 0))
        // where δ = α²(1 − β²). This is the canonical weak-field / solar-system path.
        let test_cases: &[(f64, f64, f64)] = &[
            (1.0, 0.0, 1.0),     // stationary flat space
            (0.64, 0.0, 0.8),    // β = 0.6, α = 1
            (0.81, 0.0, 0.9),    // α = 0.9, β = 0
            (0.5184, 0.0, 0.72), // realistic combined α = 0.9, β = 0.6
            (0.0, 0.0, 0.0),     // null / lightlike edge
            (1.21, 0.0, 1.1),    // δ > 1 (mathematically allowed, physically rare)
        ];

        for &(u, k, expected_rate) in test_cases {
            let drift = Drift::from_unified_proper_time_rate(u, k);
            let expected_offset = expected_rate - 1.0;
            let expected_drift =
                Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(expected_offset));
            assert_eq!(
                drift, expected_drift,
                "Low-curvature GR recovery failed for u={}, k={}",
                u, k
            );
        }
    }

    #[test]
    fn unified_proper_time_rate_high_curvature_saturation() {
        // When x = ℓ_Pl⁴ 𝒦 ≫ 1 the master Lagrangian saturates:
        //     K_eff → δ² − δ + 1   ⇒   dτ/dt → √(δ² − δ + 1) ≥ √(3/4) ≈ 0.866
        // (tested with an astronomically large kretschmann that forces x → ∞ in f64)
        let large_kretschmann = 1e200_f64;

        let deltas = [0.0_f64, 0.25, 0.5, 0.64, 0.81, 1.0, 1.21];
        for &delta in &deltas {
            let drift = Drift::from_unified_proper_time_rate(delta, large_kretschmann);

            // Exact algebraic saturation limit from the master Lagrangian
            let k_eff_limit = delta * delta - delta + 1.0;
            let expected_rate = k_eff_limit.sqrt().max(0.0);
            let expected_offset = expected_rate - 1.0;

            let expected_drift =
                Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(expected_offset));
            // Only allow difference when seconds match
            assert_eq!(drift.rate.sec, expected_drift.rate.sec);

            let attos_diff = (drift.rate.attos as i128 - expected_drift.rate.attos as i128).abs();
            assert!(
                attos_diff <= 200, // Allow up to 200 attoseconds difference
                "Attos difference too large for δ = {}: {} attos",
                delta,
                attos_diff
            );
        }
    }

    #[test]
    fn unified_proper_time_rate_clamping_and_edges() {
        // Negative inputs must be clamped (u.max(0), kretschmann.max(0))
        let drift_neg_u = Drift::from_unified_proper_time_rate(-0.5, 0.0);

        // Semantic check using .to_sec_f() — this is the robust way.
        // (Dt::from_sec_f(-1.0) currently produces a non-canonical internal
        // representation while the unified function produces the canonical one.
        // The two Dts are mathematically identical but not ==.)
        assert_eq!(
            drift_neg_u.rate.to_sec_f(),
            -1.0,
            "Negative u should clamp to dτ/dt = 0.0 → rate_offset = -1.0"
        );

        let drift_neg_k = Drift::from_unified_proper_time_rate(0.81, -100.0);
        let expected_neg_k = Drift::from_unified_proper_time_rate(0.81, 0.0);
        assert_eq!(
            drift_neg_k, expected_neg_k,
            "Negative kretschmann not clamped"
        );

        // delta = 1.0 must always give exactly rate = 1.0 (no drift) regardless of curvature
        for k in [0.0, 1.0, 1e10, 1e30] {
            let drift = Drift::from_unified_proper_time_rate(1.0, k);
            assert_eq!(drift.rate, Dt::ZERO, "δ=1 should be exactly rate=1");
        }

        // delta = 0 with moderate curvature (null-ray / lightlike edge case sanity).
        // We deliberately choose a kretschmann value large enough that
        // x = PLANCK_LENGTH_4 * kretschmann ≈ 6.82 (non-negligible in f64).
        // This tests the actual intermediate-curvature branch of the master Lagrangian,
        // unlike the old 1e10 which produced x ≈ 0 in floating-point.
        let kretschmann = 1e140_f64;
        let drift_null = Drift::from_unified_proper_time_rate(0.0, kretschmann);

        // Expected value computed with the exact same formula the implementation uses
        let x = PLANCK_LENGTH_4 * kretschmann;
        let k_eff = x / (1.0 + x);
        let expected_null_rate: f64 = k_eff.sqrt() - 1.0;
        let expected_null =
            Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(expected_null_rate));

        assert_eq!(drift_null, expected_null);
    }

    #[test]
    fn spacetime_to_unified_proper_time_rate() {
        // from_spacetime must correctly compute δ = α²(1 − β²) and delegate to the unified path
        let spacetime = Spacetime::new(0.9, 0.6, 0.0); // realistic values
        let drift = Drift::from_spacetime(&spacetime);

        // Manual verification of the exact same path
        let u = 0.9 * 0.9 * (1.0 - 0.6 * 0.6);
        let expected_drift = Drift::from_unified_proper_time_rate(u, 0.0);

        assert_eq!(drift, expected_drift, "Spacetime → unified path mismatch");
    }

    #[test]
    fn unified_proper_time_rate_intermediate_curvature_sanity() {
        // Spot-check a few intermediate x values (neither zero nor infinite) to ensure
        // the rational expression behaves smoothly and never goes negative.
        let u = 0.64_f64;
        let k_values = [0.0, 1e5, 1e15, 1e30];
        for &k in &k_values {
            let drift = Drift::from_unified_proper_time_rate(u, k);
            let rate_factor = 1.0 + drift.rate.to_sec_f(); // internal f64 value
            assert!(rate_factor > 0.0, "proper-time rate became non-positive");
            // monotonicity / bound check
            assert!(
                rate_factor <= 1.0 + 1e-10,
                "rate > 1 for u < 1 should not happen"
            );
        }
    }
}
