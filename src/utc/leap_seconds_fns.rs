//! [`Dt`](../struct.Dt.html) impls for leap-second lookup and UTC↔TAI conversion.
//!
//! Uses the built-in [`LEAP_SECS`] list or a caller-provided [`LeapSec`] slice.
//! [`LeapInfo`] is returned by [`Dt::leap_sec`](../struct.Dt.html#method.leap_sec) and related methods.

use crate::utc::leap_seconds_list::{LEAP_SECS, LeapSec};
use crate::{Dt, Scale};

#[cfg(feature = "std")]
use std::{fs, io, path::Path};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Indicates whether a queried instant falls exactly on a leap second transition.
///
/// This is returned by [`LeapInfo::is_leap_sec`] and is only ever set to a
/// non-`None` value when the queried timestamp is *exactly* at the moment
/// a leap second is inserted or removed.
///
/// - [`IsLeapSec::Add`] is returned for the inserted leap second
///   (e.g. `23:59:60`).
/// - [`IsLeapSec::Sub`] is returned for a negative (subtracted) leap second.
/// - [`IsLeapSec::None`] is returned for all normal seconds.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum IsLeapSec {
    /// This instant is **not** a leap second.
    #[default]
    None,
    /// This instant is a positive leap second (a second is being inserted).
    ///
    /// Example: `2015-06-30 23:59:60 UTC`.
    Add,
    /// This instant is a negative leap second (a second is being removed).
    ///
    /// Perhaps this would work by having clocks skip the 59th second of a
    /// minute, e.g. `2015-06-30 23:59:58 UTC` → `2015-07-01 00:00:00 UTC`.
    Sub,
}

/// Leap-second details for an instant, returned by [`Dt::leap_sec`](../struct.Dt.html#method.leap_sec)
/// and related methods.
///
/// ## See also
///
/// - [Dt::leap_sec](../struct.Dt.html#method.leap_sec)
/// - [Dt::leap_sec_using_list](../struct.Dt.html#method.leap_sec_using_list)
/// - [Dt::leap_sec_using_sec64](../struct.Dt.html#method.leap_sec_using_sec64)
/// - [Dt::leap_sec_using_sec64_and_list](../struct.Dt.html#method.leap_sec_using_sec64_and_list)
/// - [Dt::leap_sec_list_from_str](../struct.Dt.html#method.leap_sec_list_from_str)
/// - [Dt::leap_sec_list_from_file](../struct.Dt.html#method.leap_sec_list_from_file)
/// - [Dt::to_tai_from_utc_using_list](../struct.Dt.html#method.to_tai_from_utc_using_list)
/// - [Dt::to_utc_from_tai_using_list](../struct.Dt.html#method.to_utc_from_tai_using_list)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeapInfo {
    /// TAI minus UTC offset, in whole seconds.
    pub offset: i64,
    /// How many leap-second list entries come at or before the instant
    /// (`1` on 1972-01-01).
    pub n_entries_at_or_before: usize,
    /// Whether the queried instant is exactly at a leap second transition point.
    ///
    /// - [`IsLeapSec::Add`] — this is the inserted leap second (`23:59:60`).
    /// - [`IsLeapSec::Sub`] — this is a negative (removed) leap second.
    /// - [`IsLeapSec::None`] — normal second (most common).
    pub is_leap_sec: IsLeapSec,
}

