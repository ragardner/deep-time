use crate::{ATTOS_PER_SEC_I128, ATTOS_PER_WEEK, Dt, Real, SEC_PER_DAYI64, SEC_PER_WEEK, Scale};

impl Dt {
    /// Returns the GPS week number and exact Time of Week (TOW) for this instant
    /// when expressed in GPS Time (GPS).
    ///
    /// This is the format used by virtually all GNSS receivers, RINEX observation
    /// files, NMEA sentences, and raw satellite navigation messages.
    ///
    /// - **GPS week number**: Full (untruncated) count of 7-day weeks since the
    ///   traditional GPS reference epoch **1980-01-06 00:00:00 GPS**.
    ///   Returned as `i64` (effectively unlimited range).
    /// - **Time of Week (TOW)**: Exact elapsed time since the start of that GPS
    ///   week, returned as a [`Dt`] in the range `[0, 604800)` seconds.
    ///
    /// GPS weeks always begin on **Sunday 00:00:00 GPS**.
    ///
    /// # Correctness notes
    ///
    /// - The calculation is performed entirely on the **GPS** scale.
    /// - GPS has **no leap seconds** (it is a continuous time scale).
    /// - Leap seconds are automatically handled when converting from UTC or
    ///   other scales via `to_type(Scale::GPS)`.
    /// - The result is **exact** (attosecond precision) and independent of any
    ///   calendar or timezone rules.
    pub const fn to_gps_wk_and_tow(&self) -> (i64, Dt) {
        let elapsed = self.to_diff_raw(Self::GPS_EPOCH);
        let total_attos = elapsed.to_attos();
        let wk = (total_attos / ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos % ATTOS_PER_WEEK;
        (wk, Dt::from_attos(tow_attos, Scale::TAI))
    }

    /// Returns the day of the GPS week (0 = Sunday, 1 = Monday, …, 6 = Saturday).
    ///
    /// This is computed directly from GPS Time and is independent of the
    /// Gregorian calendar.
    pub const fn to_gps_day_of_wk(&self) -> u8 {
        let elapsed = self.to_diff_raw(Self::GPS_EPOCH);
        let total_sec = elapsed.to_attos() / ATTOS_PER_SEC_I128;
        let secs_into_wk = total_sec.rem_euclid(SEC_PER_WEEK as i128);
        (secs_into_wk / SEC_PER_DAYI64 as i128) as u8
    }

    /// Returns the Time of Week (TOW) as a floating-point value in seconds.
    ///
    /// This is a convenience method for code that prefers `f64` / `Real`.
    /// For full attosecond precision use [`Self::to_gps_wk_and_tow`].
    #[inline]
    pub const fn to_gps_tow_f(&self) -> Real {
        let (_, tow) = self.to_gps_wk_and_tow();
        tow.to_sec_f()
    }

    /// Returns only the GPS week number (full, untruncated).
    #[inline]
    pub const fn to_gps_week_number(&self) -> i64 {
        self.to_gps_wk_and_tow().0
    }
}
