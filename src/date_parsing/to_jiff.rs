#[cfg(feature = "jiff")]
use crate::{
    MICROQUECTOS_PER_NANOSEC,
    parser::{Error, Meridiem, ParseErr, ParsedDate, TimeZone, Weekday},
};
#[cfg(feature = "jiff")]
use alloc::string::String;
#[cfg(feature = "jiff")]
use core::result::Result;
#[cfg(feature = "jiff")]
use jiff::{
    Timestamp,
    civil::{Date, Time},
    fmt::strtime::{BrokenDownTime, Meridiem as JiffMeridiem},
    tz::{Offset, TimeZone as JiffTimeZone},
};

#[cfg(feature = "jiff")]
impl ParsedDate {
    /// Converts `ParsedDate` → Jiff’s `BrokenDownTime`.
    pub fn to_jiff_broken_down_time(&self) -> Result<BrokenDownTime, Error> {
        let to_err = || Error::simple(ParseErr::JiffBrokenDownTime);

        let mut bdt = BrokenDownTime::default();

        // === Date fields ===
        if let Some(year) = self.year {
            let y: i16 = year.try_into().map_err(|_| to_err())?;
            bdt.set_year(Some(y)).map_err(|_| to_err())?;
        }
        if let Some(m) = self.month {
            bdt.set_month(Some(m as i8)).map_err(|_| to_err())?;
        }
        if let Some(d) = self.day {
            bdt.set_day(Some(d as i8)).map_err(|_| to_err())?;
        }

        // === Week / day-of-year fields ===
        if let Some(doy) = self.day_of_year {
            bdt.set_day_of_year(Some(doy as i16))
                .map_err(|_| to_err())?;
        }
        if let Some(y) = self.iso_week_year {
            let y: i16 = y.try_into().map_err(|_| to_err())?;
            bdt.set_iso_week_year(Some(y)).map_err(|_| to_err())?;
        }
        if let Some(w) = self.iso_week {
            bdt.set_iso_week(Some(w as i8)).map_err(|_| to_err())?;
        }
        if let Some(w) = self.week_sun {
            bdt.set_sunday_based_week(Some(w as i8))
                .map_err(|_| to_err())?;
        }
        if let Some(w) = self.week_mon {
            bdt.set_monday_based_week(Some(w as i8))
                .map_err(|_| to_err())?;
        }

        // === Time of day ===
        if let Some(h) = self.hour {
            bdt.set_hour(Some(h as i8)).map_err(|_| to_err())?;
        }
        if let Some(m) = self.minute {
            bdt.set_minute(Some(m as i8)).map_err(|_| to_err())?;
        }
        if let Some(s) = self.second {
            if s == 60 {
                return Err(to_err()); // Jiff does not support leap seconds
            }
            bdt.set_second(Some(s as i8)).map_err(|_| to_err())?;
        }

        // === Subsecond precision (microquectos → nanoseconds) ===
        if let Some(mqs) = self.microquectos {
            let ns_u128 = mqs / MICROQUECTOS_PER_NANOSEC;
            let ns: i32 = if ns_u128 >= 1_000_000_000 {
                999_999_999
            } else {
                ns_u128 as i32
            };
            bdt.set_subsec_nanosecond(Some(ns)).map_err(|_| to_err())?;
        }

        // === Infallible setters ===
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

        // === Explicit Unix timestamp (highest priority) ===
        if let Some(secs) = self.unix_timestamp_seconds {
            let ts = Timestamp::from_second(secs).map_err(|_| to_err())?;
            bdt.set_timestamp(Some(ts));
        }

        // === Time zone handling (new ParsedDate fields) ===
        // Prefer IANA name if present; otherwise fall back to the custom TimeZone enum.
        if let Some(name_bytes) = &self.iana_name {
            let len = name_bytes.iter().position(|&b| b == 0).unwrap_or(48);
            if len > 0 {
                if let Ok(name) = core::str::from_utf8(&name_bytes[0..len]) {
                    let _ = bdt.set_iana_time_zone(Some(String::from(name)));
                } else {
                    return Err(to_err()); // invalid UTF-8 IANA name
                }
            }
        } else if let Some(TimeZone::Fixed(secs)) = self.tz {
            if let Ok(offset) = Offset::from_seconds(secs) {
                let _ = bdt.set_offset(Some(offset));
            } else {
                return Err(to_err()); // invalid fixed offset
            }
        } else {
            // Utc / None → treat as UTC
            let _ = bdt.set_offset(Some(Offset::UTC));
        }

        Ok(bdt)
    }

