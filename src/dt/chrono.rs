use crate::{Dt, Scale};
use chrono::{DateTime, Datelike, Duration, TimeDelta, Timelike, Utc};
use core::convert::From;

impl Dt {
    /// Converts this [`Dt`] to a [`chrono::DateTime`].
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at the minimum/maximum representable `DateTime<Utc>`
    ///   (roughly years 1678–2262) if the instant is out of range.
    #[inline]
    pub fn to_chrono_datetime_utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(Dt::to_i64(
            self.target(Scale::UTC).to_unix().to_ns().0,
        ))
    }

    /// Creates a TAI [`Dt`] from a [`chrono::DateTime`].
    ///
    /// This is the inverse of [`Dt::to_chrono_datetime_utc`].
    pub fn from_chrono_datetime_utc(dt: DateTime<Utc>) -> Dt {
        let yr = dt.year() as i64;
        let mo = dt.month().clamp(1, 12) as u8;
        let day = dt.day().clamp(1, 31) as u8;
        let hr = dt.hour().clamp(0, 23) as u8;
        let min = dt.minute().clamp(0, 59) as u8;
        let sec = dt.second().clamp(0, 60) as u8;
        let subsec_nanos = dt.nanosecond();
        let attos = Dt::from_ns(subsec_nanos as i128, 0, Scale::TAI, Scale::TAI).to_attos();

        Dt::from_ymd(yr, mo, day, Scale::UTC, hr, min, sec, Dt::to_u64(attos))
    }

    /// Creates a [`Dt`] from a [`chrono::Duration`] (nanosecond precision).
    pub fn from_chrono_duration(dur: Duration) -> Dt {
        match dur.num_nanoseconds() {
            Some(ns) => Self::from_ns(ns as i128, 0, Scale::TAI, Scale::TAI),
            None => {
                let ns = if dur > Duration::zero() {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Self::from_ns(ns as i128, 0, Scale::TAI, Scale::TAI)
            }
        }
    }

    /// Converts this [`Dt`] to a `chrono::Duration` (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (128-bit integer arithmetic).
    /// - **Saturates** at `chrono::Duration::MIN` / `chrono::Duration::MAX`
    ///   (roughly ±292 million years) if the value is out of range.
    #[inline]
    pub fn to_chrono_duration(&self) -> Duration {
        TimeDelta::nanoseconds(Dt::to_i64(self.to_ns().0))
    }
}

impl From<DateTime<Utc>> for Dt {
    #[inline]
    fn from(dt: DateTime<Utc>) -> Self {
        Self::from_chrono_datetime_utc(dt)
    }
}

impl From<Dt> for DateTime<Utc> {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_chrono_datetime_utc()
    }
}

impl From<Duration> for Dt {
    #[inline]
    fn from(dur: Duration) -> Self {
        Self::from_chrono_duration(dur)
    }
}

impl From<Dt> for Duration {
    #[inline]
    fn from(dt: Dt) -> Self {
        dt.to_chrono_duration()
    }
}