impl Dt {
    /// Get [`LeapInfo`] for a particular library timestamp (seconds from 2000-01-01 noon TAI)
    /// using a provided leap seconds list.
    ///
    /// - If the timestamp is currently on the UTC time scale then use **`true`** for the `is_utc`
    ///   parameter.
    /// - If the timestamp is currently on the TAI time scale then use **`false`** for the `is_utc`
    ///   parameter.
    /// - `sec64` should be such that it was produced using euclid division, see
    ///   [`Dt::to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) for more info. This only applies to
    ///   negative `sec64` values.
    ///
    /// ## See also
    ///
    /// For more information on how to make a leap seconds list, see the following functions:
    ///
    /// - [Dt::leap_sec_list_from_str](../struct.Dt.html#method.leap_sec_list_from_str)
    /// - [Dt::leap_sec_list_from_file](../struct.Dt.html#method.leap_sec_list_from_file)
    /// - [Dt::to_tai_from_utc_using_list](../struct.Dt.html#method.to_tai_from_utc_using_list)
    /// - [Dt::to_utc_from_tai_using_list](../struct.Dt.html#method.to_utc_from_tai_using_list)
    pub const fn leap_sec_using_sec64_and_list(
        sec64: i64,
        is_utc: bool,
        list: &[LeapSec],
    ) -> Option<LeapInfo> {
        let len = list.len();
        if len == 0 {
            return None;
        }

        // Binary search for upper_bound: first index where entry_sec > sec64
        let mut low = 0usize;
        let mut high = len;
        if is_utc {
            while low < high {
                let mid = low + (high - low) / 2;
                if list[mid].utc_sec <= sec64 {
                    low = mid + 1;
                } else {
                    high = mid;
                }
            }
        } else {
            while low < high {
                let mid = low + (high - low) / 2;
                if list[mid].tai_sec <= sec64 {
                    low = mid + 1;
                } else {
                    high = mid;
                }
            }
        }

        // low == first index with entry_sec > sec64 (or len)
        if low == 0 {
            return None;
        }

        let idx = low - 1;
        let entry = &list[idx];
        let is_leap = {
            if sec64 != if is_utc { entry.utc_sec } else { entry.tai_sec } {
                IsLeapSec::None
            } else if idx != 0 {
                let prev_leap_sec_after = list[idx - 1].leap_sec_after;
                if entry.leap_sec_after > prev_leap_sec_after {
                    IsLeapSec::Add
                } else if entry.leap_sec_after < prev_leap_sec_after {
                    IsLeapSec::Sub
                } else {
                    IsLeapSec::None
                }
            } else if entry.leap_sec_after > 0 {
                IsLeapSec::Add
            } else if entry.leap_sec_after < 0 {
                IsLeapSec::Sub
            } else {
                IsLeapSec::None
            }
        };

        Some(LeapInfo {
            offset: entry.leap_sec_after,
            n_entries_at_or_before: low,
            is_leap_sec: is_leap,
        })
    }

    /// Get [`LeapInfo`] for a particular library timestamp (seconds from 2000-01-01 noon TAI)
    /// using the library's in-built leap seconds list.
    ///
    /// - If the timestamp is currently on the UTC time scale then use **`true`** for the `is_utc`
    ///   parameter.
    /// - If the timestamp is currently on the TAI time scale then use **`false`** for the `is_utc`
    ///   parameter.
    /// - `sec64` should be such that it was produced using euclid division, see
    ///   [`Dt::to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) for more info. This only applies to
    ///   negative `sec64` values.
    #[inline(always)]
    pub const fn leap_sec_using_sec64(sec64: i64, is_utc: bool) -> Option<LeapInfo> {
        Self::leap_sec_using_sec64_and_list(sec64, is_utc, LEAP_SECS)
    }

    /// Get the leap seconds info for this instant.
    ///
    /// Uses the library's in-built leap seconds list.
    #[inline(always)]
    pub const fn leap_sec(&self, is_utc: bool) -> Option<LeapInfo> {
        Self::leap_sec_using_sec64_and_list(self.to_sec64_floor(), is_utc, LEAP_SECS)
    }

    /// Get the leap seconds info for this instant with a given list.
    #[inline(always)]
    pub const fn leap_sec_using_list(&self, is_utc: bool, list: &[LeapSec]) -> Option<LeapInfo> {
        Self::leap_sec_using_sec64_and_list(self.to_sec64_floor(), is_utc, list)
    }

    #[inline(always)]
    pub(crate) const fn utc_to_tai_using_list(&self, list: &[LeapSec]) -> Option<Dt> {
        match self.leap_sec_using_list(true, list) {
            Some(info) => Some(self.add_sec(info.offset as i128)),
            None => None,
        }
    }

    #[inline(always)]
    pub(crate) const fn tai_to_utc_using_list(&self, list: &[LeapSec]) -> Option<Dt> {
        match self.leap_sec_using_list(false, list) {
            Some(info) => Some(self.add_sec(-info.offset as i128)),
            None => None,
        }
    }

