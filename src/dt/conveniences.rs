use crate::{
    ATTOS_PER_SEC_I128, ATTOS_PER_WEEK, Dt, JD_2000_2_451_545F, Real, SEC_PER_DAYI64, Scale,
};

impl Dt {
    /// Returns this [`Dt`] but as time since the
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // because from_ymd() with Scale::UTC sets the returned
    /// // Dt's target field to Scale::UTC, we do not need to use
    /// // .target() prior to calling to_unix() in order to get
    /// // a utc unix timestamp
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    /// let unix = dt.to_unix();
    ///
    /// assert_eq!(
    ///     unix.to_sec(),
    ///     946728000,
    ///     "unix sec for 2000-01-01 12:00:00 UTC is wrong, got: {}, expected: 946728000",
    ///     unix.to_sec()
    /// );
    ///
    /// let dt2 = Dt::from_unix(unix);
    ///
    /// assert_eq!(
    ///     dt.to_attos(), dt2.to_attos(),
    ///     "round trip to Dt got wrong attos, old: {}, new: {}",
    ///     dt.to_attos(), dt2.to_attos()
    /// );
    ///
    /// let ymd = dt2.to_ymd();
    /// assert_eq!(ymd.yr(), 2000_i64);
    /// assert_eq!(ymd.mo(), 1);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// -
    #[inline(always)]
    pub const fn to_unix(&self) -> Dt {
        self.to_scale_and_diff(Self::UNIX_EPOCH, true)
    }

    /// Creates a TAI [`Dt`] from a unix (1970 epoch) timestamp.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
    ///
    /// let unix = dt.to_unix().to_sec();
    ///
    /// assert_eq!(unix, 0);
    ///
    /// let roundtrip = Dt::from_unix(Dt::from_tai_sec(unix));
    ///
    /// assert_eq!(roundtrip, dt);
    /// ```
    #[inline(always)]
    pub const fn from_unix(unix: Dt) -> Dt {
        Self::from_diff_and_scale(unix, Dt::UNIX_EPOCH, true)
    }

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // 2698012800
    /// let dt = Dt::from_ymd(1985, 7, 1, Scale::TAI, 0, 0, 0, 0);
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
    ///
    /// ## See also
    ///
    /// -
    #[inline(always)]
    pub const fn to_ntp(&self) -> Dt {
        self.to_scale_and_diff(Self::NTP_EPOCH, true)
    }

