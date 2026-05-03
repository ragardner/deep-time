use crate::{
    ATTOS_PER_NS, TimePoint, an_err,
    error::{DtErr, DtErrKind},
    {Meridiem, Offset, TimeParts, Weekday},
};
use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone as ChronoTimeZone,
};

#[cfg(feature = "chrono-tz")]
use chrono_tz::Tz;

#[cfg(not(feature = "chrono-tz"))]
use crate::tzdb::offset_info_at_local;

impl TimeParts {
    /// Converts `TimeParts` → Chrono’s `NaiveDateTime` (civil time, no TZ).
    pub fn to_chrono_naive_datetime(&self) -> Result<NaiveDateTime, DtErr> {
        let date = self.build_naive_date()?;
        let time = self.build_naive_time()?;

        Ok(date.and_time(time))
    }

    fn build_naive_date(&self) -> Result<NaiveDate, DtErr> {
        // YMD (highest priority, matches Jiff fast-path)
        if let (Some(y), Some(m), Some(d)) = (self.year, self.month, self.day) {
            let year_i32: i32 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidNumber, "year: {}: {}", y, e))?;
            return NaiveDate::from_ymd_opt(year_i32, m as u32, d as u32)
                .ok_or_else(|| an_err!(DtErrKind::InvalidInput, "ymd: {}-{}-{}", year_i32, m, d));
        }

        // Ordinal date (%j)
        if let (Some(y), Some(doy)) = (self.year, self.day_of_year) {
            let year_i32: i32 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidNumber, "year: {}: {}", y, e))?;
            return NaiveDate::from_yo_opt(year_i32, doy as u32)
                .ok_or_else(|| an_err!(DtErrKind::InvalidInput, "ydoy: {}-{}", y, doy));
        }

        // Small helper: JDN → chrono NaiveDate
        let jdn_to_naive_date = |jdn: i64| -> Result<NaiveDate, DtErr> {
            let days_from_ce: i32 = (jdn - 1721425)
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "jdn: {}: {}", jdn, e))?;
            NaiveDate::from_num_days_from_ce_opt(days_from_ce)
                .ok_or_else(|| an_err!(DtErrKind::InvalidInput, "days_from_ce: {}", days_from_ce))
        };

        // ISO week date (%G/%V + weekday)
        if let (Some(iso_y), Some(w)) = (self.iso_week_year, self.iso_week) {
            let wd = self.weekday.unwrap_or(Weekday::Monday);
            let jdn = TimePoint::ymd_to_jdn_from_iso_week(iso_y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        // Sunday-based week number (%U)
        if let (Some(y), Some(w)) = (self.year, self.week_sun) {
            let wd = self.weekday.unwrap_or(Weekday::Sunday);
            let jdn = TimePoint::ymd_to_jdn_from_week_sun(y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        // Monday-based week number (%W)
        if let (Some(y), Some(w)) = (self.year, self.week_mon) {
            let wd = self.weekday.unwrap_or(Weekday::Monday);
            let jdn = TimePoint::ymd_to_jdn_from_week_mon(y, w, wd);
            return jdn_to_naive_date(jdn);
        }

        Err(an_err!(DtErrKind::InvalidInput, "failed to convert"))
    }

    fn build_naive_time(&self) -> Result<NaiveTime, DtErr> {
        let mut hour = self.hour.unwrap_or(0) as u32;
        let minute = self.minute.unwrap_or(0) as u32;
        let mut second = self.second.unwrap_or(0) as u32;

        if let Some(meridiem) = self.meridiem {
            match (hour, meridiem) {
                (12, Meridiem::AM) => hour = 0,
                (12, Meridiem::PM) => {}
                (h, Meridiem::PM) if h < 12 => hour = h + 12,
                _ => {}
            }
        }

        let raw_ns_u64 = if let Some(attos) = self.attos {
            attos / ATTOS_PER_NS
        } else {
            0
        };

        let is_leap = second == 60 || self.is_leap_second;
        if !is_leap && raw_ns_u64 > 999_999_999 {
            return Err(an_err!(DtErrKind::OutOfRange, "leap ns: {}", raw_ns_u64));
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
            return Err(an_err!(DtErrKind::OutOfRange, "seconds: {}", second));
        }

        NaiveTime::from_hms_nano_opt(hour, minute, second, subsec_nano).ok_or_else(|| {
            an_err!(
                DtErrKind::InvalidInput,
                "hms: {} {} {} {}",
                hour,
                minute,
                second,
                subsec_nano
            )
        })
    }

    /// Helper: resolve fixed offset / UTC only.
    fn to_chrono_offset(&self) -> Result<FixedOffset, DtErr> {
        match self.offset {
            Some(Offset::Fixed(secs)) => FixedOffset::east_opt(secs)
                .ok_or_else(|| an_err!(DtErrKind::InvalidTimezoneOffset, "offset secs: {}", secs)),
            Some(Offset::Utc) | Some(Offset::None) | None => FixedOffset::east_opt(0)
                .ok_or_else(|| an_err!(DtErrKind::InvalidTimezoneOffset, "offset secs: 0")),
        }
    }

    pub fn to_chrono_datetime(&self) -> Result<DateTime<FixedOffset>, DtErr> {
        // ============================================================
        // UNIX TIMESTAMP PATH
        // Always UTC. Completely ignores offset + iana_name.
        // ============================================================
        if let Some(secs) = self.unix_timestamp_seconds {
            let subsec_nano = if let Some(attos) = self.attos {
                let ns_u64 = attos / ATTOS_PER_NS;
                if ns_u64 > 999_999_999 {
                    999_999_999
                } else {
                    ns_u64 as u32
                }
            } else {
                0
            };

            let utc_dt = DateTime::from_timestamp(secs, subsec_nano)
                .ok_or_else(|| an_err!(DtErrKind::InvalidNumber, "timestamp: {:?}", secs))?;

            return Ok(utc_dt.with_timezone(&FixedOffset::east_opt(0).unwrap()));
        }

        // ============================================================
        // CIVIL TIME PATH (local time in the given zone)
        // ============================================================
        let naive = self.to_chrono_naive_datetime()?;

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
                #[cfg(feature = "chrono-tz")]
                {
                    let tz: Tz = name_str.parse().map_err(|e| {
                        an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "unknown IANA '{}': {}",
                            name_str,
                            e
                        )
                    })?;
                    let local = tz.from_local_datetime(&naive);
                    let zoned = local.single().ok_or_else(|| {
                        an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "local time is ambiguous or invalid in timezone '{}'",
                            name_str
                        )
                    })?;
                    return Ok(zoned.fixed_offset());
                }

                #[cfg(not(feature = "chrono-tz"))]
                {
                    let provisional_unix =
                        DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc)
                            .timestamp();

                    match offset_info_at_local(name_str, provisional_unix) {
                        Some(info) => {
                            let mut local_naive = naive;

                            if info.is_gap {
                                // Non-existent time (spring-forward gap) — shift forward
                                local_naive += chrono::Duration::seconds(info.gap_size as i64);
                            }

                            let offset = FixedOffset::east_opt(info.offset).ok_or_else(|| {
                                an_err!(DtErrKind::InvalidTimezoneOffset, "offset: {}", info.offset)
                            })?;

                            return offset
                                .from_local_datetime(&local_naive)
                                .single()
                                .ok_or_else(|| {
                                    an_err!(
                                        DtErrKind::InvalidTimezoneOffset,
                                        "offset: {:?}",
                                        offset
                                    )
                                });
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
            }
        }

        // Fixed-offset civil path
        let offset = self.to_chrono_offset()?;
        offset
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| an_err!(DtErrKind::InvalidTimezoneOffset, "offset: {:?}", offset))
    }

    pub fn to_chrono_timestamp(&self) -> Result<i64, DtErr> {
        if let Some(secs) = self.unix_timestamp_seconds {
            return Ok(secs);
        }
        let dt = self.to_chrono_datetime()?;
        Ok(dt.timestamp())
    }
}
