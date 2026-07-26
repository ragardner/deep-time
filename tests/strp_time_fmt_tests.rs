//! Red-team / guarantee tests for [`StrPTimeFmt`].
//!
//! # Contract under test
//!
//! `StrPTimeFmt` is a **pre-validated parse format**. Construction
//! (`StrPTimeFmt::new` / `Dt::parse_fmt`) either:
//!
//! 1. rejects the format with a format-structure error, or
//! 2. returns a value that may be reused for **parsing** without the format
//!    itself being the failure reason.
//!
//! ## What is guaranteed
//!
//! After `new` succeeds, `to_dt` (and the parse half of `to_str` / `to_str_b`)
//! must **not** fail with format-structure kinds:
//!
//! - [`DtErrKind::UnknownItem`]
//! - [`DtErrKind::UnsupportedItem`]
//! - [`DtErrKind::TruncatedDirective`]
//! - [`DtErrKind::InvalidFractional`] (only emitted for broken `%.…` *format*
//!   syntax; never for bad input once the format is validated)
//! - [`DtErrKind::InvalidBytes`] (stored format is ASCII → always valid UTF-8)
//!
//! Parsing may still fail for **input** reasons (mismatch, incomplete date,
//! out-of-range fields, trailing characters, missing fractional digits,
//! bad offset, etc.).
//!
//! ## What is *not* guaranteed
//!
//! - **`output_fmt` on `to_str` / `to_str_b` is not pre-validated.** A broken
//!   output format can still yield `TruncatedDirective` / `UnknownItem` / etc.
//! - **Printer-only syntax** (`%.~f`, `%q`, multi-flag `~`) is outside this
//!   type’s surface; rejection at `new` is expected.
//! - **Success of parsing** is not guaranteed — only that failure is not due
//!   to the validated format being unusable.
//! - Error *kind* parity for construction rejects vs. live parser rejects
//!   (`UnexpectedEnd` vs `TruncatedDirective` on `%0`) is not part of the
//!   contract; both sides reject the format.

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, DtErrKind, Lang, Scale, StrPTimeFmt};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Error kinds that mean “the format string is broken”, not “the input failed”.
///
/// After a successful `StrPTimeFmt::new`, none of these may appear from
/// `to_dt` for the validated format (or from the parse half of `to_str*`).
///
/// Note: `UnexpectedEnd` / `ExpectedFractional` are **not** listed — they are
/// dual-use (construction *and* input). Once a format is validated, those kinds
/// only arise from input exhaustion / missing fields.
fn is_format_structure_kind(k: DtErrKind) -> bool {
    matches!(
        k,
        DtErrKind::UnknownItem
            | DtErrKind::UnsupportedItem
            | DtErrKind::TruncatedDirective
            | DtErrKind::InvalidFractional
            | DtErrKind::InvalidBytes
    )
}

/// Assert construction succeeds and that subsequent parse attempts never
/// surface a format-structure error.
fn assert_validated_never_format_errors(fmt_str: &str) {
    let fmt = StrPTimeFmt::new(fmt_str)
        .unwrap_or_else(|e| panic!("expected format to validate: {fmt_str:?} → {:?}", e.kind()));

    // Hostile / degenerate inputs — failures must be input-side only.
    let hostile: &[&str] = &[
        "",
        " ",
        "\t\n",
        "x",
        "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        "%%%%",
        "9999-99-99 99:99:99",
        "not-a-date",
        "2025",
        "2025-04-16",
        "2025-04-16T14:30:45",
        "2025-04-16 14:30:45.123456789",
        "+0000",
        "UTC",
        "Monday",
        "\0",
        "\u{7f}",
    ];

    for input in hostile {
        match fmt.to_dt(input, false, false, false) {
            Ok(_) => {}
            Err(e) => {
                let k = e.kind();
                assert!(
                    !is_format_structure_kind(k),
                    "format-structure error after successful new()\n  fmt={fmt_str:?}\n  input={input:?}\n  kind={k:?}"
                );
            }
        }
        // Lenient flags must not resurrect format errors either.
        for (a, b, c) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, true, true),
        ] {
            if let Err(e) = fmt.to_dt(input, a, b, c) {
                let k = e.kind();
                assert!(
                    !is_format_structure_kind(k),
                    "format-structure error with flags ({a},{b},{c})\n  fmt={fmt_str:?}\n  input={input:?}\n  kind={k:?}"
                );
            }
        }
    }
}

