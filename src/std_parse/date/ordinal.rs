use crate::ClockType;
use crate::LEGACY_ORDINAL_YEAR_RANGE;
use crate::TimePoint;
use crate::parser::strptime;

/// 5-digit legacy ordinal: YYDDD
#[inline(always)]
pub(crate) fn parse_yyddd(s: &str) -> Option<TimePoint> {
    if s.len() != 5 {
        return None;
    }
    let parsed = strptime("%y%j", s, true, false).ok()?;
    if let Some(y) = parsed.year {
        if !LEGACY_ORDINAL_YEAR_RANGE.contains(&(y as i32)) {
            return None;
        }
    }
    parsed.to_time_point(ClockType::UTC).ok()
}

/// 7-digit legacy ordinal: YYYYDDD (only accepted inside LEGACY_ORDINAL_YEAR_RANGE)
#[inline(always)]
pub(crate) fn parse_yyyyjjj(s: &str) -> Option<TimePoint> {
    let parsed = strptime("%Y%j", s, true, false).ok()?;
    if let Some(y) = parsed.year {
        if !LEGACY_ORDINAL_YEAR_RANGE.contains(&(y as i32)) {
            return None;
        }
    }
    parsed.to_time_point(ClockType::UTC).ok()
}
