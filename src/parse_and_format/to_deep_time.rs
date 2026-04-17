use crate::{
    ClockType, TimePoint,
    parser::{Error, Meridiem, ParseErr, ParsedDate, ParsedTimeScale, TimeZone, Weekday},
};

impl ParsedDate {
    /// Converts parsed date/time components into a high-precision [`TimePoint`].
    ///
    /// This is the core conversion routine used by the date-time parser. It supports
    /// **all five** standard date representations (in strict precedence order):
    ///
    /// 1. Classic Gregorian YMD (`year` + `month` + `day`)
    /// 2. Ordinal date (`year` + `day_of_year`)
    /// 3. ISO 8601 week date (`iso_week_year` + `iso_week`)
    /// 4. Sunday-based week-of-year (`week_sun` – `%U`)
    /// 5. Monday-based week-of-year (`week_mon` – `%W`)
    ///
    /// All civil times are interpreted **in UTC**. The resulting `TimePoint` is
    /// automatically converted to the requested [`ParsedTimeScale`].
    ///
    /// # 12-hour time + meridiem support
    ///
    /// When both `hour` and `meridiem` are present, `hour` is treated as 1–12
    /// and automatically converted to 24-hour format (12 AM → 00, 12 PM → 12, etc.).
    ///
    /// # Leap-second support
    ///
    /// `second == 60` is fully supported and produces the correct physical instant.
    /// The `is_leap_second` flag (set by [`finish()`]) is preserved for future use
    /// but does not affect the conversion math.
    ///
    /// # Calendar validation (new)
    ///
    /// - Full month/day validation (rejects Feb 30, Apr 31, invalid Feb 29, etc.).
    /// - Strict ISO week 53 validation (only years that actually have week 53 are accepted).
    ///
    /// # Errors
    ///
    /// Returns [`Error::simple`] with one of:
    ///
    /// - `TimePointIana`, `TimePointTimeZone`
    /// - `TimePointYearIncompleteDate`
    /// - `TimePointDayOfYearOutOfRange`
    /// - `TimePointIsoWeekOutOfRange` (reused for `%U`/`%W` > 53)
    /// - `TimePointJdnIsNone`
    /// - `TimePointHourOutOfRange`
    /// - `TimePointInvalidDate` (new – add this variant to `ParseErr` if missing)
    pub fn to_time_point(&self) -> Result<TimePoint, Error> {
        if self.iana_name.is_some_and(|n| n.iter().any(|&b| b != 0)) {
            return Err(Error::simple(ParseErr::TimePointIana));
        }
        if matches!(self.tz, Some(TimeZone::Fixed(_))) {
            return Err(Error::simple(ParseErr::TimePointTimeZone));
        }

        // ──────────────────────────────────────────────────────────────
        // Fast path: explicit Unix timestamp
        // ──────────────────────────────────────────────────────────────
        if let Some(unix_secs) = self.unix_timestamp_seconds {
            const UNIX_EPOCH_TO_J2000_NOON_UTC: i64 = 946_728_000;
            let sec = (unix_secs as i64) - UNIX_EPOCH_TO_J2000_NOON_UTC;
            let subsec = self.attos.unwrap_or(0);
            let utc_tp = TimePoint::new(sec, subsec, ClockType::UTC);

            return Ok(match self.timescale {
                ParsedTimeScale::Utc => utc_tp,
                ParsedTimeScale::Tai | ParsedTimeScale::SiContinuous => utc_tp.to_tai(),
                ParsedTimeScale::Tt => utc_tp.to_clock_type(ClockType::TT),
            });
        }

        // ──────────────────────────────────────────────────────────────
        // Resolve 12-hour time + meridiem (AM/PM) to 24-hour hour
        // ──────────────────────────────────────────────────────────────
        let hour = match (self.hour, self.meridiem) {
            (Some(h), Some(m)) => {
                if !(1..=12).contains(&h) {
                    return Err(Error::simple(ParseErr::TimePointHourOutOfRange));
                }
                match (h, m) {
                    (12, Meridiem::AM) => 0,
                    (12, Meridiem::PM) => 12,
                    (h, Meridiem::AM) => h,
                    (h, Meridiem::PM) => h + 12,
                }
            }
            (Some(h), None) => h,
            (None, _) => 0,
        };

        // ──────────────────────────────────────────────────────────────
        // Civil date path
        // ──────────────────────────────────────────────────────────────
        if self.year.is_none() && self.iso_week_year.is_none() {
            return Err(Error::simple(ParseErr::TimePointYearIncompleteDate));
        }

        let minute = self.minute.unwrap_or(0);
        let second = self.second.unwrap_or(0);
        let subsec = self.attos.unwrap_or(0);
        let mut jdn: Option<i64> = None;

        if let Some(year) = self.year {
            if let (Some(m), Some(d)) = (self.month, self.day) {
                // Classic YMD – highest priority + full validation
                if !TimePoint::is_valid_gregorian_date(year, m, d) {
                    return Err(Error::simple(ParseErr::TimePointInvalidDate));
                }
                jdn = Some(TimePoint::gregorian_jdn(year, m, d));
            } else if let Some(doy) = self.day_of_year {
                // Ordinal date (%j) – already validated
                if doy == 0 || doy > 366 || (doy == 366 && !TimePoint::is_leap_year(year)) {
                    return Err(Error::simple(ParseErr::TimePointDayOfYearOutOfRange));
                }
                jdn = Some(TimePoint::gregorian_jdn_from_ordinal(year, doy));
            }
        }

        if jdn.is_none() {
            if let (Some(iso_y), Some(iso_w)) = (self.iso_week_year, self.iso_week) {
                // ISO week date (%G/%V)
                if iso_w == 0 || iso_w > 53 {
                    return Err(Error::simple(ParseErr::TimePointIsoWeekOutOfRange));
                }
                if iso_w == 53 && !TimePoint::has_iso_week_53(iso_y) {
                    return Err(Error::simple(ParseErr::TimePointInvalidDate));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(TimePoint::gregorian_jdn_from_iso_week(iso_y, iso_w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_sun) {
                // Sunday-based week (%U)
                if w > 53 {
                    return Err(Error::simple(ParseErr::TimePointIsoWeekOutOfRange));
                }
                let wd = self.weekday.unwrap_or(Weekday::Sunday);
                jdn = Some(TimePoint::gregorian_jdn_from_week_sun(y, w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_mon) {
                // Monday-based week (%W)
                if w > 53 {
                    return Err(Error::simple(ParseErr::TimePointIsoWeekOutOfRange));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(TimePoint::gregorian_jdn_from_week_mon(y, w, wd));
            }
        }

        let Some(jdn) = jdn else {
            return Err(Error::simple(ParseErr::TimePointJdnIsNone));
        };
        let days_since_j2000 = jdn - 2_451_545i64;
        let seconds_from_noon_utc =
            (hour as i64 - 12) * 3600 + (minute as i64) * 60 + (second as i64);
        let sec_utc = days_since_j2000 * 86_400 + seconds_from_noon_utc;
        let utc_tp = TimePoint::new(sec_utc, subsec, ClockType::UTC);

        Ok(match self.timescale {
            ParsedTimeScale::Utc => utc_tp,
            ParsedTimeScale::Tai | ParsedTimeScale::SiContinuous => utc_tp.to_tai(),
            ParsedTimeScale::Tt => utc_tp.to_clock_type(ClockType::TT),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Error, ParseErr, ParsedDate, strptime};

    /// Small helper for readable JD assertions (matches how the rest of the crate uses `to_jd_tt()`).
    fn jd_tt(tp: &TimePoint) -> f64 {
        tp.to_jd_tt()
    }

    #[test]
    fn test_unix_epoch_1970() {
        let parsed = strptime("%s", "0", false).unwrap();
        let tp = parsed.to_time_point().unwrap();

        let jd = jd_tt(&tp);
        // Unix epoch (1970-01-01 00:00:00 UTC) in TT scale:
        // 2440587.5 + 32.184 / 86400 = 2440587.5003725 exactly.
        assert!(
            (jd - 2440587.5003725).abs() < 1e-10,
            "Expected ~2440587.5003725 (Unix epoch in TT), got {}",
            jd
        );
    }

    #[test]
    fn test_j2000_noon_via_unix_timestamp() {
        let parsed = strptime("%s", "946728000", false).unwrap();
        let tp = parsed.to_time_point().unwrap();

        let jd = jd_tt(&tp);
        // J2000.0 = JD 2451545.0 in TT. Tiny deviation expected due to leap seconds + TAI→TT.
        assert!(
            (jd - 2451545.0).abs() < 0.01,
            "Expected ~2451545.0, got {}",
            jd
        );
    }

    #[test]
    fn test_ymd_and_ordinal_produce_identical_time_point() {
        // YMD and ordinal (%j) paths both set `.year` and produce the exact same instant.
        let ymd = strptime(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
        )
        .unwrap()
        .to_time_point()
        .unwrap();

        let ordinal = strptime("%Y-%j %H:%M:%S.%.f", "2024-106 14:30:45.123456789", false)
            .unwrap()
            .to_time_point()
            .unwrap();

        assert_eq!(jd_tt(&ymd), jd_tt(&ordinal));
        assert_eq!(ymd.to_jd_tt_exact(), ordinal.to_jd_tt_exact());
    }

    #[test]
    fn test_fractional_seconds_are_preserved() {
        let parsed = strptime(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
        )
        .unwrap();
        let tp = parsed.to_time_point().unwrap();

        // 0.123456789 s = 123456789 × 10¹⁸ attoseconds
        let expected = 123_456_789u64 * 1_000_000_000;
        assert_eq!(tp.subsec, expected, "fractional seconds were not preserved");
    }

    #[test]
    fn test_jd_tt_fractional_seconds_preserved() {
        let parsed = strptime(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
        )
        .unwrap();
        let tp = parsed.to_time_point().unwrap();

        let (_, frac) = tp.to_jd_tt_exact();
        let seconds_in_day = frac.as_sec_f();

        // Explanation of the expected value:
        //
        // • Input is 2024-04-15 00:00:00.123456789 **UTC**
        // • to_jd_tt_exact() converts to Terrestrial Time (TT)
        // • TT = UTC + 37 s (leap seconds) + 32.184 s (TT–TAI) = UTC + 69.184 s
        // • Therefore midnight UTC becomes 00:01:09.184 TT
        // • Seconds past noon TT = 12 h + 69.184 s + 0.123456789 s
        //   = 43_200 + 69.184 + 0.123456789 = 43_269.307456789

        const EXPECTED_SECONDS_PAST_NOON_TT: f64 = 43269.307456789;

        assert!(
            (seconds_in_day - EXPECTED_SECONDS_PAST_NOON_TT).abs() < 1e-9,
            "JD TT fractional seconds not preserved.\n\
         Expected ~{EXPECTED_SECONDS_PAST_NOON_TT} s past noon (TT), got {seconds_in_day}"
        );
    }

    #[test]
    fn test_rejects_iana_name() {
        let parsed = strptime("%F %T %Q", "2024-04-15 12:00:00 America/New_York", false).unwrap();
        let err = parsed.to_time_point().unwrap_err();
        assert!(matches!(
            err,
            Error::Simple {
                kind: ParseErr::TimePointIana,
                ..
            }
        ));
    }

    #[test]
    fn test_rejects_fixed_timezone_offset() {
        let parsed = strptime("%F %T %z", "2024-04-15 12:00:00 -0400", false).unwrap();
        let err = parsed.to_time_point().unwrap_err();
        assert!(matches!(
            err,
            Error::Simple {
                kind: ParseErr::TimePointTimeZone,
                ..
            }
        ));
    }

    #[test]
    fn test_incomplete_date_error() {
        // Default ParsedDate has no year → early failure in to_time_point.
        let pd = ParsedDate::default();
        let err = pd.to_time_point().unwrap_err();
        assert!(matches!(
            err,
            Error::Simple {
                kind: ParseErr::TimePointYearIncompleteDate,
                ..
            }
        ));
    }

    #[test]
    fn test_day_of_year_out_of_range_non_leap_year() {
        // 2023 is not a leap year. We build a ParsedDate manually because the parser
        // rejects day 366 (u8 limit in parse_u8_padded), so we never reach to_time_point
        // with a parser-constructed value. This test directly exercises the leap-year check.
        let mut pd = ParsedDate::default();
        pd.year = Some(2023);
        pd.day_of_year = Some(366);

        let err = pd.to_time_point().unwrap_err();
        assert!(matches!(
            err,
            Error::Simple {
                kind: ParseErr::TimePointDayOfYearOutOfRange,
                ..
            }
        ));
    }

    #[test]
    fn test_iso_week_out_of_range() {
        // Parser rejects week 54, so we build manually to hit the to_time_point check.
        let mut pd = ParsedDate::default();
        pd.iso_week_year = Some(2024);
        pd.iso_week = Some(54);
        pd.weekday = Some(Weekday::Monday); // required for the ISO path

        let err = pd.to_time_point().unwrap_err();
        assert!(matches!(
            err,
            Error::Simple {
                kind: ParseErr::TimePointIsoWeekOutOfRange,
                ..
            }
        ));
    }

    #[test]
    fn test_pure_iso_week_date() {
        // Pure ISO week date (%G/%V/%u) is now fully supported in to_time_point
        // via the iso_week_year + iso_week + weekday path (no regular .year required).
        let parsed = strptime("%G-W%V-%u", "2024-W16-1", false).unwrap();
        let tp_iso = parsed.to_time_point().unwrap();

        // 2024-W16-1 is Monday, April 15, 2024
        let ymd = strptime("%Y-%m-%d", "2024-04-15", false)
            .unwrap()
            .to_time_point()
            .unwrap();

        assert_eq!(jd_tt(&tp_iso), jd_tt(&ymd));
        assert_eq!(tp_iso.to_jd_tt_exact(), ymd.to_jd_tt_exact());
    }
}
