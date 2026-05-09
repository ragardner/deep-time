use crate::{
    Dt, JD_EPOCH_NANOS, JD_RANGE, MJD_EPOCH_NANOS, MJD_RANGE, NS_PER_DAY, NS_PER_HALF_DAY, Scale,
    TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, frac_to_nanos,
};

// TODO: inefficient calculations

/// Modified Julian Date (MJD) interpreted as UTC
pub(crate) fn parse_mjd(s: &str) -> Option<Dt> {
    let (int_part, frac_part) = if let Some(dot) = s.find('.') {
        (&s[..dot], &s[dot + 1..])
    } else {
        (s, "")
    };
    let days: i64 = int_part.parse().ok()?;
    if !MJD_RANGE.contains(&days) {
        return None;
    }

    let frac_nanos = frac_to_nanos(frac_part)?;

    let unix_nanos: i128 = (days as i128) * NS_PER_DAY + frac_nanos - MJD_EPOCH_NANOS;

    let secs_since_unix = unix_nanos.div_euclid(1_000_000_000);
    let rem_nanos = unix_nanos.rem_euclid(1_000_000_000) as u64;

    let sec = (secs_since_unix as i64) - TAI_SECS_1970_MIDNIGHT_TO_2000_NOON;
    let subsec = rem_nanos * 1_000_000_000;

    Some(Dt::from(sec, subsec, Scale::UTC))
}

/// Julian Day (JD) interpreted as UTC
pub(crate) fn parse_jd(s: &str, astronomical_noon: bool) -> Option<Dt> {
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

    let sec = (secs_since_unix as i64) - TAI_SECS_1970_MIDNIGHT_TO_2000_NOON;
    let subsec = rem_nanos * 1_000_000_000;

    Some(Dt::from(sec, subsec, Scale::UTC))
}
