#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::civil_parts::Parts;
use deep_time::consts::{ATTOS_PER_SEC_I128, SEC_PER_DAY_I64};
use deep_time::{Dt, DtErrKind, Scale};

mod ccsds_tests {
    use super::*;

    // ====================== CUC ======================

    #[test]
    fn cuc_epoch() {
        let dt = Dt::CCSDS_EPOCH;
        let (buf, len) = dt.to_ccsds_cuc(4, 0, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x1C, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn cuc_one_second_after() {
        let mut dt = Dt::CCSDS_EPOCH.add_sec(1);
        let (buf, len) = dt.to_ccsds_cuc(4, 0, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x1C, 0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn cuc_fractional() {
        let dt = Dt::CCSDS_EPOCH.add_attos(500_000_000_000_000_000);
        let (buf, len) = dt.to_ccsds_cuc(1, 3, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x13, 0x00, 0x80, 0x00, 0x00]);
    }

    #[test]
    fn cuc_extension() {
        let dt = Dt::ZERO;
        let (buf, len) = dt.to_ccsds_cuc(5, 0, false).unwrap();
        assert_eq!(len, 7);
        assert_eq!(buf[0], 0x9C);
        assert_eq!(buf[1], 0x20);
    }

    // ====================== CDS ======================

    #[test]
    fn cds_epoch() {
        let dt = Dt::CCSDS_EPOCH;
        let (buf, len) = dt.to_ccsds_cds(2, 0, false).unwrap();
        assert_eq!(len, 7);
        assert_eq!(&buf[..len], &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn cds_n_day3_extension() {
        let dt = Dt::CCSDS_EPOCH;
        let (buf, len) = dt.to_ccsds_cds(3, 0, true).unwrap();
        assert_eq!(len, 9);
        assert_eq!(buf[0], 0xC4);
        assert_eq!(buf[1], 0x00);
    }

    #[test]
    fn cds_submillisecond() {
        let dt = Dt::CCSDS_EPOCH.add_attos(123_456_789_012_345_678);
        let (buf, len) = dt.to_ccsds_cds(2, 1, false).unwrap();
        assert_eq!(len, 9);
        assert_eq!(buf[0], 0x41);
    }

    // ====================== CCS ======================

    #[test]
    fn ccs_y2k_month_day() {
        let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let (buf, len) = dt.to_ccsds_ccs(false, 0).unwrap();
        assert_eq!(len, 8);
        let expected = [0x50, 0x20, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00];
        assert_eq!(&buf[..len], &expected[..]);
    }

    #[test]
    fn ccs_doy() {
        let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let (buf, len) = dt.to_ccsds_ccs(true, 0).unwrap();
        assert_eq!(len, 8);
        assert_eq!(buf[0], 0x58);
        assert_eq!(buf[3], 0x00);
        assert_eq!(buf[4], 0x01);
    }

    #[test]
    fn ccs_subsecond() {
        let dt =
            Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0).add_attos(123_456_789_012_345_678);
        let (buf, len) = dt.to_ccsds_ccs(false, 2).unwrap();
        assert_eq!(len, 10);
        assert_eq!(buf[0], 0x52);
    }

    // ====================== Error Cases ======================

    #[test]
    fn invalid_parameters() {
        let dt = Dt::ZERO;

        assert!(matches!(
            dt.to_ccsds_cuc(0, 0, false),
            Err(e) if e.kind() == DtErrKind::OutOfRange));

        assert!(matches!(
            dt.to_ccsds_cuc( 4, 11, false),
            Err(e) if e.kind() == DtErrKind::FracOutOfRange));

        assert!(matches!(
            dt.to_ccsds_cds( 1, 0, false),
            Err(e) if e.kind() == DtErrKind::InvalidNumber));

        assert!(matches!(
            dt.to_ccsds_cds( 2, 3, false),
            Err(e) if e.kind() == DtErrKind::InvalidSubmillisecond));

        assert!(matches!(
            dt.to_ccsds_ccs( false, 7),
            Err(e) if e.kind() == DtErrKind::FracOutOfRange));
    }

    // ====================== Convenience ======================

    #[test]
    fn to_ccsds_bin() {
        let tai = Dt::ZERO;
        let (buf, _) = tai.to_ccsds_bin().unwrap();
        assert_eq!(buf[0] & 0b0111_0000, 0b0001_0000);

        let utc = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let (buf, _) = utc.to_ccsds_bin().unwrap();
        assert_eq!(buf[0] & 0b0111_0000, 0b0100_0000);
    }

    // ====================== Parsing roundtrip tests ======================

    #[test]
    fn test_ccsds_c_direct_frac() {
        let c_bytes = &[0x15u8, 0x00, 0x01, 0x80];
        let parsed = Parts::from_ccsds_cuc(c_bytes).unwrap();

        assert_eq!(parsed.yr, Some(1958));
        assert_eq!(parsed.mo, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hr, 0);
        assert_eq!(parsed.min, 0);
        assert_eq!(parsed.sec, 1);
        assert!(parsed.attos > 499_000_000_000_000_000);
        assert_eq!(parsed.scale, Scale::TAI);
    }

    #[test]
    fn test_ccsds_c_2byte_pfield() {
        let c_bytes = &[0x90u8, 0x00, 0x64];
        let parsed = Parts::from_ccsds_cuc(c_bytes).unwrap();

        assert_eq!(parsed.yr, Some(1958));
        assert_eq!(parsed.mo, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hr, 0);
        assert_eq!(parsed.min, 1);
        assert_eq!(parsed.sec, 40);
    }

    #[test]
    fn test_ccsds_d_direct() {
        let d_bytes = &[0x40u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let parsed = Parts::from_ccsds_cds(d_bytes).unwrap();

        assert_eq!(parsed.yr, Some(1958));
        assert_eq!(parsed.mo, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hr, 0);
        assert_eq!(parsed.min, 0);
        assert_eq!(parsed.sec, 0);
        assert_eq!(parsed.attos, 1_000_000_000_000_000);
        assert_eq!(parsed.scale, Scale::UTC);
    }

    #[test]
    fn test_ccsds_d_direct_frac() {
        let d_bytes = &[0x41u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00];
        let parsed = Parts::from_ccsds_cds(d_bytes).unwrap();

        assert_eq!(parsed.sec, 0);
        assert_eq!(parsed.attos, 1_500_000_000_000_000);
    }

    #[test]
    fn test_ccsds_c_roundtrip() {
        let dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 123_456_789_000_000_000);

        let (buf, len) = dt.to_ccsds_cuc(4, 3, false).unwrap();
        let parsed = Parts::from_ccsds_cuc(&buf[0..len]).unwrap();

        assert_eq!(parsed.yr, Some(2025));
        assert_eq!(parsed.mo, Some(4));
        assert_eq!(parsed.day, Some(17));
        assert_eq!(parsed.hr, 14);
        assert_eq!(parsed.min, 30);
        assert_eq!(parsed.sec, 45);
        assert_eq!(parsed.scale, Scale::TAI);

        let diff = (parsed.attos as i64 - 123_456_789_000_000_000i64).abs();
        assert!(
            diff < 60_000_000_000,
            "Fractional error too large: {} attos",
            diff
        );
    }

    #[test]
    fn test_ccsds_d_roundtrip() {
        let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 400_000_000_000);

        let (buf, len) = dt.to_ccsds_cds(2, 1, false).unwrap();
        let parsed = Parts::from_ccsds_cds(&buf[0..len]).unwrap();

        assert_eq!(parsed.yr, Some(2025));
        assert_eq!(parsed.mo, Some(4));
        assert_eq!(parsed.day, Some(17));
        assert_eq!(parsed.hr, 14);
        assert_eq!(parsed.min, 30);
        assert_eq!(parsed.sec, 45);
        assert_eq!(parsed.scale, Scale::UTC);

        let diff = (parsed.attos as i64 - 400_000_000_000i64).abs();
        assert!(
            diff < 16_000_000_000,
            "Fractional error too large: {} attos",
            diff
        );
    }

    /// Helper that performs a full round-trip and verifies both the binary bytes
    /// and the recovered Parts are correct.
    fn roundtrip_ccs(tp: Dt, use_doy: bool, n_subsec: u8, expected_pfield: u8) {
        // to_ccsds_ccs
        let (buf, len) = tp.to_ccsds_ccs(use_doy, n_subsec).unwrap();
        let bytes = &buf[0..len];

        // Check P-field byte is exactly as expected
        assert_eq!(bytes[0], expected_pfield, "P-field mismatch");

        // from_ccsds_ccs (and auto-detector)
        let parsed_parts = Parts::from_ccsds_ccs(bytes).unwrap();
        let parsed_via_bin = Parts::from_ccsds_bin(bytes).unwrap();
        assert_eq!(parsed_parts, parsed_via_bin, "auto-detector failed");

        let recovered_tp = parsed_parts.to_dt().unwrap();

        // New single-field extraction (exactly matches old "sec + always-positive attos" rule)
        let aps = ATTOS_PER_SEC_I128;
        let tp_sec = tp.attos.div_euclid(aps);
        let tp_frac = (tp.attos.rem_euclid(aps)) as u64; // always ≥ 0

        let recovered_sec = recovered_tp.attos.div_euclid(aps);
        let recovered_frac = (recovered_tp.attos.rem_euclid(aps)) as u64;

        assert_eq!(
            tp_sec, recovered_sec,
            "Whole seconds mismatch after roundtrip"
        );

        // Special case for n_subsec == 0: fractional seconds are intentionally dropped
        if n_subsec == 0 {
            assert_eq!(
                recovered_frac, 0,
                "When n_subsec=0 the fractional part must be exactly zero"
            );
        } else {
            // Allowed quantization error = half the smallest representable unit at this precision
            let unit = 1_000_000_000_000_000_000u64 / 10u64.pow((2 * n_subsec) as u32);
            let max_error = unit / 2;
            let diff = (tp_frac as i64 - recovered_frac as i64).abs() as u64;
            assert!(
                diff <= max_error,
                "Fractional round-trip error too large for n_subsec={}: {} attos (max allowed {})",
                n_subsec,
                diff,
                max_error
            );
        }

        // Verify other fields
        assert_eq!(parsed_parts.scale, Scale::UTC);
        assert_eq!(parsed_parts.offset, None);
    }

    #[test]
    fn test_ccsds_ccs_month_day_variant() {
        // 2025-04-17 14:30:45.123456789 UTC (Month/Day)
        let tp = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);

        roundtrip_ccs(tp, false, 4, 0b0101_0100); // P-field: 01010100 (Code 101, MD, 4 subsec)
    }

    #[test]
    fn test_ccsds_ccs_day_of_year_variant() {
        // 2025-107 (April 17 is DOY 107 in 2025) 14:30:45.123456789 UTC
        let tp = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);

        roundtrip_ccs(tp, true, 3, 0b0101_1011); // P-field: 01011011 (Code 101, DOY, 3 subsec)
    }

    #[test]
    fn test_ccsds_ccs_leap_second() {
        // 2025-06-30 23:59:60.000000000 UTC (leap second)
        let tp = Dt::from_ymd(2025, 6, 30, Scale::UTC, 23, 59, 60, 0);

        roundtrip_ccs(tp, false, 0, 0b0101_0000); // P-field with 0 subsec
    }

    #[test]
    fn test_ccsds_ccs_various_precisions() {
        let base = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_012_345_678);

        for n in 0..=6 {
            roundtrip_ccs(base, false, n, 0b0101_0000 | n); // P-field varies only in low 3 bits
        }
    }

