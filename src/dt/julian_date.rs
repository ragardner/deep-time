use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOS_PER_SEC_I128, Dt, JD_2000_2_451_545, Real, Scale,
    floor_f,
};

impl Dt {
    /// Returns the exact Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// This is the inverse of [`Dt::from_jd`](../struct.Dt.html#method.from_jd).
    ///
    /// ## Important:
    ///
    /// - This [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling this
    ///   function if you need the JD on a particular time scale (e.g. `Scale::TT` or
    ///   `Scale::TDB`).
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A `(days, attos)` pair where:
    ///
    /// - `days` (`i64`): integer part of the Julian Date on this [`Dt`]'s `target` scale.
    /// - `attos` (`u128`): fractional part in attoseconds since the start of that JD.
    ///   Always in the range `[0, ATTOS_PER_DAY)`.
    ///
    /// The returned JD is expressed in the time scale of this [`Dt`]'s `target` field.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    #[inline(always)]
    pub const fn to_jd(&self) -> (i64, u128) {
        self.to(self.target).to_jd_raw()
    }

    /// Returns the exact Julian Date of this instant as `(integer_days, fractional_attoseconds)`
    /// **without** converting to this [`Dt`]'s `target` scale.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// This is the low-level counterpart to [`Dt::to_jd`](../struct.Dt.html#method.to_jd).
    ///
    /// ## Important:
    ///
    /// - The JD is computed directly from this [`Dt`]'s current `attos` and `scale` fields.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A `(days, attos)` pair where:
    ///
    /// - `days` (`i64`): integer part of the Julian Date on this [`Dt`]'s **current** `scale`.
    /// - `attos` (`u128`): fractional part in attoseconds since the start of that JD.
    ///   Always in the range `[0, ATTOS_PER_DAY)`.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd_f_raw`](../struct.Dt.html#method.to_jd_f_raw)
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    #[inline(always)]
    pub const fn to_jd_raw(&self) -> (i64, u128) {
        let days_since_j2000 = self.to_attos().div_euclid(ATTOS_PER_DAY);
        let remaining_attos = self.to_attos().rem_euclid(ATTOS_PER_DAY);

        let jd_int = JD_2000_2_451_545.saturating_add(days_since_j2000 as i64);

        (jd_int, remaining_attos as u128)
    }

