#[cfg(feature = "locale")]
use crate::locale_prefers_day_first;
use crate::{ConnectorType, DateClassification, OrderFirst, Token};

/// Returns the most likely date component ordering for the input string.
///
/// Heuristic priority:
/// 1. Pure-numeric compact formats (`YYYYMMDD`, `YYMMDD`, `YYMMDDHHMMSS`, …)
///    → Year (sortable / ISO-style; common in logs, filenames, DBs, APIs).
/// 2. First or second number in 13–31 → Day or Month respectively.
/// 3. Delimited string starting with a plausible 4-digit year → Year.
/// 4. ISO markers (`T`, offset/zone) → Year.
/// 5. Fallback: locale (if enabled), otherwise Day.
///
/// `/` alone is not used as a signal (day-first vs month-first is split).
#[inline]
pub(crate) fn smart_detect_date_order(s: &str, class: &DateClassification) -> OrderFirst {
    // 1. Pure-numeric compact: e.g. `240314153045` (YYMMDDHHMMSS) → Year.
    //    Handled first so digit-length doesn't get misread as day/month.
    if class.is_pure_numeric && class.num_digits >= 6 {
        return OrderFirst::Year;
    }

    let s = s.trim_start_matches(['+', '-']);

    // 2. Delimited, starts with a 4-digit year in a plausible range.
    if matches!(class.date_tokens.first(), Some(Token::Digits(n)) if *n >= 4)
        && let Some(year_candidate) = s.get(0..4).and_then(|p| p.parse::<u16>().ok())
        && (1900..=2100).contains(&year_candidate)
    {
        return OrderFirst::Year;
    }

    // 3. Numeric plausibility: only parse the second number when the first
    //    is in 1..=12 (ambiguous between day and month).
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
        return OrderFirst::Day;
    }

    let second = if (1..=12).contains(&first) {
        num_iter.next().unwrap_or(0)
    } else {
        0
    };

    if second > 12 && second <= 31 {
        return OrderFirst::Month;
    }

    // 4. ISO timestamp markers.
    if class.connector == ConnectorType::UpperT || class.has_offset_or_zone() {
        return OrderFirst::Year;
    }

    // 5. Locale fallback, or Day when locale is disabled.
    #[cfg(feature = "locale")]
    {
        if locale_prefers_day_first() {
            OrderFirst::Day
        } else {
            OrderFirst::Month
        }
    }
    #[cfg(not(feature = "locale"))]
    {
        OrderFirst::Day
    }
}
