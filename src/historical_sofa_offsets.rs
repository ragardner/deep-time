//! Historical TAI − UTC offsets (ΔAT) for the "rubber second" era
//! (1 January 1960 – 31 December 1971).
//!
//! This table contains the effective constant offset between International
//! Atomic Time (TAI) and Coordinated Universal Time (UTC) that was in effect
//! during the period before the modern leap-second system was introduced on
//! 1 January 1972.
//!
//! These values are taken directly from the official historical model used by
//! the IAU Standards of Fundamental Astronomy (SOFA) library in its `iauDat`
//! function (implemented in `dat.c`). They are also published in the U.S.
//! Naval Observatory file `tai-utc.dat`.
//!
//! ## References
//!
//! - USNO `tai-utc.dat` (primary source):
//!   https://maia.usno.navy.mil/ser7/tai-utc.dat
//!
//! - IAU SOFA `dat.c` (official implementation):
//!   https://github.com/liberfa/erfa/blob/master/src/dat.c
//!
//! - IERS Conventions (historical context):
//!   IERS Technical Note No. 32 (2004) and subsequent updates.
//!
//! The table is sorted by date. Each entry gives the TAI − UTC offset (in
//! seconds) that applies **on and after** that date until the next entry
//! (or until 1972-01-01, at which point the modern IERS integer leap-second
//! system begins and the offset becomes exactly 10.0 s).
//!
//! These values are **not** used by the library’s normal conversion functions
//! (`to_tai`, `from_tai`, `from_gregorian_ymdhms`, etc.). They exist solely
//! to support the historical reconstruction functions
//! `from_historical_gregorian_ymdhms` and `to_historical_utc`.

use crate::Real;

pub const HISTORICAL_TAI_UTC_OFFSETS: &[(i32, u8, u8, Real)] = &[
    (1960, 1, 1, f!(1.4178180)),
    (1961, 1, 1, f!(1.4228180)),
    (1961, 8, 1, f!(1.3728180)),
    (1962, 1, 1, f!(1.8458580)),
    (1963, 11, 1, f!(1.9458580)),
    (1964, 1, 1, f!(3.2401300)),
    (1964, 4, 1, f!(3.3401300)),
    (1964, 9, 1, f!(3.4401300)),
    (1965, 1, 1, f!(3.5401300)),
    (1965, 3, 1, f!(3.6401300)),
    (1965, 7, 1, f!(3.7401300)),
    (1965, 9, 1, f!(3.8401300)),
    (1966, 1, 1, f!(4.3131700)),
    (1968, 2, 1, f!(4.2131700)),
    (1972, 1, 1, f!(10.0000000)), // final value — matches start of modern table
];

/// Returns the TAI − UTC offset (in seconds) that was historically in effect
/// on the given proleptic Gregorian date according to the SOFA model.
///
/// Returns `None` if the date is before 1960-01-01 **or on/after 1972-01-01**.
pub const fn historical_tai_minus_utc_offset(year: i32, month: u8, day: u8) -> Option<Real> {
    // Explicit guard for the modern era
    if year >= 1972 {
        return None;
    }

    let mut i = 0;
    while i < HISTORICAL_TAI_UTC_OFFSETS.len() {
        let (y, m, d, offset) = HISTORICAL_TAI_UTC_OFFSETS[i];

        if year < y {
            return None;
        }
        if year > y {
            i += 1;
            continue;
        }
        if month < m {
            return None;
        }
        if month > m {
            i += 1;
            continue;
        }
        if day < d {
            return None;
        }
        return Some(offset);
    }
    None
}
