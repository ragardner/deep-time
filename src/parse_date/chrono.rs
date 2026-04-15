#[cfg(feature = "chrono")]
use crate::{
    MICROQUECTOS_PER_NANOSEC,
    parser::{Error, ParseErr, ParsedDate, TimeZone, Weekday},
};
#[cfg(feature = "chrono")]
use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as ChronoTimeZone,
    Weekday as ChronoWeekday,
};

#[cfg(feature = "chrono")]
impl ParsedDate {
    /// Converts `ParsedDate` → Chrono’s `NaiveDateTime` (civil time, no TZ).
    pub fn to_chrono_naive_datetime(&self) -> Result<NaiveDateTime, Error> {
        let date = self.build_naive_date()?;
        let time = self.build_naive_time()?;

        Ok(date.and_time(time))
    }

    fn build_naive_date(&self) -> Result<NaiveDate, Error> {
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

        let year_i32: i32 = self
            .year
            .ok_or_else(to_err)?
            .try_into()
            .map_err(|_| to_err())?;

        // YMD (highest priority, matches Jiff fast-path)
        if let (Some(m), Some(d)) = (self.month, self.day) {
            return NaiveDate::from_ymd_opt(year_i32, m as u32, d as u32).ok_or_else(to_err);
        }

        // Ordinal date (%j)
        if let Some(doy) = self.day_of_year {
            return NaiveDate::from_yo_opt(year_i32, doy as u32).ok_or_else(to_err);
        }

        // ISO week date (%G/%V + weekday)
        if let (Some(iso_y), Some(w)) = (self.iso_week_year, self.iso_week) {
            let iso_y_i32: i32 = iso_y.try_into().map_err(|_| to_err())?;
            let wd = self.weekday.unwrap_or(Weekday::Monday); // fallback reasonable for ISO
            let chrono_wd = match wd {
                Weekday::Monday => ChronoWeekday::Mon,
                Weekday::Tuesday => ChronoWeekday::Tue,
                Weekday::Wednesday => ChronoWeekday::Wed,
                Weekday::Thursday => ChronoWeekday::Thu,
                Weekday::Friday => ChronoWeekday::Fri,
                Weekday::Saturday => ChronoWeekday::Sat,
                Weekday::Sunday => ChronoWeekday::Sun,
            };
            return NaiveDate::from_isoywd_opt(iso_y_i32, w as u32, chrono_wd).ok_or_else(to_err);
        }

        // Sunday/Monday week numbers (%U/%W) not directly supported by chrono constructors.
        // (They would require extra calendar math – left as future extension or use Jiff.)
        Err(to_err())
    }

    fn build_naive_time(&self) -> Result<NaiveTime, Error> {
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

        let hour = self.hour.unwrap_or(0) as u32;
        let minute = self.minute.unwrap_or(0) as u32;
        let mut second = self.second.unwrap_or(0) as u32;

        let mut subsec_nano: u32 = if let Some(mqs) = self.microquectos {
            let ns_u128 = mqs / MICROQUECTOS_PER_NANOSEC;
            if ns_u128 > 1_999_999_999 {
                1_999_999_999
            } else {
                ns_u128 as u32
            }
        } else {
            0
        };

        // Chrono leap-second convention: second = 59 + nano >= 1_000_000_000
        if second == 60 || self.is_leap_second {
            second = 59;
            subsec_nano += 1_000_000_000;
            if subsec_nano > 1_999_999_999 {
                subsec_nano = 1_999_999_999;
            }
        } else if second > 59 {
            return Err(to_err());
        }

        NaiveTime::from_hms_nano_opt(hour, minute, second, subsec_nano).ok_or_else(to_err)
    }

    /// Helper: resolve the ParsedDate TZ to a Chrono `FixedOffset`.
    /// IANA names are **not supported** in core chrono (they require the `chrono-tz` crate).
    fn to_chrono_offset(&self) -> Result<FixedOffset, Error> {
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

        // IANA name present → explicit error (vanilla chrono cannot resolve it)
        if let Some(name_bytes) = &self.iana_name {
            let len = name_bytes.iter().position(|&b| b == 0).unwrap_or(48);
            if len > 0 {
                return Err(to_err()); // "IANA timezones not supported in chrono feature"
            }
        }

        match self.tz {
            Some(TimeZone::Fixed(secs)) => {
                if secs >= 0 {
                    FixedOffset::east_opt(secs).ok_or_else(to_err)
                } else {
                    FixedOffset::west_opt(secs.wrapping_neg()).ok_or_else(to_err)
                }
            }
            Some(TimeZone::Utc) | Some(TimeZone::None) | None => {
                Ok(FixedOffset::east_opt(0).unwrap())
            }
        }
    }

    /// Converts `ParsedDate` → absolute `DateTime<FixedOffset>` (civil time interpreted in the parsed TZ).
    pub fn to_chrono_datetime(&self) -> Result<DateTime<FixedOffset>, Error> {
        let to_err = || Error::simple(ParseErr::AssemblyFailed);

        let offset = self.to_chrono_offset()?;

        // Fast path: explicit Unix timestamp (absolute, highest priority)
        if let Some(secs) = self.unix_timestamp_seconds {
            let subsec_nano = if let Some(mqs) = self.microquectos {
                let ns_u128 = mqs / MICROQUECTOS_PER_NANOSEC;
                if ns_u128 > 999_999_999 {
                    999_999_999
                } else {
                    ns_u128 as u32
                }
            } else {
                0
            };
            let utc_dt = DateTime::from_timestamp(secs, subsec_nano).ok_or_else(to_err)?;
            return Ok(utc_dt.with_timezone(&offset));
        }

        // Normal path: build civil datetime in the chosen TZ
        let naive = self.to_chrono_naive_datetime()?;
        offset
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(to_err)
    }

    /// Converts `ParsedDate` → absolute Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    /// Fast path for unix seconds or full YMD; falls back to civil + TZ conversion.
    pub fn to_chrono_timestamp(&self) -> Result<i64, Error> {
        if let Some(secs) = self.unix_timestamp_seconds {
            return Ok(secs);
        }

        let dt = self.to_chrono_datetime()?;
        Ok(dt.timestamp())
    }
}
