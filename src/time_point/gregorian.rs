use crate::{
    ATTOSEC_PER_SEC, ATTOSEC_PER_SEC_I128, ClockType, GregorianTime, SEC_PER_DAYI64, TimePoint,
    TimeSpan, Weekday,
};

/// Combined Gregorian date + wall time with subsecond precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GregorianYmdHms {
    pub yr: i64,
    pub mo: u8,
    pub day: u8,
    pub hr: u8,
    pub min: u8,
    pub sec: u8,     // 0–60 (60 only during leap seconds)
    pub subsec: u64, // attoseconds (0 ≤ subsec < 10¹⁸)
}

impl TimePoint {
    /// Converts a Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// to a proleptic Gregorian date (year, month, day).
    #[inline]
    pub const fn unix_sec_to_gregorian_ymd(unix_sec: i64) -> (i64, u8, u8) {
        let days_since_1970 = unix_sec.div_euclid(SEC_PER_DAYI64);
        // 1970-01-01 00:00:00 UTC is JD 2440588.0
        let jdn = days_since_1970 + 2440588;
        Self::jdn_to_ymd(jdn)
    }

    pub const fn to_gregorian_time(self) -> GregorianTime {
        let clock_type = self.clock_type;

        // Use the new unified function (replaces the old to_gregorian_ymd + to_hms_subsec calls)
        let ymdhms = self.to_gregorian_ymdhms();
        let unix_attosec = self.to_attos_since(TimePoint::UNIX_EPOCH_UTC);

        // Still needed for weekday, wk_sun, wk_mon, and the jd_tt_exact field
        let (jd_days, frac) = self.to_jd_tt_exact();

        let (iso_yr, iso_wk, iso_wkday) =
            self.to_iso_week_date(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let day_of_yr = self.day_of_year(Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let wkday = self.weekday(Some((jd_days, frac)));
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
            attos: ymdhms.subsec,
            iso_yr,
            iso_wk,
            iso_wkday,
            day_of_yr,
            wkday,
            wk_of_yr_sun,
            wk_of_yr_mon,
            jd_tt_exact: (jd_days, frac),
            offset_sec: None,
            tz: None,
            tz_abbrev: None,
            clock_type,
        }
    }

    /// Stripped down version of `TimePoint::to_gregorian_time`.
    ///
    /// Returns the Gregorian date and wall time for this instant.
    ///
    /// - For `ClockType::UTC`: Uses a direct Unix-timestamp-based path (fast and clean).
    /// - For all other clock types: Uses the standard TT-based JD path.
    #[inline]
    pub const fn to_gregorian_ymdhms(self) -> GregorianYmdHms {
        match self.clock_type {
            ClockType::UTC => self.to_gregorian_ymdhms_utc(),
            _ => self.to_gregorian_ymdhms_non_utc(),
        }
    }

    /// Direct UTC civil time path (no TT/JD conversion).
    /// Correctly handles leap seconds (23:59:60 stays on the correct day).
    const fn to_gregorian_ymdhms_utc(self) -> GregorianYmdHms {
        let unix_sec = self.to_unix_sec();
        let canon = self.to_attos_since(TimePoint::UNIX_EPOCH_UTC);
        let subsec = (canon.rem_euclid(ATTOSEC_PER_SEC_I128)) as u64;

        let seconds_since_midnight = unix_sec.rem_euclid(SEC_PER_DAYI64);
        let is_leap_second = Self::is_leap_second_at_unix(unix_sec);

        let unix_sec_for_date = if is_leap_second {
            unix_sec - 1
        } else {
            unix_sec
        };

        let (yr, mo, day) = Self::unix_sec_to_gregorian_ymd(unix_sec_for_date);

        if is_leap_second {
            GregorianYmdHms {
                yr,
                mo,
                day,
                hr: 23,
                min: 59,
                sec: 60,
                subsec,
            }
        } else {
            let hr = (seconds_since_midnight / 3600) as u8;
            let min = ((seconds_since_midnight % 3600) / 60) as u8;
            let sec = (seconds_since_midnight % 60) as u8;

            GregorianYmdHms {
                yr,
                mo,
                day,
                hr,
                min,
                sec,
                subsec,
            }
        }
    }

    /// Non-UTC path (uses the existing TT-based JD machinery)
    const fn to_gregorian_ymdhms_non_utc(self) -> GregorianYmdHms {
        let (jd_days, frac) = self.to_jd_tt_exact();

        // Date
        let jdn = if frac.sec >= 43200 {
            jd_days + 1
        } else {
            jd_days
        };
        let (yr, mo, day) = Self::jdn_to_ymd(jdn);

        // Time
        let tt = self.to_clock_type(ClockType::TT);
        let (_, tt_frac) = tt.to_jd_tt_exact();

        let seconds_since_midnight = if tt_frac.sec >= 43200 {
            tt_frac.sec - 43200
        } else {
            tt_frac.sec + 43200
        };

        let hr = (seconds_since_midnight / 3600) as u8;
        let min = ((seconds_since_midnight % 3600) / 60) as u8;
        let sec = (seconds_since_midnight % 60) as u8;

        GregorianYmdHms {
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            subsec: tt_frac.subsec,
        }
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

    /// Returns the weekday number: `0 = Sunday`, `1 = Monday`, …, `6 = Saturday`.
    ///
    /// The result is computed from the civil (proleptic Gregorian) date of this
    /// `TimePoint`, matching the convention used by [`Self::jdn_to_weekday`].
    pub const fn weekday(self, jd_tt_exact: Option<(i64, TimeSpan)>) -> u8 {
        let (jd_days, frac) = if let Some(jd_tt_exact) = jd_tt_exact {
            jd_tt_exact
        } else {
            self.to_jd_tt_exact()
        };
        let jdn = if frac.sec >= 43200 {
            jd_days + 1
        } else {
            jd_days
        };
        Self::jdn_to_weekday(jdn)
    }

    /// Returns the ordinal day of the year (1-based).
    ///
    /// January 1 is day `1`; December 31 is day `365` or `366` (in leap years).
    /// Uses the proleptic Gregorian calendar.
    pub const fn day_of_year(self, ymd: Option<(i64, u8, u8)>) -> u16 {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_gregorian_ymdhms();
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
    pub const fn wk_sun(self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (year, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_gregorian_ymdhms();
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
    pub const fn wk_mon(self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (year, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_gregorian_ymdhms();
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

    /// Returns the ISO 8601 week date for this `TimePoint`.
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
    pub const fn to_iso_week_date(self, ymd: Option<(i64, u8, u8)>) -> (i64, u8, Weekday) {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_gregorian_ymdhms();
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

    /// Creates a `TimePoint` representing **00:00:00 UTC** on the given proleptic
    /// Gregorian date, converted to the requested [`ClockType`].
    ///
    /// The date components are interpreted according to POSIX civil time
    /// (leap seconds are not inserted into the day count).
    #[inline]
    pub const fn from_gregorian_ymd(yr: i64, mo: u8, day: u8, clock_type: ClockType) -> Self {
        let unix_sec = Self::ymdhms_to_unix_timestamp(yr, mo, day, 0, 0, 0);
        Self::from_unix_sec(unix_sec).to_clock_type(clock_type)
    }

    /// Creates a `TimePoint` at the specified civil UTC instant with full
    /// attosecond precision on the proleptic Gregorian calendar, then converts
    /// it to the requested [`ClockType`].
    ///
    /// All input components are clamped to their valid ranges:
    /// - `mo`   → 0..=12
    /// - `day`  → 0..=31
    /// - `hr`   → 0..=23
    /// - `min`  → 0..=59
    /// - `sec`  → 0..=60 (permits leap seconds)
    /// - `attos` → values ≥ 10¹⁸ are carried into the seconds field
    pub const fn from_gregorian_ymdhms(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        clock_type: ClockType,
    ) -> Self {
        // Clamp inputs to valid ranges
        let mo = if mo > 12 { 12 } else { mo };
        let day = if day > 31 { 31 } else { day };
        let h = if hr > 23 { 23 } else { hr };
        let m = if min > 59 { 59 } else { min };
        let s = if sec > 60 { 60 } else { sec };

        // Carry excess attoseconds into whole seconds
        let extra_sec = (attos / ATTOSEC_PER_SEC) as i64;
        let final_attos = attos % ATTOSEC_PER_SEC;

        // Special handling for leap second (sec == 60)
        // Use 59 + 1 second carry so the timestamp lands correctly
        let (civil_sec, leap_carry) = if s == 60 { (59u8, 1i64) } else { (s, 0i64) };

        let total_day_sec =
            (h as i64) * 3600 + (m as i64) * 60 + civil_sec as i64 + extra_sec + leap_carry;

        let unix_sec = Self::ymdhms_to_unix_timestamp(yr, mo, day, 0, 0, 0) + total_day_sec;

        let base = Self::from_unix_sec(unix_sec);
        base.add(TimeSpan::from_total_attos(final_attos as i128))
            .to_clock_type(clock_type)
    }

    /// Returns true if the given Unix timestamp corresponds to a leap second instant.
    /// In this library the leap second (23:59:60) is represented by the *following*
    /// midnight in the POSIX/Unix count (because leap seconds are not inserted into
    /// the civil second count).  The IANA table entry for that midnight is what we
    /// match.
    pub(crate) const fn is_leap_second_at_unix(unix_sec: i64) -> bool {
        let tod = unix_sec.rem_euclid(SEC_PER_DAYI64);
        if tod != 0 {
            return false;
        }

        const UNIX_TO_NTP: i64 = 2_208_988_800;
        let ntp = unix_sec + UNIX_TO_NTP;

        let mut i = 0usize;
        while i < crate::leap_seconds::LEAP_SECONDS.len() {
            if ntp == crate::leap_seconds::LEAP_SECONDS[i].0 {
                return true;
            }
            i += 1;
        }
        false
    }
}
