use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOS_PER_SEC_I128, Dt, JD_2000_2_451_545_I128, Real, Scale,
    floor_f,
};

impl Dt {
    /// Splits this instant's Julian Date into whole days and a remainder in attoseconds.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale before splitting. Set `target` first
    ///   if you need JD on a particular scale (e.g. `Scale::TT` or `Scale::TDB`).
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - When the whole part is negative and there is a non-zero remainder, `frac_attos`
    ///   is negative too. e.g. a jd of `-1000.25` will return the whole part as `-1000`
    ///   and the remainder as `-0.25 * ATTOS_PER_DAY` as an `i128`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// let dt = Dt::from_jd_f(2_460_782.25, Scale::TAI);
    /// // 2_460_782 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_jd(), (2_460_782, ATTOS_PER_DAY / 4));
    ///
    /// let dt = Dt::from_jd_f(-1_000.25, Scale::TAI);
    /// // -1_000 and -0.25 days in attoseconds
    /// assert_eq!(dt.to_jd(), (-1_000, -ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd_floor`](../struct.Dt.html#method.to_jd_floor)
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    #[inline(always)]
    pub const fn to_jd(&self) -> (i128, i128) {
        self.to(self.target).to_jd_raw()
    }

    /// Like [`Dt::to_jd`](../struct.Dt.html#method.to_jd), but uses this [`Dt`]'s current
    /// `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::to_jd_floor_raw`](../struct.Dt.html#method.to_jd_floor_raw)
    /// - [`Dt::to_jd_f_raw`](../struct.Dt.html#method.to_jd_f_raw)
    #[inline(always)]
    pub const fn to_jd_raw(&self) -> (i128, i128) {
        let attos = self.to_attos();
        let days_since_j2000 = attos / ATTOS_PER_DAY;
        let remaining_attos = attos % ATTOS_PER_DAY;

        let jd_int = JD_2000_2_451_545_I128.saturating_add(days_since_j2000);

        (jd_int, remaining_attos)
    }

    /// Splits this instant's Julian Date into whole days and a remainder in attoseconds.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale before splitting. Set `target` first
    ///   if you need JD on a particular scale (e.g. `Scale::TT` or `Scale::TDB`).
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - The remainder is always zero or positive and less than one day. e.g. a jd of
    ///   `-1000.25` will return the whole part as `-1001` and the remainder as
    ///   `0.75 * ATTOS_PER_DAY` as a `u128`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY_U128};
    ///
    /// let dt = Dt::from_jd_f(2_460_782.25, Scale::TAI);
    /// // 2_460_782 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_jd_floor(), (2_460_782, ATTOS_PER_HALF_DAY_U128 / 2));
    ///
    /// let dt = Dt::from_jd_f(-1_000.25, Scale::TAI);
    /// // -1_001 and effectively 0.75 days in attoseconds
    /// assert_eq!(dt.to_jd_floor(), (-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::from_jd_floor`](../struct.Dt.html#method.from_jd_floor)
    #[inline(always)]
    pub const fn to_jd_floor(&self) -> (i128, u128) {
        self.to(self.target).to_jd_floor_raw()
    }

    /// Like [`Dt::to_jd_floor`](../struct.Dt.html#method.to_jd_floor), but uses this [`Dt`]'s
    /// current `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd_floor`](../struct.Dt.html#method.to_jd_floor)
    /// - [`Dt::to_jd_raw`](../struct.Dt.html#method.to_jd_raw)
    #[inline(always)]
    pub const fn to_jd_floor_raw(&self) -> (i128, u128) {
        let attos = self.to_attos();
        let days_since_j2000 = attos.div_euclid(ATTOS_PER_DAY);
        let remaining_attos = attos.rem_euclid(ATTOS_PER_DAY);

        let jd_int = JD_2000_2_451_545_I128.saturating_add(days_since_j2000);

        (jd_int, remaining_attos as u128)
    }

