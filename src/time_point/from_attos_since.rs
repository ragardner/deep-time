use crate::{
    ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_US, ATTOSEC_PER_SEC_I128, ClockType, TimePoint, TimeSpan,
    UNIX_EPOCH_TO_J2000_NOON_UTC,
};

impl TimePoint {
    /// Inverse of [`Self::to_canonical`].
    ///
    /// Creates a `TimePoint` that is exactly `attos` attoseconds after the
    /// globally-expected canonical epoch of the requested `clock_type`.
    /// -------------------------------------------------------------------
    /// Inverse of [`Self::to_attos_since`].
    ///
    /// Creates a `TimePoint` that is exactly `attos` attoseconds after the
    /// supplied `reference` epoch, on the same `ClockType` as the reference.
    #[inline]
    pub const fn from_attos_since(attos: i128, reference: TimePoint) -> Self {
        if reference.clock_type.is_ut() {
            let ref_canon = reference.utc_civil_canonical_attos();
            let target_canon = ref_canon.saturating_add(attos);
            Self::from_utc_civil_canonical(target_canon, reference.clock_type)
        } else {
            reference.saturating_add(TimeSpan::from_total_attos(attos))
        }
    }

    pub(crate) const fn from_utc_civil_canonical(canon: i128, clock_type: ClockType) -> Self {
        let sec = canon.div_euclid(ATTOSEC_PER_SEC_I128) as i64;
        let subsec = (canon.rem_euclid(ATTOSEC_PER_SEC_I128)) as u64;
        let internal_sec = sec - UNIX_EPOCH_TO_J2000_NOON_UTC;
        TimePoint::new(internal_sec, subsec, clock_type)
    }

    // --------------------- UNIX / UTC (POSIX epoch) ---------------------

    /// Creates a `TimePoint` from **seconds** since the POSIX Unix epoch
    /// (1970-01-01 00:00:00 UTC).
    #[inline]
    pub const fn from_unix_sec(s: i64) -> Self {
        Self::from_attos_since((s as i128) * ATTOSEC_PER_SEC_I128, Self::UNIX_EPOCH_UTC)
    }

    /// Creates a `TimePoint` from **milliseconds** since the POSIX Unix epoch
    /// (full `i128` range supported to match `TimePoint`’s full representable span).
    #[inline]
    pub const fn from_unix_ms(ms: i128) -> Self {
        Self::from_attos_since(ms * (ATTOS_PER_MS as i128), Self::UNIX_EPOCH_UTC)
    }

    /// Creates a `TimePoint` from **microseconds** since the POSIX Unix epoch
    /// (full `i128` range supported).
    #[inline]
    pub const fn from_unix_us(us: i128) -> Self {
        Self::from_attos_since(us * (ATTOS_PER_US as i128), Self::UNIX_EPOCH_UTC)
    }

    /// Creates a `TimePoint` from **nanoseconds** since the POSIX Unix epoch
    /// (full `i128` range supported).
    #[inline]
    pub const fn from_unix_ns(ns: i128) -> Self {
        Self::from_attos_since(ns * (ATTOS_PER_NS as i128), Self::UNIX_EPOCH_UTC)
    }

    // --------------------- GPS / QZSS (1980-01-06 00:00:00 GPS) ---------------------

