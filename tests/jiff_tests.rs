#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

//! Interop tests for `Dt` ↔ jiff (`Timestamp`, `Span`, `SignedDuration`).
//!
//! jiff's `Timestamp` is a POSIX/Unix count (UTC offset zero, no leap seconds in
//! the numeric value). Conversion must go through deep-time UTC + Unix epoch —
//! never treat the Unix nanos as a J2000-relative TAI/UTC duration.

#[cfg(feature = "jiff")]
mod interop {
    use deep_time::{Dt, Scale};
    use jiff::{SignedDuration, Span, Timestamp};

    fn assert_ns_eq(a: Dt, b: Dt, label: &str) {
        assert_eq!(
            a.to_ns().0,
            b.to_ns().0,
            "{label}: ns mismatch (attos {} vs {})",
            a.attos,
            b.attos
        );
    }

    fn from_posix_sec(sec: i64) -> Dt {
        let attos = (sec as i128).saturating_mul(1_000_000_000_000_000_000);
        Dt::from_unix(Dt::new(attos, Scale::UTC, Scale::UTC))
    }

    // ─── Timestamp round-trips ───────────────────────────────────────────────

    #[test]
    fn timestamp_roundtrip_unix_epoch() {
        let dt = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = dt.to_jiff_timestamp();
        assert_eq!(ts.as_second(), 0);
        assert_eq!(ts.as_nanosecond(), 0);
        assert_ns_eq(Dt::from_jiff_timestamp(ts), dt, "unix epoch");
    }

    #[test]
    fn timestamp_roundtrip_j2000_utc_noon() {
        let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
        let ts = dt.to_jiff_timestamp();
        assert_eq!(ts.as_second(), 946_728_000);
        assert_ns_eq(Dt::from_jiff_timestamp(ts), dt, "j2000 utc noon");
    }

    #[test]
    fn timestamp_roundtrip_modern_with_subsec() {
        // 123_456_789 ns → 123_456_789_000_000_000 attos
        let dt = Dt::from_ymd(2024, 4, 15, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);
        let ts = dt.to_jiff_timestamp();
        assert_eq!(ts.as_second(), 1_713_191_445);
        assert_eq!(ts.as_nanosecond(), 1_713_191_445_123_456_789);
        assert_ns_eq(Dt::from_jiff_timestamp(ts), dt, "2024 with ns");
    }

