use crate::{Dt, DtErr, DtErrKind, Scale, an_err};
use jiff::{SignedDuration, Span, Timestamp};

impl Dt {
    /// Converts this [`Dt`] to a [`jiff::Timestamp`] (always in UTC).
    pub fn to_jiff_timestamp(&self, current: Scale) -> Timestamp {
        let nanos = self.to(current, Scale::UTC).to_ns();

        match Timestamp::from_nanosecond(nanos) {
            Ok(ts) => ts,
            Err(_) => {
                if nanos >= 0 {
                    Timestamp::MAX
                } else {
                    Timestamp::MIN
                }
            }
        }
    }

    /// Converts this `Dt` to a [`jiff::Span`] (seconds + nanoseconds only).
    pub fn to_jiff_span(&self) -> Span {
        let total_nanos = self.to_ns();
        let seconds = Dt::i128_to_i64(total_nanos.div_euclid(1_000_000_000));
        let nanoseconds = Dt::i128_to_i64(total_nanos.rem_euclid(1_000_000_000));

        if let Ok(base) = Span::new().try_seconds(seconds)
            && let Ok(span) = base.try_nanoseconds(nanoseconds)
        {
            return span;
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

    /// Converts this `Span` to a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - Supports the **entire** range of `Span` (never saturates).
    #[inline]
    pub fn to_jiff_signed_duration(&self) -> SignedDuration {
        SignedDuration::from_nanos_i128(self.to_ns())
    }

    /// Creates a `Dt` from a `jiff::Timestamp`.
    ///
    /// This is the inverse of [`Dt::to_jiff_timestamp`].
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Self {
        Dt::from_dt(Dt::from_ns(ts.as_nanosecond(), Scale::TAI), Scale::UTC)
    }

    /// Creates a [`Dt`] from a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// This is the inverse of [`Dt::to_jiff_signed_duration`].
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Self {
        Self::from_ns(dur.as_nanos(), Scale::TAI)
    }

    /// Creates a [`Dt`] from a `jiff::Dt`.
    ///
    /// This is the inverse of [`Dt::to_jiff_span`].
    #[inline]
    pub fn from_jiff_span(span: Span) -> Result<Self, DtErr> {
        let dur = SignedDuration::try_from(span)
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "{:?}: {}", span, e))?;
        Ok(Self::from_jiff_signed_duration(dur))
    }
}
