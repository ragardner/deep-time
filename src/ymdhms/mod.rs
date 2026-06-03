use crate::{ATTOS_PER_SEC_I128, Dt, Scale, Weekday};

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
/// let x = Dt::from_ymd(2000, 2, 30, 0, 0, 0, 0, Scale::UTC).to_ymd();
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
    pub(crate) unix_attosec: i128,
    pub(crate) scale: Scale,
}

impl YmdHms {
    /// Reconstructs a [`Dt`].
    #[inline]
    pub fn to_dt(&self) -> Dt {
        Dt::from_ymd(
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
        Dt::from_ymd(yr, mo, day, hr, min, sec, attos, scale).to_ymd()
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
        let tai = Dt::from_ymd(
            self.yr, self.mo, self.day, self.hr, self.min, self.sec, self.attos, self.scale,
        );
        let new_tai = tai.add(Dt::span(attos_delta));
        new_tai.to_ymd()
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

    #[inline]
    pub fn iso_yr(&self) -> i64 {
        let (iso_yr, _, _) = Dt::_to_iso_wk_date(self.yr, self.mo, self.day);
        iso_yr
    }

    #[inline]
    pub fn iso_wk(&self) -> u8 {
        let (_, iso_wk, _) = Dt::_to_iso_wk_date(self.yr, self.mo, self.day);
        iso_wk
    }

    #[inline]
    pub fn day_of_yr(&self) -> u16 {
        Dt::_day_of_yr(self.yr, self.mo, self.day)
    }

    #[inline]
    pub fn wkday(&self) -> Weekday {
        let jd = Dt::ymd_to_jd(self.yr, self.mo, self.day);
        Weekday::from_sunday_0_based(Dt::jd_to_wkday(jd)).unwrap_or_default()
    }

    #[inline]
    pub fn wk_of_yr_sun(&self) -> u8 {
        Dt::_wk_sun(self.yr, self.day_of_yr())
    }

    #[inline]
    pub fn wk_of_yr_mon(&self) -> u8 {
        Dt::_wk_mon(self.yr, self.day_of_yr())
    }

    /// Returns the Unix timestamp since 1970-01-01 00:00:00 as a tuple of
    /// `(whole_seconds, attoseconds)`.
    ///
    /// - The timestamp will be on whatever [`Scale`] the [`DateTime`] was created on.
    /// - `whole_seconds` can be negative (for dates before 1970).
    /// - The fractional part (`attoseconds`) is always in the range `0..=999_999_999_999_999_999`.
    pub const fn unix_timestamp(&self) -> (i64, u64) {
        const ATTOS_PER_SEC_I128: i128 = 1_000_000_000_000_000_000;
        let total = self.unix_attosec;
        let secs = (total / ATTOS_PER_SEC_I128) as i64;
        let frac = (total % ATTOS_PER_SEC_I128).unsigned_abs() as u64;
        (secs, frac)
    }
}
