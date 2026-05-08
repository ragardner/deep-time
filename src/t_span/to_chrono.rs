use crate::{TSpan, clamp_i128_to_i64};
use chrono::{DateTime, Duration, TimeDelta, Utc};

impl TSpan {
    /// Converts this `TSpan` to a `chrono::Duration` (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (128-bit integer arithmetic).
    /// - **Saturates** at `chrono::Duration::MIN` / `chrono::Duration::MAX`
    ///   (roughly ±292 million years) if the value is out of range.
    ///   Never returns an error.
    pub fn to_chrono_duration(self) -> Duration {
        let total_nanos = self.to_attos() / 1_000_000_000i128;
        let nanos = clamp_i128_to_i64(total_nanos);

        // `TimeDelta::nanoseconds` is infallible and returns exactly the
        // `chrono::Duration` alias.
        TimeDelta::nanoseconds(nanos).into()
    }

    /// Converts this `TSpan` to a `chrono::DateTime<chrono::Utc>`.
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion assumes `to_attos()` returns attoseconds since the
    ///   Unix epoch (1970-01-01 00:00:00 UTC). Leap-second handling is already
    ///   performed by `Dt` arithmetic.
    /// - **Saturates** at the minimum/maximum representable `DateTime<Utc>`
    ///   (roughly years 1678–2262) if the value is out of range.
    ///   Never returns an error.
    pub fn to_chrono_datetime_utc(self) -> DateTime<Utc> {
        let total_nanos = self.to_attos() / 1_000_000_000i128;
        let nanos = clamp_i128_to_i64(total_nanos);

        DateTime::<Utc>::from_timestamp_nanos(nanos)
    }
}
