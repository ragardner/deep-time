use crate::{
    ATTOS_PER_NS, Dt, an_err,
    error::{DtErr, DtErrKind},
    {Meridiem, Offset, Parts, Weekday},
};
use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as ChronoTimeZone,
};

impl Parts {
    /// Converts [`Parts`] → [`chrono::NaiveDateTime`] (civil time, no TZ).
    pub fn to_chrono_naive_datetime(&self) -> Result<NaiveDateTime, DtErr> {
        let date = self.build_naive_date()?;
        let time = self.build_naive_time()?;

        Ok(date.and_time(time))
    }

    fn build_naive_date(&self) -> Result<NaiveDate, DtErr> {
        // YMD (highest priority, matches Jiff fast-path)
        if let (Some(y), Some(m), Some(d)) = (self.yr, self.mo, self.day) {
            let year_i32: i32 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidYear, "year: {}: {}", y, e))?;
            return NaiveDate::from_ymd_opt(year_i32, m as u32, d as u32)
                .ok_or_else(|| an_err!(DtErrKind::InvalidDate, "ymd: {}-{}-{}", year_i32, m, d));
        }

        // Ordinal date (%j)
        if let (Some(y), Some(doy)) = (self.yr, self.day_of_yr) {
            let year_i32: i32 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidYear, "year: {}: {}", y, e))?;
            return NaiveDate::from_yo_opt(year_i32, doy as u32)
                .ok_or_else(|| an_err!(DtErrKind::InvalidDate, "ydoy: {}-{}", y, doy));
        }

        // Small helper: JD → chrono NaiveDate
        let jd_to_naive_date = |jd: i64| -> Result<NaiveDate, DtErr> {
            let days_from_ce: i32 = (jd - 1721425)
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidDate, "jd: {}: {}", jd, e))?;
            NaiveDate::from_num_days_from_ce_opt(days_from_ce)
                .ok_or_else(|| an_err!(DtErrKind::InvalidDate, "jd: {}", days_from_ce))
        };

        // ISO week date (%G/%V + weekday)
        if let (Some(iso_y), Some(w)) = (self.iso_wk_yr, self.iso_wk) {
            let wd = self.wkday.unwrap_or(Weekday::Monday);
            let jd = Dt::iso_wk_to_jd(iso_y, w, wd);
            return jd_to_naive_date(jd);
        }

        // Sunday-based week number (%U)
        if let (Some(y), Some(w)) = (self.yr, self.wk_sun) {
            let wd = self.wkday.unwrap_or(Weekday::Sunday);
            let jd = Dt::wk_sun_to_jd(y, w, wd);
            return jd_to_naive_date(jd);
        }

        // Monday-based week number (%W)
        if let (Some(y), Some(w)) = (self.yr, self.wk_mon) {
            let wd = self.wkday.unwrap_or(Weekday::Monday);
            let jd = Dt::wk_mon_to_jd(y, w, wd);
            return jd_to_naive_date(jd);
        }

        Err(an_err!(DtErrKind::InvalidDate))
    }

    fn build_naive_time(&self) -> Result<NaiveTime, DtErr> {
        let mut hour = self.hr as u32;
        let minute = self.min as u32;
        let mut second = self.sec as u32;

        if let Some(meridiem) = self.meridiem {
            match (hour, meridiem) {
                (12, Meridiem::AM) => hour = 0,
                (12, Meridiem::PM) => {}
                (h, Meridiem::PM) if h < 12 => hour = h + 12,
                _ => {}
            }
        }

        let raw_ns_u64 = if self.attos != 0 {
            self.attos / ATTOS_PER_NS
        } else {
            0
        };

        let is_leap = second == 60;
        if !is_leap && raw_ns_u64 > 999_999_999 {
            return Err(an_err!(DtErrKind::InvalidFractional));
        }

        let mut subsec_nano: u32 = if raw_ns_u64 > 1_999_999_999 {
            1_999_999_999
        } else {
            raw_ns_u64 as u32
        };

        if is_leap {
            second = 59;
            subsec_nano = subsec_nano.saturating_add(1_000_000_000);
            if subsec_nano > 1_999_999_999 {
                subsec_nano = 1_999_999_999;
            }
        } else if second > 59 {
            return Err(an_err!(DtErrKind::SecondOutOfRange));
        }

        NaiveTime::from_hms_nano_opt(hour, minute, second, subsec_nano)
            .ok_or_else(|| an_err!(DtErrKind::InvalidTime))
    }

    /// Helper: resolve fixed offset / UTC only.
    fn to_chrono_offset(&self) -> Result<FixedOffset, DtErr> {
        match self.offset {
            Some(Offset::Fixed(secs)) => FixedOffset::east_opt(secs)
                .ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "{}", secs)),
            Some(Offset::None) | None => {
                FixedOffset::east_opt(0).ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "0"))
            }
        }
    }

    /// Converts [`Parts`] → [`chrono::DateTime<FixedOffset>`].
    /// - If this [`Parts`] has a unix timestamp then it is used
    ///   instead of anything else (timezones are ignored in this route).
    pub fn to_chrono_datetime(&self) -> Result<DateTime<FixedOffset>, DtErr> {
        // ============================================================
        // UNIX TIMESTAMP PATH
        // Always UTC. Completely ignores offset + iana_name.
        // ============================================================
        if self.timestamp.is_some() {
            let offset =
                FixedOffset::east_opt(0).ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "0"))?;
            let dt = self.to_dt()?.to_chrono_datetime_utc();
            return Ok(dt.with_timezone(&offset));
        }

        // ============================================================
        // CIVIL TIME PATH
        // ============================================================
        let naive = self.to_chrono_naive_datetime()?;

        if let Some(name) = &self.iana_name {
            let name_str = name.as_str();

            if !name_str.is_empty() {
                #[cfg(feature = "jiff-tz")]
                {
                    use jiff::{Timestamp, tz::TimeZone};

                    let tz = TimeZone::get(name_str)
                        .map_err(|e| an_err!(DtErrKind::InvalidTimezone, "{}", e))?;

                    let provisional_unix =
                        DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc)
                            .timestamp();

                    let civil = Timestamp::from_second(provisional_unix)
                        .map_err(|e| an_err!(DtErrKind::InvalidTimestamp, "{}", e))?
                        .to_zoned(jiff::tz::TimeZone::UTC)
                        .datetime();

                    // Use jiff's "compatible" strategy (gaps → later, folds → earlier)
                    let zoned = tz
                        .to_ambiguous_zoned(civil)
                        .compatible()
                        .map_err(|e| an_err!(DtErrKind::InvalidDate, "{}", e))?;

                    // Use the civil time that jiff actually resolved to
                    // (critical for spring-forward gaps)
                    let resolved = zoned.datetime();

                    let resolved_naive = NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(
                            resolved.year() as i32,
                            resolved.month() as u32,
                            resolved.day() as u32,
                        )
                        .ok_or_else(|| an_err!(DtErrKind::InvalidDate))?,
                        NaiveTime::from_hms_nano_opt(
                            resolved.hour() as u32,
                            resolved.minute() as u32,
                            resolved.second() as u32,
                            resolved.subsec_nanosecond() as u32,
                        )
                        .ok_or_else(|| an_err!(DtErrKind::InvalidTime))?,
                    );

                    let offset_secs = zoned.offset().seconds();
                    let offset = FixedOffset::east_opt(offset_secs)
                        .ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "{}", offset_secs))?;

                    return offset
                        .from_local_datetime(&resolved_naive)
                        .single()
                        .ok_or_else(|| an_err!(DtErrKind::InvalidTime, "fold/gap"));
                }

                #[cfg(not(feature = "jiff-tz"))]
                {
                    use crate::tz::UTC_ALIASES;

                    if UTC_ALIASES.contains(&name_str) {
                        // UTC alias — explicitly return +00:00
                        let offset = FixedOffset::east_opt(0)
                            .ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "0"))?;

                        return offset
                            .from_local_datetime(&naive)
                            .single()
                            .ok_or_else(|| an_err!(DtErrKind::InvalidTime, "fold/gap"));
                    } else {
                        return Err(an_err!(DtErrKind::MissingFeature));
                    }
                }
            }
        }

        // Fixed offset path
        let offset = self.to_chrono_offset()?;
        offset
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| an_err!(DtErrKind::InvalidOffset, "{:?}", offset))
    }

    /// Converts [`Parts`] → [`i64`].
    /// - If this [`Parts`] has a unix timestamp then it is used
    ///   instead of anything else (timezones are ignored).
    /// - Uses [`Parts::to_chrono_datetime`] internally.
    #[inline(always)]
    pub fn to_chrono_timestamp(&self) -> Result<i64, DtErr> {
        Ok(self.to_chrono_datetime()?.timestamp())
    }
}
