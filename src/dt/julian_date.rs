use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOS_PER_SEC_I128, Dt, JD_2000_2_451_545, Real, Scale,
    floor_f,
};

impl Dt {
    /// Returns the exact Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// - The returned JD is expressed in the time scale of this [`Dt`].
    /// - The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// For a float value use [`Self::to_jd_f`].
    pub const fn to_jd(&self) -> (i64, u128) {
        let days_since_j2000 = self.to_attos().div_euclid(ATTOS_PER_DAY);
        let remaining_attos = self.to_attos().rem_euclid(ATTOS_PER_DAY);

        let jd_int = JD_2000_2_451_545.saturating_add(days_since_j2000 as i64);
        (jd_int, remaining_attos as u128)
    }

    /// Returns the Julian Date of this instant as a floating-point `Real`.
    ///
    /// This is the lossy counterpart to [`Self::to_jd`].
    #[inline]
    pub const fn to_jd_f(&self) -> Real {
        let (days, attos) = self.to_jd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Returns the exact Modified Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// - The returned MJD is expressed in the time scale of this [`Dt`].
    /// - The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// For a float value use [`Self::to_mjd_f`].
    pub const fn to_mjd(&self) -> (i64, u128) {
        let (jd_days, frac_attos) = self.to_jd();

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
    /// This is the lossy counterpart to [`Self::to_mjd`].
    #[inline]
    pub const fn to_mjd_f(&self) -> Real {
        let (days, attos) = self.to_mjd();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Creates a `Dt` from an exact Julian Date.
    ///
    /// This is the inverse of [`Self::to_jd`]. For correct round-tripping you must
    /// pass the same `on: Scale` that matches the scale of the original [`Dt`].
    pub const fn from_jd(jd_days: i64, frac_attos: u128, on: Scale) -> Self {
        let days_since_j2000 = jd_days.saturating_sub(JD_2000_2_451_545);
        let frac_attos_i128 = if frac_attos > i128::MAX as u128 {
            i128::MAX
        } else {
            frac_attos as i128
        };
        let attos_from_days = (days_since_j2000 as i128).saturating_mul(ATTOS_PER_DAY);
        let total_attos = attos_from_days.saturating_add(frac_attos_i128);

        Self::from(total_attos, on)
    }

    /// Creates a `Dt` from an exact Modified Julian Date.
    ///
    /// This is the inverse of [`Self::to_mjd`]. For correct round-tripping you must
    /// pass the same `on: Scale` that matches the scale of the original [`Dt`].
    pub const fn from_mjd(mjd_days: i64, frac_attos: u128, on: Scale) -> Self {
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

    /// Creates a `Dt` from a float Julian Date.
    ///
    /// This is the inverse of [`Self::to_jd_f`]. For correct round-tripping you must
    /// pass the same `on: Scale` that matches the scale of the original [`Dt`].
    pub const fn from_jd_f(jd: Real, on: Scale) -> Self {
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

    /// Creates a `Dt` from a float Modified Julian Date.
    ///
    /// This is the inverse of [`Self::to_mjd_f`]. For correct round-tripping you must
    /// pass the same `on: Scale` that matches the scale of the original [`Dt`].
    #[inline]
    pub const fn from_mjd_f(mjd: Real, on: Scale) -> Self {
        let jd = mjd + f!(2_400_000.5);
        Self::from_jd_f(jd, on)
    }
}
