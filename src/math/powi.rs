use crate::Real;

/// Raises a floating-point number to an integer power.
///
/// Fully handles `i32::MIN` without overflow.
#[must_use]
pub const fn powi(base: Real, exp: i32) -> Real {
    if exp == 0 {
        return 1.0;
    }
    if exp == 1 {
        return base;
    }
    if exp == -1 {
        return 1.0 / base;
    }

    let mut result = f!(1.0);
    let mut b = if exp < 0 { 1.0 / base } else { base };
    let mut e = exp.unsigned_abs();

    while e > 0 {
        if e & 1 == 1 {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }

    result
}
