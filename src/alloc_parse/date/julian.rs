use crate::{Dt, JD_RANGE, MJD_RANGE, Real, Scale};

/// Modified Julian Date (MJD) interpreted as UTC
pub(crate) fn parse_mjd(s: &str) -> Option<Dt> {
    let mjd: Real = s.parse().ok()?;
    let days = mjd as i64;
    if !MJD_RANGE.contains(&days) {
        return None;
    }
    Some(Dt::from_mjd_f(mjd, Scale::TAI))
}

/// Julian Day (JD) interpreted as UTC
pub(crate) fn parse_jd(s: &str, astronomical_noon: bool) -> Option<Dt> {
    let mut jd: Real = s.parse().ok()?;
    let days = jd as i64;
    if !JD_RANGE.contains(&days) {
        return None;
    }
    if astronomical_noon {
        jd += f!(0.5);
    }
    Some(Dt::from_jd_f(jd, Scale::TAI))
}
