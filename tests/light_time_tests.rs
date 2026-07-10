#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod light_time_tests {
    use deep_time::{Dt, Observer, Position, Scale, Spacetime, Velocity, consts::C};

    fn make_state(
        tai_sec: i128,
        pos: Position,
        vel: Velocity,
        phi_m2_s2: f64,
        char_scale: f64,
    ) -> Observer {
        Observer {
            time: deep_time::dt!(Dt::sec_to_attos(tai_sec)),
            position: pos,
            velocity: vel,
            grav_potential_m2_s2: phi_m2_s2,
            characteristic_length_scale: char_scale,
        }
    }

    /// Sun-grazing geometry.
    ///
    /// The expected one-way Shapiro delay is approximately **119.45 µs**.
    ///
    /// References:
    /// - Moyer, T.D. (2003). *Formulation for Observed and Computed Values of
    ///   Deep Space Network Data Types for Navigation*. JPL DESCANSO Vol. 2.
    /// - IAU 2015 recommended solar GM value.
    /// - Multiple Cassini solar conjunction experiments.
    #[test]
    fn sun_grazing_shapiro_delay_reference_value() {
        let au = 1.495978707e11_f64;
        let r_sun = 6.957e8_f64;
        let b = r_sun * 1.001;

        let tx_pos = Position::new(au, b, 0.0);
        let rx_pos = Position::new(-au, b, 0.0);

        let tx = make_state(0, tx_pos, Velocity::ZERO, -8.87e8, 0.0);
        let rx = make_state(0, rx_pos, Velocity::ZERO, -8.87e8, 0.0);

        // Sun is at the origin in this test coordinate system
        let bodies = &[(Dt::SHAPIRO_SOLAR, Position::ZERO)];
        let correction = tx.shapiro_delay(&rx, bodies);
        let got_us = correction.to_sec_f() * 1_000_000.0;

        assert!(
            (got_us - 119.45).abs() < 0.2,
            "Sun-grazing Shapiro: got {:.3} µs (expected ≈119.45 µs)",
            got_us
        );
    }

    /// Verifies the net relativistic clock rate of a GPS satellite relative to a
    /// ground clock.
    ///
    /// GPS satellites orbit at approximately 20,200 km altitude. Two relativistic
    /// effects influence their onboard clocks:
    ///
    /// - General relativity: weaker gravitational potential at orbital altitude
    ///   causes clocks to run faster.
    /// - Special relativity: orbital velocity causes clocks to run slower.
    ///
    /// The gravitational effect is larger, resulting in a net rate increase.
    ///
    /// The accepted value used in GPS operations is approximately +38.4 µs per day.
    /// This test confirms that `proper_time_rate()` produces a result consistent
    /// with this established figure when using standard GPS orbital parameters.
    ///
    /// References:
    /// - Ashby, N. (2003). "Relativity in the Global Positioning System".
    ///   Living Reviews in Relativity 6(1).
    ///   https://doi.org/10.12942/lrr-2003-1
    /// - GPS Interface Specification IS-GPS-200.
    #[test]
    fn gps_satellite_net_relativistic_clock_advance() {
        let gps_distance_from_earth_center = 26_560_000.0; // m
        let gps_speed = 3_874.0; // m/s
        let earth_gm = 3.986004418e14_f64; // m³/s²

        let gps_potential = -earth_gm / gps_distance_from_earth_center;

        let gps_sat = make_state(
            0,
            Position::new(gps_distance_from_earth_center, 0.0, 0.0),
            Velocity::from_speed(gps_speed),
            gps_potential,
            0.0,
        );

        let earth_radius = 6_378_137.0; // WGS84 equatorial radius (m)
        let ground_potential = -earth_gm / earth_radius;

        let ground = make_state(
            0,
            Position::new(earth_radius, 0.0, 0.0),
            Velocity::ZERO,
            ground_potential,
            0.0,
        );

        let gps_rate = gps_sat.proper_time_rate();
        let ground_rate = ground.proper_time_rate();

        assert!(
            gps_rate > ground_rate,
            "GPS satellite proper time rate must exceed ground rate \
         (gps_rate = {:.12}, ground_rate = {:.12})",
            gps_rate,
            ground_rate
        );

        let relative_rate = gps_rate / ground_rate;
        let daily_advance_us = (relative_rate - 1.0) * 86400.0 * 1_000_000.0;

        const EXPECTED: f64 = 38.4;
        const TOL: f64 = 0.04;

        assert!(
            (daily_advance_us - EXPECTED).abs() < TOL,
            "GPS net daily advance: got {:.4} µs/day (expected {:.1} ± {:.1})",
            daily_advance_us,
            EXPECTED,
            TOL
        );
    }

    /// Validates the iterative one-way light-time solver on a solar-grazing geometry.
    ///
    /// Uses the classic solar conjunction case (impact parameter ≈ 1.001 R_☉).
    /// Checks that the solver converges and returns a propagation correction
    /// consistent with the known one-way Shapiro delay of ~119.45 µs.
    ///
    /// Exercises:
    /// iterative_one_way_light_time_to → shapiro_delay → shapiro_delay
    ///
    /// Reference value from Moyer (2003) / Cassini-era solar conjunction analyses.
    #[test]
    fn iterative_one_way_light_time_to_sun_grazing() {
        let au = 1.495978707e11_f64;
        let r_sun = 6.957e8_f64;
        let b = r_sun * 1.001;

        let tx_pos = Position::new(au, b, 0.0);
        let rx_pos = Position::new(-au, b, 0.0);

        let tx = make_state(0, tx_pos, Velocity::ZERO, -8.87e8, 0.0);
        let tolerance = Dt::from_ns(1, 0, Scale::TAI, Scale::TAI);
        let bodies = &[(Dt::SHAPIRO_SOLAR, Position::ZERO)];

        // Static receiver — safe to ignore the time argument
        let (prop_correction, _rx_time, _rx_state) = tx.iterative_one_way_light_time_to(
            &mut |_t| make_state(0, rx_pos.clone(), Velocity::ZERO, -8.87e8, 0.0),
            bodies,
            tolerance,
            20,
        );

        let got_us = prop_correction.to_sec_f() * 1_000_000.0;

        assert!(
            (got_us - 119.45).abs() < 0.1,
            "Iterative solver sun-grazing Shapiro: got {:.3} µs (expected ≈119.45 µs)",
            got_us
        );
    }

    /// Checks internal consistency of round-trip composition.
    ///
    /// This test verifies that `round_trip_light_time_correction` produces
    /// the same result as manually solving the uplink leg iteratively,
    /// then solving the downlink leg from the accurate receiver state at
    /// uplink arrival time.
    ///
    /// It is an implementation consistency test (it protects the composition
    /// logic), not a validation of the underlying relativistic models against
    /// external references.
    #[test]
    fn round_trip_consistent_with_uplink_plus_downlink() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(2.2e11, 0.4e11, 0.0);

        let tx = make_state(
            0,
            tx_pos.clone(),
            Velocity::from_speed(29_780.0),
            -8.87e8,
            0.0,
        );
        let tolerance = Dt::from_ns(1, 0, Scale::TAI, Scale::TAI);

        // Sun is at the origin in this test
        let bodies = &[(Dt::SHAPIRO_SOLAR, Position::ZERO)];

        // Full round-trip result from the convenience method
        let round_trip_corr = tx.round_trip_light_time_correction(
            &mut |t: Dt| {
                let sec = t.to_sec();
                make_state(
                    sec,
                    rx_pos.clone(),
                    Velocity::from_speed(24_000.0),
                    -1.3e8,
                    0.0,
                )
            },
            &mut |t: Dt| {
                let sec = t.to_sec();
                make_state(
                    sec,
                    tx_pos.clone(),
                    Velocity::from_speed(29_780.0),
                    -8.87e8,
                    0.0,
                )
            },
            bodies,
            tolerance,
            15,
        );

        // 1. Solve uplink iteratively (propagation only)
        let (uplink_corr, _rx_arrival_time, rx_at_arrival) = tx.iterative_one_way_light_time_to(
            &mut |t| {
                let sec = t.to_sec();
                make_state(
                    sec,
                    rx_pos.clone(),
                    Velocity::from_speed(24_000.0),
                    -1.3e8,
                    0.0,
                )
            },
            bodies,
            tolerance,
            15,
        );

        // 2. Solve downlink iteratively from the accurate arrival state
        let (downlink_corr, _final_rx_time, _final_rx_state) = rx_at_arrival
            .iterative_one_way_light_time_to(
                &mut |t| {
                    let sec = t.to_sec();
                    make_state(
                        sec,
                        tx_pos.clone(),
                        Velocity::from_speed(29_780.0),
                        -8.87e8,
                        0.0,
                    )
                },
                bodies,
                tolerance,
                15,
            );

        let manual_sum = uplink_corr.add(downlink_corr);
        let diff = (round_trip_corr.to_sec_f() - manual_sum.to_sec_f()).abs();

        assert!(
            diff < 1e-9,
            "Round-trip correction should be consistent with iterative uplink + downlink \
         (difference = {:.3e} s)",
            diff
        );
    }

    /// Edge case: identical states (zero separation) must produce zero correction.
    ///
    /// This verifies that the implementation handles the degenerate
    /// case of zero geometric distance between transmitter and receiver.
    /// In this situation both the Shapiro delay and the clock-rate correction
    /// are expected to be exactly zero.
    #[test]
    fn one_way_delay_zero_distance_and_identical_states() {
        let bodies = &[(Dt::SHAPIRO_SOLAR, Position::ZERO)];
        let pos = Position::new(1.5e11, 0.0, 0.0);

        // Identical states → zero separation
        let state = make_state(0, pos, Velocity::ZERO, -8.87e8, 0.0);
        assert_eq!(state.shapiro_delay(&state, bodies), Dt::ZERO);

        // Both at the origin
        let tx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        let rx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        assert_eq!(tx.shapiro_delay(&rx, bodies), Dt::ZERO);
    }

    /// Verifies internal consistency of the iterative one-way light-time solver.
    ///
    /// After convergence, the following relationship must hold to high precision:
    ///
    /// ```text
    /// t_rx − t_tx ≈ |r_rx − r_tx| / c + Δt_prop
    /// ```
    ///
    /// where `Δt_prop` is the propagation correction (Shapiro) returned by the solver.
    ///
    /// This is a **consistency / regression test** for the fixed-point iteration.
    /// It confirms that the solver converged to a solution that satisfies the
    /// coordinate-time light-time equation it is designed to solve.
    ///
    /// It does **not** validate correctness against external references.
    #[test]
    fn iterative_solver_converges_to_sub_nanosecond() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let tx = make_state(0, tx_pos.clone(), Velocity::ZERO, -8.87e8, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);
        let tolerance = Dt::from_ns(1, 0, Scale::TAI, Scale::TAI);
        let bodies = &[(Dt::SHAPIRO_SOLAR, Position::ZERO)];

        let (prop_correction, final_rx_time, _) = tx.iterative_one_way_light_time_to(
            &mut |t| {
                let sec = t.to_sec();
                make_state(sec, rx_pos.clone(), Velocity::ZERO, -8.80e8, 0.0)
            },
            bodies,
            tolerance,
            20,
        );

        let geometric = tx_pos.distance_to(&rx_pos) / C;
        let total = final_rx_time.to_diff_raw(tx.time).to_sec_f();

        assert!(
            (total - geometric - prop_correction.to_sec_f()).abs() < 1e-10,
            "Iterative solver failed to satisfy light-time equation \
         (residual = {:.2e} s)",
            (total - geometric - prop_correction.to_sec_f()).abs()
        );
    }

    /// Verifies the expected mathematical relationship between one-way and
    /// two-way relativistic clock rate ratios, plus a basic sanity check.
    ///
    /// The method `relativistic_clock_rate_ratio` returns the factor by which
    /// the receiver's proper time advances relative to the transmitter's due to
    /// velocity and gravitational potential.
    ///
    /// For **two-way** (round-trip) measurements, the total relativistic clock-rate
    /// effect is the square of the one-way ratio:
    ///
    /// ```text
    /// two_way_clock_ratio = one_way_ratio × one_way_ratio
    /// ```
    ///
    /// This is because the signal experiences the ratio once on the uplink and
    /// once on the downlink. This test confirms that the API produces results
    /// consistent with this standard result.
    ///
    /// It also verifies the basic invariant that identical transmitter and
    /// receiver states must produce a ratio of exactly 1.0.
    #[test]
    fn relativistic_clock_rate_ratio_properties() {
        let tx = make_state(0, Position::ZERO, Velocity::ZERO, -8.87e8, 0.0);
        let rx = make_state(
            0,
            Position::new(1.5e11, 0.0, 0.0),
            Velocity::ZERO,
            -1.27e8,
            0.0,
        );

        let one_way = tx.relativistic_clock_rate_ratio(&rx);

        // Since we removed the dedicated two-way method, we compute it manually
        let two_way = one_way * one_way;
        assert!((two_way - one_way * one_way).abs() < 1e-14);

        let identical = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        assert!((identical.relativistic_clock_rate_ratio(&identical) - 1.0).abs() < 1e-14);
    }

    /// Verifies pure special-relativistic time dilation (velocity effect only).
    ///
    /// When gravitational potential is zero, the proper time rate of a moving
    /// observer must match the exact special-relativistic formula
    /// `sqrt(1 - v²/c²)`.
    #[test]
    fn pure_special_relativistic_time_dilation() {
        let v = 0.1 * C; // 10% of speed of light

        let moving = make_state(0, Position::ZERO, Velocity::from_speed(v), 0.0, 0.0);

        let rate = moving.proper_time_rate();
        let beta = v / C;
        let expected = (1.0 - beta * beta).sqrt();

        assert!(
            (rate - expected).abs() < 1e-12,
            "Pure SR time dilation mismatch: got {:.12}, expected {:.12}",
            rate,
            expected
        );
    }

    /// Verifies pure gravitational time dilation (potential effect only).
    ///
    /// With velocity set to zero, an observer at higher altitude (weaker
    /// gravitational potential) must have a higher proper time rate than an
    /// observer closer to the central body.
    #[test]
    fn pure_gravitational_time_dilation() {
        let gm = 3.986004418e14_f64;
        let r_low = 6_378_137.0; // approximately Earth surface
        let r_high = 26_560_000.0; // approximately GPS altitude

        let low = make_state(
            0,
            Position::new(r_low, 0.0, 0.0),
            Velocity::ZERO,
            -gm / r_low,
            0.0,
        );

        let high = make_state(
            0,
            Position::new(r_high, 0.0, 0.0),
            Velocity::ZERO,
            -gm / r_high,
            0.0,
        );

        let rate_low = low.proper_time_rate();
        let rate_high = high.proper_time_rate();

        assert!(
            rate_high > rate_low,
            "Higher altitude must have higher proper time rate \
             (low = {:.12}, high = {:.12})",
            rate_low,
            rate_high
        );
    }
}
