#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod tests {
    use deep_time::macros::{dt, from_sec_f};
    use deep_time::physics::{Drift, Spacetime};
    use deep_time::{Dt, Scale};

    #[test]
    fn evaluate_zero_drift() {
        let drift = Drift::ZERO;
        let dt = Dt::from_sec(1_234_567, Scale::TAI, Scale::TAI);
        assert_eq!(drift.time_diff_after(&dt), Dt::ZERO);
    }

    #[test]
    fn evaluate_constant_only() {
        let drift = Drift::from_constant(from_sec_f!(0.5));
        let dt = Dt::from_sec(1_000, Scale::TAI, Scale::TAI);
        assert_eq!(drift.time_diff_after(&dt), from_sec_f!(0.5));
    }

    #[test]
    fn evaluate_rate_only() {
        let drift = Drift::from_offset_and_rate(Dt::ZERO, from_sec_f!(1e-9)); // 1 ns/s
        let dt = Dt::from_sec(1_000_000, Scale::TAI, Scale::TAI); // 1 million seconds
        assert_eq!(drift.time_diff_after(&dt), from_sec_f!(0.001)); // 1 µs
    }

    #[test]
    fn evaluate_full_quadratic() {
        let drift = Drift::new(
            Dt::from_sec(2, Scale::TAI, Scale::TAI),
            Dt::from_ns(1, 0, Scale::TAI, Scale::TAI), // exactly 1e-9 s/s
            dt!(2),                                    // exactly 2e-18 s/s²
        );
        let dt = Dt::from_sec(1_000_000, Scale::TAI, Scale::TAI);

        // Exact mathematical result:
        // 2 + (1e-9 * 1_000_000) + (2e-18 * 1_000_000²) = 2 + 0.001 + 0.000002
        // = 2.001002 s = 2 s + 1_002_000_000_000_000 attoseconds
        assert_eq!(
            drift.time_diff_after(&dt),
            dt!(2_001_002_000_000_000_000i128)
        );
    }

    #[test]
    fn evaluate_negative_dt() {
        let drift = Drift::new(
            Dt::from_sec(5, Scale::TAI, Scale::TAI),
            Dt::from_ns(1, 0, Scale::TAI, Scale::TAI), // exactly 1e-9 s/s
            Dt::new(1, Scale::TAI, Scale::TAI),        // exactly 1e-18 s/s²
        );
        let dt = Dt::from_sec(-500_000, Scale::TAI, Scale::TAI);

        // Exact mathematical result (no f64 loss)
        let expected = Dt::from_sec(4, Scale::TAI, Scale::TAI)
            .add(Dt::from_ms(999, 0, Scale::TAI, Scale::TAI))
            .add(Dt::from_us(500, 0, Scale::TAI, Scale::TAI))
            .add(Dt::from_ns(250, 0, Scale::TAI, Scale::TAI));

        assert_eq!(drift.time_diff_after(&dt), expected);
    }

    #[test]
    fn evaluate_large_dt_exact() {
        let drift = Drift::from_offset_and_rate(Dt::ZERO, from_sec_f!(1e-12));
        let dt = Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI); // ~31.7 years
        assert_eq!(drift.time_diff_after(&dt), from_sec_f!(0.001));
    }

    /// Intermediate `rate_attos * span_attos` can exceed i128 even when the
    /// scaled result fits; mul-div must not wrap or early-saturate.
    #[test]
    fn evaluate_rate_with_overflowing_intermediate_product() {
        // rate = 1 s/s, span = 1_000_000 s
        let drift = Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec(1, Scale::TAI, Scale::TAI));
        let span = Dt::from_sec(1_000_000, Scale::TAI, Scale::TAI);
        assert_eq!(
            drift.time_diff_after(&span),
            Dt::from_sec(1_000_000, Scale::TAI, Scale::TAI)
        );

        let drift_neg =
            Drift::from_offset_and_rate(Dt::ZERO, Dt::from_sec(-1, Scale::TAI, Scale::TAI));
        assert_eq!(
            drift_neg.time_diff_after(&span),
            Dt::from_sec(-1_000_000, Scale::TAI, Scale::TAI)
        );
    }

    // ========================================================================
    // Proper-time rate: dτ/dt = α √(1 − β²)
    // ========================================================================

    #[test]
    fn proper_time_rate_matches_interval() {
        let cases: &[(f64, f64, f64)] = &[
            (1.0, 0.0, 1.0),  // stationary flat space
            (1.0, 0.6, 0.8),  // β = 0.6, α = 1
            (0.9, 0.0, 0.9),  // α = 0.9, β = 0
            (0.9, 0.6, 0.72), // α = 0.9, β = 0.6
            (0.0, 0.0, 0.0),  // null / lightlike edge
            (1.1, 0.0, 1.1),  // α > 1
        ];

        for &(alpha, beta, expected_rate) in cases {
            let st = Spacetime::new(alpha, beta);
            let drift = Drift::from_spacetime(&st);
            assert!(
                (st.proper_time_rate() - expected_rate).abs() < 1e-12,
                "rate {} vs {expected_rate} for α={alpha} β={beta}",
                st.proper_time_rate()
            );
            assert!(
                (drift.proper_time_rate() - expected_rate).abs() < 1e-12,
                "Drift rate {} vs {expected_rate} for α={alpha} β={beta}",
                drift.proper_time_rate()
            );
        }
    }

    #[test]
    fn flat_spacetime_has_zero_rate_offset() {
        let drift = Drift::from_spacetime(&Spacetime::new(1.0, 0.0));
        assert_eq!(drift.rate, Dt::ZERO);
        assert_eq!(drift.proper_time_rate(), 1.0);
    }

    #[test]
    fn superluminal_beta_clamps_rate_to_zero() {
        let st = Spacetime::new(1.0, 2.0);
        assert_eq!(st.proper_time_rate(), 0.0);
        assert_eq!(Drift::from_spacetime(&st).rate.to_sec_f(), -1.0);
    }

    #[test]
    fn spacetime_to_drift_uses_stable_offset() {
        let spacetime = Spacetime::new(0.9, 0.6);
        let drift = Drift::from_spacetime(&spacetime);
        let delta: f64 = 0.9 * 0.9 * (1.0 - 0.6 * 0.6);
        let expected_offset = (delta - 1.0) / (delta.sqrt() + 1.0);
        let expected = Drift::from_offset_and_rate(Dt::ZERO, from_sec_f!(expected_offset));
        assert_eq!(drift, expected);
    }
}
