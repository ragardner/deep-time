//! Leap seconds table from the official IANA leap-seconds.list
//![](https://data.iana.org/time-zones/data/leap-seconds.list)
//! Updated through IERS Bulletin C as of April 2026.
//! Last leap second: 2017-01-01 (TAI-UTC = 37 s)
//! File expires: 28 December 2026

use crate::TimePoint;

/// Represents a single leap second insertion event from the official IERS/IANA leap-seconds.list.
///
/// - `ntp_timestamp`: NTP timestamp of the leap second insertion (seconds since 1900-01-01 00:00:00 UTC)
/// - `leap_seconds_after`: Cumulative TAI−UTC value that takes effect immediately after this insertion
/// - `utc_sec`: Your library's internal `sec` value when creating this date as `ClockType::UTC`
/// - `tai_sec`: Your library's internal `sec` value when creating this date as `ClockType::TAI`
pub struct LeapSecond {
    pub ntp_timestamp: i64,
    pub leap_seconds_after: i64,
    pub utc_sec: i64,
    pub tai_sec: i64,
}

pub const LEAP_SECS: &[LeapSecond] = &[
    LeapSecond {
        ntp_timestamp: 2272060800,
        leap_seconds_after: 10,
        utc_sec: -883656000,
        tai_sec: -883655990,
    }, // 1 Jan 1972 (start of modern UTC definition)
    LeapSecond {
        ntp_timestamp: 2287785600,
        leap_seconds_after: 11,
        utc_sec: -867931200,
        tai_sec: -867931189,
    }, // 1 Jul 1972
    LeapSecond {
        ntp_timestamp: 2303683200,
        leap_seconds_after: 12,
        utc_sec: -852033600,
        tai_sec: -852033588,
    }, // 1 Jan 1973
    LeapSecond {
        ntp_timestamp: 2335219200,
        leap_seconds_after: 13,
        utc_sec: -820497600,
        tai_sec: -820497587,
    }, // 1 Jan 1974
    LeapSecond {
        ntp_timestamp: 2366755200,
        leap_seconds_after: 14,
        utc_sec: -788961600,
        tai_sec: -788961586,
    }, // 1 Jan 1975
    LeapSecond {
        ntp_timestamp: 2398291200,
        leap_seconds_after: 15,
        utc_sec: -757425600,
        tai_sec: -757425585,
    }, // 1 Jan 1976
    LeapSecond {
        ntp_timestamp: 2429913600,
        leap_seconds_after: 16,
        utc_sec: -725803200,
        tai_sec: -725803184,
    }, // 1 Jan 1977
    LeapSecond {
        ntp_timestamp: 2461449600,
        leap_seconds_after: 17,
        utc_sec: -694267200,
        tai_sec: -694267183,
    }, // 1 Jan 1978
    LeapSecond {
        ntp_timestamp: 2492985600,
        leap_seconds_after: 18,
        utc_sec: -662731200,
        tai_sec: -662731182,
    }, // 1 Jan 1979
    LeapSecond {
        ntp_timestamp: 2524521600,
        leap_seconds_after: 19,
        utc_sec: -631195200,
        tai_sec: -631195181,
    }, // 1 Jan 1980
    LeapSecond {
        ntp_timestamp: 2571782400,
        leap_seconds_after: 20,
        utc_sec: -583934400,
        tai_sec: -583934380,
    }, // 1 Jul 1981
    LeapSecond {
        ntp_timestamp: 2603318400,
        leap_seconds_after: 21,
        utc_sec: -552398400,
        tai_sec: -552398379,
    }, // 1 Jul 1982
    LeapSecond {
        ntp_timestamp: 2634854400,
        leap_seconds_after: 22,
        utc_sec: -520862400,
        tai_sec: -520862378,
    }, // 1 Jul 1983
    LeapSecond {
        ntp_timestamp: 2698012800,
        leap_seconds_after: 23,
        utc_sec: -457704000,
        tai_sec: -457703977,
    }, // 1 Jul 1985
    LeapSecond {
        ntp_timestamp: 2776982400,
        leap_seconds_after: 24,
        utc_sec: -378734400,
        tai_sec: -378734376,
    }, // 1 Jan 1988
    LeapSecond {
        ntp_timestamp: 2840140800,
        leap_seconds_after: 25,
        utc_sec: -315576000,
        tai_sec: -315575975,
    }, // 1 Jan 1990
    LeapSecond {
        ntp_timestamp: 2871676800,
        leap_seconds_after: 26,
        utc_sec: -284040000,
        tai_sec: -284039974,
    }, // 1 Jan 1991
    LeapSecond {
        ntp_timestamp: 2918937600,
        leap_seconds_after: 27,
        utc_sec: -236779200,
        tai_sec: -236779173,
    }, // 1 Jul 1992
    LeapSecond {
        ntp_timestamp: 2950473600,
        leap_seconds_after: 28,
        utc_sec: -205243200,
        tai_sec: -205243172,
    }, // 1 Jul 1993
    LeapSecond {
        ntp_timestamp: 2982009600,
        leap_seconds_after: 29,
        utc_sec: -173707200,
        tai_sec: -173707171,
    }, // 1 Jul 1994
    LeapSecond {
        ntp_timestamp: 3029443200,
        leap_seconds_after: 30,
        utc_sec: -126273600,
        tai_sec: -126273570,
    }, // 1 Jan 1996
    LeapSecond {
        ntp_timestamp: 3076704000,
        leap_seconds_after: 31,
        utc_sec: -79012800,
        tai_sec: -79012769,
    }, // 1 Jul 1997
    LeapSecond {
        ntp_timestamp: 3124137600,
        leap_seconds_after: 32,
        utc_sec: -31579200,
        tai_sec: -31579168,
    }, // 1 Jan 1999
    LeapSecond {
        ntp_timestamp: 3345062400,
        leap_seconds_after: 33,
        utc_sec: 189345600,
        tai_sec: 189345633,
    }, // 1 Jan 2006
    LeapSecond {
        ntp_timestamp: 3439756800,
        leap_seconds_after: 34,
        utc_sec: 284040000,
        tai_sec: 284040034,
    }, // 1 Jan 2009
    LeapSecond {
        ntp_timestamp: 3550089600,
        leap_seconds_after: 35,
        utc_sec: 394372800,
        tai_sec: 394372835,
    }, // 1 Jul 2012
    LeapSecond {
        ntp_timestamp: 3644697600,
        leap_seconds_after: 36,
        utc_sec: 488980800,
        tai_sec: 488980836,
    }, // 1 Jul 2015
    LeapSecond {
        ntp_timestamp: 3692217600,
        leap_seconds_after: 37,
        utc_sec: 536500800,
        tai_sec: 536500837,
    }, // 1 Jan 2017
];

