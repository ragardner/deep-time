use crate::{
    ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC, ATTOSEC_PER_NANOSEC, ATTOSEC_PER_SEC_I128,
    ClockType, TimePoint, TimeSpan, UNIX_EPOCH_TO_J2000_NOON_UTC,
};

impl TimePoint {
    /// Inverse of [`Self::to_canonical_attoseconds`].
    ///
    /// Creates a `TimePoint` that is exactly `attos` attoseconds after the
    /// globally-expected canonical epoch of the requested `clock_type`.
    #[inline]
    pub const fn from_canonical_attoseconds(attos: i128, clock_type: ClockType) -> Self {
        match clock_type {
            ClockType::UTC => {
                let sec = attos.div_euclid(ATTOSEC_PER_SEC_I128) as i64;
                let subsec = (attos.rem_euclid(ATTOSEC_PER_SEC_I128)) as u64;
                // Convert from Unix civil seconds → internal J2000-relative civil seconds
                let internal_sec = sec - UNIX_EPOCH_TO_J2000_NOON_UTC;

                TimePoint::new(internal_sec, subsec, ClockType::UTC)
            }
            ClockType::GPST | ClockType::QZSST => {
                Self::TRADITIONAL_GPS_EPOCH.add(TimeSpan::from_total_attos(attos))
            }
            ClockType::GST => {
                Self::TRADITIONAL_GALILEO_EPOCH.add(TimeSpan::from_total_attos(attos))
            }
            ClockType::BDT => Self::TRADITIONAL_BEIDOU_EPOCH.add(TimeSpan::from_total_attos(attos)),
            _ => TimePoint::new(0, 0, clock_type).add(TimeSpan::from_total_attos(attos)),
        }
    }

    // --------------------- UNIX / UTC (POSIX epoch) ---------------------

    /// Creates a `TimePoint` from **seconds** since the POSIX Unix epoch
    /// (1970-01-01 00:00:00 UTC).
    #[inline]
    pub const fn from_unix_sec(s: i64) -> Self {
        Self::from_canonical_attoseconds((s as i128) * ATTOSEC_PER_SEC_I128, ClockType::UTC)
    }

    /// Creates a `TimePoint` from **milliseconds** since the POSIX Unix epoch
    /// (full `i128` range supported to match `TimePoint`’s full representable span).
    #[inline]
    pub const fn from_unix_ms(ms: i128) -> Self {
        Self::from_canonical_attoseconds(ms * (ATTOSEC_PER_MILLISEC as i128), ClockType::UTC)
    }

    /// Creates a `TimePoint` from **microseconds** since the POSIX Unix epoch
    /// (full `i128` range supported).
    #[inline]
    pub const fn from_unix_us(us: i128) -> Self {
        Self::from_canonical_attoseconds(us * (ATTOSEC_PER_MICROSEC as i128), ClockType::UTC)
    }

    /// Creates a `TimePoint` from **nanoseconds** since the POSIX Unix epoch
    /// (full `i128` range supported).
    #[inline]
    pub const fn from_unix_ns(ns: i128) -> Self {
        Self::from_canonical_attoseconds(ns * (ATTOSEC_PER_NANOSEC as i128), ClockType::UTC)
    }

    // --------------------- GPS / QZSS (1980-01-06 00:00:00 GPST) ---------------------

    /// Creates a `TimePoint` from **seconds** since the traditional GPS epoch
    /// (1980-01-06 00:00:00 GPST). Works for both `GPST` and `QZSST`.
    #[inline]
    pub const fn from_gps_sec(s: i64) -> Self {
        Self::from_canonical_attoseconds((s as i128) * ATTOSEC_PER_SEC_I128, ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_ms(ms: i128) -> Self {
        Self::from_canonical_attoseconds(ms * (ATTOSEC_PER_MILLISEC as i128), ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_us(us: i128) -> Self {
        Self::from_canonical_attoseconds(us * (ATTOSEC_PER_MICROSEC as i128), ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_ns(ns: i128) -> Self {
        Self::from_canonical_attoseconds(ns * (ATTOSEC_PER_NANOSEC as i128), ClockType::GPST)
    }

    // --------------------- Galileo (1999-08-22 00:00:00 GST) ---------------------

    #[inline]
    pub const fn from_galileo_sec(s: i64) -> Self {
        Self::from_canonical_attoseconds((s as i128) * ATTOSEC_PER_SEC_I128, ClockType::GST)
    }

    #[inline]
    pub const fn from_galileo_ms(ms: i128) -> Self {
        Self::from_canonical_attoseconds(ms * (ATTOSEC_PER_MILLISEC as i128), ClockType::GST)
    }

    #[inline]
    pub const fn from_galileo_us(us: i128) -> Self {
        Self::from_canonical_attoseconds(us * (ATTOSEC_PER_MICROSEC as i128), ClockType::GST)
    }

    #[inline]
    pub const fn from_galileo_ns(ns: i128) -> Self {
        Self::from_canonical_attoseconds(ns * (ATTOSEC_PER_NANOSEC as i128), ClockType::GST)
    }

    // --------------------- BeiDou (2006-01-01 00:00:00 BDT) ---------------------

    #[inline]
    pub const fn from_beidou_sec(s: i64) -> Self {
        Self::from_canonical_attoseconds((s as i128) * ATTOSEC_PER_SEC_I128, ClockType::BDT)
    }

    #[inline]
    pub const fn from_beidou_ms(ms: i128) -> Self {
        Self::from_canonical_attoseconds(ms * (ATTOSEC_PER_MILLISEC as i128), ClockType::BDT)
    }

    #[inline]
    pub const fn from_beidou_us(us: i128) -> Self {
        Self::from_canonical_attoseconds(us * (ATTOSEC_PER_MICROSEC as i128), ClockType::BDT)
    }

    #[inline]
    pub const fn from_beidou_ns(ns: i128) -> Self {
        Self::from_canonical_attoseconds(ns * (ATTOSEC_PER_NANOSEC as i128), ClockType::BDT)
    }
}

#[test]
fn test_1972_leap_second_canonical_roundtrip() {
    // Create the leap second the "normal" way (using from_gregorian_ymdhms)
    let original = TimePoint::from_gregorian_ymdhms(1972, 6, 30, 23, 59, 60, 0, ClockType::UTC);

    // Round-trip through canonical attoseconds
    let canon = original.to_canonical_attoseconds();
    let roundtrip = TimePoint::from_canonical_attoseconds(canon, ClockType::UTC);

    // These should be identical if everything is consistent
    assert_eq!(
        original, roundtrip,
        "Round-trip failed for 1972 leap second"
    );

    // Also verify civil time is still correct
    let g = roundtrip.to_gregorian_ymdhms();
    assert_eq!(g.yr, 1972);
    assert_eq!(g.mo, 6);
    assert_eq!(g.day, 30);
    assert_eq!(g.hr, 23);
    assert_eq!(g.min, 59);
    assert_eq!(g.sec, 60, "Should still show sec=60 after round-trip");
}
