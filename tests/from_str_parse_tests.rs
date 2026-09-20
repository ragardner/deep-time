//! Red-team / characterization tests for [`Dt::from_str_parse`].
//!
//! Complements the combinatorial coverage in `date_tests.rs` and the relative
//! phrase suite in `date_relative_tests.rs` by probing:
//! - guardrails (empty / oversize / whitespace)
//! - calendar validity (leap years, ordinal/week bounds)
//! - ambiguous numeric order under Smart / Day / Month / Year
//! - pure-numeric unguided guess and `Numeric` pins
//! - named English dates, ordinals, 12-hour clock quirks
//! - syslog year inference
//! - relative phrases + bare times-of-day
//! - offsets, leap seconds, scale suffixes
//! - separators (unicode dashes, fullwidth digits, JP calendar units)
//! - adversarial / lenient garbage handling
//! - `ParseCfg` knobs (explicit formats, `assume_lowercase`, `relative`)
//! - `str_to_*` convenience helpers
//! - safety (digit-counter overflow at STRTIME_SIZE)
//! - Aho-Corasick substring false positives (month/relative inside words)
//! - relative+absolute digit glue, dual dates, 8-digit invalid civil→unix
//!
//! Several cases document *current* behavior that is surprising; those are
//! labeled `// CHARACTERIZATION` so they can be revisited deliberately.

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "parse")]
mod tests {
    use deep_time::{DateBase, Dt, DtErrKind, Numeric, Order, ParseCfg, ParseFmt, Scale, UnixUnit};

    // ── helpers ────────────────────────────────────────────────────────────

