//! Leap seconds table from the official IANA
//! [leap-seconds.list](https://data.iana.org/time-zones/data/leap-seconds.list)
//! Updated through IERS Bulletin C as of April 2026.
//! Last leap second: 2017-01-01 (TAI-UTC = 37 s)
//! File expires: 28 December 2026

use crate::{Dt, Scale, historical_utc::historical_utc_offset};

#[cfg(feature = "std")]
use std::{fs, io, path::Path};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LeapSec {
    pub ntp_timestamp: i64,
    pub leap_sec_after: i64,
    pub utc_sec: i64,
    pub tai_sec: i64,
}

pub const LEAP_SECS: &[LeapSec] = &[
    LeapSec {
        ntp_timestamp: 2272060800,
        leap_sec_after: 10,
        utc_sec: -883656000,
        tai_sec: -883655991, // was -883655990
    }, // 1 Jan 1972 (start of modern UTC definition)
    LeapSec {
        ntp_timestamp: 2287785600,
        leap_sec_after: 11,
        utc_sec: -867931200,
        tai_sec: -867931190, // was -867931189
    }, // 1 Jul 1972
    LeapSec {
        ntp_timestamp: 2303683200,
        leap_sec_after: 12,
        utc_sec: -852033600,
        tai_sec: -852033589, // was -852033588
    }, // 1 Jan 1973
    LeapSec {
        ntp_timestamp: 2335219200,
        leap_sec_after: 13,
        utc_sec: -820497600,
        tai_sec: -820497588, // was -820497587
    }, // 1 Jan 1974
    LeapSec {
        ntp_timestamp: 2366755200,
        leap_sec_after: 14,
        utc_sec: -788961600,
        tai_sec: -788961587, // was -788961586
    }, // 1 Jan 1975
    LeapSec {
        ntp_timestamp: 2398291200,
        leap_sec_after: 15,
        utc_sec: -757425600,
        tai_sec: -757425586, // was -757425585
    }, // 1 Jan 1976
    LeapSec {
        ntp_timestamp: 2429913600,
        leap_sec_after: 16,
        utc_sec: -725803200,
        tai_sec: -725803185, // was -725803184
    }, // 1 Jan 1977
    LeapSec {
        ntp_timestamp: 2461449600,
        leap_sec_after: 17,
        utc_sec: -694267200,
        tai_sec: -694267184, // was -694267183
    }, // 1 Jan 1978
    LeapSec {
        ntp_timestamp: 2492985600,
        leap_sec_after: 18,
        utc_sec: -662731200,
        tai_sec: -662731183, // was -662731182
    }, // 1 Jan 1979
    LeapSec {
        ntp_timestamp: 2524521600,
        leap_sec_after: 19,
        utc_sec: -631195200,
        tai_sec: -631195182, // was -631195181
    }, // 1 Jan 1980
    LeapSec {
        ntp_timestamp: 2571782400,
        leap_sec_after: 20,
        utc_sec: -583934400,
        tai_sec: -583934381, // was -583934380
    }, // 1 Jul 1981
    LeapSec {
        ntp_timestamp: 2603318400,
        leap_sec_after: 21,
        utc_sec: -552398400,
        tai_sec: -552398380, // was -552398379
    }, // 1 Jul 1982
    LeapSec {
        ntp_timestamp: 2634854400,
        leap_sec_after: 22,
        utc_sec: -520862400,
        tai_sec: -520862379, // was -520862378
    }, // 1 Jul 1983
    LeapSec {
        ntp_timestamp: 2698012800,
        leap_sec_after: 23,
        utc_sec: -457704000,
        tai_sec: -457703978, // was -457703977
    }, // 1 Jul 1985
    LeapSec {
        ntp_timestamp: 2776982400,
        leap_sec_after: 24,
        utc_sec: -378734400,
        tai_sec: -378734377, // was -378734376
    }, // 1 Jan 1988
    LeapSec {
        ntp_timestamp: 2840140800,
        leap_sec_after: 25,
        utc_sec: -315576000,
        tai_sec: -315575976, // was -315575975
    }, // 1 Jan 1990
    LeapSec {
        ntp_timestamp: 2871676800,
        leap_sec_after: 26,
        utc_sec: -284040000,
        tai_sec: -284039975, // was -284039974
    }, // 1 Jan 1991
    LeapSec {
        ntp_timestamp: 2918937600,
        leap_sec_after: 27,
        utc_sec: -236779200,
        tai_sec: -236779174, // was -236779173
    }, // 1 Jul 1992
    LeapSec {
        ntp_timestamp: 2950473600,
        leap_sec_after: 28,
        utc_sec: -205243200,
        tai_sec: -205243173, // was -205243172
    }, // 1 Jul 1993
    LeapSec {
        ntp_timestamp: 2982009600,
        leap_sec_after: 29,
        utc_sec: -173707200,
        tai_sec: -173707172, // was -173707171
    }, // 1 Jul 1994
    LeapSec {
        ntp_timestamp: 3029443200,
        leap_sec_after: 30,
        utc_sec: -126273600,
        tai_sec: -126273571, // was -126273570
    }, // 1 Jan 1996
    LeapSec {
        ntp_timestamp: 3076704000,
        leap_sec_after: 31,
        utc_sec: -79012800,
        tai_sec: -79012770, // was -79012769
    }, // 1 Jul 1997
    LeapSec {
        ntp_timestamp: 3124137600,
        leap_sec_after: 32,
        utc_sec: -31579200,
        tai_sec: -31579169, // was -31579168
    }, // 1 Jan 1999
    LeapSec {
        ntp_timestamp: 3345062400,
        leap_sec_after: 33,
        utc_sec: 189345600,
        tai_sec: 189345632, // was 189345633
    }, // 1 Jan 2006
    LeapSec {
        ntp_timestamp: 3439756800,
        leap_sec_after: 34,
        utc_sec: 284040000,
        tai_sec: 284040033, // was 284040034
    }, // 1 Jan 2009
    LeapSec {
        ntp_timestamp: 3550089600,
        leap_sec_after: 35,
        utc_sec: 394372800,
        tai_sec: 394372834, // was 394372835
    }, // 1 Jul 2012
    LeapSec {
        ntp_timestamp: 3644697600,
        leap_sec_after: 36,
        utc_sec: 488980800,
        tai_sec: 488980835, // was 488980836
    }, // 1 Jul 2015
    LeapSec {
        ntp_timestamp: 3692217600,
        leap_sec_after: 37,
        utc_sec: 536500800,
        tai_sec: 536500836, // was 536500837
    }, // 1 Jan 2017
];

