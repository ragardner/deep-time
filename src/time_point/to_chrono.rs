use crate::TimePoint;
use chrono::{DateTime, Utc};

impl TimePoint {
    /// Converts this `TimePoint` to a `chrono::DateTime<chrono::Utc>`.
    ///
    /// This is the main/default conversion method for absolute instants.
    ///
    /// - The `TimePoint` is first converted to TAI internally (respecting all
    ///   clock types, leap seconds, and relativistic models).
    /// - The duration since the Unix epoch (1970-01-01 00:00:00 UTC) is then
    ///   computed.
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at the minimum/maximum representable `DateTime<Utc>`
    ///   (roughly years 1678–2262) if the instant is out of range.
    ///   Never returns an error.
    pub fn to_chrono_datetime_utc(self) -> DateTime<Utc> {
        let span_since_epoch = self.to_tai_since_ref(&TimePoint::UNIX_EPOCH);

        let total_nanos = span_since_epoch.to_attos() / 1_000_000_000i128;

        let nanos = if total_nanos > i64::MAX as i128 {
            i64::MAX
        } else if total_nanos < i64::MIN as i128 {
            i64::MIN
        } else {
            total_nanos as i64
        };

        DateTime::<Utc>::from_timestamp_nanos(nanos)
    }
}
