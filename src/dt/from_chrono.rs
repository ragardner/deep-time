use crate::{Dt, TSpan};
use chrono::{DateTime, Utc};

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
            Some(ns) => Dt::UNIX_EPOCH.add(TSpan::from_ns(ns as i128)),
            None => {
                let ns = if dt > DateTime::<Utc>::UNIX_EPOCH {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Dt::UNIX_EPOCH.add(TSpan::from_ns(ns as i128))
            }
        }
    }
}
