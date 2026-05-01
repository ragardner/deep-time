use crate::Real;

/// Ultra-high-accuracy sine approximation that can be evaluated at compile time.
///
/// This is a private helper function used internally for high-precision
/// astronomical time transformations (primarily TDB-TT and related
/// relativistic corrections). It provides a balance of accuracy, speed,
/// and `const fn` compatibility without requiring any runtime tables or
/// external dependencies.
///
/// # Algorithm
///
/// 1. **Range reduction**  
///    The input angle is reduced to the interval `[-π, π]` using floating-point
///    modulo, followed by a second reduction to `[0, π/2]` by exploiting the
///    identities `sin(-x) = -sin(x)` and `sin(π - x) = sin(x)`.
///
/// 2. **Taylor series (Horner form)**  
///    The sine is computed using the Taylor series for `sin(x)` around zero,
///    truncated after the `x¹⁵` term and evaluated with Horner's method for
///    optimal numerical stability and minimal operations:
///
///    ```text
///    sin(x) = x − x³/3! + x⁵/5! − x⁷/7! + x⁹/9! − x¹¹/11! + x¹³/13! − x¹⁵/15!
///    ```
///
///    All powers of `x²` are accumulated via repeated multiplication by `y = x²`.
///
/// # Accuracy
///
/// - **Maximum absolute error**: ≈ **6.02 × 10⁻¹²** radians over the entire
///   real line (the worst case occurs near odd multiples of `π/2`).
/// - This is more than **100×** better than the previous 7-term version and
///   over **9,000×** better than the original 5-term implementation.
/// - For all practical TDB-TT and astronomical time-scale work the error is
///   completely negligible — it is smaller than the inherent uncertainty of
///   most input ephemerides.
///
/// # Performance & Const-fn Properties
///
/// - Entirely `const fn` compatible (no heap allocation, no panics on valid input).
/// - Uses only multiplication, addition/subtraction, and a single modulo.
/// - Horner evaluation requires only **8 multiplications and 8 additions** after
///   range reduction — still extremely cheap.
/// - No conditional branches inside the polynomial evaluation.
///
/// # Limitations
///
/// - For extremely large arguments (`|x| ≳ 10¹⁴`), the floating-point modulo
///   operation loses precision because the mantissa of `f64` can no longer
///   represent the fractional part of `x / (2π)` accurately. In practice this
///   is irrelevant for all astronomical time arguments encountered in TDB-TT
///   calculations.
/// - The function does not handle `NaN` or `Infinity` specially; they propagate
///   according to IEEE 754 rules.
///
/// # Design Rationale
///
/// The implementation uses a plain Taylor series (instead of a minimax or
/// Chebyshev polynomial) because:
/// - The coefficients are simple exact fractions (`1/n!`).
/// - The series is trivial to extend or truncate.
/// - Horner form already gives near-optimal accuracy for the chosen degree
///   on `[0, π/2]`.
///
/// One more term (`+x¹⁷/17!`) can be added if future requirements ever demand
/// sub-10⁻¹³ accuracy.
///
/// # See Also
///
/// - Previous versions in git history (5-term, 7-term) for regression testing.
/// - `core::f64::consts::PI` and the `f!` macro for literal typing.
pub(crate) const fn sin_approx(x: Real) -> Real {
    const PI: Real = f!(core::f64::consts::PI);
    const TWO_PI: Real = f!(2.0) * PI;

    // === Range reduction to [-π, π] ===
    // Uses the mathematical identity sin(x) = sin(x mod 2π).
    // The two-step adjustment guarantees the result lies in [-π, π].
    let mut x = x % TWO_PI;
    if x < f!(0.0) {
        x += TWO_PI;
    }
    if x > PI {
        x -= TWO_PI;
    }

    // === Sign handling and reduction to [0, π/2] ===
    // sin(-x) = -sin(x)  and  sin(π - x) = sin(x)
    let sign = if x < f!(0.0) { f!(-1.0) } else { f!(1.0) };
    let x = x.abs();

    let x = if x > PI / f!(2.0) { PI - x } else { x };

    // === Taylor series via Horner's method (up to x¹⁵ term) ===
    // y = x²
    // p(y) = 1 − y/3! + y²/5! − y³/7! + y⁴/9! − y⁵/11! + y⁶/13! − y⁷/15!
    // sin(x) = x · p(y)
    //
    // Horner form is used for:
    //   • minimal number of operations (8 muls + 8 adds)
    //   • excellent floating-point rounding properties
    //   • trivial extensibility
    let y = x * x;

    // Start with the highest-degree coefficient and work downwards.
    // All denominators are exact factorials and are exactly representable
    // as f64 constants.
    let p = f!(-1.0) / f!(1307674368000.0); // −1/15!
    let p = p * y + f!(1.0) / f!(6227020800.0); // +1/13!
    let p = p * y + f!(-1.0) / f!(39916800.0); // −1/11!
    let p = p * y + f!(1.0) / f!(362880.0); // +1/9!
    let p = p * y + f!(-1.0) / f!(5040.0); // −1/7!
    let p = p * y + f!(1.0) / f!(120.0); // +1/5!
    let p = p * y + f!(-1.0) / f!(6.0); // −1/3!
    let p = p * y + f!(1.0); // +1

    sign * (x * p)
}

/// `const fn` implementation of floor for `Real`.
///
/// This is identical to `std::f64::floor` (including signed-zero
/// preservation) while remaining fully `const fn` on stable Rust with `#![no_std]`.
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
