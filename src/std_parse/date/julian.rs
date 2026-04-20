use crate::{ClockType, TimePoint};
use crate::{
    JD_EPOCH_NANOS, JD_RANGE, MJD_EPOCH_NANOS, MJD_RANGE, NS_PER_DAY, NS_PER_HALF_DAY,
    frac_to_nanos,
};

const UNIX_EPOCH_TO_J2000_NOON_UTC: i64 = 946_728_000;

/// Modified Julian Date (MJD) interpreted as UTC
#[inline(always)]
pub(crate) fn parse_mjd(s: &str) -> Option<TimePoint> {
    let (int_part, frac_part) = if let Some(dot) = s.find('.') {
        (&s[..dot], &s[dot + 1..])
    } else {
        (s, "")
    };
    let days: i64 = int_part.parse().ok()?;
    if !MJD_RANGE.contains(&days) {
        return None;
    }

    let frac_nanos = frac_to_nanos(frac_part)?; // ← fixed: now propagates error correctly

    let unix_nanos: i128 = (days as i128) * NS_PER_DAY + frac_nanos - MJD_EPOCH_NANOS;

    let secs_since_unix = unix_nanos.div_euclid(1_000_000_000);
    let rem_nanos = unix_nanos.rem_euclid(1_000_000_000) as u64;

    let sec = (secs_since_unix as i64) - UNIX_EPOCH_TO_J2000_NOON_UTC;
    let subsec = rem_nanos * 1_000_000_000;

    Some(TimePoint::new(sec, subsec, ClockType::UTC))
}

/// Julian Day (JD) interpreted as UTC
#[inline(always)]
pub(crate) fn parse_jd(s: &str, astronomical_noon: bool) -> Option<TimePoint> {
    let (int_part, frac_part) = if let Some(dot) = s.find('.') {
        (&s[..dot], &s[dot + 1..])
    } else {
        (s, "")
    };
    let days: i64 = int_part.parse().ok()?;
    if !JD_RANGE.contains(&days) {
        return None;
    }

    let frac_nanos = if frac_part.is_empty() {
        if astronomical_noon {
            0
        } else {
            NS_PER_HALF_DAY
        }
    } else {
        frac_to_nanos(frac_part)?
    };

    let unix_nanos = (days as i128) * NS_PER_DAY + frac_nanos - JD_EPOCH_NANOS;

    let secs_since_unix = unix_nanos.div_euclid(1_000_000_000);
    let rem_nanos = unix_nanos.rem_euclid(1_000_000_000) as u64;

    let sec = (secs_since_unix as i64) - UNIX_EPOCH_TO_J2000_NOON_UTC;
    let subsec = rem_nanos * 1_000_000_000;

    Some(TimePoint::new(sec, subsec, ClockType::UTC))
}
