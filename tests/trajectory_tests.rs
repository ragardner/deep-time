#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "physics")]
mod proper_time_tests {
    use deep_time::{Dt, DtErrKind, Scale, Spacetime, Velocity};

    fn tai(sec: i128) -> Dt {
        Dt::from_sec(sec, Scale::TAI, Scale::TAI)
    }

    /// SI potential Φ (m²/s²) that yields the given weak-field lapse α.
    fn phi_for_alpha(alpha: f64) -> f64 {
        Spacetime::grav_potential_from_alpha(alpha)
    }

    // =====================================================================
    // proper_time_from_path
    // =====================================================================

    #[test]
    fn zero_duration_or_insufficient_samples_returns_zero() {
        let t = tai(0);
        let flat = Spacetime::new(1.0, 0.0, 0.0);

        // Empty path
        assert_eq!(
            Dt::proper_time_from_path(std::iter::empty::<(Dt, Spacetime)>()),
            Ok(Dt::ZERO)
        );

        // Single point
        assert_eq!(
            Dt::proper_time_from_path(std::iter::once((t, flat.clone()))),
            Ok(Dt::ZERO)
        );

        // Zero-duration path (start == end)
        let zero_dur = [(t, flat.clone()), (t, flat)];
        assert_eq!(Dt::proper_time_from_path(zero_dur), Ok(Dt::ZERO));
    }

