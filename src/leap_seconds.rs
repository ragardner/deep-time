//! Leap seconds table from the official IANA leap-seconds.list
//![](https://data.iana.org/time-zones/data/leap-seconds.list)
//! Updated through IERS Bulletin C as of April 2026.
//! Last leap second: 2017-01-01 (TAI-UTC = 37 s)
//! File expires: 28 December 2026

use crate::TimePoint;

/// (NTP timestamp of the leap second insertion, cumulative leap seconds *after* this insertion)
///
/// NTP timestamp = seconds since 1900-01-01 00:00:00 (the format used by the official file).
const LEAP_SECONDS: &[(i128, i128)] = &[
    (2272060800, 10), // 1 Jan 1972
    (2287785600, 11), // 1 Jul 1972
    (2303683200, 12), // 1 Jan 1973
    (2335219200, 13), // 1 Jan 1974
    (2366755200, 14), // 1 Jan 1975
    (2398291200, 15), // 1 Jan 1976
    (2429913600, 16), // 1 Jan 1977
    (2461449600, 17), // 1 Jan 1978
    (2492985600, 18), // 1 Jan 1979
    (2524521600, 19), // 1 Jan 1980
    (2571782400, 20), // 1 Jul 1981
    (2603318400, 21), // 1 Jul 1982
    (2634854400, 22), // 1 Jul 1983
    (2698012800, 23), // 1 Jul 1985
    (2776982400, 24), // 1 Jan 1988
    (2840140800, 25), // 1 Jan 1990
    (2871676800, 26), // 1 Jan 1991
    (2918937600, 27), // 1 Jul 1992
    (2950473600, 28), // 1 Jul 1993
    (2982009600, 29), // 1 Jul 1994
    (3029443200, 30), // 1 Jan 1996
    (3076704000, 31), // 1 Jul 1997
    (3124137600, 32), // 1 Jan 1999
    (3345062400, 33), // 1 Jan 2006
    (3439756800, 34), // 1 Jan 2009
    (3550089600, 35), // 1 Jul 2012
    (3644697600, 36), // 1 Jul 2015
    (3692217600, 37), // 1 Jan 2017
];

/// Returns leap seconds inserted **before** this TAI instant (TAI = UTC + result).
pub const fn leap_seconds_before(tai: TimePoint) -> i128 {
    let mut offset = 0i128;
    let mut i = 0usize;

    while i < LEAP_SECONDS.len() {
        let leap_ntp = LEAP_SECONDS[i].0;

        // J2000 noon (TAI zero) → NTP timestamp
        const J2000_NTP_OFFSET: i128 = 3_155_328_000 + 43_200;

        let tai_ntp = tai.sec() + J2000_NTP_OFFSET;

        if tai_ntp >= leap_ntp {
            offset = LEAP_SECONDS[i].1;
        } else {
            break;
        }
        i += 1;
    }
    offset
}