    /// Converts `ParsedDate` → absolute `Timestamp` on the SI scale.
    ///
    /// Fast path for the common cases (unix seconds or full YMD date).
    /// Falls back to `BrokenDownTime` for everything else (ordinal date, ISO week, etc.).
    pub fn to_jiff_timestamp(&self) -> Result<Timestamp, Error> {
        let to_err = || Error::simple(ParseErr::JiffTimestamp);

        if let Some(secs) = self.unix_timestamp_seconds {
            return Timestamp::from_second(secs).map_err(|_| to_err());
        }

        if let (Some(year), Some(month), Some(day)) = (self.year, self.month, self.day) {
            let year_i16: i16 = year.try_into().map_err(|_| to_err())?;

            let date = Date::new(year_i16, month as i8, day as i8).map_err(|_| to_err())?;

            let hour = self.hour.unwrap_or(0) as i8;
            let minute = self.minute.unwrap_or(0) as i8;
            let second = self.second.unwrap_or(0) as i8;

            let subsec_nanosecond: i32 = if let Some(mqs) = self.microquectos {
                let ns_u128 = mqs / MICROQUECTOS_PER_NANOSEC;
                if ns_u128 > 999_999_999 {
                    999_999_999
                } else {
                    ns_u128 as i32
                }
            } else {
                0
            };

            let time = Time::new(hour, minute, second, subsec_nanosecond).map_err(|_| to_err())?;

            let civil_dt = date.to_datetime(time);

            let tz = self.to_jiff_time_zone()?;
            let zoned = tz.to_zoned(civil_dt).map_err(|_| to_err())?;
            return Ok(zoned.timestamp());
        }

        // FALLBACK: ordinal date, ISO week date, partial fields, etc.
        let bdt = self.clone().to_jiff_broken_down_time()?;

        bdt.to_timestamp().map_err(|_| to_err())
    }

    // Helper used by to_timestamp
    #[inline]
    fn to_jiff_time_zone(&self) -> core::result::Result<JiffTimeZone, Error> {
        let to_err = || Error::simple(ParseErr::JiffTimeZone);

        // IANA name takes precedence
        if let Some(name_bytes) = &self.iana_name {
            let len = name_bytes.iter().position(|&b| b == 0).unwrap_or(48);
            if len > 0 {
                let name = core::str::from_utf8(&name_bytes[0..len]).map_err(|_| to_err())?;
                return JiffTimeZone::get(name).map_err(|_| to_err());
            }
        }

        // Fallback to the custom TimeZone enum
        match self.tz {
            Some(TimeZone::Fixed(secs)) => {
                let offset = Offset::from_seconds(secs).map_err(|_| to_err())?;
                Ok(JiffTimeZone::fixed(offset))
            }
            Some(TimeZone::Utc) | Some(TimeZone::None) | None => Ok(JiffTimeZone::UTC),
        }
    }
}

#[cfg(all(test, feature = "jiff"))]
mod tests {
    use crate::parser::parse_date;

    use super::*;
    use jiff::{SignedDuration, Timestamp};

    fn parse_ts(fmt: &str, input: &str, strict: bool) -> Result<Timestamp, Error> {
        let parsed = parse_date(fmt, input, strict)?;
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
        let parsed = parse_date("%F %T %Q", "2024-04-15 10:30:00 America/New_York", false).unwrap();
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
    fn test_leap_second_rejected_by_jiff() {
        let parsed = parse_date("%Y-%m-%d %H:%M:%S", "2024-04-15 23:59:60", false).unwrap();
        assert!(parsed.is_leap_second);

        assert!(parsed.to_jiff_broken_down_time().is_err());
        assert!(parsed.to_jiff_timestamp().is_err());
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
        let parsed =
            parse_date("%Y-%m-%d %H:%M:%S %z", "2024-04-15 14:30:45 +0200", false).unwrap();
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
