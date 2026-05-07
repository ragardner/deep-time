use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOS_PER_SEC_I128, Dt, J2000_JD_TT, JD_EPOCH_DAYS,
    MJD_1970, Real, SEC_PER_DAYI64, Scale, TSpan,
};

impl Dt {
    /// Returns the exact Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// ### Behavior by `scale`
    ///
    /// - **`UTC`, `UTCSofa`, `UTCSpice`**: Computes **JD(UTC)** using the Unix epoch
    ///   (1970-01-01 00:00:00 UTC) as reference. This produces the Julian Date that
    ///   corresponds to the civil UTC clock reading (the value used by IERS C04 tables,
    ///   most astronomy software, and online JD calculators).
    ///
    /// - **All other types** (TAI, TT, TDB, GPS, TCG, etc.): Computes **JD(TT)** (or
    ///   equivalent uniform scale) using the J2000.0 TT epoch (`J2000_JD_TT = 2451545`).
    ///   This is the continuous, leap-second-free value used for ephemerides and
    ///   dynamical calculations.
    ///
    /// The returned value therefore depends on both the physical instant *and* the
    /// declared time scale of `self`.
    ///
    /// # Precision
    /// Exact (attosecond resolution). Use [`to_jd`](Self::to_jd) for the floating-point
    /// version.
    pub const fn to_jd_exact(self, target: Scale) -> (i64, u128) {
        if target.is_ut() {
            let canon_attos = self.to_tai_attos_since(Dt::UNIX_EPOCH);
            let total_attos = canon_attos + ATTOS_PER_HALF_DAY;

            let days_since_1970 = total_attos.div_euclid(ATTOS_PER_DAY);
            let frac_attos = total_attos.rem_euclid(ATTOS_PER_DAY) as u128;

            let jd_int = 2_440_587i64 + days_since_1970 as i64;
            (jd_int, frac_attos)
        } else {
            let TSpan { sec, attos } = self.to(target);
            let days_since_j2000 = sec.div_euclid(SEC_PER_DAYI64);
            let remaining_sec = sec.rem_euclid(SEC_PER_DAYI64);
            let frac_attos = (remaining_sec as u128) * ATTOS_PER_SEC_I128 as u128 + (attos as u128);

            (J2000_JD_TT + days_since_j2000, frac_attos)
        }
    }

    /// Returns the Julian Date of this instant as a floating-point `Real` (`f64`).
    ///
    /// This is the lossy counterpart to [`to_jd_exact`](Self::to_jd_exact).
    /// See that method for the exact scale-dependent behavior (JD(UTC) vs JD(TT)).
    #[inline]
    pub const fn to_jd(self, target: Scale) -> Real {
        let (days, attos) = self.to_jd_exact(target);
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Returns the exact Modified Julian Date of this instant as `(integer_days, fractional_attoseconds)`.
    ///
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    ///
    /// ### Behavior by `scale`
    ///
    /// - **`UTC`, `UTCSofa`, `UTCSpice`**: Computes **MJD(UTC)** using the Unix epoch
    ///   (1970-01-01 00:00:00 UTC). This matches the MJD column in IERS C04 / Bulletin A
    ///   tables (0h UTC epochs) and most civil/UTC-labeled data products.
    ///
    /// - **All other types**: Computes the MJD equivalent of the uniform-scale JD
    ///   (normally JD(TT) – 2_400_000.5) with proper half-day adjustment.
    ///
    /// # Precision
    /// Exact (attosecond resolution). Use [`to_mjd`](Self::to_mjd) for the floating-point version.
    pub const fn to_mjd_exact(self, target: Scale) -> (i64, u128) {
        if target.is_ut() {
            let canon_attos = self.to_tai_attos_since(Dt::UNIX_EPOCH);
            let days_since_1970 = canon_attos.div_euclid(ATTOS_PER_DAY);
            let frac_attos = canon_attos.rem_euclid(ATTOS_PER_DAY) as u128;

            (MJD_1970 + (days_since_1970 as i64), frac_attos)
        } else {
            let (jd_days, frac_attos) = self.to_jd_exact(target);

            let mjd_days = jd_days - 2_400_001;
            let mjd_attos = frac_attos + ATTOS_PER_HALF_DAY as u128;

            if mjd_attos >= ATTOS_PER_DAY as u128 {
                (mjd_days + 1, mjd_attos - ATTOS_PER_DAY as u128)
            } else {
                (mjd_days, mjd_attos)
            }
        }
    }

    /// Returns the Modified Julian Date of this instant as a floating-point `Real` (`f64`).
    ///
    /// This is the lossy counterpart to [`to_mjd_exact`](Self::to_mjd_exact).
    /// See that method for the exact scale-dependent behavior (MJD(UTC) vs uniform MJD).
    #[inline]
    pub const fn to_mjd(self, target: Scale) -> Real {
        let (days, attos) = self.to_mjd_exact(target);
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    /// Creates a `Dt` from an exact Julian Date, interpreting the JD in the
    /// scale indicated by `orig_type`.
    ///
    /// - If `orig_type` is `UTC` / `UTCSofa` / `UTCSpice`, the input JD is treated as
    ///   **JD(UTC)** and the resulting `Dt` will have the corresponding UTC
    ///   civil time (leap-second aware).
    /// - For all other types the input JD is treated as the uniform-scale JD
    ///   (normally JD(TT)) and the resulting `Dt` is constructed on that scale.
    ///
    /// The returned `Dt` represents the physical instant whose JD (in the
    /// requested scale) matches the input.
    ///
    /// # Precision
    /// Exact (attosecond resolution).
    pub const fn from_jd_exact(jd_days: i64, frac_attos: u128, orig_type: Scale) -> Self {
        if orig_type.is_ut() {
            let canon_attos = (jd_days as i128 - JD_EPOCH_DAYS) * ATTOS_PER_DAY
                + (frac_attos as i128)
                - ATTOS_PER_HALF_DAY;

            Self::from_tai_attos_since(canon_attos, Dt::UNIX_EPOCH)
        } else {
            let days_since_j2000 = jd_days - J2000_JD_TT;
            let total_sec = days_since_j2000 * SEC_PER_DAYI64
                + (frac_attos / ATTOS_PER_SEC_I128 as u128) as i64;
            let attos = (frac_attos % ATTOS_PER_SEC_I128 as u128) as u64;

            Dt::from(total_sec, attos, orig_type)
        }
    }

    /// Creates a `Dt` from an exact Modified Julian Date, interpreting the MJD
    /// in the scale indicated by `orig_type`.
    ///
    /// This is the inverse of [`to_mjd_exact`](Self::to_mjd_exact). See that method
    /// and [`from_jd_exact`](Self::from_jd_exact) for scale-specific behavior.
    ///
    /// # Precision
    /// Exact (attosecond resolution).
    pub const fn from_mjd_exact(mjd_days: i64, frac_attos: u128, orig_type: Scale) -> Self {
        let jd_days = mjd_days + 2_400_000;
        let jd_attos = frac_attos + ATTOS_PER_HALF_DAY as u128;

        if jd_attos >= ATTOS_PER_DAY as u128 {
            Self::from_jd_exact(jd_days + 1, jd_attos - ATTOS_PER_DAY as u128, orig_type)
        } else {
            Self::from_jd_exact(jd_days, jd_attos, orig_type)
        }
    }
}
