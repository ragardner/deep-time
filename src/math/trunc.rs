use crate::Real;

/// Const-compatible version of `f64::trunc` (rounds toward zero)
pub const fn trunc(x: Real) -> Real {
    let bits = x.to_bits();
    let sign = bits & (1u64 << 63);
    let exp = ((bits >> 52) & 0x7ff) as i32 - 1023;

    if exp < 0 {
        // |x| < 1.0 → result is signed zero
        return Real::from_bits(sign);
    }

    if exp >= 52 {
        return x;
    }

    // Clear all fractional bits
    let mask = !0u64 << (52 - exp);
    let truncated = (bits & mask) | sign; // preserve sign
    Real::from_bits(truncated)
}
// tests in super::round
