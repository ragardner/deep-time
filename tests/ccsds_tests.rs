#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::constants::{ATTOS_PER_SEC_I128, SEC_PER_DAYI64};
use deep_time::{Dt, DtErrKind, Offset, Scale, TimeParts};

mod ccsds_tests {
    use super::*;

    const CUC_EPOCH_OFFSET: i64 = 1_325_419_167;
    const CDS_EPOCH_OFFSET: i64 = 1_325_419_135;

    // ====================== Helpers ======================

    fn tai_epoch() -> Dt {
        Dt::from_attos(-(CUC_EPOCH_OFFSET as i128) * ATTOS_PER_SEC_I128, Scale::TAI)
    }

    fn j2000() -> Dt {
        Dt::from_attos(0, Scale::TAI)
    }

    fn utc_epoch() -> Dt {
        Dt::from_attos(-(CDS_EPOCH_OFFSET as i128) * ATTOS_PER_SEC_I128, Scale::UTC)
    }

    // ====================== CUC ======================

    #[test]
    fn cuc_epoch() {
        let dt = tai_epoch();
        let (buf, len) = dt.to_ccsds_c(4, 0, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x1C, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn cuc_one_second_after() {
        let mut dt = tai_epoch().add_sec(1);
        let (buf, len) = dt.to_ccsds_c(4, 0, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x1C, 0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn cuc_fractional() {
        let dt = tai_epoch().add_attos(500_000_000_000_000_000);
        let (buf, len) = dt.to_ccsds_c(1, 3, false).unwrap();
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], &[0x13, 0x00, 0x80, 0x00, 0x00]);
    }

    #[test]
    fn cuc_extension() {
        let dt = j2000();
        let (buf, len) = dt.to_ccsds_c(5, 0, false).unwrap();
        assert_eq!(len, 7);
        assert_eq!(buf[0], 0x9C);
        assert_eq!(buf[1], 0x20);
    }

    // ====================== CDS ======================

    #[test]
    fn cds_epoch() {
        let dt = utc_epoch();
        let (buf, len) = dt.to_ccsds_d(2, 0, false).unwrap();
        assert_eq!(len, 7);
        assert_eq!(&buf[..len], &[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn cds_n_day3_extension() {
        let dt = utc_epoch();
        let (buf, len) = dt.to_ccsds_d(3, 0, true).unwrap();
        assert_eq!(len, 9);
        assert_eq!(buf[0], 0xC4);
        assert_eq!(buf[1], 0x00);
    }

    #[test]
    fn cds_submillisecond() {
        let dt = utc_epoch().add_attos(123_456_789_012_345_678);
        let (buf, len) = dt.to_ccsds_d(2, 1, false).unwrap();
        assert_eq!(len, 9);
        assert_eq!(buf[0], 0x41);
    }

    // ====================== CCS ======================

    #[test]
    fn ccs_y2k_month_day() {
        let dt = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
        let (buf, len) = dt.to_ccsds_ccs(false, 0).unwrap();
        assert_eq!(len, 8);
        let expected = [0x50, 0x20, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00];
        assert_eq!(&buf[..len], &expected[..]);
    }

    #[test]
    fn ccs_doy() {
        let dt = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
        let (buf, len) = dt.to_ccsds_ccs(true, 0).unwrap();
        assert_eq!(len, 8);
        assert_eq!(buf[0], 0x58);
        assert_eq!(buf[3], 0x00);
        assert_eq!(buf[4], 0x01);
    }

    #[test]
    fn ccs_subsecond() {
        let dt =
            Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC).add_attos(123_456_789_012_345_678);
        let (buf, len) = dt.to_ccsds_ccs(false, 2).unwrap();
        assert_eq!(len, 10);
        assert_eq!(buf[0], 0x52);
    }

    // ====================== Error Cases ======================

    #[test]
    fn invalid_parameters() {
        let dt = j2000();

        assert!(matches!(
            dt.to_ccsds_c(0, 0, false),
            Err(e) if e.kind() == Some(DtErrKind::OutOfRange)
        ));

        assert!(matches!(
            dt.to_ccsds_c( 4, 11, false),
            Err(e) if e.kind() == Some(DtErrKind::OutOfRange)
        ));

        assert!(matches!(
            dt.to_ccsds_d( 1, 0, false),
            Err(e) if e.kind() == Some(DtErrKind::InvalidNumber)
        ));

        assert!(matches!(
            dt.to_ccsds_d( 2, 3, false),
            Err(e) if e.kind() == Some(DtErrKind::InvalidItem)
        ));

        assert!(matches!(
            dt.to_ccsds_ccs( false, 7),
            Err(e) if e.kind() == Some(DtErrKind::OutOfRange)
        ));
    }

    // ====================== Convenience ======================

    #[test]
    fn to_ccsds_bin() {
        let tai = j2000();
        let (buf, _) = tai.to_ccsds_bin().unwrap();
        assert_eq!(buf[0] & 0b0111_0000, 0b0001_0000);

        let utc = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
        let (buf, _) = utc.to_ccsds_bin().unwrap();
        assert_eq!(buf[0] & 0b0111_0000, 0b0100_0000);
    }
}

// ====================== Parsing roundtrip tests ======================

#[test]
fn test_ccsds_c_direct_frac() {
    let c_bytes = &[0x15u8, 0x00, 0x01, 0x80];
    let parsed = TimeParts::from_ccsds_c(c_bytes).unwrap();

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
    let parsed = TimeParts::from_ccsds_c(c_bytes).unwrap();

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
    let parsed = TimeParts::from_ccsds_d(d_bytes).unwrap();

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
    let parsed = TimeParts::from_ccsds_d(d_bytes).unwrap();

    assert_eq!(parsed.sec, 0);
    assert_eq!(parsed.attos, 1_500_000_000_000_000);
}

#[test]
fn test_ccsds_c_roundtrip() {
    let days_since_1958 = TimeParts::gregorian_to_days_since_1958(2025, 4, 17);
    let sec_of_day = (14 * 3600) + (30 * 60) + 45;
    let total_tai_seconds = days_since_1958 * SEC_PER_DAYI64 + sec_of_day;

    const EPOCH_OFFSET: i64 = 1_325_419_167;
    let tai_sec = total_tai_seconds - EPOCH_OFFSET;

    let t = Dt::from_attos(
        (tai_sec as i128) * ATTOS_PER_SEC_I128 + 123_456_789_000_000_000,
        Scale::TAI,
    );

    let (buf, len) = t.to_ccsds_c(4, 3, false).unwrap();
    let parsed = TimeParts::from_ccsds_c(&buf[0..len]).unwrap();

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
    let days_since_1958 = TimeParts::gregorian_to_days_since_1958(2025, 4, 17);
    let sec_of_day = (14 * 3600) + (30 * 60) + 45;
    let total_utc_seconds = days_since_1958 * SEC_PER_DAYI64 + sec_of_day;

    const EPOCH_OFFSET: i64 = 1_325_419_135;
    let utc_sec = total_utc_seconds - EPOCH_OFFSET;

    let t = Dt::from_attos(
        (utc_sec as i128) * ATTOS_PER_SEC_I128 + 400_000_000_000,
        Scale::UTC,
    );

    let (buf, len) = t.to_ccsds_d(2, 1, false).unwrap();
    let parsed = TimeParts::from_ccsds_d(&buf[0..len]).unwrap();

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
/// and the recovered TimeParts are correct.
fn roundtrip_ccs(tp: Dt, use_doy: bool, n_subsec: u8, expected_pfield: u8) {
    // to_ccsds_ccs
    let (buf, len) = tp.to_ccsds_ccs(use_doy, n_subsec).unwrap();
    let bytes = &buf[0..len];

    // Check P-field byte is exactly as expected
    assert_eq!(bytes[0], expected_pfield, "P-field mismatch");

    // from_ccsds_ccs (and auto-detector)
    let parsed_parts = TimeParts::from_ccsds_ccs(bytes).unwrap();
    let parsed_via_bin = TimeParts::from_ccsds_bin(bytes).unwrap();
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
    assert_eq!(parsed_parts.offset, Some(Offset::Utc));
}

#[test]
fn test_ccsds_ccs_month_day_variant() {
    // 2025-04-17 14:30:45.123456789 UTC (Month/Day)
    let tp = Dt::from_ymd(2025, 4, 17, 14, 30, 45, 123_456_789_000_000_000, Scale::UTC);

    roundtrip_ccs(tp, false, 4, 0b0101_0100); // P-field: 01010100 (Code 101, MD, 4 subsec)
}

#[test]
fn test_ccsds_ccs_day_of_year_variant() {
    // 2025-107 (April 17 is DOY 107 in 2025) 14:30:45.123456789 UTC
    let tp = Dt::from_ymd(2025, 4, 17, 14, 30, 45, 123_456_789_000_000_000, Scale::UTC);

    roundtrip_ccs(tp, true, 3, 0b0101_1011); // P-field: 01011011 (Code 101, DOY, 3 subsec)
}

#[test]
fn test_ccsds_ccs_leap_second() {
    // 2025-06-30 23:59:60.000000000 UTC (leap second)
    let tp = Dt::from_ymd(2025, 6, 30, 23, 59, 60, 0, Scale::UTC);

    roundtrip_ccs(tp, false, 0, 0b0101_0000); // P-field with 0 subsec
}

#[test]
fn test_ccsds_ccs_various_precisions() {
    let base = Dt::from_ymd(2025, 4, 17, 14, 30, 45, 123_456_789_012_345_678, Scale::UTC);

    for n in 0..=6 {
        roundtrip_ccs(base, false, n, 0b0101_0000 | n); // P-field varies only in low 3 bits
    }
}

#[test]
fn test_ccsds_ccs_edge_cases() {
    // Epoch day
    let epoch = Dt::from_ymd(1958, 1, 1, 0, 0, 0, 0, Scale::UTC);
    roundtrip_ccs(epoch, false, 0, 0b0101_0000);

    // Year 9999, DOY 366 (leap year)
    let y9999 = Dt::from_ymd(9999, 12, 31, 23, 59, 59, 0, Scale::UTC);
    roundtrip_ccs(y9999, true, 2, 0b0101_1010);

    // Subsecond rounding test (exactly halfway case)
    let half = Dt::from_ymd(2025, 4, 17, 0, 0, 0, 500_000_000_000_000_000, Scale::UTC);
    let (buf, _) = half.to_ccsds_ccs(false, 1).unwrap();
    // Should round to 50 (i.e. 0.5 s)
    assert_eq!(buf[8], 0x50); // last BCD byte should be 0x50 for "50"
}

#[test]
fn test_ccsds_ccs_invalid_pfield_rejected() {
    // Extension bit set
    let bad = &[0b1101_0000u8]; // bit 7 = 1
    assert!(TimeParts::from_ccsds_ccs(bad).is_err());

    // Wrong Code ID
    let bad = &[0b0111_0000u8]; // Code ID 111
    assert!(TimeParts::from_ccsds_ccs(bad).is_err());
}

#[test]
fn test_ccsds_calendar_variants() {
    // Full calendar with fractional seconds + trailing Z
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.123456789Z").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.day_of_yr, None);
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);

    // Calendar with seconds, no fraction
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);

    // Calendar with only minutes
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 0);

    // Calendar with only hour
    let dt = TimeParts::from_str_ccsds("2024-04-18T14").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 0);
    assert_eq!(dt.sec, 0);

    // Calendar date-only
    let dt = TimeParts::from_str_ccsds("2024-04-18").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.day_of_yr, None);
    assert_eq!(dt.hr, 0);
    assert_eq!(dt.min, 0);
    assert_eq!(dt.sec, 0);
}

