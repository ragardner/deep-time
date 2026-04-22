use crate::{
    ClassifiedDate, ClockType, Lang, TimePoint, TimeUnits, classify_date,
    generate_syslog_candidates, try_compatible_formats,
};

/// Parses syslog-style dates missing the year (e.g. "Mar  5 10:23:45", "Dec 31 23:59:59").
///
/// - Try current year first.
/// - If the parsed date is **more than 30 days in the future**, assume previous year.
///   This covers the classic December-in-January case while tolerating clock drift and minor delays.
///
/// Pass `reference_date` when reprocessing historical logs for perfect reproducibility.
#[inline(always)]
pub(crate) fn parse_syslog_no_year(
    input: &str,
    reference_date: Option<TimePoint>,
    lang: Lang,
) -> Option<TimePoint> {
    let now = reference_date.unwrap_or_else(|| TimePoint::now(ClockType::UTC));
    let (this_year, _, _) = now.to_gregorian_date(None);

    let try_with_year = |year: i64| -> Option<TimePoint> {
        // Prepend the year
        let s = std::format!("{} {}", year, input);
        let cls = classify_date(&s, lang).ok()?;
        match cls {
            ClassifiedDate::Cls(c) => try_compatible_formats(&s, generate_syslog_candidates(&c)),
            _ => None,
        }
    };
    if let Some(dt) = try_with_year(this_year) {
        if dt > now + 30.days() {
            return try_with_year(this_year - 1);
        }
        return Some(dt);
    }
    try_with_year(this_year - 1)
}
