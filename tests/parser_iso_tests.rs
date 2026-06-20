#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::constants::{ATTOS_PER_SEC_I128, SEC_PER_DAYI64};
use deep_time::civil_parts::{Offset, Parts};
use deep_time::{Dt, DtErrKind, Scale};

mod from_str_iso_tests {
    use super::*;

    #[test]
    fn test_iso_offset_directly_after_date() {
        // Offset with no time component
        let tp = Parts::from_str_iso("2023-01-01+05:00").unwrap();
        assert_eq!(tp.yr, Some(2023));
        assert_eq!(tp.mo, Some(1));
        assert_eq!(tp.day, Some(1));
        assert_eq!(tp.hr, 0);
        assert_eq!(tp.min, 0);
        assert_eq!(tp.sec, 0);
        assert_eq!(tp.offset, Some(Offset::Fixed(5 * 3600)));

        // Negative offset, compact form
        let tp = Parts::from_str_iso("2023-001-0530").unwrap();
        assert_eq!(tp.day_of_yr, Some(1));
        assert_eq!(tp.offset, Some(Offset::Fixed(-5 * 3600 - 30 * 60)));
    }

    #[test]
    fn test_iso_offset_after_time() {
        let tp = Parts::from_str_iso("2024-04-18T14:30:25+02:00").unwrap();
        assert_eq!(tp.yr, Some(2024));
        assert_eq!(tp.mo, Some(4));
        assert_eq!(tp.day, Some(18));
        assert_eq!(tp.hr, 14);
        assert_eq!(tp.min, 30);
        assert_eq!(tp.sec, 25);
        assert_eq!(tp.offset, Some(Offset::Fixed(2 * 3600)));

        // With Z and offset
        let tp = Parts::from_str_iso("2024-04-18T14:30:25Z-05:30").unwrap();
        assert_eq!(tp.offset, Some(Offset::Fixed(-5 * 3600 - 30 * 60)));
    }

    #[test]
    fn test_iso_compact_offset() {
        let tp = Parts::from_str_iso("2023-12-25T00:00:00+0530").unwrap();
        assert_eq!(tp.offset, Some(Offset::Fixed(5 * 3600 + 30 * 60)));

        let tp = Parts::from_str_iso("2023-12-25+0000").unwrap();
        assert_eq!(tp.offset, Some(Offset::Fixed(0)));
    }

    #[test]
    fn test_iso_iana_name() {
        let tp = Parts::from_str_iso("2024-04-18T14:30:25 [America/New_York]").unwrap();
        assert_eq!(tp.yr, Some(2024));
        assert_eq!(tp.mo, Some(4));
        assert_eq!(tp.day, Some(18));
        assert_eq!(tp.hr, 14);
        assert_eq!(tp.min, 30);
        assert_eq!(tp.sec, 25);
        assert_eq!(tp.offset, None); // no offset in this case
        // iana_name is set via set_iana_name (LiteStr), so we just check it's Some
        assert!(tp.iana_name.is_some());
    }

    #[test]
    fn test_iso_offset_and_iana() {
        let tp = Parts::from_str_iso("2024-04-18T14:30:25+02:00 [Europe/Paris]").unwrap();
        assert_eq!(tp.offset, Some(Offset::Fixed(2 * 3600)));
        assert!(tp.iana_name.is_some());
    }

    #[test]
    fn test_iso_full_example_from_docs() {
        // Matches the example in the doc comment
        let tp =
            Parts::from_str_iso("+2000-01-01T17:00:00 -0500 [America/New_York] TAI").unwrap();

        assert_eq!(tp.yr, Some(2000));
        assert_eq!(tp.mo, Some(1));
        assert_eq!(tp.day, Some(1));
        assert_eq!(tp.hr, 17);
        assert_eq!(tp.min, 0);
        assert_eq!(tp.sec, 0);
        assert_eq!(tp.offset, Some(Offset::Fixed(-5 * 3600)));
        assert!(tp.iana_name.is_some());
        assert_eq!(tp.scale, Scale::TAI);
    }

    #[test]
    fn test_iso_whitespace_variations() {
        let tp = Parts::from_str_iso("2024-04-18  14:30:25   +02:00   [Europe/Berlin]   TAI")
            .unwrap();
        assert_eq!(tp.hr, 14);
        assert_eq!(tp.offset, Some(Offset::Fixed(2 * 3600)));
        assert!(tp.iana_name.is_some());
        assert_eq!(tp.scale, Scale::TAI);
    }

