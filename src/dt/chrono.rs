use crate::{Dt, Scale};
use chrono::{DateTime, Datelike, Duration, TimeDelta, Timelike, Utc};

/// Clamps an `i128` to the representable range of `u64`.
#[inline]
fn clamp_i128_to_u64(x: i128) -> u64 {
    if x > u64::MAX as i128 {
        u64::MAX
    } else if x < u64::MIN as i128 {
        u64::MIN
    } else {
        x as u64
    }
}

impl Dt {
    /// Converts this [`Dt`] to a [`chrono::DateTime`].
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at the minimum/maximum representable `DateTime<Utc>`
    ///   (roughly years 1678–2262) if the instant is out of range.
    #[inline]
    pub fn to_chrono_datetime_utc(&self, current: Scale) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(Dt::i128_to_i64(
            self.to_unix(current, Scale::UTC).to_ns(),
        ))
    }

    /// Creates a TAI [`Dt`] from a [`chrono::DateTime`].
    ///
    /// This is the inverse of [`Dt::to_chrono_datetime_utc`].
    pub fn from_chrono_datetime_utc(dt: DateTime<Utc>) -> Self {
        let yr = dt.year() as i64;
        let mo = dt.month().clamp(1, 12) as u8;
        let day = dt.day().clamp(1, 31) as u8;
        let hr = dt.hour().clamp(0, 23) as u8;
        let min = dt.minute().clamp(0, 59) as u8;
        let sec = dt.second().clamp(0, 60) as u8;
        let subsec_nanos = dt.nanosecond();
        let attos = Dt::from_ns(subsec_nanos as i128, Scale::TAI).to_attos();

        Dt::from_ymdhms_on(
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            clamp_i128_to_u64(attos),
            Scale::UTC,
        )
    }

    /// Creates a [`Dt`] from a [`chrono::Duration`] (nanosecond precision).
    pub fn from_chrono_duration(dur: Duration) -> Self {
        match dur.num_nanoseconds() {
            Some(ns) => Self::from_ns(ns as i128, Scale::TAI),
            None => {
                let ns = if dur > Duration::zero() {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Self::from_ns(ns as i128, Scale::TAI)
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
        TimeDelta::nanoseconds(Dt::i128_to_i64(self.to_ns()))
    }
}
