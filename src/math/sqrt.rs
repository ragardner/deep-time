#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::Real;

const RSQRT_TAB: [u16; 128] = [
    0xb451, 0xb2f0, 0xb196, 0xb044, 0xaef9, 0xadb6, 0xac79, 0xab43, 0xaa14, 0xa8eb, 0xa7c8, 0xa6aa,
    0xa592, 0xa480, 0xa373, 0xa26b, 0xa168, 0xa06a, 0x9f70, 0x9e7b, 0x9d8a, 0x9c9d, 0x9bb5, 0x9ad1,
    0x99f0, 0x9913, 0x983a, 0x9765, 0x9693, 0x95c4, 0x94f8, 0x9430, 0x936b, 0x92a9, 0x91ea, 0x912e,
    0x9075, 0x8fbe, 0x8f0a, 0x8e59, 0x8daa, 0x8cfe, 0x8c54, 0x8bac, 0x8b07, 0x8a64, 0x89c4, 0x8925,
    0x8889, 0x87ee, 0x8756, 0x86c0, 0x862b, 0x8599, 0x8508, 0x8479, 0x83ec, 0x8361, 0x82d8, 0x8250,
    0x81c9, 0x8145, 0x80c2, 0x8040, 0xff02, 0xfd0e, 0xfb25, 0xf947, 0xf773, 0xf5aa, 0xf3ea, 0xf234,
    0xf087, 0xeee3, 0xed47, 0xebb3, 0xea27, 0xe8a3, 0xe727, 0xe5b2, 0xe443, 0xe2dc, 0xe17a, 0xe020,
    0xdecb, 0xdd7d, 0xdc34, 0xdaf1, 0xd9b3, 0xd87b, 0xd748, 0xd61a, 0xd4f1, 0xd3cd, 0xd2ad, 0xd192,
    0xd07b, 0xcf69, 0xce5b, 0xcd51, 0xcc4a, 0xcb48, 0xca4a, 0xc94f, 0xc858, 0xc764, 0xc674, 0xc587,
    0xc49d, 0xc3b7, 0xc2d4, 0xc1f4, 0xc116, 0xc03c, 0xbf65, 0xbe90, 0xbdbe, 0xbcef, 0xbc23, 0xbb59,
    0xba91, 0xb9cc, 0xb90a, 0xb84a, 0xb78c, 0xb6d0, 0xb617, 0xb560,
];

#[inline]
const fn mul32(a: u32, b: u32) -> u32 {
    ((a as u64).wrapping_mul(b as u64) >> 32) as u32
}

#[inline]
const fn mul64(a: u64, b: u64) -> u64 {
    let ahi = a >> 32;
    let alo = a & 0xffffffff;
    let bhi = b >> 32;
    let blo = b & 0xffffffff;
    ahi.wrapping_mul(bhi)
        .wrapping_add(ahi.wrapping_mul(blo) >> 32)
        .wrapping_add(alo.wrapping_mul(bhi) >> 32)
}

