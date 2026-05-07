use crate::{DtErrKind, DtErr, TSpan, an_err};
use jiff::{SignedDuration, Span, Timestamp};

impl TSpan {
    /// Creates a `TSpan` from a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// This is the exact reverse of [`TSpan::to_jiff_signed_duration`].
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Self {
        Self::from_ns(dur.as_nanos())
    }

    /// Creates a `TSpan` from a `jiff::Span`.
    ///
    /// This is the exact reverse of [`TSpan::to_jiff_span`].
    ///
    /// - Works perfectly for pure time-based `Span`s (seconds + nanoseconds only).
    /// - Returns `Err` if the `Span` contains any calendar units (years, months,
    ///   weeks, days, etc.) that cannot be converted to a pure elapsed-time
    ///   duration.
    #[inline]
    pub fn from_jiff_span(span: Span) -> Result<Self, DtErr> {
        let dur = SignedDuration::try_from(span)
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "{:?}: {}", span, e))?;
        Ok(Self::from_jiff_signed_duration(dur))
    }

    /// Creates a `TSpan` representing the duration since the Unix epoch
    /// (1970-01-01 00:00:00 UTC) from a `jiff::Timestamp`.
    ///
    /// This is the exact reverse of [`TSpan::to_jiff_timestamp`].
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        Self::from_ns(ts.as_nanosecond())
    }
}
