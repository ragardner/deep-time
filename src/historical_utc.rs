//! Pre-1972 TAI−UTC historical offsets and linear drift rates
//! from the official USNO `tai-utc.dat` (used by IAU SOFA).
//!
//! Provides the piecewise-linear formula for UTC instants before 1972-01-01.

use crate::{Dt, Real};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaiUtcPre1972 {
    /// Year of the effective UTC date
    pub yr: i32,
    /// Month (1-12) of the effective UTC date
    pub mo: u8,
    /// Day of the effective UTC date
    pub day: u8,
    /// Julian Date (JD) at 0h UT on the effective date (start of interval)
    pub jd: Real,
    /// Reference MJD used in the linear drift formula: offset + (MJD - mjd_ref) * drift
    pub mjd_ref: Real,
    /// Constant offset (seconds) in the TAI−UTC formula
    pub offset: Real,
    /// Drift rate (seconds per day) — this is the historical frequency offset effect
    pub drift: Real,
    // jd for the date, includes computed and added sofa offset
    // pub jd_added: Real,
    // jd for the date, includes computed and subtracted sofa offset
    // pub jd_subbed: Real,
    // /// tai mjd for the date, includes calculated sofa offset
    // pub tai_mjd: Real,
}

/// Authoritative pre-1972 TAI−UTC entries from the official USNO `tai-utc.dat`
/// [file](https://maia.usno.navy.mil/ser7/tai-utc.dat).  
/// These 13 intervals contain the frequency offsets and linear drifts used by the IAU SOFA library
/// and all other high-precision astronomy/time-conversion software before UTC switched to pure leap-second mode.
pub const TAI_UTC_PRE_1972: &[TaiUtcPre1972] = &[
    TaiUtcPre1972 {
        yr: 1961,
        mo: 1,
        day: 1,
        jd: 2437300.5,
        mjd_ref: 37300.0,
        offset: 1.4228180,
        drift: 0.001296,
        // jd_added: 2437300.5000164676,
        // jd_subbed: 2437300.4999835324,
        // tai_mjd: 37300.0000164678,
    },
    TaiUtcPre1972 {
        yr: 1961,
        mo: 8,
        day: 1,
        jd: 2437512.5,
        mjd_ref: 37300.0,
        offset: 1.3728180,
        drift: 0.001296,
        // jd_added: 2437512.5000190693,
        // jd_subbed: 2437512.4999809307,
        // tai_mjd: 37512.0000190691,
    },
    TaiUtcPre1972 {
        yr: 1962,
        mo: 1,
        day: 1,
        jd: 2437665.5,
        mjd_ref: 37665.0,
        offset: 1.8458580,
        drift: 0.0011232,
        // jd_added: 2437665.500021364,
        // jd_subbed: 2437665.499978636,
        // tai_mjd: 37665.000021364096,
    },
    TaiUtcPre1972 {
        yr: 1963,
        mo: 11,
        day: 1,
        jd: 2438334.5,
        mjd_ref: 37665.0,
        offset: 1.9458580,
        drift: 0.0011232,
        // jd_added: 2438334.5000312184,
        // jd_subbed: 2438334.4999687816,
        // tai_mjd: 38334.00003121851,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 1,
        day: 1,
        jd: 2438395.5,
        mjd_ref: 38761.0,
        offset: 3.2401300,
        drift: 0.001296,
        // jd_added: 2438395.5000320114,
        // jd_subbed: 2438395.4999679886,
        // tai_mjd: 38395.00003201151,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 4,
        day: 1,
        jd: 2438486.5,
        mjd_ref: 38761.0,
        offset: 3.3401300,
        drift: 0.001296,
        // jd_added: 2438486.500034534,
        // jd_subbed: 2438486.499965466,
        // tai_mjd: 38486.000034533914,
    },
    TaiUtcPre1972 {
        yr: 1964,
        mo: 9,
        day: 1,
        jd: 2438639.5,
        mjd_ref: 38761.0,
        offset: 3.4401300,
        drift: 0.001296,
        // jd_added: 2438639.5000379863,
        // jd_subbed: 2438639.4999620137,
        // tai_mjd: 38639.00003798632,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 1,
        day: 1,
        jd: 2438761.5,
        mjd_ref: 38761.0,
        offset: 3.5401300,
        drift: 0.001296,
        // jd_added: 2438761.5000409735,
        // jd_subbed: 2438761.4999590265,
        // tai_mjd: 38761.000040973726,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 3,
        day: 1,
        jd: 2438820.5,
        mjd_ref: 38761.0,
        offset: 3.6401300,
        drift: 0.001296,
        // jd_added: 2438820.500043016,
        // jd_subbed: 2438820.499956984,
        // tai_mjd: 38820.00004301613,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 7,
        day: 1,
        jd: 2438942.5,
        mjd_ref: 38761.0,
        offset: 3.7401300,
        drift: 0.001296,
        // jd_added: 2438942.5000460036,
        // jd_subbed: 2438942.4999539964,
        // tai_mjd: 38942.000046003544,
    },
    TaiUtcPre1972 {
        yr: 1965,
        mo: 9,
        day: 1,
        jd: 2439004.5,
        mjd_ref: 38761.0,
        offset: 3.8401300,
        drift: 0.001296,
        // jd_added: 2439004.500048091,
        // jd_subbed: 2439004.499951909,
        // tai_mjd: 39004.00004809095,
    },
    TaiUtcPre1972 {
        yr: 1966,
        mo: 1,
        day: 1,
        jd: 2439126.5,
        mjd_ref: 39126.0,
        offset: 4.3131700,
        drift: 0.002592,
        // jd_added: 2439126.5000499208,
        // jd_subbed: 2439126.4999500792,
        // tai_mjd: 39126.00004992095,
    },
    TaiUtcPre1972 {
        yr: 1968,
        mo: 2,
        day: 1,
        jd: 2439887.5,
        mjd_ref: 39126.0,
        offset: 4.2131700,
        drift: 0.002592,
        // jd_added: 2439887.5000715936,
        // jd_subbed: 2439887.4999284064,
        // tai_mjd: 39887.00007159354,
    },
];

/// Returns the SOFA historical TAI−UTC offset (in seconds)
/// for an **un-adjusted instant**.
///
/// - Only for instants that have not already been offset
///   by a historical UTC offset value.
/// - Not as accurate as ERFA / Astropy.
/// - **Do not use this for round tripping.**
/// - Unlike ERFA it does not support dates between 1960
///   and 1961.
pub const fn historical_utc_offset(dt: &Dt) -> Option<Real> {
    // < 1961-1-1 midnight, or >= 1972-1-1 midnight
    if dt.to_attos() < -1230724800000000000000000000
        // tai attos for 1972 (10 leap seconds added)
        || dt.to_attos() >= -883655990000000000000000000
    {
        return None;
    }
    // if dt.to_attos() >= -883656000000000000000000000 {
    //     return None;
    // }

    let jd = dt.to_jd_f();
    let mjd = dt.to_mjd_f();
    let len = TAI_UTC_PRE_1972.len();
    let mut i = len;
    while i > 0 {
        i -= 1;
        let entry = &TAI_UTC_PRE_1972[i];
        if jd >= entry.jd {
            let offset = entry.offset + (mjd - entry.mjd_ref) * entry.drift;
            return Some(offset);
        }
    }

    None
}
