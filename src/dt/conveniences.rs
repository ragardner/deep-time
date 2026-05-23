use crate::ATTOS_PER_WEEK;
use crate::{ATTOS_PER_SEC_I128, SEC_PER_DAYI64};
use crate::{Dt, Real, Scale};

impl Dt {
    /// Returns this [`Dt`] but as a unix timestamp where the:
    /// - `.sec` field is seconds since the UNIX epoch (1970-01-01 00:00:00).
    /// - `.attos` field is remaining fractional seconds.
    ///
    /// ## Notes:
    ///
    /// - Assumes this [`Dt`] is from the 2000-01-01 noon epoch.
    #[inline]
    pub const fn to_unix(&self, current: Scale, new: Scale) -> Dt {
        self.to(current, new)
            .to_diff_raw(Dt::UNIX_EPOCH.to_internal(new))
    }

    /// Creates a TAI [`Dt`] from a unix (1970 epoch) timestamp.
    #[inline]
    pub const fn from_unix(unix: Real, current: Scale) -> Dt {
        Self::from_diff_and_scale(Self::from_sec_f(unix), Dt::UNIX_EPOCH, current)
    }

    /// Returns this [`Dt`] but as an ntp timestamp where the:
    ///
    /// - `.sec` field is seconds since the epoch 1900-01-01 00:00:00 UTC.
    /// - `.attos` field is remaining fractional seconds.
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
    /// let dt = Dt::from_ymd_on(1985, 7, 1, Scale::TAI);
    /// let ntp = dt.to_ntp(Scale::TAI, Scale::TAI);
    ///
    /// assert_eq!(
    ///     ntp.sec, 2698012800_i64,
    ///     "ntp sec for 1985 is wrong, got: {}, expected: {}",
    ///     ntp.sec, 2698012800_i64
    /// );
    ///
    /// let dt2 = Dt::from_ntp(ntp.to_sec_f(), Scale::TAI);
    ///
    /// assert_eq!(
    ///     dt.sec, dt2.sec,
    ///     "round trip to Dt got wrong sec, old: {}, new: {}",
    ///     dt.sec, dt2.sec
    /// );
    ///
    /// let ymd = dt2.to_ymdhms_on(Scale::TAI, Scale::TAI);
    /// assert_eq!(ymd.yr(), 1985_i64);
    /// assert_eq!(ymd.mo(), 7);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 0);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    #[inline]
    pub const fn to_ntp(&self, current: Scale, new: Scale) -> Dt {
        self.to(current, new)
            .to_diff_raw(Dt::NTP_EPOCH.to_internal(new))
    }

    /// Creates a TAI [`Dt`] from an ntp (1900 epoch) timestamp.
    #[inline]
    pub const fn from_ntp(ntp: Real, current: Scale) -> Dt {
        Self::from_diff_and_scale(Self::from_sec_f(ntp), Dt::NTP_EPOCH, current)
    }

    /// Returns the GPS week number and the exact Time of Week (TOW) for this instant
    /// when expressed in **GPS Time**.
    ///
    /// - GPS Time is continuous (no leap seconds) and starts at the
    ///   [`Dt::GPS_EPOCH`] (1980-01-06 00:00:00 UTC).
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
    /// let x = Dt::from_ymdhms_on(2000, 1, 1, 12, 0, 0, 0, Scale::TAI);
    /// let g = x.to_gps_wk_and_tow(Scale::TAI);
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    pub const fn to_gps_wk_and_tow(&self, current: Scale) -> (i64, Dt) {
        let total_attos = self.to_gps(current).to_attos();
        let wk = total_attos.div_euclid(ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos.rem_euclid(ATTOS_PER_WEEK);

        (wk, Dt::from_attos(tow_attos, Scale::TAI))
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
    /// let x = Dt::from_ymdhms_on(2000, 1, 1, 12, 0, 0, 0, Scale::TAI);
    /// let g = x.to_gps_wk_and_tow(Scale::TAI);
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    #[inline]
    pub const fn from_gps_wk_and_tow(wk: i64, tow: Dt) -> Self {
        let total_attos = (wk as i128) * ATTOS_PER_WEEK + tow.to_attos();
        Self::GPS_EPOCH.add(Dt::from_attos(total_attos, Scale::TAI))
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

    /// Creates a [`Dt`] in GPS Time from a GPS week number and
    /// floating-point Time of Week.
    ///
    /// This is the floating-point counterpart to [`Self::from_gps_wk_and_tow`].
    #[inline]
    pub const fn from_gps_wk_and_tow_f(week: i64, tow: Real) -> Self {
        let tow_span = Dt::from_sec_f(tow);
        Self::from_gps_wk_and_tow(week, tow_span)
    }

    /// Returns the elapsed time since the GPS epoch as a [`Dt`] on the GPS scale.
    ///
    /// The GPS epoch is [`Dt::GPS_EPOCH`].
    #[inline]
    pub const fn to_gps(&self, current: Scale) -> Dt {
        self.to(current, Scale::GPS)
            .to_diff_raw(Dt::GPS_EPOCH.to(Scale::TAI, Scale::GPS))
    }

    /// Inverse of [`Self::to_gps`].
    #[inline]
    pub const fn from_gps(elapsed: Dt) -> Self {
        Self::GPS_EPOCH.add(elapsed)
    }

    /// Floating-point version of [`Self::from_gps`].
    #[inline]
    pub const fn from_gps_f(elapsed_sec: Real) -> Self {
        Self::from_gps(Dt::from_sec_f(elapsed_sec))
    }

    /// Returns only the GPS week number.
    ///
    /// Convenience method. For both the week number and the Time of Week, use
    /// [`Self::to_gps_wk_and_tow`].
    #[inline]
    pub const fn to_gps_wk(&self, current: Scale) -> i64 {
        self.to_gps_wk_and_tow(current).0
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

    /// Returns the elapsed time since the Chandra X-ray Center (CXC) epoch
    /// as a [`Dt`] on the TT scale.
    ///
    /// The CXC epoch is [`Dt::CXC_EPOCH`].
    #[inline]
    pub const fn to_cxcsec(&self, current: Scale) -> Dt {
        self.to(current, Scale::TT)
            .to_diff_raw(Dt::CXC_EPOCH.to(Scale::TAI, Scale::TT))
    }

    /// Inverse of [`Self::to_cxcsec`].
    #[inline]
    pub const fn from_cxcsec(elapsed: Dt) -> Self {
        Self::CXC_EPOCH.add(elapsed)
    }

    /// Floating-point counterpart of [`Self::from_cxcsec`].
    #[inline]
    pub const fn from_cxcsec_f(elapsed_sec: Real) -> Self {
        Self::from_cxcsec(Dt::from_sec_f(elapsed_sec))
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

    /// Inverse of [`Self::to_galexsec`].
    #[inline]
    pub const fn from_galexsec(elapsed: Dt) -> Self {
        Self::GPS_EPOCH
            .to(Scale::TAI, Scale::UTC)
            .add(elapsed)
            .to(Scale::UTC, Scale::TAI)
    }

    /// Floating-point counterpart of [`Self::from_galexsec`].
    #[inline]
    pub const fn from_galexsec_f(elapsed_sec: Real) -> Self {
        Self::from_galexsec(Dt::from_sec_f(elapsed_sec))
    }
}
