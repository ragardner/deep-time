use crate::{TimePoint, TimeSpan};
use chrono::{DateTime, Utc};

impl TimePoint {
    /// Creates a `TimePoint` from a `chrono::DateTime<chrono::Utc>`.
    ///
    /// This is the exact reverse of [`TimePoint::to_chrono_datetime_utc`].
    ///
    /// - The resulting `TimePoint` is expressed in the TAI clock type
    ///   (the library's canonical internal scale).
    /// - Sub-nanosecond attoseconds are set to zero.
    /// - If the `DateTime` is outside the range representable as an `i64`
    ///   number of nanoseconds since the Unix epoch, the value is clamped
    ///   to exactly the maximum/minimum nanosecond value (`i64::MAX` /
    ///   `i64::MIN` ns) rather than saturating to `TimePoint` extremes.
    pub fn from_chrono_datetime_utc(dt: DateTime<Utc>) -> Self {
        match dt.timestamp_nanos_opt() {
            Some(ns) => TimePoint::UNIX_EPOCH_TAI.add(TimeSpan::from_ns(ns as i128)),
            None => {
                let ns = if dt > DateTime::<Utc>::UNIX_EPOCH {
                    i64::MAX
                } else {
                    i64::MIN
                };
                TimePoint::UNIX_EPOCH_TAI.add(TimeSpan::from_ns(ns as i128))
            }
        }
    }
}
