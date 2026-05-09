use crate::{Dt, clamp_i128_to_i64};
use chrono::{DateTime, Utc};

impl Dt {
    /// Converts this `Dt` to a `chrono::DateTime<chrono::Utc>`.
    ///
    /// This is the main/default conversion method for absolute instants.
    ///
    /// - The `Dt` is first converted to TAI internally (respecting all
    ///   scales, leap seconds, and relativistic models).
    /// - The duration since the Unix epoch (1970-01-01 00:00:00 UTC) is then
    ///   computed.
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at the minimum/maximum representable `DateTime<Utc>`
    ///   (roughly years 1678–2262) if the instant is out of range.
    ///   Never returns an error.
    pub fn to_chrono_datetime_utc(self) -> DateTime<Utc> {
        let span_since_epoch = self.to_diff_raw(Dt::UNIX_EPOCH);
        let total_nanos = span_since_epoch.to_attos() / 1_000_000_000i128;
        let nanos = clamp_i128_to_i64(total_nanos);

        DateTime::<Utc>::from_timestamp_nanos(nanos)
    }
}
