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
    /// - `GPS`/`QZSS`   → 1980-01-06 00:00:00 GPS
    /// - `GST`          → 1999-08-22 00:00:00 GST
    /// - `BDT`          → 2006-01-01 00:00:00 BDT
    /// - All others     → library zero (2000-01-01 12:00:xx on that scale)
    /// -------------------------------------------------------------------------
    /// Returns the signed number of attoseconds since the given `reference` epoch.
    ///
    /// - If **both** `self` and `reference` have `ClockType::UTC` → uses civil/POSIX
    ///   semantics (leap seconds are *not* inserted into the second count). This
    ///   matches the old `to_canonical` behavior for UTC exactly.
    /// - Otherwise → exact physical difference (via TAI hub). Works across any
    ///   scales, including relativistic ones, GNSS, `Proper`/`Custom`, etc.
    #[inline]
    pub const fn to_attos_since(self, reference: TimePoint) -> i128 {
        if self.clock_type.eq(ClockType::UTC) && reference.clock_type.eq(ClockType::UTC) {
            self.utc_civil_canonical_attos() - reference.utc_civil_canonical_attos()
        } else {
            self.duration_since_ref(&reference).total_attos()
        }
    }

    #[inline(always)]
    pub(crate) const fn utc_civil_canonical_attos(self) -> i128 {
        ((self.sec as i128) + (UNIX_EPOCH_TO_J2000_NOON_UTC as i128)) * ATTOSEC_PER_SEC_I128
            + (self.subsec as i128)
    }

    #[inline]
    pub(crate) const fn from_utc_civil_canonical(canon: i128) -> Self {
        let sec = canon.div_euclid(ATTOSEC_PER_SEC_I128) as i64;
        let subsec = (canon.rem_euclid(ATTOSEC_PER_SEC_I128)) as u64;
        let internal_sec = sec - UNIX_EPOCH_TO_J2000_NOON_UTC;
        TimePoint::new(internal_sec, subsec, ClockType::UTC)
    }

    // --------------------- UNIX / UTC (POSIX epoch) ---------------------

    /// Returns this instant as **seconds** since the POSIX Unix epoch (UTC).
    #[inline]
    pub const fn to_unix_sec(self) -> i64 {
        let canon = self.to_attos_since(Self::UNIX_EPOCH_UTC);
        let div = ATTOSEC_PER_SEC_I128;
        let q = canon / div;
        let r = canon % div;
        if r >= 0 { q as i64 } else { (q - 1) as i64 }
    }

    /// Returns this instant as **milliseconds** since the POSIX Unix epoch
    /// (returns `i128` to avoid truncation over the full `TimePoint` range).
    #[inline]
    pub const fn to_unix_ms(self) -> i128 {
        self.to_attos_since(Self::UNIX_EPOCH_UTC) / (ATTOSEC_PER_MILLISEC as i128)
    }

    /// Returns this instant as **microseconds** since the POSIX Unix epoch
    /// (returns `i128`).
    #[inline]
    pub const fn to_unix_us(self) -> i128 {
        self.to_attos_since(Self::UNIX_EPOCH_UTC) / (ATTOSEC_PER_MICROSEC as i128)
    }

    /// Returns this instant as **nanoseconds** since the POSIX Unix epoch
    /// (returns `i128`).
    #[inline]
    pub const fn to_unix_ns(self) -> i128 {
        self.to_attos_since(Self::UNIX_EPOCH_UTC) / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- GPS / QZSS (1980-01-06 00:00:00 GPS) ---------------------

    #[inline]
    pub const fn to_gps_sec(self) -> i64 {
        let canon = self.to_attos_since(Self::GPS_EPOCH);
        let div = ATTOSEC_PER_SEC_I128;
        let q = canon / div;
        let r = canon % div;
        if r >= 0 { q as i64 } else { (q - 1) as i64 }
    }

    #[inline]
    pub const fn to_gps_ms(self) -> i128 {
        self.to_attos_since(Self::GPS_EPOCH) / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_gps_us(self) -> i128 {
        self.to_attos_since(Self::GPS_EPOCH) / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_gps_ns(self) -> i128 {
        self.to_attos_since(Self::GPS_EPOCH) / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- GALEX (1980-01-06 00:00:00, identical to GPS) ---------------------

    #[inline]
    pub const fn to_galex_sec(self) -> i64 {
        self.to_gps_sec()
    }

    #[inline]
    pub const fn to_galex_ms(self) -> i128 {
        self.to_gps_ms()
    }

    #[inline]
    pub const fn to_galex_us(self) -> i128 {
        self.to_gps_us()
    }

    #[inline]
    pub const fn to_galex_ns(self) -> i128 {
        self.to_gps_ns()
    }

    // --------------------- Galileo (1999-08-22 00:00:00 GST) ---------------------

    #[inline]
    pub const fn to_galileo_sec(self) -> i64 {
        let canon = self.to_attos_since(Self::GALILEO_EPOCH);
        let div = ATTOSEC_PER_SEC_I128;
        let q = canon / div;
        let r = canon % div;
        if r >= 0 { q as i64 } else { (q - 1) as i64 }
    }

    #[inline]
    pub const fn to_galileo_ms(self) -> i128 {
        self.to_attos_since(Self::GALILEO_EPOCH) / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_galileo_us(self) -> i128 {
        self.to_attos_since(Self::GALILEO_EPOCH) / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_galileo_ns(self) -> i128 {
        self.to_attos_since(Self::GALILEO_EPOCH) / (ATTOSEC_PER_NANOSEC as i128)
    }

    // --------------------- BeiDou (2006-01-01 00:00:00 BDT) ---------------------

    #[inline]
    pub const fn to_beidou_sec(self) -> i64 {
        let canon = self.to_attos_since(Self::BDT_EPOCH);
        let div = ATTOSEC_PER_SEC_I128;
        let q = canon / div;
        let r = canon % div;
        if r >= 0 { q as i64 } else { (q - 1) as i64 }
    }

    #[inline]
    pub const fn to_beidou_ms(self) -> i128 {
        self.to_attos_since(Self::BDT_EPOCH) / (ATTOSEC_PER_MILLISEC as i128)
    }

    #[inline]
    pub const fn to_beidou_us(self) -> i128 {
        self.to_attos_since(Self::BDT_EPOCH) / (ATTOSEC_PER_MICROSEC as i128)
    }

    #[inline]
    pub const fn to_beidou_ns(self) -> i128 {
        self.to_attos_since(Self::BDT_EPOCH) / (ATTOSEC_PER_NANOSEC as i128)
    }

    /// Converts a proleptic Gregorian calendar date+time to a Unix timestamp
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// This version is correct for the full i64 range, including negative years.
    #[inline]
    pub const fn ymdhms_to_unix_timestamp(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> i64 {
        let jdn = Self::ymd_to_jdn(year, month, day);

        // 1970-01-01 00:00:00 UTC corresponds to JD 2440588
        let days_since_1970 = jdn - 2440588;

        let time_of_day = (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);

        days_since_1970 * SEC_PER_DAYI64 + time_of_day
    }
}
