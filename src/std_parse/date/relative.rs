use crate::{ClassifiedDate, ClockType, Lang, TimePoint, natural_duration_to_delta};
use std::string::String;

#[inline]
pub(crate) fn parse_relative(
    s: &str,
    lang: Lang,
    use_dur_finder: bool,
) -> Result<ClassifiedDate, String> {
    let delta = natural_duration_to_delta(s, lang, use_dur_finder)?;
    let time_point = TimePoint::now(ClockType::UTC);
    time_point.saturating_add_ref(&delta);
    Ok(ClassifiedDate::Parsed(time_point))
}