    #[test]
    fn test_iso_iana_unclosed_bracket_error() {
        let result = Parts::from_str_iso("2024-04-18T12:00:00 [America/New_York");
        assert!(result.is_err());
        // You can also assert the exact error kind if desired:
        // assert!(matches!(result, Err(e) if e.kind() == Some(DtErrKind::InvalidSyntax)));
    }

    #[test]
    fn test_iso_scale_after_iana() {
        let tp = Parts::from_str_iso("2024-04-18T12:00:00 [America/New_York] GPS").unwrap();
        assert!(tp.iana_name.is_some());
        assert_eq!(tp.scale, Scale::GPS);
    }

    // Optional: test that bare +/- after date is handled gracefully (current lenient behavior)
    #[test]
    fn test_iso_bare_sign_after_date() {
        // Current behavior: consumes the sign but doesn't set offset
        let tp = Parts::from_str_iso("2023-01-01+").unwrap();
        assert_eq!(tp.yr, Some(2023));
        assert_eq!(tp.mo, Some(1));
        assert_eq!(tp.day, Some(1));
        // offset remains default (not set)
    }

    #[test]
    fn test_ccsds_calendar_variants() {
        // Full calendar with fractional seconds + trailing Z
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.123456789Z").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.day_of_yr, None);
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 25);

