use crate::{
    ATTOS_PER_SEC, Dt, GregorianTime, SEC_PER_DAYI64, Scale, Weekday, YmdHms,
    leap_seconds::get_leap_sec,
};

impl Dt {
    /// Converts a Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// to a proleptic Gregorian date (year, month, day).
    #[inline]
    pub const fn unix_sec_to_ymd(unix_sec: i64) -> (i64, u8, u8) {
        let days_since_1970 = unix_sec.div_euclid(SEC_PER_DAYI64);
        // 1970-01-01 00:00:00 UTC is JD 2440588.0
        let jdn = days_since_1970.saturating_add(2440588);
        Self::jdn_to_ymd(jdn)
    }

    pub const fn to_gregorian_time(&self, current: Scale) -> GregorianTime {
        let ymdhms = self.to_ymdhms(current);
        let (iso_yr, iso_wk, iso_wkday) =
            self.to_iso_wk_date(current, Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let day_of_yr = self.day_of_yr(current, Some((ymdhms.yr, ymdhms.mo, ymdhms.day)));
        let jdn = Self::ymd_to_jdn(ymdhms.yr, ymdhms.mo, ymdhms.day);
        let wkday = Self::jdn_to_wkday(jdn);
        let wk_of_yr_sun = self.wk_sun(
            current,
            Some((ymdhms.yr, ymdhms.mo, ymdhms.day)),
            Some(day_of_yr),
        );
        let wk_of_yr_mon = self.wk_mon(
            current,
            Some((ymdhms.yr, ymdhms.mo, ymdhms.day)),
            Some(day_of_yr),
        );

        GregorianTime {
            unix_attosec: ymdhms.unix_attosec,
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
        }
    }

    /// Returns the Gregorian date and wall time for this instant.
    ///
    /// - For `Scale::UTC`: Uses a direct Unix-timestamp-based path (fast and clean).
    /// - For all other scales: Uses the standard TT-based JD path.
    #[inline]
    pub const fn to_ymdhms(&self, current: Scale) -> YmdHms {
        // tai knows whether the seconds lie exactly on a leap second
        let tai = if current.is_tai() {
            *self
        } else {
            self.to(current, Scale::TAI)
        };
        let canon = tai.to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH);

        let unix_sec = canon.sec;
        let attos = canon.attos;

        let is_leap_second = get_leap_sec(&tai, false).is_leap_sec;

        // For the date we always use the previous second when on a leap second
        // (so 23:59:60 stays on the correct civil day).
        let unix_sec_for_date = if is_leap_second {
            unix_sec - 1
        } else {
            unix_sec
        };

        let (yr, mo, day) = Self::unix_sec_to_ymd(unix_sec_for_date);

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
            unix_attosec: canon.to_attos(),
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos,
        }
    }

    /// Converts a proleptic Gregorian calendar date+time to a Unix timestamp
    /// (seconds since 1970-01-01 00:00:00 UTC).
    ///
    /// This version is correct for the full i64 range, including negative years.
    pub const fn ymdhms_to_unix_sec(yr: i64, mo: u8, day: u8, hr: u8, min: u8, sec: u8) -> i64 {
        let jdn = Self::ymd_to_jdn(yr, mo, day);
        // 1970-01-01 00:00:00 UTC corresponds to JD 2440588
        let days_since_1970 = jdn.saturating_sub(2440588);
        let time_of_day = (hr as i64) * 3600 + (min as i64) * 60 + (sec as i64);
        days_since_1970
            .saturating_mul(SEC_PER_DAYI64)
            .saturating_add(time_of_day)
    }

    /// Converts a Julian Day Number (JDN) to a proleptic Gregorian calendar date.
    ///
    /// - Returns `(year, month, day)` where `month` ∈ [1, 12] and `day` ∈ [1, 31]
    ///   (standard 1-based Gregorian values).
    /// - This is the inverse of [`Dt::ymd_to_jdn`](../struct.Dt.html#method.ymd_to_jdn).
    /// - Supports the full `i64` range, including negative years and year zero.
    pub const fn jdn_to_ymd(jdn: i64) -> (i64, u8, u8) {
        let j = jdn as i128;

        #[inline]
        const fn floor_div_pos(a: i128, b: i128) -> i128 {
            if a >= 0 { a / b } else { (a - (b - 1)) / b }
        }

        let a = j + 32044;
        let b = floor_div_pos(4 * a + 3, 146097);
        let c = a - floor_div_pos(b * 146097, 4);
        let d = floor_div_pos(4 * c + 3, 1461);
        let e = c - floor_div_pos(1461 * d, 4);
        let m = floor_div_pos(5 * e + 2, 153);
        let day = (e - floor_div_pos(153 * m + 2, 5) + 1) as u8;
        let mo = (m + 3 - 12 * floor_div_pos(m, 10)) as u8;
        let yr = b * 100 + d - 4800 + floor_div_pos(m, 10);

        (Dt::clamp_i128_to_i64(yr), mo, day)
    }

    /// Computes the Julian Day Number (JDN) for a proleptic Gregorian calendar date at noon UT.
    /// This is the inverse of [`jdn_to_ymd`].
    ///
    /// # Arguments
    ///
    /// * `yr`  - Year (any `i64`; proleptic Gregorian)
    /// * `mo` - Month (**1-based**: `1` = January, `2` = February, ..., `12` = December)
    /// * `day`   - Day of the month (**1-based**: `1` = first day of the month)
    ///
    /// The algorithm matches the standard astronomical convention used throughout the library
    /// (`ymd_to_jdn(2000, 1, 1) == 2451545`).
    ///
    /// # Notes
    ///
    /// - This function assumes a **valid** date. Passing `mo = 0` or `day = 0` (or other
    ///   out-of-range values) will produce incorrect results.
    /// - The result is the integer JDN corresponding to **noon UT** on the given date.
    #[inline]
    pub const fn ymd_to_jdn(yr: i64, mo: u8, day: u8) -> i64 {
        let y = yr as i128;
        let m = mo as i16;
        let d = day as i16;

        let a = (14 - m) / 12;
        let y = y + 4800 - a as i128;
        let m = m + 12 * a - 3;

        let y4 = y >> 2; // floor(y / 4) — arithmetic shift works for negatives

        // floor(y / 100)
        let y100 = if y >= 0 { y / 100 } else { (y - 99) / 100 };

        let y400 = y100 >> 2; // floor(y / 400)

        let day_mo = d + (153 * m + 2) / 5;
        let yr_part = 365 * y + y4 - y100 + y400 - 32045;

        Dt::clamp_i128_to_i64(day_mo as i128 + yr_part)
    }

    /// Returns `true` if the given year is a Gregorian leap year under proleptic rules.
    #[inline]
    pub const fn is_leap_yr(yr: i64) -> bool {
        yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0)
    }

    /// Creates a TAI [`Dt`] from a proleptic gregorian date which is assumed to be on
    /// the provided time scale.
    ///
    /// - Equivalent to [`Dt::from`](../struct.Dt.html#method.from) for the provided date.
    /// - Returned [`Dt`] will be on the **TAI** time scale.
    ///
    /// All input components are clamped to their valid ranges:
    /// - `mo`   → 1..=12 **1 based**
    /// - `day`  → 1..=31 **1 based**
    /// - `hr`   → 0..=23 **0 based**
    /// - `min`  → 0..=59 **0 based**
    /// - `sec`  → 0..=60 **0 based** (permits leap seconds)
    /// - `attos` → values ≥ 10¹⁸ are carried into the seconds field
    ///
    /// ### Notes:
    ///
    /// - Does not perform validation on leap seconds. If 60 seconds are
    ///   provided then an extra second will be added to the resulting [`Dt`].
    pub const fn from_ymdhms_on(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        scale: Scale,
    ) -> Self {
        let mo = Self::clamp_u8(mo, 1, 12);
        let day = Self::clamp_u8(day, 1, 31);
        let h = Self::clamp_u8(hr, 0, 23);
        let m = Self::clamp_u8(min, 0, 59);
        let s = Self::clamp_u8(sec, 0, 60);

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

        let tp =
            Self::from_diff_and_scale(Dt::new(civil_unix_sec, final_attos), Dt::UNIX_EPOCH, scale);
        if is_exact_leap_second {
            Dt::new(tp.sec.saturating_add(1), tp.attos)
        } else {
            tp
        }
    }

    /// Creates a TAI [`Dt`] from a proleptic gregorian date which is assumed to be on
    /// the provided time scale.
    ///
    /// See [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on).
    #[inline]
    pub const fn from_ymd_on(yr: i64, mo: u8, day: u8, scale: Scale) -> Self {
        Dt::from_ymdhms_on(yr, mo, day, 0, 0, 0, 0, scale)
    }

    /// Creates a TAI [`Dt`] from a proleptic gregorian **UTC** date.
    ///
    /// See [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on).
    #[inline]
    pub const fn from_ymdhms(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
    ) -> Self {
        Dt::from_ymdhms_on(yr, mo, day, hr, min, sec, attos, Scale::UTC)
    }

    /// Creates a TAI [`Dt`] from a proleptic gregorian **UTC** date.
    ///
    /// See [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on).
    #[inline]
    pub const fn from_ymd(yr: i64, mo: u8, day: u8) -> Self {
        Dt::from_ymdhms_on(yr, mo, day, 0, 0, 0, 0, Scale::UTC)
    }

    /// Computes the Julian Day Number from a Gregorian year and ordinal day-of-year.
    #[inline]
    pub const fn ydoy_to_jdn(yr: i64, day_of_yr: u16) -> i64 {
        let jdn_jan1 = Self::ymd_to_jdn(yr, 1, 1);
        jdn_jan1.saturating_add(day_of_yr as i64 - 1)
    }

    /// Converts a Julian Day Number to the corresponding weekday number (0 = Sunday … 6 = Saturday).
    #[inline]
    pub const fn jdn_to_wkday(jdn: i64) -> u8 {
        let rem = ((jdn as i128) + 1) % 7;
        let positive = if rem < 0 { rem + 7 } else { rem };
        positive as u8
    }

    /// Computes the Julian Day Number from an ISO week date (Monday-based week).
    pub const fn ymd_to_jdn_from_iso_wk(iso_yr: i64, iso_wk: u8, wkday: Weekday) -> i64 {
        let jan4_jdn = Self::ymd_to_jdn(iso_yr, 1, 4);
        let wd_jan4 = Self::jdn_to_wkday(jan4_jdn);

        let days_to_monday = {
            let tmp = (wd_jan4 as i64).saturating_add(6);
            let rem = tmp % 7;
            if rem < 0 { rem + 7 } else { rem }
        };

        let monday_wk1 = jan4_jdn.saturating_sub(days_to_monday);
        let monday_requested =
            monday_wk1.saturating_add(((iso_wk as i64).saturating_sub(1)).saturating_mul(7));

        monday_requested.saturating_add((wkday.wk_mon() - 1) as i64)
    }

    /// Computes the Julian Day Number from a Sunday-based week-of-year (`%U`).
    pub const fn ymd_to_jdn_from_wk_sun(yr: i64, wk: u8, wkday: Weekday) -> i64 {
        let jan1_jdn = Self::ymd_to_jdn(yr, 1, 1);
        let wd_jan1 = Self::jdn_to_wkday(jan1_jdn);

        let days_to_first_sunday = ((7u8 - wd_jan1) % 7u8) as i64;
        let first_sunday_jdn = jan1_jdn.saturating_add(days_to_first_sunday);

        let sunday_of_wk =
            first_sunday_jdn.saturating_add(((wk as i64).saturating_sub(1)).saturating_mul(7));

        sunday_of_wk.saturating_add(wkday.wk_sun() as i64)
    }

    /// Computes the Julian Day Number from a Monday-based week-of-year (`%W`).
    pub const fn ymd_to_jdn_from_wk_mon(yr: i64, wk: u8, wkday: Weekday) -> i64 {
        let jan1_jdn = Self::ymd_to_jdn(yr, 1, 1);
        let wd_jan1 = Self::jdn_to_wkday(jan1_jdn);

        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_jdn = jan1_jdn.saturating_add(days_to_first_monday);

        let monday_of_wk =
            first_monday_jdn.saturating_add(((wk as i64).saturating_sub(1)).saturating_mul(7));

        monday_of_wk.saturating_add((wkday.wk_mon() - 1) as i64)
    }

    /// Returns `true` if the supplied values form a valid proleptic Gregorian calendar date.
    pub const fn is_valid_ymd(yr: i64, mo: u8, day: u8) -> bool {
        if mo < 1 || mo > 12 || day < 1 {
            return false;
        }
        let days = match mo {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31u8,
            4 | 6 | 9 | 11 => 30u8,
            2 => {
                if Self::is_leap_yr(yr) {
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
    pub const fn has_iso_wk_53(yr: i64) -> bool {
        let jan1_jdn = Self::ymd_to_jdn(yr, 1, 1);
        let wd_jan1 = Self::jdn_to_wkday(jan1_jdn);
        wd_jan1 == 4 || (Self::is_leap_yr(yr) && wd_jan1 == 3)
    }

    /// Returns the ordinal day of the year (1-based).
    ///
    /// January 1 is day `1`; December 31 is day `365` or `366` (in leap years).
    /// Uses the proleptic Gregorian calendar.
    pub const fn day_of_yr(&self, current: Scale, ymd: Option<(i64, u8, u8)>) -> u16 {
        let (yr, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms(current);
            (g.yr, g.mo, g.day)
        };
        let jdn = Self::ymd_to_jdn(yr, month, day);
        let jdn_jan1 = Self::ymd_to_jdn(yr, 1, 1);

        let doy = jdn.saturating_sub(jdn_jan1).saturating_add(1);
        doy as u16
    }

    /// Sunday-based week number (`%U` in strftime).
    ///
    /// Range: `0..=53`.
    /// - Week 0 contains the days *before* the first Sunday of the year.
    /// - Week 1 begins on the first Sunday of the year.
    ///
    /// The optional `ymd` and `doy` arguments are performance optimisations
    /// (same pattern used throughout the file for `day_of_year`, `to_iso_wk_date`, etc.).
    /// Pass whichever you already have; the function will use the fastest path.
    pub const fn wk_sun(&self, current: Scale, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (yr, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms(current);
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_yr(current, ymd)
        };
        let jan1_jdn = Self::ymd_to_jdn(yr, 1, 1);
        let wd_jan1 = Self::jdn_to_wkday(jan1_jdn);
        let days_to_first_sunday = (7u8 - wd_jan1) % 7u8;
        let first_sunday_doy = days_to_first_sunday as u16 + 1;
        if doy < first_sunday_doy {
            0
        } else {
            let days_since_first_sunday = doy.saturating_sub(first_sunday_doy);
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
    /// (same pattern as `wk_sun`, `day_of_yr`, `to_iso_wk_date`, etc.).
    pub const fn wk_mon(&self, current: Scale, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (yr, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms(current);
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_yr(current, ymd)
        };
        let jan1_jdn = Self::ymd_to_jdn(yr, 1, 1);
        let wd_jan1 = Self::jdn_to_wkday(jan1_jdn);
        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_doy = days_to_first_monday as u16 + 1;
        if doy < first_monday_doy {
            0
        } else {
            let days_since_first_monday = doy.saturating_sub(first_monday_doy);
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
    pub const fn to_iso_wk_date(
        &self,
        current: Scale,
        ymd: Option<(i64, u8, u8)>,
    ) -> (i64, u8, Weekday) {
        let (yr, month, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymdhms(current);
            (g.yr, g.mo, g.day)
        };
        let jdn = Self::ymd_to_jdn(yr, month, day);
        let wd = Self::jdn_to_wkday(jdn);
        let wd_iso = if wd == 0 { 7 } else { wd };

        let jan4_jdn = Self::ymd_to_jdn(yr, 1, 4);
        let wd_jan4 = Self::jdn_to_wkday(jan4_jdn);
        let days_to_monday = {
            let tmp = (wd_jan4 as i64) + 6;
            let rem = tmp % 7;
            if rem < 0 { rem + 7 } else { rem }
        };

        let monday_wk1 = jan4_jdn - days_to_monday;

        let days_since = jdn - monday_wk1;

        let wk = if days_since < 0 {
            0u8
        } else {
            ((days_since / 7) + 1) as u8
        };

        let iso_yr = if wk == 0 {
            yr - 1
        } else if wk >= 53 && !Self::has_iso_wk_53(yr) {
            yr + 1
        } else {
            yr
        };

        let iso_wk = if wk == 0 {
            if Self::has_iso_wk_53(yr - 1) { 53 } else { 52 }
        } else if (wk == 53 && !Self::has_iso_wk_53(yr)) || wk > 53 {
            1
        } else {
            wk
        };
        let wkday_enum = match Weekday::from_monday_one_offset(wd_iso) {
            Some(w) => w,
            None => Weekday::Monday,
        };

        (iso_yr, iso_wk, wkday_enum)
    }
}
