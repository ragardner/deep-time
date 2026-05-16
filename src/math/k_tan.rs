// origin: FreeBSD /usr/src/lib/msun/src/k_tan.c
//
// ====================================================
// Copyright 2004 Sun Microsystems, Inc.  All Rights Reserved.
//
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================

#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::Real;

static T: [Real; 13] = [
    Real::from_bits(0x3fd5555555555563),
    Real::from_bits(0x3fc111111110fe7a),
    Real::from_bits(0x3faba1ba1bb341fe),
    Real::from_bits(0x3f9664f48406d637),
    Real::from_bits(0x3f8226e3e96e8493),
    Real::from_bits(0x3f6d6d22c9560328),
    Real::from_bits(0x3f57dbc8fee08315),
    Real::from_bits(0x3f4344d8f2f26501),
    Real::from_bits(0x3f3026f71a8d1068),
    Real::from_bits(0x3f147e88a03792a6),
    Real::from_bits(0x3f12b80f32f0a7e9),
    Real::from_bits(0xbef375cbdb605373),
    Real::from_bits(0x3efb2a7074bf7ad4),
];

const ONE: Real = Real::from_bits(0x3ff0000000000000);
const TWO: Real = Real::from_bits(0x4000000000000000);
const PIO4: Real = Real::from_bits(0x3fe921fb54442d18);
const PIO4_LO: Real = Real::from_bits(0x3c81a62633145c07);

/// Exact port of FreeBSD msun `k_tan` using the **original 0/1 calling convention**
/// that your `s_tan.c` (and the top-level `tan`) expects.
///
/// `odd == 0` → return tan(x+y)
/// `odd == 1` → return -1/tan(x+y)
pub(crate) const fn k_tan(mut x: Real, mut y: Real, odd: i32) -> Real {
    let hx = (Real::to_bits(x) >> 32) as u32;
    let big = (hx & 0x7fffffff) >= 0x3fe59428; /* |x| >= 0.6744 */

    if big {
        let sign = hx >> 31;
        if sign != 0 {
            x = -x;
            y = -y;
        }
        x = (PIO4 - x) + (PIO4_LO - y);
        y = Real::from_bits(0);
    }

    let z = x * x;
    let w = z * z;

    let r = T[1] + w * (T[3] + w * (T[5] + w * (T[7] + w * (T[9] + w * T[11]))));
    let v = z * (T[2] + w * (T[4] + w * (T[6] + w * (T[8] + w * (T[10] + w * T[12])))));

    let s = z * x;
    let r = y + z * (s * (r + v) + y) + s * T[0];
    let w = x + r;

    if big {
        let sign = hx >> 31; // ORIGINAL sign (important!)

        // This is the exact line from the original FreeBSD source.
        // We avoid any From<i64> by using a simple if (odd is only 0 or 1).
        let s_val = if odd == 0 { ONE } else { ONE - TWO };

        let tmp = r - (w * w / (w + s_val));
        let v = s_val - TWO * (x + tmp);

        return if sign != 0 { -v } else { v };
    }

    if odd == 0 {
        return w;
    }

    /* -1.0/(x+r) with extra precision */
    let w0 = zero_low_word(w);
    let v = r - (w0 - x);
    let a = -ONE / w;
    let a0 = zero_low_word(a);
    a0 + a * (ONE + a0 * w0 + a0 * v)
}

#[inline]
const fn zero_low_word(x: Real) -> Real {
    Real::from_bits(Real::to_bits(x) & 0xffff_ffff_0000_0000)
}
