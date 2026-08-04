use crate::{
    Dt, Epoch, JD_2000_2_451_545, SEC_PER_DAY, an_err,
    error::{DtErr, DtErrKind},
    utc::IsLeapSec,
    {Meridiem, Offset, Parts, Weekday},
};

impl Parts {
    /// Builds a [`Dt`] from this [`Parts`].
    ///
    /// ## Timestamp present
    ///
    /// If this [`Parts`] has a [`timestamp`](Parts::timestamp):
    ///
    /// - That timestamp is used. All other fields except `scale` and `target`
    ///   are ignored.
    /// - If the timestamp’s [`Epoch`] is [`Epoch::Noon2000NoConvert`], no time
    ///   scale conversion occurs. The returned [`Dt`]'s `scale` is this
    ///   [`Parts`]'s `scale`.
    /// - Otherwise the count is taken on this [`Parts`]'s `scale` and converted
    ///   to TAI (no-op if `scale` is already TAI). The returned [`Dt`]'s `scale`
    ///   is **TAI**.
    /// - If the epoch is [`Epoch::Unix`], the attosecond count is also shifted
    ///   from 1970-01-01 onto library noon (2000-01-01 noon).
    ///   [`Epoch::Noon2000`] and [`Epoch::Noon2000NoConvert`] are already counted
    ///   from library noon.
    ///
    /// ## No timestamp
    ///
    /// Date and time-of-day fields are used (see [Civil date priority](#civil-date-priority)).
    /// The instant is taken on this [`Parts`]'s `scale` and converted to TAI
    /// (no-op if `scale` is already TAI). The returned [`Dt`]'s `scale` is **TAI**.
    ///
    /// ## Target
    ///
    /// The returned [`Dt`]'s `target` is always this [`Parts`]'s `target`.
    ///
    /// ## Civil date priority
    ///
    /// When there is no timestamp, the first complete form below wins (later
    /// forms are not tried):
    ///
    /// 1. **Year + month + day** ([`yr`](Parts::yr), [`mo`](Parts::mo),
    ///    [`day`](Parts::day)).
    /// 2. **Year + day-of-year** ([`yr`](Parts::yr), [`day_of_yr`](Parts::day_of_yr)).
    /// 3. **ISO week** ([`iso_wk_yr`](Parts::iso_wk_yr), [`iso_wk`](Parts::iso_wk));
    ///    missing [`wkday`](Parts::wkday) defaults to Monday.
    /// 4. **Year + Sunday week** ([`yr`](Parts::yr), [`wk_sun`](Parts::wk_sun));
    ///    missing weekday defaults to Sunday.
    /// 5. **Year + Monday week** ([`yr`](Parts::yr), [`wk_mon`](Parts::wk_mon));
    ///    missing weekday defaults to Monday.
    ///
    /// Time-of-day uses [`hr`](Parts::hr), [`min`](Parts::min), [`sec`](Parts::sec),
    /// and [`attos`](Parts::attos). If [`meridiem`](Parts::meridiem) is set, `hr`
    /// is treated as 1–12 and mapped to 24-hour time.
    ///
    /// ## Time zone / offset
    ///
    /// Only used when there is no timestamp:
    ///
    /// - If [`iana_name`](Parts::iana_name) is set and non-empty, that zone is
    ///   applied and [`offset`](Parts::offset) is ignored.
    /// - Else if [`offset`](Parts::offset) is a fixed offset, it is applied
    ///   (local civil time → the underlying count before scale conversion).
    ///
    /// ### `jiff-tz` / `jiff-tz-bundle`
    ///
    /// Resolving a real IANA name (anything other than a UTC alias such as
    /// `UTC` / `Zulu`) needs the `jiff-tz` or `jiff-tz-bundle` feature (both
    /// need `alloc`). Without those features, a non-UTC IANA name returns
    /// [`DtErrKind::MissingFeature`](../error/enum.DtErrKind.html#variant.MissingFeature).
    /// With the feature, unknown zone names return
    /// [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone).
    ///
    /// Named time zones (IANA) go through jiff, so they only cover dates jiff
    /// supports — about year −9999 through 9999. For much older or newer
    /// calendar years, leave the zone name empty or use a fixed numeric offset.
    ///
    /// ## Range
    ///
    /// Aside from the IANA / jiff limit above, this method supports the full
    /// range of [`Dt`].
    ///
    /// ## Errors
    ///
    /// Returns a [`DtErr`]. Common kinds:
    ///
    /// - [`DtErrKind::ExpectedYear`](../error/enum.DtErrKind.html#variant.ExpectedYear)
    ///   — no year and no ISO week year, and no usable date form.
    /// - [`DtErrKind::InvalidDate`](../error/enum.DtErrKind.html#variant.InvalidDate)
    ///   — invalid YMD, or incomplete date fields that did not match a form above.
    /// - [`DtErrKind::DayOfYearOutOfRange`](../error/enum.DtErrKind.html#variant.DayOfYearOutOfRange)
    /// - [`DtErrKind::IsoWeekOutOfRange`](../error/enum.DtErrKind.html#variant.IsoWeekOutOfRange) /
    ///   [`DtErrKind::InvalidIsoWeek`](../error/enum.DtErrKind.html#variant.InvalidIsoWeek)
    /// - [`DtErrKind::WeekOutOfRange`](../error/enum.DtErrKind.html#variant.WeekOutOfRange)
    /// - [`DtErrKind::HourOutOfRange`](../error/enum.DtErrKind.html#variant.HourOutOfRange)
    ///   — meridiem set but hour not in 1..=12.
    /// - [`DtErrKind::MissingFeature`](../error/enum.DtErrKind.html#variant.MissingFeature) /
    ///   [`DtErrKind::InvalidTimeZone`](../error/enum.DtErrKind.html#variant.InvalidTimeZone) /
    ///   [`DtErrKind::InvalidTimestamp`](../error/enum.DtErrKind.html#variant.InvalidTimestamp) /
    ///   [`DtErrKind::ConversionFail`](../error/enum.DtErrKind.html#variant.ConversionFail)
    ///   — IANA resolution (see above).
    pub fn to_dt(&self) -> Result<Dt, DtErr> {
        // ──────────────────────────────────────────────────────────────
        // Timestamp path
        // ──────────────────────────────────────────────────────────────
        if let Some(ts) = &self.timestamp {
            match ts.epoch {
                Epoch::Unix => {
                    let unix = Dt::new(ts.attos, self.scale, self.target);
                    return Ok(Dt::from_unix(unix));
                }
                Epoch::Noon2000 => {
                    return Ok(Dt::new(ts.attos, self.scale, self.target).to_tai());
                }
                Epoch::Noon2000NoConvert => {
                    return Ok(Dt::new(ts.attos, self.scale, self.target));
                }
            }
        }

        // ──────────────────────────────────────────────────────────────
        // Civil date path
        // ──────────────────────────────────────────────────────────────
        let jd = 'try_jd: {
            // Most common case first: Classic YMD
            if let (Some(year), Some(m), Some(d)) = (self.yr, self.mo, self.day) {
                if !Dt::is_valid_ymd(year, m, d) {
                    return Err(an_err!(DtErrKind::InvalidDate));
                }
                break 'try_jd Dt::ymd_to_jd(year, m, d);
            }

            // Ordinal date (%j)
            if let (Some(year), Some(doy)) = (self.yr, self.day_of_yr) {
                if doy == 0 || doy > 366 || (doy == 366 && !Dt::is_leap_yr(year)) {
                    return Err(an_err!(DtErrKind::DayOfYearOutOfRange));
                }
                break 'try_jd Dt::ydoy_to_jd(year, doy);
            }

            // ISO week date (%G/%V)
            if let (Some(iso_y), Some(iso_w)) = (self.iso_wk_yr, self.iso_wk) {
                if iso_w == 0 || iso_w > 53 {
                    return Err(an_err!(DtErrKind::IsoWeekOutOfRange));
                }
                if iso_w == 53 && !Dt::has_iso_wk_53(iso_y) {
                    return Err(an_err!(DtErrKind::InvalidIsoWeek));
                }
                let wd = self.wkday.unwrap_or(Weekday::Monday);
                break 'try_jd Dt::iso_wk_to_jd(iso_y, iso_w, wd);
            }

            // Sunday-based week (%U)
            if let (Some(y), Some(w)) = (self.yr, self.wk_sun) {
                if w > 53 {
                    return Err(an_err!(DtErrKind::WeekOutOfRange));
                }
                let wd = self.wkday.unwrap_or(Weekday::Sunday);
                break 'try_jd Dt::wk_sun_to_jd(y, w, wd);
            }

            // Monday-based week (%W)
            if let (Some(y), Some(w)) = (self.yr, self.wk_mon) {
                if w > 53 {
                    return Err(an_err!(DtErrKind::WeekOutOfRange));
                }
                let wd = self.wkday.unwrap_or(Weekday::Monday);
                break 'try_jd Dt::wk_mon_to_jd(y, w, wd);
            }

            // Nothing matched
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

        let sec_is_60 = self.sec == 60;
        let s = if sec_is_60 { 59 } else { self.sec };

        // i128 because days × 86400 does not always fit in i64 for far-away years
        let days_since_j2000 = (jd as i128).saturating_sub(JD_2000_2_451_545 as i128);
        let seconds_from_noon = (hour as i128 - 12) * 3600 + (self.min as i128) * 60 + (s as i128);
        let mut total_sec = days_since_j2000
            .saturating_mul(SEC_PER_DAY)
            .saturating_add(seconds_from_noon);

        // ──────────────────────────────────────────────────────────────
        // Apply timezone correction (IANA or Fixed offset)
        // ──────────────────────────────────────────────────────────────
        if let Some(name) = &self.iana_name {
            let name_str = name.as_str();

            if !name_str.is_empty() {
                #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
                {
                    use crate::TAI_SEC_1970_MIDNIGHT_TO_2000_NOON;
                    use jiff::{Timestamp, tz::TimeZone};

                    let tz =
                        TimeZone::get(name_str).map_err(|_| an_err!(DtErrKind::InvalidTimeZone))?;

                    // jiff takes Unix seconds as i64, and only supports about year ±9999
                    let provisional_unix = Dt::to_i64(
                        total_sec.saturating_add(TAI_SEC_1970_MIDNIGHT_TO_2000_NOON as i128),
                    );

                    let civil = Timestamp::from_second(provisional_unix)
                        .map_err(|_| an_err!(DtErrKind::InvalidTimestamp))?
                        .to_zoned(jiff::tz::TimeZone::UTC)
                        .datetime();

                    let zoned = tz
                        .to_zoned(civil)
                        .map_err(|_| an_err!(DtErrKind::ConversionFail))?;

                    total_sec = (zoned.timestamp().as_second() as i128)
                        .saturating_sub(TAI_SEC_1970_MIDNIGHT_TO_2000_NOON as i128);
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
            total_sec = total_sec.saturating_sub(offset as i128);
        }

        let t =
            Dt::from_sec_and_frac(total_sec, self.attos as i128, self.scale, self.target).to_tai();
        if sec_is_60 && self.scale.uses_leap_seconds() {
            // leap_sec_using_sec64 takes i64; the table only has modern dates
            match Dt::leap_sec_using_sec64(Dt::to_i64(total_sec.saturating_add(1)), true) {
                Some(info) if matches!(info.is_leap_sec, IsLeapSec::Add) => Ok(t.add_sec(1)),
                _ => Ok(t),
            }
        } else {
            Ok(t)
        }
    }
}
