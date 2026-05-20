#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "eop-tests")]
#[cfg(test)]
mod tests {
    use deep_time::constants::{ATTOS_PER_DAY, SEC_PER_DAY_F};
    use deep_time::eop::{EopData, EopFormat, Separator};
    use deep_time::{Dt, Scale};

    #[test]
    fn test_ut1_with_finals_all_iau2000_txt() {
        // CHANGE THIS PATH to the actual location of your finals2000A.all (or finals.all) file
        let path = "finals.all.iau2000.txt";

        let provider = EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load real EOP file");

        // The exact row you gave us: MJD 56879.00 → DUT1 = -0.3170554
        let mjd = 56879.0;
        let dut1_expected = -0.3170554;

        // Verify the parser + provider correctly read that row
        let dut1 = provider
            .eop_offset(mjd)
            .expect("MJD 56879.00 should be in range")
            .offset;
        assert_eq!(
            dut1, dut1_expected,
            "provider.offset(56879.0) should be ≈ -0.3170554, got {dut1}"
        );

        // Create a Dt at exactly that MJD (midnight)
        let utc = Dt::from_mjd(56879, 0, Scale::TAI);

        // === Test to_ut1 ===
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");

        // The numerical value of the UT1 Dt is UTC + DUT1.
        // to_diff_raw() computes physical time (via TAI), so we re-interpret
        // the UT1 value as UTC to see the clock difference (DUT1).
        let diff = ut1.to_diff_raw(utc);
        assert!(
            (diff.to_sec_f() - dut1_expected).abs() < 1e-9,
            "to_ut1 applied wrong DUT1: expected {}, got {}",
            dut1_expected,
            diff.to_sec_f()
        );

        // === Test from_ut1 (full round-trip) ===
        let back_to_utc = ut1.from_eop(&provider).expect("from_ut1 failed");

        let roundtrip_diff = utc.to_diff_raw(back_to_utc);
        assert!(
            roundtrip_diff.to_sec_f().abs() < 1e-9,
            "round-trip should be exact within floating-point tolerance, got {}",
            roundtrip_diff.to_sec_f()
        );
    }

    #[test]
    fn test_ut1_with_finals2000a_all_txt() {
        // Same real EOP file used by the other test (finals2000A.all / finals.all.iau2000.txt)
        // CHANGE THIS PATH if your file lives somewhere else
        let path = "finals2000A.all.txt";

        let provider = EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load real EOP file for MJD 60961.00 test");

        // The exact row you provided:
        // 251013 60961.00 I 0.208777 ... I 0.0933562 ...
        // Our Finals2000A parser correctly extracts DUT1 = 0.0933562
        let mjd = 60961.0;
        let dut1_expected = 0.0933562f64;

        // 1. Verify the parser + lookup gives the correct DUT1
        let dut1 = provider
            .eop_offset(mjd)
            .expect("MJD 60961.00 should be inside the loaded EOP table")
            .offset;

        assert_eq!(
            dut1, dut1_expected,
            "provider.offset(60961.0) should be ≈ 0.0933562, got {dut1}"
        );

        // 2. Create exact Dt at MJD 60961.0 00:00:00 (midnight)
        let utc = Dt::from_mjd(60961, 0, Scale::TAI);

        // 3. to_ut1 (uses exact MJD path internally)
        let ut1 = utc
            .to_eop(&provider)
            .expect("to_ut1 failed for MJD 60961.00");

        // Verify the numerical difference is exactly the DUT1 we expect
        let diff = ut1.to_diff_raw(utc);
        assert!(
            (diff.to_sec_f() - dut1_expected).abs() < 1e-10,
            "to_ut1 applied wrong DUT1: expected {}, got {}",
            dut1_expected,
            diff.to_sec_f()
        );

        // 4. Round-trip back using the exact fixed-point iteration version
        let back_to_utc = ut1.from_eop(&provider).expect("from_ut1 failed");

        let roundtrip_diff = utc.to_diff_raw(back_to_utc);
        assert!(
            roundtrip_diff.to_sec_f().abs() < 1e-10,
            "UT1 ↔ UTC round-trip should be exact within machine precision, got {}",
            roundtrip_diff.to_sec_f()
        );
    }

