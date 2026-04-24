use crate::TimeSpan;
use jiff::{SignedDuration, Span, Timestamp};

impl TimeSpan {
    /// Converts this `TimeSpan` to a [`jiff::Span`] (seconds + nanoseconds only).
    ///
    /// This is the **main/default** conversion method.
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion is fully exact up to the nanosecond (using 128-bit integer arithmetic).
    /// - **Saturates** at the largest/smallest representable `Span` (roughly ±20,000 years)
    ///   if the value is out of range.
    ///   Never returns an error.
    pub fn to_jiff_span(self) -> Span {
        let total_nanos = self.total_attos() / 1_000_000_000i128;

        let seconds = (total_nanos / 1_000_000_000) as i64;
        let nanoseconds = (total_nanos % 1_000_000_000) as i64;

        // Fast path when in range
        if let Ok(base) = Span::new().try_seconds(seconds) {
            if let Ok(span) = base.try_nanoseconds(nanoseconds) {
                return span;
            }
        }

        // Saturate to Jiff's Span limits
        if total_nanos >= 0 {
            Span::new()
                .seconds(631_107_417_600i64)
                .nanoseconds(999_999_999i64)
        } else {
            Span::new()
                .seconds(-631_107_417_600i64)
                .nanoseconds(-999_999_999i64)
        }
    }

    /// Converts this `TimeSpan` to a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - Supports the **entire** range of `TimeSpan` (never saturates).
    #[inline]
    pub fn to_jiff_signed_duration(self) -> SignedDuration {
        let total_nanos = self.total_attos() / 1_000_000_000i128;
        SignedDuration::from_nanos_i128(total_nanos)
    }

    /// Converts this `TimeSpan` to a [`jiff::Timestamp`].
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - The conversion assumes `total_attos()` returns attoseconds since the Unix epoch
    ///   (UTC). Leap-second handling is already performed by `TimePoint` arithmetic.
    /// - **Saturates** at [`Timestamp::MAX`] / [`Timestamp::MIN`] if the value is out of range.
    ///   Never returns an error.
    pub fn to_jiff_timestamp(self) -> Timestamp {
        let total_nanos = self.total_attos() / 1_000_000_000i128;

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
