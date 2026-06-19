#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "jiff-tz")]
mod tests {
    use deep_time::{Dt, ParseCfg, Scale};
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
            //
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
            // North Korea "Pyongyang Time" experiment
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
            // US/Canada standard time adoption (LMT → modern zones)
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
            // Far future — safe across almost all zones
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
            // Ancient dates — safe
            (
                "0002-01-01 00:00:00",
                "UTC",
                "Year 2 proleptic Gregorian (safe lower bound)",
            ),
            ("0001-01-01 00:00:00", "UTC", "Year 1 on UTC only"),
            // 1. Fixed-offset zone with NO repeating rules at all (should always return None)
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
            // 2. A zone that abolished DST relatively recently (tests that we don't wrongly apply old repeating rules)
            (
                "2025-06-15 12:00:00",
                "Asia/Amman",
                "Jordan abolished DST in 2022",
            ),
            // 3. Subsecond precision exactly at a transition instant
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
            // 4. Very early date (before most explicit transitions) — should still work via first transition / LMT
            ("1700-01-01 12:00:00", "Europe/London", "Pre-1800 LMT era"),
            // 5. A date far in the future on a zone that has Repeating::None
            //    (tests that we keep the last known offset forever)
            (
                "9999-06-15 12:00:00",
                "Australia/Eucla",
                "Far future on a None zone (+8:45 fixed)",
            ),
            // 6. Overlap/gap with subsecond that lands inside the gap/overlap window
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
            // 7. Zone with a very recent rule change that should still get Repeating::None or correct Cycle
            (
                "2025-03-01 12:00:00",
                "Africa/Casablanca",
                "Morocco has changed DST rules many times",
            ),
            // 8. A zone whose repeating pattern starts quite late (tests truncation optimization)
            (
                "2025-06-15 12:00:00",
                "America/Argentina/Buenos_Aires",
                "Argentina abolished DST in 2009",
            ),
            // 9. Exact transition second in southern hemisphere
            (
                "2024-10-06 02:00:00.000000000",
                "Australia/Sydney",
                "Exact spring-forward second (Sydney)",
            ),
            // === More non-existent times (DST spring-forward gaps) ===
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
            // 1. Far future UTC instant → local time (exercises offset_info_at_utc + Cycle)
            (
                "9999-06-15 12:00:00",
                "America/New_York",
                "Far future UTC path on repeating zone (NY)",
            ),
            // 2. Another repeating zone far in the future
            (
                "9998-07-01 00:00:00",
                "Europe/London",
                "Far future on Europe/London repeating cycle",
            ),
            // 3. A zone that becomes fixed (Repeating::None / Future::Fixed) far in future
            (
                "9999-01-01 00:00:00",
                "Australia/Eucla",
                "Far future on a zone that has no repeating rule",
            ),
            // 4. Exact transition moment in far future (tests precision of cycle math)
            (
                "9999-03-11 07:00:00",
                "America/New_York",
                "Far future spring-forward transition (UTC path)",
            ),
            // 5. Very far future on a simple repeating zone
            (
                "3000-12-31 23:59:59",
                "America/Chicago",
                "Year 3000 on repeating US zone",
            ),
            // ("2006-04-02 02:30-05", "America/Indiana/Vevay", "github"), // errors on jiff temporal
        ];

        for (civil_str, iana_name, description) in cases {
            // ─── Jiff ground truth ─────────────────────────────────────────────────────
            let civil_dt: DateTime = civil_str
                .parse()
                .unwrap_or_else(|e| panic!("Jiff civil parse failed for '{}': {}", civil_str, e));

            let jiff_zoned: Zoned = civil_dt
                .in_tz(iana_name)
                .unwrap_or_else(|e| panic!("Jiff in_tz('{}') failed: {}", iana_name, e));
            // let jiff_str = jiff_zoned.timestamp().to_string();
            let jiff_str = jiff_zoned.to_string();

            // ─── deep-time ──────────────────────────────────────────────────────────
            let our_input = format!("{} {}", civil_str, iana_name);

            let our_dt: Dt = our_input
                .parse()
                .unwrap_or_else(|e| panic!("deep_time failed on '{}': {}", our_input, e));
            let our_str = our_dt.to_str_rfc9557(&format!("{}", iana_name)).unwrap();

            // ─── Assert (no more manual prints) ────────────────────────────────────────
            assert_eq!(
                our_str, jiff_str,
                "\n=== IANA Historical Test FAILED: {} ===\n\
             Input string   : {}\n\
             Jiff           : {}\n\
             deep_time      : {}\n",
                description, our_input, jiff_str, our_str
            );
        }
    }

    #[test]
    fn test_offset_handling_with_jiff() {
        let cases = vec![
            // === Basic whole-hour offsets ===
            ("2024-06-15 14:30:00", "+00:00", "UTC / +0"),
            ("2024-06-15 14:30:00", "+05:00", "UTC+5"),
            ("2024-06-15 14:30:00", "-04:00", "UTC-4 (EDT-style)"),
            ("2024-06-15 14:30:00", "-05:00", "UTC-5 (EST)"),
            ("2024-06-15 14:30:00", "+08:00", "UTC+8"),
            ("2024-06-15 14:30:00", "-12:00", "UTC-12"),
            ("2024-06-15 14:30:00", "+14:00", "UTC+14 (easternmost)"),
            // === Non-whole-hour offsets (the ones that catch bugs) ===
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
            // === Subsecond precision ===
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
            // === Historical / far-future / edge cases ===
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
            // === Z / Zulu alias ===
            ("2024-06-15 14:30:00", "Z", "Zulu alias (should be +00:00)"),
            ("2024-06-15 14:30:00", "z", "Lowercase z alias"),
        ];

        for (civil_str, offset_str, description) in cases {
            // ─── Jiff ground truth ─────────────────────────────────────────────────────
            let civil_with_t = civil_str.replace(' ', "T");
            let offset_part = if offset_str.eq_ignore_ascii_case("z") {
                "Z"
            } else {
                offset_str
            };

            let jiff_input = format!("{}{}", civil_with_t, offset_part);

            let jiff_ts: Timestamp = jiff_input.parse().unwrap_or_else(|e| {
                panic!(
                    "Jiff Timestamp::parse failed for '{}': {}\n\
                     (original civil: {}, offset: {})",
                    jiff_input, e, civil_str, offset_str
                )
            });

            let jiff_rfc = jiff_ts.to_string();

            // ─── deep_time ─────────────────────────────────────────────────────────────
            let our_input = format!("{} {}", civil_str, offset_str);

            let our_dt: Dt = Dt::from_str_parse(&our_input, &ParseCfg::DEFAULT)
                .unwrap_or_else(|e| panic!("deep_time failed on '{}': {}", our_input, e));

            let our_rfc = our_dt.to_str_rfc3339();

            // ─── Assert ────────────────────────────────────────────────────────────────
            assert_eq!(
                our_rfc, jiff_rfc,
                "\n=== Offset Handling Test FAILED: {} ===\n\
                 Input string   : {}\n\
                 Jiff input     : {}\n\
                 Jiff           : {}\n\
                 deep_time      : {}\n",
                description, our_input, jiff_input, jiff_rfc, our_rfc
            );
        }
    }
}