/// Every parse-side directive letter currently accepted by `validate_format`.
const SUPPORTED_DIRECTIVES: &[u8] = b"%AaBbCDdFfgGHhjIkJlLmMnNpPQRSsTtUuVWwYyz*";

// ─────────────────────────────────────────────────────────────────────────────
// Construction: rejection surface
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn rejects_truncated_percent() {
    assert_eq!(
        StrPTimeFmt::new("%").unwrap_err().kind(),
        DtErrKind::TruncatedDirective
    );
    assert_eq!(
        StrPTimeFmt::new("%Y%").unwrap_err().kind(),
        DtErrKind::TruncatedDirective
    );
    assert_eq!(
        StrPTimeFmt::new("end%").unwrap_err().kind(),
        DtErrKind::TruncatedDirective
    );
}

#[test]
fn rejects_extensions_without_directive() {
    // Flag / width / colons with nothing after → UnexpectedEnd
    for s in [
        "%0", "%-", "%_", "%^", "%#", "%:", "%::", "%:::", "%12", "%0:",
    ] {
        let err = StrPTimeFmt::new(s).unwrap_err();
        assert_eq!(
            err.kind(),
            DtErrKind::UnexpectedEnd,
            "expected UnexpectedEnd for {s:?}, got {:?}",
            err.kind()
        );
    }
}

#[test]
fn rejects_malformed_fractional_directive() {
    assert_eq!(
        StrPTimeFmt::new("%.").unwrap_err().kind(),
        DtErrKind::ExpectedFractional
    );
    assert_eq!(
        StrPTimeFmt::new("%.3").unwrap_err().kind(),
        DtErrKind::ExpectedFractional
    );
    for s in ["%.x", "%.3x", "%.~f", "%.9~f", "%.S", "%.-f"] {
        assert_eq!(
            StrPTimeFmt::new(s).unwrap_err().kind(),
            DtErrKind::InvalidFractional,
            "expected InvalidFractional for {s:?}"
        );
    }
}

#[test]
fn rejects_explicitly_unsupported_items() {
    for d in [b'c', b'r', b'x', b'X', b'Z'] {
        let s = format!("%{}", d as char);
        assert_eq!(
            StrPTimeFmt::new(&s).unwrap_err().kind(),
            DtErrKind::UnsupportedItem,
            "expected UnsupportedItem for {s}"
        );
    }
}

#[test]
fn rejects_unknown_directive_letters() {
    // Sample of letters that are neither supported nor in the unsupported set.
    for d in b"EioOqv@" {
        let s = format!("%{}", *d as char);
        assert_eq!(
            StrPTimeFmt::new(&s).unwrap_err().kind(),
            DtErrKind::UnknownItem,
            "expected UnknownItem for {s}"
        );
    }
    // Printer-only trim flag is not a parse directive.
    assert_eq!(
        StrPTimeFmt::new("%~f").unwrap_err().kind(),
        DtErrKind::UnknownItem
    );
    // Second flag after the first is treated as the directive byte.
    assert_eq!(
        StrPTimeFmt::new("%0-d").unwrap_err().kind(),
        DtErrKind::UnknownItem
    );
    assert_eq!(
        StrPTimeFmt::new("%-_d").unwrap_err().kind(),
        DtErrKind::UnknownItem
    );
}

#[test]
fn rejects_non_ascii() {
    assert_eq!(
        StrPTimeFmt::new("é%Y").unwrap_err().kind(),
        DtErrKind::InvalidInput
    );
    assert_eq!(
        StrPTimeFmt::new("%Y–%m").unwrap_err().kind(), // en-dash
        DtErrKind::InvalidInput
    );
    assert_eq!(
        StrPTimeFmt::new("%Y\u{2603}").unwrap_err().kind(),
        DtErrKind::InvalidInput
    );
}

