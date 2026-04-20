use crate::Delta;
use chrono::{DateTime, Duration, Utc};

impl Delta {
    /// Creates a `Delta` from a `chrono::Duration` / `TimeDelta` (nanosecond precision).
    ///
    /// This is the exact reverse of [`Delta::to_chrono_duration`].
    ///
    /// - The conversion is **lossless** when the chrono duration fits inside an `i64`
    ///   number of nanoseconds.
    /// - Uses your existing `from_ns` helper (which already does the correct
    ///   attosecond scaling and normalization).
    /// - If `num_nanoseconds()` returns `None` (the chrono value is outside the
    ///   range that chrono itself can represent as nanoseconds), we clamp to
    ///   **exactly** the maximum/minimum nanosecond value that chrono can store
    ///   (`i64::MAX` / `i64::MIN` nanoseconds) rather than saturating to
    ///   `Delta::MAX` / `Delta::MIN`.
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

    /// Creates a `Delta` representing the duration since the Unix epoch
    /// (1970-01-01 00:00:00 UTC) from a `chrono::DateTime<Utc>`.
    ///
    /// This is the exact reverse of [`Delta::to_chrono_datetime_utc`].
    ///
    /// - Returns a `Delta` whose value is the number of nanoseconds since the
    ///   Unix epoch (with sub-nanosecond attoseconds set to zero).
    /// - Uses the safe `timestamp_nanos_opt()` API.
    /// - If the DateTime is outside the nanosecond range chrono can represent,
    ///   we clamp to **exactly** the maximum/minimum nanosecond value that chrono
    ///   itself can store (`i64::MAX` / `i64::MIN` nanoseconds since epoch)
    ///   rather than saturating to `Delta::MAX` / `Delta::MIN`.
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
