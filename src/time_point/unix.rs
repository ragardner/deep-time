//! UNIX timestamp (seconds since 1970-01-01 00:00:00 UTC) helpers.
//!
//! These are thin wrappers around existing UTC ↔ TAI conversion logic
//! so they automatically handle leap seconds correctly.

use crate::{ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC, ClockType, Delta, TimePoint};

impl TimePoint {
    /// Creates a `TimePoint` from a classic Unix timestamp **in seconds**
    /// since 1970-01-01 00:00:00 UTC.
    ///
    /// The returned `TimePoint` is in TAI (your internal hub).
    #[inline]
    pub const fn from_unix_seconds(seconds: i64) -> Self {
        TimePoint::new(seconds, 0, ClockType::UTC).to_tai()
    }

    /// Creates a `TimePoint` from a Unix timestamp **in milliseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_milliseconds(millis: i64) -> Self {
        TimePoint::new(0, 0, ClockType::UTC)
            .add(Delta::from_ms(millis))
            .to_tai()
    }

    /// Creates a `TimePoint` from a Unix timestamp **in microseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_microseconds(micros: i64) -> Self {
        TimePoint::new(0, 0, ClockType::UTC)
            .add(Delta::from_us(micros))
            .to_tai()
    }

    /// Creates a `TimePoint` from a Unix timestamp **in nanoseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_nanoseconds(nanos: i64) -> Self {
        TimePoint::new(0, 0, ClockType::UTC)
            .add(Delta::from_ns(nanos))
            .to_tai()
    }

    /// Returns this instant as a Unix timestamp in **seconds**
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// Sub-second precision is truncated (floor).
    #[inline]
    pub const fn to_unix_seconds(self) -> i64 {
        let utc = self.to_clock_type(ClockType::UTC).carry_over();
        utc.sec()
    }

    /// Returns this instant as a Unix timestamp in **milliseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn to_unix_milliseconds(self) -> i64 {
        let utc = self.to_clock_type(ClockType::UTC);
        utc.sec() * 1_000 + (utc.subsec() / ATTOSEC_PER_MILLISEC) as i64
    }

    /// Returns this instant as a Unix timestamp in **microseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn to_unix_microseconds(self) -> i64 {
        let utc = self.to_clock_type(ClockType::UTC);
        utc.sec() * 1_000_000 + (utc.subsec() / ATTOSEC_PER_MICROSEC) as i64
    }

    /// Converts a proleptic Gregorian calendar date+time to a Unix timestamp
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// - `year` can be any i64 (negative years = BC, positive = AD).
    /// - `month` 1-12, `day` 1-31, `hour` 0-23, `minute` 0-59, `second` 0-60.
    /// - No validation is performed (assumes the caller passes a valid civil date).
    /// - Works for the full practical i64 range (±292 billion years) with no overflow.
    /// - Pure Rust, no dependencies, no_std compatible.
    #[inline]
    pub const fn ymdhms_to_unix_timestamp(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> i64 {
        let mut y = year;
        let mut m = month as i64;

        // January and February are counted as months 13 and 14 of the previous year
        if m <= 2 {
            y -= 1;
            m += 12;
        }

        // Days since 1970-01-01 (proleptic Gregorian)
        let days =
            y * 365 + y / 4 - y / 100 + y / 400 + (m * 153 - 457) / 5 + (day as i64) - 719469;

        // Seconds in the day
        let time_of_day = (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);

        days * 86_400 + time_of_day
    }
}