#[test]
fn test_ccsds_doy_variants() {
    // DOY with fractional seconds + Z
    let dt = TimeParts::from_str_ccsds("2024-109T14:30:25.5Z").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.day_of_yr, Some(109));
    assert_eq!(dt.mo, None);
    assert_eq!(dt.day, None);
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);

    // DOY date-only
    let dt = TimeParts::from_str_ccsds("2024-001").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.day_of_yr, Some(1));
    assert_eq!(dt.mo, None);
    assert_eq!(dt.day, None);

    // DOY with seconds only (no fraction)
    let dt = TimeParts::from_str_ccsds("2024-366T23:59:59").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.day_of_yr, Some(366));
    assert_eq!(dt.hr, 23);
    assert_eq!(dt.min, 59);
    assert_eq!(dt.sec, 59);
}

#[test]
fn test_ccsds_separators_and_z() {
    // Space instead of T
    let dt = TimeParts::from_str_ccsds("2024-04-18 14:30:25").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);

    // Lowercase t
    let dt = TimeParts::from_str_ccsds("2024-109t14:30").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.day_of_yr, Some(109));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);

    // Trailing Z (case-insensitive) is stripped and still works
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25Z").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);
}

#[test]
fn test_ccsds_fractional_seconds_various_lengths() {
    // 1 digit
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.1").unwrap();
    assert_eq!(dt.attos, 100_000_000_000_000_000);

    // 3 digits
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.123").unwrap();
    assert_eq!(dt.attos, 123_000_000_000_000_000);

    // 6 digits
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.123456").unwrap();
    assert_eq!(dt.attos, 123_456_000_000_000_000);

    // 9 digits (full nanos)
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.123456789").unwrap();
    assert_eq!(dt.attos, 123_456_789_000_000_000);
}

