use crate::{DtStdError, TimeSpan};
use alloc::string::ToString;
use jiff::{SignedDuration, Span, Timestamp};

impl TimeSpan {
    /// Creates a `TimeSpan` from a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// This is the exact reverse of [`TimeSpan::to_jiff_signed_duration`].
    ///
    /// - The conversion is lossless when the value fits in an `i64` number of
    ///   nanoseconds.
    /// - If the `SignedDuration` exceeds the range that can be represented as
    ///   an `i64` number of nanoseconds, we clamp to **exactly** the
    ///   maximum/minimum nanosecond value (`i64::MAX` / `i64::MIN` ns).
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Self {
        let nanos = dur.as_nanos();

        let ns = if nanos > i64::MAX as i128 {
            i64::MAX
        } else if nanos < i64::MIN as i128 {
            i64::MIN
        } else {
            nanos as i64
        };

        Self::from_ns(ns)
    }

    /// Creates a `TimeSpan` from a `jiff::Span`.
    ///
    /// This is the exact reverse of [`TimeSpan::to_jiff_span`].
    ///
    /// - Works perfectly for pure time-based `Span`s (seconds + nanoseconds only).
    /// - Returns `Err` if the `Span` contains any calendar units (years, months,
    ///   weeks, days, etc.) that cannot be converted to a pure elapsed-time
    ///   duration.
    pub fn from_jiff_span(span: Span) -> Result<Self, DtStdError> {
        let dur = SignedDuration::try_from(span).map_err(|e| DtStdError::reason(e.to_string()))?;

        Ok(Self::from_jiff_signed_duration(dur))
    }

    /// Creates a `TimeSpan` representing the duration since the Unix epoch
    /// (1970-01-01 00:00:00 UTC) from a `jiff::Timestamp`.
    ///
    /// This is the exact reverse of [`TimeSpan::to_jiff_timestamp`].
    ///
    /// - If the timestamp is extremely far in the future or past, we clamp to
    ///   **exactly** the maximum/minimum nanosecond value (`i64::MAX` /
    ///   `i64::MIN` ns).
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        let nanos = ts.as_nanosecond();

        let ns = if nanos > i64::MAX as i128 {
            i64::MAX
        } else if nanos < i64::MIN as i128 {
            i64::MIN
        } else {
            nanos as i64
        };

        Self::from_ns(ns)
    }
}
