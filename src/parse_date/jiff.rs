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
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

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
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

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
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

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