#[test]
fn rejects_overlong_format() {
    let exact = "a".repeat(StrPTimeFmt::MAX_FMT_LEN);
    assert!(StrPTimeFmt::new(&exact).is_ok());

    let over = "a".repeat(StrPTimeFmt::MAX_FMT_LEN + 1);
    assert_eq!(
        StrPTimeFmt::new(&over).unwrap_err().kind(),
        DtErrKind::InvalidLen
    );

    // Over-long with a valid directive still rejects on length first.
    let over_fmt = format!("{}%Y", "x".repeat(StrPTimeFmt::MAX_FMT_LEN));
    assert_eq!(
        StrPTimeFmt::new(&over_fmt).unwrap_err().kind(),
        DtErrKind::InvalidLen
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Construction: acceptance surface (every supported directive)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn accepts_every_supported_directive_alone() {
    for &d in SUPPORTED_DIRECTIVES {
        let s = format!("%{}", d as char);
        assert_validated_never_format_errors(&s);
    }
}

#[test]
fn accepts_common_compound_formats() {
    let formats = [
        "",
        "literal only",
        "%%",
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.3fZ",
        "%Y-%m-%dT%H:%M:%S.%f",
        "%F %T",
        "%F%T",
        "%D",
        "%R",
        "%m/%d/%y",
        "%G-W%V-%u",
        "%Y-%j",
        "%s",
        "%J %L",
        "%*",
        "%I:%M:%S %p",
        "%A, %B %d, %Y",
        "%a %b %e",
        "%_d %_H",
        "%-d %-H",
        "%0d %0H",
        "%^A %#a",
        "%3Y %12m %001d",
        "%999Y",
        "%.f",
        "%.N",
        "%.3f",
        "%.6N",
        "%.0f",
        "%.18f",
        "%3f",
        "%6N",
        "%z",
        "%:z",
        "%::z",
        "%:::z",
        "%Q",
        "%n%t",
        " %Y ",
        "\t%Y\n",
        "100%% complete %Y",
        "%Y%%",
        "%%Y",
        "%-0d", // flag `-`, width `0`, directive `d`
        "%::Y", // colons are ignored on non-`z` directives by the parser
    ];
    for f in formats {
        assert_validated_never_format_errors(f);
    }
}

#[test]
fn max_fmt_len_boundary_with_directives() {
    // Pack as many `%Y` as fit exactly in MAX_FMT_LEN.
    let unit = "%Y";
    let n = StrPTimeFmt::MAX_FMT_LEN / unit.len();
    let s = unit.repeat(n);
    assert!(s.len() <= StrPTimeFmt::MAX_FMT_LEN);
    assert_validated_never_format_errors(&s);

    // Pad with literals up to exactly MAX_FMT_LEN.
    let pad = "x".repeat(StrPTimeFmt::MAX_FMT_LEN - "%F %T".len());
    let exact = format!("{pad}%F %T");
    assert_eq!(exact.len(), StrPTimeFmt::MAX_FMT_LEN);
    assert_validated_never_format_errors(&exact);
}

// ─────────────────────────────────────────────────────────────────────────────
// Core guarantee: validated ⇒ no format-structure errors
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn validated_formats_never_emit_format_structure_errors() {
    // Broad sweep: each directive with flags / width / colon variants.
    let mut cases = Vec::new();
    for &d in SUPPORTED_DIRECTIVES {
        let c = d as char;
        cases.push(format!("%{c}"));
        cases.push(format!("%-{c}"));
        cases.push(format!("%_{c}"));
        cases.push(format!("%0{c}"));
        cases.push(format!("%^{c}"));
        cases.push(format!("%#{c}"));
        cases.push(format!("%2{c}"));
        cases.push(format!("%02{c}"));
        cases.push(format!("%12{c}"));
        cases.push(format!("%:{c}"));
        cases.push(format!("%::{c}"));
        cases.push(format!("%:::{c}"));
    }
    // Fractional forms
    for tail in ["f", "N", "3f", "6N", "9f", "18N", "0f"] {
        cases.push(format!("%.{tail}"));
    }
    // Multi-directive
    cases.push("%Y-%m-%dT%H:%M:%S%.3f%z".into());
    cases.push("%F %T %L".into());
    cases.push("%* %m %d %H:%M:%S".into());

    for f in &cases {
        assert_validated_never_format_errors(f);
    }
}

/// Exhaustive single-byte directive × extension-prefix sweep over printable ASCII.
///
/// This is the heavy red-team net: every `%{prefix}{byte}` for common prefixes
/// and bytes `0x20..=0x7e`. Accepted formats must never surface format-structure
/// errors under hostile input and full flag combinations.
#[test]
fn exhaustive_ascii_directive_extension_sweep() {
    let prefixes: &[&str] = &[
        "", "-", "_", "0", "^", "#", //
        "1", "2", "12", "001", "999", //
        ":", "::", ":::", //
        "0:", "-:", "_2", "02", "^2", "#:", //
        "0::", "-:::", "12:", //
    ];
    let hostile = ["", "x", "%%%%%%%%", "2025-01-01 00:00:00"];
    let flag_sets = [
        (false, false, false),
        (true, true, true),
        (true, false, true),
        (false, true, false),
    ];

    let mut accepted = 0u32;
    for p in prefixes {
        for d in 0x20u8..=0x7eu8 {
            let s = format!("%{p}{}", d as char);
            let Ok(fmt) = StrPTimeFmt::new(&s) else {
                continue;
            };
            accepted += 1;
            for input in hostile {
                for (a, b, c) in flag_sets {
                    if let Err(e) = fmt.to_dt(input, a, b, c) {
                        assert!(
                            !is_format_structure_kind(e.kind()),
                            "guarantee break: fmt={s:?} input={input:?} flags=({a},{b},{c}) kind={:?}",
                            e.kind()
                        );
                    }
                }
            }
        }
    }
    // Sanity: we do accept a non-trivial set (not "everything rejected").
    assert!(
        accepted > 100,
        "expected many accepted formats, got {accepted}"
    );
}

/// Force a long input so `inp_can_end_before_fmt` cannot skip later directives;
/// still no format-structure errors on a kitchen-sink validated format.
#[test]
fn full_format_walk_kitchen_sink() {
    let fmt = StrPTimeFmt::new(
        "%Y-%m-%dT%H:%M:%S%.18f %z %Q %L %A %B %p %* %J %s %F %T %D %R %% %n%t %:::z",
    )
    .unwrap();
    let long = format!(
        "{}{}",
        "0".repeat(200),
        "A Monday January AM +0000 UTC TAI GPS America/New_York % "
    );
    for flags in [
        (false, false, false),
        (false, true, true),
        (true, true, true),
    ] {
        if let Err(e) = fmt.to_dt(&long, flags.0, flags.1, flags.2) {
            assert!(
                !is_format_structure_kind(e.kind()),
                "kitchen-sink format-structure error: {:?}",
                e.kind()
            );
        }
    }
}

/// `output_fmt` is a separate, unvalidated surface — broken output formats may
/// still fail with format-structure kinds. This documents the scope limit.
#[test]
fn output_fmt_is_not_covered_by_the_guarantee() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d").unwrap();
    let err = fmt
        .to_str_b("2020-01-01", "%", false, false, false, Lang::En)
        .unwrap_err();
    assert!(
        is_format_structure_kind(err.kind()) || err.kind() == DtErrKind::UnexpectedEnd,
        "expected a format-structure failure for broken output_fmt, got {:?}",
        err.kind()
    );
}

/// Regression: more than three colons must be rejected at construction.
///
/// The parser hard-caps colon consumption at 3; a fourth `:` becomes the
/// directive byte (`UnknownItem`). Validation used to swallow unlimited
/// colons and accept `%::::z`, then `to_dt` failed with `UnknownItem` —
/// breaking the “pre-validated format never fails as a format” guarantee.
#[test]
fn more_than_three_colons_rejected_at_construction() {
    for s in ["%::::z", "%::::Y", "%:::::z", "%0::::z", "%-::::z"] {
        let err = StrPTimeFmt::new(s).unwrap_err();
        assert_eq!(
            err.kind(),
            DtErrKind::UnknownItem,
            "expected UnknownItem for {s:?}, got {:?}",
            err.kind()
        );
    }
    // Three colons remain legal.
    assert!(StrPTimeFmt::new("%:::z").is_ok());
    assert_validated_never_format_errors("%:::z");
}

// ─────────────────────────────────────────────────────────────────────────────
// Use-case guarantees: parse / reformat with a validated format
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn to_dt_parses_matching_input() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d %H:%M:%S").unwrap();
    let dt = fmt
        .to_dt("2025-05-23 14:30:00", false, false, false)
        .unwrap();
    let g = dt.to_ymd();
    assert_eq!(
        (g.yr(), g.mo(), g.day(), g.hr(), g.min(), g.sec()),
        (2025, 5, 23, 14, 30, 0)
    );
}