/// Computes sqrt(x) using the table-driven Goldschmidt iteration
/// from musl libc. Correctly rounded to nearest-even for all Real inputs.
/// const, no std, no alloc friendly.
pub const fn sqrt(x: Real) -> Real {
    let mut ix = x.to_bits();
    let mut top = ix >> 52;

    // Special cases: subnormal, inf, nan, negative, zero
    if top.wrapping_sub(0x001) >= 0x7fe {
        if ix << 1 == 0 {
            return x; // ±0.0
        }
        if ix == 0x7ff0_0000_0000_0000 {
            return x; // +inf
        }
        if ix > 0x7ff0_0000_0000_0000 {
            // negative or NaN → quiet NaN, preserve sign bit for -inf/-num
            let nan_bits = 0x7ff8_0000_0000_0000 | (ix & 0x8000_0000_0000_0000);
            return Real::from_bits(nan_bits);
        }
        // Subnormal: normalize by multiplying by 2^52
        let scale = Real::from_bits(0x4330_0000_0000_0000); // 2^52
        ix = (x * scale).to_bits();
        top = (ix >> 52).wrapping_sub(52);
    }

    let even = top & 1;
    let mut m = (ix << 11) | 0x8000_0000_0000_0000u64;
    if even != 0 {
        m >>= 1;
    }
    let top = (top.wrapping_add(0x3ff)) >> 1; // result exponent (biased)

    // Table-driven initial reciprocal sqrt estimate + Goldschmidt iterations
    // All vars u64 to match C closely; mul32/mul64 return u64 for simplicity
    let three: u64 = 0xc000_0000;
    let i = ((ix >> 46) % 128) as usize;
    let mut r: u64 = (RSQRT_TAB[i] as u64) << 16;

    let mut s: u64 = mul32((m >> 32) as u32, r as u32) as u64;
    let mut d: u64 = mul32(s as u32, r as u32) as u64;
    let mut u: u64 = three - d;
    r = (mul32(r as u32, u as u32) << 1) as u64;
    s = (mul32(s as u32, u as u32) << 1) as u64;

    d = mul32(s as u32, r as u32) as u64;
    u = three - d;
    r = (mul32(r as u32, u as u32) << 1) as u64;

    r <<= 32;
    s = mul64(m, r);
    d = mul64(s, r);
    u = (three << 32) - d;
    s = mul64(s, u);

    // Final adjustment and rounding decision
    s = (s - 2) >> 9;

    let d0 = (m << 42).wrapping_sub(s.wrapping_mul(s));
    let d1 = s.wrapping_sub(d0);
    let _d2 = d1.wrapping_add(s).wrapping_add(1);

    if (d1 >> 63) != 0 {
        s = s.wrapping_add(1);
    }
    s &= 0x000f_ffff_ffff_ffff;
    s |= top << 52;

    Real::from_bits(s)
}

const SPLIT: Real = 134217728. + 1.; // 0x1p27 + 1 === (2 ^ 27) + 1

const fn sq(x: Real) -> (Real, Real) {
    let xc: Real = x * SPLIT;
    let xh: Real = x - xc + xc;
    let xl: Real = x - xh;
    let hi = x * x;
    let lo = xh * xh - hi + 2. * xh * xl + xl * xl;
    (hi, lo)
}

/// Computes `sqrt(x² + y²)` without overflow or harmful underflow.
///
/// A `const fn`-compatible port of the musl `hypot` implementation.
/// Returns `|x|` when `y` is zero, and follows IEEE special-case rules
/// (e.g. `hypot(±∞, NaN)` returns `+∞`).
pub const fn hypot(mut x: Real, mut y: Real) -> Real {
    let x1p700 = Real::from_bits(0x6bb0000000000000); // 0x1p700 === 2 ^ 700
    let x1p_700 = Real::from_bits(0x1430000000000000); // 0x1p-700 === 2 ^ -700

    let mut uxi = x.to_bits();
    let mut uyi = y.to_bits();
    let uti;
    let mut z: Real;

    /* arrange |x| >= |y| */
    uxi &= -1i64 as u64 >> 1;
    uyi &= -1i64 as u64 >> 1;
    if uxi < uyi {
        uti = uxi;
        uxi = uyi;
        uyi = uti;
    }

    /* special cases */
    let ex: i64 = (uxi >> 52) as i64;
    let ey: i64 = (uyi >> 52) as i64;
    x = Real::from_bits(uxi);
    y = Real::from_bits(uyi);
    /* note: hypot(inf,nan) == inf */
    if ey == 0x7ff {
        return y;
    }
    if ex == 0x7ff || uyi == 0 {
        return x;
    }
    /* note: hypot(x,y) ~= x + y*y/x/2 with inexact for small y/x */
    /* 64 difference is enough for ld80 double_t */
    if ex - ey > 64 {
        return x + y;
    }

    /* precise sqrt argument in nearest rounding mode without overflow */
    /* xh*xh must not overflow and xl*xl must not underflow in sq */
    z = 1.;
    if ex > 0x3ff + 510 {
        z = x1p700;
        x *= x1p_700;
        y *= x1p_700;
    } else if ey < 0x3ff - 450 {
        z = x1p_700;
        x *= x1p700;
        y *= x1p700;
    }
    let (hx, lx) = sq(x);
    let (hy, ly) = sq(y);
    z * sqrt(ly + lx + hy + hx)
}

#[cfg(all(test, feature = "std"))]
mod sqrt_tests {
    use super::sqrt;
    use std::{f64, vec, vec::Vec};

