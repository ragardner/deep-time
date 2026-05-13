use {
    crate::{
        ATTOS_PER_NS, Dt, Meridiem, Offset, TimeParts, Weekday, an_err,
        error::{DtErr, DtErrKind},
        tzdb::offset_info_at_local,
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
    pub fn to_jiff_broken_down_time(&self) -> Result<BrokenDownTime, DtErr> {
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
            let ns_u64 = attos / ATTOS_PER_NS;
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

    pub fn to_jiff_zoned(&self) -> Result<Zoned, DtErr> {
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
    pub fn to_jiff_timestamp(&self) -> Result<Timestamp, DtErr> {
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
                let ns_u64 = attos / ATTOS_PER_NS;
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
    fn to_jiff_time_zone(&self) -> core::result::Result<JiffTimeZone, DtErr> {
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
                    Dt::ymdhms_to_unix_sec(
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
                // TODO: use jiffs tzdb with feature
                if let Some(info) = offset_info_at_local(name_str, probe_ts) {
                    let jiff_offset = JiffOffset::from_seconds(info.offset).map_err(|e| {
                        an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "offset secs: {}: {}",
                            info.offset,
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
