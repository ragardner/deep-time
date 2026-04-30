use {
    crate::{
        ATTOSEC_PER_NANOSEC, Meridiem, Offset, TimeParts, TimePoint, Weekday, an_err,
        error::{DtErrKind, DtError},
        tzdb::offset_at,
    },
    alloc::string::String,
    core::result::Result,
    jiff::{
        Timestamp, Zoned,
        civil::{Date, Time},
        fmt::strtime::{BrokenDownTime, Meridiem as JiffMeridiem},
        tz::{Offset as JiffOffset, TimeZone as JiffTimeZone},
    },
};

impl TimeParts {
    /// Converts `TimeParts` → Jiff’s `BrokenDownTime`.
    pub fn to_jiff_broken_down_time(&self) -> Result<BrokenDownTime, DtError> {
        let mut bdt = BrokenDownTime::default();

        // Date fields
        if let Some(year) = self.year {
            let y: i16 = year
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "year: {}: {}", year, e))?;
            bdt.set_year(Some(y))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "year: {}: {}", y, e))?;
        }
        if let Some(m) = self.month {
            bdt.set_month(Some(m as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "month: {}: {}", m, e))?;
        }
        if let Some(d) = self.day {
            bdt.set_day(Some(d as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "day: {}: {}", d, e))?;
        }

        // Week / day-of-year fields
        if let Some(doy) = self.day_of_year {
            bdt.set_day_of_year(Some(doy as i16))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "doy: {}: {}", doy, e))?;
        }
        if let Some(y) = self.iso_week_year {
            let y: i16 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "iso wk yr: {}: {}", y, e))?;
            bdt.set_iso_week_year(Some(y))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "iso wk yr: {}: {}", y, e))?;
        }
        if let Some(w) = self.iso_week {
            bdt.set_iso_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "iso wk: {}: {}", w, e))?;
        }
        if let Some(w) = self.week_sun {
            bdt.set_sunday_based_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "sun based wk: {}: {}", w, e))?;
        }
        if let Some(w) = self.week_mon {
            bdt.set_monday_based_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "mon based wk: {}: {}", w, e))?;
        }

        // Time of day
        if let Some(h) = self.hour {
            bdt.set_hour(Some(h as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "hour: {}: {}", h, e))?;
        }
        if let Some(m) = self.minute {
            bdt.set_minute(Some(m as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "minute: {}: {}", m, e))?;
        }
        if let Some(s) = self.second {
            let non_ls_s = if s == 60 { 59 } else { s };
            bdt.set_second(Some(non_ls_s as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "second: {}: {}", non_ls_s, e))?;
        }

        // Subsecond precision (attoseconds → nanoseconds)
        if let Some(attos) = self.attos {
            let ns_u64 = attos / ATTOSEC_PER_NANOSEC;
            let ns: i32 = if ns_u64 >= 1_000_000_000 {
                999_999_999
            } else {
                ns_u64 as i32
            };
            bdt.set_subsec_nanosecond(Some(ns))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "ns: {}: {}", ns, e))?;
        }

        // Infallible setters
        if let Some(wd) = self.weekday {
            let jwd = match wd {
                Weekday::Sunday => jiff::civil::Weekday::Sunday,
                Weekday::Monday => jiff::civil::Weekday::Monday,
                Weekday::Tuesday => jiff::civil::Weekday::Tuesday,
                Weekday::Wednesday => jiff::civil::Weekday::Wednesday,
                Weekday::Thursday => jiff::civil::Weekday::Thursday,
                Weekday::Friday => jiff::civil::Weekday::Friday,
                Weekday::Saturday => jiff::civil::Weekday::Saturday,
            };
            bdt.set_weekday(Some(jwd));
        }
        if let Some(mer) = self.meridiem {
            let jmer = match mer {
                Meridiem::AM => JiffMeridiem::AM,
                Meridiem::PM => JiffMeridiem::PM,
            };
            bdt.set_meridiem(Some(jmer));
        }

        // Explicit Unix timestamp (highest priority)
        if let Some(secs) = self.unix_timestamp_seconds {
            let ts = Timestamp::from_second(secs)
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "timestamp: {}: {}", secs, e))?;
            bdt.set_timestamp(Some(ts));
        }

        // Prefer IANA name if present; otherwise fall back to the custom TimeZone enum.
        if let Some(name) = &self.iana_name {
            match name.as_str() {
                Ok(s) if !s.is_empty() => {
                    let _ = bdt.set_iana_time_zone(Some(String::from(s)));
                }
                Ok(_) => {} // empty name — do nothing
                Err(e) => {
                    return Err(an_err!(
                        DtErrKind::InvalidBytes,
                        "invalid iana ascii: {:?}: {}",
                        name,
                        e
                    ));
                }
            }
        } else if let Some(Offset::Fixed(secs)) = self.offset {
            if let Ok(jiff_offset) = JiffOffset::from_seconds(secs) {
                let _ = bdt.set_offset(Some(jiff_offset));
            } else {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "offset secs: {}",
                    secs
                ));
            }
        } else {
            // Utc / None → treat as UTC
            let _ = bdt.set_offset(Some(JiffOffset::UTC));
        }

        Ok(bdt)
    }

    pub fn to_jiff_zoned(&self) -> Result<Zoned, DtError> {
        let bdt = self.to_jiff_broken_down_time()?;
        if let Ok(zoned) = bdt.to_zoned() {
            return Ok(zoned);
        }
        if let Ok(ts) = bdt.to_timestamp() {
            if let Ok(zoned) = ts.in_tz("UTC") {
                return Ok(zoned);
            }
        }
        if let Ok(dt) = bdt.to_datetime() {
            if let Ok(zoned) = dt.in_tz("UTC") {
                return Ok(zoned);
            }
        }
        if let Ok(date) = bdt.to_date() {
            if let Ok(dt) = date.at(0, 0, 0, 0).in_tz("UTC") {
                return Ok(dt);
            }
        }
        Err(an_err!(
            DtErrKind::InvalidInput,
            "could not convert to jiff zoned"
        ))
    }

    /// Converts `TimeParts` → absolute `Timestamp` on the SI scale.
    pub fn to_jiff_timestamp(&self) -> Result<Timestamp, DtError> {
        if let Some(secs) = self.unix_timestamp_seconds {
            return Timestamp::from_second(secs)
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "timestamp: {}: {}", secs, e));
        }

        if let (Some(year), Some(month), Some(day)) = (self.year, self.month, self.day) {
            let year_i16: i16 = year
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "year: {}: {}", year, e))?;

            let date = Date::new(year_i16, month as i8, day as i8).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidInput,
                    "ymd: {} {} {}: {}",
                    year_i16,
                    month,
                    day,
                    e
                )
            })?;

            let hour = self.hour.unwrap_or(0) as i8;
            let minute = self.minute.unwrap_or(0) as i8;
            let second = self.second.unwrap_or(0) as i8;

            let subsec_nanosecond: i32 = if let Some(attos) = self.attos {
                let ns_u64 = attos / ATTOSEC_PER_NANOSEC;
                if ns_u64 > 999_999_999 {
                    999_999_999
                } else {
                    ns_u64 as i32
                }
            } else {
                0
            };

            let time = Time::new(hour, minute, second, subsec_nanosecond).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidInput,
                    "hms: {} {} {} {}: {}",
                    hour,
                    minute,
                    second,
                    subsec_nanosecond,
                    e
                )
            })?;

            let civil_dt = date.to_datetime(time);

            let tz = self.to_jiff_time_zone()?;
            let zoned = tz
                .to_zoned(civil_dt)
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "civil to zoned: {}", e))?;
            return Ok(zoned.timestamp());
        }

        // FALLBACK: ordinal date, ISO week date, partial fields, etc.
        let bdt = self.clone().to_jiff_broken_down_time()?;

        bdt.to_timestamp()
            .map_err(|e| an_err!(DtErrKind::InvalidInput, "to timestamp: {}", e))
    }

    // Helper used by to_timestamp
    fn to_jiff_time_zone(&self) -> core::result::Result<JiffTimeZone, DtError> {
        // IANA name takes precedence — use OUR own tz database only
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
                let probe_ts = if let Some(ts) = self.unix_timestamp_seconds {
                    ts
                } else if let (Some(y), Some(m), Some(d)) = (self.year, self.month, self.day) {
                    TimePoint::ymdhms_to_unix_timestamp(
                        y,
                        m,
                        d,
                        self.hour.unwrap_or(0),
                        self.minute.unwrap_or(0),
                        self.second.unwrap_or(0),
                    )
                } else {
                    0
                };

                if let Some(offset) = offset_at(name_str, probe_ts) {
                    let jiff_offset = JiffOffset::from_seconds(offset).map_err(|e| {
                        an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "offset secs: {}: {}",
                            offset,
                            e
                        )
                    })?;
                    return Ok(JiffTimeZone::fixed(jiff_offset));
                } else {
                    return Err(an_err!(
                        DtErrKind::InvalidTimezoneOffset,
                        "iana: {}",
                        name_str
                    ));
                }
            }
        }

        // Fallback to the custom TimeZone enum
        match self.offset {
            Some(Offset::Fixed(secs)) => {
                let offset = JiffOffset::from_seconds(secs).map_err(|e| {
                    an_err!(
                        DtErrKind::InvalidTimezoneOffset,
                        "offset secs: {}: {}",
                        secs,
                        e
                    )
                })?;
                Ok(JiffTimeZone::fixed(offset))
            }
            Some(Offset::Utc) | Some(Offset::None) | None => Ok(JiffTimeZone::UTC),
        }
    }
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use crate::TimeParts;

    use super::*;
    use jiff::{SignedDuration, Timestamp};

    fn parse_ts(fmt: &str, input: &str, strict: bool) -> Result<Timestamp, DtError> {
        let parsed = TimeParts::from_str(fmt, input, strict, false, false)?;
        parsed.to_jiff_timestamp()
    }

    #[test]
    fn test_basic_ymd_hms_utc() {
        let ts = parse_ts("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191445).unwrap());
    }

    #[test]
    fn test_unix_timestamp_direct() {
        let ts = parse_ts("%s", "1713191445", false).unwrap();
        assert_eq!(ts.as_second(), 1713191445);
    }

    #[test]
    fn test_fixed_offset() {
        let ts = parse_ts("%F %T%z", "2024-04-15 10:30:00-0400", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_fixed_offset_with_colons() {
        let ts = parse_ts("%F %T%:z", "2024-04-15 10:30:00-04:00", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_iana_timezone() {
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
            false,
        )
        .unwrap();
        assert!(parsed.iana_name.is_some());
        let ts = parsed.to_jiff_timestamp().unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191400).unwrap());
    }

    #[test]
    fn test_fractional_seconds_various_widths() {
        let base = Timestamp::from_second(1713191445).unwrap();

        let ts = parse_ts(
            "%Y-%m-%d %H:%M:%S%.f",
            "2024-04-15 14:30:45.123456789",
            false,
        )
        .unwrap();
        assert_eq!(
            ts,
            base.checked_add(SignedDuration::from_nanos(123_456_789))
                .unwrap()
        );

        let ts2 = parse_ts("%Y-%m-%d %H:%M:%S%3N", "2024-04-15 14:30:45.123", false).unwrap();
        assert_eq!(
            ts2,
            base.checked_add(SignedDuration::from_millis(123)).unwrap()
        );
    }

    #[test]
    fn test_ordinal_date_fallback_path() {
        let ts = parse_ts("%Y-%j %H:%M:%S", "2024-106 14:30:45", false).unwrap();
        assert_eq!(ts, Timestamp::from_second(1713191445).unwrap());
    }

    #[test]
    fn test_iso_week_date_fallback_path() {
        let ts = parse_ts("%G-W%V-%u %H:%M:%S", "2024-W16-2 14:30:45", false).unwrap();
        // 2024-W16-2 is Tuesday April 16 (not April 15)
        assert_eq!(ts, Timestamp::from_second(1713277845).unwrap());
    }

    #[test]
    fn test_shortcut_formats() {
        let ts_f = parse_ts("%F %T", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(ts_f, Timestamp::from_second(1713191445).unwrap());

        let ts_d = parse_ts("%D", "04/15/24", false).unwrap();
        // %D is date-only → defaults to 00:00:00 UTC
        assert_eq!(ts_d, Timestamp::from_second(1713139200).unwrap());
    }

    #[test]
    fn test_broken_down_time_assembly() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S %z",
            "2024-04-15 14:30:45 +0200",
            false,
            false,
            false,
        )
        .unwrap();
        let bdt = parsed.to_jiff_broken_down_time().unwrap();

        assert_eq!(bdt.year(), Some(2024));
        assert_eq!(bdt.month(), Some(4));
        assert_eq!(bdt.day(), Some(15));
        assert_eq!(bdt.hour(), Some(14));
        assert_eq!(bdt.minute(), Some(30));
        assert_eq!(bdt.second(), Some(45));
        assert_eq!(
            bdt.offset(),
            Some(jiff::tz::Offset::from_seconds(7200).unwrap())
        );
    }
}
