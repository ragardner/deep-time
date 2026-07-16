//! Proper-time rate tests via [`Spacetime`] / [`Drift`] (`physics` feature).

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod spacetime_rate_tests {
    use deep_time::consts::{C, C_SQUARED};
    use deep_time::physics::{Drift, Spacetime, Velocity};

    fn rate_from_phi_speed(phi_m2_s2: f64, speed_m_s: f64) -> f64 {
        Spacetime::from_potential_velocity_and_scale(
            phi_m2_s2 / C_SQUARED,
            Velocity::from_speed(speed_m_s),
            0.0,
        )
        .proper_time_rate()
    }

    /// GPS satellite vs geoid clock — Ashby’s circular factory frequency offset.
    #[test]
    fn gps_ashby_circular_factory_offset() {
        const GM: f64 = 3.986_004_415e14;
        const A1: f64 = 6_378_137.0;
        const J2: f64 = 1.082_63e-3;
        const A_GPS: f64 = 26_562_000.0;
        const OMEGA: f64 = 7.292_115_0e-5;

        let phi_gnd = -GM / A1 * (1.0 + 0.5 * J2) - 0.5 * (OMEGA * A1).powi(2);
        let phi_sat = -GM / A_GPS;
        let v_sat = (GM / A_GPS).sqrt();

        let rate_sat = rate_from_phi_speed(phi_sat, v_sat);
        let rate_gnd = rate_from_phi_speed(phi_gnd, 0.0);
        assert!(rate_sat > rate_gnd);

        let factory_lib = rate_sat / rate_gnd - 1.0;
        let factory_ashby = -1.5 * GM / (A_GPS * C_SQUARED)
            + GM / (A1 * C_SQUARED) * (1.0 + 0.5 * J2)
            + 0.5 * (OMEGA * A1).powi(2) / C_SQUARED;

        let us_per_day = |frac: f64| frac * 86_400.0 * 1_000_000.0;
        let us_lib = us_per_day(factory_lib);
        let us_ashby = us_per_day(factory_ashby);

        const EXPECTED_US: f64 = 38.575;
        const TOL_US: f64 = 0.001;

        assert!((us_ashby - EXPECTED_US).abs() < TOL_US);
        assert!((us_lib - EXPECTED_US).abs() < TOL_US);
        assert!((us_lib - us_ashby).abs() < TOL_US);
    }

    #[test]
    fn pure_special_relativistic_time_dilation() {
        let v = 0.1 * C;
        let rate = rate_from_phi_speed(0.0, v);
        let beta = v / C;
        let expected = (1.0 - beta * beta).sqrt();
        assert!((rate - expected).abs() < 1e-12);
    }

    #[test]
    fn pure_gravitational_time_dilation() {
        let gm = 3.986004418e14_f64;
        let r_low = 6_378_137.0;
        let r_high = 26_560_000.0;
        let rate_low = rate_from_phi_speed(-gm / r_low, 0.0);
        let rate_high = rate_from_phi_speed(-gm / r_high, 0.0);
        assert!(rate_high > rate_low);
    }

    #[test]
    fn rate_ratio_identical_states_is_one() {
        let st =
            Spacetime::from_potential_velocity_and_scale(-8.87e8 / C_SQUARED, Velocity::ZERO, 0.0);
        let r = st.proper_time_rate();
        assert!((r / r - 1.0).abs() < 1e-14);
    }

    #[test]
    fn drift_from_velocity_potential_matches_spacetime_rate() {
        let phi = -8.87e8_f64;
        let speed = 7_000.0_f64;
        let st = Spacetime::from_potential_velocity_and_scale(
            phi / C_SQUARED,
            Velocity::from_speed(speed),
            0.0,
        );
        let drift = Drift::from_velocity_potential_and_scale(speed, phi, 0.0);
        assert!((st.proper_time_rate() - drift.proper_time_rate()).abs() < 1e-15);
    }
}
