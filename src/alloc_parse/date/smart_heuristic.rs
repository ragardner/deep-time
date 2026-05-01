#[cfg(feature = "locale")]
use crate::locale_prefers_day_first;
use crate::{ConnectorType, DateClassification, DateToken, DetectedDateOrder};

/// Returns the most likely **date component ordering** for the input string.
///
/// Research-backed heuristics (confirmed via date parsers, filename conventions,
/// log formats, DB keys, and ISO practices):
/// • Pure-numeric compact formats (`YYYYMMDD`, `YYMMDD`, `YYMMDDHHMMSS`, etc.)
///   are **overwhelmingly** YearFirst in real-world usage (logs, filenames,
///   databases, APIs, configs). Even two-digit-year versions are YearFirst.
/// • 13–31 numeric plausibility is the strongest universal signal.
/// • Delimited 4-digit year start + plausible range is the next strong signal.
/// • ISO timestamp markers (`T`, `Z`, zoned/offset) are very reliable YearFirst.
/// • `/` is deliberately ignored (culturally split).
/// • Final fallback = cached locale (if enabled) or `DayFirst` (global majority).
#[inline]
pub(crate) fn smart_detect_date_order(s: &str, class: &DateClassification) -> DetectedDateOrder {
    // ------------------------------------------------------------------
    // 1. Pure-numeric compact formats (the exact case you reported)
    //    `240314153045` = classic YYMMDDHHMMSS → YearFirst.
    //    Research confirms: these are almost always YearFirst (sortable,
    //    ISO-derived, dominant in logs/filenames/DBs). We special-case
    //    them *before* everything else.
    // ------------------------------------------------------------------
    if class.is_pure_numeric && class.num_digits >= 6 {
        return DetectedDateOrder::YearFirst;
    }

    let s = s.trim_start_matches(|c: char| c == '+' || c == '-');

    // ------------------------------------------------------------------
    // 2. Delimited formats starting with a 4-digit year
    //    (only reached for non-pure-numeric strings)
    // ------------------------------------------------------------------
    if matches!(class.tokens.first(), Some(DateToken::Digits(n)) if *n >= 4) {
        if let Some(year_candidate) = s.get(0..4).and_then(|p| p.parse::<u16>().ok()) {
            if (1900..=2100).contains(&year_candidate) {
                return DetectedDateOrder::YearFirst;
            }
        }
    }

    // ------------------------------------------------------------------
    // 3. Numeric plausibility check (micro-optimized)
    //    We parse the second number *only* when the first is ambiguous.
    // ------------------------------------------------------------------
    let mut num_iter = s
        .split(|c: char| matches!(c, '/' | '-' | '.' | ' ' | 'T'))
        .filter_map(|p| {
            let p = p.trim();
            if p.is_empty() {
                None
            } else {
                p.parse::<u32>().ok()
            }
        });

    let first = num_iter.next().unwrap_or(0);

    if first > 12 && first <= 31 {
        return DetectedDateOrder::DayFirst;
    }

    let second = if first >= 1 && first <= 12 {
        num_iter.next().unwrap_or(0)
    } else {
        0
    };

    if second > 12 && second <= 31 {
        return DetectedDateOrder::MonthFirst;
    }

    // ------------------------------------------------------------------
    // 4. Strong ISO timestamp markers
    // ------------------------------------------------------------------

    if class.connector == ConnectorType::UpperT || class.has_offset_or_zone() {
        return DetectedDateOrder::YearFirst;
    }

    // ------------------------------------------------------------------
    // 5. Locale / global majority fallback
    // ------------------------------------------------------------------
    #[cfg(feature = "locale")]
    {
        if locale_prefers_day_first() {
            DetectedDateOrder::DayFirst
        } else {
            DetectedDateOrder::MonthFirst
        }
    }
    #[cfg(not(feature = "locale"))]
    {
        DetectedDateOrder::DayFirst
    }
}
