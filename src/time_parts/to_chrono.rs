use crate::{
    ATTOSEC_PER_NANOSEC, TimePoint,
    error::{DtErrKind, DtError},
    {Meridiem, TimeParts, TimeZone, Weekday},
};
use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as ChronoTimeZone,
};

impl TimeParts {
    /// Converts `TimeParts` → Chrono’s `NaiveDateTime` (civil time, no TZ).
    pub fn to_chrono_naive_datetime(&self) -> Result<NaiveDateTime, DtError> {
        let date = self.build_naive_date()?;
        let time = self.build_naive_time()?;

        Ok(date.and_time(time))
    }

    fn build_naive_date(&self) -> Result<NaiveDate, DtError> {
        let to_err = || DtError::new(DtErrKind::ChronoNaiveDate);

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

        // Small helper: JDN → chrono NaiveDate
        // (JDN 1721426 == proleptic Gregorian 0001-01-01; chrono counts days since then)
        let jdn_to_naive_date = |jdn: i64| -> Result<NaiveDate, DtError> {
            let days_from_ce: i32 = (jdn - 1721425).try_into().map_err(|_| to_err())?;
            NaiveDate::from_num_days_from_ce_opt(days_from_ce).ok_or_else(to_err)
        };

        // ISO week date (%G/%V + weekday)
        if let (Some(iso_y), Some(w)) = (self.iso_week_year, self.iso_week) {
            let wd = self.weekday.unwrap_or(Weekday::Monday); // ISO weeks start on Monday
            let jdn = TimePoint::ymd_to_jdn_from_iso_week(iso_y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        // Sunday-based week number (%U)
        if let (Some(y), Some(w)) = (self.year, self.week_sun) {
            let wd = self.weekday.unwrap_or(Weekday::Sunday); // %U weeks start on Sunday
            let jdn = TimePoint::ymd_to_jdn_from_week_sun(y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        // Monday-based week number (%W)
        if let (Some(y), Some(w)) = (self.year, self.week_mon) {
            let wd = self.weekday.unwrap_or(Weekday::Monday); // %W weeks start on Monday
            let jdn = TimePoint::ymd_to_jdn_from_week_mon(y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        // No supported date format
        Err(to_err())
    }

    fn build_naive_time(&self) -> Result<NaiveTime, DtError> {
        let to_err = || DtError::new(DtErrKind::ChronoNaiveTime);

        let mut hour = self.hour.unwrap_or(0) as u32;
        let minute = self.minute.unwrap_or(0) as u32;
        let mut second = self.second.unwrap_or(0) as u32;

        // AM/PM (12-hour → 24-hour) normalization
        if let Some(meridiem) = self.meridiem {
            match (hour, meridiem) {
                (12, Meridiem::AM) => hour = 0,
                (12, Meridiem::PM) => {}
                (h, Meridiem::PM) if h < 12 => hour = h + 12,
                _ => {}
            }
        }

        // Raw subsecond conversion (attoseconds → nanoseconds)
        let raw_ns_u64 = if let Some(attos) = self.attos {
            attos / ATTOSEC_PER_NANOSEC
        } else {
            0
        };

        // Determine leap-second case once (used for both clamping and transformation)
        let is_leap = second == 60 || self.is_leap_second;
        //   • Non-leap seconds → strictly < 1 s (error if exceeded)
        //   • Leap seconds      → allow up to ~2 s (chrono’s internal representation)
        if !is_leap && raw_ns_u64 > 999_999_999 {
            return Err(to_err());
        }

        let mut subsec_nano: u32 = if raw_ns_u64 > 1_999_999_999 {
            1_999_999_999
        } else {
            raw_ns_u64 as u32
        };

        // Chrono leap-second convention: second = 59 + nano >= 1_000_000_000
        if is_leap {
            second = 59;
            subsec_nano = subsec_nano.saturating_add(1_000_000_000);
            if subsec_nano > 1_999_999_999 {
                subsec_nano = 1_999_999_999;
            }
        } else if second > 59 {
            return Err(to_err());
        }

        NaiveTime::from_hms_nano_opt(hour, minute, second, subsec_nano).ok_or_else(to_err)
    }

    /// Helper: resolve the TimeParts TZ to a Chrono `FixedOffset`.
    /// IANA names are **not supported** in core chrono (they require the `chrono-tz` crate).
    /// TODO: Add chrono-tz feature?
    fn to_chrono_offset(&self) -> Result<FixedOffset, DtError> {
        let to_err = || DtError::new(DtErrKind::ChronoOffset);
        // IANA name present → explicit error (vanilla chrono cannot resolve it)
        if let Some(name) = &self.iana_name {
            let name_str = name.as_str().map_err(|_| to_err())?;
            if !name_str.is_empty() {
                return Err(to_err()); // "IANA timezones not supported in chrono feature"
            }
        }
        match self.tz {
            Some(TimeZone::Fixed(secs)) => {
                // east_opt already handles negative values correctly:
                // positive = east of UTC, negative = west of UTC.
                FixedOffset::east_opt(secs).ok_or_else(to_err)
            }
            Some(TimeZone::Utc) | Some(TimeZone::None) | None => {
                Ok(FixedOffset::east_opt(0).unwrap())
            }
        }
    }

    /// Converts `TimeParts` → absolute `DateTime<FixedOffset>`.
    ///
    /// If `unix_timestamp_seconds` is present, it is treated as the
    /// **absolute source of truth** for the instant (UTC seconds since the Unix epoch).
    ///
    /// ### Precedence rules (this is deliberate and final):
    /// - When `%s` (or equivalent) was parsed → **ignore all civil fields** (`year`, `month`,
    ///   `day`, `hour`, `minute`, `second`, `attos`, `weekday`, `day_of_year`, etc.).
    ///   The timestamp defines the exact physical moment.
    /// - The parsed timezone/offset (if any) is used **only** to choose the *displayed* civil
    ///   time on the returned `DateTime<FixedOffset>`. In other words, `%s` gives you the instant;
    ///   the TZ gives you the wall-clock representation of that instant.
    /// - If no `%s` is present, the normal civil path is taken: the date/time components are
    ///   interpreted as local time *in the parsed timezone*.
    pub fn to_chrono_datetime(&self) -> Result<DateTime<FixedOffset>, DtError> {
        let to_err = || DtError::new(DtErrKind::ChronoDateTime);

        let offset = self.to_chrono_offset()?;

        // Fast path: %s is gospel — absolute UTC instant (civil fields are ignored)
        if let Some(secs) = self.unix_timestamp_seconds {
            let subsec_nano = if let Some(attos) = self.attos {
                let ns_u64 = attos / ATTOSEC_PER_NANOSEC;
                // Unix/POSIX timestamps are continuous (no leap-second representation).
                // We clamp strictly to the range chrono::DateTime::from_timestamp accepts.
                if ns_u64 > 999_999_999 {
                    999_999_999
                } else {
                    ns_u64 as u32
                }
            } else {
                0
            };

            // Note: is_leap_second is ignored here (and cannot be set when unix_timestamp_seconds
            // is present, per TimeParts::finish()).
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

    /// Converts `TimeParts` → absolute Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    /// Fast path for unix seconds or full YMD; falls back to civil + TZ conversion.
    pub fn to_chrono_timestamp(&self) -> Result<i64, DtError> {
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
    use crate::TimeParts;

    #[test]
    fn test_to_chrono_naive_datetime_basic_ymd_hms() {
        let parsed =
            TimeParts::from_str("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false, false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_ordinal_date() {
        let parsed =
            TimeParts::from_str("%Y-%j %H:%M:%S", "2024-106 14:30:45", false, false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        let expected_date = NaiveDate::from_yo_opt(2024, 106).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_iso_week_date() {
        use chrono::Weekday as ChronoWeekday;

        let parsed =
            TimeParts::from_str("%G-W%V-%u %H:%M:%S", "2024-W16-2 14:30:45", false, false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // 2024-W16-2 = Tuesday 2024-04-16
        let expected_date = NaiveDate::from_isoywd_opt(2024, 16, ChronoWeekday::Tue).unwrap();
        let expected_time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_naive_datetime_fractional_seconds() {
        let parsed = TimeParts::from_str(
            "%Y-%m-%d %H:%M:%S.%N",
            "2024-04-15 14:30:45.123456789012345678901234567890",
            false,
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
        let parsed =
            TimeParts::from_str("%Y-%m-%d %H:%M:%S", "2024-04-15 23:59:60", false, false).unwrap();
        let ndt = parsed.to_chrono_naive_datetime().unwrap();

        // Chrono represents leap second as 23:59:59 + 1_000_000_000 ns
        let expected_date = NaiveDate::from_ymd_opt(2024, 4, 15).unwrap();
        let expected_time = NaiveTime::from_hms_nano_opt(23, 59, 59, 1_000_000_000).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(ndt, expected);
    }

    #[test]
    fn test_to_chrono_datetime_fixed_offset() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 14:30:45 -0400", false, false).unwrap();
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
        let parsed =
            TimeParts::from_str("%F %T %:z", "2024-04-15 14:30:45 -04:00", false, false).unwrap();
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
        let parsed = TimeParts::from_str("%s", "1713191445", false, false).unwrap();
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
        let parsed = TimeParts::from_str("%s.%N", "1713191445.123456789", false, false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected_utc = DateTime::from_timestamp(1713191445, 123_456_789).unwrap();
        let offset = FixedOffset::east_opt(0).unwrap();
        let expected = expected_utc.with_timezone(&offset);

        assert_eq!(dt, expected);
    }

    #[test]
    fn test_to_chrono_timestamp_basic() {
        let parsed =
            TimeParts::from_str("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false, false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_unix_direct() {
        let parsed = TimeParts::from_str("%s", "1713191445", false, false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_timestamp_with_offset() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 10:30:45 -0400", false, false).unwrap();
        let ts = parsed.to_chrono_timestamp().unwrap();
        // 10:30:45 EDT = 14:30:45 UTC → same as above
        assert_eq!(ts, 1713191445);
    }

    #[test]
    fn test_to_chrono_datetime_iana_name_errors() {
        let parsed = TimeParts::from_str(
            "%F %T %Q",
            "2024-04-15 10:30:00 America/New_York",
            false,
            false,
        )
        .unwrap();
        let err = parsed.to_chrono_datetime().unwrap_err();
        // IANA is rejected in to_chrono_offset (vanilla chrono cannot resolve it)
        assert!(matches!(err.kind, DtErrKind::ChronoOffset));
    }

    #[test]
    fn test_to_chrono_naive_datetime_incomplete_date_fails_in_finish_but_assembly_fails_here() {
        // Parser already rejects incomplete date in finish(), but we test the assembly path too
        let parsed = TimeParts::from_str("%H:%M:%S", "14:30:45", false, false);
        assert!(parsed.is_err()); // finish() already fails with IncompleteDate
    }

    #[test]
    fn test_to_chrono_datetime_utc_explicit() {
        let parsed =
            TimeParts::from_str("%F %T %z", "2024-04-15 14:30:45 +0000", false, false).unwrap();
        let dt = parsed.to_chrono_datetime().unwrap();

        let expected = DateTime::from_timestamp(1713191445, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        assert_eq!(dt, expected);
    }
}