#[test]
fn to_dt_matches_dt_from_strptime() {
    let formats_and_inputs = [
        ("%F %T", "2024-04-15 14:30:45"),
        ("%Y-%m-%dT%H:%M:%S", "2000-01-01T12:00:00"),
        ("%s", "1713191445"),
        ("%Y-%j", "2025-106"),
        ("%m/%d/%y", "04/16/25"),
    ];
    for (f, input) in formats_and_inputs {
        let pre = StrPTimeFmt::new(f).unwrap();
        let via_fmt = pre.to_dt(input, false, false, false).unwrap();
        let via_dt = Dt::from_strptime(input, f, false, false, false).unwrap();
        assert_eq!(via_fmt, via_dt, "mismatch for fmt={f:?} input={input:?}");
    }
}

#[test]
fn to_str_b_reformats_without_format_structure_errors() {
    let fmt = StrPTimeFmt::new("%Y-%m-%dT%H:%M:%S").unwrap();
    let out = fmt
        .to_str_b(
            "2000-01-01T12:00:00",
            "%d %m %Y %H:%M:%S",
            false,
            false,
            false,
            Lang::En,
        )
        .unwrap();
    assert_eq!(out.as_str(), "01 01 2000 12:00:00");
}

#[test]
fn to_str_b_input_errors_are_not_format_structure_errors() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d").unwrap();
    let err = fmt
        .to_str_b("nope", "%F", false, false, false, Lang::En)
        .unwrap_err();
    assert!(!is_format_structure_kind(err.kind()));
}

