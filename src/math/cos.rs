// origin: FreeBSD /usr/src/lib/msun/src/s_cos.c */
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunPro, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================

#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use super::{k_cos, k_sin, rem_pio2};
use crate::Real;

/// Computes the cosine of `x` in radians (`cos(x)`).
///
/// Returns a value in the range `[-1.0, 1.0]`.
///
/// ## Special cases
///
/// - `cos(NaN)` returns `NaN`
/// - `cos(±∞)` returns `NaN`
/// - `cos(±0.0)` returns `1.0` (preserves the even property of cosine)
///
/// ## Implementation notes
///
/// This is a `const fn`-compatible port of the FreeBSD `libm` implementation
/// (`s_cos.c`). It first checks whether `|x| ≤ π/4` (fast path via `k_cos`),
/// handles infinities/NaNs, and otherwise performs argument reduction with
/// `rem_pio2` before dispatching to the appropriate kernel (`k_cos` or
/// `k_sin`) based on the quadrant.
///
/// ## Modifications for this crate
///
/// - Adapted to the generic `Real` type (which is `f64` under the hood)
/// - Made fully `const fn` compatible
/// - Uses the crate's own `rem_pio2`, `k_cos`, and `k_sin` implementations
///
/// ## Testing
///
/// The function is validated by a comprehensive test suite that includes:
/// - Special values (NaN, infinities, zeros)
/// - Symmetry (`cos(-x) == cos(x)`)
/// - Small arguments (fast path)
/// - Values near multiples of π/2 (critical accuracy region)
/// - Medium and very large arguments (argument reduction stress)
/// - Subnormal numbers
/// - Deterministic pseudo-random inputs
/// - Explicit `const` evaluation checks
///
/// All tests pass and produce results matching `std::f64::cos` within 1 ULP.
pub const fn cos(x: Real) -> Real {
    /* High word of x. */
    let ix = (Real::to_bits(x) >> 32) as u32 & 0x7fffffff;

    /* |x| ~< pi/4 */
    if ix <= 0x3fe921fb {
        return k_cos(x, Real::from_bits(0));
    }

    /* cos(Inf or NaN) is NaN */
    if ix >= 0x7ff00000 {
        return x - x;
    }

    /* argument reduction needed */
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => k_cos(y0, y1),
        1 => -k_sin(y0, y1, 1),
        2 => -k_cos(y0, y1),
        _ => k_sin(y0, y1, 1),
    }
}

#[cfg(all(test, feature = "std"))]
mod cos_tests {
    use super::cos;
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
        let expected = x.cos();
        let actual = cos(x);

        if expected.is_nan() {
            assert!(actual.is_nan(), "cos({x}) should be NaN, got {actual}");
            return;
        }

        // Any finite input must produce a result in [-1, 1].
        if x.is_finite() {
            assert!(
                (-1.0000001..=1.0000001).contains(&actual),
                "cos({x}) = {actual} is outside reasonable [-1, 1] range"
            );
        }

        let ulps = ulp_diff(actual, expected);
        assert!(
            ulps <= MAX_ULP,
            "cos({x}) failed: expected = {expected:.17e}, got = {actual:.17e}, ULP diff = {ulps} (max allowed {MAX_ULP})"
        );
    }

    #[test]
    fn special_values() {
        let cases = [
            0.0,
            -0.0,
            PI / 2.0,
            PI,
            3.0 * PI / 2.0,
            2.0 * PI,
            -PI / 2.0,
            -PI,
            -3.0 * PI / 2.0,
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
    fn symmetry() {
        // `cos(-x) == cos(x)` must hold exactly (including for huge magnitudes)
        for i in -400..=400 {
            let x = (i as f64) * 0.031415926535;
            assert_eq!(cos(-x), cos(x), "symmetry failed at {x}");

            let x_large = x * 1e10;
            assert_eq!(
                cos(-x_large),
                cos(x_large),
                "large symmetry failed at {x_large}"
            );
        }
    }

    #[test]
    fn small_arguments() {
        // Direct `k_cos` fast-path: |x| ≤ π/4
        let bound = PI / 4.0;
        for i in -10000..=10000 {
            let x = (i as f64 / 10000.0) * bound;
            check(x);
        }
    }

    #[test]
    fn near_critical_points() {
        // High-accuracy region around odd multiples of π/2 where cos(x) ≈ 0
        for k in -50..=50 {
            let base = (k as f64) * PI / 2.0;
            for i in -200..=200 {
                let x = base + (i as f64) * 1e-9;
                check(x);
            }
        }
    }

    #[test]
    fn medium_arguments() {
        // Argument reduction around multiples of π
        for k in 0..=50 {
            let base = (k as f64) * PI;
            for i in -200..=200 {
                let x = base + (i as f64) * 0.012345;
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
            check(x + 0.123456789);
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
        let mut x: f64 = 0.987654321;
        for _ in 0..30_000 {
            check(x);
            x = x * 1.618033988749895 + 0.2718281828459045; // φ + e
            if x.abs() > 1e16 {
                x = x.fract() * 100.0;
            }
        }
    }

    #[test]
    fn const_compatibility() {
        // Verify that the `const fn` works in a const context and produces sensible values.
        // This is the core guarantee we want from the implementation.
        const C0: f64 = cos(0.0);
        const C_PI: f64 = cos(PI);
        const C_PI2: f64 = cos(PI / 2.0);
        const C_PI4: f64 = cos(PI / 4.0);

        assert_eq!(C0, 1.0, "cos(0.0) must be exactly 1.0");
        assert_eq!(C_PI, -1.0, "cos(π) must be exactly -1.0");

        // cos(PI/2) is the only tricky case.
        // `std::f64::consts::PI` is *not* exact π, so the general argument reduction
        // (rem_pio2 + kernel) in const context produces a tiny non-zero remainder
        // (~6.12e-17). This is expected and perfectly acceptable — it is still
        // within ~1 ULP of the true mathematical result and within our MAX_ULP budget.
        assert!(
            C_PI2.abs() < 1e-14,
            "cos(π/2) = {C_PI2} should be very close to 0.0"
        );

        // Also cross-check against runtime std::cos for the more delicate cases
        assert!(
            (C_PI2 - (PI / 2.0).cos()).abs() < 1e-14,
            "const cos(π/2) differs too much from std::cos"
        );
        assert!(
            (C_PI4 - (PI / 4.0).cos()).abs() < 1e-14,
            "const cos(π/4) differs too much from std::cos"
        );
    }
}
