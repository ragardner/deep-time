use crate::{Dt, Scale, clamp_i128_to_i64};
use chrono::{DateTime, Duration, TimeDelta, Utc};

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
    pub fn to_chrono_datetime_utc(&self) -> DateTime<Utc> {
        let span_since_epoch = self.to_diff_raw(Dt::UNIX_EPOCH);
        let total_nanos = span_since_epoch.to_attos() / 1_000_000_000i128;
        let nanos = clamp_i128_to_i64(total_nanos);

        DateTime::<Utc>::from_timestamp_nanos(nanos)
    }

    /// Converts this `Span` to a `chrono::Duration` (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (128-bit integer arithmetic).
    /// - **Saturates** at `chrono::Duration::MIN` / `chrono::Duration::MAX`
    ///   (roughly ±292 million years) if the value is out of range.
    ///   Never returns an error.
    pub fn to_chrono_duration(&self) -> Duration {
        let total_nanos = self.to_attos() / 1_000_000_000i128;
        let nanos = clamp_i128_to_i64(total_nanos);

        // `TimeDelta::nanoseconds` is infallible and returns exactly the
        // `chrono::Duration` alias.
        TimeDelta::nanoseconds(nanos).into()
    }
}