    /// Converts **UTC -> TAI** using a provided Leap seconds list.
    ///
    /// - If the
    ///   [`Dt`](../struct.Dt.html) is before the provided leap second list's
    ///   first entry then the library's own conversion is used to convert to
    ///   [`Scale::TAI`](../enum.Scale.html#variant.TAI)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use deep_time::utc::{IsLeapSec, LEAP_SECS};
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_list =
    ///     Dt::leap_sec_list_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_list[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_sec = x.leap_sec_using_list(false, &leap_seconds_list).unwrap();
    /// assert!(leap_sec.is_leap_sec == IsLeapSec::Add);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_list(Scale::UTC, &leap_seconds_list);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_list(&leap_seconds_list);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_list_from_str](../struct.Dt.html#method.leap_sec_list_from_str)
    /// - [Dt::leap_sec_list_from_file](../struct.Dt.html#method.leap_sec_list_from_file)
    /// - [Dt::to_utc_from_tai_using_list](../struct.Dt.html#method.to_utc_from_tai_using_list)
    pub const fn to_tai_from_utc_using_list(&self, list: &[LeapSec]) -> Dt {
        match self.scale {
            // we're going utc -> tai, check if it's
            // post start of list using the provided leap seconds list
            Scale::UTC | Scale::UtcHist | Scale::UtcSpice => {
                match self.utc_to_tai_using_list(list) {
                    // leap seconds list returned an offset, so use that
                    Some(dt) => dt.with(Scale::TAI),
                    // leap seconds list returned None so it must be pre 1972
                    None => match self.scale {
                        Scale::UtcHist => match self.historical_utc_offset() {
                            Some(offset) => self.add(Dt::span_f(offset)).with(Scale::TAI),
                            None => self.with(Scale::TAI),
                        },
                        Scale::UtcSpice => self.add_sec(9).with(Scale::TAI),
                        _ => self.with(Scale::TAI),
                    },
                }
            }
            // defer to library conversion function
            _ => self.to(Scale::TAI),
        }
    }

    /// Converts **TAI -> UTC** using a provided Leap seconds list.
    ///
    /// - If `new` is
    ///   [`Scale::UtcHist`](../enum.Scale.html#variant.UtcHist) or
    ///   [`Scale::UtcSpice`](../enum.Scale.html#variant.UtcSpice) and the
    ///   [`Dt`](../struct.Dt.html) is before the provided leap second list's
    ///   first entry then the library's own conversion is used to convert to
    ///   `new`.
    /// - If `new` is not one of the scales that uses leap seconds then the library's
    ///   own conversion is used to convert to `new`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use deep_time::utc::{IsLeapSec, LEAP_SECS};
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_list =
    ///     Dt::leap_sec_list_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_list[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_sec = x.leap_sec_using_list(false, &leap_seconds_list).unwrap();
    /// assert!(leap_sec.is_leap_sec == IsLeapSec::Add);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_list(Scale::UTC, &leap_seconds_list);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_list(&leap_seconds_list);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_list_from_str](../struct.Dt.html#method.leap_sec_list_from_str)
    /// - [Dt::leap_sec_list_from_file](../struct.Dt.html#method.leap_sec_list_from_file)
    /// - [Dt::to_tai_from_utc_using_list](../struct.Dt.html#method.to_tai_from_utc_using_list)
    pub const fn to_utc_from_tai_using_list(&self, new: Scale, list: &[LeapSec]) -> Dt {
        match new {
            Scale::UTC | Scale::UtcHist | Scale::UtcSpice => {
                match self.tai_to_utc_using_list(list) {
                    // leap seconds list returned an offset, so use that
                    Some(dt) => dt.with(new),
                    // leap seconds list returned None so it must be pre 1972
                    None => match new {
                        Scale::UtcHist => match self.historical_utc_offset() {
                            Some(offset) => self.sub(Dt::span_f(offset)).with(new),
                            None => self.with(new),
                        },
                        Scale::UtcSpice => self.add_sec(-9).with(new),
                        _ => self.with(new),
                    },
                }
            }
            // defer to library conversion function
            _ => self.to(new),
        }
    }
}

