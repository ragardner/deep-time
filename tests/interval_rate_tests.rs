//! Proper-time rate tests: interval vs weak-field Φ, v fill (`physics` feature).

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod interval_rate_tests {
    use deep_time::physics::{Drift, Spacetime, Velocity};
    use deep_time::{Real, consts::C_SQUARED, math::sqrt};

    fn interval_rate(alpha: Real, beta: Real) -> Real {
        let delta = (alpha * alpha * (1.0 - beta * beta)).max(0.0);
        delta.sqrt().max(0.0)
    }

    #[test]
    fn spacetime_rate_matches_interval() {
        let ls = Spacetime::new(0.9999999993, 0.0);
        assert!((ls.proper_time_rate() - interval_rate(ls.alpha, ls.beta)).abs() < 1e-15);

        let ls = Spacetime::new(0.9999999995, 1.3e-5);
        assert!((ls.proper_time_rate() - interval_rate(ls.alpha, ls.beta)).abs() < 1e-15);

        let ls = Spacetime::new(0.9999999992, 2.6e-5);
        let via_drift = Drift::from_spacetime(&ls).proper_time_rate();
        assert!((via_drift - interval_rate(ls.alpha, ls.beta)).abs() < 1e-15);
    }

    #[test]
    fn from_potential_and_velocity_matches_interval() {
        let v = 7800.0;
        let phi = -6.26e7;

        let classic = {
            let alpha = (1.0 + 2.0 * phi / C_SQUARED).sqrt().max(0.0);
            let beta = v / 299792458.0;
            alpha * sqrt(1.0 - beta * beta)
        };

        let st = Spacetime::from_potential_and_velocity(phi, Velocity::from_speed(v));
        let drift = Drift::from_spacetime(&st);
        assert!((st.proper_time_rate() - classic).abs() < 1e-15);
        assert!((drift.proper_time_rate() - classic).abs() < 1e-15);
    }

    #[test]
    fn from_potential_and_velocity_matches_lapse_fill() {
        let phi = -6.26e7_f64;
        let ls = Spacetime::from_potential_and_velocity(phi, Velocity::ZERO);
        let expected_alpha = (1.0 + 2.0 * phi / C_SQUARED).sqrt();
        assert!((ls.alpha - expected_alpha).abs() < 1e-15);
        assert_eq!(ls.beta, 0.0);
        assert!((ls.proper_time_rate() - expected_alpha).abs() < 1e-15);
    }
}