    /// Creates a `TimePoint` from **seconds** since the traditional GPS epoch
    /// (1980-01-06 00:00:00 GPS). Works for both `GPS` and `QZSS`.
    #[inline]
    pub const fn from_gps_sec(s: i64) -> Self {
        Self::from_attos_since((s as i128) * ATTOSEC_PER_SEC_I128, Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_gps_ms(ms: i128) -> Self {
        Self::from_attos_since(ms * (ATTOS_PER_MS as i128), Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_gps_us(us: i128) -> Self {
        Self::from_attos_since(us * (ATTOS_PER_US as i128), Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_gps_ns(ns: i128) -> Self {
        Self::from_attos_since(ns * (ATTOS_PER_NS as i128), Self::GPS_EPOCH)
    }

    // --------------------- GALEX (1980-01-06 00:00:00, identical to GPS) ---------------------

    #[inline]
    pub const fn from_galex_sec(s: i64) -> Self {
        Self::from_attos_since((s as i128) * ATTOSEC_PER_SEC_I128, Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_galex_ms(ms: i128) -> Self {
        Self::from_attos_since(ms * (ATTOS_PER_MS as i128), Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_galex_us(us: i128) -> Self {
        Self::from_attos_since(us * (ATTOS_PER_US as i128), Self::GPS_EPOCH)
    }

    #[inline]
    pub const fn from_galex_ns(ns: i128) -> Self {
        Self::from_attos_since(ns * (ATTOS_PER_NS as i128), Self::GPS_EPOCH)
    }

    // --------------------- Galileo (1999-08-22 00:00:00 GST) ---------------------

    #[inline]
    pub const fn from_galileo_sec(s: i64) -> Self {
        Self::from_attos_since((s as i128) * ATTOSEC_PER_SEC_I128, Self::GALILEO_EPOCH)
    }

    #[inline]
    pub const fn from_galileo_ms(ms: i128) -> Self {
        Self::from_attos_since(ms * (ATTOS_PER_MS as i128), Self::GALILEO_EPOCH)
    }

    #[inline]
    pub const fn from_galileo_us(us: i128) -> Self {
        Self::from_attos_since(us * (ATTOS_PER_US as i128), Self::GALILEO_EPOCH)
    }

    #[inline]
    pub const fn from_galileo_ns(ns: i128) -> Self {
        Self::from_attos_since(ns * (ATTOS_PER_NS as i128), Self::GALILEO_EPOCH)
    }

    // --------------------- BeiDou (2006-01-01 00:00:00 BDT) ---------------------

    #[inline]
    pub const fn from_beidou_sec(s: i64) -> Self {
        Self::from_attos_since((s as i128) * ATTOSEC_PER_SEC_I128, Self::BDT_EPOCH)
    }

    #[inline]
    pub const fn from_beidou_ms(ms: i128) -> Self {
        Self::from_attos_since(ms * (ATTOS_PER_MS as i128), Self::BDT_EPOCH)
    }

    #[inline]
    pub const fn from_beidou_us(us: i128) -> Self {
        Self::from_attos_since(us * (ATTOS_PER_US as i128), Self::BDT_EPOCH)
    }

    #[inline]
    pub const fn from_beidou_ns(ns: i128) -> Self {
        Self::from_attos_since(ns * (ATTOS_PER_NS as i128), Self::BDT_EPOCH)
    }
}

#[test]
fn test_1972_leap_second_canonical_roundtrip() {
    // Create the leap second the "normal" way (using from_ymdhms)
    let original = TimePoint::from_ymdhms(1972, 6, 30, 23, 59, 60, 0, ClockType::UTC);

    // Round-trip through attoseconds since the Unix epoch
    // (this exercises the exact civil/POSIX UTC path in to_attos_since/from_attos_since)
    let canon = original.to_attos_since(TimePoint::UNIX_EPOCH_UTC);
    let roundtrip = TimePoint::from_attos_since(canon, TimePoint::UNIX_EPOCH_UTC);

    // These should be identical if everything is consistent
    assert_eq!(
        original, roundtrip,
        "Round-trip failed for 1972 leap second"
    );

    // Also verify civil time is still correct
    let g = roundtrip.to_ymdhms();
    assert_eq!(g.yr, 1972);
    assert_eq!(g.mo, 6);
    assert_eq!(g.day, 30);
    assert_eq!(g.hr, 23);
    assert_eq!(g.min, 59);
    assert_eq!(g.sec, 60, "Should still show sec=60 after round-trip");
}