    #[test]
    fn test_special_cases() {
        assert_eq!(sqrt(0.0), 0.0);
        assert_eq!(sqrt(-0.0), -0.0);
        assert!(sqrt(f64::INFINITY).is_infinite() && sqrt(f64::INFINITY) > 0.0);
        assert!(sqrt(f64::NEG_INFINITY).is_nan());
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
        // signaling nan? but in practice quiet
    }

    #[test]
    fn test_perfect_squares() {
        for i in 0..100u32 {
            let x = (i * i) as f64;
            let r = sqrt(x);
            assert!((r - i as f64).abs() < 1e-10 || r.is_nan());
        }
    }

    #[test]
    fn test_random_vs_std() {
        // 5M deterministic LCG random normals in [1,2) — exercises table + Goldschmidt fully
        let mut failures = 0u32;
        let mut state: u64 = 0x123456789abcdef0;
        for _ in 0..5_000_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let bits = (state & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000; // positive normal [1,2)
            let val = f64::from_bits(bits);
            let r1 = sqrt(val);
            let r2 = val.sqrt();
            if r1.to_bits() != r2.to_bits() {
                failures += 1;
                // if failures < 3 {
                //     eprintln!(
                //         "Mismatch at {:016x}: ours={:016x} std={:016x}",
                //         bits,
                //         r1.to_bits(),
                //         r2.to_bits()
                //     );
                // }
            }
        }
        assert_eq!(
            failures, 0,
            "Found {} mismatches in 5M random normals [1,2)",
            failures
        );
    }

