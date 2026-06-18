use crate::{ATTOS_PER_SEC, Dt, SEC_PER_DAYI64, Scale, Weekday, YmdHms, leap_seconds::leap_sec};

impl Dt {
    pub(crate) const DAYS_IN_GREGORIAN_MONTHS: [u8; 12] =
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    // pub(crate) const DAYS_IN_GREGORIAN_MONTHS_LEAP_YR: [u8; 12] =
    //     [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    /// Converts a Unix timestamp (seconds since 1970-01-01 00:00:00)
    /// to a proleptic Gregorian date (year, month, day).
    pub const fn unix_sec_to_ymd(unix_sec: i64) -> (i64, u8, u8) {
        let days = unix_sec.div_euclid(86400);

        // Shift so we work relative to 0000-03-01 (makes leap year math cleaner)
        let z = days + 719468;

        let era = if z >= 0 {
            z / 146097
        } else {
            (z - 146096) / 146097
        };
        let doe = z - era * 146097; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2) / 153; // [0, 11]
        let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
        let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]

        let yr = y + if m <= 2 { 1 } else { 0 };

        (yr, m as u8, d as u8)
    }

    /// Returns the calendar date and time for this instant.
    ///
    /// Converts to this [`Dt`]s `target` time scale using the internal current
    /// `scale` before producing a result.
    ///
    /// ## Returns
    ///
    /// A [`YmdHms`] containing:
    ///
    /// - `yr`, `mo`, `day` — calendar date
    /// - `hr` (0–23), `min` (0–59), `sec` (0–60)
    /// - `attos` — fractional second in attoseconds (`0 ≤ attos < 10¹⁸`)
    /// - `scale` — time scale that the object is in
    ///
    /// ## Leap-second handling
    ///
    /// If the [`Dt`]'s `target` time scale is one of the scales that use leap seconds
    /// (`UTC`, `UtcSpice`, or `UtcHist`) **and** the instant falls exactly on a leap
    /// second, (requires the objects current time scale **not** be UTC) the returned
    /// `sec` will be `60`. In every other case `sec` is in the range `0..=59`.
    ///
    /// The implementation converts internally to TAI before checking leap-second status.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // `from_ymd` always returns a TAI instant
    /// let dt = Dt::from_ymd(2024, 6, 15, Scale::UTC, 12, 30, 45, 0);
    /// let ymd = dt.to_ymd();
    ///
    /// assert_eq!(ymd.yr(), 2024);
    /// assert_eq!(ymd.mo(), 6);
    /// assert_eq!(ymd.day(), 15);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 30);
    /// assert_eq!(ymd.sec(), 45);
    /// assert!(ymd.attos() == 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
    pub const fn to_ymd(&self) -> YmdHms {
        let from_unix_epoch = self.to_scale_and_diff(Dt::UNIX_EPOCH, true);

        let unix_sec = from_unix_epoch.to_sec64();
        let frac = from_unix_epoch.to_sec_ufrac();
        let (yr, mo, day) = Self::unix_sec_to_ymd(unix_sec);

        let seconds_since_midnight = unix_sec.rem_euclid(SEC_PER_DAYI64);
        let hr = (seconds_since_midnight / 3600) as u8;
        let min = ((seconds_since_midnight % 3600) / 60) as u8;
        let mut sec = (seconds_since_midnight % 60) as u8;
        let is_leap = if self.target.uses_leap_seconds() {
            match self.to_tai().leap_sec(false) {
                Some(i) => i.is_leap_sec,
                None => false,
            }
        } else {
            false
        };
        if is_leap {
            sec += 1;
        }

        YmdHms {
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos: frac,
            scale: self.target,
            old_scale: self.scale,
        }
    }

    /// Converts a proleptic Gregorian calendar date+time to a Unix timestamp
    /// (seconds since 1970-01-01 00:00:00).
    ///
    /// - Expects **1 based** `mo` and `day`, and **0 based** `hr`, `min`, and `sec`.
    /// - Does not perform any time scale conversions.
    /// - Expects pre-clamped values.
    pub const fn ymd_to_unix_sec(yr: i64, mo: u8, day: u8, hr: u8, min: u8, sec: u8) -> i64 {
        let jd = Self::ymd_to_jd(yr, mo, day);
        // 1970-01-01 00:00:00 UTC corresponds to JD 2440588
        let days_since_1970 = jd.saturating_sub(2440588);
        let time_of_day = (hr as i64) * 3600 + (min as i64) * 60 + (sec as i64);
        days_since_1970
            .saturating_mul(SEC_PER_DAYI64)
            .saturating_add(time_of_day)
    }

    /// Converts a Julian Day Number (JD) to a proleptic Gregorian calendar date.
    ///
    /// - Returns `(year, month, day)` where `month` ∈ [1, 12] and `day` ∈ [1, 31]
    ///   (standard 1-based Gregorian values).
    /// - This is the inverse of [`Dt::ymd_to_jd`](../struct.Dt.html#method.ymd_to_jd).
    /// - Supports the full `i64` range, including negative years and year zero.
    pub const fn jd_to_ymd(jd: i64) -> (i64, u8, u8) {
        let j = jd as i128;

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

        (Dt::i128_to_i64(yr), mo, day)
    }

    /// Computes the Julian Day Number (JD) for a proleptic Gregorian calendar date at noon UT.
    /// This is the inverse of [`jd_to_ymd`].
    ///
    /// ## Arguments
    ///
    /// * `yr`  - Year (any `i64`; proleptic Gregorian)
    /// * `mo` - Month (**1-based**: `1` = January, `2` = February, ..., `12` = December)
    /// * `day`   - Day of the month (**1-based**: `1` = first day of the month)
    ///
    /// The algorithm matches the standard astronomical convention used throughout the library
    /// (`ymd_to_jd(2000, 1, 1) == 2451545`).
    ///
    /// ## Notes
    ///
    /// - This function expects **1 based** `mo` and `day`. Passing `mo = 0` or `day = 0` (or other
    ///   out-of-range values) will produce incorrect results as this function does not perform
    ///   value clamping.
    /// - Does not deal with bad inputs like February with 30 days, does not do any clamping. If you
    ///   need to sanitize a year, month, day input use
    ///   [`Dt::clamp_mdhms`](../struct.Dt.html#method.clamp_mdhms) first.
    /// - The result is the integer JD corresponding to **noon** on the given date.
    #[inline]
    pub const fn ymd_to_jd(yr: i64, mo: u8, day: u8) -> i64 {
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

        Dt::i128_to_i64(day_mo as i128 + yr_part)
    }

    /// Creates a **TAI** [`Dt`] from a proleptic gregorian date which is assumed to be on
    /// the provided time scale.
    ///
    /// - Equivalent to converting to `TAI` for the provided date. This means for example that
    ///   when using `Scale::UTC` leap seconds are potentially added to the returned [`Dt`].
    /// - The returned [`Dt`] will have its `scale` field set to `TAI` and its `target` field
    ///   set to the provided time scale argument from this fn. This makes functions such as
    ///   [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) more ergonomic.
    ///
    /// All input components are clamped to their valid ranges:
    /// - `mo`   → 1..=12 **1 based**
    /// - `day`  → 1..=31 **1 based**
    /// - `hr`   → 0..=23 **0 based**
    /// - `min`  → 0..=59 **0 based**
    /// - `sec`  → 0..=60 **0 based** (permits leap seconds)
    /// - `attos` → 10¹⁸ **0 based** (clamped to under 1 second)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "jiff-tz")]
    /// # {
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// // library zero is 2000-01-01 noon TAI
    /// let tai = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// assert_eq!(tai, Dt::ZERO);
    ///
    /// // utc noon
    /// let utc = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    /// // output with timezone requires jiff-tz feature
    /// // because from_ymd used Scale::UTC, the output is converted
    /// // back to UTC before being offset by the timezone
    /// let s = utc.to_str_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En).unwrap();
    /// assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    pub const fn from_ymd(
        yr: i64,
        mo: u8,
        day: u8,
        scale: Scale,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
    ) -> Dt {
        let (mo, day, hr, min, sec) = Dt::clamp_mdhms(yr, mo, day, hr, min, sec);
        let attos = Dt::clamp_u64(attos, 0, ATTOS_PER_SEC - 1);

        let sec_is_60 = sec == 60;
        let s_for_unix = if sec_is_60 { 59 } else { sec };

        let unix_sec = Dt::ymd_to_unix_sec(yr, mo, day, hr, min, s_for_unix);
        let unix_attos = Dt::sec_to_attos(unix_sec as i128) + (attos as i128);

        if sec_is_60 && scale.uses_leap_seconds() {
            let t =
                Dt::from_diff_and_scale(Dt::new(unix_attos, scale, scale), Dt::UNIX_EPOCH, false);
            let is_leap = match leap_sec(t.add_sec(1).to_sec64(), false) {
                Some(i) => i.is_leap_sec,
                None => false,
            };
            if is_leap { t.add_sec(1) } else { t }
        } else {
            Dt::from_diff_and_scale(Dt::new(unix_attos, scale, scale), Dt::UNIX_EPOCH, false)
        }
    }

    /// Computes the Julian Day Number from a Gregorian year and ordinal day-of-year.
    #[inline]
    pub const fn ydoy_to_jd(yr: i64, day_of_yr: u16) -> i64 {
        let jd_jan1 = Self::ymd_to_jd(yr, 1, 1);
        jd_jan1.saturating_add(day_of_yr as i64 - 1)
    }

    /// Converts a Julian Day Number to the corresponding weekday number (0 = Sunday … 6 = Saturday).
    #[inline]
    pub const fn jd_to_wkday(jd: i64) -> u8 {
        let rem = ((jd as i128) + 1) % 7;
        let positive = if rem < 0 { rem + 7 } else { rem };
        positive as u8
    }

    /// Computes the Julian Day Number from an ISO week date (Monday-based week).
    pub const fn iso_wk_to_jd(iso_yr: i64, iso_wk: u8, wkday: Weekday) -> i64 {
        let jan4_jd = Self::ymd_to_jd(iso_yr, 1, 4);
        let wd_jan4 = Self::jd_to_wkday(jan4_jd);

        let days_to_monday = {
            let tmp = (wd_jan4 as i64).saturating_add(6);
            let rem = tmp % 7;
            if rem < 0 { rem + 7 } else { rem }
        };

        let monday_wk1 = jan4_jd.saturating_sub(days_to_monday);
        let monday_requested =
            monday_wk1.saturating_add(((iso_wk as i64).saturating_sub(1)).saturating_mul(7));

        monday_requested.saturating_add((wkday.wkday_mon_0_based()) as i64)
    }

    /// Computes the Julian Day Number from a Sunday-based week-of-year (`%U`).
    pub const fn wk_sun_to_jd(yr: i64, wk: u8, wkday: Weekday) -> i64 {
        let jan1_jd = Self::ymd_to_jd(yr, 1, 1);
        let wd_jan1 = Self::jd_to_wkday(jan1_jd);

        let days_to_first_sunday = ((7u8 - wd_jan1) % 7u8) as i64;
        let first_sunday_jd = jan1_jd.saturating_add(days_to_first_sunday);

        let sunday_of_wk =
            first_sunday_jd.saturating_add(((wk as i64).saturating_sub(1)).saturating_mul(7));

        sunday_of_wk.saturating_add(wkday.wkday_sun_0_based() as i64)
    }

    /// Computes the Julian Day Number from a Monday-based week-of-year (`%W`).
    pub const fn wk_mon_to_jd(yr: i64, wk: u8, wkday: Weekday) -> i64 {
        let jan1_jd = Self::ymd_to_jd(yr, 1, 1);
        let wd_jan1 = Self::jd_to_wkday(jan1_jd);

        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_jd = jan1_jd.saturating_add(days_to_first_monday);

        let monday_of_wk =
            first_monday_jd.saturating_add(((wk as i64).saturating_sub(1)).saturating_mul(7));

        monday_of_wk.saturating_add((wkday.wkday_mon_0_based()) as i64)
    }

    /// Returns `true` if the given year is a Gregorian leap year under proleptic rules.
    #[inline(always)]
    pub const fn is_leap_yr(yr: i64) -> bool {
        (yr & 3 == 0) && ((yr & 15 == 0) || (yr % 25 != 0))
    }

    /// Returns `true` if the supplied values form a valid proleptic Gregorian calendar date.
    #[inline]
    pub const fn is_valid_ymd(yr: i64, mo: u8, day: u8) -> bool {
        if mo < 1 || mo > 12 || day < 1 {
            return false;
        }
        // 0 = Jan, 1 = Feb, ..., 11 = Dec
        let days = Self::DAYS_IN_GREGORIAN_MONTHS[(mo - 1) as usize];
        if mo == 2 && Self::is_leap_yr(yr) {
            day <= days + 1 // 28 → 29
        } else {
            day <= days
        }
    }

    /// Returns `true` if the given Gregorian year contains an ISO week 53.
    pub const fn has_iso_wk_53(yr: i64) -> bool {
        let jan1_jd = Self::ymd_to_jd(yr, 1, 1);
        let wd_jan1 = Self::jd_to_wkday(jan1_jd);
        wd_jan1 == 4 || (Self::is_leap_yr(yr) && wd_jan1 == 3)
    }

    /// Returns the ordinal day of the year (1-based).
    ///
    /// January 1 is day `1`; December 31 is day `365` or `366` (in leap years).
    /// Uses the proleptic Gregorian calendar.
    pub const fn day_of_yr(&self, ymd: Option<(i64, u8, u8)>) -> u16 {
        let (yr, mo, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymd();
            (g.yr, g.mo, g.day)
        };
        Self::_day_of_yr(yr, mo, day)
    }

    pub(crate) const fn _day_of_yr(yr: i64, mo: u8, day: u8) -> u16 {
        let jd = Self::ymd_to_jd(yr, mo, day);
        let jd_jan1 = Self::ymd_to_jd(yr, 1, 1);

        let doy = jd.saturating_sub(jd_jan1).saturating_add(1);
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
    pub const fn wk_sun(&self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (yr, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymd();
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_yr(ymd)
        };
        Self::_wk_sun(yr, doy)
    }

    pub(crate) const fn _wk_sun(yr: i64, doy: u16) -> u8 {
        let jan1_jd = Self::ymd_to_jd(yr, 1, 1);
        let wd_jan1 = Self::jd_to_wkday(jan1_jd);
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
    pub const fn wk_mon(&self, ymd: Option<(i64, u8, u8)>, doy: Option<u16>) -> u8 {
        let (yr, _, _) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymd();
            (g.yr, g.mo, g.day)
        };
        let doy = if let Some(doy) = doy {
            doy
        } else {
            self.day_of_yr(ymd)
        };
        Self::_wk_mon(yr, doy)
    }

    pub(crate) const fn _wk_mon(yr: i64, doy: u16) -> u8 {
        let jan1_jd = Self::ymd_to_jd(yr, 1, 1);
        let wd_jan1 = Self::jd_to_wkday(jan1_jd);
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
    pub const fn to_iso_wk_date(&self, ymd: Option<(i64, u8, u8)>) -> (i64, u8, Weekday) {
        let (yr, mo, day) = if let Some(ymd) = ymd {
            ymd
        } else {
            let g = self.to_ymd();
            (g.yr, g.mo, g.day)
        };
        Self::_to_iso_wk_date(yr, mo, day)
    }

    pub(crate) const fn _to_iso_wk_date(yr: i64, mo: u8, day: u8) -> (i64, u8, Weekday) {
        let jd = Self::ymd_to_jd(yr, mo, day);
        let wd = Self::jd_to_wkday(jd);
        let wd_iso = if wd == 0 { 7 } else { wd };

        let jan4_jd = Self::ymd_to_jd(yr, 1, 4);
        let wd_jan4 = Self::jd_to_wkday(jan4_jd);
        let days_to_monday = {
            let tmp = (wd_jan4 as i64) + 6;
            let rem = tmp % 7;
            if rem < 0 { rem + 7 } else { rem }
        };

        let monday_wk1 = jan4_jd - days_to_monday;

        let days_since = jd - monday_wk1;

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
        let wkday_enum = match Weekday::from_monday_1_based(wd_iso) {
            Some(w) => w,
            None => Weekday::Monday,
        };

        (iso_yr, iso_wk, wkday_enum)
    }

    /// Number of days in a month under proleptic Gregorian rules.
    #[inline]
    pub const fn days_in_month(yr: i64, mo: u8) -> u8 {
        match mo {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_yr(yr) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    /// Clamps month, day, hour, minutes, and seconds values. Clamps days to what is
    /// correct for that particular propleptic gregorian month.
    ///
    /// For example the year 2000 is a leap year, and February in that year has 29 days
    /// so the days are clamped to 1-29 in that year, but 1-28 in non-leap years.
    pub const fn clamp_mdhms(
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
    ) -> (u8, u8, u8, u8, u8) {
        let mo = Self::clamp_u8(mo, 1, 12);
        let max_day = Self::days_in_month(yr, mo);
        let day = Self::clamp_u8(day, 1, max_day);
        let h = Self::clamp_u8(hr, 0, 23);
        let m = Self::clamp_u8(min, 0, 59);
        let s = Self::clamp_u8(sec, 0, 60);

        (mo, day, h, m, s)
    }

    /// Number of days since 1958-01-01 (proleptic Gregorian) → `(year, month, day)`.
    /// This is the inverse of [`Dt::gregorian_to_days_since_1958`].
    #[inline]
    pub const fn days_since_1958_to_gregorian(days_since_epoch: i64) -> (i64, u8, u8) {
        let jd_1958 = Dt::ymd_to_jd(1958, 1, 1);
        let jd = jd_1958.saturating_add(days_since_epoch);
        Dt::jd_to_ymd(jd)
    }

    /// Inverse of [`Dt::days_since_1958_to_gregorian`].
    #[inline]
    pub const fn gregorian_to_days_since_1958(year: i64, month: u8, day: u8) -> i64 {
        let jd = Dt::ymd_to_jd(year, month, day);
        let jd_1958 = Dt::ymd_to_jd(1958, 1, 1);
        jd.saturating_sub(jd_1958)
    }
}
