#[cfg(feature = "ut1-tests")]
#[cfg(test)]
mod tests {
    use deep_time::constants::{ATTOS_PER_DAY, SEC_PER_DAY_F};
    use deep_time::{ClockType, Separator, TimePoint, Ut1Data, Ut1Format};

    #[test]
    fn test_ut1_with_finals_all_iau2000_txt() {
        // CHANGE THIS PATH to the actual location of your finals2000A.all (or finals.all) file
        let path = "finals.all.iau2000.txt";

        let provider = Ut1Data::from_text_file(path, Ut1Format::Finals2000A, Separator::Whitespace)
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
        let utc = TimePoint::from_mjd_exact(56879, 0, ClockType::UTC);

        // === Test to_ut1 ===
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");
        assert_eq!(
            ut1.clock_type(),
            ClockType::UT1,
            "UT1 should be stored as UT1"
        );

        // The numerical value of the UT1 TimePoint is UTC + DUT1.
        // duration_since() computes physical time (via TAI), so we re-interpret
        // the UT1 value as UTC to see the clock difference (DUT1).
        let diff = ut1.with_type(ClockType::UTC).duration_since(utc);
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
    }

    #[test]
    fn test_ut1_with_finals2000a_all_txt() {
        // Same real EOP file used by the other test (finals2000A.all / finals.all.iau2000.txt)
        // CHANGE THIS PATH if your file lives somewhere else
        let path = "finals2000A.all.txt";

        let provider = Ut1Data::from_text_file(path, Ut1Format::Finals2000A, Separator::Whitespace)
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
        let utc = TimePoint::from_mjd_exact(60961, 0, ClockType::UTC);

        // 3. to_ut1 (uses exact MJD path internally)
        let ut1 = utc
            .to_ut1(&provider)
            .expect("to_ut1 failed for MJD 60961.00");

        assert_eq!(
            ut1.clock_type(),
            ClockType::UT1,
            "UT1 should be stored as UT1"
        );

        // Verify the numerical difference is exactly the DUT1 we expect
        let diff = ut1.with_type(ClockType::UTC).duration_since(utc);
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
    }

    #[test]
    fn test_ut1_c04_specific_row_57259() {
        let path = "EOP_20u24_C04_one_file_1962-now.txt"; // ← change to your C04 file

        let provider = Ut1Data::from_text_file(path, Ut1Format::C04, Separator::Whitespace)
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
        let utc = TimePoint::from_mjd_exact(57259, 0, ClockType::UTC);

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
    }

    // ============================================================
    // Helper to load a provider (reuses your existing test data)
    // ============================================================
    fn load_finals2000a() -> Ut1Data {
        let path = "finals.all.iau2000.txt";
        Ut1Data::from_text_file(path, Ut1Format::Finals2000A, Separator::Whitespace)
            .expect("failed to load finals2000A.all / finals.all.iau2000.txt")
    }

    // ============================================================
    // 1. Basic round-trip: UT1 → JD_UT1 → back to UT1
    // ============================================================
    #[test]
    fn test_jd_ut1_exact_roundtrip() {
        let provider = load_finals2000a();

        // Use a known good row (MJD 56879.00, DUT1 ≈ -0.3170554)
        let utc = TimePoint::from_mjd_exact(56879, 0, ClockType::UTC);
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");

        // Round-trip through JD_UT1
        let (jd_days, frac) = ut1.to_jd_exact();
        let roundtrip = TimePoint::from_jd_exact(jd_days, frac, ClockType::UT1);

        assert_eq!(roundtrip.clock_type(), ClockType::UT1);
        assert_eq!(ut1.sec(), roundtrip.sec());
        assert_eq!(ut1.subsec(), roundtrip.subsec());

        // Physical round-trip check (via TAI)
        let diff = ut1.duration_since(roundtrip);
        assert!(
            diff.as_sec_f().abs() < 1e-12,
            "JD_UT1 round-trip error too large: {} s",
            diff.as_sec_f()
        );
    }

