use crate::{
    ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC, ATTOSEC_PER_NANOSEC, ATTOSEC_PER_SEC_I128,
    ClockType, SEC_PER_DAYI64, TimePoint, UNIX_EPOCH_TO_J2000_NOON_UTC,
};

impl TimePoint {
    /// Returns the **signed** number of attoseconds since the globally-expected
    /// canonical epoch for this `TimePoint`’s [`ClockType`].
    ///
    /// Canonical (traditional) epochs used:
    /// - `UTC`          → 1970-01-01 00:00:00 UTC
    /// - `GPST`/`QZSST` → 1980-01-06 00:00:00 GPST
    /// - `GST`          → 1999-08-22 00:00:00 GST
    /// - `BDT`          → 2006-01-01 00:00:00 BDT
    /// - All others     → library zero (2000-01-01 12:00:xx on that scale)
    #[inline]
    pub const fn to_canonical_attoseconds(self) -> i128 {
        match self.clock_type {
            ClockType::UTC => {
                // Unix timestamps represent civil seconds since the POSIX epoch
                // (1970-01-01 00:00:00 UTC) on the Gregorian calendar. Leap seconds
                // are not inserted into the count. The internal `sec` field of a
                // UTC `TimePoint` already stores this civil count relative to
                // J2000 noon. Therefore the canonical value is computed by direct
                // offset rather than via `duration_since_ref`, which converts both
                // instants to TAI and incorporates the accumulated leap seconds.
                ((self.sec as i128) + (UNIX_EPOCH_TO_J2000_NOON_UTC as i128)) * ATTOSEC_PER_SEC_I128
                    + (self.subsec as i128)
            }
            ClockType::GPST | ClockType::QZSST => self
                .duration_since_ref(&Self::TRADITIONAL_GPS_EPOCH)
                .total_attos(),
            ClockType::GST => self
                .duration_since_ref(&Self::TRADITIONAL_GALILEO_EPOCH)
                .total_attos(),
            ClockType::BDT => self
                .duration_since_ref(&Self::TRADITIONAL_BEIDOU_EPOCH)
                .total_attos(),
            _ => self.total_attos(),
        }
    }

    // --------------------- UNIX / UTC (POSIX epoch) ---------------------

    /// Returns this instant as **seconds** since the POSIX Unix epoch (UTC).
    #[inline]
    pub const fn to_unix_sec(self) -> i64 {
        (self.to_canonical_attoseconds() / ATTOSEC_PER_SEC_I128) as i64
    }

    /// Returns this instant as **milliseconds** since the POSIX Unix epoch
    /// (returns `i128` to avoid truncation over the full `TimePoint` range).
    #[inline]
    pub const fn to_unix_ms(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MILLISEC as i128)
    }

    /// Returns this instant as **microseconds** since the POSIX Unix epoch
    /// (returns `i128`).
    #[inline]
    pub const fn to_unix_us(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MICROSEC as i128)
    }

    /// Returns this instant as **nanoseconds** since the POSIX Unix epoch
    /// (returns `i128`).
    #[inline]
    pub const fn to_unix_ns(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- GPS / QZSS (1980-01-06 00:00:00 GPST) ---------------------

    #[inline]
    pub const fn to_gps_sec(self) -> i64 {
        (self.to_canonical_attoseconds() / ATTOSEC_PER_SEC_I128) as i64
    }

    #[inline]
    pub const fn to_gps_ms(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_gps_us(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_gps_ns(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- Galileo (1999-08-22 00:00:00 GST) ---------------------

    #[inline]
    pub const fn to_galileo_sec(self) -> i64 {
        (self.to_canonical_attoseconds() / ATTOSEC_PER_SEC_I128) as i64
    }

    #[inline]
    pub const fn to_galileo_ms(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_galileo_us(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_galileo_ns(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- BeiDou (2006-01-01 00:00:00 BDT) ---------------------

    #[inline]
    pub const fn to_beidou_sec(self) -> i64 {
        (self.to_canonical_attoseconds() / ATTOSEC_PER_SEC_I128) as i64
    }

    #[inline]
    pub const fn to_beidou_ms(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_beidou_us(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_beidou_ns(self) -> i128 {
        self.to_canonical_attoseconds() / (ATTOSEC_PER_NANOSEC as i128)
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

        days * SEC_PER_DAYI64 + time_of_day
    }
}
