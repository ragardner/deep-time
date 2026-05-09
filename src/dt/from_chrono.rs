use crate::{Dt, Scale};
use chrono::{DateTime, Duration, Utc};

impl Dt {
    /// Creates a `Dt` from a `chrono::DateTime<chrono::Utc>`.
    ///
    /// This is the exact reverse of [`Dt::to_chrono_datetime_utc`].
    ///
    /// - The resulting `Dt` is expressed in the TAI scale
    ///   (the library's canonical internal scale).
    /// - Sub-nanosecond attoseconds are set to zero.
    /// - If the `DateTime` is outside the range representable as an `i64`
    ///   number of nanoseconds since the Unix epoch, the value is clamped
    ///   to exactly the maximum/minimum nanosecond value (`i64::MAX` /
    ///   `i64::MIN` ns) rather than saturating to `Dt` extremes.
    pub fn from_chrono_datetime_utc(dt: DateTime<Utc>) -> Self {
        match dt.timestamp_nanos_opt() {
            Some(ns) => Dt::UNIX_EPOCH.add(Dt::from_ns(ns as i128, Scale::TAI)),
            None => {
                let ns = if dt > DateTime::<Utc>::UNIX_EPOCH {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Dt::UNIX_EPOCH.add(Dt::from_ns(ns as i128, Scale::TAI))
            }
        }
    }

    /// Creates a `Dt` from a `chrono::Duration` / `TimeSpan` (nanosecond precision).
    ///
    /// This is the exact reverse of [`Dt::to_chrono_duration`].
    ///
    /// - The conversion is **lossless** when the chrono duration fits inside an `i64`
    ///   number of nanoseconds.
    /// - Uses existing `from_ns` helper.
    /// - If `num_nanoseconds()` returns `None` (the chrono value is outside the
    ///   range that chrono itself can represent as nanoseconds), we clamp to
    ///   **exactly** the maximum/minimum nanosecond value that chrono can store
    ///   (`i64::MAX` / `i64::MIN` nanoseconds) rather than saturating to
    ///   `Dt::MAX` / `Dt::MIN`.
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
}
