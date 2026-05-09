#[cfg(test)]
mod proper_time_samples_tests {
    use deep_time::{Dt, LocalSpacetime, Scale};

    fn make_state(tai_sec: i64) -> Dt {
        Dt::from(tai_sec, 0, Scale::TAI)
    }

    #[test]
    fn zero_duration_returns_zero() {
        let t0 = make_state(0);
        let t1 = make_state(0);
        let samples = [LocalSpacetime::new(1.0, 0.0, 0.0); 2];

        let dtau = t0.proper_time_interval_samples(t1, &samples);
        assert_eq!(dtau, Dt::ZERO);
    }

    #[test]
    fn constant_flat_space_rate_equals_coordinate_time() {
        let t0 = make_state(0);
        let t1 = make_state(1000);

        let flat = LocalSpacetime::new(1.0, 0.0, 0.0);
        let samples = [flat; 2];

        let dtau = t0.proper_time_interval_samples(t1, &samples);
        assert_eq!(dtau, Dt::from_sec(1000, Scale::TAI));
    }

    #[test]
    fn constant_relativistic_rate_gravitational_slowdown() {
        let t0 = make_state(0);
        let t1 = make_state(1000);

        let slow = LocalSpacetime::new(0.9, 0.0, 0.0);
        let samples = [slow; 2];

        let dtau = t0.proper_time_interval_samples(t1, &samples);
        assert_eq!(dtau, Dt::from_sec(900, Scale::TAI));
    }

    #[test]
    fn relativistic_correction_returns_delta_tau_minus_delta_t() {
        let t0 = make_state(0);
        let t1 = make_state(1000);

        let slow = LocalSpacetime::new(0.9, 0.0, 0.0);
        let samples = [slow; 2];

        let correction = t0.relativistic_correction_with_samples(t1, &samples);
        assert_eq!(correction, Dt::from_sec(-100, Scale::TAI));
    }

    #[test]
    fn negative_interval_is_handled_correctly() {
        let t0 = make_state(1000);
        let t1 = make_state(0);

        let slow = LocalSpacetime::new(0.9, 0.0, 0.0);
        let samples = [slow; 2];

        let dtau = t0.proper_time_interval_samples(t1, &samples);
        assert_eq!(dtau, Dt::from_sec(-900, Scale::TAI));
    }

    #[test]
    fn velocity_only_relativistic_rate() {
        let t0 = make_state(0);
        let t1 = make_state(500);

        let moving = LocalSpacetime::new(1.0, 0.6, 0.0);
        let samples = [moving; 2];

        let dtau = t0.proper_time_interval_samples(t1, &samples);
        assert_eq!(dtau, Dt::from_sec(400, Scale::TAI));
    }
}
