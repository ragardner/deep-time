//! UNIX timestamp (seconds since 1970-01-01 00:00:00 UTC) helpers.
//!
//! These are thin wrappers around existing UTC ↔ TAI conversion logic
//! so they automatically handle leap seconds correctly.

use crate::{Delta, MICROQUECTOS_PER_MILLISEC, Point, TimePov};

impl Point {
    /// Creates a `Point` from a classic Unix timestamp **in seconds**
    /// since 1970-01-01 00:00:00 UTC.
    ///
    /// The returned `Point` is in TAI (your internal hub).
    #[inline]
    pub const fn from_unix_seconds(seconds: i128) -> Self {
        Point::new(seconds, 0, TimePov::UTC).to_tai()
    }

    /// Creates a `Point` from a Unix timestamp **in milliseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_milliseconds(millis: i128) -> Self {
        Point::new(0, 0, TimePov::UTC)
            .add(Delta::from_ms(millis))
            .to_tai()
    }

    /// Creates a `Point` from a Unix timestamp **in microseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_microseconds(micros: i128) -> Self {
        Point::new(0, 0, TimePov::UTC)
            .add(Delta::from_us(micros))
            .to_tai()
    }

    /// Creates a `Point` from a Unix timestamp **in nanoseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn from_unix_nanoseconds(nanos: i128) -> Self {
        Point::new(0, 0, TimePov::UTC)
            .add(Delta::from_ns(nanos))
            .to_tai()
    }

    /// Returns this instant as a Unix timestamp in **seconds**
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// Sub-second precision is truncated (floor).
    #[inline]
    pub const fn to_unix_seconds(self) -> i128 {
        let utc = self.to_pov(TimePov::UTC);
        utc.sec()
    }

    /// Returns this instant as a Unix timestamp in **milliseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn to_unix_milliseconds(self) -> i128 {
        let utc = self.to_pov(TimePov::UTC);
        utc.sec() * 1_000 + (utc.subsec() / MICROQUECTOS_PER_MILLISEC) as i128
    }

    /// Returns this instant as a Unix timestamp in **microseconds**
    /// since 1970-01-01 00:00:00 UTC.
    #[inline]
    pub const fn to_unix_microseconds(self) -> i128 {
        let utc = self.to_pov(TimePov::UTC);
        utc.sec() * 1_000_000 + (utc.subsec() / crate::MICROQUECTOS_PER_MICROSEC) as i128
    }
}
