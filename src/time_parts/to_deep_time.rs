use crate::tzdb::offset_info_at_local;
use crate::{
    ClockType, TimePoint,
    error::{DtErrKind, DtErr},
    {Meridiem, Offset, TimeParts, Weekday},
};
use crate::{J2000_JD_TT, SEC_PER_DAYI64, UNIX_EPOCH_TO_J2000_NOON_UTC, an_err};

impl TimeParts {
    pub fn to_time_point(&self, clock_type: Option<ClockType>) -> Result<TimePoint, DtErr> {
        // ──────────────────────────────────────────────────────────────
        // Fast path: explicit Unix timestamp
        // ──────────────────────────────────────────────────────────────
        if let Some(unix_secs) = self.unix_timestamp_seconds {
            let sec = (unix_secs as i64) - UNIX_EPOCH_TO_J2000_NOON_UTC;
            let subsec = self.attos.unwrap_or(0);
            return Ok(TimePoint::new(sec, subsec, ClockType::UTC)
                .to_clock_type(clock_type.unwrap_or(self.clock_type)));
        }

        // ──────────────────────────────────────────────────────────────
        // Resolve 12-hour time + meridiem (AM/PM) to 24-hour hour
        // ──────────────────────────────────────────────────────────────
        let hour = match (self.hour, self.meridiem) {
            (Some(h), Some(m)) => {
                if !(1..=12).contains(&h) {
                    return Err(an_err!(DtErrKind::OutOfRange, "hour: {}", h));
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
            return Err(an_err!(DtErrKind::Incomplete, "no year"));
        }

        let minute = self.minute.unwrap_or(0);
        let second = self.second.unwrap_or(0);
        let subsec = self.attos.unwrap_or(0);
        let mut jdn: Option<i64> = None;

        if let Some(year) = self.year {
            if let (Some(m), Some(d)) = (self.month, self.day) {
                // Classic YMD – highest priority + full validation
                if !TimePoint::is_valid_ymd(year, m, d) {
                    return Err(an_err!(DtErrKind::InvalidInput, "ymd"));
                }
                jdn = Some(TimePoint::ymd_to_jdn(year, m, d));
            } else if let Some(doy) = self.day_of_year {
                // Ordinal date (%j) – already validated
                if doy == 0 || doy > 366 || (doy == 366 && !TimePoint::is_leap_year(year)) {
                    return Err(an_err!(DtErrKind::OutOfRange, "day of year"));
                }
                jdn = Some(TimePoint::ydoy_to_jdn(year, doy));
            }
        }

        if jdn.is_none() {
            if let (Some(iso_y), Some(iso_w)) = (self.iso_week_year, self.iso_week) {
                // ISO week date (%G/%V)
                if iso_w == 0 || iso_w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "iso week"));
                }
                if iso_w == 53 && !TimePoint::has_iso_week_53(iso_y) {
                    return Err(an_err!(DtErrKind::InvalidItem, "iso week"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(TimePoint::ymd_to_jdn_from_iso_week(iso_y, iso_w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_sun) {
                // Sunday-based week (%U)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Sunday);
                jdn = Some(TimePoint::ymd_to_jdn_from_week_sun(y, w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_mon) {
                // Monday-based week (%W)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(TimePoint::ymd_to_jdn_from_week_mon(y, w, wd));
            }
        }

        let Some(jdn) = jdn else {
            return Err(an_err!(DtErrKind::InvalidInput, "could not create julian"));
        };
        let days_since_j2000 = jdn - J2000_JD_TT;
        let seconds_from_noon_utc =
            (hour as i64 - 12) * 3600 + (minute as i64) * 60 + (second as i64);
        let mut sec_utc = days_since_j2000 * SEC_PER_DAYI64 + seconds_from_noon_utc;

        // ──────────────────────────────────────────────────────────────
        // Apply timezone correction (IANA or Fixed offset)
        // ──────────────────────────────────────────────────────────────

        if let Some(name) = &self.iana_name {
            let name_str = name.as_str().map_err(|e| {
                an_err!(
                    DtErrKind::InvalidBytes,
                    "invalid iana ascii: {:?}: {}",
                    name,
                    e
                )
            })?;

            if !name_str.is_empty() {
                let provisional_unix = sec_utc + UNIX_EPOCH_TO_J2000_NOON_UTC;
                match offset_info_at_local(name_str, provisional_unix) {
                    Some(info) => {
                        if info.is_gap {
                            // Non-existent time (spring-forward gap) — shift forward
                            sec_utc += info.gap_size as i64; // shift local time into the valid post-gap period
                            sec_utc -= info.offset as i64; // apply the post-jump offset
                        } else {
                            sec_utc -= info.offset as i64;
                        }
                    }
                    None => {
                        return Err(an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "invalid iana: {}",
                            name_str
                        ));
                    }
                }
            }
        } else if let Some(Offset::Fixed(offset)) = self.offset {
            sec_utc -= offset as i64; // local civil time → true UTC instant
        }
        Ok(TimePoint::new(sec_utc, subsec, ClockType::UTC)
            .to_clock_type(clock_type.unwrap_or(self.clock_type)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeParts;
    use crate::error::DtErrKind;

    /// Small helper for readable JD assertions (matches how the rest of the crate uses `to_jd_tt()`).
    fn jd_tt(tp: &TimePoint) -> f64 {
        tp.to_jd_tt()
    }

    #[test]
    fn test_unix_epoch_1970() {
        let parsed = TimeParts::from_str("%s", "0", false, false, false).unwrap();
        let tp = parsed.to_time_point(Some(ClockType::TAI)).unwrap();

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
        let parsed = TimeParts::from_str("%s", "946728000", false, false, false).unwrap();
        let tp = parsed.to_time_point(Some(ClockType::TAI)).unwrap();

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
        let ymd = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap()
        .to_time_point(Some(ClockType::TAI))
        .unwrap();

        let ordinal = TimeParts::from_str(
            "%Y-%j %H:%M:%S.%.f",
            "2024-106 14:30:45.123456789",
            false,
            false,
            false,
        )
        .unwrap()
        .to_time_point(Some(ClockType::TAI))
        .unwrap();

        assert_eq!(jd_tt(&ymd), jd_tt(&ordinal));
        assert_eq!(ymd.to_jd_tt_exact(), ordinal.to_jd_tt_exact());
    }

    #[test]
    fn test_fractional_seconds_are_preserved() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
            false,
            false,
        )
        .unwrap();
        let tp = parsed.to_time_point(Some(ClockType::TAI)).unwrap();

        // 0.123456789 s = 123456789 × 10¹⁸ attoseconds
        let expected = 123_456_789u64 * 1_000_000_000;
        assert_eq!(tp.subsec, expected, "fractional seconds were not preserved");
    }

    #[test]
    fn test_jd_tt_fractional_seconds_preserved() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%9N",
            "2024-04-15 00:00:00.123456789",
            false,
            false,
            false,
        )
        .unwrap();
        let tp = parsed.to_time_point(Some(ClockType::TAI)).unwrap();

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
    fn test_incomplete_date_error() {
        // Default TimeParts has no year → early failure in to_time_point.
        let pd = TimeParts::default();
        let err = pd.to_time_point(Some(ClockType::TAI)).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::Incomplete));
    }

    #[test]
    fn test_day_of_year_out_of_range_non_leap_year() {
        // 2023 is not a leap year. We build a TimeParts manually because the parser
        // rejects day 366 (u8 limit in parse_u8_padded), so we never reach to_time_point
        // with a parser-constructed value. This test directly exercises the leap-year check.
        let mut pd = TimeParts::default();
        pd.year = Some(2023);
        pd.day_of_year = Some(366);
        let err = pd.to_time_point(Some(ClockType::TAI)).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::OutOfRange));
    }

    #[test]
    fn test_iso_week_out_of_range() {
        // Parser rejects week 54, so we build manually to hit the to_time_point check.
        let mut pd = TimeParts::default();
        pd.iso_week_year = Some(2024);
        pd.iso_week = Some(54);
        pd.weekday = Some(Weekday::Monday); // required for the ISO path
        let err = pd.to_time_point(Some(ClockType::TAI)).unwrap_err();
        assert!(matches!(err.kind().unwrap(), DtErrKind::OutOfRange));
    }

    #[test]
    fn test_pure_iso_week_date() {
        // Pure ISO week date (%G/%V/%u) is now fully supported in to_time_point
        // via the iso_week_year + iso_week + weekday path (no regular .year required).
        let parsed = TimeParts::from_str("%G-W%V-%u", "2024-W16-1", false, false, false).unwrap();
        let tp_iso = parsed.to_time_point(Some(ClockType::TAI)).unwrap();

        // 2024-W16-1 is Monday, April 15, 2024
        let ymd = TimeParts::from_str("%Y-%m-%d", "2024-04-15", false, false, false)
            .unwrap()
            .to_time_point(Some(ClockType::TAI))
            .unwrap();

        assert_eq!(jd_tt(&tp_iso), jd_tt(&ymd));
        assert_eq!(tp_iso.to_jd_tt_exact(), ymd.to_jd_tt_exact());
    }
}
