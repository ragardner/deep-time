//! Red-team / characterization tests for [`Dt::from_str_parse`].
//!
//! Complements the combinatorial coverage in `date_tests.rs` and the relative
//! phrase suite in `date_relative_tests.rs` by probing:
//! - guardrails (empty / oversize / whitespace)
//! - calendar validity (leap years, ordinal/week bounds)
//! - ambiguous numeric order under Smart / Day / Month / Year
//! - pure-numeric modes (Auto / Legacy / Scientific / UnixTimestamp)
//! - named English dates, ordinals, 12-hour clock quirks
//! - syslog year inference
//! - relative phrases + bare times-of-day
//! - offsets, leap seconds, scale suffixes
//! - separators (unicode dashes, fullwidth digits, JP calendar units)
//! - adversarial / lenient garbage handling
//! - `ParseCfg` knobs (explicit formats, `to_lower`, `relative`)
//! - `str_to_*` convenience helpers
//!
//! Several cases document *current* behavior that is surprising; those are
//! labeled `// CHARACTERIZATION` so they can be revisited deliberately.

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "parse")]
mod tests {
    use deep_time::{Dt, DtErrKind, Mode, Order, ParseCfg, Scale};

    // ── helpers ────────────────────────────────────────────────────────────

    fn def() -> ParseCfg {
        ParseCfg::DEFAULT
    }

    /// Fixed "now": Wednesday 2025-01-15 12:00 UTC
    fn ref_cfg() -> ParseCfg {
        ParseCfg {
            ref_time: Some(Dt::from_ymd(2025, 1, 15, Scale::UTC, 12, 0, 0, 0)),
            ..Default::default()
        }
    }

    fn cfg_order(order: Order) -> ParseCfg {
        ParseCfg {
            order,
            ..Default::default()
        }
    }

    fn cfg_mode(mode: Mode) -> ParseCfg {
        ParseCfg {
            mode,
            ..Default::default()
        }
    }

    fn parse(input: &str, cfg: &ParseCfg) -> Dt {
        Dt::from_str_parse(input, cfg).unwrap_or_else(|e| {
            panic!("expected Ok for {input:?}, got Err({e})");
        })
    }

    fn assert_rfc(input: &str, expected: &str, cfg: &ParseCfg) {
        let dt = parse(input, cfg);
        let actual = dt.to_str_rfc3339_nf(9);
        assert_eq!(actual, expected, "input={input:?}");
    }

    fn assert_err(input: &str, cfg: &ParseCfg) {
        assert!(
            Dt::from_str_parse(input, cfg).is_err(),
            "expected Err for {input:?}"
        );
    }

    fn assert_err_kind(input: &str, cfg: &ParseCfg, kind: DtErrKind) {
        match Dt::from_str_parse(input, cfg) {
            Ok(dt) => panic!(
                "expected Err({kind:?}) for {input:?}, got Ok({})",
                dt.to_str_rfc3339_nf(9)
            ),
            Err(e) => assert_eq!(e.kind(), kind, "input={input:?}, err={e}"),
        }
    }

    // ── 1. Guardrails ──────────────────────────────────────────────────────

    #[test]
    fn guardrails_empty_and_oversize() {
        let cfg = def();
        assert_err_kind("", &cfg, DtErrKind::Empty);
        // STRTIME_SIZE is 512; one past that is InvalidLen.
        assert_err_kind(&"x".repeat(513), &cfg, DtErrKind::InvalidLen);
        // Exactly at the limit is length-ok but content is garbage.
        assert_err(&"x".repeat(512), &cfg);
        assert_err("   ", &cfg);
        assert_err("\t\n", &cfg);
    }

    // ── 2. ISO / calendar validity ─────────────────────────────────────────

    #[test]
    fn iso_happy_paths_and_offsets() {
        let cfg = def();
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("2024-03-15T14:30:00Z", "2024-03-15T14:30:00Z", &cfg);
        assert_rfc(
            "2024-03-15T14:30:00.123456789Z",
            "2024-03-15T14:30:00.123456789Z",
            &cfg,
        );
        assert_rfc("2024-03-15T14:30:00.5Z", "2024-03-15T14:30:00.5Z", &cfg);
        assert_rfc("20240315T143000", "2024-03-15T14:30:00Z", &cfg);
        // Offset is applied; result is the UTC instant.
        assert_rfc("2024-03-15T14:30:00+01:00", "2024-03-15T13:30:00Z", &cfg);
        assert_rfc("2024-03-15T14:30:00-05:30", "2024-03-15T20:00:00Z", &cfg);
        assert_rfc("2024-03-15 14:30 +0100", "2024-03-15T13:30:00Z", &cfg);
        assert_rfc("2024-03-15 14:30 -0530", "2024-03-15T20:00:00Z", &cfg);
    }

