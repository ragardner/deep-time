use crate::{ATTOS_PER_SEC_I128, Dt, Scale};
#[cfg(feature = "jiff-tz")]
use crate::{DtErr, DtErrKind, an_err};

mod to_str;

/// Combined Gregorian date + wall time with subsecond precision.
/// Has calendar aware and, when the `jiff-tz` feature is enabled,
/// timezone aware math functions.
///
/// ## Examples
///
/// **Creating a** [`YmdHms`].
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// // clamped to 29
/// let x = Dt::from_ymd(2000, 2, 30, 0, 0, 0, 0, Scale::UTC).to_ymd();
///
/// assert_eq!(x.day(), 29);
/// ```
///
/// **Adding a year.** 2000 is a leap year and Feb. 29th is possible, but
/// 2001 isn't a leap year so the day is clamped to the 28th.
///
/// ```rust
/// use deep_time::{Dt, Scale};
///
/// let x = Dt::from_ymd(2000, 2, 29, 0, 0, 0, 0, Scale::UTC).to_ymd();
/// let x = x.add_yr(1);
///
/// assert_eq!(x.day(), 28);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct YmdHms {
    pub(crate) yr: i64,
    pub(crate) mo: u8,
    pub(crate) day: u8,
    pub(crate) hr: u8,
    pub(crate) min: u8,
    pub(crate) sec: u8,    // 0–60 (60 only during leap seconds)
    pub(crate) attos: u64, // attoseconds (0 ≤ subsec < 10¹⁸)
    pub(crate) scale: Scale,
}

