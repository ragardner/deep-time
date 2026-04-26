use crate::TimeSpan;
use chrono::{DateTime, Duration, Utc};

impl TimeSpan {
    /// Creates a `TimeSpan` from a `chrono::Duration` / `TimeTimeSpan` (nanosecond precision).
    ///
    /// This is the exact reverse of [`TimeSpan::to_chrono_duration`].
    ///
    /// - The conversion is **lossless** when the chrono duration fits inside an `i64`
    ///   number of nanoseconds.
    /// - Uses existing `from_ns` helper.
    /// - If `num_nanoseconds()` returns `None` (the chrono value is outside the
    ///   range that chrono itself can represent as nanoseconds), we clamp to
    ///   **exactly** the maximum/minimum nanosecond value that chrono can store
    ///   (`i64::MAX` / `i64::MIN` nanoseconds) rather than saturating to
    ///   `TimeSpan::MAX` / `TimeSpan::MIN`.
    #[inline]
    pub fn from_chrono_duration(dur: Duration) -> Self {
        match dur.num_nanoseconds() {
            Some(ns) => Self::from_ns(ns),
            None => {
                let ns = if dur > Duration::zero() {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Self::from_ns(ns)
            }
        }
    }

    /// Creates a `TimeSpan` representing the duration since the Unix epoch
    /// (1970-01-01 00:00:00 UTC) from a `chrono::DateTime<Utc>`.
    ///
    /// This is the exact reverse of [`TimeSpan::to_chrono_datetime_utc`].
    ///
    /// - Returns a `TimeSpan` whose value is the number of nanoseconds since the
    ///   Unix epoch (with sub-nanosecond attoseconds set to zero).
    /// - Uses the safe `timestamp_nanos_opt()` API.
    /// - If the DateTime is outside the nanosecond range chrono can represent,
    ///   we clamp to **exactly** the maximum/minimum nanosecond value that chrono
    ///   itself can store (`i64::MAX` / `i64::MIN` nanoseconds since epoch)
    ///   rather than saturating to `TimeSpan::MAX` / `TimeSpan::MIN`.
    #[inline]
    pub fn from_chrono_datetime_utc(dt: DateTime<Utc>) -> Self {
        match dt.timestamp_nanos_opt() {
            Some(ns) => Self::from_ns(ns),
            None => {
                let ns = if dt > DateTime::<Utc>::UNIX_EPOCH {
                    i64::MAX
                } else {
                    i64::MIN
                };
                Self::from_ns(ns)
            }
        }
    }
}