#[cfg(feature = "alloc")]
#[test]
fn to_str_matches_to_str_b() {
    let fmt = StrPTimeFmt::new("%Y-%m-%dT%H:%M:%S").unwrap();
    let a = fmt
        .to_str(
            "2000-01-01T12:00:00",
            "%d %m %Y %H:%M:%S",
            false,
            false,
            false,
            Lang::En,
        )
        .unwrap();
    let b = fmt
        .to_str_b(
            "2000-01-01T12:00:00",
            "%d %m %Y %H:%M:%S",
            false,
            false,
            false,
            Lang::En,
        )
        .unwrap();
    assert_eq!(a, b.as_str());
}

#[test]
fn reusable_across_many_parses() {
    let fmt = StrPTimeFmt::new("%F %T").unwrap();
    for day in 1..=28 {
        let s = format!("2025-02-{day:02} 10:11:12");
        let dt = fmt.to_dt(&s, false, false, false).unwrap();
        assert_eq!(dt.to_ymd().day(), day);
    }
}

#[test]
fn clone_preserves_behavior() {
    let a = StrPTimeFmt::new("%Y-%m-%d").unwrap();
    let b = a.clone();
    let da = a.to_dt("2021-08-09", false, false, false).unwrap();
    let db = b.to_dt("2021-08-09", false, false, false).unwrap();
    assert_eq!(da, db);
}

// ─────────────────────────────────────────────────────────────────────────────
// Input-side failures remain expected (guarantee is not “always Ok”)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn parse_still_fails_on_bad_input() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d %H:%M:%S").unwrap();

    let empty = fmt.to_dt("", false, false, false).unwrap_err();
    assert!(!is_format_structure_kind(empty.kind()));

    let mismatch = fmt
        .to_dt("2025/04/16 14:30:45", false, false, false)
        .unwrap_err();
    assert_eq!(mismatch.kind(), DtErrKind::MismatchedLiteral);

    let trailing = fmt
        .to_dt("2025-04-16 14:30:45 trailing", false, false, false)
        .unwrap_err();
    assert_eq!(trailing.kind(), DtErrKind::TrailingCharacters);

    // Incomplete date components with partial disallowed.
    let year_only = StrPTimeFmt::new("%Y").unwrap();
    let incomplete = year_only.to_dt("2025", false, false, false).unwrap_err();
    assert_eq!(incomplete.kind(), DtErrKind::Incomplete);

    // Same format with allow_partial_date succeeds.
    let ok = year_only.to_dt("2025", false, false, true).unwrap();
    assert_eq!(ok.to_ymd().yr(), 2025);
}

