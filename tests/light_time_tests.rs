#[cfg(test)]
mod relativistic_tests {
    use deep_time::{
        Dt, LightContext, LocalSpacetime, ObserverState, Position, Scale, TSpan, Velocity,
        constants::C,
    };

    /// Small helper to build a `ObserverState` quickly.
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

    #[test]
    fn zero_relativity_gives_zero_correction() {
        let origin = Position::ZERO;
        let zero_vel = Velocity::from_speed(0.0);
        let tx = make_state(0, origin, zero_vel, 0.0, 0.0);
        let rx = make_state(0, origin, zero_vel, 0.0, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::FLAT);
        assert_eq!(correction, TSpan::ZERO);
    }

    #[test]
    fn pure_shapiro_delay_sun_grazing() {
        let au = 1.495978707e11_f64;
        let r_sun = 6.957e8_f64;
        let b = r_sun * 1.001;

        let tx_pos = Position::new(au, b, 0.0);
        let rx_pos = Position::new(-au, b, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let got_us = correction.to_sec_f() * 1_000_000.0;

        assert!(
            (got_us - 119.45).abs() < 0.5,
            "Sun-grazing Shapiro delay: got {:.3} µs (expected ~119.45 µs)",
            got_us
        );
    }

    #[test]
    fn clock_rate_correction_only() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let corr_sec = correction.to_sec_f();

        assert!(
            corr_sec.abs() < 1e-5,
            "Clock-rate correction unreasonably large"
        );
        assert!(
            corr_sec.abs() > 1e-10,
            "Clock-rate correction suspiciously small"
        );
    }

    #[test]
    fn iterative_solver_converges_quickly() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);

        let rx_pos = Position::new(1.52e11, 0.0, 0.0);
        let rx_phi = -8.80e8;

        let tolerance = TSpan::from_ns(1);
        let (correction, final_rx_time) = tx.iterative_one_way_relativistic_delay_to(
            |guessed_time| {
                make_state(
                    guessed_time.sec(),
                    rx_pos,
                    Velocity::from_speed(0.0),
                    rx_phi,
                    0.0,
                )
            },
            LightContext::SOLAR,
            tolerance,
            12,
        );

        assert!(correction.to_sec_f().abs() < 1e-5);

        let geometric_sec = tx_pos.distance_to(rx_pos) / C;
        let total_sec = final_rx_time.to_diff_raw(tx.time).to_sec_f();
        assert!(
            (total_sec - geometric_sec).abs() < 1e-4,
            "Converged receive time deviates from geometric light time by {:.6} s",
            (total_sec - geometric_sec).abs()
        );
    }

    #[test]
    fn round_trip_correction_is_roughly_twice_one_way() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
        let rx = make_state(0, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

        let round_trip_measured = TSpan::from_sec_f(1010.0);

        let correction =
            tx.round_trip_relativistic_correction(round_trip_measured, rx, LightContext::SOLAR);
        let corr_sec = correction.to_sec_f();

        assert!(corr_sec > 0.0);
        assert!(corr_sec < 1e-3);
    }

    #[test]
    fn integrated_low_samples_matches_trapezoidal() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        let trapezoidal = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);

        let samples = [LocalSpacetime::new(1.0, 0.0, 0.0); 2];

        let integrated =
            tx.one_way_relativistic_delay_integrated(rx, LightContext::SOLAR, &samples);

        let diff = (trapezoidal.to_sec_f() - integrated.to_sec_f()).abs();
        assert!(
            diff < 2e-7, // small numerical difference is expected when going through LocalSpacetime
            "n=2 mismatch: {}",
            diff
        );
    }

    #[test]
    fn integrated_high_samples_still_reasonable() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        // 21 samples with zero relative drift (sanity check – no huge blow-up)
        let samples = [LocalSpacetime::new(1.0, 0.0, 0.0); 21];

        let integrated = tx.one_way_relativistic_delay_integrated(rx, LightContext::FLAT, &samples);

        let corr_sec = integrated.to_sec_f();
        assert!(corr_sec.abs() < 1e-3);
    }
}