#[test]
fn test_ccsds_leap_second() {
    let dt = TimeParts::from_str_ccsds("2024-06-30T23:59:60Z").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(6));
    assert_eq!(dt.day, Some(30));
    assert_eq!(dt.sec, 60);
}

#[test]
fn test_ccsds_doy_vs_calendar_detection() {
    // Must be detected as DOY (exactly 3 digits after year separator, next char is not a digit)
    let doy = TimeParts::from_str_ccsds("2024-123T12:00:00").unwrap();
    assert_eq!(doy.day_of_yr, Some(123));
    assert_eq!(doy.mo, None);
    assert_eq!(doy.day, None);

    // Must be detected as calendar date
    let cal = TimeParts::from_str_ccsds("2024-12-03T12:00:00").unwrap();
    assert_eq!(cal.mo, Some(12));
    assert_eq!(cal.day, Some(3));
    assert_eq!(cal.day_of_yr, None);
}

#[test]
fn test_from_str_junk() {
    // skip junk before and after the timestamp
    let x = TimeParts::from_str_ccsds("sdfsdfs sdfsdf 2024-123T12:00:00dsfsdf").unwrap();
    assert_eq!(x.yr, Some(2024));
    assert_eq!(x.day_of_yr, Some(123));
    assert_eq!(x.mo, None);
    assert_eq!(x.day, None);
    assert_eq!(x.hr, 12);
    assert_eq!(x.min, 0);
    assert_eq!(x.sec, 0);
    assert_eq!(x.attos, 0); // defaults to 0 attos
    assert_eq!(x.scale, Scale::UTC); // default scale when none given

    // parse scale at the end (late scale)
    let x = TimeParts::from_str_ccsds("sdfsdfs sdfsdf 2024-123T12:00:00 TDB").unwrap();
    assert_eq!(x.yr, Some(2024));
    assert_eq!(x.day_of_yr, Some(123));
    assert_eq!(x.hr, 12);
    assert_eq!(x.min, 0);
    assert_eq!(x.sec, 0);
    assert_eq!(x.scale, Scale::TDB);

    // parse scale at the end with trailing Z
    let x = TimeParts::from_str_ccsds("sdfsdfs sdfsdf 2024-123T12:00:00Z TDB").unwrap();
    assert_eq!(x.yr, Some(2024));
    assert_eq!(x.day_of_yr, Some(123));
    assert_eq!(x.hr, 12);
    assert_eq!(x.min, 0);
    assert_eq!(x.sec, 0);
    assert_eq!(x.scale, Scale::TDB);

    // parse early scale (right after DOY, no time)
    let x = TimeParts::from_str_ccsds("sdfsdfs sdfsdf 2024-123TDB fdsfsdfsdf").unwrap();
    assert_eq!(x.yr, Some(2024));
    assert_eq!(x.day_of_yr, Some(123));
    assert_eq!(x.hr, 0); // time is optional
    assert_eq!(x.min, 0);
    assert_eq!(x.sec, 0);
    assert_eq!(x.scale, Scale::TDB);
}

