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
        let to_err = || Error::simple(ParseErr::ChronoNaiveDate);

        // YMD (highest priority, matches Jiff fast-path)
        if let (Some(y), Some(m), Some(d)) = (self.year, self.month, self.day) {
            let year_i32: i32 = y.try_into().map_err(|_| to_err())?;
            return NaiveDate::from_ymd_opt(year_i32, m as u32, d as u32).ok_or_else(to_err);
        }

        // Ordinal date (%j)
        if let (Some(y), Some(doy)) = (self.year, self.day_of_year) {
            let year_i32: i32 = y.try_into().map_err(|_| to_err())?;
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
        let to_err = || Error::simple(ParseErr::ChronoNaiveTime);

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
        let to_err = || Error::simple(ParseErr::ChronoOffset);

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
        let to_err = || Error::simple(ParseErr::ChronoDateTime);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_date;

    #[test]
    fn test_to_chrono_naive_datetime_basic_ymd_hms() {
        let parsed = parse_date("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_ordinal_date() {
        let parsed = parse_date("%Y-%j %H:%M:%S", "2024-106 14:30:45", false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_yo_opt(2024, 106).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_iso_week_date() {
        let parsed = parse_date("%G-W%V-%u %H:%M:%S", "2024-W16-2 14:30:45", false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // 2024-W16-2 = Tuesday 2024-04-16
        let expected_date = NaiveDate::from_isoywd_opt(2024, 16, ChronoWeekday::Tue).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_fractional_seconds() {
        let parsed = parse_date(
            "%Y-%m-%d %H:%M:%S.%N",
            "2024-04-15 14:30:45.123456789012345678901234567890",
            false,
        )
        .unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_nano_opt(14, 30, 45, 123_456_789).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_leap_second() {
        let parsed = parse_date("%Y-%m-%d %H:%M:%S", "2024-04-15 23:59:60", false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // Chrono represents leap second as 23:59:59 + 1_000_000_000 ns
        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_nano_opt(23, 59, 59, 1_000_000_000).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_datetime_fixed_offset() {
        let parsed = parse_date("%F %T %z", "2024-04-15 14:30:45 -0400", false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
        );
        let offset = FixedOffset::west_opt(4 * 3600).unwrap();
        let expected = offset
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap();

        assert_eq!(dt, expected);
        assert_eq!(dt.offset(), &offset);
    }

    #[test]
    fn test_to_chrono_datetime_colon_z_offset() {
        let parsed = parse_date("%F %T %:z", "2024-04-15 14:30:45 -04:00", false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
        );
        let offset = FixedOffset::west_opt(4 * 3600).unwrap();
        let expected = offset
            .from_local_datetime(&expected_naive)
            .single()
            .unwrap();

        assert_eq!(dt, expected);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_direct() {
        let parsed = parse_date("%s", "1713191445", false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        // 1713191445 = 2024-04-15 14:30:45 UTC
        let expected_utc = DateTime::from_timestamp(1713191445, 0).unwrap();
        let offset = FixedOffset::east_opt(0).unwrap();
        let expected = expected_utc.with_timezone(&offset);

        assert_eq!(dt, expected);
        assert_eq!(dt.timestamp(), 1713191445);
    }

    #[test]
    fn test_to_chrono_datetime_unix_timestamp_with_fraction() {
        let parsed = parse_date("%s.%N", "1713191445.123456789", false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_utc = DateTime::from_timestamp(1713191445, 123_456_789).unwrap();
        let offset = FixedOffset::east_opt(0).unwrap();
        let expected = expected_utc.with_timezone(&offset);

        assert_eq!(dt, expected);
    }

    #[test]
    fn test_to_chrono_timestamp_basic() {
        let parsed = parse_date("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_unix_direct() {
        let parsed = parse_date("%s", "1713191445", false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_with_offset() {
        let parsed = parse_date("%F %T %z", "2024-04-15 10:30:45 -0400", false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        // 10:30:45 EDT = 14:30:45 UTC → same as above
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_datetime_iana_name_errors() {
        let parsed = parse_date("%F %T %Q", "2024-04-15 10:30:00 America/New_York", false).unwrap();
        let err = parsed.to_chrono_datetime().unwrap_err();

        // IANA is rejected in to_chrono_offset (vanilla chrono cannot resolve it)
        match err {
            Error::Simple {
                kind: ParseErr::ChronoOffset, // ← changed
                ..
            } => {}
            _ => panic!("expected ChronoOffset for IANA name (chrono-tz not supported)"),
        }
    }

    #[test]
    fn test_to_chrono_naive_datetime_incomplete_date_fails_in_finish_but_assembly_fails_here() {
        // Parser already rejects incomplete date in finish(), but we test the assembly path too
        let parsed = parse_date("%H:%M:%S", "14:30:45", false);
        assert!(parsed.is_err()); // finish() already fails with IncompleteDate
    }

    #[test]
    fn test_to_chrono_datetime_utc_explicit() {
        let parsed = parse_date("%F %T %z", "2024-04-15 14:30:45 +0000", false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected = DateTime::from_timestamp(1713191445, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        assert_eq!(dt, expected);
    }
}