        // Calendar with seconds, no fraction
        let dt = Parts::from_str_iso("2024-04-18T14:30:25").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 25);

        // Calendar with only minutes
        let dt = Parts::from_str_iso("2024-04-18T14:30").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 0);

        // Calendar with only hour
        let dt = Parts::from_str_iso("2024-04-18T14").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 0);
        assert_eq!(dt.sec, 0);

        // Calendar date-only
        let dt = Parts::from_str_iso("2024-04-18").unwrap();
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
        let dt = Parts::from_str_iso("2024-109T14:30:25.5Z").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.day_of_yr, Some(109));
        assert_eq!(dt.mo, None);
        assert_eq!(dt.day, None);
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 25);

        // DOY date-only
        let dt = Parts::from_str_iso("2024-001").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.day_of_yr, Some(1));
        assert_eq!(dt.mo, None);
        assert_eq!(dt.day, None);

        // DOY with seconds only (no fraction)
        let dt = Parts::from_str_iso("2024-366T23:59:59").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.day_of_yr, Some(366));
        assert_eq!(dt.hr, 23);
        assert_eq!(dt.min, 59);
        assert_eq!(dt.sec, 59);
    }

    #[test]
    fn test_ccsds_separators_and_z() {
        // Space instead of T
        let dt = Parts::from_str_iso("2024-04-18 14:30:25").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 25);

        // Lowercase t
        let dt = Parts::from_str_iso("2024-109t14:30").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.day_of_yr, Some(109));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);

        // Trailing Z (case-insensitive) is stripped and still works
        let dt = Parts::from_str_iso("2024-04-18T14:30:25Z").unwrap();
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
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.1").unwrap();
        assert_eq!(dt.attos, 100_000_000_000_000_000);

        // 3 digits
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.123").unwrap();
        assert_eq!(dt.attos, 123_000_000_000_000_000);

        // 6 digits
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.123456").unwrap();
        assert_eq!(dt.attos, 123_456_000_000_000_000);

        // 9 digits (full nanos)
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.123456789").unwrap();
        assert_eq!(dt.attos, 123_456_789_000_000_000);
    }

    #[test]
    fn test_ccsds_leap_second() {
        let dt = Parts::from_str_iso("2024-06-30T23:59:60Z").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(6));
        assert_eq!(dt.day, Some(30));
        assert_eq!(dt.sec, 60);
    }

    #[test]
    fn test_ccsds_doy_vs_calendar_detection() {
        // Must be detected as DOY (exactly 3 digits after year separator, next char is not a digit)
        let doy = Parts::from_str_iso("2024-123T12:00:00").unwrap();
        assert_eq!(doy.day_of_yr, Some(123));
        assert_eq!(doy.mo, None);
        assert_eq!(doy.day, None);

        // Must be detected as calendar date
        let cal = Parts::from_str_iso("2024-12-03T12:00:00").unwrap();
        assert_eq!(cal.mo, Some(12));
        assert_eq!(cal.day, Some(3));
        assert_eq!(cal.day_of_yr, None);
    }

    #[test]
    fn test_from_str_junk() {
        // skip junk before and after the timestamp
        let x = Parts::from_str_iso("sdfsdfs sdfsdf 2024-123T12:00:00dsfsdf").unwrap();
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
        let x = Parts::from_str_iso("sdfsdfs sdfsdf 2024-123T12:00:00 TDB").unwrap();
        assert_eq!(x.yr, Some(2024));
        assert_eq!(x.day_of_yr, Some(123));
        assert_eq!(x.hr, 12);
        assert_eq!(x.min, 0);
        assert_eq!(x.sec, 0);
        assert_eq!(x.scale, Scale::TDB);

        // parse scale at the end with trailing Z
        let x = Parts::from_str_iso("sdfsdfs sdfsdf 2024-123T12:00:00Z TDB").unwrap();
        assert_eq!(x.yr, Some(2024));
        assert_eq!(x.day_of_yr, Some(123));
        assert_eq!(x.hr, 12);
        assert_eq!(x.min, 0);
        assert_eq!(x.sec, 0);
        assert_eq!(x.scale, Scale::TDB);

        // parse early scale (right after DOY, no time)
        let x = Parts::from_str_iso("sdfsdfs sdfsdf 2024-123TDB fdsfsdfsdf").unwrap();
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
        let dt = Parts::from_str_iso("2024-001TAI").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.day_of_yr, Some(1));
        assert_eq!(dt.scale, Scale::TAI);

        let dt = Parts::from_str_iso("2024-04-18 TDB").unwrap();
        assert_eq!(dt.yr, Some(2024));
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.scale, Scale::TDB);

        // === EARLY scale + time ===
        let dt = Parts::from_str_iso("2024-109T12:00:00LTC").unwrap();
        assert_eq!(dt.day_of_yr, Some(109));
        assert_eq!(dt.hr, 12);
        assert_eq!(dt.min, 0);
        assert_eq!(dt.sec, 0);
        assert_eq!(dt.scale, Scale::LTC);

        let dt = Parts::from_str_iso("2024-04-18 14:30:25 UTC").unwrap();
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hr, 14);
        assert_eq!(dt.min, 30);
        assert_eq!(dt.sec, 25);
        assert_eq!(dt.scale, Scale::UTC);

        // === LATE scale (after time) ===
        let dt = Parts::from_str_iso("2024-001T12:00:00 TDB").unwrap();
        assert_eq!(dt.day_of_yr, Some(1));
        assert_eq!(dt.hr, 12);
        assert_eq!(dt.scale, Scale::TDB);

        let dt = Parts::from_str_iso("2024-06-30T23:59:60Z GPS").unwrap();
        assert_eq!(dt.sec, 60);
        assert_eq!(dt.scale, Scale::GPS);

        // === BOTH orders with fractional seconds ===
        let dt = Parts::from_str_iso("2024-04-18T14:30:25.123456789 TCL").unwrap();
        assert_eq!(dt.scale, Scale::TCL);
        assert_eq!(dt.attos, 123_456_789_000_000_000);

        let dt = Parts::from_str_iso("2024-109 14:30:25.5 TAI").unwrap();
        assert_eq!(dt.scale, Scale::TAI);
        assert_eq!(dt.attos, 500_000_000_000_000_000);

        // === Time completely optional, scale only ===
        let dt = Parts::from_str_iso("2024-001 TAI").unwrap();
        assert_eq!(dt.day_of_yr, Some(1));
        assert_eq!(dt.scale, Scale::TAI);
        assert_eq!(dt.hr, 0); // defaults
        assert_eq!(dt.min, 0);
        assert_eq!(dt.sec, 0);

        let dt = Parts::from_str_iso("2024-04-18UTC").unwrap();
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
            let dt = Parts::from_str_iso(&s).unwrap();
            assert!(dt.scale != Scale::Custom, "failed to parse {}", scale_str);
        }
    }

    #[test]
    fn test_ccsds_no_time_no_scale_still_works() {
        let dt = Parts::from_str_iso("2024-001").unwrap();
        assert_eq!(dt.day_of_yr, Some(1));
        assert_eq!(dt.scale, Scale::UTC); // default is UTC

        let dt = Parts::from_str_iso("2024-04-18").unwrap();
        assert_eq!(dt.mo, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.scale, Scale::UTC); // default is UTC
    }
}
