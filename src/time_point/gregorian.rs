use crate::{
    ATTOSEC_PER_SEC_I128, ClockType, GregorianTime, SEC_PER_DAYI128, TT_TAI_OFFSET_SPAN, TimePoint,
    TimeSpan, Weekday, leap_seconds::leap_seconds_before,
};

impl TimePoint {
    pub const fn to_gregorian_time(self) -> GregorianTime {
        let clock_type = self.clock_type;
        let utc = self.to_clock_type(ClockType::UTC);
        let unix_attosec = self.to_canonical_attoseconds();
        let (jd_days, frac) = utc.to_jd_tt_exact();
        let (yr, mo, day) = utc.to_gregorian_date(Some((jd_days, frac)));
        let (hr, min, sec, attos) = utc.to_hms_subsec();
        let (iso_yr, iso_wk, iso_wkday) = utc.to_iso_week_date(Some((yr, mo, day)));
        let day_of_yr = utc.day_of_year(Some((yr, mo, day)));
        let wkday = utc.weekday(Some((jd_days, frac)));
        let wk_of_yr_sun = utc.wk_sun(Some((yr, mo, day)), Some(day_of_yr));
        let wk_of_yr_mon = utc.wk_mon(Some((yr, mo, day)), Some(day_of_yr));
        GregorianTime {
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
            jd_tt_exact: (jd_days, frac),
            offset_sec: None,
            tz: None,
            tz_abbrev: None,
            clock_type: clock_type,
        }
    }

    pub const fn to_gregorian_date(self, jd_tt_exact: Option<(i64, TimeSpan)>) -> (i64, u8, u8) {
        let (jd_days, frac) = if let Some(jd_tt_exact) = jd_tt_exact {
            jd_tt_exact
        } else {
            self.to_jd_tt_exact()
        };
        let jdn = match self.clock_type {
            ClockType::UTC => {
                let tai = self.to_tai();
                let leaps = leap_seconds_before(tai);
                let offset_attos =
                    (leaps as i128) * ATTOSEC_PER_SEC_I128 + TT_TAI_OFFSET_SPAN.total_attos();

                let mut utc_frac_attos = frac.total_attos() as i128 - offset_attos;
                let day_attos = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;
                let mut utc_jd_days = jd_days;

                if utc_frac_attos < 0 {
                    utc_frac_attos += day_attos;
                    utc_jd_days -= 1;
                } else if utc_frac_attos >= day_attos {
                    utc_frac_attos -= day_attos;
                    utc_jd_days += 1;
                }

                let seconds_since_noon = (utc_frac_attos / ATTOSEC_PER_SEC_I128) as i64;
                if seconds_since_noon >= 43200 {
                    utc_jd_days + 1
                } else {
                    utc_jd_days
                }
            }
            _ => {
                if frac.sec >= 43200 {
                    jd_days + 1
                } else {
                    jd_days
                }
            }
        };

        Self::jdn_to_gregorian(jdn)
    }

    pub const fn to_hms_subsec(self) -> (u8, u8, u8, u64) {
        match self.clock_type {
            ClockType::UTC => {
                let tai = self.to_tai();
                let leaps = leap_seconds_before(tai);
                let offset_attos =
                    (leaps as i128) * ATTOSEC_PER_SEC_I128 + TT_TAI_OFFSET_SPAN.total_attos();

                let (_, frac) = self.to_jd_tt_exact();
                let mut utc_frac_attos = frac.total_attos() as i128 - offset_attos;

                // Normalize to [0, one day) — this is still "seconds since noon"
                let day_attos = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;
                if utc_frac_attos < 0 {
                    utc_frac_attos += day_attos;
                } else if utc_frac_attos >= day_attos {
                    utc_frac_attos -= day_attos;
                }

                let seconds_since_noon = (utc_frac_attos / ATTOSEC_PER_SEC_I128) as i64;
                let subsec = (utc_frac_attos % ATTOSEC_PER_SEC_I128) as u64;

                // Convert "seconds since noon" → "seconds since midnight" (same logic as non-UTC path)
                let seconds_since_midnight = if seconds_since_noon >= 43200 {
                    seconds_since_noon - 43200
                } else {
                    seconds_since_noon + 43200
                };

                if seconds_since_midnight == 86400 {
                    // Leap second case
                    (23, 59, 60, subsec)
                } else {
                    let hour = (seconds_since_midnight / 3600) as u8;
                    let minute = ((seconds_since_midnight % 3600) / 60) as u8;
                    let second = (seconds_since_midnight % 60) as u8;
                    (hour, minute, second, subsec)
                }
            }
            _ => {
                // All other scales (including TT, TAI, TDB, TCG, etc.) use the standard
                // TT-based JD machinery for time-of-day.
                let tt = self.to_clock_type(ClockType::TT);
                let (_, frac) = tt.to_jd_tt_exact();
                let seconds_since_midnight = if frac.sec >= 43200 {
                    frac.sec - 43200
                } else {
                    frac.sec + 43200
                };
                let hour = (seconds_since_midnight / 3600) as u8;
                let minute = ((seconds_since_midnight % 3600) / 60) as u8;
                let second = (seconds_since_midnight % 60) as u8;
                (hour, minute, second, frac.subsec)
            }
        }
    }

