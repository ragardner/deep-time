use crate::Dt;

#[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
use crate::DtErr;

impl Dt {
    /// Adds (or subtracts) calendar years, preserving month and day-of-month.
    /// - Uses standard last-day-of-month clamping.
    /// - Negative values subtract.
    #[inline(always)]
    pub const fn add_years(&self, n: i64) -> Self {
        self.to_ymd().add_years(n).to_dt()
    }

    /// Adds (or subtracts) calendar months.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_months(&self, n: i64) -> Self {
        self.to_ymd().add_months(n).to_dt()
    }

    /// Adds (or subtracts) calendar weeks.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_weeks(&self, n: i64) -> Self {
        self.to_ymd().add_weeks(n).to_dt()
    }

    /// Adds (or subtracts) calendar days.
    /// Negative values subtract.
    #[inline(always)]
    pub const fn add_days(&self, n: i64) -> Self {
        self.to_ymd().add_days(n).to_dt()
    }
}

#[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
impl Dt {
    /// Adds the given number of years in the specified IANA timezone,
    /// respecting timezone rules (including DST) and calendar arithmetic.
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_years_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_years_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of months in the specified IANA timezone,
    /// respecting timezone rules and calendar month-end clamping.
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_months_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_months_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of weeks in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_weeks_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_weeks_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of calendar days in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_days_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_days_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of hours in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_hours_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_hours_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of minutes in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_mins_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_mins_tz(n, tz)?.to_dt())
    }

    /// Adds the given number of seconds in the specified IANA timezone,
    /// respecting timezone rules (including DST).
    ///
    /// ## Notes
    ///
    /// - Requires the `jiff-tz` feature.
    /// - Assumes this [`Dt`] is counting seconds from the library's
    ///   `2000-01-01 12:00:00` epoch.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange) if the year of the date is outside the
    ///   `-9999..=9999` range (checked before involving Jiff).
    /// - Specific errors for invalid time components when preparing values for Jiff:
    ///   [`DtErrKind::InvalidHour`](../error/enum.DtErrKind.html#variant.InvalidHour), [`DtErrKind::InvalidMinute`](../error/enum.DtErrKind.html#variant.InvalidMinute),
    ///   [`DtErrKind::InvalidSecond`](../error/enum.DtErrKind.html#variant.InvalidSecond), [`DtErrKind::InvalidMonth`](../error/enum.DtErrKind.html#variant.InvalidMonth), or [`DtErrKind::InvalidDay`](../error/enum.DtErrKind.html#variant.InvalidDay).
    /// - [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) if Jiff cannot find/resolve the IANA timezone name.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if the result of the calendar arithmetic operation
    ///   would be outside the range supported by Jiff (the checked_add fails).
    #[inline(always)]
    pub fn add_sec_tz(&self, n: i64, tz: &str) -> Result<Self, DtErr> {
        Ok(self.to_ymd().add_sec_tz(n, tz)?.to_dt())
    }
}
