use crate::{
    ClassifiedDate, DateClassification, DateToken, Lang, TimePoint, classify_date,
    generate_unambiguous_candidates, try_compatible_formats,
};
use alloc::string::ToString;

/// Returns true if the input uses an ISO week format but is missing the weekday
/// (e.g. "2024-W11", "2025-W01", "2024-W11 14:30:00", "2024-W11T14:30:00", etc.).
#[inline(always)]
pub(crate) fn is_week_date_missing_weekday(cls: &DateClassification) -> bool {
    cls.has_w                                   // has the literal "W"
        && cls.num_hyphen == 1                  // exactly one hyphen in the date part
        && matches!(cls.tokens.first(), Some(DateToken::Digits(n)) if *n >= 4)
}

/// Expects a pre-classified (normalized to English) date string.
pub(crate) fn parse_week_date_no_weekday(
    normalized: &str,
    lang: Lang,
    ref_time: &Option<TimePoint>,
) -> Option<TimePoint> {
    // Insert "-1" (Monday) right after the week number.
    // This works whether the string is pure date or date+time.
    let Some(w_pos) = normalized.find("-W") else {
        return None;
    };

    let mut normalized = normalized.to_string();
    let week_end = w_pos + 4; // after "-W" + 2 week digits
    if week_end <= normalized.len() {
        let after = &normalized[week_end..];
        if after.is_empty() || after.starts_with(' ') || after.starts_with('T') {
            normalized.insert_str(week_end, "-1");
        }
    }
    let classification = classify_date(&normalized.to_ascii_lowercase(), lang, ref_time).ok()?;
    match classification {
        ClassifiedDate::Cls(cls) => {
            try_compatible_formats(&cls.date, generate_unambiguous_candidates(&cls))
        }
        _ => None,
    }
}