    /// Returns this instant's Julian Date as an `f64`.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale first. Set `target` first if you need
    ///   JD on a particular scale (e.g. `Scale::TT` or `Scale::TDB`).
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - Same value as [`Dt::to_jd`](../struct.Dt.html#method.to_jd), expressed as a single
    ///   `f64` instead of a `(days, frac_attos)` pair.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_jd_f(2_460_782.25, Scale::TAI);
    /// assert_eq!(dt.to_jd_f(), 2_460_782.25);
    ///
    /// let dt = Dt::from_jd_f(-1_000.25, Scale::TAI);
    /// assert_eq!(dt.to_jd_f(), -1_000.25);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::to_jd_f_raw`](../struct.Dt.html#method.to_jd_f_raw)
    /// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
    #[inline]
    pub const fn to_jd_f(&self) -> Real {
        let (days, attos) = self.to_jd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Like [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f), but uses this [`Dt`]'s current
    /// `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::to_jd_raw`](../struct.Dt.html#method.to_jd_raw)
    #[inline]
    pub const fn to_jd_f_raw(&self) -> Real {
        let (days, attos) = self.to_jd_raw();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Splits this instant's Modified Julian Date into whole days and a remainder in attoseconds.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale before splitting. Set `target` first
    ///   if you need MJD on a particular scale.
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - MJD and JD relate by `JD = MJD + 2_400_000.5`.
    /// - e.g. an mjd of `-1000.25` will return the whole part as `-1001` and the
    ///   remainder as `0.75 * ATTOS_PER_DAY` as an `i128`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// let dt = Dt::from_mjd_f(60_961.25, Scale::TAI);
    /// // 60_961 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_mjd(), (60_961, ATTOS_PER_DAY / 4));
    ///
    /// let dt = Dt::from_mjd_f(-1_000.25, Scale::TAI);
    /// // -1_001 and effectively 0.75 days in attoseconds
    /// assert_eq!(dt.to_mjd(), (-1_001, 3 * ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd_floor`](../struct.Dt.html#method.to_mjd_floor)
    /// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    #[inline(always)]
    pub const fn to_mjd(&self) -> (i128, i128) {
        self.to(self.target).to_mjd_raw()
    }

    /// Like [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd), but uses this [`Dt`]'s current
    /// `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::to_mjd_floor_raw`](../struct.Dt.html#method.to_mjd_floor_raw)
    /// - [`Dt::to_mjd_f_raw`](../struct.Dt.html#method.to_mjd_f_raw)
    pub const fn to_mjd_raw(&self) -> (i128, i128) {
        let (jd_days, frac_attos) = self.to_jd_raw();

        let mut mjd_days = jd_days.saturating_sub(2_400_001);
        let mut mjd_attos = frac_attos.saturating_add(ATTOS_PER_HALF_DAY);

        if mjd_attos >= ATTOS_PER_DAY {
            mjd_days = mjd_days.saturating_add(1);
            mjd_attos = mjd_attos.saturating_sub(ATTOS_PER_DAY);
        } else if mjd_attos < 0 {
            mjd_days = mjd_days.saturating_sub(1);
            mjd_attos = mjd_attos.saturating_add(ATTOS_PER_DAY);
        }

        (mjd_days, mjd_attos)
    }

    /// Splits this instant's Modified Julian Date into whole days and a remainder in attoseconds.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale before splitting. Set `target` first
    ///   if you need MJD on a particular scale.
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - MJD and JD relate by `JD = MJD + 2_400_000.5`.
    /// - The remainder is always zero or positive and less than one day. e.g. an mjd of
    ///   `-1000.25` will return the whole part as `-1001` and the remainder as
    ///   `0.75 * ATTOS_PER_DAY` as a `u128`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY_U128};
    ///
    /// let dt = Dt::from_mjd_f(60_961.25, Scale::TAI);
    /// // 60_961 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_mjd_floor(), (60_961, ATTOS_PER_HALF_DAY_U128 / 2));
    ///
    /// let dt = Dt::from_mjd_f(-1_000.25, Scale::TAI);
    /// // -1_001 and effectively 0.75 days in attoseconds
    /// assert_eq!(dt.to_mjd_floor(), (-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
    /// - [`Dt::from_mjd_floor`](../struct.Dt.html#method.from_mjd_floor)
    #[inline(always)]
    pub const fn to_mjd_floor(&self) -> (i128, u128) {
        self.to(self.target).to_mjd_floor_raw()
    }

    /// Like [`Dt::to_mjd_floor`](../struct.Dt.html#method.to_mjd_floor), but uses this [`Dt`]'s
    /// current `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd_floor`](../struct.Dt.html#method.to_mjd_floor)
    /// - [`Dt::to_mjd_raw`](../struct.Dt.html#method.to_mjd_raw)
    pub const fn to_mjd_floor_raw(&self) -> (i128, u128) {
        let (jd_days, frac_attos) = self.to_jd_floor_raw();

        let mjd_days = jd_days.saturating_sub(2_400_001);
        let mjd_attos = frac_attos.saturating_add(ATTOS_PER_HALF_DAY as u128);

        if mjd_attos >= ATTOS_PER_DAY as u128 {
            (
                mjd_days.saturating_add(1),
                mjd_attos.saturating_sub(ATTOS_PER_DAY as u128),
            )
        } else {
            (mjd_days, mjd_attos)
        }
    }

    /// Returns this instant's Modified Julian Date as an `f64`.
    ///
    /// ## Important
    ///
    /// - Converts to this [`Dt`]'s `target` scale first. Set `target` first if you need
    ///   MJD on a particular scale.
    /// - Assumes this [`Dt`] is on the 2000-01-01 noon epoch.
    /// - Same value as [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd), expressed as a single
    ///   `f64` instead of a `(days, frac_attos)` pair.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_mjd_f(60_961.25, Scale::TAI);
    /// assert_eq!(dt.to_mjd_f(), 60_961.25);
    ///
    /// let dt = Dt::from_mjd_f(-1_000.25, Scale::TAI);
    /// assert_eq!(dt.to_mjd_f(), -1_000.25);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::to_mjd_f_raw`](../struct.Dt.html#method.to_mjd_f_raw)
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    #[inline]
    pub const fn to_mjd_f(&self) -> Real {
        let (days, attos) = self.to_mjd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Like [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f), but uses this [`Dt`]'s current
    /// `scale` instead of converting to `target` first.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
    /// - [`Dt::to_mjd_raw`](../struct.Dt.html#method.to_mjd_raw)
    #[inline]
    pub const fn to_mjd_f_raw(&self) -> Real {
        let (days, attos) = self.to_mjd_raw();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Builds a **TAI** [`Dt`] from a Julian Date given as whole days plus attoseconds.
    ///
    /// ## Important
    ///
    /// This converts from the `on` time scale to `TAI` so for example, an
    /// `on` scale of `Scale::UTC` will add leap seconds to the end result.
    ///
    /// To avoid a time scale conversion use `Scale::TAI` for the `on` argument.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] counting attoseconds since the library epoch
    /// [`Dt::ZERO`](../constant.ZERO.html) with its `scale` field set to
    /// `TAI` and its `target` field set to the `on` arg.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// // 2_460_782.25 as whole days plus 0.25 days in attoseconds
    /// let dt = Dt::from_jd(2_460_782, ATTOS_PER_DAY / 4, Scale::TAI);
    /// assert_eq!(dt.to_jd(), (2_460_782, ATTOS_PER_DAY / 4));
    ///
    /// // -1_000.25 as whole days plus -0.25 days in attoseconds
    /// let dt = Dt::from_jd(-1_000, -ATTOS_PER_DAY / 4, Scale::TAI);
    /// assert_eq!(dt.to_jd(), (-1_000, -ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_jd_floor`](../struct.Dt.html#method.from_jd_floor)
    /// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    pub const fn from_jd(jd_days: i128, frac_attos: i128, on: Scale) -> Dt {
        let days_since_j2000 = jd_days.saturating_sub(JD_2000_2_451_545_I128);
        let attos_from_days = days_since_j2000.saturating_mul(ATTOS_PER_DAY);
        let total_attos = attos_from_days.saturating_add(frac_attos);

        Self::from_attos(total_attos, on)
    }

    /// Builds a **TAI** [`Dt`] from a Julian Date given as whole days plus attoseconds.
    ///
    /// ## Important
    ///
    /// This converts from the `on` time scale to `TAI` so for example, an
    /// `on` scale of `Scale::UTC` will add leap seconds to the end result.
    ///
    /// To avoid a time scale conversion use `Scale::TAI` for the `on` argument.
    ///
    /// `frac_attos` must be zero or positive and less than one day. e.g. whole `-1001`
    /// and remainder `0.75 * ATTOS_PER_DAY` builds jd `-1000.25`.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] counting attoseconds since the library epoch
    /// [`Dt::ZERO`](../constant.ZERO.html) with its `scale` field set to
    /// `TAI` and its `target` field set to the `on` arg.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY_U128};
    ///
    /// // 2_460_782.25 as whole days plus 0.25 days in attoseconds
    /// let dt = Dt::from_jd_floor(2_460_782, ATTOS_PER_HALF_DAY_U128 / 2, Scale::TAI);
    /// assert_eq!(dt.to_jd_floor(), (2_460_782, ATTOS_PER_HALF_DAY_U128 / 2));
    ///
    /// // -1_000.25 as -1_001 plus 0.75 days in attoseconds
    /// let dt = Dt::from_jd_floor(-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2, Scale::TAI);
    /// assert_eq!(dt.to_jd_floor(), (-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    /// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
    /// - [`Dt::to_jd_floor`](../struct.Dt.html#method.to_jd_floor)
    pub const fn from_jd_floor(jd_days: i128, frac_attos: u128, on: Scale) -> Dt {
        let days_since_j2000 = jd_days.saturating_sub(JD_2000_2_451_545_I128);
        let attos_from_days = days_since_j2000.saturating_mul(ATTOS_PER_DAY);
        let total_attos = attos_from_days.saturating_add(Self::to_i128(frac_attos));

        Self::from_attos(total_attos, on)
    }

    /// Builds a **TAI** [`Dt`] from a Modified Julian Date given as whole days plus attoseconds.
    ///
    /// ## Important
    ///
    /// This converts from the `on` time scale to `TAI` so for example, an
    /// `on` scale of `Scale::UTC` will add leap seconds to the end result.
    ///
    /// To avoid a time scale conversion use `Scale::TAI` for the `on` argument.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] counting attoseconds since the library epoch
    /// [`Dt::ZERO`](../constant.ZERO.html) with its `scale` field set to
    /// `TAI` and its `target` field set to the `on` arg.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// // 60_961.25 as whole days plus 0.25 days in attoseconds
    /// let dt = Dt::from_mjd(60_961, ATTOS_PER_DAY / 4, Scale::TAI);
    /// assert_eq!(dt.to_mjd(), (60_961, ATTOS_PER_DAY / 4));
    ///
    /// // -1_000.25 as -1_001 plus 0.75 days in attoseconds
    /// let dt = Dt::from_mjd(-1_001, 3 * ATTOS_PER_DAY / 4, Scale::TAI);
    /// assert_eq!(dt.to_mjd(), (-1_001, 3 * ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_mjd_floor`](../struct.Dt.html#method.from_mjd_floor)
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    pub const fn from_mjd(mjd_days: i128, frac_attos: i128, on: Scale) -> Dt {
        // Inverse of `to_mjd_raw`: that subtracts 2_400_001 from the JD integer part and
        // adds half a day to the fraction. Here we undo both steps.
        let jd_days = mjd_days.saturating_add(2_400_001);
        let jd_attos = frac_attos.saturating_sub(ATTOS_PER_HALF_DAY);

        if jd_attos < 0 {
            Self::from_jd(
                jd_days.saturating_sub(1),
                jd_attos.saturating_add(ATTOS_PER_DAY),
                on,
            )
        } else if jd_attos >= ATTOS_PER_DAY {
            Self::from_jd(
                jd_days.saturating_add(1),
                jd_attos.saturating_sub(ATTOS_PER_DAY),
                on,
            )
        } else {
            Self::from_jd(jd_days, jd_attos, on)
        }
    }

    /// Builds a **TAI** [`Dt`] from a Modified Julian Date given as whole days plus attoseconds.
    ///
    /// ## Important
    ///
    /// This converts from the `on` time scale to `TAI` so for example, an
    /// `on` scale of `Scale::UTC` will add leap seconds to the end result.
    ///
    /// To avoid a time scale conversion use `Scale::TAI` for the `on` argument.
    ///
    /// MJD and JD relate by `JD = MJD + 2_400_000.5`.
    ///
    /// `frac_attos` must be zero or positive and less than one day. e.g. whole `-1001`
    /// and remainder `0.75 * ATTOS_PER_DAY` builds mjd `-1000.25`.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] counting attoseconds since the library epoch
    /// [`Dt::ZERO`](../constant.ZERO.html) with its `scale` field set to
    /// `TAI` and its `target` field set to the `on` arg.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY_U128};
    ///
    /// // 60_961.25 as whole days plus 0.25 days in attoseconds
    /// let dt = Dt::from_mjd_floor(60_961, ATTOS_PER_HALF_DAY_U128 / 2, Scale::TAI);
    /// assert_eq!(dt.to_mjd_floor(), (60_961, ATTOS_PER_HALF_DAY_U128 / 2));
    ///
    /// // -1_000.25 as -1_001 plus 0.75 days in attoseconds
    /// let dt = Dt::from_mjd_floor(-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2, Scale::TAI);
    /// assert_eq!(dt.to_mjd_floor(), (-1_001, 3 * ATTOS_PER_HALF_DAY_U128 / 2));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    /// - [`Dt::to_mjd_floor`](../struct.Dt.html#method.to_mjd_floor)
    pub const fn from_mjd_floor(mjd_days: i128, frac_attos: u128, on: Scale) -> Dt {
        let jd_days = mjd_days.saturating_add(2_400_000);
        let jd_attos = frac_attos.saturating_add(ATTOS_PER_HALF_DAY as u128);

        if jd_attos >= ATTOS_PER_DAY as u128 {
            Self::from_jd_floor(
                jd_days.saturating_add(1),
                jd_attos.saturating_sub(ATTOS_PER_DAY as u128),
                on,
            )
        } else {
            Self::from_jd_floor(jd_days, jd_attos, on)
        }
    }

    /// Builds a **TAI** [`Dt`] from a floating-point Julian Date.
    ///
    /// ## Important
    ///
    /// - The `on` scale becomes this [`Dt`]'s `target`; its `scale` is always `TAI`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// let dt = Dt::from_jd_f(2_460_782.25, Scale::TAI);
    /// // 2_460_782 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_jd(), (2_460_782, ATTOS_PER_DAY / 4));
    ///
    /// let dt = Dt::from_jd_f(-1_000.25, Scale::TAI);
    /// // -1_000 and -0.25 days in attoseconds
    /// assert_eq!(dt.to_jd(), (-1_000, -ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    /// - [`Dt::from_jd_floor`](../struct.Dt.html#method.from_jd_floor)
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    pub const fn from_jd_f(jd: Real, on: Scale) -> Dt {
        let jd_days_f = floor_f(jd);
        let jd_days = jd_days_f as i128;

        let mut frac_day = jd - jd_days_f;
        if frac_day < 0.0 {
            frac_day = 0.0;
        } else if frac_day >= 1.0 {
            frac_day = 1.0 - f64::EPSILON;
        }

        let total_sec_f = frac_day * 86_400.0;
        let whole_sec = floor_f(total_sec_f) as i64;
        let frac_sec = total_sec_f - (whole_sec as Real);

        let attos_whole: i128 = (whole_sec as i128).saturating_mul(ATTOS_PER_SEC_I128);

        let attos_frac_f = frac_sec * 1_000_000_000_000_000_000.0;
        let attos_frac: i128 = floor_f(attos_frac_f + 0.5) as i128;

        let mut total_attos: i128 = attos_whole.saturating_add(attos_frac);

        let mut extra_days: i128 = 0;
        if total_attos >= ATTOS_PER_DAY {
            extra_days = 1;
            total_attos = total_attos.saturating_sub(ATTOS_PER_DAY);
        } else if total_attos < 0 {
            extra_days = -1;
            total_attos = total_attos.saturating_add(ATTOS_PER_DAY);
        }

        let final_jd_days = jd_days.saturating_add(extra_days);
        let frac_attos = total_attos as u128;

        Self::from_jd_floor(final_jd_days, frac_attos, on)
    }

    /// Builds a **TAI** [`Dt`] from a floating-point Modified Julian Date.
    ///
    /// ## Important
    ///
    /// This converts from the `on` time scale to `TAI` so for example, an
    /// `on` scale of `Scale::UTC` will add leap seconds to the end result.
    ///
    /// To avoid a time scale conversion use `Scale::TAI` for the `on` argument.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] counting attoseconds since the library epoch
    /// [`Dt::ZERO`](../constant.ZERO.html) with its `scale` field set to
    /// `TAI` and its `target` field set to the `on` arg.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY};
    ///
    /// let dt = Dt::from_mjd_f(60_961.25, Scale::TAI);
    /// // 60_961 and 0.25 days in attoseconds
    /// assert_eq!(dt.to_mjd(), (60_961, ATTOS_PER_DAY / 4));
    ///
    /// let dt = Dt::from_mjd_f(-1_000.25, Scale::TAI);
    /// // -1_001 and effectively 0.75 days in attoseconds
    /// assert_eq!(dt.to_mjd(), (-1_001, 3 * ATTOS_PER_DAY / 4));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    /// - [`Dt::from_mjd_floor`](../struct.Dt.html#method.from_mjd_floor)
    /// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
    /// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
    #[inline]
    pub const fn from_mjd_f(mjd: Real, on: Scale) -> Dt {
        let jd = mjd + f!(2_400_000.5);
        Self::from_jd_f(jd, on)
    }
}
