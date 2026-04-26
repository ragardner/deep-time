use crate::{
    ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC, ATTOSEC_PER_NANOSEC, ATTOSEC_PER_SEC_I128,
    ClockType, TimePoint, TimeSpan,
};

impl TimePoint {
    /// Inverse of [`Self::to_canonical_attoseconds`].
    ///
    /// Creates a `TimePoint` that is exactly `attos` attoseconds after the
    /// globally-expected canonical epoch of the requested `clock_type`.
    #[inline]
    pub const fn from_canonical_attoseconds(attos: i128, clock_type: ClockType) -> Self {
        match clock_type {
            ClockType::UTC => Self::UNIX_EPOCH_UTC
                .add(TimeSpan::from_total_attos(attos))
                .to_tai(),
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
