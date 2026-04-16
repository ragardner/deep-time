use crate::{
    ClockType, TimePoint,
    parser::{Error, ParseErr, ParsedDate, ParsedTimeScale, TimeZone, Weekday},
};

impl ParsedDate {
    /// Same method as before, now with full support for:
    /// • YMD (existing)
    /// • Ordinal date (`%j` / `day_of_year`)
    /// • ISO week date (`%G` / `%V` + `weekday`)
    ///
    /// Priority order (exactly like Jiff/Chrono):
    /// 1. Unix timestamp (if present)
    /// 2. YMD
    /// 3. Ordinal (year + day_of_year)
    /// 4. ISO week date (iso_week_year + iso_week + weekday)
    /// 5. Error `IncompleteDate`
    pub fn to_time_point(&self) -> Result<TimePoint, Error> {
        // Still bail exactly as you originally requested
        if self.iana_name.is_some_and(|n| n.iter().any(|&b| b != 0)) {
            return Err(Error::simple(ParseErr::TimePointIana));
        }
        if matches!(self.tz, Some(TimeZone::Fixed(_))) {
            return Err(Error::simple(ParseErr::TimePointTimeZone));
        }

        // ──────────────────────────────────────────────────────────────
        // Fast path: explicit Unix timestamp
        // ──────────────────────────────────────────────────────────────
        if let Some(unix_secs) = self.unix_timestamp_seconds {
            const UNIX_EPOCH_TO_J2000_NOON_UTC: i128 = 946_728_000;
            let sec = (unix_secs as i128) - UNIX_EPOCH_TO_J2000_NOON_UTC;
            let subsec = self.microquectos.unwrap_or(0);
            let utc_tp = TimePoint::new(sec, subsec, ClockType::UTC);

            return Ok(match self.timescale {
                ParsedTimeScale::Utc => utc_tp,
                ParsedTimeScale::Tai | ParsedTimeScale::SiContinuous => utc_tp.to_tai(),
                ParsedTimeScale::Tt => utc_tp.to_clock_type(ClockType::TT),
            });
        }

        // ──────────────────────────────────────────────────────────────
        // Civil date path (now supports all three calendar styles)
        // ──────────────────────────────────────────────────────────────
        let year = self
            .year
            .ok_or_else(|| Error::simple(ParseErr::TimePointYearIncompleteDate))?;
        let hour = self.hour.unwrap_or(0);
        let minute = self.minute.unwrap_or(0);
        let second = self.second.unwrap_or(0);
        let subsec = self.microquectos.unwrap_or(0);

        let jdn = if let (Some(m), Some(d)) = (self.month, self.day) {
            // Classic YMD (highest priority)
            TimePoint::gregorian_jdn(year, m, d)
        } else if let Some(doy) = self.day_of_year {
            // Ordinal date (%j)
            if doy == 0 || doy > 366 || (doy == 366 && !TimePoint::is_leap_year(year)) {
                return Err(Error::simple(ParseErr::TimePointDayOfYearOutOfRange));
            }
            TimePoint::gregorian_jdn_from_ordinal(year, doy)
        } else if let (Some(iso_y), Some(w)) = (self.iso_week_year, self.iso_week) {
            // ISO week date (%G/%V)
            if w == 0 || w > 53 {
                return Err(Error::simple(ParseErr::TimePointIsoWeekOutOfRange));
            }
            let wd = self.weekday.unwrap_or(Weekday::Monday);
            TimePoint::gregorian_jdn_from_iso_week(iso_y, w, wd)
        } else {
            return Err(Error::simple(ParseErr::TimePointJdnIncompleteDate));
        };

        let days_since_j2000 = jdn - 2_451_545i128;

        let seconds_from_noon_utc =
            (hour as i128 - 12) * 3600 + (minute as i128) * 60 + (second as i128);

        let sec_utc = days_since_j2000 * 86_400 + seconds_from_noon_utc;

        let utc_tp = TimePoint::new(sec_utc, subsec, ClockType::UTC);

        Ok(match self.timescale {
            ParsedTimeScale::Utc => utc_tp,
            ParsedTimeScale::Tai | ParsedTimeScale::SiContinuous => utc_tp.to_tai(),
            ParsedTimeScale::Tt => utc_tp.to_clock_type(ClockType::TT),
        })
    }
}