impl YmdHms {
    /// Reconstructs a [`Dt`].
    #[inline]
    pub const fn to_dt(&self) -> Dt {
        Dt::from_ymd(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Internal helper that round-trips through [`Dt`] to obtain a normalized
    /// `YmdHms` (handles clamping, leap seconds, etc.).
    #[inline(always)]
    const fn reconstruct(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        scale: Scale,
    ) -> Self {
        Dt::from_ymd(yr, mo, day, hr, min, sec, attos, scale).to_ymd()
    }

    /// Adds (or subtracts) whole years, preserving month and day-of-month.
    /// - Uses standard last-day-of-month clamping.
    /// - Negative values subtract.
    pub const fn add_yr(&self, n: i64) -> Self {
        if n == 0 {
            return *self;
        }
        let new_yr = self.yr.saturating_add(n);
        let max_day = Dt::days_in_month(new_yr, self.mo);
        let new_day = Dt::clamp_u8(self.day, 1, max_day);
        Self::reconstruct(
            new_yr, self.mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Adds (or subtracts) calendar months. Negative values subtract.
    pub const fn add_mo(&self, n: i64) -> Self {
        if n == 0 {
            return *self;
        }

        let yr = self.yr as i128;
        let mo = self.mo as i128;
        let delta = n as i128;

        let total_months = yr * 12 + (mo - 1) + delta;

        let new_yr = Dt::i128_to_i64(total_months.div_euclid(12));
        let new_mo = Dt::clamp_u8((total_months.rem_euclid(12) + 1) as u8, 1, 12);

        let max_day = Dt::days_in_month(new_yr, new_mo);
        let new_day = Dt::clamp_u8(self.day, 1, max_day);

        Self::reconstruct(
            new_yr, new_mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Adds (or subtracts) calendar weeks. Negative values subtract.
    #[inline]
    pub const fn add_wk(&self, n: i64) -> Self {
        self.add_days(n.saturating_mul(7))
    }

    /// Adds (or subtracts) calendar days. Negative values subtract.
    pub const fn add_days(&self, n: i64) -> Self {
        if n == 0 {
            return *self;
        }
        let jd = Dt::ymd_to_jd(self.yr, self.mo, self.day);
        let new_jd = jd.saturating_add(n);
        let (new_yr, new_mo, new_day) = Dt::jd_to_ymd(new_jd);
        Self::reconstruct(
            new_yr, new_mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Internal implementation detail for all sub-day / physical-time additions.
    /// Creates a temporary [`Dt`], performs the addition, then converts back to `YmdHms`.
    #[inline(never)]
    const fn _add_attos(&self, attos_delta: i128) -> Self {
        let tai = Dt::from_ymd(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        );
        let new_tai = tai.add(Dt::span(attos_delta));
        new_tai.to_ymd()
    }

    /// Adds (or subtracts) attoseconds. Negative values subtract.
    #[inline]
    pub const fn add_attos(&self, n: i128) -> Self {
        self._add_attos(n)
    }

    /// Adds (or subtracts) whole seconds. Negative values subtract.
    #[inline]
    pub const fn add_sec(&self, n: i64) -> Self {
        self._add_attos((n as i128).saturating_mul(ATTOS_PER_SEC_I128))
    }

    /// Adds (or subtracts) whole minutes. Negative values subtract.
    #[inline]
    pub const fn add_min(&self, n: i64) -> Self {
        let delta = (n as i128)
            .saturating_mul(60)
            .saturating_mul(ATTOS_PER_SEC_I128);
        self._add_attos(delta)
    }

    /// Adds (or subtracts) whole hours. Negative values subtract.
    #[inline]
    pub const fn add_hr(&self, n: i64) -> Self {
        let delta = (n as i128)
            .saturating_mul(3600)
            .saturating_mul(ATTOS_PER_SEC_I128);
        self._add_attos(delta)
    }

    /// Returns the year component.
    #[inline]
    pub const fn yr(&self) -> i64 {
        self.yr
    }

    /// Returns the month component (1–12).
    #[inline]
    pub const fn mo(&self) -> u8 {
        self.mo
    }

    /// Returns the day-of-month component (1–31, depending on month/year).
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Returns the hour component (0–23).
    #[inline]
    pub const fn hr(&self) -> u8 {
        self.hr
    }

    /// Returns the minute component (0–59).
    #[inline]
    pub const fn min(&self) -> u8 {
        self.min
    }

    /// Returns the second component (0–60). The value 60 only occurs during
    /// a positive leap second on `Scale::UTC` / `UtcSpice` / `UtcHist`.
    #[inline]
    pub const fn sec(&self) -> u8 {
        self.sec
    }

    /// Returns the attosecond (sub-second) component (0 ≤ attos < 10¹⁸).
    #[inline]
    pub const fn attos(&self) -> u64 {
        self.attos
    }

    /// The time scale that the object was created on.
    #[inline]
    pub const fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns the **ISO week year** (can differ from the calendar year near
    /// January 1 / December 31).
    #[inline]
    pub const fn iso_yr(&self) -> i64 {
        let (iso_yr, _, _) = Dt::_to_iso_wk_date(self.yr, self.mo, self.day);
        iso_yr
    }

    /// Returns the **ISO week number** (1–53). Weeks start on Monday; week 1
    /// is the week containing the first Thursday of the year.
    #[inline]
    pub const fn iso_wk(&self) -> u8 {
        let (_, iso_wk, _) = Dt::_to_iso_wk_date(self.yr, self.mo, self.day);
        iso_wk
    }

    /// Returns the **day of the year** (ordinal date), 1-based (Jan 1 = 1,
    /// Dec 31 = 365 or 366 in leap years).
    #[inline]
    pub const fn day_of_yr(&self) -> u16 {
        Dt::_day_of_yr(self.yr, self.mo, self.day)
    }

    /// Returns the **weekday** number according to [`Dt::jd_to_wkday`]
    /// (typically 0 = Sunday … 6 = Saturday; exact convention is defined
    /// by the Julian Day helper).
    #[inline]
    pub const fn wkday(&self) -> u8 {
        let jd = Dt::ymd_to_jd(self.yr, self.mo, self.day);
        Dt::jd_to_wkday(jd)
    }

    /// Returns the **week of year** number when weeks are considered to start
    /// on Sunday (US-style numbering).
    #[inline]
    pub const fn wk_of_yr_sun(&self) -> u8 {
        Dt::_wk_sun(self.yr, self.day_of_yr())
    }

    /// Returns the **week of year** number when weeks are considered to start
    /// on Monday.
    #[inline]
    pub const fn wk_of_yr_mon(&self) -> u8 {
        Dt::_wk_mon(self.yr, self.day_of_yr())
    }
}

#[cfg(feature = "jiff-tz")]
impl YmdHms {
    /// Adds the given number of years in the specified IANA timezone,
    /// respecting timezone rules (including DST) and proper calendar arithmetic.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_yr_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().years(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(zoned))
    }

    /// Adds the given number of months in the specified IANA timezone,
    /// respecting timezone rules and calendar month-end clamping.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_mo_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().months(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(zoned))
    }

    /// Adds the given number of weeks in the specified IANA timezone.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline]
    pub fn add_wk_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        self.add_days_tz(n.saturating_mul(7), tz)
    }

    /// Adds the given number of calendar days in the specified IANA timezone.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_days_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().days(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(zoned))
    }

    /// Adds the given number of hours in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_hr_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let new_zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().hours(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(new_zoned))
    }

    /// Adds the given number of minutes in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_min_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().minutes(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(zoned))
    }

    /// Adds the given number of seconds in the specified IANA timezone.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    pub fn add_sec_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        let zoned = self
            .to_jiff_zoned(tz)?
            .checked_add(jiff::Span::new().seconds(n))
            .map_err(|e| an_err!(DtErrKind::OutOfRange, "{}", e))?;
        Ok(self.from_jiff_zoned(zoned))
    }

    // helpers

    fn to_jiff_zoned(&self, tz: &str) -> Result<jiff::Zoned, DtErr> {
        use jiff::civil;

        if !(-9999..=9999).contains(&self.yr) {
            return Err(an_err!(
                DtErrKind::OutOfRange,
                "yr {} is outside Jiff's supported range (-9999..=9999)",
                self.yr
            ));
        }

        let hr: i8 = self
            .hr
            .try_into()
            .map_err(|_| an_err!(DtErrKind::InvalidNumber, "hr: {} u8 -> i8", self.hr))?;
        let min: i8 = self
            .min
            .try_into()
            .map_err(|_| an_err!(DtErrKind::InvalidNumber, "min: {} u8 -> i8", self.min))?;

        let sec_for_jiff: i8 = if self.sec == 60 {
            59
        } else {
            self.sec
                .try_into()
                .map_err(|_| an_err!(DtErrKind::InvalidNumber, "sec: {} u8 -> i8", self.sec))?
        };

        let mo: i8 = self
            .mo
            .try_into()
            .map_err(|_| an_err!(DtErrKind::InvalidNumber, "mo: {} u8 -> i8", self.mo))?;
        let day: i8 = self
            .day
            .try_into()
            .map_err(|_| an_err!(DtErrKind::InvalidNumber, "day: {} u8 -> i8", self.day))?;

        let civil_time = civil::date(self.yr as i16, mo, day).at(hr, min, sec_for_jiff, 0);

        civil_time
            .in_tz(tz)
            .map_err(|e| an_err!(DtErrKind::InvalidTimezoneOffset, "{}", e))
    }

    fn from_jiff_zoned(&self, zoned: jiff::Zoned) -> Self {
        let civil = zoned.datetime();

        Self::reconstruct(
            civil.year() as i64,
            civil.month() as u8,
            civil.day() as u8,
            civil.hour() as u8,
            civil.minute() as u8,
            civil.second() as u8,
            self.attos,
            self.scale,
        )
    }
}