    #[test]
    fn test_subnormals_random() {
        // 100k random subnormals (exp=0) — critical for normalize path
        let mut failures = 0u32;
        let mut state: u64 = 0xdeadbeefcafebabe;
        for _ in 0..100_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            // subnormal: exp=0, random mantissa (low 52 bits)
            let bits = state & 0x000f_ffff_ffff_ffff; // clears sign + exp
            let val = f64::from_bits(bits);
            if val == 0.0 {
                continue;
            } // skip zero
            let r1 = sqrt(val);
            let r2 = val.sqrt();
            if r1.to_bits() != r2.to_bits() {
                failures += 1;
                // if failures < 3 {
                //     eprintln!(
                //         "Subnormal mismatch at {:016x}: ours={:016x} std={:016x}",
                //         bits,
                //         r1.to_bits(),
                //         r2.to_bits()
                //     );
                // }
            }
        }
        assert_eq!(
            failures, 0,
            "Found {} mismatches in 100k random subnormals",
            failures
        );
    }

    #[test]
    fn test_boundaries() {
        // Critical boundaries: min/max normal, subnormal boundary, overflow edge, powers of 2
        let boundaries: [f64; 8] = [
            f64::MIN_POSITIVE,                         // 2^-1022 (smallest normal)
            f64::from_bits(0x0010_0000_0000_0000),     // 2^-1021
            f64::from_bits(0x000f_ffff_ffff_ffff),     // largest subnormal
            2.0f64.powi(-1074),                        // smallest positive subnormal (2^-1074)
            f64::MAX,                                  // ~1.8e308
            f64::from_bits(0x7fe0_0000_0000_0000),     // largest finite < inf
            2.0f64.powi(1023),                         // 2^1023 (largest power of 2)
            2.0f64.powi(-1022) * (1.0 + f64::EPSILON), // just above min normal
        ];
        for &x in &boundaries {
            let r1 = sqrt(x);
            let r2 = x.sqrt();
            assert_eq!(r1.to_bits(), r2.to_bits(), "Boundary mismatch for {:e}", x);
            // Also check sqrt(x*x) ~ |x| for positive x (within rounding), but skip underflow cases
            if x > 0.0 && x.is_finite() && x > 1e-200 {
                let xx = x * x;
                if xx.is_finite() && xx.is_normal() {
                    let r = sqrt(xx);
                    let rel = ((r - x).abs() / x).max(0.0);
                    assert!(
                        rel < 1e-14 || r.is_nan(),
                        "sqrt(x*x) not close to x for {}",
                        x
                    );
                }
            }
        }
    }

    #[test]
    fn test_known_hard_cases() {
        // Known hard-to-round / exact / boundary cases — all must match std bit-exactly
        let cases: &[f64] = &[
            2.0,
            0.5,
            4.0,
            9.0,
            0.0,
            f64::INFINITY,
            1.0e-300,                              // very small normal
            f64::from_bits(0x0010_0000_0000_0001), // just above min normal
            1.0 + f64::EPSILON,                    // next after 1.0
            f64::from_bits(0x7fefffffffffffff),    // largest finite
        ];
        for &x in cases {
            let r = sqrt(x);
            // bit-exact check vs Rust std (the gold standard for this platform)
            assert_eq!(r.to_bits(), x.sqrt().to_bits(), "Bit mismatch for {:e}", x);
        }
    }

    // Manual nextUp / nextDown
    fn next_up(x: f64) -> f64 {
        if x.is_nan() || x == f64::INFINITY {
            return x;
        }
        if x == 0.0 {
            return f64::from_bits(1);
        }
        let bits = x.to_bits();
        if x > 0.0 {
            f64::from_bits(bits + 1)
        } else {
            f64::from_bits(bits - 1)
        }
    }
    fn next_down(x: f64) -> f64 {
        if x.is_nan() || x == f64::NEG_INFINITY {
            return x;
        }
        if x == -0.0 || x == 0.0 {
            return f64::from_bits(0x8000_0000_0000_0001);
        }
        let bits = x.to_bits();
        if x > 0.0 {
            f64::from_bits(bits - 1)
        } else {
            f64::from_bits(bits + 1)
        }
    }

    #[test]
    fn test_powers_of_two() {
        // All representable powers of 2 (even exponents must be exact, odd use std)
        for exp in -1074i32..=1023 {
            let x = if exp >= -1022 {
                2.0f64.powi(exp)
            } else {
                // subnormal 2^exp = 2^(exp + 1074) * 2^-1074
                f64::from_bits(1u64 << (exp + 1074))
            };
            if !x.is_finite() || x == 0.0 {
                continue;
            }
            let r1 = sqrt(x);
            let r2 = x.sqrt();
            assert_eq!(
                r1.to_bits(),
                r2.to_bits(),
                "Power-of-2 mismatch for 2^{}",
                exp
            );
            // For even exponents, result should be exactly 2^(exp/2) when representable
            if exp % 2 == 0 {
                let expected_exp = exp / 2;
                if expected_exp >= -1022 {
                    let expected = 2.0f64.powi(expected_exp);
                    assert_eq!(
                        r1.to_bits(),
                        expected.to_bits(),
                        "Even power-of-2 not exact for 2^{}",
                        exp
                    );
                }
            }
        }
    }

    #[test]
    fn test_nextafter_edges() {
        // nextUp / nextDown around critical points (0, 1, min_normal, max)
        let mut edges: Vec<f64> = vec![
            f64::from_bits(1),                     // smallest positive subnormal
            f64::from_bits(0x0000_0000_0000_0002), // next subnormal
            next_down(f64::MIN_POSITIVE),          // largest subnormal
            f64::MIN_POSITIVE,                     // smallest normal
            next_up(f64::MIN_POSITIVE),
            next_down(1.0),
            1.0,
            next_up(1.0),
            next_down(f64::MAX),
            f64::MAX,
        ];
        // Also a few negative edges (should all produce NaN)
        edges.push(next_up(-f64::MIN_POSITIVE)); // negative smallest normal-ish
        for &x in &edges {
            let r1 = sqrt(x);
            let r2 = x.sqrt();
            assert_eq!(
                r1.to_bits(),
                r2.to_bits(),
                "nextafter edge mismatch for {:e} (bits {:016x})",
                x,
                x.to_bits()
            );
        }
    }

    #[test]
    fn test_negative_subnormals() {
        // All negative subnormals must produce NaN (sign bit set in result)
        let mut state: u64 = 0xfeedface_deadbeef;
        for _ in 0..10_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let bits = (state & 0x000f_ffff_ffff_ffff) | 0x8000_0000_0000_0000; // negative subnormal
            let val = f64::from_bits(bits);
            if val == 0.0 {
                continue;
            }
            let r = sqrt(val);
            assert!(
                r.is_nan(),
                "Negative subnormal did not produce NaN: {:e}",
                val
            );
            // sign bit should be set (negative NaN)
            assert!(
                r.to_bits() & 0x8000_0000_0000_0000 != 0,
                "NaN sign bit not set for negative subnormal"
            );
        }
    }
}
