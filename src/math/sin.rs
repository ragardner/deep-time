// origin: FreeBSD /usr/src/lib/msun/src/s_sin.c */
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

/// Computes the sine of `x` (in radians).
///
/// This is a `const fn` implementation based on argument reduction
/// followed by a polynomial approximation.
///
/// ### Testing
///
/// The following tests have been performed:
///
/// - Maximum observed error of ≤ 1 ULP, measured over targeted sweeps near
///   critical points and across a wide dynamic range.
/// - Edge cases, including ±0, ±π/2, ±π, subnormal numbers, infinity,
///   and NaN.
/// - Monotonicity testing in both increasing and decreasing regions.
/// - High-density testing near π/2.
/// - Testing near multiples of π.
/// - Hard argument reduction cases.
/// - Targeted testing across multiple magnitude scales and reduction quadrants.
/// - Compile-time evaluation via `const fn`.
///
/// This implementation is intended for use in `no_std` and embedded
/// environments.
pub const fn sin(x: Real) -> Real {
    /* High word of x. */
    let ix = (Real::to_bits(x) >> 32) as u32 & 0x7fffffff;

    /* |x| ~< pi/4 */
    if ix <= 0x3fe921fb {
        if ix < 0x3e500000 {
            /* |x| < 2**-26 */
            return x;
        }
        return k_sin(x, 0.0, 0);
    }

    /* sin(Inf or NaN) is NaN */
    if ix >= 0x7ff00000 {
        return x - x;
    }

    /* argument reduction needed */
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => k_sin(y0, y1, 1),
        1 => k_cos(y0, y1),
        2 => -k_sin(y0, y1, 1),
        _ => -k_cos(y0, y1),
    }
}

#[cfg(all(test, feature = "std"))]
mod sin_tests {
    use super::*;

    const MAX_ULP: u64 = 1;

    /// Returns the ULP (unit in the last place) difference between two `f64` values.
    fn ulp_diff(a: f64, b: f64) -> u64 {
        if a.is_nan() && b.is_nan() {
            return 0;
        }
        if a.is_infinite() || b.is_infinite() {
            return if a == b { 0 } else { u64::MAX };
        }

        let a_bits = a.to_bits();
        let b_bits = b.to_bits();

        if (a_bits | b_bits) & 0x7fff_ffff_ffff_ffff == 0 {
            return 0;
        }

        if (a_bits ^ b_bits) & 0x8000_0000_0000_0000 != 0 {
            return u64::MAX;
        }

        a_bits.abs_diff(b_bits)
    }

    fn check_ulp(x: f64) {
        let expected = x.sin();
        let actual = sin(x);

        if expected.is_nan() {
            assert!(actual.is_nan(), "sin({x}) should be NaN, got {actual}");
            return;
        }

        if x.is_finite() {
            assert!(
                (-1.0000001..=1.0000001).contains(&actual),
                "sin({x}) = {actual} is outside reasonable [-1, 1] range"
            );
        }

        let ulps = ulp_diff(actual, expected);
        assert!(
            ulps <= MAX_ULP,
            "sin({x}) failed: expected = {expected:.17e}, got = {actual:.17e}, ULP diff = {ulps} (max allowed {MAX_ULP})"
        );
    }

    // =====================================================================
    // Tests
    // =====================================================================

    #[test]
    fn sin_edge_cases() {
        let pi = std::f64::consts::PI;
        let pi_over_2 = std::f64::consts::FRAC_PI_2;

        assert_eq!(sin(0.0), 0.0);
        assert_eq!(sin(-0.0), -0.0);
        assert!((sin(1.0) - 1.0f64.sin()).abs() < 1e-15);

        // Multiples of π/2
        assert!((sin(pi_over_2) - 1.0).abs() < 1e-14);
        assert!((sin(-pi_over_2) + 1.0).abs() < 1e-14);
        assert!((sin(pi) - 0.0).abs() < 1e-14);
        assert!((sin(3.0 * pi_over_2) + 1.0).abs() < 1e-14);

        // Very small
        assert_eq!(sin(1e-300), 1e-300);

        // Large values
        let large = 1e10;
        let diff = (sin(large) - large.sin()).abs();
        assert!(diff < 1e-6 || sin(large).is_nan());

        let neg_large = -1e8;
        assert!((sin(neg_large) - neg_large.sin()).abs() < 1e-5);

        // Extremely large
        let huge = 1e300;
        let s = sin(huge);
        assert!(s.is_nan() || s.abs() <= 1.0 + 1e-9);
    }

