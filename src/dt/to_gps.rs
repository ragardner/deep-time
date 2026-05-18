use crate::{ATTOS_PER_SEC_I128, ATTOS_PER_WEEK, Dt, Real, SEC_PER_DAYI64, Scale};

impl Dt {
    /// Returns the GPS week number and the exact Time of Week (TOW) for this instant
    /// when expressed in **GPS Time**.
    ///
    /// - GPS Time is continuous (no leap seconds) and starts at the
    ///   [`Dt::GPS_EPOCH`] (1980-01-06 00:00:00 UTC).
    /// - The returned TOW is a full-precision `Dt` (attosecond resolution) on the
    ///   TAI scale.
    ///
    /// This is the most precise way to obtain GPS week + TOW information.
    pub const fn to_gps_wk_and_tow(&self, current: Scale) -> (i64, Dt) {
        let total_attos = self.to_gps(current).to_attos();

        let wk = total_attos.div_euclid(ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos.rem_euclid(ATTOS_PER_WEEK);

        (wk, Dt::from_attos(tow_attos, Scale::TAI))
    }

    /// Returns the day of the GPS week (0 = Sunday, 1 = Monday, …, 6 = Saturday).
    ///
    /// This value is computed directly from the GPS Time of Week and is
    /// independent of the Gregorian calendar or civil time.
    pub const fn to_gps_day_of_wk(&self, current: Scale) -> u8 {
        let (_, tow) = self.to_gps_wk_and_tow(current);
        let secs = tow.to_attos() / ATTOS_PER_SEC_I128;

        (secs / SEC_PER_DAYI64 as i128) as u8
    }

    /// Returns the Time of Week (TOW) as a floating-point value in seconds.
    ///
    /// This is a convenience method for code that prefers `f64` / `Real`.
    /// For full attosecond precision use [`Self::to_gps_wk_and_tow`].
    #[inline]
    pub const fn to_gps_tow_f(&self, current: Scale) -> Real {
        let (_, tow) = self.to_gps_wk_and_tow(current);
        tow.to_sec_f()
    }

    /// Returns only the GPS week number.
    ///
    /// Convenience method. For both the week number and the Time of Week, use
    /// [`Self::to_gps_wk_and_tow`].
    #[inline]
    pub const fn to_gps_wk(&self, current: Scale) -> i64 {
        self.to_gps_wk_and_tow(current).0
    }

    /// Returns the elapsed time since the GPS/Galileo Experiment (GALEX) epoch
    /// as a [`Dt`] on the TAI scale.
    ///
    /// The GALEX epoch is [`Self::GPS_EPOCH`].
    #[inline]
    pub const fn to_galexsec(&self, current: Scale) -> Dt {
        self.to(current, Scale::UTC)
            .to_diff_raw(Dt::GPS_EPOCH.to(Scale::TAI, Scale::UTC))
    }

    /// Returns the elapsed time since the GPS epoch as a [`Dt`] on the GPS scale.
    ///
    /// The GPS epoch is [`Dt::GPS_EPOCH`].
    #[inline]
    pub const fn to_gps(&self, current: Scale) -> Dt {
        self.to(current, Scale::GPS)
            .to_diff_raw(Dt::GPS_EPOCH.to(Scale::TAI, Scale::GPS))
    }

    /// Returns the elapsed time since the Chandra X-ray Center (CXC) epoch
    /// as a [`Dt`] on the TT scale.
    ///
    /// The CXC epoch is [`Dt::CXC_EPOCH`].
    #[inline]
    pub const fn to_cxcsec(&self, current: Scale) -> Dt {
        self.to(current, Scale::TT)
            .to_diff_raw(Dt::CXC_EPOCH.to(Scale::TAI, Scale::TT))
    }
}
