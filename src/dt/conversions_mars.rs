use crate::{
    Dt, MARS_REF_TT, MARS_REF_TT_ATTOS, MARS_SOL_ATTOS, MARS_SOL_LENGTH_SEC, Real, Scale, floor_f,
    to_sec_f,
};

impl Dt {
    /// Exact helper: elapsed attoseconds since the Mars MSD reference epoch (JD 2405522.0028779 TT).
    #[inline]
    pub(crate) const fn to_attos_since_mars_msd_epoch(numerical_tt: Dt) -> i128 {
        numerical_tt.to_attos() - MARS_REF_TT_ATTOS
    }

    /// Returns the exact Mars Sol Date (MSD) as a tuple of integer sols and the fractional part of a sol.
    ///
    /// The computation follows the canonical NASA GISS / AM2000 formulation and works for any input
    /// [`Scale`]. Leap seconds are automatically accounted for when converting from UTC.
    pub const fn to_msd_exact(&self, current: Scale) -> (i64, u128) {
        let tt = self.to(current, Scale::TT);
        let elapsed = Self::to_attos_since_mars_msd_epoch(tt);
        let whole_sols = elapsed.div_euclid(MARS_SOL_ATTOS);
        let frac_attos = elapsed.rem_euclid(MARS_SOL_ATTOS) as u128;

        (Dt::clamp_i128_to_i64(whole_sols), frac_attos)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`Dt`] representing
    /// seconds into the current sol (range `[0, one Martian sol)`).
    #[inline]
    pub const fn to_mtc(&self, current: Scale) -> Dt {
        let (_, frac_attos) = self.to_msd_exact(current);
        Dt::from_attos(frac_attos as i128, Scale::TAI)
    }

    /// Creates a `Dt` (in TT) from an exact Mars Sol Date using full library precision.
    pub const fn from_msd_exact(whole_sols: i64, frac_attos: u128) -> Self {
        let elapsed_attos = (whole_sols as i128) * MARS_SOL_ATTOS + frac_attos as i128;
        let tt = MARS_REF_TT.add(Dt::from_attos(elapsed_attos, Scale::TAI));
        Self::from(tt.sec, tt.attos, Scale::TT)
    }

    /// Creates a `Dt` (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    pub const fn from_msd(msd: Real) -> Self {
        let whole = floor_f(msd) as i64;
        let frac = msd - f!(whole);
        let frac_span = Dt::from_sec_f(frac * MARS_SOL_LENGTH_SEC);
        Self::from_msd_exact(whole, frac_span.to_attos() as u128)
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd(&self, current: Scale) -> Real {
        let (whole, frac) = self.to_msd_exact(current);
        f!(whole) + to_sec_f(frac) / MARS_SOL_LENGTH_SEC
    }
}
