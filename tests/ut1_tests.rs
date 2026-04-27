#[cfg(feature = "ut1-tests")]
#[cfg(test)]
mod tests {
    use deep_time_core::{ClockType, EopFormat, Separator, TimePoint, TimeSpan, Ut1Provider};
    use std::eprintln;

    #[test]
    fn test_ut1_with_finals_all_iau2000_txt() {
        // CHANGE THIS PATH to the actual location of your finals2000A.all (or finals.all) file
        let path = "finals.all.iau2000.txt";

        let provider = Ut1Provider::from_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load real EOP file");

        // The exact row you gave us: MJD 56879.00 → DUT1 = -0.3170554
        let mjd = 56879.0;
        let dut1_expected = -0.3170554;

        // Verify the parser + provider correctly read that row
        let dut1 = provider
            .ut1_minus_utc(mjd)
            .expect("MJD 56879.00 should be in range");
        assert!(
            (dut1 - dut1_expected).abs() < 1e-9,
            "provider.ut1_minus_utc(56879.0) should be ≈ -0.3170554, got {dut1}"
        );

        // Create a UTC TimePoint at exactly that MJD (midnight)
        let utc = TimePoint::from_mjd_utc_exact(56879, TimeSpan::ZERO);

        // === Test to_ut1 ===
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");
        assert_eq!(
            ut1.clock_type(),
            ClockType::Custom,
            "UT1 should be stored as Custom"
        );

        // The numerical value of the Custom TimePoint is UTC + DUT1.
        // duration_since() computes physical time (via TAI), so we re-interpret
        // the Custom value as UTC to see the clock difference (DUT1).
        let diff = ut1.with_clock_type(ClockType::UTC).duration_since(utc);
        assert!(
            (diff.as_sec_f() - dut1_expected).abs() < 1e-9,
            "to_ut1 applied wrong DUT1: expected {}, got {}",
            dut1_expected,
            diff.as_sec_f()
        );

        // === Test from_ut1 (full round-trip) ===
        let back_to_utc = TimePoint::from_ut1(ut1, &provider).expect("from_ut1 failed");
        assert_eq!(back_to_utc.clock_type(), ClockType::UTC);

        let roundtrip_diff = utc.duration_since(back_to_utc);
        assert!(
            roundtrip_diff.as_sec_f().abs() < 1e-9,
            "round-trip should be exact within floating-point tolerance, got {}",
            roundtrip_diff.as_sec_f()
        );

        eprintln!("✅ UT1 round-trip test passed for MJD 56879.00 (DUT1 ≈ {dut1})");
    }

    #[test]
    fn test_ut1_with_finals2000a_all_txt() {
        // Same real EOP file used by the other test (finals2000A.all / finals.all.iau2000.txt)
        // CHANGE THIS PATH if your file lives somewhere else
        let path = "finals2000A.all.txt";

        let provider = Ut1Provider::from_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load real EOP file for MJD 60961.00 test");

        // The exact row you provided:
        // 251013 60961.00 I 0.208777 ... I 0.0933562 ...
        // Our Finals2000A parser correctly extracts DUT1 = 0.0933562
        let mjd = 60961.0;
        let dut1_expected = 0.0933562f64;

        // 1. Verify the parser + lookup gives the correct DUT1
        let dut1 = provider
            .ut1_minus_utc(mjd)
            .expect("MJD 60961.00 should be inside the loaded EOP table");

        assert!(
            (dut1 - dut1_expected).abs() < 1e-9,
            "provider.ut1_minus_utc(60961.0) should be ≈ 0.0933562, got {dut1}"
        );

        // 2. Create exact UTC TimePoint at MJD 60961.0 00:00:00 (midnight)
        let utc = TimePoint::from_mjd_utc_exact(60961, TimeSpan::ZERO);

        // 3. to_ut1 (uses exact MJD path internally)
        let ut1 = utc
            .to_ut1(&provider)
            .expect("to_ut1 failed for MJD 60961.00");

        assert_eq!(
            ut1.clock_type(),
            ClockType::Custom,
            "UT1 should be stored as Custom"
        );

        // Verify the numerical difference is exactly the DUT1 we expect
        let diff = ut1.with_clock_type(ClockType::UTC).duration_since(utc);
        assert!(
            (diff.as_sec_f() - dut1_expected).abs() < 1e-10,
            "to_ut1 applied wrong DUT1: expected {}, got {}",
            dut1_expected,
            diff.as_sec_f()
        );

        // 4. Round-trip back to UTC using the exact fixed-point iteration version
        let back_to_utc = TimePoint::from_ut1(ut1, &provider).expect("from_ut1 failed");

        assert_eq!(back_to_utc.clock_type(), ClockType::UTC);

        let roundtrip_diff = utc.duration_since(back_to_utc);
        assert!(
            roundtrip_diff.as_sec_f().abs() < 1e-10,
            "UT1 ↔ UTC round-trip should be exact within machine precision, got {}",
            roundtrip_diff.as_sec_f()
        );

        eprintln!(
            "✅ Specific EOP row test passed for MJD 60961.00 (DUT1 ≈ {:.7})",
            dut1
        );
    }

    #[test]
    fn test_ut1_c04_specific_row_57259() {
        let path = "EOP_20u24_C04_one_file_1962-now.txt"; // ← change to your C04 file

        let provider = Ut1Provider::from_file(path, EopFormat::C04, Separator::Whitespace)
            .expect("failed to load C04 file");

        let mjd = 57259.0;
        let dut1_expected = 0.2813082;

        // 1. Test the provider directly (this is what a user would do)
        let dut1 = provider
            .ut1_minus_utc(mjd)
            .expect("MJD 57259 should be in the C04 table");

        assert!(
            (dut1 - dut1_expected).abs() < 1e-9,
            "C04 parser gave wrong DUT1: expected {}, got {}",
            dut1_expected,
            dut1
        );

        // 2. Create exact UTC midnight
        let utc = TimePoint::from_mjd_utc_exact(57259, TimeSpan::ZERO);

        // 3. User-style round-trip (the way real code uses it)
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");
        let back_to_utc = TimePoint::from_ut1(ut1, &provider).expect("from_ut1 failed");

        // 4. Verify round-trip is exact (within floating-point tolerance)
        let roundtrip_error = utc.duration_since(back_to_utc).as_sec_f();
        assert!(
            roundtrip_error.abs() < 1e-10,
            "C04 round-trip error too large: {} s",
            roundtrip_error
        );

        eprintln!("✅ C04 row 57259.00 passed (DUT1 ≈ {:.7})", dut1);
    }
}
