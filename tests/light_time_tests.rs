#[cfg(test)]
mod relativistic_tests {
    use deep_time::{
        Dt, LightContext, Spacetime, ObserverState, Position, Scale, Velocity, constants::C,
    };

    // =========================================================================================
    // Test Helpers
    // =========================================================================================

    fn make_state(
        tai_sec: i64,
        pos: Position,
        vel: Velocity,
        phi_m2_s2: f64,
        char_scale: f64,
    ) -> ObserverState {
        ObserverState {
            time: Dt::from(tai_sec, 0, Scale::TAI),
            position: pos,
            velocity: vel,
            grav_potential_m2_s2: phi_m2_s2,
            characteristic_length_scale: char_scale,
        }
    }

    // =========================================================================================
    // SECTION 1: LightContext & Shapiro Delay (Reference Values)
    // =========================================================================================

    /// **Hard reference case**: Sun-grazing geometry.
    ///
    /// This is the classic test case used throughout deep-space navigation literature.
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

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let got_us = correction.to_sec_f() * 1_000_000.0;

        assert!(
            (got_us - 119.45).abs() < 0.2,
            "Sun-grazing Shapiro: got {:.3} µs (expected ≈119.45 µs)",
            got_us
        );
    }

    // =========================================================================================
    // SECTION 2: Real-World GNSS / GPS Usage (Most Important Everyday Test)
    // =========================================================================================

    /// **Real-world GPS satellite clock rate test**.
    ///
    /// This replicates the exact relativistic correction applied to every GPS satellite.
    ///
    /// **Known values** (widely published):
    /// - Net effect: GPS clocks run **faster** than ground clocks by ~ +38.4 µs per day.
    /// - Without this correction, GPS position errors would grow ~10 km per day.
    ///
    /// References:
    /// - Ashby, N. (2003). "Relativity in the Global Positioning System".
    ///   *Living Reviews in Relativity* 6(1).
    ///   https://link.springer.com/article/10.12942/lrr-2003-1
    /// - GPS Interface Specification IS-GPS-200 (current revision).
    #[test]
    fn gps_satellite_net_clock_rate_is_faster() {
        let gps_distance_from_earth_center = 26_560_000.0; // meters
        let gps_speed = 3_874.0; // m/s
        let earth_gm = 3.986004418e14_f64;

        let gps_potential = -earth_gm / gps_distance_from_earth_center;
        let gps_sat = make_state(
            0,
            Position::new(gps_distance_from_earth_center, 0.0, 0.0),
            Velocity::from_speed(gps_speed),
            gps_potential,
            0.0,
        );

        // Ground clock (stationary on Earth's surface)
        let earth_radius = 6_378_137.0; // WGS84 equatorial radius
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
            "GPS satellite clocks must run faster than ground clocks \
             (gps_rate = {:.10}, ground_rate = {:.10})",
            gps_rate,
            ground_rate
        );

        let relative_rate = gps_rate / ground_rate;
        let daily_advance_us = (relative_rate - 1.0) * 86400.0 * 1_000_000.0;

        assert!(
            daily_advance_us > 30.0,
            "GPS daily advance should be > +30 µs/day (got {:.1} µs/day)",
            daily_advance_us
        );
    }

    // =========================================================================================
    // SECTION 3: Mission Geometries
    // =========================================================================================

    /// **Earth–Mars geometry** (moderate impact parameter).
    ///
    /// This represents a typical Earth-Mars ranging geometry during opposition season
    /// with non-zero impact parameter. The resulting Shapiro delay is modest (~17 µs)
    /// because the line-of-sight does not pass close to the Sun.
    ///
    /// This geometry is representative of many actual DSN tracking passes.
    #[test]
    fn one_way_delay_earth_mars_geometry() {
        let earth_pos = Position::new(1.5e11, 0.0, 0.0);
        let mars_pos = Position::new(1.8e11, 0.8e11, 0.0);

        let tx = make_state(0, earth_pos, Velocity::from_speed(29_780.0), -8.87e8, 0.0);
        let rx = make_state(1200, mars_pos, Velocity::from_speed(24_130.0), -1.27e8, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let corr_us = correction.to_sec_f() * 1e6;

        assert!(
            corr_us > 10.0 && corr_us < 30.0,
            "Earth-Mars geometry: got {:.1} µs (expected 15-25 µs)",
            corr_us
        );
    }

    /// **Near-Earth round-trip** (small separation, moderate impact parameter).
    ///
    /// This geometry is typical of early mission phases, lunar ranging, or cislunar tracking.
    /// The total round-trip relativistic correction is small (~4 µs) because both the
    /// geometric distance and impact parameter are modest.
    #[test]
    fn round_trip_near_earth_geometry() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.3e11, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
        let tolerance = Dt::from_ns(1, Scale::TAI);

        let correction = tx.round_trip_relativistic_correction(
            &mut |_t| make_state(520, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0),
            &mut |_t| make_state(1040, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0),
            LightContext::SOLAR,
            tolerance,
            12,
        );

        let corr_us = correction.to_sec_f() * 1e6;

        assert!(
            corr_us > 1.0 && corr_us < 15.0,
            "Near-Earth round-trip: got {:.1} µs (expected 3–8 µs)",
            corr_us
        );
    }

    // =========================================================================================
    // SECTION 4: Core Functionality & Edge Cases
    // =========================================================================================

    /// Verifies `LightContext::FLAT` disables all gravitational delay and that
    /// `LightContext::from_grav_param` produces sensible values for other bodies
    /// (e.g. Jupiter, a common deep-space target).
    #[test]
    fn light_context_flat_and_from_grav_param() {
        let tx = make_state(
            0,
            Position::new(1.5e11, 0.0, 0.0),
            Velocity::ZERO,
            -8.87e8,
            0.0,
        );
        let rx = make_state(
            0,
            Position::new(-1.5e11, 0.0, 0.0),
            Velocity::ZERO,
            -8.87e8,
            0.0,
        );
        assert_eq!(
            tx.one_way_relativistic_delay_to(rx, LightContext::FLAT),
            Dt::ZERO
        );

        let jupiter = LightContext::from_grav_param(1.2668654e17);
        assert!(jupiter.two_grav_param_over_c3 > 0.0 && jupiter.two_grav_param_over_c3 < 2e-7);
    }

    /// Verifies both constructors (`new` and `new_strong_field`) and that
    /// `proper_time_rate()` returns a physically bounded value (< 1.0) for
    /// bound solar-system observers, as required by the unified master Lagrangian.
    #[test]
    fn observer_state_constructors_and_proper_time_rate() {
        let normal = ObserverState::new(
            Dt::from(0, 0, Scale::TAI),
            Position::new(1.5e11, 0.0, 0.0),
            Velocity::from_speed(30_000.0),
            -8.87e8,
        );
        assert_eq!(normal.characteristic_length_scale, 0.0);

        let strong = ObserverState::new_strong_field(
            Dt::from(0, 0, Scale::TAI),
            Position::new(1.5e11, 0.0, 0.0),
            Velocity::from_speed(30_000.0),
            -8.87e8,
            1e6,
        );
        assert_eq!(strong.characteristic_length_scale, 1e6);

        let earth = make_state(0, Position::ZERO, Velocity::from_speed(465.0), -6.26e7, 0.0);
        let rate = earth.proper_time_rate();
        assert!((0.999999999..=1.0).contains(&rate));
    }

    /// Edge cases: zero distance must produce zero correction, and identical
    /// states in flat spacetime must also produce zero correction.
    #[test]
    fn one_way_delay_zero_distance_and_flat_identical() {
        let pos = Position::new(1.5e11, 0.0, 0.0);
        let state = make_state(0, pos, Velocity::ZERO, -8.87e8, 0.0);
        assert_eq!(
            state.one_way_relativistic_delay_to(state, LightContext::SOLAR),
            Dt::ZERO
        );

        let tx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        let rx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        assert_eq!(
            tx.one_way_relativistic_delay_to(rx, LightContext::FLAT),
            Dt::ZERO
        );
    }

    // =========================================================================================
    // SECTION 5: Iterative Solver & Integrated Path
    // =========================================================================================

    /// The iterative solver must converge to sub-nanosecond accuracy.
    #[test]
    fn iterative_solver_converges_to_sub_nanosecond() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let tx = make_state(0, tx_pos, Velocity::ZERO, -8.87e8, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);
        let tolerance = Dt::from_ns(1, Scale::TAI);

        let (correction, final_rx_time, _) = tx.iterative_one_way_relativistic_delay_to(
            &mut |t| {
                let sec = t.to_sec_f() as i64;
                make_state(sec, rx_pos, Velocity::ZERO, -8.80e8, 0.0)
            },
            LightContext::SOLAR,
            tolerance,
            20,
        );

        let geometric = tx_pos.distance_to(rx_pos) / C;
        let total = final_rx_time.to_diff_raw(tx.time).to_sec_f();

        assert!(
            (total - geometric - correction.to_sec_f()).abs() < 1e-10,
            "Iterative solver failed to satisfy light-time equation"
        );
    }

    /// When the gravitational environment is constant along the path, the
    /// integrated method must return **exactly** the same result as the direct
    /// method. Also verifies the documented fallback behavior for empty samples.
    #[test]
    fn integrated_matches_direct_when_constant_and_falls_back() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);
        let common_potential = -8.835e8_f64;
        let common_vel = Velocity::ZERO;

        let tx = make_state(0, tx_pos, common_vel, common_potential, 0.0);
        let rx = make_state(520, rx_pos, common_vel, common_potential, 0.0);

        let direct = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);

        let samples: Vec<Spacetime> = (0..=20)
            .map(|_| {
                Spacetime::from_potential_velocity_and_scale(
                    common_potential / (C * C),
                    common_vel,
                    0.0,
                )
            })
            .collect();

        let integrated =
            tx.one_way_relativistic_delay_integrated(rx, LightContext::SOLAR, &samples);
        assert!((direct.to_sec_f() - integrated.to_sec_f()).abs() < 1e-12);

        let integrated_empty =
            tx.one_way_relativistic_delay_integrated(rx, LightContext::SOLAR, &[]);
        assert_eq!(direct, integrated_empty);
    }

    // =========================================================================================
    // SECTION 6: Doppler Factors
    // =========================================================================================

    /// Verifies that the two-way Doppler factor is exactly the square of the
    /// one-way factor (standard result) and that identical states produce
    /// a factor of exactly 1.0.
    #[test]
    fn doppler_factors() {
        let tx = make_state(0, Position::ZERO, Velocity::ZERO, -8.87e8, 0.0);
        let rx = make_state(
            0,
            Position::new(1.5e11, 0.0, 0.0),
            Velocity::ZERO,
            -1.27e8,
            0.0,
        );

        let one_way = tx.relativistic_clock_doppler_factor(rx);
        let two_way = tx.two_way_relativistic_doppler_factor(rx);
        assert!((two_way - one_way * one_way).abs() < 1e-14);

        let identical = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
        assert!((identical.relativistic_clock_doppler_factor(identical) - 1.0).abs() < 1e-14);
    }
}
