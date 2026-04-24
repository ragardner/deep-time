use crate::TimePoint;
use jiff::Timestamp;

impl TimePoint {
    /// Converts this `TimePoint` to a [`jiff::Timestamp`] (always in UTC).
    ///
    /// This is the main/default conversion method for absolute instants.
    ///
    /// - The `TimePoint` is first converted to TAI internally (respecting all
    ///   clock types, leap seconds, and relativistic models).
    /// - The duration since the Unix epoch (1970-01-01 00:00:00 UTC) is then
    ///   computed.
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at [`Timestamp::MIN`] / [`Timestamp::MAX`] if the instant
    ///   is outside the range supported by Jiff (roughly years 0000–9999).
    ///   Never returns an error.
    pub fn to_jiff_timestamp(self) -> Timestamp {
        let span_since_epoch = self.duration_since_ref(&TimePoint::UNIX_EPOCH_TAI);
        let total_nanos = span_since_epoch.total_attos() / 1_000_000_000i128;

        match Timestamp::from_nanosecond(total_nanos) {
            Ok(ts) => ts,
            Err(_) => {
                if total_nanos >= 0 {
                    Timestamp::MAX
                } else {
                    Timestamp::MIN
                }
            }
        }
    }
}
