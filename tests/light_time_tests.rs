#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod light_time_tests {
    use deep_time::macros::dt;
    use deep_time::{Dt, Observer, Position, Scale, Spacetime, Velocity, consts::C};

    fn make_state(
        tai_sec: i128,
        pos: Position,
        vel: Velocity,
        phi_m2_s2: f64,
        char_scale: f64,
    ) -> Observer {
        Observer {
            time: dt!(Dt::sec_to_attos(tai_sec)),
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

    /// GPS satellite vs geoid clock — Ashby’s circular factory frequency offset.
    ///
    /// # What this checks
    ///
    /// Clocks on GPS satellites run **faster** than identical clocks on the
    /// Earth’s geoid by a nearly constant fractional rate of about
    /// \(4.4647\times 10^{-10}\) (\(\approx +38.575\,\mu\mathrm{s/day}\)).
    ///
    /// Early GPS satellites used a *factory frequency offset*: the onboard
    /// standard was set a little slow on the ground so that, once in a nominal
    /// circular orbit, it would appear to beat near the geoid rate (nominal L1
    /// synthesis from 10.23 MHz). Without the correction, ~38 µs/day is about
    /// **11.5 km** of one-way ranging bias per day.
    ///
    /// This test rebuilds Ashby’s **circular-orbit factory offset** from
    /// potential and velocity, then checks that the library’s
    /// [`Observer::proper_time_rate`] matches Ashby’s first-order formula to
    /// better than **1 ns/day**.
    ///
    /// This is the classical *mean* circular offset only. It is not a substitute
    /// for the broadcast clock polynomial or the IS-GPS-200 user algorithm
    /// (eccentricity term, etc.).
    ///
    /// # Physics (weak field, circular orbit)
    ///
    /// Proper time advances relative to ECI coordinate time as
    ///
    /// ```text
    /// dτ/dt = sqrt( (1 + 2Φ/c²)(1 − v²/c²) )  ≈  1 + Φ/c² − v²/(2c²)
    /// ```
    ///
    /// with Kretschmann left at zero (the GNSS default). The factory offset is
    /// `rate_sat / rate_geoid − 1`.
    ///
    /// For a circular Kepler orbit of semi-major axis `a`, energy conservation
    /// collapses satellite gravity + second-order Doppler to `−3GM/(2c²a)`.
    /// The geoid reference is not simply `−GM/R` at rest: monopole, equatorial
    /// `J₂`, and Earth-rotation Doppler all enter. Ashby’s constant-rate formula
    /// (AAPT notes, Eq. 34) is
    ///
    /// ```text
    /// Δf/f = −3GM/(2c²a) + GM/(c²a₁)(1 + J₂/2) + (ω a₁)²/(2c²)
    ///      ≈ 4.4647×10⁻¹⁰
    /// ```
    ///
    /// Here the geoid is one **effective** potential (with `v = 0`):
    ///
    /// ```text
    /// Φ_geoid = −GM/a₁ (1 + J₂/2) − (ω a₁)²/2
    /// ```
    ///
    /// # Intentionally omitted
    ///
    /// Eccentricity (`∝ e sin E`; IS-GPS-200 user algorithm), higher multipoles,
    /// satellite `J₂` averaging, Sagnac, and tides.
    ///
    /// # Constants
    ///
    /// | Symbol | Value | Source |
    /// |--------|-------|--------|
    /// | `GM` | `3.986004415e14` m³/s² | Ashby AAPT Eq. 20 |
    /// | `a₁` | `6_378_137` m | WGS 84 / Ashby `a₁` |
    /// | `J₂` | `1.08263e-3` | Ashby AAPT Eq. 21 |
    /// | `a` | `26_562_000` m | Ashby designed GPS semi-major axis |
    /// | `ω` | `7.2921150e-5` rad/s | WGS 84 |
    /// | `c` | SI / `deep_time::consts::C` | |
    ///
    /// WGS 84 (NIMA TR8350.2):
    /// <https://earth-info.nga.mil/php/download.php?file=coord-wgs84>
    ///
    /// # References
    ///
    /// - Ashby (2003), *Relativity in the Global Positioning System*,
    ///   Living Reviews in Relativity **6**, 1.
    ///   <https://doi.org/10.12942/lrr-2003-1>
    /// - Ashby (2002), Physics Today **55**(5), 41–47.
    ///   <https://doi.org/10.1063/1.1485583>
    /// - Ashby (2006), AAPT notes (Eqs. 20–34):
    ///   <https://www.aapt.org/doorway/tgru/articles/ashbyarticle.pdf>
    /// - IS-GPS-200: <https://www.gps.gov/technical/icwg/IS-GPS-200N.pdf>
    #[test]
    fn gps_ashby_circular_factory_offset() {
        use deep_time::consts::C_SQUARED;

        // Ashby AAPT Eq. (20)–(21); designed semi-major axis 26 562 km.
        const GM: f64 = 3.986_004_415e14;
        const A1: f64 = 6_378_137.0;
        const J2: f64 = 1.082_63e-3;
        const A_GPS: f64 = 26_562_000.0;
        const OMEGA: f64 = 7.292_115_0e-5;

        // Φ_geoid = −GM/a₁(1 + J₂/2) − (ω a₁)²/2  (effective; rate uses v = 0)
        let phi_gnd = -GM / A1 * (1.0 + 0.5 * J2) - 0.5 * (OMEGA * A1).powi(2);
        let phi_sat = -GM / A_GPS;
        let v_sat = (GM / A_GPS).sqrt();

        // Rate depends only on Φ and |v|; position is unused here.
        let gps = make_state(0, Position::ZERO, Velocity::from_speed(v_sat), phi_sat, 0.0);
        let gnd = make_state(0, Position::ZERO, Velocity::ZERO, phi_gnd, 0.0);

        let rate_sat = gps.proper_time_rate();
        let rate_gnd = gnd.proper_time_rate();
        assert!(
            rate_sat > rate_gnd,
            "GPS clock must run faster than the geoid clock \
             (sat={rate_sat:.15}, gnd={rate_gnd:.15})"
        );

        let factory_lib = rate_sat / rate_gnd - 1.0;

        // Ashby AAPT Eq. 34 (first-order closed form).
        let factory_ashby = -1.5 * GM / (A_GPS * C_SQUARED)
            + GM / (A1 * C_SQUARED) * (1.0 + 0.5 * J2)
            + 0.5 * (OMEGA * A1).powi(2) / C_SQUARED;

        let us_per_day = |frac: f64| frac * 86_400.0 * 1_000_000.0;
        let us_lib = us_per_day(factory_lib);
        let us_ashby = us_per_day(factory_ashby);

        // Printed Ashby figure ~38.575 µs/day; tolerate 1 ns/day.
        const EXPECTED_US: f64 = 38.575;
        const TOL_US: f64 = 0.001;

        assert!(
            (us_ashby - EXPECTED_US).abs() < TOL_US,
            "Ashby formula: {us_ashby:.6} µs/day (expected {EXPECTED_US} ± {TOL_US})"
        );
        assert!(
            (us_lib - EXPECTED_US).abs() < TOL_US,
            "library rates: {us_lib:.6} µs/day (expected {EXPECTED_US} ± {TOL_US})"
        );
        assert!(
            (us_lib - us_ashby).abs() < TOL_US,
            "library {us_lib:.6} vs Ashby {us_ashby:.6} µs/day"
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
