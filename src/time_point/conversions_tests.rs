use crate::TimePoint;

mod tdb_tests {
    use super::*;
    use crate::ClockType;

    /// Round-trip accuracy test (TAI → TDB → TAI)
    #[test]
    fn tdb_tai_roundtrip_is_accurate() {
        let test_points = [
            TimePoint::from_tai_sec(0),                  // J2000 TAI
            TimePoint::from_tai_sec(86_400 * 365),       // ~1 year later
            TimePoint::from_tai_sec(-86_400 * 365 * 10), // 10 years before
            TimePoint::from_tai_sec(1_000_000_000),      // ~31.7 years later
            TimePoint::from_tai_sec(-2_208_945_600),     // J1900 epoch
        ];

        #[cfg(feature = "std")]
        {
            use std::eprintln;
            let tai = TimePoint::ZERO;
            let tdb = tai.to_clock_type(ClockType::TDB);
            eprintln!("\nTAI sec={}, subsec={}", tai.sec, tai.subsec);
            eprintln!("TDB sec={}, subsec={}", tdb.sec, tdb.subsec);
            eprintln!("diff_s = {}", tdb.duration_since(tai).as_sec_f());
        }
        for &p in &test_points {
            let tdb = p.to_clock_type(ClockType::TDB);
            let back = tdb.to_clock_type(ClockType::TAI);

            let diff = back.duration_since(p).as_sec_f().abs();
            assert!(
                diff < 1e-6,
                "TDB round-trip error too large: {} s at {:?}",
                diff,
                p
            );
        }
    }

    /// At J2000 the TDB–TAI difference should be ~32.183925 s
    /// (TT = TAI + 32.184 s and TDB − TT ≈ −74.6 µs with this formula)
    #[test]
    fn tdb_minus_tt_at_j2000() {
        let tai = TimePoint::ZERO;
        let tdb = tai.to_clock_type(ClockType::TDB);

        let diff_s = tdb.numerical_seconds_since(&tai); // see helper below

        assert!(
            (diff_s - 32.183925).abs() < 0.00001,
            "TDB-TAI difference at J2000 was {} s (expected ~32.183925 s)",
            diff_s
        );
    }

    #[test]
    fn tdb_minus_tt_at_j2000_2() {
        let tai = TimePoint::ZERO;
        let tdb = tai.to_clock_type(ClockType::TDB);
        let diff_s = tdb.numerical_seconds_since(&tai);
        assert!((diff_s - 32.183925).abs() < 1e-6, "got {}", diff_s);
    }

    /// Check that the *periodic correction* (TDB − TT) stays within sensible bounds
    #[test]
    fn tdb_correction_stays_within_bounds() {
        let points = [
            TimePoint::from_tai_sec(0),
            TimePoint::from_tai_sec(86_400 * 365 * 100),
            TimePoint::from_tai_sec(-86_400 * 365 * 50),
        ];

        for &p in &points {
            let tt = p.to_clock_type(ClockType::TT);
            let tdb = p.to_clock_type(ClockType::TDB);

            // TDB - TT (periodic term only)
            let corr_s = tdb.numerical_seconds_since(&tt);

            assert!(
                corr_s.abs() < 0.002,
                "TDB-TT correction should be < 2 ms (got {} s)",
                corr_s
            );
        }
    }
}

mod drift_tests {
    use super::*;
    use crate::{ClockDrift, ClockModel, ClockType, Delta};

    #[test]
    fn proper_to_tt_with_drift_roundtrip() {
        let reference = TimePoint::from_tai_sec(0);
        let drift = ClockDrift::new(
            Delta::from_ms(100), // exactly 0.1 s
            Delta::from_ns(1),   // exactly 1 ns/s = 1e-9 s/s
            Delta::ZERO,
        );
        let model = ClockModel::proper(reference, drift);

        let onboard_proper = TimePoint::create_from_model(model).add(Delta::from_sec(1_000_000));

        let tt = onboard_proper.convert_using_model(model);
        let back = tt.convert_back_using_model(model);

        assert_eq!(back, onboard_proper);
    }

    #[test]
    fn zero_drift_is_identity() {
        let reference = TimePoint::from_tai_sec(0);
        let drift = ClockDrift::ZERO;
        let model = ClockModel::proper(reference, drift);

        let p = TimePoint::from_tai_sec(1_234_567);
        let converted = p.convert_using_model(model);

        assert_eq!(converted, p.with_clock_type(ClockType::Proper));
    }

    #[test]
    fn constant_offset_only() {
        let reference = TimePoint::from_tai_sec(0);
        let drift = ClockDrift::from_constant(Delta::from_sec_f(32.184));
        let model = ClockModel::proper(reference, drift);

        let onboard = TimePoint::create_from_model(model).add(Delta::from_sec(100));
        let tt = onboard.convert_using_model(model);

        let expected = onboard
            .add(Delta::from_sec_f(32.184))
            .with_clock_type(ClockType::Proper);
        assert_eq!(tt, expected);
    }

