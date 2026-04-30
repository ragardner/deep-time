#[cfg(feature = "jiff-tz")]
mod tests {
    use deep_time_core::TimePoint;
    use jiff::{Zoned, civil::DateTime};

    #[test]
    fn test_historical_iana_with_jiff() {
        // Historical / interesting IANA timezone cases.
        // Jiff is used as the ground-truth (full tzdb historical rules).
        // Your parser gets the exact same input string.

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
        ];

        for (civil_str, iana_name, description) in cases {
            // ─── Jiff ground truth ─────────────────────────────────────────────────────
            let civil_dt: DateTime = civil_str
                .parse()
                .unwrap_or_else(|e| panic!("Jiff civil parse failed for '{}': {}", civil_str, e));

            let jiff_zoned: Zoned = civil_dt
                .in_tz(iana_name)
                .unwrap_or_else(|e| panic!("Jiff in_tz('{}') failed: {}", iana_name, e));

            // CHANGED: Convert to Timestamp so Jiff prints the *same* pure UTC RFC 3339
            // string with Z that deep_time_core::to_rfc3339() produces.
            let jiff_rfc = jiff_zoned.timestamp().to_string();

            // ─── Your library ──────────────────────────────────────────────────────────
            let our_input = format!("{} {}", civil_str, iana_name);

            let our_dt: TimePoint = TimePoint::from_str_parse(&our_input, &None)
                .unwrap_or_else(|e| panic!("deep_time_core failed on '{}': {}", our_input, e));

            let our_rfc = our_dt.to_str_rfc3339().unwrap();

            // ─── Assert (no more manual prints) ────────────────────────────────────────
            assert_eq!(
                our_rfc, jiff_rfc,
                "\n=== IANA Historical Test FAILED: {} ===\n\
             Input string       : {}\n\
             Jiff               : {}\n\
             deep_time_core     : {}\n",
                description, our_input, jiff_rfc, our_rfc
            );
        }
    }
}