    #[test]
    fn week_and_ordinal_dates() {
        let cfg = def();
        assert_rfc("2024-W11-4", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("2024-074", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("2024074", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("2024-366", "2024-12-31T00:00:00Z", &cfg); // leap year
        // Compact week+weekday (week 11, Thursday).
        assert_rfc("2024W114", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("2024W15", "2024-04-08T00:00:00Z", &cfg);
        assert_err("2023-366", &cfg); // non-leap year
        assert_err("2024-000", &cfg);
        assert_err("2024-367", &cfg);
        assert_err("2024-W00", &cfg);
        assert_err("2024-W54", &cfg);
        assert_err("2024-W15-0", &cfg);
        assert_err("2024-W15-8", &cfg);
    }

    #[test]
    fn iso_week_no_weekday() {
        let cfg = def();
        assert_rfc("2024-W11", "2024-03-11T00:00:00Z", &cfg);
        assert_rfc("2024W11", "2024-03-11T00:00:00Z", &cfg);
    }

    #[test]
    fn calendar_bounds_and_leap_years() {
        let cfg = def();
        // Extreme but valid years.
        assert_rfc("0000-01-01", "0000-01-01T00:00:00Z", &cfg);
        assert_rfc("9999-12-31", "9999-12-31T00:00:00Z", &cfg);
        assert_rfc("-0001-01-01", "-0001-01-01T00:00:00Z", &cfg);
        assert_rfc("-2024-03-15", "-2024-03-15T00:00:00Z", &cfg);

        // Gregorian leap rules.
        assert_rfc("2024-02-29", "2024-02-29T00:00:00Z", &cfg);
        assert_rfc("2000-02-29", "2000-02-29T00:00:00Z", &cfg); // century leap
        assert_err("2023-02-29", &cfg);
        assert_err("1900-02-29", &cfg); // century non-leap
        assert_err("2100-02-29", &cfg);

        // Impossible civil dates.
        assert_err("2024-04-31", &cfg);
        assert_err("2024-00-15", &cfg);
        assert_err("2024-13-01", &cfg);
        assert_err("2024-03-00", &cfg);
        assert_err("2024-03-32", &cfg);
        assert_err("0000-00-00", &cfg);
    }

    #[test]
    fn time_component_bounds() {
        let cfg = def();
        assert_rfc(
            "2024-03-15 12:00:00.999999999",
            "2024-03-15T12:00:00.999999999Z",
            &cfg,
        );
        assert_err("2024-03-15 24:00:00", &cfg);
        assert_err("2024-03-15 25:00:00", &cfg);
        assert_err("2024-03-15 12:60:00", &cfg);
        // CHARACTERIZATION: second=60 outside a leap-second context is
        // accepted and clamped/snapped rather than hard-failing.
        let dt = parse("2024-03-15 12:00:60", &cfg);
        assert_eq!(dt.to_ymd().sec(), 59);
    }

    // ── 3. Ambiguous numeric order ─────────────────────────────────────────

    #[test]
    fn order_day_vs_month_vs_year_forced() {
        // 01/02/2003: day-first → 1 Feb; month-first → 2 Jan.
        assert_rfc("01/02/2003", "2003-02-01T00:00:00Z", &cfg_order(Order::Day));
        assert_rfc(
            "01/02/2003",
            "2003-01-02T00:00:00Z",
            &cfg_order(Order::Month),
        );
        // Year-forced cannot treat leading 01 as a year → fail.
        assert_err("01/02/2003", &cfg_order(Order::Year));

        // Unambiguous by magnitude.
        assert_rfc("13/01/2003", "2003-01-13T00:00:00Z", &cfg_order(Order::Day));
        assert_err("13/01/2003", &cfg_order(Order::Month));
        assert_rfc(
            "01/13/2003",
            "2003-01-13T00:00:00Z",
            &cfg_order(Order::Month),
        );
        assert_err("01/13/2003", &cfg_order(Order::Day));
    }

    #[test]
    fn order_smart_heuristic_signals() {
        let smart = cfg_order(Order::Smart);
        // First field 13–31 → day-first.
        assert_rfc("13/01/2003", "2003-01-13T00:00:00Z", &smart);
        // First 1–12 and second 13–31 → month-first.
        assert_rfc("01/13/2003", "2003-01-13T00:00:00Z", &smart);
        // Leading 4-digit year in modern range → year-first.
        assert_rfc("2024.03.15", "2024-03-15T00:00:00Z", &smart);
        // US-style dotted date (month.day.yy).
        assert_rfc("03.15.24", "2024-03-15T00:00:00Z", &smart);
        // Without locale, fully-ambiguous 2-digit triples prefer year-first
        // when components are all ≤12 (compact/tech convention in Smart path
        // after pure-numeric; delimited falls to day-first fallback — both
        // resolve 01/02/03 as 2001-02-03 here).
        assert_rfc("01/02/03", "2001-02-03T00:00:00Z", &smart);
        assert_rfc("05/06/07", "2005-06-07T00:00:00Z", &smart);
        // Single-digit components with hyphens are currently rejected.
        assert_err("3-4-5", &smart);
    }

    // ── 4. Pure-numeric modes ──────────────────────────────────────────────

    #[test]
    fn pure_numeric_years_and_compact_dates() {
        let auto = cfg_mode(Mode::Auto);
        let sci = cfg_mode(Mode::Scientific);
        let leg = cfg_mode(Mode::Legacy);

        // 2-digit year pivot (≤68 → 20xx).
        assert_rfc("24", "2024-01-01T00:00:00Z", &auto);
        // Scientific treats 1–4 digits as literal year.
        assert_rfc("24", "0024-01-01T00:00:00Z", &sci);
        assert_rfc("5", "0005-01-01T00:00:00Z", &sci);
        // Auto/Legacy: 1- and 3-digit years fall through to unix seconds.
        assert_rfc("5", "1970-01-01T00:00:05Z", &auto);
        assert_rfc("202", "1970-01-01T00:03:22Z", &auto);

        assert_rfc("2024", "2024-01-01T00:00:00Z", &auto);
        assert_rfc("240315", "2024-03-15T00:00:00Z", &auto);
        assert_rfc("20240315", "2024-03-15T00:00:00Z", &auto);
        // 6-digit with plausible YYYYMM year → first-of-month.
        assert_rfc("202403", "2024-03-01T00:00:00Z", &auto);
        assert_rfc("202403", "2024-03-01T00:00:00Z", &sci);
        assert_rfc("202403", "2024-03-01T00:00:00Z", &leg);
    }

    #[test]
    fn pure_numeric_ordinal_mjd_jd_by_mode() {
        let auto = cfg_mode(Mode::Auto);
        let sci = cfg_mode(Mode::Scientific);
        let leg = cfg_mode(Mode::Legacy);

        // 5-digit YYDDD ordinal (24-123 → 2024-05-02).
        assert_rfc("24123", "2024-05-02T00:00:00Z", &auto);
        assert_rfc("24123", "2024-05-02T00:00:00Z", &leg);

        // MJD 60400 → 2024-03-31 (Auto integer prefers ordinal when valid;
        // 60400 as YYDDD is invalid so MJD wins; Sci prefers MJD).
        assert_rfc("60400", "2024-03-31T00:00:00Z", &auto);
        assert_rfc("60400", "2024-03-31T00:00:00Z", &sci);
        // Legacy is ordinal-only; 60400 is not a valid YYDDD → unix fallback.
        assert_rfc("60400", "1970-01-01T16:46:40Z", &leg);

        // Fractional MJD.
        assert_rfc("60400.75", "2024-03-31T18:00:00Z", &auto);
        assert_rfc("60400.75", "2024-03-31T18:00:00Z", &sci);

        // 7-digit YYYYDDD ordinal.
        assert_rfc("2024123", "2024-05-02T00:00:00Z", &auto);
        assert_rfc("2024123", "2024-05-02T00:00:00Z", &leg);
        // Sci prefers JD for integer 7-digit (JD noon convention for integers).
        assert_rfc("2024123", "0829-10-05T00:00:00Z", &sci);

        // Famous JD of Unix epoch (fractional, so no +0.5 noon adjust).
        assert_rfc("2440587.5", "1970-01-01T00:00:00Z", &sci);
    }

    #[test]
    fn pure_numeric_unix_timestamps() {
        let auto = cfg_mode(Mode::Auto);
        let unix = cfg_mode(Mode::UnixTimestamp);

        // Seconds / ms / µs / ns of 2025-01-01 00:00:00 UTC.
        assert_rfc("1735689600", "2025-01-01T00:00:00Z", &auto);
        assert_rfc("1735689600", "2025-01-01T00:00:00Z", &unix);
        assert_rfc("1735689600123", "2025-01-01T00:00:00.123Z", &auto);
        assert_rfc("1735689600123456", "2025-01-01T00:00:00.123456Z", &auto);
        assert_rfc(
            "1735689600123456789",
            "2025-01-01T00:00:00.123456789Z",
            &auto,
        );
        assert_rfc("1735689600.5", "2025-01-01T00:00:00.5Z", &auto);

        // Signed unix seconds.
        assert_rfc("0", "1970-01-01T00:00:00Z", &unix);
        assert_rfc("-1", "1969-12-31T23:59:59Z", &unix);

        // Mode::UnixTimestamp forces short numbers into the unix path.
        assert_rfc("2024", "1970-01-01T00:33:44Z", &unix);
        assert_rfc("1000000000", "2001-09-09T01:46:40Z", &unix); // 1e9 sec
        assert_rfc("1000000000000", "2001-09-09T01:46:40Z", &unix); // 1e12 ms
    }

    // ── 5. Named English dates ─────────────────────────────────────────────

    #[test]
    fn named_english_and_ordinal_days() {
        let cfg = def();
        let cases = [
            ("15 March 2024", "2024-03-15T00:00:00Z"),
            ("March 15, 2024", "2024-03-15T00:00:00Z"),
            ("15th March 2024", "2024-03-15T00:00:00Z"),
            ("the 15th of March 2024", "2024-03-15T00:00:00Z"),
            ("15 of March 2024", "2024-03-15T00:00:00Z"),
            ("Mar 15 2024", "2024-03-15T00:00:00Z"),
            ("15-Mar-2024", "2024-03-15T00:00:00Z"),
            ("2024 March 15", "2024-03-15T00:00:00Z"),
            ("Thursday, March 14, 2024", "2024-03-14T00:00:00Z"),
            ("1st January 2000", "2000-01-01T00:00:00Z"),
            ("31st December 1999", "1999-12-31T00:00:00Z"),
            ("Feb 29 2024", "2024-02-29T00:00:00Z"),
            ("Sept 15 2024", "2024-09-15T00:00:00Z"),
            ("Sep 15 2024", "2024-09-15T00:00:00Z"),
            ("September 15th, 2024", "2024-09-15T00:00:00Z"),
            (
                "on the 5th of april 2024 at 00:00am",
                "2024-04-05T00:00:00Z",
            ),
        ];
        for (input, expected) in cases {
            assert_rfc(input, expected, &cfg);
        }
        assert_err("Feb 29 2023", &cfg);
    }

    /// Month + year with no day → first of that month.
    #[test]
    fn month_and_year_without_day() {
        let cfg = def();
        let cases = [
            ("2024-03", "2024-03-01T00:00:00Z"),
            ("2024/03", "2024-03-01T00:00:00Z"),
            ("2024/3", "2024-03-01T00:00:00Z"),
            ("202403", "2024-03-01T00:00:00Z"),
            // Month-first numeric (invoice / US style MM/YYYY).
            ("03/2024", "2024-03-01T00:00:00Z"),
            ("3/2024", "2024-03-01T00:00:00Z"),
            ("12/2024", "2024-12-01T00:00:00Z"),
            ("03-2024", "2024-03-01T00:00:00Z"),
            ("March 2024", "2024-03-01T00:00:00Z"),
            ("Mar 2024", "2024-03-01T00:00:00Z"),
            ("March, 2024", "2024-03-01T00:00:00Z"),
            ("2024 March", "2024-03-01T00:00:00Z"),
            ("2024 Mar", "2024-03-01T00:00:00Z"),
            ("january 2000", "2000-01-01T00:00:00Z"),
            ("Dec 1999", "1999-12-01T00:00:00Z"),
            ("Sept 2024", "2024-09-01T00:00:00Z"),
        ];
        for (input, expected) in cases {
            assert_rfc(input, expected, &cfg);
        }
        // Month alone is not enough.
        assert_err("March", &cfg);
        assert_err("Mar", &cfg);
        // Invalid month or ambiguous MM/YY (2-digit year).
        assert_err("13/2024", &cfg);
        assert_err("00/2024", &cfg);
        assert_err("03/24", &cfg);
    }

    #[test]
    fn twelve_hour_clock_and_at_glue() {
        let cfg = def();
        assert_rfc("14 Mar 2024 2:30 PM", "2024-03-14T14:30:00Z", &cfg);
        assert_rfc("14 Mar 2024 2:30PM", "2024-03-14T14:30:00Z", &cfg);
        assert_rfc("14 Mar 2024 at 2:30pm", "2024-03-14T14:30:00Z", &cfg);
    }

    /// Bare hour + meridian with no minutes (`2PM` → 14:00 on the same day).
    #[test]
    fn bare_hour_2pm() {
        let cfg = def();
        assert_rfc("14 Mar 2024 2PM", "2024-03-14T14:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 2pm", "2024-03-14T14:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 2 PM", "2024-03-14T14:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 2AM", "2024-03-14T02:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 12PM", "2024-03-14T12:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 12AM", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("March 14, 2024 2PM", "2024-03-14T14:00:00Z", &cfg);
        assert_rfc("2024-03-14 2PM", "2024-03-14T14:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 at 2PM", "2024-03-14T14:00:00Z", &cfg);
    }

    // /// Time component before the date (24h and 12h / AM/PM).
    // #[test]
    // fn time_before_date() {
    //     let cfg = def();

    //     // ── 24-hour clock, no meridiem ─────────────────────────────────────
    //     let h24 = [
    //         ("14:30 2024-03-15", "2024-03-15T14:30:00Z"),
    //         ("14:30:00 2024-03-15", "2024-03-15T14:30:00Z"),
    //         ("00:00 2024-03-15", "2024-03-15T00:00:00Z"),
    //         ("23:59:59 2024-03-15", "2024-03-15T23:59:59Z"),
    //         ("14:30 15/03/2024", "2024-03-15T14:30:00Z"),
    //         ("14:30 15 March 2024", "2024-03-15T14:30:00Z"),
    //         ("14:30 March 15, 2024", "2024-03-15T14:30:00Z"),
    //         ("14:30 15 Mar 2024", "2024-03-15T14:30:00Z"),
    //         ("9:05 2024-03-15", "2024-03-15T09:05:00Z"),
    //         ("09:05:00 15-03-2024", "2024-03-15T09:05:00Z"),
    //     ];
    //     for (input, expected) in h24 {
    //         assert_rfc(input, expected, &cfg);
    //     }

    //     // ── 12-hour clock with AM/PM ───────────────────────────────────────
    //     let h12 = [
    //         ("2:30 PM 2024-03-15", "2024-03-15T14:30:00Z"),
    //         ("2:30PM 2024-03-15", "2024-03-15T14:30:00Z"),
    //         ("2:30 pm 2024-03-15", "2024-03-15T14:30:00Z"),
    //         ("2:30PM 15/03/2024", "2024-03-15T14:30:00Z"),
    //         ("2:30 pm 15 March 2024", "2024-03-15T14:30:00Z"),
    //         ("2:30 PM March 15, 2024", "2024-03-15T14:30:00Z"),
    //         ("2:30PM 15 Mar 2024", "2024-03-15T14:30:00Z"),
    //         ("9:00 am 2024-03-15", "2024-03-15T09:00:00Z"),
    //         ("9:00am 2024-03-15", "2024-03-15T09:00:00Z"),
    //         ("12:00 am 2024-03-15", "2024-03-15T00:00:00Z"),
    //         ("12:00 pm 2024-03-15", "2024-03-15T12:00:00Z"),
    //         ("12:00AM 15 Mar 2024", "2024-03-15T00:00:00Z"),
    //         ("12:00PM 15 Mar 2024", "2024-03-15T12:00:00Z"),
    //         // Bare hour + meridiem before date
    //         ("2PM 2024-03-15", "2024-03-15T14:00:00Z"),
    //         ("2pm 2024-03-15", "2024-03-15T14:00:00Z"),
    //         ("2 PM 2024-03-15", "2024-03-15T14:00:00Z"),
    //         ("2AM 2024-03-15", "2024-03-15T02:00:00Z"),
    //         ("9am 15 March 2024", "2024-03-15T09:00:00Z"),
    //         ("12AM 2024-03-15", "2024-03-15T00:00:00Z"),
    //         ("12PM 2024-03-15", "2024-03-15T12:00:00Z"),
    //         ("2PM 15 Mar 2024", "2024-03-15T14:00:00Z"),
    //         ("2:30 PM on 15 Mar 2024", "2024-03-15T14:30:00Z"),
    //         ("at 2:30pm on March 15, 2024", "2024-03-15T14:30:00Z"),
    //     ];
    //     for (input, expected) in h12 {
    //         assert_rfc(input, expected, &cfg);
    //     }

    //     // Date-first controls still work (same instants, opposite order).
    //     assert_rfc("2024-03-15 14:30", "2024-03-15T14:30:00Z", &cfg);
    //     assert_rfc("15 March 2024 2:30 PM", "2024-03-15T14:30:00Z", &cfg);
    //     assert_rfc("15 Mar 2024 2PM", "2024-03-15T14:00:00Z", &cfg);
    // }

    // ── 6. Syslog / year-less dates ────────────────────────────────────────

    #[test]
    fn syslog_year_inference_from_ref_time() {
        // ref = 2025-01-15. Dates more than 2 days in the future snap to
        // the previous year (classic Dec-in-Jan syslog behaviour).
        let cfg = ref_cfg();
        assert_rfc("Mar  5 10:23:45", "2024-03-05T10:23:45Z", &cfg);
        assert_rfc("Dec 31 23:59:59", "2024-12-31T23:59:59Z", &cfg);
        assert_rfc("Jun 16 12:00:00", "2024-06-16T12:00:00Z", &cfg);
        // Jan 1 2025 is still "this year" relative to Jan 15.
        assert_rfc("Jan  1 00:00:00", "2025-01-01T00:00:00Z", &cfg);
    }

    // ── 7. Relative phrases & bare times ───────────────────────────────────

    #[test]
    fn relative_core_phrases() {
        let cfg = ref_cfg(); // Wed 2025-01-15 12:00
        let cases = [
            ("tomorrow", "2025-01-16T12:00:00Z"),
            ("yesterday", "2025-01-14T12:00:00Z"),
            ("today", "2025-01-15T12:00:00Z"),
            ("now", "2025-01-15T12:00:00Z"),
            ("next Monday", "2025-01-20T12:00:00Z"),
            ("last Friday", "2025-01-10T12:00:00Z"),
            ("this Monday", "2025-01-13T12:00:00Z"),
            ("coming Friday", "2025-01-17T12:00:00Z"),
            ("in 3 days", "2025-01-18T12:00:00Z"),
            ("2 weeks ago", "2025-01-01T12:00:00Z"),
            ("next Monday at 14:00", "2025-01-20T14:00:00Z"),
            ("tomorrow at 9am", "2025-01-16T09:00:00Z"),
            ("next year", "2026-01-15T12:00:00Z"),
            ("last year", "2024-01-15T12:00:00Z"),
            ("Monday next", "2025-01-20T12:00:00Z"),
            ("14:00 next Monday", "2025-01-20T14:00:00Z"),
            ("at 14:00", "2025-01-15T14:00:00Z"),
        ];
        for (input, expected) in cases {
            assert_rfc(input, expected, &cfg);
        }
    }

    #[test]
    fn relative_partial_and_surprising() {
        let cfg = ref_cfg();
        assert_rfc("the day after tomorrow", "2025-01-17T12:00:00Z", &cfg);
        assert_rfc("day after tomorrow", "2025-01-17T12:00:00Z", &cfg);
        assert_rfc("the day before yesterday", "2025-01-13T12:00:00Z", &cfg);
        assert_rfc("day before yesterday", "2025-01-13T12:00:00Z", &cfg);
        // Bare week/month (+ ignored "a"/"in") default to quantity 1.
        assert_rfc("in a week", "2025-01-22T12:00:00Z", &cfg);
        assert_rfc("a week", "2025-01-22T12:00:00Z", &cfg);
        assert_rfc("week", "2025-01-22T12:00:00Z", &cfg);
        assert_rfc("week ago", "2025-01-08T12:00:00Z", &cfg);
        assert_rfc("a month ago", "2024-12-15T12:00:00Z", &cfg);
        assert_rfc("in a month", "2025-02-15T12:00:00Z", &cfg);
        // CHARACTERIZATION: bare relative tokens alone act as "now".
        assert_rfc("next", "2025-01-15T12:00:00Z", &cfg);
        assert_rfc("ago", "2025-01-15T12:00:00Z", &cfg);

        assert_err("noon", &cfg);
        assert_err("midnight", &cfg);
        // Bare hour + meridiem (no colon) is TOD on the ref day.
        assert_rfc("9am", "2025-01-15T09:00:00Z", &cfg);
        assert_rfc("9 am", "2025-01-15T09:00:00Z", &cfg);
        assert_rfc("9pm", "2025-01-15T21:00:00Z", &cfg);
        assert_rfc("12am", "2025-01-15T00:00:00Z", &cfg);
        assert_rfc("12pm", "2025-01-15T12:00:00Z", &cfg);
    }

    #[test]
    fn bare_time_of_day_uses_ref_date() {
        let cfg = ref_cfg();
        assert_rfc("17:00", "2025-01-15T17:00:00Z", &cfg);
        assert_rfc("14:30", "2025-01-15T14:30:00Z", &cfg);
        assert_rfc("2:30 PM", "2025-01-15T14:30:00Z", &cfg);
        assert_rfc("9:00 am", "2025-01-15T09:00:00Z", &cfg);
        assert_rfc("00:00", "2025-01-15T00:00:00Z", &cfg);
        // Bare H:M:S is relative time-of-day (hour no longer left as a date field).
        assert_rfc("15:30:45", "2025-01-15T15:30:45Z", &cfg);
        assert_rfc("9:30:00", "2025-01-15T09:30:00Z", &cfg);
        assert_rfc("12:00", "2025-01-15T12:00:00Z", &cfg);
        assert_rfc("12:00:00", "2025-01-15T12:00:00Z", &cfg);
        assert_rfc("12:00 am", "2025-01-15T00:00:00Z", &cfg);
        assert_rfc("12:00 pm", "2025-01-15T12:00:00Z", &cfg);
        // Elapsed H:MM when civil parse_hms rejects the hour.
        assert_rfc("24:00", "2025-01-16T12:00:00Z", &cfg);
        assert_rfc("72:30", "2025-01-18T12:30:00Z", &cfg);
        assert_rfc("25:00:00", "2025-01-16T13:00:00Z", &cfg);
        assert_rfc("72:30 ago", "2025-01-12T11:30:00Z", &cfg);
    }

    #[test]
    fn relative_cfg_false_disables_relative_phrases() {
        let cfg = ParseCfg {
            relative: false,
            ref_time: Some(Dt::from_ymd(2025, 1, 15, Scale::UTC, 12, 0, 0, 0)),
            ..Default::default()
        };
        assert_err("tomorrow", &cfg);
        assert_err("in 3 days", &cfg);
        // Absolute dates still parse.
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 8. Scales & leap seconds ───────────────────────────────────────────

    #[test]
    fn scale_suffixes_and_leap_seconds() {
        let cfg = def();
        // Scale suffix is recognized; RFC3339 view is the civil form on that
        // scale projected through the usual conversion path.
        for s in [
            "2024-03-15T12:00:00 TAI",
            "2024-03-15T12:00:00 TT",
            "2024-03-15T12:00:00 UTC",
            "2024-03-15T12:00:00 GPS",
        ] {
            assert!(
                Dt::from_str_parse(s, &cfg).is_ok(),
                "scale suffix should parse: {s}"
            );
        }

        // Real positive leap-second instants.
        assert_rfc("2015-06-30T23:59:60", "2015-06-30T23:59:60Z", &cfg);
        assert_rfc("1972-06-30T23:59:60", "1972-06-30T23:59:60Z", &cfg);
        assert_rfc("2016-12-31T23:59:60", "2016-12-31T23:59:60Z", &cfg);
        assert_err("2015-06-30T23:59:61", &cfg);
        // CHARACTERIZATION: `23:59:60` is accepted even on days that are not
        // historical leap-second insertions (lenient leap-second slot).
        assert_rfc("2012-06-30T23:59:60", "2012-06-30T23:59:60Z", &cfg);
    }

    // ── 9. Separators, unicode, fullwidth ──────────────────────────────────

    #[test]
    fn separators_and_unicode_digits() {
        let cfg = def();
        // Leading/trailing ASCII whitespace is fine.
        assert_rfc("  2024-03-15  ", "2024-03-15T00:00:00Z", &cfg);
        // Unicode dashes used as separators.
        assert_rfc("2024\u{2010}03\u{2010}15", "2024-03-15T00:00:00Z", &cfg); // hyphen
        assert_rfc("2024\u{2013}03\u{2013}15", "2024-03-15T00:00:00Z", &cfg); // en-dash
        assert_rfc("2024\u{2014}03\u{2014}15", "2024-03-15T00:00:00Z", &cfg); // em-dash
        assert_rfc("2024_03_15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("2024,03,15", "2024-03-15T00:00:00Z", &cfg);
        // Fullwidth digits.
        assert_rfc("２０２４-０３-１５", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("１５/０３/２０２４", "2024-03-15T00:00:00Z", &cfg);
        // Japanese calendar units.
        assert_rfc("2024年3月15日", "2024-03-15T00:00:00Z", &cfg);
        // Zero-width space prefix is tolerated.
        assert_rfc("\u{200b}2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Tab between date and time is treated like a separator (normalised
        // to a hyphen in classify) and the time is still recovered.
        assert_rfc("2024-03-15\t14:30", "2024-03-15T14:30:00Z", &cfg);
    }

    // ── 10. Compact datetime layouts ───────────────────────────────────────

    #[test]
    fn compact_datetime_layouts() {
        let cfg = def();
        let cases = [
            ("20240315143045", "2024-03-15T14:30:45Z"),
            ("240315143045", "2024-03-15T14:30:45Z"),
            ("20240315 143045", "2024-03-15T14:30:45Z"),
            ("2024-03-15 1430", "2024-03-15T14:30:00Z"),
            ("20240315T14:30:45", "2024-03-15T14:30:45Z"),
            ("2024-03-15T143045", "2024-03-15T14:30:45Z"),
            ("2024031514:30:45", "2024-03-15T14:30:45Z"),
            ("2024-03", "2024-03-01T00:00:00Z"),
            ("202403", "2024-03-01T00:00:00Z"),
            ("2024/03", "2024-03-01T00:00:00Z"),
            ("03/2024", "2024-03-01T00:00:00Z"),
        ];
        for (input, expected) in cases {
            assert_rfc(input, expected, &cfg);
        }
    }

    // ── 11. HTTP / RFC 2822-ish ────────────────────────────────────────────

    #[test]
    fn http_and_rfc2822_style() {
        let cfg = def();
        assert_rfc(
            "Thu, 14 Mar 2024 15:30:45 GMT",
            "2024-03-14T15:30:45Z",
            &cfg,
        );
        assert_rfc(
            "Thu, 14 Mar 2024 15:30:45 +0000",
            "2024-03-14T15:30:45Z",
            &cfg,
        );
        assert_rfc("14 Mar 2024 15:30:45 GMT", "2024-03-14T15:30:45Z", &cfg);
    }

    // ── 12. Adversarial / lenient garbage ──────────────────────────────────

    #[test]
    fn adversarial_hard_rejects() {
        let cfg = ref_cfg();
        for s in [
            "not a date",
            "null",
            "NaN",
            "Infinity",
            "yes",
            "no",
            "true",
            "false",
            "----",
            "Mar",
            "March",
            "Monday",
            "1/2/3/4",
            "0x2024",
            "1e9",
            "123abc",
            "999999999999999999999999999999",
        ] {
            assert_err(s, &cfg);
        }
    }

    #[test]
    fn adversarial_lenient_accepts() {
        let cfg = ref_cfg();
        // ISO-ish time with leading T is treated as time-of-day on ref date.
        assert_rfc("T14:30:00", "2025-01-15T14:30:00Z", &cfg);
        // CHARACTERIZATION: trailing / leading non-date tokens are stripped
        // when a valid date remains.
        assert_rfc("2024-03-15T14:30:00Z garbage", "2024-03-15T14:30:00Z", &cfg);
        assert_rfc("garbage 2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Trailing T connector with no time still yields midnight.
        assert_rfc("2024-03-15T", "2024-03-15T00:00:00Z", &cfg);
        // Quotes / brackets / parens around a date are ignored.
        assert_rfc("\"2024-03-15\"", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("'2024-03-15'", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("[2024-03-15]", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("(2024-03-15)", "2024-03-15T00:00:00Z", &cfg);
        // CHARACTERIZATION: alphabetic noise + digits can fall through to a
        // pure-numeric unix-seconds path (abc123 → 123 seconds past epoch).
        assert_rfc("abc123", "1970-01-01T00:02:03Z", &cfg);
        // CHARACTERIZATION: scientific-notation-looking tokens can be
        // reinterpreted as date pieces (2.5e10 → 2010-05-02).
        assert_rfc("2.5e10", "2010-05-02T00:00:00Z", &cfg);
    }

    #[test]
    fn iana_zone_bracket_requires_tz_feature() {
        let cfg = def();
        // Without jiff-tz*, bracketed IANA zones are not applied / rejected.
        #[cfg(not(any(feature = "jiff-tz", feature = "jiff-tz-bundle")))]
        {
            assert_err("2024-03-15T14:30:00[Europe/Paris]", &cfg);
            assert_err("2024-03-15T14:30:00+01:00[Europe/Paris]", &cfg);
        }
        #[cfg(any(feature = "jiff-tz", feature = "jiff-tz-bundle"))]
        {
            // With TZ support the zoned form should at least parse.
            assert!(
                Dt::from_str_parse("2024-03-15T14:30:00+01:00[Europe/Paris]", &cfg).is_ok()
                    || Dt::from_str_parse("2024-03-15T14:30:00[Europe/Paris]", &cfg).is_ok()
            );
        }
    }

    // ── 13. ParseCfg knobs ─────────────────────────────────────────────────

    #[test]
    fn explicit_mode_only_tries_listed_formats() {
        let cfg = ParseCfg {
            mode: Mode::Explicit,
            parse: Some(vec!["%Y-%m-%d".into()]),
            ..Default::default()
        };
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Completely different layout is rejected (no Auto fallback).
        assert_err("15/03/2024", &cfg);
        assert_err("March 15, 2024", &cfg);
        // CHARACTERIZATION: `%Y-%m-%d` succeeds and the trailing time is
        // ignored / absorbed by the lower-level `from_str` path, so this
        // does *not* hard-fail under Explicit.
        assert_rfc("2024-03-15 12:00", "2024-03-15T00:00:00Z", &cfg);
    }

    #[test]
    fn explicit_formats_then_fallback_when_not_explicit_mode() {
        let cfg = ParseCfg {
            mode: Mode::Auto,
            parse: Some(vec!["%Y-%m-%d".into()]),
            ..Default::default()
        };
        // Listed format works.
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Fallback still handles other layouts.
        assert_rfc("15/03/2024", "2024-03-15T00:00:00Z", &cfg);
    }

    #[test]
    fn to_lower_false_requires_already_lowercase_names() {
        let cfg = ParseCfg {
            to_lower: false,
            ..Default::default()
        };
        // Numeric ISO is case-insensitive in practice.
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Title-case month names fail when to_lower is off.
        assert_err("March 15, 2024", &cfg);
        assert_rfc("march 15, 2024", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 14. Helper wrappers ────────────────────────────────────────────────

    #[test]
    fn str_to_helpers_agree_with_from_str_parse() {
        let cfg = def();
        let s = "2024-03-15T12:00:00Z";
        let dt = parse(s, &cfg);

        assert_eq!(Dt::str_to_attos(s, &cfg), Some(dt.to_attos()));
        assert_eq!(Dt::str_to_ms(s, &cfg), Some(dt.to_ms().0));
        assert_eq!(Dt::str_to_ns(s, &cfg), Some(dt.to_ns().0));
        assert_eq!(
            Dt::str_to_unix_ms(s, &cfg),
            Some(dt.to_scale_and_diff(Dt::UNIX_EPOCH, false).to_ms().0)
        );
        assert_eq!(
            Dt::str_to_unix_ns(s, &cfg),
            Some(dt.to_scale_and_diff(Dt::UNIX_EPOCH, false).to_ns().0)
        );

        assert_eq!(Dt::str_to_attos("not-a-date", &cfg), None);
        assert_eq!(Dt::str_to_unix_ms("not-a-date", &cfg), None);
    }

    // ── 15. Round-trip smoke ───────────────────────────────────────────────

    #[test]
    fn rfc3339_roundtrip_smoke() {
        let cfg = def();
        for s in [
            "2024-03-15T00:00:00Z",
            "2024-03-15T14:30:45.123456789Z",
            "1970-01-01T00:00:00Z",
            "2000-01-01T12:00:00Z",
            "-0001-06-15T00:00:00Z",
        ] {
            let dt = parse(s, &cfg);
            let again = dt.to_str_rfc3339_nf(9);
            let dt2 = parse(&again, &cfg);
            assert_eq!(dt.to_attos(), dt2.to_attos(), "roundtrip {s} → {again}");
        }
    }

    // ── 16. Smart order matrix on classic pitfalls ─────────────────────────

    #[test]
    fn smart_order_matrix_classic_pitfalls() {
        // Table-driven matrix: (input, Smart, Day, Month, Year) where None = Err.
        type Exp = Option<&'static str>;
        let rows: &[(&str, Exp, Exp, Exp, Exp)] = &[
            (
                "01/02/2003",
                Some("2003-02-01T00:00:00Z"), // Smart → day (1≤12, 2≤12 → fallback Day)
                Some("2003-02-01T00:00:00Z"),
                Some("2003-01-02T00:00:00Z"),
                None,
            ),
            (
                "02/01/2003",
                Some("2003-01-02T00:00:00Z"),
                Some("2003-01-02T00:00:00Z"),
                Some("2003-02-01T00:00:00Z"),
                None,
            ),
            (
                "12/11/10",
                Some("2012-11-10T00:00:00Z"),
                Some("2012-11-10T00:00:00Z"),
                Some("2012-11-10T00:00:00Z"),
                Some("2012-11-10T00:00:00Z"),
            ),
            (
                "15.03.24",
                Some("2015-03-24T00:00:00Z"),
                Some("2015-03-24T00:00:00Z"),
                Some("2015-03-24T00:00:00Z"),
                Some("2015-03-24T00:00:00Z"),
            ),
        ];

        for &(input, smart, day, month, year) in rows {
            for (order, exp) in [
                (Order::Smart, smart),
                (Order::Day, day),
                (Order::Month, month),
                (Order::Year, year),
            ] {
                let cfg = cfg_order(order);
                match exp {
                    Some(expected) => assert_rfc(input, expected, &cfg),
                    None => assert_err(input, &cfg),
                }
            }
        }
    }
}
