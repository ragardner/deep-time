//! Leap seconds table from the official IANA
//! [leap-seconds.list](https://data.iana.org/time-zones/data/leap-seconds.list)
//! Last leap second: 2017-01-01 (TAI-UTC = 37 s)
//! File expires: 28 December 2026

/// Holds info about a leap-second transition. Used by [LEAP_SECS](constant.LEAP_SECS.html).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LeapSec {
    /// NTP timestamp of the transition (IANA file, column 1)
    pub ntp_timestamp: i64,
    /// Cumulative TAI-UTC offset in seconds after this transition (IANA column 2)
    pub leap_sec_after: i64,
    /// Library timestamp of the transition on the UTC scale
    pub utc_sec: i64,
    /// Library timestamp of the transition on the TAI scale
    pub tai_sec: i64,
}

/// Embedded leap-seconds list shipped with the library.
///
/// Each entry records the instant when the cumulative TAI-UTC offset
/// changes. Rows are sorted chronologically.
pub const LEAP_SECS: &[LeapSec] = &[
    LeapSec {
        ntp_timestamp: 2272060800,
        leap_sec_after: 10,
        utc_sec: -883656000,
        tai_sec: -883655991,
    }, // 1 Jan 1972
    LeapSec {
        ntp_timestamp: 2287785600,
        leap_sec_after: 11,
        utc_sec: -867931200,
        tai_sec: -867931190,
    }, // 1 Jul 1972
    LeapSec {
        ntp_timestamp: 2303683200,
        leap_sec_after: 12,
        utc_sec: -852033600,
        tai_sec: -852033589,
    }, // 1 Jan 1973
    LeapSec {
        ntp_timestamp: 2335219200,
        leap_sec_after: 13,
        utc_sec: -820497600,
        tai_sec: -820497588,
    }, // 1 Jan 1974
    LeapSec {
        ntp_timestamp: 2366755200,
        leap_sec_after: 14,
        utc_sec: -788961600,
        tai_sec: -788961587,
    }, // 1 Jan 1975
    LeapSec {
        ntp_timestamp: 2398291200,
        leap_sec_after: 15,
        utc_sec: -757425600,
        tai_sec: -757425586,
    }, // 1 Jan 1976
    LeapSec {
        ntp_timestamp: 2429913600,
        leap_sec_after: 16,
        utc_sec: -725803200,
        tai_sec: -725803185,
    }, // 1 Jan 1977
    LeapSec {
        ntp_timestamp: 2461449600,
        leap_sec_after: 17,
        utc_sec: -694267200,
        tai_sec: -694267184,
    }, // 1 Jan 1978
    LeapSec {
        ntp_timestamp: 2492985600,
        leap_sec_after: 18,
        utc_sec: -662731200,
        tai_sec: -662731183,
    }, // 1 Jan 1979
    LeapSec {
        ntp_timestamp: 2524521600,
        leap_sec_after: 19,
        utc_sec: -631195200,
        tai_sec: -631195182,
    }, // 1 Jan 1980
    LeapSec {
        ntp_timestamp: 2571782400,
        leap_sec_after: 20,
        utc_sec: -583934400,
        tai_sec: -583934381,
    }, // 1 Jul 1981
    LeapSec {
        ntp_timestamp: 2603318400,
        leap_sec_after: 21,
        utc_sec: -552398400,
        tai_sec: -552398380,
    }, // 1 Jul 1982
    LeapSec {
        ntp_timestamp: 2634854400,
        leap_sec_after: 22,
        utc_sec: -520862400,
        tai_sec: -520862379,
    }, // 1 Jul 1983
    LeapSec {
        ntp_timestamp: 2698012800,
        leap_sec_after: 23,
        utc_sec: -457704000,
        tai_sec: -457703978,
    }, // 1 Jul 1985
    LeapSec {
        ntp_timestamp: 2776982400,
        leap_sec_after: 24,
        utc_sec: -378734400,
        tai_sec: -378734377,
    }, // 1 Jan 1988
    LeapSec {
        ntp_timestamp: 2840140800,
        leap_sec_after: 25,
        utc_sec: -315576000,
        tai_sec: -315575976,
    }, // 1 Jan 1990
    LeapSec {
        ntp_timestamp: 2871676800,
        leap_sec_after: 26,
        utc_sec: -284040000,
        tai_sec: -284039975,
    }, // 1 Jan 1991
    LeapSec {
        ntp_timestamp: 2918937600,
        leap_sec_after: 27,
        utc_sec: -236779200,
        tai_sec: -236779174,
    }, // 1 Jul 1992
    LeapSec {
        ntp_timestamp: 2950473600,
        leap_sec_after: 28,
        utc_sec: -205243200,
        tai_sec: -205243173,
    }, // 1 Jul 1993
    LeapSec {
        ntp_timestamp: 2982009600,
        leap_sec_after: 29,
        utc_sec: -173707200,
        tai_sec: -173707172,
    }, // 1 Jul 1994
    LeapSec {
        ntp_timestamp: 3029443200,
        leap_sec_after: 30,
        utc_sec: -126273600,
        tai_sec: -126273571,
    }, // 1 Jan 1996
    LeapSec {
        ntp_timestamp: 3076704000,
        leap_sec_after: 31,
        utc_sec: -79012800,
        tai_sec: -79012770,
    }, // 1 Jul 1997
    LeapSec {
        ntp_timestamp: 3124137600,
        leap_sec_after: 32,
        utc_sec: -31579200,
        tai_sec: -31579169,
    }, // 1 Jan 1999
    LeapSec {
        ntp_timestamp: 3345062400,
        leap_sec_after: 33,
        utc_sec: 189345600,
        tai_sec: 189345632,
    }, // 1 Jan 2006
    LeapSec {
        ntp_timestamp: 3439756800,
        leap_sec_after: 34,
        utc_sec: 284040000,
        tai_sec: 284040033,
    }, // 1 Jan 2009
    LeapSec {
        ntp_timestamp: 3550089600,
        leap_sec_after: 35,
        utc_sec: 394372800,
        tai_sec: 394372834,
    }, // 1 Jul 2012
    LeapSec {
        ntp_timestamp: 3644697600,
        leap_sec_after: 36,
        utc_sec: 488980800,
        tai_sec: 488980835,
    }, // 1 Jul 2015
    LeapSec {
        ntp_timestamp: 3692217600,
        leap_sec_after: 37,
        utc_sec: 536500800,
        tai_sec: 536500836,
    }, // 1 Jan 2017
];
