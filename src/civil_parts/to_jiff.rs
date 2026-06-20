use {
    crate::{
        ATTOS_PER_NS, Meridiem, Offset, Parts, Weekday, an_err,
        error::{DtErr, DtErrKind},
    },
    alloc::string::String,
    core::result::Result,
    jiff::{
        Timestamp, Zoned,
        fmt::strtime::{BrokenDownTime, Meridiem as JiffMeridiem},
        tz::Offset as JiffOffset,
    },
};

impl Parts {
    /// Converts [`Parts`] → [`jiff::fmt::strtime::BrokenDownTime`].
    pub fn to_jiff_broken_down_time(&self) -> Result<BrokenDownTime, DtErr> {
        let mut bdt = BrokenDownTime::default();

        // Date fields
        if let Some(year) = self.yr {
            let y: i16 = year
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "year: {}: {}", year, e))?;
            bdt.set_year(Some(y))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "year: {}: {}", y, e))?;
        }
        if let Some(m) = self.mo {
            bdt.set_month(Some(m as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "month: {}: {}", m, e))?;
        }
        if let Some(d) = self.day {
            bdt.set_day(Some(d as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "day: {}: {}", d, e))?;
        }

        // Week / day-of-year fields
        if let Some(doy) = self.day_of_yr {
            bdt.set_day_of_year(Some(doy as i16))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "doy: {}: {}", doy, e))?;
        }
        if let Some(y) = self.iso_wk_yr {
            let y: i16 = y
                .try_into()
                .map_err(|e| an_err!(DtErrKind::InvalidInput, "iso wk yr: {}: {}", y, e))?;
            bdt.set_iso_week_year(Some(y))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "iso wk yr: {}: {}", y, e))?;
        }
        if let Some(w) = self.iso_wk {
            bdt.set_iso_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "iso wk: {}: {}", w, e))?;
        }
        if let Some(w) = self.wk_sun {
            bdt.set_sunday_based_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "sun based wk: {}: {}", w, e))?;
        }
        if let Some(w) = self.wk_mon {
            bdt.set_monday_based_week(Some(w as i8))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "mon based wk: {}: {}", w, e))?;
        }

        // Time of day
        bdt.set_hour(Some(self.hr as i8))
            .map_err(|e| an_err!(DtErrKind::InvalidItem, "hour: {}: {}", self.hr, e))?;
        bdt.set_minute(Some(self.min as i8))
            .map_err(|e| an_err!(DtErrKind::InvalidItem, "minute: {}: {}", self.min, e))?;
        let non_ls_s = if self.sec == 60 { 59 } else { self.sec };
        bdt.set_second(Some(non_ls_s as i8))
            .map_err(|e| an_err!(DtErrKind::InvalidItem, "second: {}: {}", non_ls_s, e))?;

        // Subsecond precision (attoseconds → nanoseconds)
        if self.attos != 0 {
            let ns_u64 = self.attos / ATTOS_PER_NS;
            let ns: i32 = if ns_u64 >= 1_000_000_000 {
                999_999_999
            } else {
                ns_u64 as i32
            };
            bdt.set_subsec_nanosecond(Some(ns))
                .map_err(|e| an_err!(DtErrKind::InvalidItem, "ns: {}: {}", ns, e))?;
        }

        // Infallible setters
        if let Some(wd) = self.wkday {
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
            let name_str = name.as_str();
            if !name_str.is_empty() {
                bdt.set_iana_time_zone(Some(String::from(name_str)));
            }
        } else if let Some(Offset::Fixed(secs)) = self.offset {
            if let Ok(jiff_offset) = JiffOffset::from_seconds(secs) {
                bdt.set_offset(Some(jiff_offset));
            } else {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "offset secs: {}",
                    secs
                ));
            }
        } else {
            // Utc / None → treat as UTC
            bdt.set_offset(Some(JiffOffset::UTC));
        }

        Ok(bdt)
    }

    /// Converts [`Parts`] → [`jiff::Zoned`].
    pub fn to_jiff_zoned(&self) -> Result<Zoned, DtErr> {
        let bdt = self.to_jiff_broken_down_time()?;
        if let Ok(zoned) = bdt.to_zoned() {
            return Ok(zoned);
        }
        if let Ok(ts) = bdt.to_timestamp()
            && let Ok(zoned) = ts.in_tz("UTC")
        {
            return Ok(zoned);
        }
        if let Ok(dt) = bdt.to_datetime()
            && let Ok(zoned) = dt.in_tz("UTC")
        {
            return Ok(zoned);
        }
        if let Ok(date) = bdt.to_date()
            && let Ok(dt) = date.at(0, 0, 0, 0).in_tz("UTC")
        {
            return Ok(dt);
        }

        Err(an_err!(
            DtErrKind::InvalidInput,
            "could not convert to jiff zoned"
        ))
    }

    /// Converts [`Parts`] → [`jiff::Timestamp`].
    #[inline(always)]
    pub fn to_jiff_timestamp(&self) -> Result<Timestamp, DtErr> {
        self.to_jiff_zoned().map(|z| z.timestamp())
    }
}
