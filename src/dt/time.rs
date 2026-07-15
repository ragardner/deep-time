use crate::{Dt, Scale};
use core::convert::From;
use time::{Duration, OffsetDateTime, Timestamp, UtcDateTime, UtcOffset};

impl Dt {
    /// Converts this [`Dt`] to a [`time::Timestamp`].
    ///
    /// ## Time scale
    ///
    /// [`time::Timestamp`] is a **Unix / POSIX** instant: nanoseconds since
    /// 1970-01-01 00:00:00 UTC. Like `chrono` and `jiff`, it does **not** count
    /// leap seconds in the numeric value. Conversion therefore goes through
    /// [`Scale::UTC`](../enum.Scale.html#variant.UTC) and the Unix epoch so deep-time's leap-second
    /// tables are applied on the way in and out of TAI storage.
    ///
    /// This is **not** a TAI timestamp.
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at [`Timestamp::MIN`] / [`Timestamp::MAX`] if out of range
    ///   (year range depends on the `time` crate's `large-dates` feature; without
    ///   it, roughly ±9999).
    pub fn to_time_timestamp(&self) -> Timestamp {
        let nanos = self.target(Scale::UTC).to_unix().to_ns().0;
        match Timestamp::from_nanoseconds(nanos) {
            Ok(ts) => ts,
            Err(_) => {
                if nanos >= 0 {
                    Timestamp::MAX
                } else {
                    Timestamp::MIN
                }
            }
        }
    }

    /// Creates a TAI [`Dt`] from a [`time::Timestamp`].
    ///
    /// Inverse of [`Dt::to_time_timestamp`]. The Unix nanosecond count is treated
    /// as a POSIX/UTC elapsed time since the Unix epoch (not TAI), then converted
    /// into deep-time's TAI storage via [`Dt::from_unix`].
    #[inline]
    pub fn from_time_timestamp(ts: Timestamp) -> Dt {
        Self::from_unix_ns(ts.as_nanoseconds())
    }

    /// Converts this [`Dt`] to a [`time::OffsetDateTime`] with a UTC offset of zero.
    ///
    /// ## Time scale
    ///
    /// Same Unix/POSIX UTC semantics as [`Dt::to_time_timestamp`]. The returned value
    /// always has [`UtcOffset::UTC`]; wall-clock fields are UTC civil time, not TAI.
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at the minimum/maximum representable `OffsetDateTime` in UTC
    ///   (via [`Timestamp::MIN`] / [`Timestamp::MAX`]).
    #[inline]
    pub fn to_time_offset_datetime_utc(&self) -> OffsetDateTime {
        self.to_time_timestamp().to_offset(UtcOffset::UTC)
    }

    /// Creates a TAI [`Dt`] from a [`time::OffsetDateTime`].
    ///
    /// Uses [`OffsetDateTime::unix_timestamp_nanos`], so the absolute instant is
    /// taken correctly regardless of the value's fixed offset. The offset itself
    /// is not preserved on the resulting [`Dt`] (deep-time stores scale, not
    /// civil zone offset).
    ///
    /// Same POSIX/UTC Unix semantics as [`Dt::from_time_timestamp`].
    #[inline]
    pub fn from_time_offset_datetime(dt: OffsetDateTime) -> Dt {
        Self::from_unix_ns(dt.unix_timestamp_nanos())
    }

    /// Converts this [`Dt`] to a [`time::UtcDateTime`].
    ///
    /// ## Time scale
    ///
    /// Same Unix/POSIX UTC semantics as [`Dt::to_time_timestamp`]. Fields are UTC
    /// civil time, not TAI.
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at [`UtcDateTime::MIN`] / [`UtcDateTime::MAX`] if out of range.
    #[inline]
    pub fn to_time_utc_datetime(&self) -> UtcDateTime {
        self.to_time_timestamp().to_utc()
    }

    /// Creates a TAI [`Dt`] from a [`time::UtcDateTime`].
    ///
    /// Inverse of [`Dt::to_time_utc_datetime`]. Same POSIX/UTC Unix semantics as
    /// [`Dt::from_time_timestamp`].
    #[inline]
    pub fn from_time_utc_datetime(dt: UtcDateTime) -> Dt {
        Self::from_unix_ns(dt.unix_timestamp_nanos())
    }

    /// Converts this [`Dt`] to a [`time::Duration`] (nanosecond precision).
    ///
    /// ## Time scale
    ///
    /// A [`time::Duration`] is a pure elapsed span (SI seconds + nanoseconds). It is
    /// **not** tied to UTC, TAI, or any other time scale. The conversion uses this
    /// [`Dt`]'s raw attosecond count (same convention as chrono/jiff duration interop).
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at [`Duration::MIN`] / [`Duration::MAX`]
    ///   (roughly ±292 billion years) if out of range.
    pub fn to_time_duration(&self) -> Duration {
        let total_nanos = self.to_ns().0;
        let max_ns = Duration::MAX.whole_nanoseconds();
        let min_ns = Duration::MIN.whole_nanoseconds();
        if total_nanos >= max_ns {
            Duration::MAX
        } else if total_nanos <= min_ns {
            Duration::MIN
        } else {
            Duration::nanoseconds_i128(total_nanos)
        }
    }

    /// Creates a [`Dt`] from a [`time::Duration`] (nanosecond precision).
    ///
    /// Inverse of [`Dt::to_time_duration`]. The result is a span stored on TAI
    /// (no leap-second adjustment of the duration itself), matching chrono/jiff
    /// duration interop.
    #[inline]
    pub fn from_time_duration(dur: Duration) -> Dt {
        Self::from_ns(dur.whole_nanoseconds(), 0, Scale::TAI, Scale::TAI)
    }
}

impl From<Timestamp> for Dt {
    #[inline]
    fn from(ts: Timestamp) -> Self {
        Self::from_time_timestamp(ts)
    }
}

impl From<Dt> for Timestamp {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_time_timestamp()
    }
}

impl From<OffsetDateTime> for Dt {
    #[inline]
    fn from(dt: OffsetDateTime) -> Self {
        Self::from_time_offset_datetime(dt)
    }
}

impl From<Dt> for OffsetDateTime {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_time_offset_datetime_utc()
    }
}

impl From<UtcDateTime> for Dt {
    #[inline]
    fn from(dt: UtcDateTime) -> Self {
        Self::from_time_utc_datetime(dt)
    }
}

impl From<Dt> for UtcDateTime {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_time_utc_datetime()
    }
}

impl From<Duration> for Dt {
    #[inline]
    fn from(dur: Duration) -> Self {
        Self::from_time_duration(dur)
    }
}

impl From<Dt> for Duration {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_time_duration()
    }
}