    /// Converts a Julian Day Number (JDN) to a proleptic Gregorian calendar date.
    ///
    /// Returns `(year, month, day)` where `month` ∈ [1, 12] and `day` ∈ [1, 31]
    /// (standard 1-based Gregorian values).
    ///
    /// This is the inverse of [`Self::gregorian_jdn`]. Supports the full `i64`
    /// range, including negative years and year zero.
    pub const fn jdn_to_gregorian(jdn: i64) -> (i64, u8, u8) {
        // Use i128 internally to avoid overflow on full i64 JDN range
        let j = jdn as i128;
        let a = j + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (b * 146097) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;
        let day = (e - (153 * m + 2) / 5 + 1) as u8;
        let month = (m + 3 - 12 * (m / 10)) as u8;
        let year = b * 100 + d - 4800 + (m / 10);
        (year as i64, month, day)
    }

    /// Computes the Julian Day Number (JDN) for a proleptic Gregorian calendar date at noon UT.
    ///
    /// The algorithm matches the standard astronomical convention used throughout the library
    /// (`gregorian_jdn(2000, 1, 1) == 2451545`).
    #[inline]
    pub const fn gregorian_jdn(year: i64, month: u8, day: u8) -> i64 {
        let a = (14 - month as i64) / 12;
        let y = year + 4800 - a;
        let m = month as i64 + 12 * a - 3;
        day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }

    /// Returns `true` if the given year is a Gregorian leap year under proleptic rules.
    #[inline(always)]
    pub const fn is_leap_year(year: i64) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Computes the Julian Day Number from a Gregorian year and ordinal day-of-year.
    #[inline]
    pub const fn gregorian_jdn_from_ordinal(year: i64, day_of_year: u16) -> i64 {
        let jdn_jan1 = Self::gregorian_jdn(year, 1, 1);
        jdn_jan1 + (day_of_year as i64 - 1)
    }

    /// Converts a Julian Day Number to the corresponding weekday number (0 = Sunday … 6 = Saturday).
    #[inline(always)]
    pub const fn jdn_to_weekday(jdn: i64) -> u8 {
        ((jdn + 1) % 7) as u8
    }

    /// Computes the Julian Day Number from an ISO week date (Monday-based week).
    #[inline]
    pub const fn gregorian_jdn_from_iso_week(iso_year: i64, iso_week: u8, weekday: Weekday) -> i64 {
        let jan4_jdn = Self::gregorian_jdn(iso_year, 1, 4);
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
    pub const fn gregorian_jdn_from_week_sun(year: i64, week: u8, weekday: Weekday) -> i64 {
        let jan1_jdn = Self::gregorian_jdn(year, 1, 1);
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
    pub const fn gregorian_jdn_from_week_mon(year: i64, week: u8, weekday: Weekday) -> i64 {
        let jan1_jdn = Self::gregorian_jdn(year, 1, 1);
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
    pub const fn is_valid_gregorian_date(year: i64, month: u8, day: u8) -> bool {
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
    #[inline]
    pub const fn has_iso_week_53(year: i64) -> bool {
        let jan1_jdn = Self::gregorian_jdn(year, 1, 1);
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
    #[inline]
    pub const fn day_of_year(self, ymd: Option<(i64, u8, u8)>) -> u16 {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            self.to_gregorian_date(None)
        };
        let jdn = Self::gregorian_jdn(year, month, day);
        let jdn_jan1 = Self::gregorian_jdn(year, 1, 1);
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
            self.to_gregorian_date(None)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_year(ymd)
        };
        let jan1_jdn = Self::gregorian_jdn(year, 1, 1);
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
            self.to_gregorian_date(None)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_year(ymd)
        };
        let jan1_jdn = Self::gregorian_jdn(year, 1, 1);
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
    /// it is used directly; otherwise [`to_gregorian_date`](Self::to_gregorian_date)
    /// is called internally.
    pub const fn to_iso_week_date(self, ymd: Option<(i64, u8, u8)>) -> (i64, u8, Weekday) {
        let (year, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            self.to_gregorian_date(None)
        };
        let jdn = Self::gregorian_jdn(year, month, day);
        let wd = Self::jdn_to_weekday(jdn);
        let wd_iso = if wd == 0 { 7 } else { wd };

        let jan4_jdn = Self::gregorian_jdn(year, 1, 4);
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