    #[test]
    fn test_ut1_c04_specific_row_57259() {
        let path = "EOP_20u24_C04_one_file_1962-now.txt"; // ← change to your C04 file

        let provider = EopData::from_text_file(path, EopFormat::C04, Separator::Whitespace)
            .expect("failed to load C04 file");

        let mjd = 57259.0;
        let dut1_expected = 0.2813082;

        // 1. Test the provider directly (this is what a user would do)
        let dut1 = provider
            .eop_offset(mjd)
            .expect("MJD 57259 should be in the C04 table")
            .offset;

        assert_eq!(
            dut1, dut1_expected,
            "C04 parser gave wrong DUT1: expected {}, got {}",
            dut1_expected, dut1
        );

        // 2. Create exact UTC midnight
        let utc = Dt::from_mjd(57259, 0, Scale::UTC);

        // 3. User-style round-trip
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");
        let back_to_utc = ut1.from_eop(&provider).expect("from_ut1 failed");

        // 4. Verify round-trip is exact (within floating-point tolerance)
        let roundtrip_error = utc.to_diff_raw(back_to_utc).to_sec_f();
        assert!(
            roundtrip_error.abs() < 1e-10,
            "C04 round-trip error too large: {} s",
            roundtrip_error
        );
    }

    // ============================================================
    // Helper to load a provider (reuses your existing test data)
    // ============================================================
    fn load_finals2000a() -> EopData {
        let path = "finals.all.iau2000.txt";
        EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load finals2000A.all / finals.all.iau2000.txt")
    }

    // ============================================================
    // 1. Basic round-trip: UT1 → JD_UT1 → back to UT1
    // ============================================================
    #[test]
    fn test_jd_ut1_exact_roundtrip() {
        let provider = load_finals2000a();

        // Use a known good row (MJD 56879.00, DUT1 ≈ -0.3170554)
        let utc = Dt::from_mjd(56879, 0, Scale::UTC);
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");

        // Round-trip through JD_UT1
        let (jd_days, frac) = ut1.to_jd();
        let roundtrip = Dt::from_jd(jd_days, frac, Scale::Custom);

        assert_eq!(ut1.sec, roundtrip.sec);
        assert_eq!(ut1.attos, roundtrip.attos);

        // Physical round-trip check (via TAI)
        let diff = ut1.to_diff_raw(roundtrip);
        assert!(
            diff.to_sec_f().abs() < 1e-12,
            "JD_UT1 round-trip error too large: {} s",
            diff.to_sec_f()
        );
    }

    // ============================================================
    // 2. Full pipeline round-trip using real EOP data
    //    UTC → UT1 → JD_UT1 → back to UT1 → back to UTC
    // ============================================================
    #[test]
    fn test_full_pipeline_jd_ut1_roundtrip() {
        let provider = load_finals2000a();

        let original_utc = Dt::from_mjd(60961, 0, Scale::UTC); // known row
        let ut1 = original_utc.to_eop(&provider).expect("to_ut1 failed");

        // Convert to JD in UT1
        let (jd_days, frac) = ut1.to_jd();

        // Go back
        let ut1_back = Dt::from_jd(jd_days, frac, Scale::Custom);
        let utc_back = ut1_back.from_eop(&provider).expect("from_ut1 failed");

        // Final check: should be extremely close to original UTC
        let error = original_utc.to_diff_raw(utc_back).to_sec_f();
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

        let utc = Dt::from_mjd(57259, 0, Scale::UTC);
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");

        let (mjd_days, frac) = ut1.to_mjd();
        let roundtrip = Dt::from_mjd(mjd_days, frac, Scale::Custom);

        assert_eq!(ut1.sec, roundtrip.sec);
        assert_eq!(ut1.attos, roundtrip.attos);

        let diff = ut1.to_diff_raw(roundtrip).to_sec_f();
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
        let utc = Dt::from_mjd(56879, 0, Scale::UTC);
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");

        let (jd_ut1, frac_ut1_attos) = ut1.to_jd();
        let (jd_utc, frac_utc_attos) = utc.to_jd();

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
        let utc = Dt::from_mjd(60961, 12 * 3600, Scale::UTC);
        let ut1 = utc.to_eop(&provider).expect("to_ut1 failed");

        let (jd_days, frac2) = ut1.to_jd();
        let roundtrip = Dt::from_jd(jd_days, frac2, Scale::Custom);

        let diff = ut1.to_diff_raw(roundtrip).to_sec_f();
        assert!(
            diff.abs() < 1e-11,
            "Fractional day round-trip error: {} s",
            diff
        );
    }
}