#[cfg(feature = "alloc")]
impl Dt {
    /// Load directly from a str (e.g. the official IANA `leap-seconds.list`).
    ///
    /// Format should be the same as the file available at:
    /// <https://data.iana.org/time-zones/data/leap-seconds.list>
    ///
    /// For rows that don't start with # (the list rows) the first column
    /// should be the NTP timestamp, the second column (separated by whitespace)
    /// should be the offset against TAI in seconds (the number of leap seconds at
    /// that point).
    ///
    /// e.g.
    ///
    /// | #NTP Time  |    DTAI  |
    /// |------------|----------|
    /// | #          |          |
    /// | 2272060800 |     10   |
    /// | 2287785600 |     11   |
    /// | 2303683200 |     12   |
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_list_from_file](../struct.Dt.html#method.leap_sec_list_from_file)
    /// - [Dt::to_utc_from_tai_using_list](../struct.Dt.html#method.to_utc_from_tai_using_list)
    /// - [Dt::to_tai_from_utc_using_list](../struct.Dt.html#method.to_tai_from_utc_using_list)
    pub fn leap_sec_list_from_str(s: &str) -> Vec<LeapSec> {
        use crate::Scale;

        let mut list = Vec::new();
        let mut prev_leap_sec_after: i64 = 0;
        let mut entries_pushed: usize = 0;

        for line in s.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("#") {
                continue;
            }

            let mut parts = trimmed.split_whitespace();

            let ntp_timestamp = if let Some(num) = parts.next() {
                match num.parse::<i64>() {
                    Ok(n) => n,
                    Err(_) => continue,
                }
            } else {
                continue;
            };
            let leap_sec_after = if let Some(num) = parts.next() {
                match num.parse::<i64>() {
                    Ok(n) => n,
                    Err(_) => continue,
                }
            } else {
                continue;
            };

            // don't use current: UTC because it would use the internal leap list
            let utc_sec =
                Dt::from_ntp(Dt::from_sec(ntp_timestamp as i128, Scale::TAI)).to_sec64_floor();

            let tai_sec = if entries_pushed == 0 {
                if leap_sec_after > 0 {
                    utc_sec + leap_sec_after - 1
                } else {
                    // hypothetical negative first entry
                    utc_sec + leap_sec_after
                }
            } else {
                utc_sec + prev_leap_sec_after
            };

            list.push(LeapSec {
                ntp_timestamp,
                leap_sec_after,
                utc_sec,
                tai_sec,
            });

            prev_leap_sec_after = leap_sec_after;
            entries_pushed += 1;
        }

        list
    }
}

#[cfg(feature = "std")]
impl Dt {
    /// Load directly from a file (e.g. the official IANA `leap-seconds.list`).
    ///
    /// Format should be the same as the file available at:
    /// <https://data.iana.org/time-zones/data/leap-seconds.list>
    ///
    /// For rows that don't start with # (the list rows) the first column
    /// should be the NTP timestamp, the second column (separated by whitespace)
    /// should be the offset against TAI in seconds (the number of leap seconds at
    /// that point).
    ///
    /// e.g.
    ///
    /// | #NTP Time  |    DTAI  |
    /// |------------|----------|
    /// | #          |          |
    /// | 2272060800 |     10   |
    /// | 2287785600 |     11   |
    /// | 2303683200 |     12   |
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use deep_time::utc::{IsLeapSec, LEAP_SECS};
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_list =
    ///     Dt::leap_sec_list_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_list[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_sec = x.leap_sec_using_list(false, &leap_seconds_list).unwrap();
    /// assert!(leap_sec.is_leap_sec == IsLeapSec::Add);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_list(Scale::UTC, &leap_seconds_list);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_list(&leap_seconds_list);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_list_from_str](../struct.Dt.html#method.leap_sec_list_from_str)
    /// - [Dt::to_utc_from_tai_using_list](../struct.Dt.html#method.to_utc_from_tai_using_list)
    /// - [Dt::to_tai_from_utc_using_list](../struct.Dt.html#method.to_tai_from_utc_using_list)
    #[inline]
    pub fn leap_sec_list_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<LeapSec>> {
        let content = fs::read_to_string(path)?;
        Ok(Self::leap_sec_list_from_str(&content))
    }
}
