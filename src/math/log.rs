#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::Real;

const LN2_HI: Real = 6.93147180369123816490e-01; /* 3fe62e42 fee00000 */
const LN2_LO: Real = 1.90821492927058770002e-10; /* 3dea39ef 35793c76 */
const LG1: Real = 6.666666666666735130e-01; /* 3FE55555 55555593 */
const LG2: Real = 3.999999999940941908e-01; /* 3FD99999 9997FA04 */
const LG3: Real = 2.857142874366239149e-01; /* 3FD24924 94229359 */
const LG4: Real = 2.222219843214978396e-01; /* 3FCC71C5 1D8E78AF */
const LG5: Real = 1.818357216161805012e-01; /* 3FC74664 96CB03DE */
const LG6: Real = 1.531383769920937332e-01; /* 3FC39A09 D078C69F */
const LG7: Real = 1.479819860511658591e-01; /* 3FC2F112 DF3E5244 */

/// The natural logarithm of `x` (Real).
pub const fn log(mut x: Real) -> Real {
    let x1p54 = Real::from_bits(0x4350000000000000); // 0x1p54 === 2 ^ 54

    let mut ui = x.to_bits();
    let mut hx: u32 = (ui >> 32) as u32;
    let mut k: i32 = 0;

    if (hx < 0x00100000) || ((hx >> 31) != 0) {
        /* x < 2**-126  */
        if ui << 1 == 0 {
            return -1. / (x * x); /* log(+-0)=-inf */
        }
        if hx >> 31 != 0 {
            return (x - x) / 0.0; /* log(-#) = NaN */
        }
        /* subnormal number, scale x up */
        k -= 54;
        x *= x1p54;
        ui = x.to_bits();
        hx = (ui >> 32) as u32;
    } else if hx >= 0x7ff00000 {
        return x;
    } else if hx == 0x3ff00000 && ui << 32 == 0 {
        return 0.;
    }

    /* reduce x into [sqrt(2)/2, sqrt(2)] */
    hx += 0x3ff00000 - 0x3fe6a09e;
    k += ((hx >> 20) as i32) - 0x3ff;
    hx = (hx & 0x000fffff) + 0x3fe6a09e;
    ui = ((hx as u64) << 32) | (ui & 0xffffffff);
    x = Real::from_bits(ui);

    let f: Real = x - 1.0;
    let hfsq: Real = 0.5 * f * f;
    let s: Real = f / (2.0 + f);
    let z: Real = s * s;
    let w: Real = z * z;
    let t1: Real = w * (LG2 + w * (LG4 + w * LG6));
    let t2: Real = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    let r: Real = t2 + t1;
    let dk: Real = k as Real;
    s * (hfsq + r) + dk * LN2_LO - hfsq + f + dk * LN2_HI
}