    /// Creates a TAI [`Dt`] from an ntp (1900 epoch) timestamp.
    #[inline(always)]
    pub const fn from_ntp(ntp: Dt) -> Dt {
        Self::from_diff_and_scale(ntp, Self::NTP_EPOCH, true)
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
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    pub const fn to_gps_wk_and_tow(&self) -> (i64, Dt) {
        let total_attos = self.to_gps().to_attos();
        let wk = total_attos.div_euclid(ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos.rem_euclid(ATTOS_PER_WEEK);
        // was converted to target scale, scale is now target
        (wk, Dt::new(tow_attos, self.target, self.target))
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
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    pub const fn from_gps_wk_and_tow(wk: i64, tow: Dt) -> Dt {
        let total_attos = (wk as i128)
            .saturating_mul(ATTOS_PER_WEEK)
            .saturating_add(tow.to_attos());

        Self::from_gps(Dt::new(total_attos, tow.scale, tow.target))
    }

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_gps`](../struct.Dt.html#method.from_gps)
    /// - [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
    /// - [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    ///
    /// ## Implementation
    ///
    /// `convert_epoch` is `true`. If we did not convert the epoch, we would not get seconds
    /// since the GPS epoch; we would get seconds since something else.
    ///
    /// [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd) / [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    /// do the opposite: if they converted the epoch too, the difference would cancel out. See
    /// [`to_ymd`](../struct.Dt.html#method.to_ymd).
    #[inline(always)]
    pub const fn to_gps(&self) -> Dt {
        self.to_scale_and_diff(Self::GPS_EPOCH, true)
    }

    /// Inverse of [`Dt::to_gps`](../struct.Dt.html#method.to_gps).
    #[inline(always)]
    pub const fn from_gps(elapsed: Dt) -> Dt {
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

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let cxc = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0)
    ///     .target(Scale::TT)
    ///     .to_cxcsec()
    ///     .to_sec_f();
    ///
    /// // cxcsec 694224032.184 (matches Astropy)
    /// assert_eq!(cxc, 694224032.184);
    /// ```
    #[inline(always)]
    pub const fn to_cxcsec(&self) -> Dt {
        self.to_scale_and_diff(Self::CXC_EPOCH, true)
    }

    /// Inverse of [`Dt::to_cxcsec`](../struct.Dt.html#method.to_cxcsec).
    #[inline(always)]
    pub const fn from_cxcsec(elapsed: Dt) -> Dt {
        Self::from_diff_and_scale(elapsed, Self::CXC_EPOCH, true)
    }

    /// Floating-point counterpart of [`Self::from_cxcsec`].
    #[inline(always)]
    pub const fn from_cxcsec_f(elapsed_sec: Real) -> Dt {
        Self::from_cxcsec(Dt::from_sec_f(elapsed_sec, Scale::TAI))
    }

    /// Returns the elapsed time since the GALEX epoch as a [`Dt`] expressed
    /// in this object's current `target` scale.
    ///
    /// The GALEX epoch is [`Self::GPS_EPOCH`] (same epoch used by GPS time).
    ///
    /// This method can match Astropy’s `Time.galexsec` format. To match
    /// Astropy output, set `.target(Scale::UTC)` (or the appropriate scale)
    /// before calling.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the [`Dt::GPS_EPOCH`] to the time
    ///   scale of its `target` field before doing a raw difference with the epoch.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch, if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let galexsec = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0)
    ///     .target(Scale::UTC)
    ///     .to_galexsec()
    ///     .to_sec_f();
    ///
    /// assert_eq!(galexsec, 1261871963.0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec)
    #[inline(always)]
    pub const fn to_galexsec(&self) -> Dt {
        self.to_scale_and_diff(Self::GPS_EPOCH, true)
    }

    /// Inverse of [`Dt::to_galexsec`](../struct.Dt.html#method.to_galexsec).
    #[inline(always)]
    pub const fn from_galexsec(elapsed: Dt) -> Dt {
        Self::from_diff_and_scale(elapsed, Self::GPS_EPOCH, true)
    }

    /// Floating-point counterpart of
    /// [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec).
    #[inline(always)]
    pub const fn from_galexsec_f(elapsed_sec: Real) -> Dt {
        Self::from_galexsec(Dt::from_sec_f(elapsed_sec, Scale::TAI))
    }

    /// Returns the **Julian epoch year** (JYEAR) for this instant.
    ///
    /// Julian years are defined as exactly 365.25 days of 86400 SI seconds.
    /// This is the system used for J2000.0 and many astronomical calculations.
    ///
    /// This is **not** the same as [`Self::to_decimalyear`], which uses the
    /// actual length of the specific Gregorian year.
    ///
    /// This is the inverse of [`Self::from_jyear`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert_eq!(x.to_jyear(), 2019.9986310746065);
    /// ```
    #[inline(always)]
    pub const fn to_jyear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(2000.0) + (jd_tt - JD_2000_2_451_545F) / f!(365.25)
    }

    /// Inverse of [`Self::to_jyear`].
    pub const fn from_jyear(jyear: Real, scale: Scale) -> Dt {
        if jyear.is_nan() {
            return Self::ZERO;
        }
        if jyear.is_infinite() {
            return if jyear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = JD_2000_2_451_545F + (jyear - f!(2000.0)) * f!(365.25);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **Besselian epoch year** (BYEAR) for this instant.
    ///
    /// Besselian years are an older astronomical convention based on a
    /// tropical year length of approximately 365.242198781 days.
    ///
    /// This is the inverse of [`Self::from_byear`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert!((x.to_byear() - 2020.000335739628).abs() < 1e-12);
    /// ```
    #[inline]
    pub const fn to_byear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(1900.0) + (jd_tt - f!(2415020.31352)) / f!(365.242198781)
    }

    /// Inverse of [`Self::to_byear`].
    pub const fn from_byear(byear: Real, scale: Scale) -> Dt {
        if byear.is_nan() {
            return Self::ZERO;
        }
        if byear.is_infinite() {
            return if byear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = f!(2415020.31352) + (byear - f!(1900.0)) * f!(365.242198781);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **decimal year** (Gregorian calendar year + fraction of the year).
    ///
    /// This is the direct equivalent of Astropy’s `Time.decimalyear`:
    /// - Uses the *actual* length of the specific Gregorian year (365 or 366 days,
    ///   plus any leap seconds on UTC/UtcSpice/etc.).
    /// - Scale-aware (TAI, TT, UTC, TDB, etc.), converts to this [`Dt`]'s target time
    ///   scale before producing an output.
    /// - Exact integer arithmetic for the year boundaries, then a high-precision
    ///   `to_sec_f` division (lossy only at the final `Real` step, same as Astropy).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert_eq!(x.to_decimalyear(), 2020.0);
    ///
    /// // Also works for negative years
    /// let y = Dt::from_ymd(-2000, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert_eq!(y.to_decimalyear(), -2000.0);
    /// ```
    pub fn to_decimalyear(&self) -> Real {
        let ymd = self.to_ymd();
        let year = ymd.yr;

        let start = Self::from_ymd(year, 1, 1, self.target, 0, 0, 0, 0);
        let next_start = Self::from_ymd(year + 1, 1, 1, self.target, 0, 0, 0, 0);

        let elapsed = self.to_diff_raw(start).to_sec_f();
        let year_length = next_start.to_diff_raw(start).to_sec_f();

        // year_length is never zero for representable years
        f!(year) + elapsed / year_length
    }
}
