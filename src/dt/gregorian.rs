use crate::{
    ATTOS_PER_SEC, Scale, GregorianTime, SEC_PER_DAYI64, Dt, TSpan, Weekday,
    leap_seconds::get_leap_seconds,
};

/// Combined Gregorian date + wall time with subsecond precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct YmdHms {
    pub yr: i64,
    pub mo: u8,
    pub day: u8,
    pub hr: u8,
    pub min: u8,
    pub sec: u8,    // 0–60 (60 only during leap seconds)
    pub attos: u64, // attoseconds (0 ≤ subsec < 10¹⁸)
}

impl Dt {
    /// Converts a Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// to a proleptic Gregorian date (year, month, day).
    #[inline]
    pub const fn unix_sec_to_gregorian_ymd(unix_sec: i64) -> (i64, u8, u8) {
        let days_since_1970 = unix_sec.div_euclid(SEC_PER_DAYI64);
        // 1970-01-01 00:00:00 UTC is JD 2440588.0
        let jdn = days_since_1970 + 2440588;
        Self::jdn_to_ymd(jdn)
    }

    pub const fn to_gregorian_time(&self) -> GregorianTime {
        let scale = self.scale;

        // Use the new unified function (replaces the old to_gregorian_ymd + to_hms_subsec calls)
        let ymdhms = self.to_ymdhms();
        let unix_attosec = self
            .to_epoch(Dt::UNIX_EPOCH, Scale::UTC)
            .to_attos();

        let (iso_yr, iso_wk, iso_wkday) =
            self.to_iso_week_date(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let day_of_yr = self.day_of_year(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let jdn = Self::ymd_to_jdn(ymdhms.yr, ymdhms.mo, ymdhms.day);
        let wkday = Self::jdn_to_weekday(jdn);
        let wk_of_yr_sun = self.wk_sun(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)), Some(day_of_yr));
        let wk_of_yr_mon = self.wk_mon(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)), Some(day_of_yr));

        GregorianTime {
            unix_attosec,
            yr: ymdhms.yr,
            mo: ymdhms.mo,
            day: ymdhms.day,
            hr: ymdhms.hr,
            min: ymdhms.min,
            sec: ymdhms.sec,
            attos: ymdhms.attos,
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

    /// Stripped down version of `Dt::to_gregorian_time`.
    ///
    /// Returns the Gregorian date and wall time for this instant.
    ///
    /// - For `Scale::UTC`: Uses a direct Unix-timestamp-based path (fast and clean).
    /// - For all other scales: Uses the standard TT-based JD path.
    #[inline]
    pub const fn to_ymdhms(&self) -> YmdHms {
        // Single call gets us the full civil attos since Unix epoch (POSIX style).
        // This replaces both to_unix_sec() + the old to_attos_since(UNIX_EPOCH).
        let canon = self.to_epoch(Dt::UNIX_EPOCH, Scale::UTC);

        let unix_sec = canon.sec;
        let subsec = canon.subsec;

        let is_leap_second = get_leap_seconds(&self, false).is_leap_second;

        // For the date we always use the previous second when on a leap second
        // (so 23:59:60 stays on the correct civil day).
        let unix_sec_for_date = if is_leap_second {
            unix_sec - 1
        } else {
            unix_sec
        };

        let (yr, mo, day) = Self::unix_sec_to_gregorian_ymd(unix_sec_for_date);

        // Only the hour/minute/second fields differ for a leap second.
        let (hr, min, sec) = if is_leap_second {
            (23, 59, 60)
        } else {
            let seconds_since_midnight = unix_sec.rem_euclid(SEC_PER_DAYI64);
            let hr = (seconds_since_midnight / 3600) as u8;
            let min = ((seconds_since_midnight % 3600) / 60) as u8;
            let sec = (seconds_since_midnight % 60) as u8;
            (hr, min, sec)
        };

        YmdHms {
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos: subsec,
        }
    }

    /// Converts a proleptic Gregorian calendar date+time to a Unix timestamp
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// This version is correct for the full i64 range, including negative years.
    pub const fn ymdhms_to_unix_sec(
        year: i64,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> i64 {
        let jdn = Self::ymd_to_jdn(year, month, day);
        // 1970-01-01 00:00:00 UTC corresponds to JD 2440588
        let days_since_1970 = jdn - 2440588;
        let time_of_day = (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);
        days_since_1970 * SEC_PER_DAYI64 + time_of_day
    }

    /// Converts a Julian Day Number (JDN) to a proleptic Gregorian calendar date.
    ///
    /// Returns `(year, month, day)` where `month` ∈ [1, 12] and `day` ∈ [1, 31]
    /// (standard 1-based Gregorian values).
    ///
    /// This is the inverse of [`Self::ymd_to_jdn`]. Supports the full `i64`
    /// range, including negative years and year zero.
    pub const fn jdn_to_ymd(jdn: i64) -> (i64, u8, u8) {
        // Use i128 internally to avoid overflow on full i64 JDN range
        let j = jdn as i128;

        // Floor division helper (required for negative JDNs)
        const fn floor_div(a: i128, b: i128) -> i128 {
            let q = a / b;
            let r = a % b;
            if (r > 0 && b < 0) || (r < 0 && b > 0) {
                q - 1
            } else {
                q
            }
        }

        let a = j + 32044;
        let b = floor_div(4 * a + 3, 146097);
        let c = a - floor_div(b * 146097, 4);
        let d = floor_div(4 * c + 3, 1461);
        let e = c - floor_div(1461 * d, 4);
        let m = floor_div(5 * e + 2, 153);
        let day = (e - floor_div(153 * m + 2, 5) + 1) as u8;
        let month = (m + 3 - 12 * floor_div(m, 10)) as u8;
        let year = b * 100 + d - 4800 + floor_div(m, 10);
        (year as i64, month, day)
    }

    /// Computes the Julian Day Number (JDN) for a proleptic Gregorian calendar date at noon UT.
    ///
    /// The algorithm matches the standard astronomical convention used throughout the library
    /// (`ymd_to_jdn(2000, 1, 1) == 2451545`).
    pub const fn ymd_to_jdn(year: i64, month: u8, day: u8) -> i64 {
        let a = (14 - month as i64) / 12;
        let y = year + 4800 - a;
        let m = month as i64 + 12 * a - 3;

        // Floor division helper (must be const fn, not a closure)
        const fn floor_div(a: i64, b: i64) -> i64 {
            let q = a / b;
            let r = a % b;
            if (r > 0 && b < 0) || (r < 0 && b > 0) {
                q - 1
            } else {
                q
            }
        }

        let y4 = floor_div(y, 4);
        let y100 = floor_div(y, 100);
        let y400 = floor_div(y, 400);

        day as i64 + (153 * m + 2) / 5 + 365 * y + y4 - y100 + y400 - 32045
    }

    /// Returns `true` if the given year is a Gregorian leap year under proleptic rules.
    #[inline]
    pub const fn is_leap_year(year: i64) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Creates a `Dt` at the specified civil UTC instant with full
    /// attosecond precision on the proleptic Gregorian calendar, then converts
    /// it to the requested [`Scale`].
    ///
    /// All input components are clamped to their valid ranges:
    /// - `mo`   → 0..=12
    /// - `day`  → 0..=31
    /// - `hr`   → 0..=23
    /// - `min`  → 0..=59
    /// - `sec`  → 0..=60 (permits leap seconds)
    /// - `attos` → values ≥ 10¹⁸ are carried into the seconds field
    pub const fn from_ymdhms(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        scale: Scale,
    ) -> Self {
        let mo = if mo > 12 { 12 } else { mo };
        let day = if day > 31 { 31 } else { day };
        let h = if hr > 23 { 23 } else { hr };
        let m = if min > 59 { 59 } else { min };
        let s = if sec > 60 { 60 } else { sec };

        let extra_sec = (attos / ATTOS_PER_SEC) as i64;
        let final_attos = attos % ATTOS_PER_SEC;

        // For an exact leap second (sec==60 with no sub-second carry), compute
        // the civil Unix timestamp using 23:59:59, create that instant, then
        // add exactly 1 physical second. This lands on the correct internal TAI
        // slot (matching LEAP_SECS.tai_sec) while preserving the library's
        // convention that to_epoch_attos(UTC) for the leap second returns the
        // "following midnight" civil value. On non-leap days or with carry,
        // the normal rollover path is used and to_ymdhms_utc will display
        // correctly because is_leap_second only triggers on exact tai_sec match.
        let is_exact_leap_second = s == 60 && extra_sec == 0;
        let s_for_unix = if is_exact_leap_second { 59 } else { s };

        let civil_unix_sec = Self::ymdhms_to_unix_sec(yr, mo, day, h, m, s_for_unix) + extra_sec;

        let mut tp = Self::from_epoch(
            TSpan::new(civil_unix_sec, final_attos),
            Dt::UNIX_EPOCH,
            scale.to_ut(),
        );
        if is_exact_leap_second {
            tp = tp.add(TSpan::from_sec(1));
        }
        tp.with_type(scale)
    }

    /// Creates a `Dt` representing **00:00:00 UTC** on the given proleptic
    /// Gregorian date, converted to the requested [`Scale`].
    ///
    /// The date components are interpreted according to POSIX civil time
    /// (leap seconds are not inserted into the day count).
    pub const fn from_ymd(yr: i64, mo: u8, day: u8, scale: Scale) -> Self {
        let unix_sec = Self::ymdhms_to_unix_sec(yr, mo, day, 0, 0, 0);

        Self::from_epoch(
            TSpan::new(unix_sec, 0),
            Dt::UNIX_EPOCH,
            scale.to_ut(),
        )
        .with_type(scale)
    }

    /// Computes the Julian Day Number from a Gregorian year and ordinal day-of-year.
    #[inline]
    pub const fn ydoy_to_jdn(year: i64, day_of_year: u16) -> i64 {
        let jdn_jan1 = Self::ymd_to_jdn(year, 1, 1);
        jdn_jan1 + (day_of_year as i64 - 1)
    }

    /// Converts a Julian Day Number to the corresponding weekday number (0 = Sunday … 6 = Saturday).
    #[inline]
    pub const fn jdn_to_weekday(jdn: i64) -> u8 {
        ((jdn + 1) % 7) as u8
    }

    /// Computes the Julian Day Number from an ISO week date (Monday-based week).
    pub const fn ymd_to_jdn_from_iso_week(iso_year: i64, iso_week: u8, weekday: Weekday) -> i64 {
        let jan4_jdn = Self::ymd_to_jdn(iso_year, 1, 4);
        let wd_jan4 = Self::jdn_to_weekday(jan4_jdn);
        let days_to_monday = (wd_jan4 + 6) % 7;
        let monday_week1 = jan4_jdn - (days_to_monday as i64);
        let monday_requested = monday_week1 + (iso_week as i64 - 1) * 7;

        let wd_offset = match weekday {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        };

        monday_requested + (wd_offset as i64)
    }

    /// Computes the Julian Day Number from a Sunday-based week-of-year (`%U`).
    pub const fn ymd_to_jdn_from_week_sun(year: i64, week: u8, weekday: Weekday) -> i64 {
        let jan1_jdn = Self::ymd_to_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jan1_jdn);

        let days_to_first_sunday = ((7u8 - wd_jan1) % 7u8) as i64;
        let first_sunday_jdn = jan1_jdn + days_to_first_sunday;

        let sunday_of_week = first_sunday_jdn + (week as i64 - 1) * 7;

        let wd_offset = match weekday {
            Weekday::Sunday => 0,
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
        };
        sunday_of_week + (wd_offset as i64)
    }

    /// Computes the Julian Day Number from a Monday-based week-of-year (`%W`).
    pub const fn ymd_to_jdn_from_week_mon(year: i64, week: u8, weekday: Weekday) -> i64 {
        let jan1_jdn = Self::ymd_to_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jan1_jdn);

        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_jdn = jan1_jdn + days_to_first_monday;

        let monday_of_week = first_monday_jdn + (week as i64 - 1) * 7;

        let wd_offset = match weekday {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        };
        monday_of_week + (wd_offset as i64)
    }

    /// Returns `true` if the supplied values form a valid proleptic Gregorian calendar date.
    pub const fn is_valid_ymd(year: i64, month: u8, day: u8) -> bool {
        if month < 1 || month > 12 || day < 1 {
            return false;
        }
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31u8,
            4 | 6 | 9 | 11 => 30u8,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };
        day <= days
    }

    /// Returns `true` if the given Gregorian year contains an ISO week 53.
    pub const fn has_iso_week_53(year: i64) -> bool {
        let jan1_jdn = Self::ymd_to_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jan1_jdn);
        wd_jan1 == 4 || (Self::is_leap_year(year) && wd_jan1 == 3)
    }

    /// Returns the ordinal day of the year (1-based).
    ///
    /// January 1 is day `1`; December 31 is day `365` or `366` (in leap years).
    /// Uses the proleptic Gregorian calendar.
    pub const fn day_of_year(&self, ymd: Option<(i64, u8, u8)>) -> u16 {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms();
            (g.yr, g.mo, g.day)
        };
        let jdn = Self::ymd_to_jdn(year, month, day);
        let jdn_jan1 = Self::ymd_to_jdn(year, 1, 1);
        (jdn - jdn_jan1 + 1) as u16
    }

    /// Sunday-based week number (`%U` in strftime).
    ///
    /// Range: `0..=53`.
    /// - Week 0 contains the days *before* the first Sunday of the year.
    /// - Week 1 begins on the first Sunday of the year.
    ///
    /// The optional `ymd` and `doy` arguments are performance optimisations
    /// (same pattern used throughout the file for `day_of_year`, `to_iso_week_date`, etc.).
    /// Pass whichever you already have; the function will use the fastest path.
    pub const fn wk_sun(&self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (year, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms();
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_year(ymd)
        };
        let jan1_jdn = Self::ymd_to_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jan1_jdn);
        let days_to_first_sunday = (7u8 - wd_jan1) % 7u8;
        let first_sunday_doy = days_to_first_sunday as u16 + 1;
        if doy < first_sunday_doy {
            0
        } else {
            let days_since_first_sunday = doy - first_sunday_doy;
            ((days_since_first_sunday / 7) + 1) as u8
        }
    }

    /// Monday-based week number (`%W` in strftime).
    ///
    /// Range: `0..=53`.
    /// - Week 0 contains the days *before* the first Monday of the year.
    /// - Week 1 begins on the first Monday of the year.
    ///
    /// The optional `ymd` and `doy` arguments are performance optimisations
    /// (same pattern as `wk_sun`, `day_of_year`, `to_iso_week_date`, etc.).
    pub const fn wk_mon(&self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (year, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms();
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_year(ymd)
        };
        let jan1_jdn = Self::ymd_to_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jan1_jdn);
        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_doy = days_to_first_monday as u16 + 1;
        if doy < first_monday_doy {
            0
        } else {
            let days_since_first_monday = doy - first_monday_doy;
            ((days_since_first_monday / 7) + 1) as u8
        }
    }

    /// Returns the ISO 8601 week date for this `Dt`.
    ///
    /// Returns `(iso_year, iso_week, weekday)` where:
    /// - `iso_year` is the ISO week year (may differ from the Gregorian year near
    ///   year boundaries),
    /// - `iso_week` is the week number in the range `1..=53`,
    /// - `weekday` is a [`Weekday`] value (Monday-based week).
    ///
    /// Follows the ISO 8601 standard: weeks start on Monday and week 1 is the
    /// week containing January 4.
    ///
    /// The optional `ymd` argument is a performance optimization. If provided,
    /// it is used directly; otherwise [`to_gregorian_ymd`](Self::to_gregorian_ymd)
    /// is called internally.
    pub const fn to_iso_week_date(&self, ymd: Option<(i64, u8, u8)>) -> (i64, u8, Weekday) {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms();
            (g.yr, g.mo, g.day)
        };
        let jdn = Self::ymd_to_jdn(year, month, day);
        let wd = Self::jdn_to_weekday(jdn);
        let wd_iso = if wd == 0 { 7 } else { wd };

        let jan4_jdn = Self::ymd_to_jdn(year, 1, 4);
        let wd_jan4 = Self::jdn_to_weekday(jan4_jdn);
        let days_to_monday = (wd_jan4 + 6) % 7;
        let monday_week1 = jan4_jdn - (days_to_monday as i64);

        let days_since = jdn - monday_week1;

        let week = if days_since < 0 {
            0u8
        } else {
            ((days_since / 7) + 1) as u8
        };

        let iso_year = if week == 0 {
            year - 1
        } else if (week == 53 || week > 53) && !Self::has_iso_week_53(year) {
            year + 1
        } else {
            year
        };

        let iso_week = if week == 0 {
            if Self::has_iso_week_53(year - 1) {
                53
            } else {
                52
            }
        } else if week == 53 && !Self::has_iso_week_53(year) {
            1
        } else if week > 53 {
            1
        } else {
            week
        };

        let weekday_enum = match wd_iso {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => Weekday::Sunday,
        };

        (iso_year, iso_week, weekday_enum)
    }
}