#[derive(Copy, Clone, Debug)]
pub struct LeapInfo {
    pub offset: i64,
    pub leaps_inserted: i64,
    pub is_leap_sec: bool,
}

impl Dt {
    /// Get the leap seconds info for this instant.
    ///
    /// Uses the library's in-built leap seconds table.
    #[inline(always)]
    pub const fn leap_sec(&self, is_utc: bool) -> Option<LeapInfo> {
        leap_sec(self.to_sec64(), is_utc)
    }

    /// Get the leap seconds info for this instant with a given table.
    #[inline(always)]
    pub const fn leap_sec_using(&self, is_utc: bool, table: &[LeapSec]) -> Option<LeapInfo> {
        leap_sec_using(self.to_sec64(), is_utc, table)
    }

    #[inline(always)]
    pub(crate) const fn utc_to_tai_using_leaps(&self, data: &[LeapSec]) -> Option<Dt> {
        match self.leap_sec_using(true, data) {
            Some(info) => Some(self.add_sec(info.offset as i128)),
            None => None,
        }
    }

    #[inline(always)]
    pub(crate) const fn tai_to_utc_using_leaps(&self, data: &[LeapSec]) -> Option<Dt> {
        match self.leap_sec_using(false, data) {
            Some(info) => Some(self.add_sec(-info.offset as i128)),
            None => None,
        }
    }