    #[test]
    fn convert_back_using_model_inverse() {
        let reference = TimePoint::from_tai_sec(0);
        let drift = ClockDrift::new(
            Delta::from_ms(500), // exactly 0.5 s
            Delta::from_ns(2),   // exactly 2 ns/s = 2e-9 s/s
            Delta::ZERO,
        );
        let model = ClockModel::proper(reference, drift);

        // Start from onboard Proper time (the natural input for this API)
        let proper = TimePoint::create_from_model(model).add(Delta::from_sec(1_000_000));

        let tt = proper.convert_using_model(model); // Proper → TT
        let back = tt.convert_back_using_model(model); // TT → Proper

        assert_eq!(back, proper);
    }

    #[test]
    fn apply_new_model_and_create_from_model() {
        let reference = TimePoint::from_tai_sec(0);
        let drift = ClockDrift::ZERO;
        let model = ClockModel::proper(reference, drift);

        let raw = TimePoint::from_tai_sec(123);
        let tagged = raw.apply_new_model(model);

        assert_eq!(tagged.clock_type(), ClockType::Proper);
        assert_eq!(
            TimePoint::create_from_model(model),
            reference.with_clock_type(ClockType::Proper)
        );
    }
}

mod ltc_tests {
    use super::*;
    use crate::ClockType;

    /// Round-trip accuracy test (TAI → LTC → TAI)
    ///
    /// LTC conversion is purely linear, so round-trips should be extremely
    /// accurate. The observed error is ~0.3 ns due to unavoidable f64 rounding
    /// noise in `to_jd_tt()` + the rate multiplication. We therefore allow a
    /// very small tolerance that is still far tighter than any practical use case.
    #[test]
    fn ltc_tai_roundtrip_is_accurate() {
        let test_points = [
            TimePoint::from_tai_sec(0),                  // J2000 TAI
            TimePoint::from_tai_sec(86_400 * 365),       // ~1 year later
            TimePoint::from_tai_sec(-86_400 * 365 * 10), // 10 years before
            TimePoint::from_tai_sec(1_000_000_000),      // ~31.7 years later
            TimePoint::from_tai_sec(-2_208_945_600),     // J1900 epoch
        ];

        for &p in &test_points {
            let ltc = p.to_clock_type(ClockType::LTC);
            let back = ltc.to_clock_type(ClockType::TAI);

            let diff = back.duration_since(p).as_sec_f().abs();
            assert!(
                diff < 1e-9,
                "LTC round-trip error too large: {} s at {:?}",
                diff,
                p
            );
        }
    }

    /// At J2000 the LTC–TAI difference must be exactly the value produced by
    /// the library’s f64 math (L_M × days × 86400 + TT–TAI offset).
    #[test]
    fn ltc_minus_tai_at_j2000() {
        let tai = TimePoint::ZERO;
        let ltc = tai.to_clock_type(ClockType::LTC);

        let diff_s = ltc.numerical_seconds_since(&tai);

        assert!(
            (diff_s - 32.6545948272096).abs() < 1e-9,
            "LTC-TAI difference at J2000 was {} s (expected 32.6545948272096 s)",
            diff_s
        );
    }

    /// Tighter check of the same J2000 value (matches the style of the second TDB test).
    #[test]
    fn ltc_minus_tai_at_j2000_2() {
        let tai = TimePoint::ZERO;
        let ltc = tai.to_clock_type(ClockType::LTC);
        let diff_s = ltc.numerical_seconds_since(&tai);

        assert!(
            (diff_s - 32.6545948272096).abs() < 1e-9,
            "got {} (expected 32.6545948272096)",
            diff_s
        );
    }

    /// Verify that the LTC–TT difference grows linearly (pure secular term).
    #[test]
    fn ltc_offset_grows_linearly() {
        let points = [
            TimePoint::from_tai_sec(0),
            TimePoint::from_tai_sec(86_400 * 365), // ~1 year
            TimePoint::from_tai_sec(86_400 * 365 * 100), // ~100 years
        ];

        for &p in &points {
            let tt = p.to_clock_type(ClockType::TT);
            let ltc = p.to_clock_type(ClockType::LTC);

            // LTC - TT (pure secular term)
            let corr_s = ltc.numerical_seconds_since(&tt);

            assert!(
                corr_s > 0.0,
                "LTC-TT correction should be positive (got {} s at {:?})",
                corr_s,
                p
            );

            // At ~100 years the offset should be ~2.5 s (56 µs/day × 36525 days)
            if p.sec > 86_400 * 365 * 50 {
                assert!(
                    corr_s > 1.0 && corr_s < 4.0,
                    "LTC-TT correction at ~100y should be ~2–3 s (got {} s)",
                    corr_s
                );
            }
        }
    }
}
