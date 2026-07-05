use crate::{
    Dt, Epoch, JD_2000_2_451_545, SEC_PER_DAYI64, an_err,
    error::{DtErr, DtErrKind},
    utc::IsLeapSec,
    {Meridiem, Offset, Parts, Weekday},
};

impl Parts {
    /// Converts [`Parts`] → [`Dt`].
    /// - Resulting [`Dt`] is on the TAI timescale.
    /// - If this [`Parts`] has a timestamp then it is used
    ///   instead of anything else.
    pub fn to_dt(&self) -> Result<Dt, DtErr> {
        // ──────────────────────────────────────────────────────────────
        // Explicit timestamp (%s or %J)
        // ──────────────────────────────────────────────────────────────
        if let Some(ts) = self.timestamp {
            match ts.epoch {
                Epoch::Unix => {
                    let unix = Dt::new(ts.attos, self.scale, self.scale);
                    return Ok(Dt::from_unix(unix));
                }
                Epoch::Noon2000 => return Ok(Dt::from_attos(ts.attos, self.scale)),
            }
        }

        // ──────────────────────────────────────────────────────────────
        // Civil date path
        // ──────────────────────────────────────────────────────────────
        let mut jd: Option<i64> = None;

        // Most common case first: Classic YMD
        if let (Some(year), Some(m), Some(d)) = (self.yr, self.mo, self.day) {
            if !Dt::is_valid_ymd(year, m, d) {
                return Err(an_err!(DtErrKind::InvalidDate));
            }
            jd = Some(Dt::ymd_to_jd(year, m, d));
        }
        // Ordinal date (%j)
        else if let (Some(year), Some(doy)) = (self.yr, self.day_of_yr) {
            if doy == 0 || doy > 366 || (doy == 366 && !Dt::is_leap_yr(year)) {
                return Err(an_err!(DtErrKind::DayOfYearOutOfRange));
            }
            jd = Some(Dt::ydoy_to_jd(year, doy));
        }
        // ISO week date (%G/%V)
        else if let (Some(iso_y), Some(iso_w)) = (self.iso_wk_yr, self.iso_wk) {
            if iso_w == 0 || iso_w > 53 {
                return Err(an_err!(DtErrKind::IsoWeekOutOfRange));
            }
            if iso_w == 53 && !Dt::has_iso_wk_53(iso_y) {
                return Err(an_err!(DtErrKind::InvalidIsoWeek));
            }
            let wd = self.wkday.unwrap_or(Weekday::Monday);
            jd = Some(Dt::iso_wk_to_jd(iso_y, iso_w, wd));
        }
        // Sunday-based week (%U)
        else if let (Some(y), Some(w)) = (self.yr, self.wk_sun) {
            if w > 53 {
                return Err(an_err!(DtErrKind::WeekOutOfRange));
            }
            let wd = self.wkday.unwrap_or(Weekday::Sunday);
            jd = Some(Dt::wk_sun_to_jd(y, w, wd));
        }
        // Monday-based week (%W)
        else if let (Some(y), Some(w)) = (self.yr, self.wk_mon) {
            if w > 53 {
                return Err(an_err!(DtErrKind::WeekOutOfRange));
            }
            let wd = self.wkday.unwrap_or(Weekday::Monday);
            jd = Some(Dt::wk_mon_to_jd(y, w, wd));
        }

        let Some(jd) = jd else {
            if self.yr.is_none() && self.iso_wk_yr.is_none() {
                return Err(an_err!(DtErrKind::ExpectedYear));
            } else {
                return Err(an_err!(DtErrKind::InvalidDate));
            }
        };

        // ──────────────────────────────────────────────────────────────
        // Resolve 12-hour time + meridiem (AM/PM) to 24-hour hour
        // ──────────────────────────────────────────────────────────────
        let hour = match self.meridiem {
            None => self.hr,
            Some(m) => {
                if !(1..=12).contains(&self.hr) {
                    return Err(an_err!(DtErrKind::HourOutOfRange));
                }
                match (self.hr, m) {
                    (12, Meridiem::AM) => 0,
                    (12, Meridiem::PM) => 12,
                    (h, Meridiem::AM) => h,
                    (h, Meridiem::PM) => h + 12,
                }
            }
        };

        let minute = self.min as i64;
        let mut second = self.sec as i64;
        let sec_is_60 = second == 60;
        if sec_is_60 {
            second -= 1;
        }

        let days_since_j2000 = jd.saturating_sub(JD_2000_2_451_545);
        let seconds_from_noon_utc = (hour as i64 - 12) * 3600 + minute * 60 + second;
        let mut total_sec: i64 = days_since_j2000
            .saturating_mul(SEC_PER_DAYI64)
            .saturating_add(seconds_from_noon_utc);

        // ──────────────────────────────────────────────────────────────
        // Apply timezone correction (IANA or Fixed offset)
        // ──────────────────────────────────────────────────────────────
        if let Some(name) = &self.iana_name {
            let name_str = name.as_str();

            if !name_str.is_empty() {
                #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
                {
                    use crate::TAI_SECS_1970_MIDNIGHT_TO_2000_NOON;
                    use jiff::{Timestamp, tz::TimeZone};

                    let tz =
                        TimeZone::get(name_str).map_err(|_| an_err!(DtErrKind::InvalidTimeZone))?;

                    let provisional_unix =
                        total_sec.saturating_add(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);

                    let civil = Timestamp::from_second(provisional_unix)
                        .map_err(|_| an_err!(DtErrKind::InvalidTimestamp))?
                        .to_zoned(jiff::tz::TimeZone::UTC)
                        .datetime();

                    let zoned = tz
                        .to_zoned(civil)
                        .map_err(|_| an_err!(DtErrKind::ConversionFail))?;

                    total_sec = zoned
                        .timestamp()
                        .as_second()
                        .saturating_sub(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);
                }
                #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
                {
                    use crate::tz::UTC_ALIASES;

                    if !UTC_ALIASES.contains(&name_str) {
                        return Err(an_err!(DtErrKind::MissingFeature));
                    }
                }
            }
        } else if let Some(Offset::Fixed(offset)) = self.offset {
            // local civil time → true UTC instant
            total_sec = total_sec.saturating_sub(offset as i64);
        }

        // ──────────────────────────────────────────────────────────────
        // Final construction
        // ──────────────────────────────────────────────────────────────
        if !sec_is_60 {
            Ok(Dt::from_sec_and_ufrac(total_sec, self.attos, self.scale))
        // sec is 60
        } else if self.scale.uses_leap_seconds() {
            let t = Dt::from_sec_and_ufrac(total_sec, self.attos, self.scale);
            match Dt::leap_sec_using_sec64(total_sec.saturating_add(1), true) {
                Some(info) => match info.is_leap_sec {
                    IsLeapSec::Add => Ok(t.add_sec(1)),
                    // Negative leaps have no civil 23:59:60; treat as ordinary :59.
                    _ => Ok(t),
                },
                None => Ok(t),
            }
        } else {
            Ok(Dt::from_sec_and_ufrac(total_sec, self.attos, self.scale))
        }
    }
}
