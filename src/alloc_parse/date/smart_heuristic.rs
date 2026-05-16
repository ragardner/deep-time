#[cfg(feature = "locale")]
use crate::locale_prefers_day_first;
use crate::{ConnectorType, DateClassification, DateOrderFirst, DateToken};

/// Returns the most likely **date component ordering** for the input string.
///
/// Research-backed heuristics (confirmed via date parsers, filename conventions,
/// log formats, DB keys, and ISO practices):
/// • Pure-numeric compact formats (`YYYYMMDD`, `YYMMDD`, `YYMMDDHHMMSS`, etc.)
///   are **overwhelmingly** Year in real-world usage (logs, filenames,
///   databases, APIs, configs). Even two-digit-year versions are Year.
/// • 13–31 numeric plausibility is the strongest universal signal.
/// • Delimited 4-digit year start + plausible range is the next strong signal.
/// • ISO timestamp markers (`T`, `Z`, zoned/offset) are very reliable Year.
/// • `/` is deliberately ignored (culturally split).
/// • Final fallback = cached locale (if enabled) or `Day` (global majority).
#[inline]
pub(crate) fn smart_detect_date_order(s: &str, class: &DateClassification) -> DateOrderFirst {
    // ------------------------------------------------------------------
    // 1. Pure-numeric compact formats (the exact case you reported)
    //    `240314153045` = classic YYMMDDHHMMSS → Year.
    //    Research confirms: these are almost always Year (sortable,
    //    ISO-derived, dominant in logs/filenames/DBs). We special-case
    //    them *before* everything else.
    // ------------------------------------------------------------------
    if class.is_pure_numeric && class.num_digits >= 6 {
        return DateOrderFirst::Year;
    }

    let s = s.trim_start_matches(['+', '-']);

    // ------------------------------------------------------------------
    // 2. Delimited formats starting with a 4-digit year
    //    (only reached for non-pure-numeric strings)
    // ------------------------------------------------------------------
    if matches!(class.tokens.first(), Some(DateToken::Digits(n)) if *n >= 4)
        && let Some(year_candidate) = s.get(0..4).and_then(|p| p.parse::<u16>().ok())
        && (1900..=2100).contains(&year_candidate)
    {
        return DateOrderFirst::Year;
    }

    // ------------------------------------------------------------------
    // 3. Numeric plausibility check (micro-optimized)
    //    We parse the second number *only* when the first is ambiguous.
    // ------------------------------------------------------------------
    let mut num_iter = s.split(['/', '-', '.', ' ', 'T']).filter_map(|p| {
        let p = p.trim();
        if p.is_empty() {
            None
        } else {
            p.parse::<u32>().ok()
        }
    });

    let first = num_iter.next().unwrap_or(0);

    if first > 12 && first <= 31 {
        return DateOrderFirst::Day;
    }

    let second = if (1..=12).contains(&first) {
        num_iter.next().unwrap_or(0)
    } else {
        0
    };

    if second > 12 && second <= 31 {
        return DateOrderFirst::Month;
    }

    // ------------------------------------------------------------------
    // 4. Strong ISO timestamp markers
    // ------------------------------------------------------------------

    if class.connector == ConnectorType::UpperT || class.has_offset_or_zone() {
        return DateOrderFirst::Year;
    }

    // ------------------------------------------------------------------
    // 5. Locale / global majority fallback
    // ------------------------------------------------------------------
    #[cfg(feature = "locale")]
    {
        if locale_prefers_day_first() {
            DateOrderFirst::Day
        } else {
            DateOrderFirst::Month
        }
    }
    #[cfg(not(feature = "locale"))]
    {
        DateOrderFirst::Day
    }
}
