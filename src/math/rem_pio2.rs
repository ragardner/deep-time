#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

// origin: FreeBSD /usr/src/lib/msun/src/e_rem_pio2.c
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunPro, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================
//
// Optimized by Bruce D. Evans. */
use super::rem_pio2_large;
use crate::Real;

// #if FLT_EVAL_METHOD==0 || FLT_EVAL_METHOD==1
// #define EPS DBL_EPSILON
const EPS: Real = 2.2204460492503131e-16;
// #elif FLT_EVAL_METHOD==2
// #define EPS LDBL_EPSILON
// #endif

const TO_INT: Real = 1.5 / EPS;
/// 53 bits of 2/pi
const INV_PIO2: Real = 6.36619772367581382433e-01; /* 0x3FE45F30, 0x6DC9C883 */
/// first 33 bits of pi/2
const PIO2_1: Real = 1.57079632673412561417e+00; /* 0x3FF921FB, 0x54400000 */
/// pi/2 - PIO2_1
const PIO2_1T: Real = 6.07710050650619224932e-11; /* 0x3DD0B461, 0x1A626331 */
/// second 33 bits of pi/2
const PIO2_2: Real = 6.07710050630396597660e-11; /* 0x3DD0B461, 0x1A600000 */
/// pi/2 - (PIO2_1+PIO2_2)
const PIO2_2T: Real = 2.02226624879595063154e-21; /* 0x3BA3198A, 0x2E037073 */
/// third 33 bits of pi/2
const PIO2_3: Real = 2.02226624871116645580e-21; /* 0x3BA3198A, 0x2E000000 */
/// pi/2 - (PIO2_1+PIO2_2+PIO2_3)
const PIO2_3T: Real = 8.47842766036889956997e-32; /* 0x397B839A, 0x252049C1 */

// return the remainder of x rem pi/2 in y[0]+y[1]
// use rem_pio2_large() for large x
//
// caller must handle the case when reduction is not needed: |x| ~<= pi/4 */
pub(crate) const fn rem_pio2(x: Real) -> (i32, Real, Real) {
    let x1p24 = Real::from_bits(0x4170000000000000);

    let sign = (Real::to_bits(x) >> 63) as i32;
    let ix = (Real::to_bits(x) >> 32) as u32 & 0x7fffffff;

    const fn medium(x: Real, ix: u32) -> (i32, Real, Real) {
        let tmp = x * INV_PIO2 + TO_INT;
        let f_n = tmp - TO_INT;
        let n = f_n as i32;
        let mut r = x - f_n * PIO2_1;
        let mut w = f_n * PIO2_1T;
        let mut y0 = r - w;
        let ui = Real::to_bits(y0);
        let ey = (ui >> 52) as i32 & 0x7ff;
        let ex = (ix >> 20) as i32;

        if ex - ey > 16 {
            let t = r;
            w = f_n * PIO2_2;
            r = t - w;
            w = f_n * PIO2_2T - ((t - r) - w);
            y0 = r - w;
            let ey = (Real::to_bits(y0) >> 52) as i32 & 0x7ff;
            if ex - ey > 49 {
                let t = r;
                w = f_n * PIO2_3;
                r = t - w;
                w = f_n * PIO2_3T - ((t - r) - w);
                y0 = r - w;
            }
        }
        let y1 = (r - y0) - w;
        (n, y0, y1)
    }

    // Very small values are handled in sin/cos before calling rem_pio2

    if ix <= 0x400f6a7a {
        /* |x| ~<= 5π/4 */
        if (ix & 0xfffff) == 0x921fb {
            return medium(x, ix);
        }
        // Use medium() for better accuracy instead of single-round special cases
        return medium(x, ix);
    }

    if ix <= 0x401c463b {
        /* |x| ~<= 9π/4 */
        if ix == 0x4012d97c || ix == 0x401921fb {
            return medium(x, ix);
        }
        // Use medium() for better accuracy
        return medium(x, ix);
    }

    if ix < 0x413921fb {
        /* |x| ~< 2^20 * (π/2) */
        return medium(x, ix);
    }

    /* Large arguments */
    if ix >= 0x7ff00000 {
        let y0 = x - x;
        let y1 = y0;
        return (0, y0, y1);
    }

    /* Very large arguments -> use rem_pio2_large */
    let mut ui = Real::to_bits(x);
    ui &= (!1) >> 12;
    ui |= (0x3ff + 23) << 52;
    let mut z = Real::from_bits(ui);

    let mut tx = [0.0; 3];
    let mut i: usize = 0;
    while i < 2 {
        tx[i] = z as i32 as Real;
        z = (z - tx[i]) * x1p24;
        i += 1;
    }
    tx[2] = z;

    let mut i = 2;
    while i != 0 && tx[i] == 0.0 {
        i -= 1;
    }

    let mut ty = [0.0; 3];
    let n = rem_pio2_large(&tx, i + 1, &mut ty, ((ix as i32) >> 20) - (0x3ff + 23), 1);

    if sign != 0 {
        return (-n, -ty[0], -ty[1]);
    }
    (n, ty[0], ty[1])
}
