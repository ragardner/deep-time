use crate::Dt;

#[cfg(feature = "jiff-tz")]
use crate::DtErr;

impl Dt {
    /// Adds (or subtracts) calendar years, preserving month and day-of-month.
    /// - Uses standard last-day-of-month clamping.
    /// - Negative values subtract.
    #[inline(always)]
    pub const fn add_yr(&self, n: i64) -> Self {
        self.to_ymd().add_yr(n).to_dt()
    }

    /// Adds (or subtracts) calendar months.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_mo(&self, n: i64) -> Self {
        self.to_ymd().add_mo(n).to_dt()
    }

    /// Adds (or subtracts) calendar weeks.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_wk(&self, n: i64) -> Self {
        self.to_ymd().add_wk(n).to_dt()
    }

    /// Adds (or subtracts) calendar days.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_days(&self, n: i64) -> Self {
        self.to_ymd().add_days(n).to_dt()
    }
}

#[cfg(feature = "jiff-tz")]
impl Dt {
    /// Adds the given number of years in the specified IANA timezone,
    /// respecting timezone rules (including DST) and calendar arithmetic.
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_yr_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_yr_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of months in the specified IANA timezone,
    /// respecting timezone rules and calendar month-end clamping.
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_mo_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_mo_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of weeks in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_wk_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_wk_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of calendar days in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_days_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_days_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of hours in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_hr_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_hr_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of minutes in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_min_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_min_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of seconds in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Important
    ///
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    /// - Will error if the year is outside of `-9999..=9999`.
    ///
    /// ## Errors
    ///
    /// - Jiff only supports years in the range `-9999..=9999`. Years outside
    ///   this range will return a [`DtErr`].
    /// - If Jiff cannot find the timezone name or if applying the timezone would cause
    ///   the [`jiff::Zoned`] to be outside the `-9999..=9999` year range then a
    ///   [`DtErr`] with [`DtErrKind::InvalidTimezoneOffset`] is returned.
    #[inline(always)]
    pub fn add_sec_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_sec_tz(n, tz)?.to_dt())
    }
}
