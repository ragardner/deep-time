use crate::{Dt, DtErr, DtErrKind, Scale, an_err};
use jiff::{SignedDuration, Span, Timestamp};

impl Dt {
    /// Converts this [`Dt`] to a [`jiff::Timestamp`].
    pub fn to_jiff_timestamp(&self) -> Timestamp {
        let nanos = self.target(Scale::UTC).to_unix().to_ns().0;

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

    /// Creates a [`Dt`] from a [`jiff::Timestamp`].
    ///
    /// This is the inverse of [`Dt::to_jiff_timestamp`].
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Dt {
        Dt::from_diff_and_scale(
            Dt::from_ns_floor(ts.as_nanosecond(), 0, Scale::UTC),
            Self::UNIX_EPOCH,
            false,
        )
    }

    /// Converts this [`Dt`] to a [`jiff::Span`] (seconds + nanoseconds only).
    pub fn to_jiff_span(&self) -> Span {
        let (total_nanos, _) = self.to_ns();
        let seconds = Dt::to_i64(total_nanos / 1_000_000_000);
        let nanoseconds = Dt::to_i64(total_nanos % 1_000_000_000);

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
        SignedDuration::from_nanos_i128(self.to_ns().0)
    }

    /// Creates a [`Dt`] from a `jiff::SignedDuration` (nanosecond precision).
    ///
    /// This is the inverse of [`Dt::to_jiff_signed_duration`].
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Dt {
        Self::from_ns_floor(dur.as_nanos(), 0, Scale::TAI)
    }

    /// Creates a [`Dt`] from a `jiff::Dt`.
    ///
    /// This is the inverse of [`Dt::to_jiff_span`].
    #[inline]
    pub fn from_jiff_span(span: Span) -> Result<Self, DtErr> {
        let dur = SignedDuration::try_from(span)
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "{}", e))?;
        Ok(Self::from_jiff_signed_duration(dur))
    }
}
