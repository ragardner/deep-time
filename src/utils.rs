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

/// Const-fn compatible floor (replaces f64::floor)
pub(crate) const fn floor_f64(x: f64) -> f64 {
    let i = x as i64;
    if x >= 0.0 {
        i as f64
    } else if (i as f64) == x {
        i as f64
    } else {
        i as f64 - 1.0
    }
}

/// Const-fn compatible ceil (replaces f64::ceil)
pub(crate) const fn ceil_f64(x: f64) -> f64 {
    let i = x as i64;
    if x <= 0.0 {
        i as f64
    } else if (i as f64) == x {
        i as f64
    } else {
        i as f64 + 1.0
    }
}

/// Const-fn compatible round (halfway cases round away from zero, matching Rust's f64::round)
pub(crate) const fn round_f64(x: f64) -> f64 {
    let floor = floor_f64(x);
    let frac = x - floor;
    if frac < 0.5 {
        floor
    } else if frac > 0.5 {
        floor + 1.0
    } else {
        // halfway case
        if x >= 0.0 { floor + 1.0 } else { floor }
    }
}
