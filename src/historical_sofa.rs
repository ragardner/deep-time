use crate::{ClockType, Real, SEC_PER_DAY, TimePoint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaiUtcPre1972 {
    /// Year of the effective UTC date
    pub yr: i32,
    /// Month (1-12) of the effective UTC date
    pub mo: u8,
    /// Day of the effective UTC date
    pub day: u8,
    /// Julian Date (JD) at 0h UT on the effective date (start of interval)
    pub jd: f64,
    /// Reference MJD used in the linear drift formula: offset + (MJD - mjd_ref) * drift
    pub mjd_ref: f64,
    /// Constant offset (seconds) in the TAI−UTC formula
    pub offset: f64,
    /// Drift rate (seconds per day) — this is the historical frequency offset effect
    pub drift: f64,
    /// Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)  
    /// for this effective date at exactly 00:00:00 UTC.
    /// All values are negative because these dates are pre-1970.
    pub unix_sec: i64,
    /// Library UTC timestamp against library epoch
    pub dt_tai_sec: i64,
    pub dt_tai_subsec: u64,
}

/// Authoritative pre-1972 TAI−UTC entries from the official USNO `tai-utc.dat`
/// [file](https://maia.usno.navy.mil/ser7/tai-utc.dat).  
/// These 13 intervals contain the exact frequency offsets and linear drifts used by the IAU SOFA library
/// and all other high-precision astronomy/time-conversion software before UTC switched to pure leap-second mode.
pub const SOFA_TAI_UTC_PRE_1972: &[TaiUtcPre1972] = &[
    TaiUtcPre1972 {
        yr: 1961,
        mo: 1,
        day: 1,
        jd: 2437300.5,
        mjd_ref: 37300.0,
        offset: 1.4228180,
        drift: 0.001296,
        unix_sec: -283996800,
        dt_tai_sec: -1230724799,
        dt_tai_subsec: 422817999999999936,
    },
    TaiUtcPre1972 {
        yr: 1961,
        mo: 8,
        day: 1,
        jd: 2437512.5,
        mjd_ref: 37300.0,
        offset: 1.3728180,
        drift: 0.001296,
        unix_sec: -265680000,
        dt_tai_sec: -1212407999,
        dt_tai_subsec: 647570000000000000,
    },
    TaiUtcPre1972 {
        yr: 1962,
        mo: 1,
        day: 1,
        jd: 2437665.5,
        mjd_ref: 37665.0,
        offset: 1.8458580,
        drift: 0.0011232,
        unix_sec: -252460800,
        dt_tai_sec: -1199188799,
        dt_tai_subsec: 845858000000000000,
    },
    TaiUtcPre1972 {
        yr: 1963,
        mo: 11,
        day: 1,
        jd: 2438334.5,
        mjd_ref: 37665.0,
        offset: 1.9458580,
        drift: 0.0011232,
        unix_sec: -194659200,
        dt_tai_sec: -1141387198,
        dt_tai_subsec: 697278800000000256,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 1,
        day: 1,
        jd: 2438395.5,
        mjd_ref: 38761.0,
        offset: 3.2401300,
        drift: 0.001296,
        unix_sec: -189388800,
        dt_tai_sec: -1136116798,
        dt_tai_subsec: 765794000000000128,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 4,
        day: 1,
        jd: 2438486.5,
        mjd_ref: 38761.0,
        offset: 3.3401300,
        drift: 0.001296,
        unix_sec: -181526400,
        dt_tai_sec: -1128254398,
        dt_tai_subsec: 983730000000000000,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 9,
        day: 1,
        jd: 2438639.5,
        mjd_ref: 38761.0,
        offset: 3.4401300,
        drift: 0.001296,
        unix_sec: -168307200,
        dt_tai_sec: -1115035197,
        dt_tai_subsec: 282017999999999872,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 1,
        day: 1,
        jd: 2438761.5,
        mjd_ref: 38761.0,
        offset: 3.5401300,
        drift: 0.001296,
        unix_sec: -157766400,
        dt_tai_sec: -1104494397,
        dt_tai_subsec: 540130000000000000,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 3,
        day: 1,
        jd: 2438820.5,
        mjd_ref: 38761.0,
        offset: 3.6401300,
        drift: 0.001296,
        unix_sec: -152668800,
        dt_tai_sec: -1099396797,
        dt_tai_subsec: 716594000000000128,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 7,
        day: 1,
        jd: 2438942.5,
        mjd_ref: 38761.0,
        offset: 3.7401300,
        drift: 0.001296,
        unix_sec: -142128000,
        dt_tai_sec: -1088855997,
        dt_tai_subsec: 974706000000000256,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 9,
        day: 1,
        jd: 2439004.5,
        mjd_ref: 38761.0,
        offset: 3.8401300,
        drift: 0.001296,
        unix_sec: -136771200,
        dt_tai_sec: -1083499196,
        dt_tai_subsec: 155057999999999488,
    },
    TaiUtcPre1972 {
        yr: 1966,
        mo: 1,
        day: 1,
        jd: 2439126.5,
        mjd_ref: 39126.0,
        offset: 4.3131700,
        drift: 0.002592,
        unix_sec: -126230400,
        dt_tai_sec: -1072958396,
        dt_tai_subsec: 313170000000000384,
    },
    TaiUtcPre1972 {
        yr: 1968,
        mo: 2,
        day: 1,
        jd: 2439887.5,
        mjd_ref: 39126.0,
        offset: 4.2131700,
        drift: 0.002592,
        unix_sec: -60480000,
        dt_tai_sec: -1007207994,
        dt_tai_subsec: 185681999999999904,
    },
];

#[inline]
const fn unix_to_mjd(utc_sec: i64) -> Real {
    f!(40587.0) + f!(utc_sec) / f!(86400.0)
}

pub const fn historical_sofa_offset_from_unix(unix_secs: i64) -> Option<Real> {
    const FIRST_UNIX: i64 = -283996800;
    const CUTOFF_1972: i64 = 63072000;

    if unix_secs < FIRST_UNIX || unix_secs >= CUTOFF_1972 {
        return None;
    }

    let mjd = unix_to_mjd(unix_secs);

    let len = SOFA_TAI_UTC_PRE_1972.len();
    let mut i = len;
    while i > 0 {
        i -= 1;
        let entry = &SOFA_TAI_UTC_PRE_1972[i];

        if unix_secs >= entry.unix_sec {
            let offset = entry.offset + (mjd - entry.mjd_ref) * entry.drift;
            return Some(offset);
        }
    }

    None
}

/// Returns the SOFA historical TAI−UTC offset (in seconds) for a given TAI instant.
/// This is the inverse of `historical_sofa_from_unix`.
///
/// The offset is computed using the same piecewise linear formula as the forward direction:
/// `offset = entry.offset + (MJD − entry.mjd_ref) × entry.drift`
pub const fn historical_sofa_offset_from_tai(tai: &TimePoint) -> Option<Real> {
    if !matches!(tai.clock_type, ClockType::TAI) {
        return None;
    }

    // Get MJD of this TAI time (using your existing exact method)
    let (mjd_days, frac) = tai.to_mjd_tai_exact();
    let mjd = (mjd_days as Real) + (frac.as_sec_f() / SEC_PER_DAY);

    let len = SOFA_TAI_UTC_PRE_1972.len();
    let mut i = len;

    // Walk backwards from newest to oldest — first match is the correct interval
    while i > 0 {
        i -= 1;
        let entry = &SOFA_TAI_UTC_PRE_1972[i];

        // Is the given TAI time at/after this row's effective TAI instant?
        if tai.sec > entry.dt_tai_sec
            || (tai.sec == entry.dt_tai_sec && tai.subsec >= entry.dt_tai_subsec)
        {
            let offset = entry.offset + (mjd - entry.mjd_ref) * entry.drift;
            return Some(offset);
        }
    }

    None
}
