#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::Real;

/// Scale `x` by `2^n` for Real.
pub(crate) const fn scalbn(mut x: Real, mut n: i32) -> Real {
    const EXP_BIAS: i32 = 1023;
    const EXP_MAX: i32 = 1023;
    const EXP_MIN: i32 = -1022;
    const SIG_BITS: i32 = 52;

    if n == 0 || x == 0.0 {
        return x;
    }

    // Large positive n
    if n > EXP_MAX {
        x *= Real::from_bits(0x7ff0000000000000); // +∞ as scaling factor
        n -= EXP_MAX;
        if n > EXP_MAX {
            x *= Real::from_bits(0x7ff0000000000000);
            n -= EXP_MAX;
            if n > EXP_MAX {
                n = EXP_MAX;
            }
        }
    }
    // Large negative n (scaling toward zero)
    else if n < EXP_MIN {
        // Prescale to avoid going subnormal too early
        let mul = Real::from_bits(0x0010000000000000 << 52); // roughly 2^(-1022+53)
        let add = -EXP_MIN - SIG_BITS;

        x *= mul;
        n += add;

        if n < EXP_MIN {
            x *= mul;
            n += add;
            if n < EXP_MIN {
                n = EXP_MIN;
            }
        }
    }

    // Final scaling step
    let exp = (EXP_BIAS + n) as u32;
    let scale = Real::from_bits((exp as u64) << 52);
    x * scale
}