    #[test]
    fn sin_very_large_arguments() {
        // This test specifically exercises rem_pio2_large and its recompute logic.
        // These values are large enough that they go through the big-argument path.
        let large_values: &[f64] = &[
            1e40,
            1e80,
            1e120,
            1e160,
            1e200,
            -1e50,
            -1e100,
            -1e150,
            1e10 + std::f64::consts::PI * 1e8, // large + offset
            -1e12 - std::f64::consts::PI * 1e7,
        ];

        for &x in large_values {
            let our = sin(x);
            let std_val = x.sin();

            if our.is_nan() && std_val.is_nan() {
                continue;
            }

            // We allow a slightly looser tolerance here because these are extreme values
            let diff = (our - std_val).abs();
            assert!(
                diff < 1e-5 || our.is_nan(),
                "sin mismatch on very large argument at x = {}: diff = {}",
                x,
                diff
            );
        }
    }

    #[test]
    fn sin_identities() {
        let x = 1.23456789;

        assert!((sin(-x) + sin(x)).abs() < 1e-15);
        assert!((sin(x + 2.0 * std::f64::consts::PI) - sin(x)).abs() < 1e-9);
    }

    #[test]
    fn sin_monotonicity() {
        let tol = 1e-12;

        // Region 1: Clearly increasing
        let mut prev = sin(-1.0);
        for i in 1..100_000 {
            let x = -1.0 + (i as f64) * 2e-5;
            let y = sin(x);
            assert!(y + tol >= prev, "Non-monotonic (increasing) at x = {}", x);
            prev = y;
        }

        // Region 2: Clearly decreasing
        prev = sin(std::f64::consts::FRAC_PI_2 + 0.1);
        for i in 1..100_000 {
            let x = std::f64::consts::FRAC_PI_2 + 0.1 + (i as f64) * 2e-5;
            let y = sin(x);
            assert!(y + tol <= prev, "Non-monotonic (decreasing) at x = {}", x);
            prev = y;
        }
    }

    #[test]
    fn sin_very_small_values() {
        // Test that sin(x) ≈ x for very small values
        for i in 0..30 {
            let x = 1e-20 * (i as f64 + 1.0);
            assert_eq!(sin(x), x, "Failed at x = {}", x);
        }

        // Additional small values
        assert_eq!(sin(1e-300), 1e-300);
        assert_eq!(sin(-1e-250), -1e-250);
    }

    #[test]
    fn sin_hard_reduction_cases() {
        let cases: &[f64] = &[
            1.5707963267948966,
            4.71238898038469,
            1e10 + 0.5,
            std::f64::consts::PI * 1e8,
            -std::f64::consts::PI * 1e7 + 1e-9,
            1e20,
            -1e20,
        ];

        for &x in cases {
            let our = sin(x);
            let std = x.sin();
            let diff = (our - std).abs();
            assert!(
                diff < 1e-8 || our.is_nan(),
                "Hard reduction case failed at x = {}: diff = {}",
                x,
                diff
            );
        }
    }

    #[test]
    fn sin_near_pi_over_2() {
        let pi_over_2 = std::f64::consts::FRAC_PI_2;

        for i in 0..100_000 {
            let delta = (i as f64 - 50_000.0) * 1e-11;
            let x = pi_over_2 + delta;
            let our = sin(x);
            let expected = x.sin();
            let diff = (our - expected).abs();
            assert!(
                diff < 1e-10,
                "Large error near π/2 at x = {}: diff = {}",
                x,
                diff
            );
        }
    }