    fn default_cfg() -> ParseCfg {
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

    fn cfg_numeric(numeric: Numeric) -> ParseCfg {
        ParseCfg {
            numeric: Some(numeric),
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
        assert_rfc("2024-W11", "2024-03-11T00:00:00Z", &cfg);
        assert_rfc("2024W11", "2024-03-11T00:00:00Z", &cfg);
    }

    #[test]
    fn calendar_bounds_and_leap_years() {
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        // Prefer order, then the other two (same chains as Smart).
        assert_rfc("01/02/2003", "2003-02-01T00:00:00Z", &cfg_order(Order::Day));
        assert_rfc(
            "01/02/2003",
            "2003-01-02T00:00:00Z",
            &cfg_order(Order::Month),
        );
        assert_rfc(
            "01/02/2003",
            "2003-02-01T00:00:00Z",
            &cfg_order(Order::Year),
        ); // → day

        assert_rfc("13/01/2003", "2003-01-13T00:00:00Z", &cfg_order(Order::Day));
        assert_rfc(
            "13/01/2003",
            "2003-01-13T00:00:00Z",
            &cfg_order(Order::Month),
        ); // → day
        assert_rfc(
            "01/13/2003",
            "2003-01-13T00:00:00Z",
            &cfg_order(Order::Month),
        );
        assert_rfc("01/13/2003", "2003-01-13T00:00:00Z", &cfg_order(Order::Day)); // → month

        // 2-digit year triples.
        assert_rfc("15/03/24", "2024-03-15T00:00:00Z", &cfg_order(Order::Day));
        assert_rfc("15/03/24", "2024-03-15T00:00:00Z", &cfg_order(Order::Month)); // → day
        assert_rfc("15/03/24", "2015-03-24T00:00:00Z", &cfg_order(Order::Year));
        assert_rfc("01/02/03", "2003-02-01T00:00:00Z", &cfg_order(Order::Day));
        assert_rfc("01/02/03", "2003-01-02T00:00:00Z", &cfg_order(Order::Month));
        assert_rfc("01/02/03", "2001-02-03T00:00:00Z", &cfg_order(Order::Year));
    }

    #[test]
    fn order_smart_heuristic_signals() {
        let smart = cfg_order(Order::Smart);
        assert_rfc("13/01/2003", "2003-01-13T00:00:00Z", &smart); // first 13–31 → day
        assert_rfc("01/13/2003", "2003-01-13T00:00:00Z", &smart); // 1–12 then 13–31 → month
        assert_rfc("2024.03.15", "2024-03-15T00:00:00Z", &smart); // leading 4-digit year
        assert_rfc("03.15.24", "2024-03-15T00:00:00Z", &smart); // US m.d.yy
        assert_rfc("15/03/24", "2024-03-15T00:00:00Z", &smart); // EU d/m/yy
        assert_rfc("01/02/03", "2003-02-01T00:00:00Z", &smart); // all ≤12 → day fallback
        assert_rfc("05/06/07", "2007-06-05T00:00:00Z", &smart);
        assert_err("3-4-5", &smart);
    }

    // ── 4. Pure-numeric modes ──────────────────────────────────────────────

    #[test]
    fn pure_numeric_years_and_compact_dates() {
        let year_lit = cfg_numeric(Numeric::Year {
            two_digit_pivot: false,
        });
        let compact = cfg_numeric(Numeric::CompactYmd);

        // 2-digit year pivot (≤68 → 20xx).
        assert_rfc("24", "2024-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        // Literal year pin: digits are the year.
        assert_rfc("24", "0024-01-01T00:00:00Z", &year_lit);
        assert_rfc("5", "0005-01-01T00:00:00Z", &year_lit);
        // Unguided 1- and 3-digit are years, not unix.
        assert_rfc("5", "0005-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("202", "0202-01-01T00:00:00Z", &ParseCfg::DEFAULT);

        assert_rfc("2024", "2024-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("240315", "2024-03-15T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("20240315", "2024-03-15T00:00:00Z", &ParseCfg::DEFAULT);
        // 6-digit with plausible YYYYMM year → first-of-month.
        assert_rfc("202403", "2024-03-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("202403", "2024-03-01T00:00:00Z", &compact);
    }

    #[test]
    fn pure_numeric_ordinal_mjd_jd() {
        let mjd = cfg_numeric(Numeric::Mjd);
        let jd = cfg_numeric(Numeric::Jd);
        let ord = cfg_numeric(Numeric::Ordinal);

        // 5-digit YYDDD ordinal (24-123 → 2024-05-02).
        assert_rfc("24123", "2024-05-02T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("24123", "2024-05-02T00:00:00Z", &ord);

        // MJD 60400 → 2024-03-31 (unguided integer prefers ordinal when valid;
        // 60400 as YYDDD is invalid so MJD wins).
        assert_rfc("60400", "2024-03-31T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("60400", "2024-03-31T00:00:00Z", &mjd);
        // Ordinal pin: 60400 is not a valid YYDDD → error, not unix.
        assert_err("60400", &ord);

        // Fractional MJD.
        assert_rfc("60400.75", "2024-03-31T18:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("60400.75", "2024-03-31T18:00:00Z", &mjd);

        // 7-digit YYYYDDD ordinal.
        assert_rfc("2024123", "2024-05-02T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("2024123", "2024-05-02T00:00:00Z", &ord);
        // JD pin: integer 7-digit uses noon convention.
        assert_rfc("2024123", "0829-10-05T00:00:00Z", &jd);

        // Famous JD of Unix epoch (fractional, so no +0.5 noon adjust).
        assert_rfc("2440587.5", "1970-01-01T00:00:00Z", &jd);
        assert_rfc("2440587.5", "1970-01-01T00:00:00Z", &ParseCfg::DEFAULT);
    }

    #[test]
    fn date_serial_pin_only() {
        let d1900 = cfg_numeric(Numeric::DateSerial(DateBase::Epoch1900));
        let d1904 = cfg_numeric(Numeric::DateSerial(DateBase::Epoch1904));
        let d1899 = cfg_numeric(Numeric::DateSerial(DateBase::Epoch1899));

        // OOXML 1900 date base (fictitious leap day).
        assert_rfc("1", "1900-01-01T00:00:00Z", &d1900);
        assert_rfc("59", "1900-02-28T00:00:00Z", &d1900);
        assert_err("60", &d1900);
        assert_rfc("61", "1900-03-01T00:00:00Z", &d1900);
        assert_rfc("25569", "1970-01-01T00:00:00Z", &d1900);
        assert_rfc("25569.5", "1970-01-01T12:00:00Z", &d1900);
        assert_rfc("39448", "2008-01-01T00:00:00Z", &d1900);
        assert_rfc("40729", "2011-07-05T00:00:00Z", &d1900);
        assert_rfc("0", "1899-12-31T00:00:00Z", &d1900);
        assert_rfc("-1", "1899-12-30T00:00:00Z", &d1900);
        assert_rfc("-1.5", "1899-12-29T12:00:00Z", &d1900);

        // Unguided 5-digit is still MJD, not a spreadsheet serial.
        let unguided = parse("40729", &ParseCfg::DEFAULT);
        let pinned = parse("40729", &d1900);
        assert_ne!(unguided.to_str_rfc3339_nf(9), pinned.to_str_rfc3339_nf(9));

        assert_rfc("0", "1904-01-01T00:00:00Z", &d1904);
        assert_rfc("24107", "1970-01-01T00:00:00Z", &d1904);
        assert_rfc("-1", "1903-12-31T00:00:00Z", &d1904);

        // LibreOffice Calc default: serial 0 = 1899-12-30; serial 2 = 1900-01-01.
        assert_rfc("0", "1899-12-30T00:00:00Z", &d1899);
        assert_rfc("2", "1900-01-01T00:00:00Z", &d1899);
        assert_rfc("60", "1900-02-28T00:00:00Z", &d1899);
        assert_rfc("61", "1900-03-01T00:00:00Z", &d1899);
        assert_rfc("25569", "1970-01-01T00:00:00Z", &d1899);
        assert_rfc("-1", "1899-12-29T00:00:00Z", &d1899);
    }

    #[test]
    fn pure_numeric_unix_timestamps() {
        let unix_s = cfg_numeric(Numeric::Unix(UnixUnit::Seconds));
        let unix_ms = cfg_numeric(Numeric::Unix(UnixUnit::Millis));

        // Seconds / ms / µs / ns of 2025-01-01 00:00:00 UTC.
        assert_rfc("1735689600", "2025-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("1735689600", "2025-01-01T00:00:00Z", &unix_s);
        assert_rfc(
            "1735689600123",
            "2025-01-01T00:00:00.123Z",
            &ParseCfg::DEFAULT,
        );
        assert_rfc(
            "1735689600123456",
            "2025-01-01T00:00:00.123456Z",
            &ParseCfg::DEFAULT,
        );
        assert_rfc(
            "1735689600123456789",
            "2025-01-01T00:00:00.123456789Z",
            &ParseCfg::DEFAULT,
        );
        assert_rfc("1735689600.5", "2025-01-01T00:00:00.5Z", &ParseCfg::DEFAULT);

        // Signed unix seconds.
        assert_rfc("0", "1970-01-01T00:00:00Z", &unix_s);
        assert_rfc("-1", "1969-12-31T23:59:59Z", &unix_s);

        // Unix pin: every pure-numeric string is that unit, including short values.
        assert_rfc("2024", "1970-01-01T00:33:44Z", &unix_s);
        assert_rfc("3", "1970-01-01T00:00:00.003Z", &unix_ms);
        assert_rfc("1000000000", "2001-09-09T01:46:40Z", &unix_s);
        assert_rfc("1000000000000", "2001-09-09T01:46:40Z", &unix_ms);
    }

    // ── 5. Named English dates ─────────────────────────────────────────────

    #[test]
    fn named_english_and_ordinal_days() {
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
        assert_rfc("14 Mar 2024 2:30 PM", "2024-03-14T14:30:00Z", &cfg);
        assert_rfc("14 Mar 2024 2:30PM", "2024-03-14T14:30:00Z", &cfg);
        assert_rfc("14 Mar 2024 at 2:30pm", "2024-03-14T14:30:00Z", &cfg);
    }

    /// Bare hour + meridian with no minutes (`2PM` → 14:00 on the same day).
    #[test]
    fn bare_hour_2pm() {
        let cfg = default_cfg();
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
        assert_rfc("15:30:45", "2025-01-15T15:30:45Z", &cfg);
        assert_rfc("9:30:00", "2025-01-15T09:30:00Z", &cfg);
        assert_rfc("12:00", "2025-01-15T12:00:00Z", &cfg);
        assert_rfc("12:00:00", "2025-01-15T12:00:00Z", &cfg);
        assert_rfc("12:00 am", "2025-01-15T00:00:00Z", &cfg);
        assert_rfc("12:00 pm", "2025-01-15T12:00:00Z", &cfg);
        // Ops H:MM / H:MM:SS when hour is not a civil clock.
        assert_rfc("24:00", "2025-01-16T12:00:00Z", &cfg);
        assert_rfc("72:30", "2025-01-18T12:30:00Z", &cfg);
        assert_rfc("25:00:00", "2025-01-16T13:00:00Z", &cfg);
        assert_rfc("72:30 ago", "2025-01-12T11:30:00Z", &cfg);
    }

    #[test]
    fn invalid_clock_fields_do_not_glue_to_days() {
        let cfg = ref_cfg();
        // Invalid ops duration → Err (not digit-glue into multi-day offsets).
        assert_err("24:60", &cfg);
        assert_err("25:99", &cfg);
        assert_err("at 25:00", &cfg);
        assert_err("in 3 days at 25:00", &cfg);
        assert_err("tomorrow at 25:00", &cfg);
        assert_err("tomorrow at 99:00", &cfg);
        assert_rfc("in 3 days at 14:00", "2025-01-18T14:00:00Z", &cfg);
        assert_rfc("at 14:00", "2025-01-15T14:00:00Z", &cfg);
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        // Prefix stripped; remaining 3 digits are an unguided year.
        assert_rfc("abc123", "0123-01-01T00:00:00Z", &cfg);
        // CHARACTERIZATION: scientific-notation-looking tokens can be
        // reinterpreted as date pieces (2.5e10 → 2010-05-02).
        assert_rfc("2.5e10", "2010-05-02T00:00:00Z", &cfg);
    }

    #[test]
    fn iana_zone_bracket_requires_tz_feature() {
        let cfg = default_cfg();
        // Without jiff-tz*, bracketed real IANA zones are rejected (parser/zone list)
        #[cfg(not(any(feature = "jiff-tz", feature = "jiff-tz-bundle")))]
        {
            assert_err("2024-03-15T14:30:00[Europe/Paris]", &cfg);
            assert_err("2024-03-15T14:30:00+01:00[Europe/Paris]", &cfg);
            // UTC aliases remain accepted without jiff-tz
            for s in [
                "2024-03-15T14:30:00[UTC]",
                "2024-03-15T14:30:00[Zulu]",
                "2024-03-15T14:30:00[Etc/UTC]",
            ] {
                Dt::from_str_parse(s, &cfg)
                    .unwrap_or_else(|e| panic!("{s} should succeed without jiff-tz: {e}"));
            }
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
            fmt: ParseFmt::Only(vec!["%Y-%m-%d".into()]),
            ..Default::default()
        };
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Completely different layout is rejected (no Auto fallback).
        assert_err("15/03/2024", &cfg);
        assert_err("March 15, 2024", &cfg);
        // Trailing time is leftover input; Only does not ignore it.
        assert_err("2024-03-15 12:00", &cfg);
    }

    #[test]
    fn explicit_formats_then_fallback_when_not_explicit_mode() {
        let cfg = ParseCfg {
            fmt: ParseFmt::Prefer(vec!["%Y-%m-%d".into()]),
            ..Default::default()
        };
        // Listed format works.
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Fallback still handles other layouts.
        assert_rfc("15/03/2024", "2024-03-15T00:00:00Z", &cfg);
    }

    #[test]
    fn assume_lowercase_requires_already_lowercase_names() {
        let cfg = ParseCfg {
            assume_lowercase: true,
            ..Default::default()
        };
        // Numeric ISO is case-insensitive in practice.
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Title-case month names fail when lowercasing is skipped.
        assert_err("March 15, 2024", &cfg);
        assert_rfc("march 15, 2024", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 14. Helper wrappers ────────────────────────────────────────────────

    #[test]
    fn str_to_helpers_agree_with_from_str_parse() {
        let cfg = default_cfg();
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
        let cfg = default_cfg();
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
        // input → Smart, Day, Month, Year (prefer + fallback).
        let rows: &[(&str, &str, &str, &str, &str)] = &[
            (
                "01/02/2003",
                "2003-02-01T00:00:00Z",
                "2003-02-01T00:00:00Z",
                "2003-01-02T00:00:00Z",
                "2003-02-01T00:00:00Z",
            ),
            (
                "02/01/2003",
                "2003-01-02T00:00:00Z",
                "2003-01-02T00:00:00Z",
                "2003-02-01T00:00:00Z",
                "2003-01-02T00:00:00Z",
            ),
            (
                "12/11/10",
                "2010-11-12T00:00:00Z",
                "2010-11-12T00:00:00Z",
                "2010-12-11T00:00:00Z",
                "2012-11-10T00:00:00Z",
            ),
            (
                "15.03.24",
                "2024-03-15T00:00:00Z",
                "2024-03-15T00:00:00Z",
                "2024-03-15T00:00:00Z",
                "2015-03-24T00:00:00Z",
            ),
        ];

        for &(input, smart, day, month, year) in rows {
            for (order, expected) in [
                (Order::Smart, smart),
                (Order::Day, day),
                (Order::Month, month),
                (Order::Year, year),
            ] {
                assert_rfc(input, expected, &cfg_order(order));
            }
        }
    }

    // ── 17. Safety: digit-counter overflow (STRTIME_SIZE allows ≤512 digits) ─

    /// Regression: `num_digits: u8` used to overflow and panic in debug at 256
    /// digits (and wrap in release). Must never panic for inputs ≤ STRTIME_SIZE.
    #[test]
    fn digit_counter_no_panic_at_strtime_limit() {
        let cfg = default_cfg();
        for n in [255usize, 256, 300, 400, 512] {
            let s = "1".repeat(n);
            // Must not panic; Ok or Err are both acceptable outcomes.
            let _ = Dt::from_str_parse(&s, &cfg);
        }
        // Slash-separated digit groups that also push digit count past 255.
        let heavy = format!("1{}", "/2".repeat(200));
        assert!(heavy.len() <= 512, "fixture must fit STRTIME_SIZE");
        let _ = Dt::from_str_parse(&heavy, &cfg);
        // Oversize still rejected by the length guard (no classify path).
        assert_err_kind(&"1".repeat(513), &cfg, DtErrKind::InvalidLen);
    }

    // ── 18. Dictionary hits require alphabetic standalone neighbors ────────

    /// Mid-word dictionary hits are ignored (same rule as the old am/pm guard).
    /// Digit/punct neighbors stay allowed (`14Mar2024`, `2pm`).
    #[test]
    fn substring_token_false_positives_rejected() {
        let cfg = ref_cfg();

        // Mid-word month ignored; remaining absolute date can still parse.
        assert_rfc("junk 2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("decorate 2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Not April/May/August — only the year survives.
        assert_rfc("aprilfool 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_rfc("mayhem 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_rfc("augustine 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_err("marching 15 2024", &cfg);

        assert_err("knowledge", &cfg);
        assert_err("snow", &cfg);
        assert_err("unknown", &cfg);
        assert_err("wagon", &cfg);
        // Not tomorrow/sun/mon — year only once mid-word hit is dropped.
        assert_rfc("tomorrowland 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_rfc("sunflower 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_rfc("monster 2024", "2024-01-01T00:00:00Z", &cfg);
        assert_err("saturn 15 2024", &cfg);

        assert_err("america", &cfg);
        assert_rfc("12am", "2025-01-15T00:00:00Z", &cfg);
        assert_rfc("14 Mar 2024 2pm", "2024-03-14T14:00:00Z", &cfg);

        assert_rfc("jun 2024", "2024-06-01T00:00:00Z", &cfg);
        assert_rfc("June 2024", "2024-06-01T00:00:00Z", &cfg);
        assert_rfc("Sept 15 2024", "2024-09-15T00:00:00Z", &cfg);
        assert_rfc("Sept 2024", "2024-09-01T00:00:00Z", &cfg);
        assert_rfc("tomorrow", "2025-01-16T12:00:00Z", &cfg);
        assert_rfc("coming Friday", "2025-01-17T12:00:00Z", &cfg);
        assert_rfc("14Mar2024", "2024-03-14T00:00:00Z", &cfg);
        assert_rfc("15-Mar-2024", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 19. Relative parser: glued absolute dates → multi-millennium offsets ─

    /// Leftover bare numbers default to **days**. Hyphenated ISO dates glue to
    /// a single huge day count (`2024-03-15` → 20_240_315 days).
    #[test]
    fn relative_plus_absolute_date_blows_up_years() {
        let cfg = ref_cfg(); // 2025-01-15
        // tomorrow + 20240315 days ≈ year 57441.
        assert_rfc("tomorrow 2024-03-15", "57441-02-22T12:00:00Z", &cfg);
        assert_rfc("in 3 days 2024-03-15", "57441-02-24T12:00:00Z", &cfg);
        assert_rfc("next Monday 2024-03-15", "57441-02-26T12:00:00Z", &cfg);
        // "now" inside "noway" is not standalone → no relative early-out;
        // remaining digits parse as an absolute date (or fail). Mid-word
        // relative no longer produces the multi-millennium offset.
        assert_rfc("noway 2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Absolute-first still wins (relative tail ignored / stripped).
        assert_rfc("2024-03-15 tomorrow", "2024-03-15T00:00:00Z", &cfg);
        // Relative + civil clock is fine.
        assert_rfc("tomorrow 15:00", "2025-01-16T15:00:00Z", &cfg);
        // Bare "N days" is intentional relative duration.
        assert_rfc("3 days", "2025-01-18T12:00:00Z", &cfg);
    }

    // ── 20. Dual absolute dates / second date eaten as time ────────────────

    #[test]
    fn dual_absolute_dates_partially_reinterpreted() {
        let cfg = default_cfg();
        // Second ISO date partially consumed as time fields.
        // CHARACTERIZATION: not an error.
        assert_rfc("2024-03-15 2025-04-16", "2024-03-16T12:25:00Z", &cfg);
        assert_rfc("15/03/2024 16/04/2025", "2024-03-15T16:00:00Z", &cfg);
        // Two named months is hard-rejected.
        assert_err("March 15 2024 March 16 2025", &cfg);
    }

    // ── 21. Invalid compact forms ──────────────────────────────────────────

    #[test]
    fn invalid_eight_digit_civil_is_unix_seconds() {
        let cfg = default_cfg();
        // 8-digit: valid YYYYMMDD first, else unix seconds (1970–1973).
        assert_rfc("20241315", "1970-08-23T06:35:15Z", &cfg); // month 13
        assert_rfc("20240230", "1970-08-23T06:17:10Z", &cfg); // Feb 30
        assert_rfc("99999999", "1973-03-03T09:46:39Z", &cfg);
        // Compact datetime with hour 24 → leftover unix ms, not civil reject.
        assert_rfc("20240315249999", "2611-05-23T21:47:29.999Z", &cfg);
        assert_rfc("20240315243000", "2611-05-23T21:47:23Z", &cfg);
        // 6-digit invalid YYMMDD is not unix.
        assert_err("123456", &cfg);
        assert_err("991332", &cfg);
    }

    // ── 22. ParseFmt::Only with empty / missing format list ─────────────

    #[test]
    fn only_empty_format_list_fails() {
        let empty = ParseCfg {
            fmt: ParseFmt::Only(vec![]),
            ..Default::default()
        };
        assert_err("15/03/2024", &empty);

        // Guess (default) still parses.
        assert_rfc("15/03/2024", "2024-03-15T00:00:00Z", &default_cfg());

        // Non-empty list that cannot match → hard fail (no guess fallback).
        let ymd_only = ParseCfg {
            fmt: ParseFmt::Only(vec!["%Y-%m-%d".into()]),
            ..Default::default()
        };
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &ymd_only);
        assert_err("15/03/2024", &ymd_only);
    }

    /// Explicit `%Y` / `%Y-%m` must default missing month/day to 1.
    ///
    /// Regression: the `parse` path used `allow_partial_date = false`, so
    /// year-only formats failed with `Incomplete` even though generated
    /// candidates (and the intended Explicit UX) allow partial dates.
    #[test]
    fn explicit_partial_date_formats_default_missing_fields() {
        let only_year = ParseCfg {
            fmt: ParseFmt::Only(vec!["%Y".into()]),
            ..Default::default()
        };
        assert_rfc("2024", "2024-01-01T00:00:00Z", &only_year);
        assert_rfc("0001", "0001-01-01T00:00:00Z", &only_year);
        assert_rfc("9999", "9999-01-01T00:00:00Z", &only_year);
        // Leftover after `%Y` is not a year-only string.
        assert_err("2024-03-15", &only_year);

        let year_month = ParseCfg {
            fmt: ParseFmt::Only(vec!["%Y-%m".into()]),
            ..Default::default()
        };
        assert_rfc("2024-03", "2024-03-01T00:00:00Z", &year_month);
        assert_rfc("2024-12", "2024-12-01T00:00:00Z", &year_month);
        assert_err("15/03/2024", &year_month);

        let two_digit = ParseCfg {
            fmt: ParseFmt::Only(vec!["%y".into()]),
            ..Default::default()
        };
        assert_rfc("24", "2024-01-01T00:00:00Z", &two_digit);
        assert_rfc("99", "1999-01-01T00:00:00Z", &two_digit);

        // Multiple formats: first match wins; partial year still usable.
        let multi = ParseCfg {
            fmt: ParseFmt::Only(vec!["%d/%m/%Y".into(), "%Y".into()]),
            ..Default::default()
        };
        assert_rfc("15/03/2024", "2024-03-15T00:00:00Z", &multi);
        assert_rfc("2024", "2024-01-01T00:00:00Z", &multi);
    }

    // ── 23. Garbage embedding / prefix stripping ───────────────────────────

    #[test]
    fn embedded_date_in_noise_is_accepted() {
        let cfg = default_cfg();
        // CHARACTERIZATION: surrounding non-date text is often ignored.
        // Do **not** use from_str_parse as a strict validator for untrusted input.
        assert_rfc("2024-03-15'; DROP TABLE", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("../../../2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("{\"date\":\"2024-03-15\"}", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("<time>2024-03-15</time>", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("\"2024-03-15\"", "2024-03-15T00:00:00Z", &cfg);
        // Trailing alphabetic glued to date is also stripped.
        assert_rfc("2024-03-15x", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("x2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // Trailing Z without time is accepted (Zulu-ish).
        assert_rfc("2024-03-15Z", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("2024-03-15 junk", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("garbage 2024-03-15", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 24. Offset extremes, 12h bounds, leap-second clamp ─────────────────

    #[test]
    fn offset_and_clock_edge_cases() {
        let cfg = default_cfg();
        // Offsets well outside civil TZ ranges still apply.
        assert_rfc("2024-03-15T12:00:00+15:00", "2024-03-14T21:00:00Z", &cfg);
        assert_rfc("2024-03-15T12:00:00-13:00", "2024-03-16T01:00:00Z", &cfg);
        assert_rfc("2024-03-15T12:00:00+23:59", "2024-03-14T12:01:00Z", &cfg);
        assert_err("2024-03-15T12:00:00+99:00", &cfg);
        // Hour-only offset (no minutes).
        assert_rfc("2024-03-15T12:00:00+01", "2024-03-15T11:00:00Z", &cfg);

        // Invalid 12-hour hours hard-fail.
        assert_err("2024-03-15 0am", &cfg);
        assert_err("2024-03-15 13pm", &cfg);
        assert_err("2024-03-15 24am", &cfg);
        // 00:00am is accepted as midnight.
        assert_rfc("2024-03-15 00:00am", "2024-03-15T00:00:00Z", &cfg);

        // ISO 24:00 end-of-day is rejected (unlike bare ops "24:00" relative).
        assert_err("2024-03-15T24:00:00Z", &cfg);
    }

    #[test]
    fn leap_second_slot_only_at_day_boundary_on_leap_days() {
        let cfg = default_cfg();
        // Real leap-second insertion days keep second=60.
        assert_rfc("2015-06-30T23:59:60Z", "2015-06-30T23:59:60Z", &cfg);
        assert_rfc("2012-06-30T23:59:60Z", "2012-06-30T23:59:60Z", &cfg);
        // Non-leap days clamp 23:59:60 → 23:59:59 (not hard-fail).
        assert_rfc("2024-01-01T23:59:60Z", "2024-01-01T23:59:59Z", &cfg);
        assert_rfc("2024-03-15T23:59:60Z", "2024-03-15T23:59:59Z", &cfg);
        // Mid-day :60 always clamps (never a leap-second site).
        assert_rfc("2024-03-15T12:00:60Z", "2024-03-15T12:00:59Z", &cfg);
    }

    // ── 25. Unicode / control tolerance ────────────────────────────────────

    #[test]
    fn control_and_bidi_prefix_tolerance() {
        let cfg = default_cfg();
        // Null / CR-LF / BOM / bidi marks do not block a trailing date.
        assert_rfc("2024-03-15\0", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("2024-03-15\r\n", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("\u{feff}2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("2024\u{202e}03-15", "2024-03-15T00:00:00Z", &cfg);
        assert_rfc("\u{200f}2024-03-15", "2024-03-15T00:00:00Z", &cfg);
        // NBSP and em-space act as separators.
        assert_rfc("2024-03-15\u{00a0}12:00", "2024-03-15T12:00:00Z", &cfg);
        assert_rfc("2024\u{2003}03\u{2003}15", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 26. Pure-numeric unit mis-detection edges ──────────────────────────

    #[test]
    fn pure_numeric_digit_length_unit_edges() {
        // 9-digit unguided is unix seconds.
        assert_rfc("202403151", "1976-05-31T15:05:51Z", &ParseCfg::DEFAULT);
        // 10-digit unix seconds (1e10) → far future.
        assert_rfc("10000000000", "2286-11-20T17:46:40Z", &ParseCfg::DEFAULT);
        // 11-digit still treated as seconds (not ms).
        assert_rfc("17356896000", "2520-01-08T00:00:00Z", &ParseCfg::DEFAULT);
        // 12-digit → milliseconds path.
        assert_rfc("100000000000", "1973-03-03T09:46:40Z", &ParseCfg::DEFAULT);
        // 19-digit → nanoseconds path (i64-max-ish).
        assert_rfc(
            "9223372036854775807",
            "2262-04-11T23:47:16.854775807Z",
            &ParseCfg::DEFAULT,
        );
        // 2-digit year pivot boundary.
        assert_rfc("68", "2068-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("69", "1969-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("00", "2000-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        // Signed year / date.
        assert_rfc("+2024", "2024-01-01T00:00:00Z", &ParseCfg::DEFAULT);
        assert_rfc("+2024-03-15", "2024-03-15T00:00:00Z", &ParseCfg::DEFAULT);
        // Very large negative pure-numeric still yields a Dt (far past).
        let neg = format!("-{}", "9".repeat(25));
        let dt = parse(&neg, &ParseCfg::DEFAULT);
        assert!(dt.to_ymd().yr() < -9999);
    }

    // ── 27. relative=false disables bare TOD as well as phrases ────────────

    #[test]
    fn relative_false_also_blocks_bare_tod() {
        let cfg = ParseCfg {
            relative: false,
            ref_time: Some(Dt::from_ymd(2025, 1, 15, Scale::UTC, 12, 0, 0, 0)),
            ..Default::default()
        };
        assert_err("tomorrow", &cfg);
        assert_err("in 3 days", &cfg);
        assert_err("14:00", &cfg);
        assert_err("9am", &cfg);
        assert_err("week", &cfg);
        assert_rfc("2024-03-15", "2024-03-15T00:00:00Z", &cfg);
    }

    // ── 28. Display form (`Dt` Display / `.to_string()`) ───────────────────

    #[test]
    fn display_form_roundtrip_via_from_str_parse() {
        let cfg = default_cfg();
        let cases = [
            Dt::ZERO,
            Dt::new(
                86400 * deep_time::consts::ATTOS_PER_SEC_I128,
                Scale::TAI,
                Scale::UTC,
            ),
            Dt::new(-1_500_000_000_000_000_000, Scale::TT, Scale::GPS),
            Dt::new(1, Scale::TDB, Scale::TDB),
            Dt::MAX,
            Dt::MIN,
        ];
        for original in cases {
            let s = original.to_string();
            let back = parse(&s, &cfg);
            assert_eq!(back.attos, original.attos, "attos for {s:?}");
            assert_eq!(back.scale, original.scale, "scale for {s:?}");
            assert_eq!(back.target, original.target, "target for {s:?}");
        }
    }

    #[test]
    fn display_form_agrees_with_dt_from_str() {
        let cfg = default_cfg();
        for s in [
            "[0s TAI>TAI]",
            "[86400s TAI>UTC]",
            "[-1.5s TT>GPS]",
            "  [0s TAI>TAI]",
            "junk [42s GPS>UTC] trailing",
        ] {
            let via_parse = parse(s, &cfg);
            let via_fast = Dt::from_str(s).unwrap_or_else(|e| {
                panic!("Dt::from_str failed for {s:?}: {e}");
            });
            assert_eq!(via_parse, via_fast, "mismatch for {s:?}");
        }
    }

    #[test]
    fn display_form_not_confused_with_civil_brackets() {
        let cfg = default_cfg();
        // Normal civil / Zulu still works.
        assert_rfc("2024-03-15T12:00:00Z", "2024-03-15T12:00:00Z", &cfg);

        // Display form keeps the encoded scale/target (no TAI conversion).
        let disp = parse("[-1.5s TT>GPS]", &cfg);
        assert_eq!(disp.scale, Scale::TT);
        assert_eq!(disp.target, Scale::GPS);
        assert_eq!(disp.attos, -1_500_000_000_000_000_000);

        // `[` + civil date is not Display: failed handoff falls through to the
        // smart path, which treats the brackets as junk around a real date.
        let civil = parse("[2024-03-15]", &cfg);
        assert_eq!(civil.scale, Scale::TAI);
        assert_eq!(civil.target, Scale::UTC);
        assert_eq!(civil.to_ymd().yr(), 2024);
        assert_eq!(civil.to_ymd().mo(), 3);
        assert_eq!(civil.to_ymd().day(), 15);
    }
}
