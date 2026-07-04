//! UTC time-scale data and TAI−UTC conversion tables.
//!
//! [`Dt`](../struct.Dt.html) handles leap-second lookup, custom leap-second lists, and
//! pre-1972 historical offsets. Import this module for the embedded tables ([`LEAP_SECS`],
//! [`UTC_HIST_TABLE`]), their row types ([`LeapSec`], [`UtcHistRow`]), and [`LeapInfo`]
//! (returned by [`Dt::leap_sec`](../struct.Dt.html#method.leap_sec)).
//!
//! Pre-1972 UTC uses piecewise-linear offsets; from 1972 onward, discrete leap seconds apply.

mod historical;
mod leap_seconds_fns;
mod leap_seconds_list;

pub use historical::{UTC_HIST_TABLE, UtcHistRow};
pub use leap_seconds_fns::{IsLeapSec, LeapInfo};
pub use leap_seconds_list::{LEAP_SECS, LeapSec};
