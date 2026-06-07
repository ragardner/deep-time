use crate::leap_seconds::leap_sec;
use crate::{
    Dt, JD_2000_2_451_545, SEC_PER_DAYI64, TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, an_err,
    error::{DtErr, DtErrKind},
    {Meridiem, Offset, TimeParts, Weekday},
};

impl TimeParts {
    /// Converts [`TimeParts`] → [`Dt`].
    /// - Resulting [`Dt`] is on the TAI timescale.
    /// - If this [`TimeParts`] has a unix timestamp then it is used
    ///   instead of anything else.
    pub fn to_dt(&self) -> Result<Dt, DtErr> {
        // ──────────────────────────────────────────────────────────────
        // Fast path: explicit Unix timestamp
        // ──────────────────────────────────────────────────────────────
        if let Some(unix_secs) = self.unix_timestamp_seconds {
            let total_sec = unix_secs.saturating_sub(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);
            return Ok(Dt::from_sec_and_attos(
                total_sec,
                self.attos.unwrap_or(0),
                self.scale,
            ));
        }

        // ──────────────────────────────────────────────────────────────
        // Civil date path
        // ──────────────────────────────────────────────────────────────
        let mut jd: Option<i64> = None;

        if let Some(year) = self.yr {
            if let (Some(m), Some(d)) = (self.mo, self.day) {
                // Classic YMD – highest priority + full validation
                if !Dt::is_valid_ymd(year, m, d) {
                    return Err(an_err!(DtErrKind::InvalidInput, "ymd"));
                }
                jd = Some(Dt::ymd_to_jd(year, m, d));
            } else if let Some(doy) = self.day_of_yr {
                // Ordinal date (%j) – already validated
                if doy == 0 || doy > 366 || (doy == 366 && !Dt::is_leap_yr(year)) {
                    return Err(an_err!(DtErrKind::OutOfRange, "day of year"));
                }
                jd = Some(Dt::ydoy_to_jd(year, doy));
            }
        }

        if jd.is_none() {
            if let (Some(iso_y), Some(iso_w)) = (self.iso_wk_yr, self.iso_wk) {
                // ISO week date (%G/%V)
                if iso_w == 0 || iso_w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "iso week"));
                }
                if iso_w == 53 && !Dt::has_iso_wk_53(iso_y) {
                    return Err(an_err!(DtErrKind::InvalidItem, "iso week"));
                }
                let wd = self.wkday.unwrap_or(Weekday::Monday);
                jd = Some(Dt::iso_wk_to_jd(iso_y, iso_w, wd));
            } else if let (Some(y), Some(w)) = (self.yr, self.wk_sun) {
                // Sunday-based week (%U)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.wkday.unwrap_or(Weekday::Sunday);
                jd = Some(Dt::wk_sun_to_jd(y, w, wd));
            } else if let (Some(y), Some(w)) = (self.yr, self.wk_mon) {
                // Monday-based week (%W)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.wkday.unwrap_or(Weekday::Monday);
                jd = Some(Dt::wk_mon_to_jd(y, w, wd));
            }
        }

        let Some(jd) = jd else {
            if self.yr.is_none() && self.iso_wk_yr.is_none() {
                return Err(an_err!(DtErrKind::Incomplete, "no year"));
            } else {
                return Err(an_err!(DtErrKind::InvalidInput, "could not create julian"));
            }
        };

        // ──────────────────────────────────────────────────────────────
        // Resolve 12-hour time + meridiem (AM/PM) to 24-hour hour
        // ──────────────────────────────────────────────────────────────
        let hour = match (self.hr, self.meridiem) {
            (Some(h), Some(m)) => {
                if !(1..=12).contains(&h) {
                    return Err(an_err!(DtErrKind::OutOfRange, "hour: {}", h));
                }
                match (h, m) {
                    (12, Meridiem::AM) => 0,
                    (12, Meridiem::PM) => 12,
                    (h, Meridiem::AM) => h,
                    (h, Meridiem::PM) => h + 12,
                }
            }
            (Some(h), None) => h,
            (None, _) => 0,
        };

        let minute = self.min.unwrap_or(0) as i64;
        let mut second = self.sec.unwrap_or(0) as i64;
        let sec_is_60 = second == 60;
        if sec_is_60 {
            second = second.saturating_sub(1)
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
            let name_str = name.as_str().map_err(|e| {
                an_err!(
                    DtErrKind::InvalidBytes,
                    "invalid iana ascii: {:?}: {}",
                    name,
                    e
                )
            })?;

            if !name_str.is_empty() {
                #[cfg(feature = "jiff-tz")]
                {
                    use jiff::{Timestamp, tz::TimeZone};

                    let tz = TimeZone::get(name_str).map_err(|e| {
                        an_err!(
                            DtErrKind::InvalidTimezoneOffset,
                            "invalid iana {:?}: {}",
                            name,
                            e
                        )
                    })?;

                    let provisional_unix =
                        total_sec.saturating_add(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);

                    let civil = Timestamp::from_second(provisional_unix)
                        .map_err(|e| {
                            an_err!(
                                DtErrKind::InvalidNumber,
                                "invalid unix {:?}: {}",
                                provisional_unix,
                                e
                            )
                        })?
                        .to_zoned(TimeZone::UTC)
                        .datetime();

                    let zoned = tz.to_ambiguous_zoned(civil).compatible().map_err(|e| {
                        an_err!(
                            DtErrKind::OutOfRange,
                            "jiff .compatible(): {:?}: {}",
                            civil,
                            e
                        )
                    })?;

                    total_sec = zoned
                        .timestamp()
                        .as_second()
                        .saturating_sub(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);
                }
                #[cfg(not(feature = "jiff-tz"))]
                {
                    use crate::tz::UTC_ALIASES;

                    if !UTC_ALIASES.contains(&name_str) {
                        return Err(an_err!(
                            DtErrKind::InvalidBytes,
                            "non-utc tz: {} requires jiff-tz feature",
                            name_str,
                        ));
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
            Ok(Dt::from_sec_and_attos(
                total_sec,
                self.attos.unwrap_or(0),
                self.scale,
            ))
        } else {
            if self.scale.uses_leap_seconds() {
                let t = Dt::from_sec_and_attos(total_sec, self.attos.unwrap_or(0), self.scale);
                let is_leap_sec = match leap_sec(total_sec.saturating_add(1), true) {
                    Some(info) => info.is_leap_sec,
                    None => false,
                };
                if is_leap_sec { Ok(t.add_sec(1)) } else { Ok(t) }
            } else {
                Ok(Dt::from_sec_and_attos(
                    total_sec,
                    self.attos.unwrap_or(0),
                    self.scale,
                ))
            }
        }
    }
}