#[test]
fn parse_out_of_range_is_input_error() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d").unwrap();
    let err = fmt.to_dt("2025-13-01", false, false, false).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            DtErrKind::MonthOutOfRange | DtErrKind::InvalidDate | DtErrKind::DayOutOfRange
        ),
        "unexpected kind {:?}",
        err.kind()
    );
    assert!(!is_format_structure_kind(err.kind()));
}

// ─────────────────────────────────────────────────────────────────────────────
// API surface / aliases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn dt_parse_fmt_is_strp_time_fmt_new() {
    let overlong = "a".repeat(StrPTimeFmt::MAX_FMT_LEN + 1);
    let samples: &[&str] = &["%F %T", "%", "%c", "%q", "%.3f", overlong.as_str(), "café"];
    for s in samples {
        let a = StrPTimeFmt::new(s).map(|_| ()).map_err(|e| e.kind());
        let b = Dt::parse_fmt(s).map(|_| ()).map_err(|e| e.kind());
        assert_eq!(
            a, b,
            "Dt::parse_fmt diverged from StrPTimeFmt::new for {s:?}"
        );
    }
}

#[test]
fn max_fmt_len_constant() {
    assert_eq!(StrPTimeFmt::MAX_FMT_LEN, 256);
}

// ─────────────────────────────────────────────────────────────────────────────
// Cross-check: raw parser agrees that accepted formats are walkable
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn accepted_format_walks_without_unknown_or_unsupported() {
    // Empty input + inp_can_end_before_fmt lets the walk stop early without
    // requiring field values; we only care that the format path itself is
    // free of Unknown/Unsupported/Truncated/InvalidFractional.
    let formats = [
        "%Y-%m-%d",
        "%F %T %z",
        "%:::z",
        "%.18f",
        "%^A %#B",
        "%* %L %J %s",
        "%G-W%V-%u %U %W %w",
        "%I %l %p %P %k %H",
        "%e %d %h %b %m",
        "%% %n %t",
    ];
    for f in formats {
        let fmt = StrPTimeFmt::new(f).unwrap();
        if let Err(e) = fmt.to_dt("", true, true, true) {
            assert!(
                !is_format_structure_kind(e.kind()),
                "fmt={f:?} kind={:?}",
                e.kind()
            );
        }
    }
}

#[test]
fn rejected_formats_also_fail_raw_from_strptime() {
    // Construction is a strict filter: anything it rejects must not be
    // silently accepted by the live parser either (otherwise clients who
    // skip StrPTimeFmt would see different behavior).
    let bad = [
        "%", "%0", "%.", "%.x", "%c", "%r", "%x", "%X", "%Z", "%q", "%~f", "%.~f", "%::::z", "%Y%",
    ];
    for f in bad {
        assert!(
            StrPTimeFmt::new(f).is_err(),
            "expected construction failure for {f:?}"
        );
        // Live parse with any input should also fail (format or input — but
        // for these pure-format defects, the failure is deterministic).
        let live = Dt::from_strptime("2025-01-01", f, true, true, true);
        assert!(
            live.is_err(),
            "raw from_strptime unexpectedly accepted format {f:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Happy path with Scale / leap second still works through StrPTimeFmt
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn leap_second_and_scale_via_validated_fmt() {
    let fmt = StrPTimeFmt::new("%Y-%m-%d %H:%M:%S %L").unwrap();
    let dt = fmt
        .to_dt("2016-12-31 23:59:60 UTC", false, false, false)
        .unwrap();
    let g = dt.to_ymd();
    assert_eq!(g.sec(), 60);
    assert_eq!(dt.target, Scale::UTC);
}
