#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod unified_vs_gr_tests {
    use deep_time::{Drift, Real, Spacetime, constants::C_SQUARED, f, math::sqrt};

    /// Classic GR rate (what every existing pipeline uses)
    fn classic_gr_rate(alpha: Real, beta: Real) -> Real {
        let delta = (alpha * alpha * (1.0 - beta * beta)).max(0.0);
        delta.sqrt().max(0.0)
    }

    #[test]
    fn exact_match_when_kretschmann_zero() {
        // Earth surface (stationary)
        let ls = Spacetime::new(0.9999999993, 0.0, 0.0);
        let unified = ls.proper_time_rate();
        let classic = classic_gr_rate(ls.alpha, ls.beta);
        assert!((unified - classic).abs() < 1e-300);

        // GNSS satellite (MEO)
        let ls = Spacetime::new(0.9999999995, 1.3e-5, 0.0);
        let unified = ls.proper_time_rate();
        let classic = classic_gr_rate(ls.alpha, ls.beta);
        assert!((unified - classic).abs() < 1e-300);

        // Low Earth orbit
        let ls = Spacetime::new(0.9999999992, 2.6e-5, 0.0);
        let unified = Drift::from_spacetime(&ls).proper_time_rate();
        let classic = classic_gr_rate(ls.alpha, ls.beta);
        assert!((unified - classic).abs() < 1e-300);
    }

    #[test]
    fn difference_is_insanely_small_even_with_realistic_kretschmann() {
        let ls = Spacetime::new(0.85, 0.1, 1e-20);
        let unified = ls.proper_time_rate();
        let classic = classic_gr_rate(ls.alpha, ls.beta);
        let rel_diff = (unified - classic).abs() / classic.max(1e-300);
        assert!(rel_diff < 1e-100, "rel_diff = {}", rel_diff);
    }

    #[test]
    fn strong_field_saturation_activates_at_planck_curvature() {
        let huge_kretschmann: Real = 1e150;

        let alpha = f!(0.1);
        let ls = Spacetime::new(alpha, f!(0.0), huge_kretschmann);
        let rate = ls.proper_time_rate();

        let delta = alpha * alpha;
        let expected = (delta * delta - delta + 1.0).sqrt();

        assert!(
            (rate - expected).abs() < 1e-10,
            "Expected saturation to {:.6}, got {}",
            expected,
            rate
        );
        assert!(rate > 0.99);
    }

    #[test]
    fn saturation_minimum_is_866() {
        let huge_k = 1e160;
        let ls = Spacetime::new(f!(core::f64::consts::FRAC_1_SQRT_2), f!(0.0), huge_k);
        let rate = ls.proper_time_rate();
        assert!(
            rate >= 0.866,
            "Rate dropped below theoretical minimum: {}",
            rate
        );
    }

    #[test]
    fn from_velocity_potential_and_scale_matches_classic_when_scale_zero() {
        let v = 7800.0;
        let phi = -6.26e7;

        let classic = {
            let alpha = (1.0 + 2.0 * phi / C_SQUARED).sqrt().max(0.0);
            let beta = v / 299792458.0;
            alpha * sqrt(1.0 - beta * beta) // using the imported sqrt
        };

        let drift = Drift::from_velocity_potential_and_scale(v, phi, 0.0);
        let unified = drift.proper_time_rate();

        assert!((unified - classic).abs() < 1e-300);
    }
}