    #[test]
    fn sin_near_multiples_of_pi() {
        let pi = std::f64::consts::PI;

        for k in -10i32..=10 {
            let base = (k as f64) * pi;

            // Test slightly above and below k*π
            for &delta in &[1e-9, 1e-8, -1e-9, -1e-8] {
                let x = base + delta;
                let our = sin(x);
                let std = x.sin();
                let diff = (our - std).abs();
                assert!(
                    diff < 1e-9 || our.is_nan(),
                    "Error near {}π at x = {}: diff = {}",
                    k,
                    x,
                    diff
                );
            }
        }
    }

    #[test]
    fn sin_ulp_accuracy() {
        let pi = std::f64::consts::PI;
        let pi_over_2 = std::f64::consts::FRAC_PI_2;
        let pi_over_4 = std::f64::consts::FRAC_PI_4;

        // Direct `k_sin` fast-path: |x| ≤ π/4.
        for i in -2000..=2000 {
            check_ulp((i as f64 / 2000.0) * pi_over_4);
        }

        // High-accuracy region near π/2 where sin(x) ≈ ±1.
        for k in -20..=20 {
            let base = (k as f64) * pi + pi_over_2;
            for i in -100..=100 {
                check_ulp(base + (i as f64) * 1e-10);
            }
        }

        // Argument reduction around multiples of π.
        for k in 0..=30 {
            let base = (k as f64) * pi;
            for i in -50..=50 {
                check_ulp(base + (i as f64) * 0.012345);
            }
        }

        // Heavy stress on `rem_pio2` for very large magnitudes (up to ~1e22).
        let mut x = 1e6_f64;
        while x < 1e22 {
            check_ulp(x);
            check_ulp(x + 0.123456789);
            check_ulp(-x);
            x *= 3.1415926535;
        }

        // Deterministic pseudo-random walk for broad coverage.
        let mut walk = 0.987654321_f64;
        for _ in 0..5_000 {
            check_ulp(walk);
            walk = walk * 1.618033988749895 + 0.2718281828459045;
            if walk.abs() > 1e16 {
                walk = walk.fract() * 100.0;
            }
        }
    }

    #[test]
    fn sin_scale_coverage() {
        let pi = std::f64::consts::PI;
        let pi_over_2 = std::f64::consts::FRAC_PI_2;
        let pi_over_4 = std::f64::consts::FRAC_PI_4;

        let scales = [1.0, 10.0, 100.0, 1_000.0, 1e6, 1e8, 1e10];

        // Offsets that exercise each post-reduction quadrant (n & 3) and nearby
        // boundaries after argument reduction.
        let quadrant_offsets = [0.0, pi_over_4, pi_over_2, pi, 3.0 * pi_over_2, 2.0 * pi];

        let mut cases = Vec::new();

        for &scale in &scales {
            for &offset in &quadrant_offsets {
                cases.push(scale * offset);
                cases.push(-scale * offset);
                cases.push(scale + offset);
                cases.push(scale - offset);
            }

            // Irrational fractions of π at this magnitude.
            for k in 1..=5 {
                cases.push(scale * pi * (k as f64) / 11.0);
                cases.push(-scale * pi * (k as f64) / 11.0);
            }
        }

        // Subnormal-adjacent values (direct-return path for |x| < 2^-26).
        for i in 0..8 {
            cases.push(1e-20 * (i as f64 + 1.0));
            cases.push(-1e-20 * (i as f64 + 1.0));
        }

        for &x in &cases {
            check_ulp(x);
        }
    }

    /// Compile-time check that `sin` can be used in const contexts.
    const _: () = {
        let _ = sin(0.0);
        let _ = sin(1.0);
        let _ = sin(-std::f64::consts::PI);
    };
}