    #[test]
    fn timestamp_roundtrip_near_leap_seconds() {
        // Just after the 2016-12-31 leap second
        let dt = Dt::from_ymd(2017, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = dt.to_jiff_timestamp();
        assert_eq!(ts.as_second(), 1_483_228_800);
        assert_ns_eq(Dt::from_jiff_timestamp(ts), dt, "2017-01-01 UTC");
    }

    #[test]
    fn timestamp_roundtrip_across_leap_history() {
        // Dates where a buggy from_jiff (treating Unix ns as J2000-relative UTC)
        // previously applied the wrong leap-second offset.
        let cases: &[(i64, &str)] = &[
            (0, "1970-01-01"),
            (63_072_000, "1972-01-01"),
            (78_796_800, "1972-07-01 first leap"),
            (315_964_800, "1980-01-06 GPS"),
            (946_684_799, "1999-12-31 23:59:59"),
            (946_728_000, "2000-01-01 noon"),
            (1_435_708_800, "2015-07-01"),
            (1_483_228_800, "2017-01-01"),
            (1_577_836_800, "2020-01-01"),
            (1_713_191_445, "2024-04-15 14:30:45"),
            (-315_619_200, "1960-01-01"),
        ];

        for &(unix_sec, label) in cases {
            let ts = Timestamp::from_second(unix_sec).unwrap();
            let dt = Dt::from_jiff_timestamp(ts);
            let expected = from_posix_sec(unix_sec);

            assert_eq!(
                dt.to_attos(),
                expected.to_attos(),
                "{label}: from_jiff attos mismatch"
            );
            assert_eq!(
                dt.to_jiff_timestamp().as_second(),
                unix_sec,
                "{label}: round-trip unix second"
            );
            assert_eq!(
                expected.to_jiff_timestamp().as_second(),
                unix_sec,
                "{label}: to_jiff from known instant"
            );
        }
    }

    #[test]
    fn timestamp_from_tai_uses_utc_unix() {
        // Same civil UTC instant should map to the same Unix timestamp whether
        // the Dt was built as UTC or as the equivalent TAI instant.
        let utc = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let tai = utc.to_tai();
        assert_eq!(
            utc.to_jiff_timestamp().as_nanosecond(),
            tai.to_jiff_timestamp().as_nanosecond()
        );
    }

    #[test]
    fn timestamp_is_posix_not_tai() {
        // At 2020-01-01, TAI-UTC is 37 s. A mistaken TAI Unix conversion would
        // shift the second count by that amount.
        let utc = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let ts = utc.to_jiff_timestamp();
        assert_eq!(ts.as_second(), 1_577_836_800);

        // Reconstructing from that Unix second and reading civil UTC fields
        // must yield 2020-01-01, not 2019-12-31 23:59:23.
        let back = Dt::from_jiff_timestamp(ts);
        let ymd = back.target(Scale::UTC).to_ymd();
        assert_eq!(ymd.yr(), 2020);
        assert_eq!(ymd.mo(), 1);
        assert_eq!(ymd.day(), 1);
        assert_eq!(ymd.hr(), 0);
        assert_eq!(ymd.min(), 0);
        assert_eq!(ymd.sec(), 0);
    }

    #[test]
    fn timestamp_roundtrip_pre_1972() {
        // Pre-1972: modern UTC leap table does not apply rubber-second offsets;
        // POSIX still counts SI-like seconds from 1970.
        let dt = Dt::from_ymd(1969, 7, 20, Scale::UTC, 20, 17, 0, 0);
        let ts = dt.to_jiff_timestamp();
        assert_ns_eq(Dt::from_jiff_timestamp(ts), dt, "moon landing");
    }

    #[test]
    fn timestamp_from_jiff_matches_from_unix() {
        let nanos: i128 = 1_713_191_445_123_456_789;
        let ts = Timestamp::from_nanosecond(nanos).unwrap();
        let via_jiff = Dt::from_jiff_timestamp(ts);
        let via_unix = Dt::from_unix(Dt::new(
            nanos.saturating_mul(1_000_000_000),
            Scale::UTC,
            Scale::UTC,
        ));
        assert_ns_eq(via_jiff, via_unix, "from_jiff == from_unix");
    }

    #[test]
    fn timestamp_leap_second_civil_maps_like_jiff() {
        // deep-time can represent 23:59:60; jiff does not. to_jiff goes through
        // Unix (POSIX), so the leap second shares the same Unix second as the
        // following 00:00:00 in many leap-second models — assert both directions
        // stay consistent with our UTC tables and jiff's no-leap POSIX count.
        let after = Dt::from_ymd(2017, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let after_ts = after.to_jiff_timestamp();
        assert_eq!(after_ts.as_second(), 1_483_228_800);

        // jiff constrains second=60 → 59 when parsing; our conversion is numeric.
        let jiff_parsed: Timestamp = "2016-12-31T23:59:60Z".parse().unwrap();
        assert_eq!(jiff_parsed.to_string(), "2016-12-31T23:59:59Z");

        let from_parsed = Dt::from_jiff_timestamp(jiff_parsed);
        let ymd = from_parsed.target(Scale::UTC).to_ymd();
        assert_eq!(ymd.yr(), 2016);
        assert_eq!(ymd.mo(), 12);
        assert_eq!(ymd.day(), 31);
        assert_eq!(ymd.hr(), 23);
        assert_eq!(ymd.min(), 59);
        assert_eq!(ymd.sec(), 59);
    }

    #[test]
    fn timestamp_truncates_sub_nanosecond() {
        // 1.5 ns of attoseconds beyond an exact Unix second → truncate toward 0 on to_jiff
        let base = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let with_sub_ns = base.add(Dt::from_attos(1_500_000_000, Scale::TAI)); // 1.5 ns
        let ts = with_sub_ns.to_jiff_timestamp();
        // Truncation toward zero: +1 ns of the 1.5 ns
        assert_eq!(
            ts.as_nanosecond(),
            base.to_jiff_timestamp().as_nanosecond() + 1
        );
    }

    #[test]
    fn timestamp_saturates_out_of_range() {
        let far_future = Dt::from_ymd(50_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(far_future.to_jiff_timestamp(), Timestamp::MAX);

        let far_past = Dt::from_ymd(-50_000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(far_past.to_jiff_timestamp(), Timestamp::MIN);
    }

    #[test]
    fn timestamp_roundtrip_negative_nanos() {
        // Slightly before Unix epoch
        let ts = Timestamp::from_nanosecond(-1).unwrap();
        let dt = Dt::from_jiff_timestamp(ts);
        assert_eq!(dt.to_jiff_timestamp().as_nanosecond(), -1);
        let ymd = dt.target(Scale::UTC).to_ymd();
        assert_eq!(ymd.yr(), 1969);
        assert_eq!(ymd.mo(), 12);
        assert_eq!(ymd.day(), 31);
    }

    // ─── Span / SignedDuration ───────────────────────────────────────────────

    #[test]
    fn duration_roundtrip() {
        let span = Dt::from_ns_floor(3_600_000_000_000 + 123, 0, Scale::TAI); // 1 hour + 123 ns
        let dur = span.to_jiff_signed_duration();
        assert_eq!(dur.as_secs(), 3_600);
        assert_eq!(dur.subsec_nanos(), 123);
        assert_ns_eq(Dt::from_jiff_signed_duration(dur), span, "duration");
    }

    #[test]
    fn duration_negative_roundtrip() {
        let span = Dt::from_ns_floor(-5_000_000_001, 0, Scale::TAI);
        let dur = span.to_jiff_signed_duration();
        assert!(dur.is_negative());
        assert_eq!(dur.as_nanos(), -5_000_000_001);
        assert_ns_eq(Dt::from_jiff_signed_duration(dur), span, "neg duration");
    }

    #[test]
    fn duration_truncates_sub_nanosecond() {
        // 1.5 ns worth of attoseconds → truncates toward zero to 1 ns
        let span = Dt::from_attos(1_500_000_000, Scale::TAI);
        let dur = span.to_jiff_signed_duration();
        assert_eq!(dur.as_nanos(), 1);
    }

    #[test]
    fn span_roundtrip() {
        let dt = Dt::from_ns_floor(3_600_000_000_000 + 123, 0, Scale::TAI);
        let jiff_span = dt.to_jiff_span();
        let back = Dt::from_jiff_span(jiff_span).unwrap();
        assert_ns_eq(back, dt, "span");
    }

    #[test]
    fn span_negative_roundtrip() {
        let dt = Dt::from_ns_floor(-90_000_000_001, 0, Scale::TAI);
        let jiff_span = dt.to_jiff_span();
        let back = Dt::from_jiff_span(jiff_span).unwrap();
        assert_ns_eq(back, dt, "neg span");
    }

    #[test]
    fn span_zero() {
        let dt = Dt::from_ns_floor(0, 0, Scale::TAI);
        let jiff_span = dt.to_jiff_span();
        assert_eq!(jiff_span.fieldwise(), Span::new().fieldwise());
        assert_ns_eq(Dt::from_jiff_span(jiff_span).unwrap(), dt, "zero span");
    }

    #[test]
    fn span_and_duration_agree() {
        let dt = Dt::from_ns_floor(12_345_678_901, 0, Scale::TAI);
        let from_span = Dt::from_jiff_span(dt.to_jiff_span()).unwrap();
        let from_dur = Dt::from_jiff_signed_duration(dt.to_jiff_signed_duration());
        assert_ns_eq(from_span, from_dur, "span vs duration");
    }
}

// TZ / civil parsing comparisons (require jiff-tz + parse). These exercise
// deep-time's parser against jiff as ground truth; they do not replace the
// Dt↔Timestamp interop tests above.

#[cfg(all(feature = "jiff-tz", feature = "parse"))]
mod tz_parse {
    use deep_time::{Dt, ParseCfg};
    use jiff::{Timestamp, Zoned, civil::DateTime};

    #[test]
    fn test_historical_iana_with_jiff() {
        let cases = vec![
            (
                "2020-06-15 14:30:00",
                "America/New_York",
                "NY summer (EDT = UTC-4)",
            ),
            (
                "2020-12-15 14:30:00",
                "America/New_York",
                "NY winter (EST = UTC-5)",
            ),
            (
                "2023-11-05 01:30:00",
                "America/New_York",
                "NY DST fall-back (ambiguous hour - overlap)",
            ),
            (
                "2023-03-12 02:30:00",
                "America/New_York",
                "NY DST spring-forward (gap)",
            ),
            (
                "2024-03-10 02:30:00",
                "America/New_York",
                "Recent spring-forward gap",
            ),
            (
                "2024-11-03 01:30:00",
                "America/New_York",
                "Recent fall-back overlap",
            ),
            ("1999-12-31 23:59:59", "Europe/London", "London historical"),
            (
                "1969-07-20 20:17:00",
                "America/New_York",
                "Moon landing (very old TZ rules)",
            ),
            (
                "1955-01-01 00:00:00",
                "Europe/London",
                "Mid-20th century rules (pre-modern DST)",
            ),
            ("1970-01-01 00:00:00", "UTC", "UTC edge case"),
            (
                "2024-06-15 14:30:00",
                "Asia/Kolkata",
                "Half-hour offset (India UTC+5:30 - no DST)",
            ),
            (
                "2024-02-15 12:00:00",
                "Australia/Sydney",
                "Southern hemisphere DST (spring-forward in Oct)",
            ),
            (
                "2024-12-15 14:30:00",
                "Pacific/Chatham",
                "Unusual 45-minute offset (Chatham Islands)",
            ),
            (
                "2035-07-01 12:00:00",
                "America/New_York",
                "Far future date (repeating tail / far-future logic)",
            ),
            (
                "2023-03-12 02:00:00",
                "America/New_York",
                "Exact spring-forward transition moment (gap boundary)",
            ),
            (
                "2023-11-05 01:00:00",
                "America/New_York",
                "Exact fall-back transition moment (overlap boundary)",
            ),
            (
                "2024-10-06 02:30:00",
                "Australia/Sydney",
                "Southern hemisphere spring-forward gap",
            ),
            (
                "2023-10-01 12:00:00",
                "Pacific/Honolulu",
                "Zone with no DST ever (Hawaii)",
            ),
            (
                "1800-01-01 12:00:00",
                "Europe/London",
                "Very old historical (LMT era)",
            ),
            (
                "2024-06-15 14:30:00",
                "Asia/Kathmandu",
                "Nepal +5:45 offset (no DST, non-whole-hour)",
            ),
            (
                "2024-06-15 14:30:00",
                "Australia/Lord_Howe",
                "Lord Howe Island +10:30 standard / +11:00 DST (30-minute DST jump)",
            ),
            (
                "2024-06-15 14:30:00",
                "Australia/Eucla",
                "Australian Central Western +8:45 (another 45-minute offset, no DST)",
            ),
            (
                "2024-07-15 12:00:00",
                "America/Phoenix",
                "Arizona fixed UTC-7, never observes DST (contrasts with other US zones)",
            ),
            (
                "2024-06-15 14:30:00",
                "Pacific/Kiritimati",
                "Kiribati +14:00 (most eastern offset on Earth)",
            ),
            (
                "2011-12-29 12:00:00",
                "Pacific/Apia",
                "Samoa BEFORE 2011 dateline jump (UTC-11)",
            ),
            (
                "2011-12-31 12:00:00",
                "Pacific/Apia",
                "Samoa AFTER 2011 dateline jump (UTC+13) — day skipped on 2011-12-30",
            ),
            (
                "2024-06-15 14:30:00",
                "Europe/Dublin",
                "Ireland — uses negative DST modeling in TZDB (can expose subtle bugs)",
            ),
            (
                "2015-08-15 00:00:00",
                "Asia/Pyongyang",
                "North Korea adopts UTC+8:30 (Pyongyang Time)",
            ),
            (
                "2018-05-05 00:00:00",
                "Asia/Pyongyang",
                "North Korea returns to UTC+9",
            ),
            (
                "1883-11-17 12:00:00",
                "America/New_York",
                "Day before US standard time (LMT era)",
            ),
            (
                "1883-11-18 12:00:00",
                "America/New_York",
                "US standard time begins (EST UTC-5 at noon)",
            ),
            ("2024-06-15 14:30:00", "US/Eastern", "Zone alias US/Eastern"),
            ("2024-06-15 14:30:00", "US/Pacific", "Zone alias US/Pacific"),
            ("2024-06-15 14:30:00", "Etc/UTC", "Etc/UTC alias"),
            ("2024-06-15 14:30:00", "GMT", "GMT alias"),
            (
                "2024-06-15 14:30:00",
                "Etc/GMT+5",
                "Etc/GMT+5 (note: means UTC-5)",
            ),
            ("2024-06-15 14:30:00", "Zulu", "Zulu alias (if supported)"),
            (
                "2024-06-15 14:30:00.123456789",
                "America/New_York",
                "Full nanosecond precision",
            ),
            (
                "2024-06-15 14:30:00.5",
                "America/New_York",
                "Half-second precision",
            ),
            (
                "2024-11-03 01:30:00.999999999",
                "America/New_York",
                "Subsecond during fall-back overlap",
            ),
            (
                "2024-09-29 02:30:00",
                "Pacific/Auckland",
                "New Zealand spring-forward gap",
            ),
            (
                "2024-04-07 02:30:00",
                "Pacific/Auckland",
                "New Zealand fall-back overlap",
            ),
            (
                "2024-10-06 02:00:00",
                "America/Santiago",
                "Chile spring-forward (recent rule changes)",
            ),
            (
                "2024-12-31 23:59:59",
                "America/New_York",
                "Recent Dec 31 - should be EST",
            ),
            ("9999-01-01 00:00:00", "UTC", "Year 9999 on UTC (safe)"),
            (
                "9998-12-31 23:59:59",
                "America/New_York",
                "Far future year 9998 in EST (still safe)",
            ),
            (
                "9999-06-15 12:00:00",
                "Pacific/Kiritimati",
                "Year 9999 on +14 zone (pushes closest to limit)",
            ),
            (
                "0002-01-01 00:00:00",
                "UTC",
                "Year 2 proleptic Gregorian (safe lower bound)",
            ),
            ("0001-01-01 00:00:00", "UTC", "Year 1 on UTC only"),
            (
                "2025-06-15 12:00:00",
                "Etc/GMT+12",
                "Fixed +12 with no DST ever",
            ),
            (
                "2025-06-15 12:00:00",
                "Etc/GMT-12",
                "Fixed -12 with no DST ever",
            ),
            (
                "2025-06-15 12:00:00",
                "Asia/Amman",
                "Jordan abolished DST in 2022",
            ),
            (
                "2023-03-12 02:00:00.000000000",
                "America/New_York",
                "Exact spring-forward second (nanosecond)",
            ),
            (
                "2023-11-05 01:00:00.999999999",
                "America/New_York",
                "Last nanosecond before fall-back overlap",
            ),
            ("1700-01-01 12:00:00", "Europe/London", "Pre-1800 LMT era"),
            (
                "9999-06-15 12:00:00",
                "Australia/Eucla",
                "Far future on a None zone (+8:45 fixed)",
            ),
            (
                "2023-03-12 02:00:00.5",
                "America/New_York",
                "Half-second inside spring-forward gap",
            ),
            (
                "2023-11-05 01:30:00.123456789",
                "America/New_York",
                "Subsecond deep inside fall-back overlap",
            ),
            (
                "2025-03-01 12:00:00",
                "Africa/Casablanca",
                "Morocco has changed DST rules many times",
            ),
            (
                "2025-06-15 12:00:00",
                "America/Argentina/Buenos_Aires",
                "Argentina abolished DST in 2009",
            ),
            (
                "2024-10-06 02:00:00.000000000",
                "Australia/Sydney",
                "Exact spring-forward second (Sydney)",
            ),
            (
                "2025-03-09 02:30:00",
                "America/New_York",
                "NY 2025 spring-forward gap (non-existent local time)",
            ),
            (
                "2025-03-09 02:00:00",
                "America/New_York",
                "Exact NY 2025 spring-forward boundary (gap start)",
            ),
            (
                "2025-03-09 02:30:00.5",
                "America/New_York",
                "Half-second inside NY 2025 spring-forward gap",
            ),
            (
                "2025-03-30 01:30:00",
                "Europe/London",
                "London 2025 spring-forward gap (1am→2am BST, non-existent)",
            ),
            (
                "2025-03-30 01:00:00",
                "Europe/London",
                "Exact London 2025 spring-forward boundary",
            ),
            (
                "2025-03-30 01:30:00.123456789",
                "Europe/London",
                "Subsecond deep inside London 2025 spring-forward gap",
            ),
            (
                "2025-03-09 02:30:00",
                "America/Chicago",
                "Chicago 2025 spring-forward gap (different US zone)",
            ),
            (
                "2024-10-06 02:30:00.999999999",
                "Australia/Sydney",
                "Last nanosecond inside Sydney spring-forward gap",
            ),
            (
                "2023-10-29 01:30:00",
                "Europe/London",
                "UK DST fall-back overlap (BST → GMT) - prefers earlier occurrence",
            ),
            (
                "9999-06-15 12:00:00",
                "America/New_York",
                "Far future UTC path on repeating zone (NY)",
            ),
            (
                "9998-07-01 00:00:00",
                "Europe/London",
                "Far future on Europe/London repeating cycle",
            ),
            (
                "9999-01-01 00:00:00",
                "Australia/Eucla",
                "Far future on a zone that has no repeating rule",
            ),
            (
                "9999-03-11 07:00:00",
                "America/New_York",
                "Far future spring-forward transition (UTC path)",
            ),
            (
                "3000-12-31 23:59:59",
                "America/Chicago",
                "Year 3000 on repeating US zone",
            ),
        ];

        for (civil_str, iana_name, description) in cases {
            let civil_dt: DateTime = civil_str
                .parse()
                .unwrap_or_else(|e| panic!("Jiff civil parse failed for '{civil_str}': {e}"));

            let jiff_zoned: Zoned = civil_dt
                .in_tz(iana_name)
                .unwrap_or_else(|e| panic!("Jiff in_tz('{iana_name}') failed: {e}"));
            let jiff_str = jiff_zoned.to_string();

            let our_input = format!("{civil_str} {iana_name}");
            let our_dt: Dt = our_input
                .parse()
                .unwrap_or_else(|e| panic!("deep_time failed on '{our_input}': {e}"));
            let our_str = our_dt.to_str_rfc9557(&format!("{iana_name}")).unwrap();

            assert_eq!(
                our_str, jiff_str,
                "\n=== IANA Historical Test FAILED: {description} ===\n\
             Input string   : {our_input}\n\
             Jiff           : {jiff_str}\n\
             deep_time      : {our_str}\n"
            );

            // Also check Dt ↔ jiff::Timestamp numeric agreement on the instant.
            let our_ts = our_dt.to_jiff_timestamp();
            assert_eq!(
                our_ts.as_nanosecond(),
                jiff_zoned.timestamp().as_nanosecond(),
                "{description}: Timestamp ns mismatch for {our_input}"
            );
        }
    }

    #[test]
    fn test_offset_handling_with_jiff() {
        let cases = vec![
            ("2024-06-15 14:30:00", "+00:00", "UTC / +0"),
            ("2024-06-15 14:30:00", "+05:00", "UTC+5"),
            ("2024-06-15 14:30:00", "-04:00", "UTC-4 (EDT-style)"),
            ("2024-06-15 14:30:00", "-05:00", "UTC-5 (EST)"),
            ("2024-06-15 14:30:00", "+08:00", "UTC+8"),
            ("2024-06-15 14:30:00", "-12:00", "UTC-12"),
            ("2024-06-15 14:30:00", "+14:00", "UTC+14 (easternmost)"),
            ("2024-06-15 14:30:00", "+05:30", "India +5:30"),
            ("2024-06-15 14:30:00", "+05:45", "Nepal +5:45"),
            ("2024-06-15 14:30:00", "+08:45", "Australia/Eucla +8:45"),
            (
                "2024-06-15 14:30:00",
                "+10:30",
                "Lord Howe +10:30 (standard)",
            ),
            ("2024-06-15 14:30:00", "-09:30", "Marquesas Islands -9:30"),
            ("2024-06-15 14:30:00", "+12:45", "Chatham Islands +12:45"),
            (
                "2024-06-15 14:30:00.123456789",
                "+05:30",
                "Full nanosecond precision",
            ),
            ("2024-06-15 14:30:00.5", "+08:00", "Half-second precision"),
            (
                "2024-11-05 01:30:00.999999999",
                "-05:00",
                "Last nanosecond of a second",
            ),
            (
                "1800-01-01 12:00:00",
                "-00:44",
                "Very old fixed offset (LMT era)",
            ),
            (
                "1883-11-18 12:00:00",
                "-05:00",
                "US Standard Time adoption day",
            ),
            ("9998-12-31 23:59:59", "+14:00", "Far future +14:00"),
            ("0001-01-01 00:00:00", "+00:00", "Year 1 with fixed offset"),
            ("2024-06-15 14:30:00", "Z", "Zulu alias (should be +00:00)"),
            ("2024-06-15 14:30:00", "z", "Lowercase z alias"),
        ];

        for (civil_str, offset_str, description) in cases {
            let civil_with_t = civil_str.replace(' ', "T");
            let offset_part = if offset_str.eq_ignore_ascii_case("z") {
                "Z"
            } else {
                offset_str
            };

            let jiff_input = format!("{civil_with_t}{offset_part}");

            let jiff_ts: Timestamp = jiff_input.parse().unwrap_or_else(|e| {
                panic!(
                    "Jiff Timestamp::parse failed for '{jiff_input}': {e}\n\
                     (original civil: {civil_str}, offset: {offset_str})"
                )
            });

            let jiff_rfc = jiff_ts.to_string();

            let our_input = format!("{civil_str} {offset_str}");
            let our_dt: Dt = Dt::from_str_parse(&our_input, &ParseCfg::DEFAULT)
                .unwrap_or_else(|e| panic!("deep_time failed on '{our_input}': {e}"));

            let our_rfc = our_dt.to_str_rfc3339();

            assert_eq!(
                our_rfc, jiff_rfc,
                "\n=== Offset Handling Test FAILED: {description} ===\n\
                 Input string   : {our_input}\n\
                 Jiff input     : {jiff_input}\n\
                 Jiff           : {jiff_rfc}\n\
                 deep_time      : {our_rfc}\n"
            );

            // Numeric Timestamp agreement (string form can differ in Z vs +00:00).
            assert_eq!(
                our_dt.to_jiff_timestamp().as_nanosecond(),
                jiff_ts.as_nanosecond(),
                "{description}: Timestamp ns mismatch"
            );
            assert_eq!(
                Dt::from_jiff_timestamp(jiff_ts)
                    .to_jiff_timestamp()
                    .as_nanosecond(),
                jiff_ts.as_nanosecond(),
                "{description}: from_jiff_timestamp round-trip"
            );
        }
    }
}
