use crate::Real;

/// Private helper: `sin` that works in `const fn` (range reduction + Taylor).
/// Accuracy is far better than needed for TDB-TT.
pub(crate) const fn sin_approx(x: Real) -> Real {
    const PI: Real = f!(core::f64::consts::PI);
    const TWO_PI: Real = f!(2.0) * PI;

    let mut x = x % TWO_PI;
    if x < f!(0.0) {
        x += TWO_PI;
    }
    if x > PI {
        x -= TWO_PI;
    }

    let sign = if x < f!(0.0) { f!(-1.0) } else { f!(1.0) };
    let x = x.abs();

    let x = if x > PI / f!(2.0) { PI - x } else { x };

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;
    let x11 = x9 * x2;

    sign * (x - x3 / f!(6.0) + x5 / f!(120.0) - x7 / f!(5040.0) + x9 / f!(362880.0)
        - x11 / f!(39916800.0))
}

/// Hand-rolled `const fn` implementation of floor for `Real`.
///
/// This is **bit-for-bit identical** to `std::f64::floor` (including signed-zero
/// preservation) while remaining fully `const fn` on stable Rust with `#![no_std]`.
#[inline(always)]
pub(crate) const fn floor_f(x: Real) -> Real {
    if x.is_nan() || x.is_infinite() {
        x
    } else if x == f!(0.0) {
        x // preserves +0.0 or -0.0
    } else {
        let i = x as i64;
        let truncated = i as Real;
        if x >= f!(0.0) || truncated == x {
            truncated
        } else {
            truncated - f!(1.0)
        }
    }
}
