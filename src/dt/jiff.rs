use crate::{Dt, DtErr, DtErrKind, Scale, an_err};
use jiff::{SignedDuration, Span, Timestamp};

impl Dt {
    /// Converts this [`Dt`] to a [`jiff::Timestamp`].
    ///
    /// ## Time scale
    ///
    /// [`jiff::Timestamp`] is a **Unix / POSIX** instant: nanoseconds since
    /// `1970-01-01 00:00:00Z`. Jiff documents this as “the Unix timescale with a
    /// UTC offset of zero” and **does not support leap seconds** (it behaves as if
    /// they do not exist in the numeric count). Conversion therefore goes through
    /// [`Scale::UTC`](crate::Scale::UTC) and the Unix epoch so deep-time's
    /// leap-second tables are applied on the way out of TAI storage.
    ///
    /// This is **not** a TAI timestamp. A [`Dt`] stored on TAI (or any other scale)
    /// is converted to the equivalent UTC civil instant before the Unix count is
    /// taken.
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at [`Timestamp::MIN`] / [`Timestamp::MAX`] if out of range
    ///   (jiff's supported range is roughly years −9999…9999).
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

    /// Creates a TAI [`Dt`] from a [`jiff::Timestamp`].
    ///
    /// Inverse of [`Dt::to_jiff_timestamp`]. The Unix nanosecond count is treated
    /// as a POSIX/UTC elapsed time since the Unix epoch (not TAI, and not as a
    /// count since J2000), then converted into deep-time's TAI storage via
    /// [`Dt::from_unix`].
    #[inline]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Dt {
        Dt::from_unix_ns(ts.as_nanosecond())
    }

    /// Converts this [`Dt`] to a [`jiff::Span`] (seconds + nanoseconds only).
    ///
    /// ## Time scale
    ///
    /// A [`Span`] built this way is a pure elapsed span of SI nanoseconds taken
    /// from this [`Dt`]'s raw attosecond count. It is **not** tied to UTC leap
    /// seconds or calendar units (years/months/days are left at zero).
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero.
    /// - Saturates at jiff's `Span` second/nanosecond limits
    ///   (`±631_107_417_600` s and `±999_999_999` ns) if out of range.
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

    /// Converts this [`Dt`] to a [`jiff::SignedDuration`] (nanosecond precision).
    ///
    /// ## Time scale
    ///
    /// A [`SignedDuration`] is a pure elapsed span (SI seconds + nanoseconds). It
    /// is **not** tied to UTC, TAI, or any other time scale. The conversion uses
    /// this [`Dt`]'s raw attosecond count (same convention as chrono/`time` duration
    /// interop).
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are **truncated toward zero**.
    /// - Supports the **entire** range of [`Dt`]'s nanosecond projection
    ///   (`SignedDuration::from_nanos_i128`; never saturates here).
    #[inline]
    pub fn to_jiff_signed_duration(&self) -> SignedDuration {
        SignedDuration::from_nanos_i128(self.to_ns().0)
    }

    /// Creates a [`Dt`] from a [`jiff::SignedDuration`] (nanosecond precision).
    ///
    /// Inverse of [`Dt::to_jiff_signed_duration`]. The result is a span stored on
    /// TAI (no leap-second adjustment of the duration itself), matching
    /// chrono/`time` duration interop.
    #[inline]
    pub fn from_jiff_signed_duration(dur: SignedDuration) -> Dt {
        Self::from_ns_floor(dur.as_nanos(), 0, Scale::TAI)
    }

    /// Creates a [`Dt`] from a [`jiff::Span`] (nanosecond precision).
    ///
    /// Inverse of [`Dt::to_jiff_span`]. Converts the span to a
    /// [`SignedDuration`] (seconds + nanoseconds only; calendar units must already
    /// be zero or convertible without a relative datetime) and then uses
    /// [`Dt::from_jiff_signed_duration`].
    pub fn from_jiff_span(span: Span) -> Result<Self, DtErr> {
        let dur = SignedDuration::try_from(span)
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "{}", e))?;
        Ok(Self::from_jiff_signed_duration(dur))
    }
}
