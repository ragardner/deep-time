use crate::{ATTOS_PER_NS, Dt, DtErr, DtErrKind, Scale, an_err};
use core::convert::{From, TryFrom};
use icu_calendar::{Date, Iso};
use icu_time::{DateTime, Time};

impl Dt {
    /// Converts this [`Dt`] to an ICU4X [`DateTime`]`<`[`Iso`]`>` (civil ISO date + time).
    ///
    /// ## Time scale
    ///
    /// Fields are **UTC civil** wall time: the instant is converted to
    /// [`Scale::UTC`](../enum.Scale.html#variant.UTC) (applying leap-second tables as
    /// usual) before the calendar date and clock fields are read. This matches the
    /// UTC convention used by [`Dt::to_chrono_datetime_utc`] and the Unix-based
    /// jiff/`time` interop paths.
    ///
    /// ## Precision and range
    ///
    /// - Sub-nanosecond attoseconds are truncated toward zero into
    ///   [`icu_time::Nanosecond`] (`0..=999_999_999`).
    /// - Leap seconds are preserved when UTC civil time shows `sec == 60`
    ///   (`Time` allows seconds `0..=60`).
    /// - Year must fit ICU's ISO constructor range (`-9999..=9999`); otherwise
    ///   returns [`DtErrKind::YearOutOfRange`].
    /// - Other out-of-range civil fields map to [`DtErrKind::InvalidDate`] or
    ///   [`DtErrKind::InvalidTime`].
    ///
    /// ## Example
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_ymd(2024, 4, 15, Scale::UTC, 14, 30, 45, 123_456_789_000_000_000);
    /// let icu = dt.to_icu_datetime_iso().unwrap();
    /// assert_eq!(icu.date.era_year().extended_year, 2024);
    /// assert_eq!(icu.date.month().ordinal, 4);
    /// assert_eq!(icu.date.day_of_month().0, 15);
    /// assert_eq!(icu.time.hour.number(), 14);
    /// assert_eq!(icu.time.minute.number(), 30);
    /// assert_eq!(icu.time.second.number(), 45);
    /// assert_eq!(icu.time.subsecond.number(), 123_456_789);
    /// ```
    pub fn to_icu_datetime_iso(&self) -> Result<DateTime<Iso>, DtErr> {
        let ymd = self.target(Scale::UTC).to_ymd();
        let year_i32: i32 = ymd
            .yr()
            .try_into()
            .map_err(|_| an_err!(DtErrKind::YearOutOfRange, "year={}", ymd.yr()))?;

        let date = Date::try_new_iso(year_i32, ymd.mo(), ymd.day())
            .map_err(|_| an_err!(DtErrKind::InvalidDate, "ymd"))?;

        let nanos = (ymd.attos() / ATTOS_PER_NS) as u32;
        let time = Time::try_new(ymd.hr(), ymd.min(), ymd.sec(), nanos)
            .map_err(|_| an_err!(DtErrKind::InvalidTime, "hms.ns"))?;

        Ok(DateTime { date, time })
    }

    /// Creates a TAI [`Dt`] from an ICU4X [`DateTime`]`<`[`Iso`]`>`.
    ///
    /// Inverse of [`Dt::to_icu_datetime_iso`]. Civil fields are interpreted as
    /// **UTC** wall time (same convention as [`Dt::from_chrono_datetime_utc`]),
    /// then stored on TAI via [`Dt::from_ymd`].
    ///
    /// ## Precision
    ///
    /// Nanoseconds are expanded to attoseconds (`ns * 10⁹`). Values below one
    /// nanosecond that were truncated on the way out are not recovered.
    ///
    /// ## Example
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    /// use icu_calendar::Date;
    /// use icu_time::{DateTime, Time};
    ///
    /// let icu = DateTime {
    ///     date: Date::try_new_iso(2024, 4, 15).unwrap(),
    ///     time: Time::try_new(14, 30, 45, 123_456_789).unwrap(),
    /// };
    /// let dt = Dt::from_icu_datetime_iso(icu);
    /// let ymd = dt.target(Scale::UTC).to_ymd();
    /// assert_eq!((ymd.yr(), ymd.mo(), ymd.day()), (2024, 4, 15));
    /// assert_eq!((ymd.hr(), ymd.min(), ymd.sec()), (14, 30, 45));
    /// assert_eq!(ymd.attos(), 123_456_789_000_000_000);
    /// ```
    pub fn from_icu_datetime_iso(dt: DateTime<Iso>) -> Dt {
        let yr = i64::from(dt.date.era_year().extended_year);
        let mo = dt.date.month().ordinal;
        let day = dt.date.day_of_month().0;
        let hr = dt.time.hour.number();
        let min = dt.time.minute.number();
        let sec = dt.time.second.number();
        let attos = u64::from(dt.time.subsecond.number()).saturating_mul(ATTOS_PER_NS);

        Dt::from_ymd(yr, mo, day, Scale::UTC, hr, min, sec, attos)
    }
}

impl TryFrom<Dt> for DateTime<Iso> {
    type Error = DtErr;

    #[inline]
    fn try_from(dt: Dt) -> Result<Self, Self::Error> {
        dt.to_icu_datetime_iso()
    }
}

impl From<DateTime<Iso>> for Dt {
    #[inline]
    fn from(dt: DateTime<Iso>) -> Self {
        Self::from_icu_datetime_iso(dt)
    }
}
