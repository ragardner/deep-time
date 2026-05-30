use crate::ATTOS_PER_WEEK;
use crate::{ATTOS_PER_SEC_I128, SEC_PER_DAYI64};
use crate::{Dt, Real, Scale};

impl Dt {
    /// Returns this [`Dt`] but as a unix timestamp since the UNIX epoch (1970-01-01 00:00:00).
    ///
    /// ## Notes:
    ///
    /// - Assumes this [`Dt`] is from the 2000-01-01 noon epoch.
    ///
    #[inline]
    pub const fn to_unix(&self) -> Dt {
        // TODO: go around and check all fns
        // that use this fn and need utc correctly set scale to utc before
        // calling it
        self.convert_internal(self.tag)
            .to_diff_raw(Dt::UNIX_EPOCH.convert_internal(self.tag))
    }

    /// Creates a TAI [`Dt`] from a unix (1970 epoch) timestamp.
    #[inline]
    pub const fn from_unix(unix: Dt) -> Dt {
        Self::from_diff_and_scale(unix, Dt::UNIX_EPOCH, true)
    }

    /// Returns this [`Dt`] but as an ntp timestamp since the epoch 1900-01-01 00:00:00 UTC.
    ///
    /// ## Notes:
    ///
    /// - Assumes this [`Dt`] is from the 2000-01-01 noon epoch.
    ///
    /// ## Example:
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// // 2698012800
    /// let dt = Dt::from_ymd(1985, 7, 1, 0, 0, 0, 0, Scale::TAI);
    /// let ntp = dt.to_ntp();
    ///
    /// assert_eq!(
    ///     ntp.to_attos(), Dt::sec_to_attos(2698012800_i128),
    ///     "ntp sec for 1985 is wrong, got: {}, expected: {}",
    ///     ntp.to_attos(), Dt::sec_to_attos(2698012800_i128)
    /// );
    ///
    /// let dt2 = Dt::from_ntp(ntp);
    ///
    /// assert_eq!(
    ///     dt.to_attos(), dt2.to_attos(),
    ///     "round trip to Dt got wrong sec, old: {}, new: {}",
    ///     dt.to_attos(), dt2.to_attos()
    /// );
    ///
    /// let ymd = dt2.to_ymd();
    /// assert_eq!(ymd.yr(), 1985_i64);
    /// assert_eq!(ymd.mo(), 7);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 0);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    #[inline]
    pub const fn to_ntp(&self) -> Dt {
        self.convert_internal(self.tag)
            .to_diff_raw(Dt::NTP_EPOCH.convert_internal(self.tag))
    }

    /// Creates a TAI [`Dt`] from an ntp (1900 epoch) timestamp.
    #[inline]
    pub const fn from_ntp(ntp: Dt) -> Dt {
        Self::from_diff_and_scale(ntp, Dt::NTP_EPOCH, true)
    }

    /// Returns the GPS week number and the exact Time of Week (TOW) for this instant
    /// when expressed in **GPS Time**.
    ///
    /// - GPS Time is continuous (no leap seconds) and starts at the
    ///   [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH)
    ///   (1980-01-06 00:00:00 UTC).
    /// - The returned TOW is a [`Dt`] on the TAI scale.
    ///
    /// This is the inverse of
    /// [`Dt::from_gps_wk_and_tow`](../struct.Dt.html#method.from_gps_wk_and_tow).
    ///
    /// - `week`: Full GPS week number (can be negative for dates before 1980).
    /// - `tow`: Time of Week as a [`Dt`]. Values ≥ 604800 seconds are
    ///   automatically carried into the week number.
    ///
    /// ## Examples:
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::TAI);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    pub const fn to_gps_wk_and_tow(&self) -> (i64, Dt) {
        let total_attos = self.to_gps().to_attos();
        let wk = total_attos.div_euclid(ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos.rem_euclid(ATTOS_PER_WEEK);

        (wk, Dt::new(tow_attos, self.tag))
    }

    /// Creates a [`Dt`] from a GPS week number and Time of Week (TOW).
    ///
    /// This is the inverse of
    /// [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow).
    ///
    /// - `week`: Full GPS week number (can be negative for dates before 1980).
    /// - `tow`: Time of Week as a [`Dt`]. Values ≥ 604800 seconds are
    ///   automatically carried into the week number.
    ///
    /// ## Examples:
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::TAI);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    #[inline]
    pub const fn from_gps_wk_and_tow(wk: i64, tow: Dt) -> Self {
        let total_attos = (wk as i128)
            .saturating_mul(ATTOS_PER_WEEK)
            .saturating_add(tow.to_attos());

        Self::from_gps(Dt::new(total_attos, tow.tag))
    }

    /// Returns the elapsed time since the GPS epoch as a [`Dt`] on the GPS scale.
    ///
    /// The GPS epoch is [`Dt::GPS_EPOCH`].
    #[inline]
    pub const fn to_gps(&self) -> Dt {
        self.to_scale_and_then_diff(Self::GPS_EPOCH, true)
    }

    /// TODO: scale conversion?
    /// Inverse of [`Self::to_gps`].
    #[inline]
    pub const fn from_gps(elapsed: Dt) -> Self {
        Self::from_diff_and_scale(elapsed, Self::GPS_EPOCH, true)
    }

    /// Returns the day of the GPS week (0 = Sunday, 1 = Monday, …, 6 = Saturday).
    ///
    /// This value is computed directly from the GPS Time of Week and is
    /// independent of the Gregorian calendar or civil time.
    pub const fn to_gps_day_of_wk(&self) -> u8 {
        let (_, tow) = self.to_gps_wk_and_tow();
        let secs = tow.to_attos() / ATTOS_PER_SEC_I128;

        (secs / SEC_PER_DAYI64 as i128) as u8
    }

    /// Returns the elapsed time since the Chandra X-ray Center (CXC) epoch
    /// as a [`Dt`] on the TT scale.
    ///
    /// The CXC epoch is [`Dt::CXC_EPOCH`].
    #[inline]
    pub const fn to_cxcsec(&self) -> Dt {
        self.to_scale_and_then_diff(Self::CXC_EPOCH, true)
    }

    /// TODO: scale conversion?
    /// Inverse of [`Self::to_cxcsec`].
    #[inline]
    pub const fn from_cxcsec(elapsed: Dt) -> Self {
        Self::from_diff_and_scale(elapsed, Self::CXC_EPOCH, true)
    }

    /// Floating-point counterpart of [`Self::from_cxcsec`].
    #[inline]
    pub const fn from_cxcsec_f(elapsed_sec: Real) -> Self {
        Self::from_cxcsec(Dt::from_sec_f(elapsed_sec, Scale::TAI))
    }

    /// Returns the elapsed time since the GPS/Galileo Experiment (GALEX) epoch
    /// as a [`Dt`] on the TAI scale.
    ///
    /// The GALEX epoch is [`Self::GPS_EPOCH`].
    #[inline]
    pub const fn to_galexsec(&self) -> Dt {
        self.to_scale_and_then_diff(Self::GPS_EPOCH, true)
    }

    /// Inverse of [`Self::to_galexsec`].
    #[inline]
    pub const fn from_galexsec(elapsed: Dt) -> Self {
        Self::from_diff_and_scale(elapsed, Self::GPS_EPOCH, true)
    }

    /// Floating-point counterpart of [`Self::from_galexsec`].
    #[inline]
    pub const fn from_galexsec_f(elapsed_sec: Real) -> Self {
        Self::from_galexsec(Dt::from_sec_f(elapsed_sec, Scale::TAI))
    }
}
