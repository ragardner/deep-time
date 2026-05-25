use crate::{
    ClassifiedDate, DateClassification, Dt, Lang, Token, classify_date,
    generate_unambiguous_candidates, try_compatible_formats,
};
use alloc::string::ToString;

/// Returns true if the input uses an ISO week format but is missing the weekday
/// (e.g. "2024-W11", "2025-W01", "2024-W11 14:30:00", "2024-W11T14:30:00", etc.).
#[inline]
pub(crate) fn is_week_date_missing_weekday(cls: &DateClassification) -> bool {
    cls.has_w                                   // has the literal "W"
        && matches!(cls.date_tokens.first(), Some(Token::Digits(n)) if *n >= 4)
}

/// For ISO week dates missing the weekday (e.g. "2024W11", "2024-W11"),
/// defaults the weekday to Monday (`-1`).
pub(crate) fn parse_week_date_no_weekday(
    orig_class: &DateClassification,
    lang: Lang,
    ref_time: &Option<Dt>,
) -> Option<Dt> {
    let date = &orig_class.date;

    // Find the start of the week part ("W" or "-W")
    let w_pos = match date.find("-W") {
        Some(pos) => pos + 1, // point to the actual 'W'
        None => date.find('W')?,
    };

    // Find the end of the week number (dynamically, in case someone writes W5 instead of W05)
    let mut week_end = w_pos + 1;
    while week_end < date.len() && date.as_bytes()[week_end].is_ascii_digit() {
        week_end += 1;
    }

    let mut new_date = date.to_string();

    // Only insert "-1" if there's nothing (or only a separator/time marker) after the week number
    let after = &new_date[week_end..];
    if after.is_empty() || after.starts_with([' ', 'T', '-', '.', '/', ',']) {
        new_date.insert_str(week_end, "-1");
    } else {
        // Already has a weekday or something unexpected — don't touch it
        return None;
    }

    let classification = classify_date(&new_date.to_ascii_lowercase(), lang, ref_time).ok()?;

    match classification {
        ClassifiedDate::Cls(cls) => {
            try_compatible_formats(&cls.date, generate_unambiguous_candidates(&cls))
        }
        _ => None,
    }
}
