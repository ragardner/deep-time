/// Private helper: `sin` that works in `const fn` (range reduction + Taylor).
/// Accuracy is far better than needed for TDB-TT.
pub const fn sin_approx(x: f64) -> f64 {
    const PI: f64 = core::f64::consts::PI;
    const TWO_PI: f64 = 2.0 * PI;

    let mut x = x % TWO_PI;
    if x < 0.0 {
        x += TWO_PI;
    }
    if x > PI {
        x -= TWO_PI;
    }

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let x = if x > PI / 2.0 { PI - x } else { x };

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;
    let x11 = x9 * x2;

    sign * (x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0 + x9 / 362880.0 - x11 / 39916800.0)
}
