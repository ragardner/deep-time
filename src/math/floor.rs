#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::Real;

/// Floor function for Real.
///
/// This implementation uses bit manipulation for safety and correctness in const contexts.
/// It does not track inexact status.
pub const fn floor_f(x: Real) -> Real {
    // Handle special cases
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    if x == 0.0 {
        return x; // preserve signed zero
    }

    let bits = x.to_bits();
    let sign = bits >> 63; // 0 = positive, 1 = negative
    let exp = ((bits >> 52) & 0x7ff) as i32 - 1023;

    // |x| < 1.0
    if exp < 0 {
        return if sign == 1 { -1.0 } else { 0.0 };
    }

    // Already an integer, or |x| is so large that it has no fractional bits
    if exp >= 52 {
        return x;
    }

    // Create mask to clear fractional bits
    let frac_bits = 52 - exp;
    let mask = (1u64 << frac_bits) - 1;
    let cleared = bits & !mask;

    let result = Real::from_bits(cleared);

    // For negative numbers: if we had any fractional part, we must subtract 1
    // to round toward negative infinity.
    if sign == 1 && cleared != bits {
        result - 1.0
    } else {
        result
    }
}
