use crate::{Dt, DtErr, DtErrKind, Scale, an_err};
use jiff::{SignedDuration, Span, Timestamp};

impl Dt {
    /// Creates a `Dt` from a `jiff::Timestamp`.
    ///
    /// This is the exact reverse of [`Dt::to_jiff_timestamp`].
    ///
    /// `jiff::Timestamp` is the primary absolute instant type in the Jiff
    /// ecosystem (broadly convertible to `Zoned`, civil datetimes, etc.).
    ///
    /// - The resulting `Dt` is expressed in the TAI scale
    ///   (the library's canonical internal scale).
    /// - Sub-nanosecond attoseconds are set to zero.
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        Dt::UNIX_EPOCH.add(Dt::from_ns(ts.as_nanosecond(), Scale::TAI))
    }

    /// Creates a `Dt` from a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// This is the exact reverse of [`Dt::to_jiff_signed_duration`].
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Self {
        Self::from_ns(dur.as_nanos(), Scale::TAI)
    }

    /// Creates a `Dt` from a `jiff::Dt`.
    ///
    /// This is the exact reverse of [`Dt::to_jiff_span`].
    ///
    /// - Works perfectly for pure time-based `Dt`s (seconds + nanoseconds only).
    /// - Returns `Err` if the `Dt` contains any calendar units (years, months,
    ///   weeks, days, etc.) that cannot be converted to a pure elapsed-time
    ///   duration.
    #[inline]
    pub fn from_jiff_span(span: Span) -> Result<Self, DtErr> {
        let dur = SignedDuration::try_from(span)
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "{:?}: {}", span, e))?;
        Ok(Self::from_jiff_signed_duration(dur))
    }
}