/// Returns the cumulative leap seconds (TAI − UTC offset in whole seconds)
/// that are valid **at or after** the given `TimePoint`.
///
/// Works for both `ClockType::UTC` and `ClockType::TAI` by using the
/// pre-computed `utc_sec` / `tai_sec` values from the `LEAP_SECS` table.
pub const fn leap_seconds_before(tp: &TimePoint) -> i64 {
    let is_utc = tp.uses_leap_sec();

    let mut offset = 0i64;
    let mut i = 0usize;

    while i < LEAP_SECS.len() {
        let entry = &LEAP_SECS[i];

        // Choose the appropriate sec field based on clock type
        let entry_sec = if is_utc { entry.utc_sec } else { entry.tai_sec };

        if tp.sec() >= entry_sec {
            offset = entry.leap_seconds_after;
        } else {
            break;
        }
        i += 1;
    }
    offset
}

/// Returns true if the given `TimePoint` falls exactly on a leap second insertion instant.
pub const fn is_leap_second(tp: &TimePoint) -> bool {
    if !tp.uses_leap_sec() {
        return false;
    }

    let is_utc = tp.uses_leap_sec();

    let mut i = 0usize;
    while i < LEAP_SECS.len() {
        let entry = &LEAP_SECS[i];
        let entry_sec = if is_utc { entry.utc_sec } else { entry.tai_sec };

        if tp.sec() == entry_sec {
            return true;
        }
        if tp.sec() < entry_sec {
            break;
        }
        i += 1;
    }
    false
}

// (NTP timestamp of the leap second insertion, cumulative leap seconds *after* this insertion)
//
// NTP timestamp = seconds since 1900-01-01 00:00:00 UTC (the format used by the official file).
// pub const LEAP_SEC_OLD: &[(i64, i64)] = &[
//     (2272060800, 10), // 1 Jan 1972
//     (2287785600, 11), // 1 Jul 1972
//     (2303683200, 12), // 1 Jan 1973
//     (2335219200, 13), // 1 Jan 1974
//     (2366755200, 14), // 1 Jan 1975
//     (2398291200, 15), // 1 Jan 1976
//     (2429913600, 16), // 1 Jan 1977
//     (2461449600, 17), // 1 Jan 1978
//     (2492985600, 18), // 1 Jan 1979
//     (2524521600, 19), // 1 Jan 1980
//     (2571782400, 20), // 1 Jul 1981
//     (2603318400, 21), // 1 Jul 1982
//     (2634854400, 22), // 1 Jul 1983
//     (2698012800, 23), // 1 Jul 1985
//     (2776982400, 24), // 1 Jan 1988
//     (2840140800, 25), // 1 Jan 1990
//     (2871676800, 26), // 1 Jan 1991
//     (2918937600, 27), // 1 Jul 1992
//     (2950473600, 28), // 1 Jul 1993
//     (2982009600, 29), // 1 Jul 1994
//     (3029443200, 30), // 1 Jan 1996
//     (3076704000, 31), // 1 Jul 1997
//     (3124137600, 32), // 1 Jan 1999
//     (3345062400, 33), // 1 Jan 2006
//     (3439756800, 34), // 1 Jan 2009
//     (3550089600, 35), // 1 Jul 2012
//     (3644697600, 36), // 1 Jul 2015
//     (3692217600, 37), // 1 Jan 2017
// ];