    /// Returns the Julian Date of this instant as a floating-point `Real`.
    ///
    /// This is the lossy counterpart to [`Dt::to_jd`](../struct.Dt.html#method.to_jd).
    ///
    /// ## Important:
    ///
    /// - This [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling this
    ///   function if you need the JD on a particular time scale (e.g. `Scale::TT` or
    ///   `Scale::TDB`).
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// The Julian Date as a `Real`, expressed in the time scale of this [`Dt`]'s `target` field.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::to_jd_f_raw`](../struct.Dt.html#method.to_jd_f_raw)
    #[inline]
    pub const fn to_jd_f(&self) -> Real {
        let (days, attos) = self.to_jd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Returns the Julian Date of this instant as a floating-point `Real`
    /// **without** converting to this [`Dt`]'s `target` scale.
    ///
    /// This is the low-level counterpart to [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f).
    ///
    /// ## Important:
    ///
    /// - The JD is computed directly from this [`Dt`]'s current `attos` and `scale` fields.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// The Julian Date as a `Real`, expressed in this [`Dt`]'s **current** `scale`.
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

    /// Returns the exact Modified Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// This is the inverse of [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd).
    ///
    /// ## Important:
    ///
    /// - This [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling this
    ///   function if you need the MJD on a particular time scale.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A `(days, attos)` pair where:
    ///
    /// - `days` (`i64`): integer part of the Modified Julian Date on this [`Dt`]'s `target` scale.
    /// - `attos` (`u128`): fractional part in attoseconds since the start of that MJD.
    ///
    /// The returned MJD is expressed in the time scale of this [`Dt`]'s `target` field.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f)
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    #[inline(always)]
    pub const fn to_mjd(&self) -> (i64, u128) {
        self.to(self.target).to_mjd_raw()
    }

    /// Returns the exact Modified Julian Date of this instant as `(integer_days, fractional_attoseconds)`
    /// **without** converting to this [`Dt`]'s `target` scale.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// This is the low-level counterpart to [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd).
    ///
    /// ## Important:
    ///
    /// - The MJD is computed directly from this [`Dt`]'s current `attos` and `scale` fields.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A `(days, attos)` pair where:
    ///
    /// - `days` (`i64`): integer part of the Modified Julian Date on this [`Dt`]'s **current** `scale`.
    /// - `attos` (`u128`): fractional part in attoseconds since the start of that MJD.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd_f_raw`](../struct.Dt.html#method.to_mjd_f_raw)
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    #[inline(always)]
    pub const fn to_mjd_raw(&self) -> (i64, u128) {
        let (jd_days, frac_attos) = self.to_jd_raw();

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

    /// Returns the Modified Julian Date of this instant as a floating-point `Real`.
    ///
    /// This is the lossy counterpart to [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd).
    ///
    /// ## Important:
    ///
    /// - This [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling this
    ///   function if you need the MJD on a particular time scale.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// The Modified Julian Date as a `Real`, expressed in the time scale of this [`Dt`]'s `target` field.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::to_mjd_f_raw`](../struct.Dt.html#method.to_mjd_f_raw)
    #[inline]
    pub const fn to_mjd_f(&self) -> Real {
        let (days, attos) = self.to_mjd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Returns the Modified Julian Date of this instant as a floating-point `Real`
    /// **without** converting to this [`Dt`]'s `target` scale.
    ///
    /// This is the low-level counterpart to [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f).
    ///
    /// ## Important:
    ///
    /// - The MJD is computed directly from this [`Dt`]'s current `attos` and `scale` fields.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch. If it is not, the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// The Modified Julian Date as a `Real`, expressed in this [`Dt`]'s **current** `scale`.
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

    /// Creates a **TAI** [`Dt`] from an exact Julian Date `(integer_days, fractional_attoseconds)`.
    ///
    /// This is the inverse of [`Dt::to_jd`](../struct.Dt.html#method.to_jd).
    ///
    /// ## Important:
    ///
    /// - The `on` parameter becomes the `target` of the returned [`Dt`].
    /// - The returned [`Dt`] always has `scale = TAI`.
    /// - Internally the input JD is interpreted on the `on` scale and then converted to TAI.
    /// - For correct round-tripping you must pass the same [`Scale`] that was used when
    ///   the original JD was produced (or the scale you want the resulting [`Dt`]'s `target` to be).
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] (its `scale` field is `TAI`). Its `target` field is set to `on`.
    /// The internal `attos` are relative to the library epoch (2000-01-01 noon TAI).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f)
    /// - [`Dt::to_jd`](../struct.Dt.html#method.to_jd)
    /// - [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd)
    pub const fn from_jd(jd_days: i64, frac_attos: u128, on: Scale) -> Dt {
        let days_since_j2000 = jd_days.saturating_sub(JD_2000_2_451_545);
        let frac_attos_i128 = if frac_attos > i128::MAX as u128 {
            i128::MAX
        } else {
            frac_attos as i128
        };
        let attos_from_days = (days_since_j2000 as i128).saturating_mul(ATTOS_PER_DAY);
        let total_attos = attos_from_days.saturating_add(frac_attos_i128);

        Self::from_attos(total_attos, on)
    }

    /// Creates a **TAI** [`Dt`] from an exact Modified Julian Date `(integer_days, fractional_attoseconds)`.
    ///
    /// This is the inverse of [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd).
    ///
    /// ## Important:
    ///
    /// - The `on` parameter becomes the `target` of the returned [`Dt`].
    /// - The returned [`Dt`] always has `scale = TAI`.
    /// - Internally the input MJD is interpreted on the `on` scale and then converted to TAI.
    /// - For correct round-tripping you must pass the same [`Scale`] that was used when
    ///   the original MJD was produced.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    /// - [`Dt::to_mjd`](../struct.Dt.html#method.to_mjd)
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    pub const fn from_mjd(mjd_days: i64, frac_attos: u128, on: Scale) -> Dt {
        let jd_days = mjd_days.saturating_add(2_400_000);
        let jd_attos = frac_attos.saturating_add(ATTOS_PER_HALF_DAY as u128);

        if jd_attos >= ATTOS_PER_DAY as u128 {
            Self::from_jd(
                jd_days.saturating_add(1),
                jd_attos.saturating_sub(ATTOS_PER_DAY as u128),
                on,
            )
        } else {
            Self::from_jd(jd_days, jd_attos, on)
        }
    }

    /// Creates a **TAI** [`Dt`] from a floating-point Julian Date.
    ///
    /// This is the inverse of [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f).
    ///
    /// ## Important:
    ///
    /// - The `on` parameter becomes the `target` of the returned [`Dt`].
    /// - The returned [`Dt`] always has `scale = TAI`.
    /// - Internally the input JD is interpreted on the `on` scale and then converted to TAI.
    /// - For correct round-tripping you must pass the same [`Scale`] that matches the
    ///   scale of the original JD.
    /// - Fractional days are handled with high precision (attosecond level).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_jd`](../struct.Dt.html#method.from_jd)
    /// - [`Dt::to_jd_f`](../struct.Dt.html#method.to_jd_f)
    /// - [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f)
    pub const fn from_jd_f(jd: Real, on: Scale) -> Dt {
        let jd_days_f = floor_f(jd);
        let jd_days = jd_days_f as i64;

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

        let mut extra_days: i64 = 0;
        if total_attos >= ATTOS_PER_DAY {
            extra_days = 1;
            total_attos = total_attos.saturating_sub(ATTOS_PER_DAY);
        } else if total_attos < 0 {
            extra_days = -1;
            total_attos = total_attos.saturating_add(ATTOS_PER_DAY);
        }

        let final_jd_days = jd_days.saturating_add(extra_days);
        let frac_attos = total_attos as u128;

        Self::from_jd(final_jd_days, frac_attos, on)
    }

    /// Creates a **TAI** [`Dt`] from a floating-point Modified Julian Date.
    ///
    /// This is the inverse of [`Dt::to_mjd_f`](../struct.Dt.html#method.to_mjd_f).
    ///
    /// ## Important:
    ///
    /// - The `on` parameter becomes the `target` of the returned [`Dt`].
    /// - The returned [`Dt`] always has `scale = TAI`.
    /// - Internally the input MJD is interpreted on the `on` scale and then converted to TAI.
    #[inline]
    pub const fn from_mjd_f(mjd: Real, on: Scale) -> Dt {
        let jd = mjd + f!(2_400_000.5);
        Self::from_jd_f(jd, on)
    }
}