    #[test]
    fn constant_rate_flat_spacetime_yields_exact_coordinate_time() {
        let t0 = tai(0);
        let t1 = tai(86400);

        // Flat spacetime → proper time rate = 1.0 (no dilation)
        let flat = Spacetime::new(1.0, 0.0, 0.0);

        // Build a two-point path
        let path = [(t0, flat.clone()), (t1, flat)];

        let dtau = Dt::proper_time_from_path(path).expect("path should be valid");
        assert_eq!(dtau, Dt::from_sec(86400, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn constant_gravitational_time_dilation_exact() {
        let t0 = tai(0);
        let t1 = tai(1000);

        // Constant rate of 0.9 (e.g. gravitational time dilation)
        let slow = Spacetime::new(0.9, 0.0, 0.0);
        let path = [(t0, slow.clone()), (t1, slow)];

        let dtau = Dt::proper_time_from_path(path).expect("valid path");
        assert_eq!(dtau, Dt::from_sec(900, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn constant_special_relativistic_time_dilation_exact() {
        let t0 = tai(0);
        let t1 = tai(500);

        // Spacetime with velocity β = 0.6 → proper time rate ≈ 0.8
        let moving = Spacetime::new(1.0, 0.6, 0.0);
        let path = [(t0, moving.clone()), (t1, moving)];

        let dtau = Dt::proper_time_from_path(path).expect("valid path");
        assert_eq!(dtau, Dt::from_sec(400, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn multi_segment_trapezoidal_with_varying_rate() {
        let t0 = tai(0);
        let t_mid = tai(300);
        let t1 = tai(600);

        // Three samples with a linearly decreasing rate
        let path = [
            (t0, Spacetime::new(1.00, 0.0, 0.0)),
            (t_mid, Spacetime::new(0.95, 0.0, 0.0)),
            (t1, Spacetime::new(0.90, 0.0, 0.0)),
        ];
        let dtau = Dt::proper_time_from_path(path).expect("valid path");

        // With piecewise trapezoidal integration, the result is 570 seconds
        assert_eq!(dtau, Dt::from_sec(570, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn multi_segment_trapezoidal_with_odd_number_of_intervals() {
        // 4 samples → 3 segments (odd number of intervals)
        let path = [
            (tai(0), Spacetime::new(1.00, 0.0, 0.0)),
            (tai(300), Spacetime::new(0.96, 0.0, 0.0)),
            (tai(600), Spacetime::new(0.92, 0.0, 0.0)),
            (tai(900), Spacetime::new(0.88, 0.0, 0.0)),
        ];

        let dtau = Dt::proper_time_from_path(path).expect("valid path");

        // Piecewise trapezoidal result over 3 segments
        assert_eq!(dtau, Dt::from_sec(846, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn drift_equals_proper_time_minus_coordinate_time() {
        let t0 = tai(0);
        let t1 = tai(1000);

        let slow = Spacetime::new(0.9, 0.0, 0.0);
        let path = [(t0, slow.clone()), (t1, slow)];

        let dtau = Dt::proper_time_from_path(path).unwrap();
        let dt = t1.to_diff_raw(t0);

        assert_eq!(dtau.sub(dt), Dt::from_sec(-100, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn planck_saturation_activates_in_extreme_curvature() {
        let t0 = tai(0);
        let t1 = tai(100);

        let extreme = Spacetime::new(0.5, 0.0, 1e200);
        let path = [(t0, extreme.clone()), (t1, extreme)];

        let dtau = Dt::proper_time_from_path(path).expect("valid path");

        // K_eff saturates at ≈ 0.9013878188659973
        let expected = Dt::from_sec_f(90.13878188659973, Scale::TAI, Scale::TAI);
        assert_eq!(dtau, expected);
    }

    #[test]
    fn planck_term_is_negligible_in_solar_system_regimes() {
        let t0 = tai(0);
        let t1 = tai(86400);

        let alpha = 0.999_999;
        let beta = 1.2e-4;

        let no_k = Spacetime::new(alpha, beta, 0.0);
        let with_k = Spacetime::new(alpha, beta, 1e20);

        let path_no_k = [(t0, no_k.clone()), (t1, no_k)];
        let path_with_k = [(t0, with_k.clone()), (t1, with_k)];

        let dtau_no_k = Dt::proper_time_from_path(path_no_k).unwrap();
        let dtau_with_k = Dt::proper_time_from_path(path_with_k).unwrap();

        let diff_attos = dtau_no_k.sub(dtau_with_k).to_attos().abs();
        assert!(diff_attos < 10);
    }

    #[test]
    fn proper_time_from_path_handles_empty_or_single_point_path() {
        let empty: &[(Dt, Spacetime)] = &[];
        assert_eq!(
            Dt::proper_time_from_path(empty.iter().cloned()),
            Ok(Dt::ZERO)
        );

        let single = &[(tai(0), Spacetime::new(1.0, 0.0, 0.0))];
        assert_eq!(
            Dt::proper_time_from_path(single.iter().cloned()),
            Ok(Dt::ZERO)
        );
    }

    #[test]
    fn proper_time_from_path_consistent_with_two_point_path() {
        let t0 = tai(0);
        let t1 = tai(300);
        let ls = Spacetime::new(0.95, 0.0, 0.0);

        let via_path = Dt::proper_time_from_path([(t0, ls.clone()), (t1, ls)]).unwrap();

        // With constant rate, this should equal rate * Δt
        let expected = Dt::from_sec_f(0.95 * 300.0, Scale::TAI, Scale::TAI);
        assert_eq!(via_path, expected);
    }

    /// Smoke test at Apollo 12 mission duration: a synthetic path with GET times
    /// taken from Lavery TN D-6681 Table 1 should produce a spacecraft–ground
    /// differential of the same order as the published splashdown figure
    /// (+570.345 μs). Alphas/betas are approximate and partly tuned — this is
    /// not a reconstruction from positions and potentials.
    ///
    /// Source: https://ntrs.nasa.gov/api/citations/19720022040/downloads/19720022040.pdf
    #[test]
    fn apollo_12_scale_clock_differential() {
        // Lavery Table 1 cumulative times (GET, s) and published Δ at splashdown.
        // GET (s)     | Event           | Cumulative Δ (μs)
        // 10 380.0    | Begin TLC       | 0.0
        // 247 080.0   | TLC             | 153.797
        // 425 580.0   | Lunar orbit     | 271.328
        // 514 380.0   | Lunar orbit     | 329.228
        // 671 280.0   | TEC             | 434.334
        // 879 769.6   | Reentry         | 570.430
        // 880 557.6   | Splashdown      | 570.345

        let total_duration: i64 = 880_558; // splashdown GET ≈ 880557.6 s
        let num_samples: usize = 2_500;

        // Synthetic (α, β) milestones; GET times include Table 1 events plus
        // rough pre-TLC points. Not NASA trajectory-tape samples.
        let milestones = [
            (0, 0.999_999_999_326, 2.60e-5),
            (4_000, 0.999_999_999_42, 1.80e-5),
            (10_380, 0.999_999_999_65, 1.05e-5),
            (247_080, 0.999_999_999_9997, 1.40e-6),
            (425_580, 0.999_999_999_9997, 4.10e-6),
            (514_380, 0.999_999_999_9997, 4.10e-6),
            (671_280, 0.999_999_999_9997, 1.55e-6),
            (879_770, 0.999_999_999_74, 2.30e-5),
            (total_duration, 0.999_999_999_305, 0.0),
        ];

        let mut spacecraft_path: Vec<(Dt, Spacetime)> = Vec::with_capacity(num_samples);

        // Constant coast α chosen so the integrated differential falls near
        // the published 570.345 μs figure (order-of-magnitude check only).
        let coast_alpha = 0.999_999_999_9969685_f64;

        for i in 0..num_samples {
            let t = (i as f64 / (num_samples - 1) as f64) * total_duration as f64;

            let mut alpha = 0.999_999_999_9997_f64;
            let mut beta = 1.60e-6_f64;

            for j in 0..milestones.len() - 1 {
                let (t0, a0, b0) = milestones[j];
                let (t1, a1, b1) = milestones[j + 1];
                if t >= t0 as f64 && t <= t1 as f64 {
                    let frac = (t - t0 as f64) / (t1 as f64 - t0 as f64);
                    alpha = a0 + frac * (a1 - a0);
                    beta = b0 + frac * (b1 - b0);
                    break;
                }
            }

            if (30_000.0..879_000.0).contains(&t) {
                alpha = coast_alpha;
            }

            let ls = Spacetime::new(alpha, beta, 0.0);
            spacecraft_path.push((tai(t as i128), ls));
        }

        let spacecraft_dtau = Dt::proper_time_from_path(spacecraft_path.iter().cloned())
            .expect("spacecraft path should be valid");

        let t0 = spacecraft_path[0].0;
        let t1 = spacecraft_path.last().unwrap().0;
        let ground_ls = Spacetime::new(0.999_999_999_305, 0.0, 0.0);

        let ground_dtau = t0.proper_time_between_constant_rate(t1, ground_ls.proper_time_rate());

        let differential = spacecraft_dtau.sub(ground_dtau);
        let differential_us = differential.to_sec_f() * 1e6_f64;

        assert!(
            differential.to_sec_f() > 0.0,
            "expected spacecraft proper time ahead of ground over this path"
        );

        // Loose band around Lavery's +570.345 μs splashdown figure.
        assert!(
            (568.0..573.0).contains(&differential_us),
            "differential was {:.3} μs (expected roughly +570 μs)",
            differential_us
        );

        let mut summed = Dt::ZERO;
        for w in spacecraft_path.windows(2) {
            let segment = vec![w[0].clone(), w[1].clone()];
            let seg = Dt::proper_time_from_path(segment).expect("segment should be valid");
            summed = summed.add(seg);
        }
        assert_eq!(
            spacecraft_dtau, summed,
            "proper_time_from_path must equal manual segment sum"
        );
    }

    // =====================================================================
    // proper_time_drift_from_states
    // =====================================================================

    #[test]
    fn drift_from_states_matches_full_path_when_endpoints_align() {
        let t0 = tai(0);
        let t1 = tai(1000);
        let phi = phi_for_alpha(0.9);
        let states = [
            (t0, Velocity::ZERO, phi),
            (t1, Velocity::ZERO, phi),
        ];

        let drift = Dt::proper_time_drift_from_states(t0, t1, states, 0.0).unwrap();
        // dτ = 0.9 * 1000, Δt = 1000 → drift = −100
        assert_eq!(drift, Dt::from_sec(-100, Scale::TAI, Scale::TAI));

        let dtau = Dt::proper_time_from_states(states, 0.0).unwrap();
        assert_eq!(drift, dtau.sub(t1.to_diff_raw(t0)));
    }

    #[test]
    fn drift_from_states_windows_to_requested_interval() {
        let phi = phi_for_alpha(0.9);
        // Samples span [0, 1000]; request only [100, 900]
        let states = [
            (tai(0), Velocity::ZERO, phi),
            (tai(1000), Velocity::ZERO, phi),
        ];

        let drift =
            Dt::proper_time_drift_from_states(tai(100), tai(900), states, 0.0).unwrap();
        // Window Δt = 800; dτ = 0.9 * 800 → drift = −80
        assert_eq!(drift, Dt::from_sec(-80, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn drift_from_states_ignores_samples_outside_window() {
        let phi = phi_for_alpha(0.9);
        let states = [
            (tai(0), Velocity::ZERO, phi),
            (tai(100), Velocity::ZERO, phi),
            (tai(900), Velocity::ZERO, phi),
            (tai(1000), Velocity::ZERO, phi),
        ];

        let windowed =
            Dt::proper_time_drift_from_states(tai(100), tai(900), states, 0.0).unwrap();
        let exact = Dt::proper_time_drift_from_states(
            tai(100),
            tai(900),
            [
                (tai(100), Velocity::ZERO, phi),
                (tai(900), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap();

        assert_eq!(windowed, exact);
        assert_eq!(windowed, Dt::from_sec(-80, Scale::TAI, Scale::TAI));
    }

    #[test]
    fn drift_from_states_zero_interval_returns_zero() {
        let t = tai(42);
        let phi = phi_for_alpha(0.9);
        // Even with empty states, start == end is zero drift.
        assert_eq!(
            Dt::proper_time_drift_from_states(t, t, std::iter::empty(), 0.0),
            Ok(Dt::ZERO)
        );
        assert_eq!(
            Dt::proper_time_drift_from_states(
                t,
                t,
                [(t, Velocity::ZERO, phi)],
                0.0
            ),
            Ok(Dt::ZERO)
        );
    }

    #[test]
    fn drift_from_states_rejects_inverted_interval() {
        let phi = phi_for_alpha(1.0);
        let err = Dt::proper_time_drift_from_states(
            tai(100),
            tai(0),
            [
                (tai(0), Velocity::ZERO, phi),
                (tai(100), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap_err();
        assert_eq!(err.kind(), DtErrKind::OutOfRange);
    }

    #[test]
    fn drift_from_states_rejects_incomplete_coverage() {
        let phi = phi_for_alpha(0.9);

        // Empty
        let err = Dt::proper_time_drift_from_states(
            tai(0),
            tai(100),
            std::iter::empty(),
            0.0,
        )
        .unwrap_err();
        assert_eq!(err.kind(), DtErrKind::Incomplete);

        // First sample after start
        let err = Dt::proper_time_drift_from_states(
            tai(0),
            tai(100),
            [
                (tai(10), Velocity::ZERO, phi),
                (tai(100), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap_err();
        assert_eq!(err.kind(), DtErrKind::Incomplete);

        // Path ends before `end`
        let err = Dt::proper_time_drift_from_states(
            tai(0),
            tai(100),
            [
                (tai(0), Velocity::ZERO, phi),
                (tai(50), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap_err();
        assert_eq!(err.kind(), DtErrKind::Incomplete);
    }

    #[test]
    fn drift_from_states_rejects_non_monotonic() {
        let phi = phi_for_alpha(0.9);
        let err = Dt::proper_time_drift_from_states(
            tai(0),
            tai(100),
            [
                (tai(0), Velocity::ZERO, phi),
                (tai(80), Velocity::ZERO, phi),
                (tai(40), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap_err();
        assert_eq!(err.kind(), DtErrKind::NonMonotonic);
    }

    #[test]
    fn drift_from_states_flat_spacetime_is_zero() {
        let phi = phi_for_alpha(1.0);
        let drift = Dt::proper_time_drift_from_states(
            tai(0),
            tai(86400),
            [
                (tai(0), Velocity::ZERO, phi),
                (tai(86400), Velocity::ZERO, phi),
            ],
            0.0,
        )
        .unwrap();
        assert_eq!(drift, Dt::ZERO);
    }
}
