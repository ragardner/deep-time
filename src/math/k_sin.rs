// origin: FreeBSD /usr/src/lib/msun/src/k_sin.c
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunSoft, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================
//
// Made const fn friendly

use crate::Real;

const S1: Real = -1.66666666666666324348e-01; /* 0xBFC55555, 0x55555549 */
const S2: Real = 8.33333333332248946124e-03; /* 0x3F811111, 0x1110F8A6 */
const S3: Real = -1.98412698298579493134e-04; /* 0xBF2A01A0, 0x19C161D5 */
const S4: Real = 2.75573137070700676789e-06; /* 0x3EC71DE3, 0x57B1FE7D */
const S5: Real = -2.50507602534068634195e-08; /* 0xBE5AE5E6, 0x8A2B9CEB */
const S6: Real = 1.58969099521155010221e-10; /* 0x3DE5D93A, 0x5ACFD57C */

/// Kernel sin function on ~[-pi/4, pi/4] (except on -0).
///
/// Made `const fn` compatible.
pub(crate) const fn k_sin(x: Real, y: Real, iy: i32) -> Real {
    let z = x * x;
    let w = z * z;
    let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let v = z * x;

    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}
