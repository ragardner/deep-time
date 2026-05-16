// origin: FreeBSD /usr/src/lib/msun/src/s_tan.c
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunPro, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================

use super::{k_tan, rem_pio2};
use crate::Real;

pub const fn tan(x: Real) -> Real {
    let ix = (Real::to_bits(x) >> 32) as u32 & 0x7fffffff;

    /* |x| ~< pi/4 */
    if ix <= 0x3fe921fb {
        if ix < 0x3e400000 {
            /* |x| < 2**-27 */
            return x;
        }
        return k_tan(x, Real::from_bits(0), 0); // ← must be 0 (not 1)
    }

    /* tan(Inf or NaN) is NaN */
    if ix >= 0x7ff00000 {
        return x - x;
    }

    /* argument reduction */
    let (n, y0, y1) = rem_pio2(x);
    k_tan(y0, y1, n & 1)
}

#[cfg(all(test, feature = "std"))]
mod tan_tests {
    use super::tan;
    use std::f64::consts::PI;

    const MAX_ULP: u64 = 1;

    /// Returns the ULP (unit in the last place) difference between two `f64` values.
    /// Correctly handles NaNs, infinities, signed zeros, and sign-bit mismatches.
    fn ulp_diff(a: f64, b: f64) -> u64 {
        if a.is_nan() && b.is_nan() {
            return 0;
        }
        if a.is_infinite() || b.is_infinite() {
            return if a == b { 0 } else { u64::MAX };
        }

        let a_bits = a.to_bits();
        let b_bits = b.to_bits();

        // +0.0 and -0.0 (and any zero representation) are considered identical.
        if (a_bits | b_bits) & 0x7fff_ffff_ffff_ffff == 0 {
            return 0;
        }

        // Non-zero values with different signs → catastrophic difference.
        if (a_bits ^ b_bits) & 0x8000_0000_0000_0000 != 0 {
            return u64::MAX;
        }

        a_bits.abs_diff(b_bits)
    }

    fn check(x: f64) {
        let expected = x.tan();
        let actual = tan(x);

        if expected.is_nan() {
            assert!(actual.is_nan(), "tan({x}) should be NaN, got {actual}");
            return;
        }

        if expected.is_infinite() {
            assert!(
                actual.is_infinite(),
                "tan({x}) should be infinite (expected {expected}, got {actual})"
            );
            assert_eq!(
                actual.is_sign_positive(),
                expected.is_sign_positive(),
                "tan({x}) has wrong sign of infinity"
            );
            return;
        }

        let ulps = ulp_diff(actual, expected);
        assert!(
            ulps <= MAX_ULP,
            "tan({x}) failed: expected = {expected:.17e}, got = {actual:.17e}, ULP diff = {ulps} (max allowed {MAX_ULP})"
        );
    }

    #[test]
    fn special_values() {
        let cases = [
            0.0,
            -0.0,
            PI / 4.0,
            -PI / 4.0,
            PI / 2.0,
            -PI / 2.0,
            PI,
            -PI,
            3.0 * PI / 2.0,
            -3.0 * PI / 2.0,
            2.0 * PI,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MIN,
            f64::MAX,
            f64::MIN_POSITIVE,
        ];
        for &x in &cases {
            check(x);
        }
    }

    #[test]
    fn tiny_arguments_exact() {
        // The implementation has an explicit fast-path: |x| < 2^-27 returns x exactly.
        // This test validates that optimization (critical for const fn and performance).
        let threshold = f64::from_bits(0x3e400000u64 << 32); // 2^-27
        let step = threshold * 1e-4;
        let mut x = -threshold * 0.999;
        while x < threshold * 0.999 {
            assert_eq!(tan(x), x, "tiny fast-path failed at {x}");
            x += step;
        }
    }

    #[test]
    fn odd_symmetry() {
        // tan(-x) == -tan(x) must hold exactly (including for huge magnitudes)
        for i in -500..=500 {
            let x = (i as f64) * 0.013;
            let tx = tan(x);
            assert_eq!(tan(-x), -tx, "odd symmetry failed at {x}");

            let x_large = x * 1e10;
            assert_eq!(
                tan(-x_large),
                -tan(x_large),
                "large odd symmetry failed at {x_large}"
            );
        }
    }

    #[test]
    fn small_arguments() {
        // |x| ≤ π/4 → uses the direct `k_tan` fast path
        let bound = PI / 4.0;
        for i in -12000..=12000 {
            let x = (i as f64 / 12000.0) * bound;
            check(x);
        }
    }

    #[test]
    fn approaching_poles() {
        // The most numerically challenging region for tan: vertical asymptotes at (2k+1)π/2.
        // We approach from both sides with progressively smaller deltas.
        for k in -40..=40 {
            let pole = (2 * k + 1) as f64 * PI / 2.0;

            // From the left
            for i in 1..=400 {
                let delta = 1e-8 * (0.9999f64).powi(i as i32);
                check(pole - delta);
            }
            // From the right
            for i in 1..=400 {
                let delta = 1e-8 * (0.9999f64).powi(i as i32);
                check(pole + delta);
            }
        }
    }

    #[test]
    fn medium_arguments() {
        // Argument reduction around multiples of π
        for k in 0..=50 {
            let base = (k as f64) * PI;
            for i in -250..=250 {
                let x = base + (i as f64) * 0.0123;
                check(x);
            }
        }
    }

    #[test]
    fn large_arguments() {
        // Heavy stress on `rem_pio2` for very large magnitudes (up to ~1e22)
        let mut x = 1e6_f64;
        while x < 1e22 {
            check(x);
            check(x + 0.23456789);
            check(-x);
            x *= 3.1415926535;
        }
    }

    #[test]
    fn subnormal_arguments() {
        // Extremely tiny values (underflow territory)
        let mut x = f64::MIN_POSITIVE;
        for _ in 0..100 {
            check(x);
            check(-x);
            x /= 2.0;
        }
    }

    #[test]
    fn randomish_tests() {
        // Deterministic pseudo-random walk providing broad coverage
        let mut x: f64 = 1.23456789;
        for _ in 0..35_000 {
            check(x);
            x = x * 1.618033988749895 + 0.5772156649015329; // φ + Euler's γ
            if x.abs() > 1e15 {
                x = x.fract() * 75.0;
            }
        }
    }

    #[test]
    fn const_compatibility() {
        // Verify that the `const fn` works in a const context and produces correct values.
        const T0: f64 = tan(0.0);
        const T_PI4: f64 = tan(PI / 4.0);
        const T_PI: f64 = tan(PI);
        const T_SMALL: f64 = tan(1e-12);

        assert_eq!(T0, 0.0, "tan(0.0) must be exactly 0.0");
        assert!(
            (T_PI4 - 1.0).abs() < 1e-14,
            "tan(π/4) should be very close to 1.0"
        );
        assert!((T_PI).abs() < 1e-14, "tan(k·π) should be very close to 0.0");
        assert!(
            (T_SMALL - 1e-12).abs() < 1e-25,
            "tiny values must be accurate in const context"
        );
    }
}