    // ============================================================
    // 2. Full pipeline round-trip using real EOP data
    //    UTC → UT1 → JD_UT1 → back to UT1 → back to UTC
    // ============================================================
    #[test]
    fn test_full_pipeline_jd_ut1_roundtrip() {
        let provider = load_finals2000a();

        let original_utc = TimePoint::from_mjd_exact(60961, 0, ClockType::UTC); // known row
        let ut1 = original_utc.to_ut1(&provider).expect("to_ut1 failed");

        // Convert to JD in UT1
        let (jd_days, frac) = ut1.to_jd_exact();

        // Go back
        let ut1_back = TimePoint::from_jd_exact(jd_days, frac, ClockType::UT1);
        let utc_back = TimePoint::from_ut1(ut1_back, &provider).expect("from_ut1 failed");

        // Final check: should be extremely close to original UTC
        let error = original_utc.duration_since(utc_back).as_sec_f();
        assert!(
            error.abs() < 1e-10,
            "Full pipeline round-trip error too large: {} s",
            error
        );
    }

    // ============================================================
    // 3. MJD_UT1 round-trip
    // ============================================================
    #[test]
    fn test_mjd_ut1_exact_roundtrip() {
        let provider = load_finals2000a();

        let utc = TimePoint::from_mjd_exact(57259, 0, ClockType::UTC);
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");

        let (mjd_days, frac) = ut1.to_mjd_exact();
        let roundtrip = TimePoint::from_mjd_exact(mjd_days, frac, ClockType::UT1);

        assert_eq!(roundtrip.clock_type(), ClockType::UT1);
        assert_eq!(ut1.sec(), roundtrip.sec());
        assert_eq!(ut1.subsec(), roundtrip.subsec());

        let diff = ut1.duration_since(roundtrip).as_sec_f();
        assert!(diff.abs() < 1e-12, "MJD_UT1 round-trip error: {} s", diff);
    }

    // ============================================================
    // 4. Cross-check: JD_UT1 vs JD_UTC should differ by DUT1 / 86400
    //    uses total JD to avoid noon wrap-around)
    // ============================================================
    #[test]
    fn test_jd_ut1_vs_jd_utc_consistency() {
        let provider = load_finals2000a();
        let dut1_expected = -0.3170554; // known value for MJD 56879.00

        // Create exact UTC midnight using the modern constructor
        let utc = TimePoint::from_mjd_exact(56879, 0, ClockType::UTC);
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");

        // Get JD in both time scales (now both return (i64, u128))
        let (jd_ut1, frac_ut1_attos) = ut1.to_jd_exact();
        let (jd_utc, frac_utc_attos) = utc.to_jd_exact(); // modern main API

        // Convert attoseconds → fraction of day
        let total_jd_ut1 = jd_ut1 as f64 + (frac_ut1_attos as f64) / (ATTOS_PER_DAY as f64);
        let total_jd_utc = jd_utc as f64 + (frac_utc_attos as f64) / (ATTOS_PER_DAY as f64);

        let diff_days = total_jd_ut1 - total_jd_utc;
        let expected_diff = dut1_expected / SEC_PER_DAY_F;

        assert!(
            (diff_days - expected_diff).abs() < 1e-9,
            "JD difference mismatch: got {}, expected {}",
            diff_days,
            expected_diff
        );
    }

    // ============================================================
    // 5. Round-trip with non-zero fractional day
    // ============================================================
    #[test]
    fn test_jd_ut1_roundtrip_with_fractional_day() {
        let provider = load_finals2000a();

        // 12:00:00 UTC on a known day
        let utc = TimePoint::from_mjd_exact(60961, 12 * 3600, ClockType::UTC);
        let ut1 = utc.to_ut1(&provider).expect("to_ut1 failed");

        let (jd_days, frac2) = ut1.to_jd_exact();
        let roundtrip = TimePoint::from_jd_exact(jd_days, frac2, ClockType::UT1);

        let diff = ut1.duration_since(roundtrip).as_sec_f();
        assert!(
            diff.abs() < 1e-11,
            "Fractional day round-trip error: {} s",
            diff
        );
    }
}