    #[test]
    fn test_ccsds_ccs_edge_cases() {
        // Epoch day
        let epoch = Dt::from_ymd(1958, 1, 1, Scale::UTC, 0, 0, 0, 0);
        roundtrip_ccs(epoch, false, 0, 0b0101_0000);

        // Year 9999, DOY 366 (leap year)
        let y9999 = Dt::from_ymd(9999, 12, 31, Scale::UTC, 23, 59, 59, 0);
        roundtrip_ccs(y9999, true, 2, 0b0101_1010);

        // Subsecond rounding test (exactly halfway case)
        let half = Dt::from_ymd(2025, 4, 17, Scale::UTC, 0, 0, 0, 500_000_000_000_000_000);
        let (buf, _) = half.to_ccsds_ccs(false, 1).unwrap();
        // Should round to 50 (i.e. 0.5 s)
        assert_eq!(buf[8], 0x50); // last BCD byte should be 0x50 for "50"
    }

    #[test]
    fn test_ccsds_ccs_invalid_pfield_rejected() {
        // Extension bit set
        let bad = &[0b1101_0000u8]; // bit 7 = 1
        assert!(Parts::from_ccsds_ccs(bad).is_err());

        // Wrong Code ID
        let bad = &[0b0111_0000u8]; // Code ID 111
        assert!(Parts::from_ccsds_ccs(bad).is_err());
    }

