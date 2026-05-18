use crate::Dt;
use crate::LEGACY_ORDINAL_YEAR_RANGE;
use crate::TimeParts;

/// 5-digit legacy ordinal: YYDDD
#[inline]
pub(crate) fn parse_yyddd(s: &str) -> Option<Dt> {
    if s.len() != 5 {
        return None;
    }
    let parsed = TimeParts::from_str("%y%j", s, true, true, false).ok()?;
    if let Some(y) = parsed.year
        && !LEGACY_ORDINAL_YEAR_RANGE.contains(&(y as i32))
    {
        return None;
    }
    parsed.to_dt().ok()
}

/// 7-digit legacy ordinal: YYYYDDD (only accepted inside LEGACY_ORDINAL_YEAR_RANGE)
#[inline]
pub(crate) fn parse_yyyyjjj(s: &str) -> Option<Dt> {
    let parsed = TimeParts::from_str("%Y%j", s, true, true, false).ok()?;
    if let Some(y) = parsed.year
        && !LEGACY_ORDINAL_YEAR_RANGE.contains(&(y as i32))
    {
        return None;
    }
    parsed.to_dt().ok()
}
