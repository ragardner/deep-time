use crate::leap_seconds::leap_sec;
use crate::tzdb::offset_info_at_local;
use crate::{
    Dt, JD_2000_2_451_545, SEC_PER_DAYI64, Scale, TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, an_err,
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
            if self.scale == Scale::UTC {
                return Ok(Dt::from_sec_and_attos(
                    total_sec + leap_sec(total_sec, true).offset,
                    self.attos.unwrap_or(0),
                    Scale::TAI,
                )
                .target(Scale::UTC)); // TODO: perf
            } else {
                return Ok(Dt::from_sec_and_attos(
                    total_sec,
                    self.attos.unwrap_or(0),
                    self.scale,
                ));
            }
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
                jd = Some(Dt::ymd_to_jd_from_iso_wk(iso_y, iso_w, wd));
            } else if let (Some(y), Some(w)) = (self.yr, self.wk_sun) {
                // Sunday-based week (%U)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.wkday.unwrap_or(Weekday::Sunday);
                jd = Some(Dt::ymd_to_jd_from_wk_sun(y, w, wd));
            } else if let (Some(y), Some(w)) = (self.yr, self.wk_mon) {
                // Monday-based week (%W)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.wkday.unwrap_or(Weekday::Monday);
                jd = Some(Dt::ymd_to_jd_from_wk_mon(y, w, wd));
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
        let second = self.sec.unwrap_or(0) as i64;
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
                let provisional_unix =
                    total_sec.saturating_add(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON);
                match offset_info_at_local(name_str, provisional_unix) {
                    Some(info) => {
                        if info.is_gap {
                            // Non-existent time (spring-forward gap) — shift forward
                            total_sec = total_sec.saturating_add(info.gap_size);
                            total_sec = total_sec.saturating_sub(info.offset as i64);
                        } else {
                            total_sec = total_sec.saturating_sub(info.offset as i64);
                        }
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
        } else if let Some(Offset::Fixed(offset)) = self.offset {
            // local civil time → true UTC instant
            total_sec = total_sec.saturating_sub(offset as i64);
        }

        // ──────────────────────────────────────────────────────────────
        // Final construction
        // ──────────────────────────────────────────────────────────────
        let lookup_offset = if second == 60 { 1 } else { 0 };
        if self.scale == Scale::UTC {
            Ok(Dt::from_sec_and_attos(
                total_sec + leap_sec(total_sec - lookup_offset, true).offset,
                self.attos.unwrap_or(0),
                Scale::TAI,
            )
            .target(Scale::UTC)) // TODO perf
        } else {
            Ok(Dt::from_sec_and_attos(
                total_sec,
                self.attos.unwrap_or(0),
                self.scale,
            ))
        }
    }
}
