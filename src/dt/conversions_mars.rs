use crate::{
    ATTOS_PER_SEC_I128, Dt, JD_2000_2_451_545, MARS_MSD_REF_JD_INT, MARS_MSD_REF_TOD_SEC,
    MARS_MSD_REF_TOD_SUBSEC, MARS_REF_TT, MARS_SOL_ATTOS, MARS_SOL_LENGTH_SEC, Real,
    SEC_PER_DAYI64, SEC_PER_DAYI128, Scale, TSpan, floor_f, to_sec_f,
};

impl Dt {
    /// Exact helper: elapsed attoseconds since the Mars MSD reference epoch (JD 2405522.0028779 TT).
    pub(crate) const fn elapsed_to_attos_since_mars_msd_epoch(numerical_tt: TSpan) -> i128 {
        let days_since_j2000 = numerical_tt.sec.div_euclid(SEC_PER_DAYI64);
        let tod_sec = numerical_tt.sec.rem_euclid(SEC_PER_DAYI64);

        let jd_days = JD_2000_2_451_545 + days_since_j2000;
        let days_diff = jd_days - MARS_MSD_REF_JD_INT;

        let mut sec_diff = (days_diff as i128) * SEC_PER_DAYI128
            + (tod_sec as i128 - MARS_MSD_REF_TOD_SEC as i128);
        let mut attos_diff = (numerical_tt.attos as i128) - (MARS_MSD_REF_TOD_SUBSEC as i128);

        if attos_diff < 0 {
            attos_diff += ATTOS_PER_SEC_I128;
            sec_diff -= 1;
        }

        sec_diff * ATTOS_PER_SEC_I128 + attos_diff
    }

    /// Returns the exact Mars Sol Date (MSD) as a tuple of integer sols and the fractional part of a sol.
    ///
    /// The computation follows the canonical NASA GISS / AM2000 formulation and works for any input
    /// [`Scale`]. Leap seconds are automatically accounted for when converting from UTC.
    pub const fn to_msd_exact(self) -> (i64, u128) {
        let tt = self.to(Scale::TT);
        let elapsed = Self::elapsed_to_attos_since_mars_msd_epoch(tt);
        let attos_per_sol = MARS_SOL_ATTOS;

        let whole_sols = elapsed.div_euclid(attos_per_sol) as i64;
        let frac_attos = elapsed.rem_euclid(attos_per_sol) as u128;

        (whole_sols, frac_attos)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`TSpan`] representing
    /// seconds into the current sol (range `[0, one Martian sol)`).
    #[inline]
    pub const fn to_mtc(self) -> TSpan {
        let (_, frac_attos) = self.to_msd_exact();
        TSpan::from_attos(frac_attos as i128)
    }

    /// Creates a `Dt` (in TT) from an exact Mars Sol Date using full library precision.
    pub const fn from_msd_exact(whole_sols: i64, frac_attos: u128) -> Self {
        let elapsed_attos = (whole_sols as i128) * MARS_SOL_ATTOS + frac_attos as i128;

        let tt = MARS_REF_TT.add(TSpan::from_attos(elapsed_attos));
        Self::from(tt.sec, tt.attos, Scale::TT)
    }

    /// Creates a `Dt` (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    pub const fn from_msd(msd: Real) -> Self {
        let whole = floor_f(msd) as i64;
        let frac = msd - f!(whole);
        let frac_span = TSpan::from_sec_f(frac * MARS_SOL_LENGTH_SEC);
        Self::from_msd_exact(whole, frac_span.to_attos() as u128)
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd(self) -> Real {
        let (whole, frac) = self.to_msd_exact();
        f!(whole) + to_sec_f(frac) / MARS_SOL_LENGTH_SEC
    }
}
