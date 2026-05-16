use crate::tzdb::offset_info_at_local;
use crate::{
    Dt, JD_2000_2_451_545, SEC_PER_DAYI64, Scale, TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, an_err,
    error::{DtErr, DtErrKind},
    {Meridiem, Offset, TimeParts, Weekday},
};

impl TimeParts {
    pub fn to_time_point(&self) -> Result<Dt, DtErr> {
        // ──────────────────────────────────────────────────────────────
        // Fast path: explicit Unix timestamp
        // ──────────────────────────────────────────────────────────────
        if let Some(unix_secs) = self.unix_timestamp_seconds {
            let sec = unix_secs - TAI_SECS_1970_MIDNIGHT_TO_2000_NOON;
            let subsec = self.attos.unwrap_or(0);
            return Ok(Dt::from(sec, subsec, Scale::UTC));
        }

        // ──────────────────────────────────────────────────────────────
        // Resolve 12-hour time + meridiem (AM/PM) to 24-hour hour
        // ──────────────────────────────────────────────────────────────
        let hour = match (self.hour, self.meridiem) {
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

        // ──────────────────────────────────────────────────────────────
        // Civil date path
        // ──────────────────────────────────────────────────────────────
        if self.year.is_none() && self.iso_week_year.is_none() {
            return Err(an_err!(DtErrKind::Incomplete, "no year"));
        }

        let minute = self.minute.unwrap_or(0);
        let second = self.second.unwrap_or(0);
        let subsec = self.attos.unwrap_or(0);
        let mut jdn: Option<i64> = None;

        if let Some(year) = self.year {
            if let (Some(m), Some(d)) = (self.month, self.day) {
                // Classic YMD – highest priority + full validation
                if !Dt::is_valid_ymd(year, m, d) {
                    return Err(an_err!(DtErrKind::InvalidInput, "ymd"));
                }
                jdn = Some(Dt::ymd_to_jdn(year, m, d));
            } else if let Some(doy) = self.day_of_year {
                // Ordinal date (%j) – already validated
                if doy == 0 || doy > 366 || (doy == 366 && !Dt::is_leap_year(year)) {
                    return Err(an_err!(DtErrKind::OutOfRange, "day of year"));
                }
                jdn = Some(Dt::ydoy_to_jdn(year, doy));
            }
        }

        if jdn.is_none() {
            if let (Some(iso_y), Some(iso_w)) = (self.iso_week_year, self.iso_week) {
                // ISO week date (%G/%V)
                if iso_w == 0 || iso_w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "iso week"));
                }
                if iso_w == 53 && !Dt::has_iso_week_53(iso_y) {
                    return Err(an_err!(DtErrKind::InvalidItem, "iso week"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(Dt::ymd_to_jdn_from_iso_week(iso_y, iso_w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_sun) {
                // Sunday-based week (%U)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Sunday);
                jdn = Some(Dt::ymd_to_jdn_from_week_sun(y, w, wd));
            } else if let (Some(y), Some(w)) = (self.year, self.week_mon) {
                // Monday-based week (%W)
                if w > 53 {
                    return Err(an_err!(DtErrKind::OutOfRange, "week number"));
                }
                let wd = self.weekday.unwrap_or(Weekday::Monday);
                jdn = Some(Dt::ymd_to_jdn_from_week_mon(y, w, wd));
            }
        }

        let Some(jdn) = jdn else {
            return Err(an_err!(DtErrKind::InvalidInput, "could not create julian"));
        };
        let days_since_j2000 = jdn - JD_2000_2_451_545;
        let seconds_from_noon_utc =
            (hour as i64 - 12) * 3600 + (minute as i64) * 60 + (second as i64);
        let mut sec_utc = days_since_j2000 * SEC_PER_DAYI64 + seconds_from_noon_utc;

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
                let provisional_unix = sec_utc + TAI_SECS_1970_MIDNIGHT_TO_2000_NOON;
                match offset_info_at_local(name_str, provisional_unix) {
                    Some(info) => {
                        if info.is_gap {
                            // Non-existent time (spring-forward gap) — shift forward
                            sec_utc += info.gap_size; // shift local time into the valid post-gap period
                            sec_utc -= info.offset as i64; // apply the post-jump offset
                        } else {
                            sec_utc -= info.offset as i64;
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
            sec_utc -= offset as i64; // local civil time → true UTC instant
        }
        Ok(Dt::from(sec_utc, subsec, Scale::UTC))
    }
}