    /// Converts **UTC -> TAI** using a provided Leap seconds table.
    ///
    /// - If the
    ///   [`Dt`](../struct.Dt.html) is before the provided leap second table's
    ///   first entry then the library's own conversion is used to convert to
    ///   [`Scale::TAI`](../enum.Scale.html#variant.TAI)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use deep_time::leap_seconds::LEAP_SECS;
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_table =
    ///     Dt::leap_sec_data_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_table[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_info = Dt::leap_sec_using(&x, false, &leap_seconds_table).unwrap();
    /// assert!(leap_info.is_leap_sec == true);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_leaps(Scale::UTC, &leap_seconds_table);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_leaps(&leap_seconds_table);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_data_from_str](../struct.Dt.html#method.leap_sec_data_from_str)
    /// - [Dt::leap_sec_data_from_file](../struct.Dt.html#method.leap_sec_data_from_file)
    /// - [Dt::to_utc_from_tai_using_leaps](../struct.Dt.html#method.to_utc_from_tai_using_leaps)
    #[inline(always)]
    pub const fn to_tai_from_utc_using_leaps(&self, data: &[LeapSec]) -> Dt {
        match self.scale {
            // we're going utc -> tai, check if it's
            // post start of table using the provided leap seconds table
            Scale::UTC | Scale::UtcHist | Scale::UtcSpice => {
                match self.utc_to_tai_using_leaps(&data) {
                    // leap seconds table returned an offset, so use that
                    Some(dt) => dt.with(Scale::TAI),
                    // leap seconds table returned None so it must be pre 1972
                    None => match self.scale {
                        Scale::UtcHist => match historical_utc_offset(self) {
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

    /// Converts **TAI -> UTC** using a provided Leap seconds table.
    ///
    /// - If `new` is
    ///   [`Scale::UtcHist`](../enum.Scale.html#variant.UtcHist) or
    ///   [`Scale::UtcSpice`](../enum.Scale.html#variant.UtcSpice) and the
    ///   [`Dt`](../struct.Dt.html) is before the provided leap second table's
    ///   first entry then the library's own conversion is used to convert to
    ///   `new`.
    /// - If `new` is not one of the scales that uses leap seconds then the library's
    ///   own conversion is used to convert to `new`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")] {
    /// use deep_time::leap_seconds::LEAP_SECS;
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_table =
    ///     Dt::leap_sec_data_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_table[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_info = Dt::leap_sec_using(&x, false, &leap_seconds_table).unwrap();
    /// assert!(leap_info.is_leap_sec == true);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_leaps(Scale::UTC, &leap_seconds_table);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_leaps(&leap_seconds_table);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_data_from_str](../struct.Dt.html#method.leap_sec_data_from_str)
    /// - [Dt::leap_sec_data_from_file](../struct.Dt.html#method.leap_sec_data_from_file)
    /// - [Dt::to_tai_from_utc_using_leaps](../struct.Dt.html#method.to_tai_from_utc_using_leaps)
    #[inline(always)]
    pub const fn to_utc_from_tai_using_leaps(&self, new: Scale, data: &[LeapSec]) -> Dt {
        match new {
            Scale::UTC | Scale::UtcHist | Scale::UtcSpice => {
                match self.tai_to_utc_using_leaps(data) {
                    // leap seconds table returned an offset, so use that
                    Some(dt) => dt.with(new),
                    // leap seconds table returned None so it must be pre 1972
                    None => match new {
                        Scale::UtcHist => match historical_utc_offset(self) {
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

/// Get [`LeapInfo`] for a particular library timestamp (seconds from 2000-01-01 noon TAI)
/// using the library's internal leap seconds table.
///
/// - If the timestamp is currently on the UTC time scale then use **`true`** for the `is_utc`
///   parameter.
/// - If the timestamp is currently on the TAI time scale then use **`false`** for the `is_utc`
///   parameter.
/// - `target` should be such that it was produced using euclid division, see
///   [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64) for more info. This only applies to
///   negative `target` values.
#[inline(always)]
pub const fn leap_sec(target: i64, is_utc: bool) -> Option<LeapInfo> {
    leap_sec_using(target, is_utc, LEAP_SECS)
}

/// Get [`LeapInfo`] for a particular library timestamp (seconds from 2000-01-01 noon TAI)
/// using a provided leap seconds table.
///
/// - If the timestamp is currently on the UTC time scale then use **`true`** for the `is_utc`
///   parameter.
/// - If the timestamp is currently on the TAI time scale then use **`false`** for the `is_utc`
///   parameter.
/// - `target` should be such that it was produced using euclid division, see
///   [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64) for more info. This only applies to
///   negative `target` values.
///
/// ## See also
///
/// For more information on how to make a leap seconds table, see the following functions:
///
/// - [Dt::leap_sec_data_from_str](../struct.Dt.html#method.leap_sec_data_from_str)
/// - [Dt::leap_sec_data_from_file](../struct.Dt.html#method.leap_sec_data_from_file)
/// - [Dt::to_tai_from_utc_using_leaps](../struct.Dt.html#method.to_tai_from_utc_using_leaps)
/// - [Dt::to_utc_from_tai_using_leaps](../struct.Dt.html#method.to_utc_from_tai_using_leaps)
pub const fn leap_sec_using(target: i64, is_utc: bool, table: &[LeapSec]) -> Option<LeapInfo> {
    let len = table.len();
    if len == 0 {
        return None;
    }

    // Binary search for upper_bound: first index where entry_sec > target
    let mut low = 0usize;
    let mut high = len;
    if is_utc {
        while low < high {
            let mid = low + (high - low) / 2;
            if table[mid].utc_sec <= target {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
    } else {
        while low < high {
            let mid = low + (high - low) / 2;
            if table[mid].tai_sec <= target {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
    }

    // low == first index with entry_sec > target (or len)
    if low == 0 {
        return None;
    }

    let idx = low - 1;
    let entry = &table[idx];
    let entry_sec = if is_utc { entry.utc_sec } else { entry.tai_sec };
    let is_leap = target == entry_sec;

    Some(LeapInfo {
        offset: entry.leap_sec_after,
        leaps_inserted: low as i64,
        is_leap_sec: is_leap,
    })
}

#[cfg(feature = "std")]
impl Dt {
    /// Load directly from a file (e.g. the official IANA `leap-seconds.list`).
    ///
    /// Format should be the same as the file available at:
    /// <https://data.iana.org/time-zones/data/leap-seconds.list>
    ///
    /// For rows that don't start with # (the data rows) the first column
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
    /// use deep_time::leap_seconds::LEAP_SECS;
    /// use deep_time::{Dt, Scale};
    ///
    /// let leap_seconds_table =
    ///     Dt::leap_sec_data_from_file("tests/assets/leap-seconds.list.txt").unwrap();
    /// assert_eq!(leap_seconds_table[1], LEAP_SECS[1]);
    ///
    /// let x = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 0);
    /// let leap_info = Dt::leap_sec_using(&x, false, &leap_seconds_table).unwrap();
    /// assert!(leap_info.is_leap_sec == true);
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    ///
    /// let utc1 = dt.to(Scale::UTC);
    /// let utc2 = dt.to_utc_from_tai_using_leaps(Scale::UTC, &leap_seconds_table);
    /// assert_eq!(utc1, utc2);
    ///
    /// let tai1 = utc1.to_tai();
    /// let tai2 = utc2.to_tai_from_utc_using_leaps(&leap_seconds_table);
    /// assert_eq!(tai1, tai2);
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [Dt::leap_sec_data_from_str](../struct.Dt.html#method.leap_sec_data_from_str)
    /// - [Dt::to_utc_from_tai_using_leaps](../struct.Dt.html#method.to_utc_from_tai_using_leaps)
    /// - [Dt::to_tai_from_utc_using_leaps](../struct.Dt.html#method.to_tai_from_utc_using_leaps)
    #[inline]
    pub fn leap_sec_data_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<LeapSec>> {
        let content = fs::read_to_string(path)?;
        Ok(Self::leap_sec_data_from_str(&content))
    }
}

#[cfg(feature = "alloc")]
impl Dt {
    /// Load directly from a str (e.g. the official IANA `leap-seconds.list`).
    ///
    /// Format should be the same as the file available at:
    /// <https://data.iana.org/time-zones/data/leap-seconds.list>
    ///
    /// For rows that don't start with # (the data rows) the first column
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
    /// - [Dt::leap_sec_data_from_file](../struct.Dt.html#method.leap_sec_data_from_file)
    /// - [Dt::to_utc_from_tai_using_leaps](../struct.Dt.html#method.to_utc_from_tai_using_leaps)
    /// - [Dt::to_tai_from_utc_using_leaps](../struct.Dt.html#method.to_tai_from_utc_using_leaps)
    pub fn leap_sec_data_from_str(s: &str) -> Vec<LeapSec> {
        use crate::Scale;

        let mut table = Vec::new();
        let mut prev_leap_sec_after: i64 = 0;

        for line in s.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("#") {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();

            if parts.len() < 2 {
                continue;
            }
            let Ok(ntp_timestamp) = parts[0].parse::<i64>() else {
                continue;
            };
            let Ok(leap_sec_after) = parts[1].parse::<i64>() else {
                continue;
            };

            // don't use current: UTC because it would use the internal leap table
            let utc_sec = Dt::from_ntp(Dt::from_sec(ntp_timestamp as i128, Scale::TAI)).to_sec64();

            let tai_sec = if prev_leap_sec_after == 0 {
                utc_sec + leap_sec_after - 1
            } else {
                utc_sec + leap_sec_after - (leap_sec_after - prev_leap_sec_after)
            };

            table.push(LeapSec {
                ntp_timestamp,
                leap_sec_after,
                utc_sec,
                tai_sec,
            });

            prev_leap_sec_after = leap_sec_after;
        }

        table
    }
}