    #[test]
    fn compare_cds_with_spacepackets_py() {
        use deep_time::Scale;

        // Reference values from spacepackets-py
        let cases = [
            // (description, year, month, day, hour, min, sec, attos, expected_bytes)
            (
                "CDS Epoch (1958-01-01 00:00:00 UTC)",
                1958,
                1,
                1,
                0,
                0,
                0,
                0,
                &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..],
            ),
            (
                "Y2K (2000-01-01 00:00:00 UTC)",
                2000,
                1,
                1,
                0,
                0,
                0,
                0,
                &[0x40, 0x3B, 0xEC, 0x00, 0x00, 0x00, 0x00][..],
            ),
            (
                "Y2K Noon (2000-01-01 12:00:00 UTC)",
                2000,
                1,
                1,
                12,
                0,
                0,
                0,
                &[0x40, 0x3B, 0xEC, 0x02, 0x93, 0x2E, 0x00][..],
            ),
        ];

        for (desc, y, m, d, h, min, s, attos, expected) in cases {
            let dt = Dt::from_ymd(y, m, d, Scale::UTC, h, min, s, attos);

            let (buf, len) = dt.to_ccsds_cds(2, 0, false).unwrap(); // 2-byte day, no sub-ms

            assert_eq!(&buf[..len], expected, "Mismatch for case: {}", desc);
        }
    }

    // ====================== Additional CCSDS Binary Tests ======================

    #[test]
    fn test_ccsds_c_extended_pfield_roundtrip() {
        // Use values that force the 2-byte extended P-field
        let dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 123_456_789_012_345_678);

        // n_coarse=5 and n_frac=4 both require the extended P-field
        let (buf, len) = dt.to_ccsds_cuc(5, 4, false).unwrap();
        assert_eq!(len, 5 + 4 + 2); // P1 + P2 + 5 coarse + 4 frac

        let parsed = Parts::from_ccsds_cuc(&buf[0..len]).unwrap();
        let recovered = parsed.to_dt().unwrap();

        // Compare civil time fields only (attos may have small quantization error)
        let orig = dt.to_ymd();
        let rec = recovered.to_ymd();

        assert_eq!(rec.yr(), orig.yr());
        assert_eq!(rec.mo(), orig.mo());
        assert_eq!(rec.day(), orig.day());
        assert_eq!(rec.hr(), orig.hr());
        assert_eq!(rec.min(), orig.min());
        assert_eq!(rec.sec(), orig.sec());

        // Allow small quantization error due to fractional scaling
        let diff = (recovered.attos as i128 - dt.attos).abs();
        assert!(diff < 1_000_000_000_000_000, "Fractional error too large");
    }

    #[test]
    fn test_ccsds_c_max_parameters() {
        // Use a clean fractional value (0.5s) that is more likely to survive n_frac=10
        let dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 500_000_000_000_000_000);

        let (buf, len) = dt.to_ccsds_cuc(7, 8, false).unwrap();
        assert_eq!(len, 7 + 8 + 2);

        let parsed = Parts::from_ccsds_cuc(&buf[0..len]).unwrap();
        let recovered = parsed.to_dt().unwrap();

        // Civil time should match perfectly
        let orig = dt.to_ymd();
        let rec = recovered.to_ymd();

        assert_eq!(rec.yr(), orig.yr());
        assert_eq!(rec.mo(), orig.mo());
        assert_eq!(rec.day(), orig.day());
        assert_eq!(rec.hr(), orig.hr());
        assert_eq!(rec.min(), orig.min());
        assert_eq!(rec.sec(), orig.sec());

        // With 0.5s fractional, this should now roundtrip exactly at n_frac=10
        assert_eq!(
            rec.attos(),
            orig.attos(),
            "Attos should roundtrip exactly with 0.5s"
        );
    }

    #[test]
    fn test_ccsds_d_sub_ms_code_2() {
        // Test the 2⁻³² sub-millisecond path
        let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_012_345_678);

        let (buf, len) = dt.to_ccsds_cds(2, 2, false).unwrap();
        assert_eq!(len, 11); // P-field (1) + 2 days + 4 ms + 4 sub-ms

        let parsed = Parts::from_ccsds_cds(&buf[0..len]).unwrap();
        let recovered = parsed.to_dt().unwrap();

        // Compare civil time fields only
        let orig = dt.to_ymd();
        let rec = recovered.to_ymd();

        assert_eq!(rec.yr(), orig.yr());
        assert_eq!(rec.mo(), orig.mo());
        assert_eq!(rec.day(), orig.day());
        assert_eq!(rec.hr(), orig.hr());
        assert_eq!(rec.min(), orig.min());
        assert_eq!(rec.sec(), orig.sec());

        // Allow reasonable quantization error for 2⁻³² precision
        let diff = (recovered.attos as i128 - dt.attos).abs();
        assert!(
            diff < 250_000_000_000_000,
            "Fractional error too large for sub_ms_code=2, got: {}",
            diff,
        );
    }

    #[test]
    fn test_ccsds_pre_1958_rejection() {
        // CUC
        let before = Dt::from_ymd(1957, 12, 31, Scale::TAI, 23, 59, 59, 0);
        assert!(
            matches!(
                before.to_ccsds_cuc(4, 0, false),
                Err(e) if e.kind() == DtErrKind::YearOutOfRange),
            "CUC should reject pre-1958 time"
        );

        // CDS
        let before_utc = Dt::from_ymd(1957, 12, 31, Scale::UTC, 23, 59, 59, 0);
        assert!(
            matches!(
                before_utc.to_ccsds_cds(2, 0, false),
                Err(e) if e.kind() == DtErrKind::YearOutOfRange),
            "CDS should reject pre-1958 time"
        );
    }

    #[test]
    fn test_ccsds_c_extended_pfield_variations() {
        let base_dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 123_456_789_012_345_678);

        // Case 1: Extension triggered by n_coarse > 4
        let (buf1, len1) = base_dt.to_ccsds_cuc(5, 2, false).unwrap();
        let parsed1 = Parts::from_ccsds_cuc(&buf1[0..len1]).unwrap();
        let recovered1 = parsed1.to_dt().unwrap();
        assert_eq!(recovered1.to_ymd().yr(), 2025);
        assert_eq!(recovered1.to_ymd().sec(), 45);

        // Case 2: Extension triggered by n_frac > 3 (use n_coarse=4 so the date fits)
        let (buf2, len2) = base_dt.to_ccsds_cuc(4, 5, false).unwrap();
        let parsed2 = Parts::from_ccsds_cuc(&buf2[0..len2]).unwrap();
        let recovered2 = parsed2.to_dt().unwrap();
        assert_eq!(recovered2.to_ymd().yr(), 2025);

        // Case 3: Explicit extension flag
        let (buf3, len3) = base_dt.to_ccsds_cuc(4, 3, true).unwrap();
        assert_eq!(buf3[0] & 0b1000_0000, 0b1000_0000); // extension bit set
        let parsed3 = Parts::from_ccsds_cuc(&buf3[0..len3]).unwrap();
        assert_eq!(parsed3.yr, Some(2025));
    }

    #[test]
    fn test_ccsds_d_combined_features() {
        let dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 123_456_789_012_345_678);

        // n_day=3 + sub_ms_code=2 + extension=true
        let (buf, len) = dt.to_ccsds_cds(3, 2, true).unwrap();

        // Expected length: 1 (P1) + 1 (P2) + 3 (days) + 4 (ms) + 4 (sub-ms) = 13
        assert_eq!(len, 13);
        assert_eq!(buf[0] & 0b1000_0000, 0b1000_0000); // extension bit
        assert_eq!(buf[0] & 0b0000_0100, 0b0000_0100); // 3-byte day count
        assert_eq!(buf[0] & 0b0000_0011, 0b0000_0010); // sub_ms_code=2

        let parsed = Parts::from_ccsds_cds(&buf[0..len]).unwrap();
        let recovered = parsed.to_dt().unwrap();

        // Verify civil time
        let orig = dt.to_ymd();
        let rec = recovered.to_ymd();
        assert_eq!(rec.yr(), orig.yr());
        assert_eq!(rec.mo(), orig.mo());
        assert_eq!(rec.day(), orig.day());
        assert_eq!(rec.hr(), orig.hr());
        assert_eq!(rec.min(), orig.min());
        assert_eq!(rec.sec(), orig.sec());
    }

    #[test]
    fn test_from_ccsds_bin_auto_detector() {
        // CUC (Code ID 001)
        let cuc_dt = Dt::from_ymd(2025, 4, 17, Scale::TAI, 14, 30, 45, 0);
        let (cuc_buf, cuc_len) = cuc_dt.to_ccsds_cuc(4, 0, false).unwrap();
        let cuc_parsed = Parts::from_ccsds_bin(&cuc_buf[0..cuc_len]).unwrap();
        assert_eq!(cuc_parsed.scale, Scale::TAI);
        assert_eq!(cuc_parsed.yr, Some(2025));

        // CDS (Code ID 100)
        let cds_dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 0);
        let (cds_buf, cds_len) = cds_dt.to_ccsds_cds(2, 0, false).unwrap();
        let cds_parsed = Parts::from_ccsds_bin(&cds_buf[0..cds_len]).unwrap();
        assert_eq!(cds_parsed.scale, Scale::UTC);
        assert_eq!(cds_parsed.yr, Some(2025));

        // CCS (Code ID 101)
        let ccs_dt = Dt::from_ymd(2025, 4, 17, Scale::UTC, 14, 30, 45, 0);
        let (ccs_buf, ccs_len) = ccs_dt.to_ccsds_ccs(false, 0).unwrap();
        let ccs_parsed = Parts::from_ccsds_bin(&ccs_buf[0..ccs_len]).unwrap();
        assert_eq!(ccs_parsed.scale, Scale::UTC);
        assert_eq!(ccs_parsed.yr, Some(2025));
    }
}
