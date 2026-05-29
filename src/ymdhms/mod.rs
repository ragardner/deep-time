use crate::{ATTOS_PER_SEC_I128, Dt, LiteStr, Scale, Weekday};

mod to_str;

/// Combined Gregorian date + wall time with subsecond precision.
///
/// Has some basic calendar aware math, but not time zone aware.
///
/// ## Examples
///
/// **Creating a** [`YmdHms`].
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// // clamped to 29
/// let x = Dt::from_ymd(2000, 2, 30).to_ymdhms(Scale::TAI);
///
/// assert_eq!(x.day(), 29);
/// ```
///
/// **Adding a year.** 2000 is a leap year and Feb. 29th is possible, but
/// 2001 isn't a leap year so the day is clamped to the 28th.
///
/// ```
/// use deep_time::{Dt, Scale};
///
/// let x = Dt::from_ymd(2000, 2, 29).to_ymdhms(Scale::TAI);
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
    pub(crate) unix_attosec: i128,
    pub(crate) scale: Scale,
}

impl YmdHms {
    /// Reconstructs a [`Dt`].
    #[inline]
    pub fn to_dt(&self) -> Dt {
        Dt::from_ymdhms_on(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    #[inline(always)]
    fn reconstruct(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        scale: Scale,
    ) -> Self {
        Dt::from_ymdhms_on(yr, mo, day, hr, min, sec, attos, scale).to_ymdhms_on(Scale::TAI, scale)
    }

    /// Adds (or subtracts) whole years, preserving month and day-of-month.
    /// Negative values subtract years. Uses standard last-day-of-month clamping.
    pub fn add_yr(&self, years: i64) -> Self {
        if years == 0 {
            return *self;
        }
        let new_yr = self.yr.saturating_add(years);
        let max_day = Dt::days_in_month(new_yr, self.mo);
        let new_day = Dt::clamp_u8(self.day, 1, max_day);
        Self::reconstruct(
            new_yr, self.mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Adds (or subtracts) whole months. Negative values subtract months.
    /// Uses `i128` total-month arithmetic to avoid overflow at extreme years.
    pub fn add_mo(&self, months: i64) -> Self {
        if months == 0 {
            return *self;
        }
        let yr = self.yr as i128;
        let mo = self.mo as i128;
        let delta = months as i128;

        let total_months = yr * 12 + (mo - 1) + delta;

        let new_yr = Dt::i128_to_i64(total_months.div_euclid(12));
        let new_mo = Dt::clamp_u8((total_months.rem_euclid(12) + 1) as u8, 1, 12);

        let max_day = Dt::days_in_month(new_yr, new_mo);
        let new_day = Dt::clamp_u8(self.day, 1, max_day);

        Self::reconstruct(
            new_yr, new_mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Adds (or subtracts) calendar days using Julian Day arithmetic.
    /// Negative values subtract days.
    pub fn add_days(&self, days: i64) -> Self {
        if days == 0 {
            return *self;
        }
        let jd = Dt::ymd_to_jd(self.yr, self.mo, self.day);
        let new_jd = jd.saturating_add(days);
        let (new_yr, new_mo, new_day) = Dt::jd_to_ymd(new_jd);
        Self::reconstruct(
            new_yr, new_mo, new_day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    #[inline]
    pub fn add_wk(&self, weeks: i64) -> Self {
        self.add_days(weeks.saturating_mul(7))
    }

    #[inline(never)]
    fn _add_attos(&self, attos_delta: i128) -> Self {
        let tai = Dt::from_ymdhms_on(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        );
        let delta_dt = Dt::from(attos_delta, Scale::TAI);
        let new_tai = tai.add(delta_dt);
        new_tai.to_ymdhms_on(Scale::TAI, self.scale)
    }

    #[inline]
    pub fn add_attos(&self, attos: i128) -> Self {
        self._add_attos(attos)
    }

    #[inline]
    pub fn add_sec(&self, sec: i64) -> Self {
        self._add_attos(sec as i128 * ATTOS_PER_SEC_I128)
    }

    #[inline]
    pub fn add_min(&self, min: i64) -> Self {
        self._add_attos(min as i128 * 60 * ATTOS_PER_SEC_I128)
    }

    #[inline]
    pub fn add_hr(&self, hr: i64) -> Self {
        self._add_attos(hr as i128 * 3600 * ATTOS_PER_SEC_I128)
    }

    #[inline]
    pub fn yr(&self) -> i64 {
        self.yr
    }

    #[inline]
    pub fn mo(&self) -> u8 {
        self.mo
    }

    #[inline]
    pub fn day(&self) -> u8 {
        self.day
    }

    #[inline]
    pub fn hr(&self) -> u8 {
        self.hr
    }

    #[inline]
    pub fn min(&self) -> u8 {
        self.min
    }

    #[inline]
    pub fn sec(&self) -> u8 {
        self.sec
    }

    #[inline]
    pub fn attos(&self) -> u64 {
        self.attos
    }

    /// Attoseconds since 1970-01-01 midnight, on whatever time scale
    /// the object was created on.
    #[inline]
    pub fn unix_attosec(&self) -> i128 {
        self.unix_attosec
    }

    /// The time scale that the object was created on.
    #[inline]
    pub fn scale(&self) -> Scale {
        self.scale
    }

    pub(crate) fn to_ymdhms_rich(
        &self,
        iso_yr: i64,
        iso_wk: u8,
        iso_wkday: Weekday,
        day_of_yr: u16,
        wkday: u8,
        wk_of_yr_sun: u8,
        wk_of_yr_mon: u8,
    ) -> YmdHmsRich {
        YmdHmsRich::new(
            self.unix_attosec,
            self.yr,
            self.mo,
            self.day,
            self.hr,
            self.min,
            self.sec,
            self.attos,
            iso_yr,
            iso_wk,
            iso_wkday,
            day_of_yr,
            wkday,
            wk_of_yr_sun,
            wk_of_yr_mon,
            self.scale,
        )
    }
}

/// Gregorian calendar and time-of-day components of a [`Dt`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YmdHmsRich {
    /// UNIX attoseconds counting from 1970 epoch
    pub(crate) unix_attosec: i128,
    /// Gregorian year (proleptic Gregorian calendar, supports negative years and year 0).
    pub(crate) yr: i64,
    /// Gregorian month in the range [1, 12].
    pub(crate) mo: u8,
    /// Gregorian day of the month in the range [1, 31].
    pub(crate) day: u8,
    /// Hour of the day in the range [0, 23].
    pub(crate) hr: u8,
    /// Minute in the range [0, 59].
    pub(crate) min: u8,
    /// Second in the range [0, 60] (60 only during UTC leap seconds).
    pub(crate) sec: u8,
    /// Fractional part of the second expressed in attoseconds (u64).
    pub(crate) attos: u64,
    /// ISO 8601 week year.
    pub(crate) iso_yr: i64,
    /// ISO 8601 week number in the range [1, 53].
    pub(crate) iso_wk: u8,
    /// ISO 8601 weekday enum e.g. Monday/Tuesday/...
    pub(crate) iso_wkday: Weekday,
    /// Ordinal day of the year (1-based).
    pub(crate) day_of_yr: u16,
    /// Weekday number (0 = Sunday … 6 = Saturday).
    pub(crate) wkday: u8,
    /// Sunday based week of year (Range: `0..=53`).
    pub(crate) wk_of_yr_sun: u8,
    /// Monday based week of year (Range: `0..=53`).
    pub(crate) wk_of_yr_mon: u8,
    /// Used for formatting (strftime).
    /// A stored offset in seconds, used within the crate.
    pub(crate) offset_sec: Option<i32>,
    /// A stored IANA name, used within the crate, %Q.
    pub(crate) tz: Option<LiteStr<49>>,
    /// UTC, EST, %Z
    pub(crate) tz_abbrev: Option<LiteStr<49>>,
    /// Scale the instance was created on
    pub(crate) scale: Scale,
}

impl YmdHmsRich {
    /// Creates a new [`YmdHmsRich`] with all fields specified.
    #[inline]
    pub(crate) const fn new(
        unix_attosec: i128,
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        iso_yr: i64,
        iso_wk: u8,
        iso_wkday: Weekday,
        day_of_yr: u16,
        wkday: u8,
        wk_of_yr_sun: u8,
        wk_of_yr_mon: u8,
        scale: Scale,
    ) -> Self {
        Self {
            unix_attosec,
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos,
            iso_yr,
            iso_wk,
            iso_wkday,
            day_of_yr,
            wkday,
            wk_of_yr_sun,
            wk_of_yr_mon,
            offset_sec: None,
            tz: None,
            tz_abbrev: None,
            scale,
        }
    }

    /// Reconstructs a [`Dt`].
    #[inline]
    pub fn to_dt(&self) -> Dt {
        Dt::from_ymdhms_on(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        )
    }

    /// Attoseconds since 1970-01-01 midnight, on whatever time scale
    /// the object was created on.
    #[inline]
    pub const fn unix_attosec(&self) -> i128 {
        self.unix_attosec
    }

    /// The time scale that the object was created on.
    #[inline]
    pub const fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns the Unix timestamp since 1970-01-01 00:00:00 as a tuple of
    /// `(whole_seconds, attoseconds)`.
    ///
    /// - The timestamp will be on whatever [`Scale`] the [`DateTime`] was created on.
    /// - `whole_seconds` can be negative (for dates before 1970).
    /// - The fractional part (`attoseconds`) is always in the range `0..=999_999_999_999_999_999`.
    #[inline]
    pub const fn unix_timestamp(&self) -> (i64, u64) {
        const ATTOS_PER_SEC_I128: i128 = 1_000_000_000_000_000_000;
        let total = self.unix_attosec;
        let secs = (total / ATTOS_PER_SEC_I128) as i64;
        let frac = (total % ATTOS_PER_SEC_I128).unsigned_abs() as u64;
        (secs, frac)
    }

    /// Gregorian year (proleptic Gregorian calendar, supports negative years and year 0).
    #[inline]
    pub const fn yr(&self) -> i64 {
        self.yr
    }

    /// Gregorian month in the range [1, 12].
    #[inline]
    pub const fn mo(&self) -> u8 {
        self.mo
    }

    /// Gregorian day of the month in the range [1, 31].
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Hour of the day in the range [0, 23].
    #[inline]
    pub const fn hr(&self) -> u8 {
        self.hr
    }

    /// Minute in the range [0, 59].
    #[inline]
    pub const fn min(&self) -> u8 {
        self.min
    }

    /// Second in the range [0, 60] (60 only during UTC leap seconds).
    #[inline]
    pub const fn sec(&self) -> u8 {
        self.sec
    }

    /// Fractional part of the second expressed in attoseconds (`0 ≤ attos < 10¹⁸`).
    #[inline]
    pub const fn attos(&self) -> u64 {
        self.attos
    }

    /// ISO 8601 week year.
    #[inline]
    pub const fn iso_yr(&self) -> i64 {
        self.iso_yr
    }

    /// ISO 8601 week number in the range [1, 53].
    #[inline]
    pub const fn iso_wk(&self) -> u8 {
        self.iso_wk
    }

    /// ISO 8601 weekday (Monday-based [`Weekday`] enum).
    #[inline]
    pub const fn iso_wkday(&self) -> Weekday {
        self.iso_wkday
    }

    /// Ordinal day of the year (1-based).
    #[inline]
    pub const fn day_of_yr(&self) -> u16 {
        self.day_of_yr
    }

    /// Weekday number (0 = Sunday … 6 = Saturday).
    #[inline]
    pub const fn wkday_sun(&self) -> u8 {
        self.wkday
    }

    /// ISO 8601 weekday (0 = Monday ... 6 = Sunday).
    #[inline]
    pub const fn wkday_mon(&self) -> u8 {
        self.iso_wkday.wk_mon()
    }

    /// Sunday based week of year (Range: `0..=53`).
    #[inline]
    pub const fn wk_of_yr_sun(&self) -> u8 {
        self.wk_of_yr_sun
    }

    /// Monday based week of year (Range: `0..=53`).
    #[inline]
    pub const fn wk_of_yr_mon(&self) -> u8 {
        self.wk_of_yr_mon
    }

    #[inline]
    pub(crate) const fn offset_sec(&self) -> Option<i32> {
        self.offset_sec
    }

    #[inline]
    pub(crate) const fn tz(&self) -> Option<&LiteStr<49>> {
        self.tz.as_ref()
    }

    #[inline]
    pub(crate) const fn tz_abbrev(&self) -> Option<&LiteStr<49>> {
        self.tz_abbrev.as_ref()
    }

    #[inline]
    pub(crate) fn set_offset(&mut self, offset_sec: Option<i32>) -> &mut Self {
        self.offset_sec = offset_sec;
        self
    }

    #[inline]
    pub(crate) fn set_tz(&mut self, tz: Option<&str>) -> &mut Self {
        self.tz = tz.map(LiteStr::new);
        self
    }

    #[inline]
    pub(crate) fn set_tz_abbrev(&mut self, tz_abbrev: Option<&str>) -> &mut Self {
        self.tz_abbrev = tz_abbrev.map(LiteStr::new);
        self
    }
}
