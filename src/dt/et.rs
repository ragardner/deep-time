use crate::{Dt, Real, Scale, TT_TAI_OFFSET, sin};

const M0: Real = f!(6.239996);
const M1: Real = f!(1.99096871e-7);
const EB: Real = f!(0.01671);
const K: Real = f!(0.001657);

impl Dt {
    /// NAIF/SPICE simplified ET–TT periodic correction (only the ~1.657 ms term).
    pub const fn et_minus_tt(seconds_since_j2000_tt: Real) -> Real {
        let m = M0 + M1 * seconds_since_j2000_tt;
        let e = m + EB * sin(m);
        K * sin(e)
    }

    /// Converts a TAI [`Dt`] to ET (NAIF/SPICE simplified model).
    pub const fn tai_to_et(&self) -> Dt {
        let tt = self.add(TT_TAI_OFFSET);
        let correction = Self::et_minus_tt(tt.to_sec_f());
        tt.add(Dt::from_sec_f(correction, Scale::TAI))
    }

    /// Converts an ET [`Dt`] to TAI (NAIF/SPICE simplified model).
    pub const fn et_to_tai(et: Dt) -> Dt {
        // Initial guess: subtract the constant TT–TAI offset
        let mut tt = et.sub(TT_TAI_OFFSET);

        // Fixed-point iteration: TT_{n+1} = ET − P(TT_n)
        let mut i = 0u8;
        while i < 8 {
            let p = Self::et_minus_tt(tt.to_sec_f());
            let new_tt = et.sub(Dt::from_sec_f(p, Scale::TAI));

            // Early exit when change is smaller than ~1 attosecond
            let delta = new_tt.to_diff_raw(tt);
            if delta.to_attos().abs() < 1 {
                tt = new_tt;
                break;
            }

            tt = new_tt;
            i += 1;
        }

        tt.sub(TT_TAI_OFFSET)
    }
}