#[test]
fn test_ccsds_early_and_late_scale() {
    // === EARLY scale (right after date) ===
    let dt = TimeParts::from_str_ccsds("2024-001TAI").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.day_of_yr, Some(1));
    assert_eq!(dt.scale, Scale::TAI);

    let dt = TimeParts::from_str_ccsds("2024-04-18 TDB").unwrap();
    assert_eq!(dt.yr, Some(2024));
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.scale, Scale::TDB);

    // === EARLY scale + time ===
    let dt = TimeParts::from_str_ccsds("2024-109T12:00:00LTC").unwrap();
    assert_eq!(dt.day_of_yr, Some(109));
    assert_eq!(dt.hr, 12);
    assert_eq!(dt.min, 0);
    assert_eq!(dt.sec, 0);
    assert_eq!(dt.scale, Scale::LTC);

    let dt = TimeParts::from_str_ccsds("2024-04-18 14:30:25 UTC").unwrap();
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.hr, 14);
    assert_eq!(dt.min, 30);
    assert_eq!(dt.sec, 25);
    assert_eq!(dt.scale, Scale::UTC);

    // === LATE scale (after time) ===
    let dt = TimeParts::from_str_ccsds("2024-001T12:00:00 TDB").unwrap();
    assert_eq!(dt.day_of_yr, Some(1));
    assert_eq!(dt.hr, 12);
    assert_eq!(dt.scale, Scale::TDB);

    let dt = TimeParts::from_str_ccsds("2024-06-30T23:59:60Z GPS").unwrap();
    assert_eq!(dt.sec, 60);
    assert_eq!(dt.scale, Scale::GPS);

    // === BOTH orders with fractional seconds ===
    let dt = TimeParts::from_str_ccsds("2024-04-18T14:30:25.123456789 TCL").unwrap();
    assert_eq!(dt.scale, Scale::TCL);
    assert_eq!(dt.attos, 123_456_789_000_000_000);

    let dt = TimeParts::from_str_ccsds("2024-109 14:30:25.5 TAI").unwrap();
    assert_eq!(dt.scale, Scale::TAI);
    assert_eq!(dt.attos, 500_000_000_000_000_000);

    // === Time completely optional, scale only ===
    let dt = TimeParts::from_str_ccsds("2024-001 TAI").unwrap();
    assert_eq!(dt.day_of_yr, Some(1));
    assert_eq!(dt.scale, Scale::TAI);
    assert_eq!(dt.hr, 0); // defaults
    assert_eq!(dt.min, 0);
    assert_eq!(dt.sec, 0);

    let dt = TimeParts::from_str_ccsds("2024-04-18UTC").unwrap();
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.scale, Scale::UTC);
}

#[test]
fn test_ccsds_scale_case_insensitivity_and_variants() {
    for scale_str in &[
        "TAI", "tai", "Tai", "UTC", "utc", "TDB", "ltc", "TCL", "GPS", "gst",
    ] {
        let s = format!("2024-001T12:00:00 {}", scale_str);
        let dt = TimeParts::from_str_ccsds(&s).unwrap();
        assert!(dt.scale != Scale::Custom, "failed to parse {}", scale_str);
    }
}

#[test]
fn test_ccsds_no_time_no_scale_still_works() {
    let dt = TimeParts::from_str_ccsds("2024-001").unwrap();
    assert_eq!(dt.day_of_yr, Some(1));
    assert_eq!(dt.scale, Scale::UTC); // default is UTC

    let dt = TimeParts::from_str_ccsds("2024-04-18").unwrap();
    assert_eq!(dt.mo, Some(4));
    assert_eq!(dt.day, Some(18));
    assert_eq!(dt.scale, Scale::UTC); // default is UTC
}
