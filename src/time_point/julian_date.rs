use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOSEC_PER_SEC_I128, ClockType, J2000_JD_TT, MJD_1970,
    Real, SEC_PER_DAYI64, TimePoint,
};

impl TimePoint {
    /// Returns `(integer_days, fractional_attoseconds)`.
    /// The fractional part is always in `[0, ATTOS_PER_DAY)`.
    pub const fn to_jd_exact(self) -> (i64, u128) {
        match self.clock_type {
            ClockType::UTC | ClockType::UTCSofa | ClockType::UTCSpice => {
                let canon_attos = self.to_attos_since(TimePoint::UNIX_EPOCH_UTC);
                let total_attos = canon_attos + ATTOS_PER_HALF_DAY;

                let days_since_1970 = total_attos.div_euclid(ATTOS_PER_DAY);
                let frac_attos = total_attos.rem_euclid(ATTOS_PER_DAY) as u128;

                let jd_int = 2_440_587i64 + (days_since_1970 as i64);
                (jd_int, frac_attos)
            }
            _ => {
                let days_since_j2000 = self.sec.div_euclid(SEC_PER_DAYI64);
                let remaining_sec = self.sec.rem_euclid(SEC_PER_DAYI64);
                let frac_attos =
                    (remaining_sec as u128) * ATTOSEC_PER_SEC_I128 as u128 + (self.subsec as u128);

                (J2000_JD_TT + days_since_j2000, frac_attos)
            }
        }
    }

    #[inline]
    pub const fn to_jd(self) -> Real {
        let (days, attos) = self.to_jd_exact();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    pub const fn to_mjd_exact(self) -> (i64, u128) {
        match self.clock_type {
            ClockType::UTC | ClockType::UTCSofa | ClockType::UTCSpice => {
                let canon_attos = self.to_attos_since(TimePoint::UNIX_EPOCH_UTC);
                let days_since_1970 = canon_attos.div_euclid(ATTOS_PER_DAY);
                let frac_attos = canon_attos.rem_euclid(ATTOS_PER_DAY) as u128;

                (MJD_1970 + (days_since_1970 as i64), frac_attos)
            }
            _ => {
                let (jd_days, frac_attos) = self.to_jd_exact();

                let mjd_days = jd_days - 2_400_001;
                let mjd_attos = frac_attos + ATTOS_PER_HALF_DAY as u128;

                if mjd_attos >= ATTOS_PER_DAY as u128 {
                    (mjd_days + 1, mjd_attos - ATTOS_PER_DAY as u128)
                } else {
                    (mjd_days, mjd_attos)
                }
            }
        }
    }

    #[inline]
    pub const fn to_mjd(self) -> Real {
        let (days, attos) = self.to_mjd_exact();
        f!(days) + f!(attos) / f!(ATTOS_PER_DAY)
    }

    pub const fn from_jd_exact(jd_days: i64, frac_attos: u128, orig_type: ClockType) -> Self {
        match orig_type {
            ClockType::UTC | ClockType::UTCSofa | ClockType::UTCSpice => {
                let canon_attos = (jd_days as i128 - 2_440_587i128) * ATTOS_PER_DAY
                    + (frac_attos as i128)
                    - ATTOS_PER_HALF_DAY;

                Self::from_to_attos_since(canon_attos, TimePoint::UNIX_EPOCH_UTC)
                    .with_type(orig_type)
            }
            _ => {
                let days_since_j2000 = jd_days - J2000_JD_TT;
                let total_sec = days_since_j2000 * SEC_PER_DAYI64
                    + (frac_attos / ATTOSEC_PER_SEC_I128 as u128) as i64;
                let subsec = (frac_attos % ATTOSEC_PER_SEC_I128 as u128) as u64;

                let point = TimePoint::new(total_sec, subsec, ClockType::TT);
                point.to_type(orig_type)
            }
        }
    }

    pub const fn from_mjd_exact(mjd_days: i64, frac_attos: u128, orig_type: ClockType) -> Self {
        let jd_days = mjd_days + 2_400_000;
        let jd_attos = frac_attos + ATTOS_PER_HALF_DAY as u128;

        if jd_attos >= ATTOS_PER_DAY as u128 {
            Self::from_jd_exact(jd_days + 1, jd_attos - ATTOS_PER_DAY as u128, orig_type)
        } else {
            Self::from_jd_exact(jd_days, jd_attos, orig_type)
        }
    }
}
