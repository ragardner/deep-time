//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used by spacecraft and probes to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `ClockType`). The polynomial is evaluated
//! with full 36-digit exact arithmetic via `DtBig` — no floating-point loss even over centuries.

use crate::{C_SQUARED, Delta, DtBig, MICROQUECTOS_PER_SEC};

/// Quadratic polynomial: `constant + rate·dt + accel·dt²`
///
/// - `constant` – fixed offset (seconds)
/// - `rate`     – linear drift (s/s, dimensionless when multiplied by `dt`)
/// - `accel`    – quadratic drift (s/s²)
///
/// All fields are `Delta` so the entire expression stays exact to 10⁻³⁶ s.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct ClockDrift {
    /// Constant term a₀ (seconds)
    pub constant: Delta,
    /// Linear rate term a₁ (seconds per second)
    pub rate: Delta,
    /// Quadratic acceleration term a₂ (seconds per second²)
    pub accel: Delta,
}

impl ClockDrift {
    /// Creates a new polynomial with all three coefficients.
    #[inline]
    pub const fn new(constant: Delta, rate: Delta, accel: Delta) -> Self {
        Self {
            constant,
            rate,
            accel,
        }
    }

    /// Zero polynomial (no correction).
    pub const ZERO: Self = Self {
        constant: Delta::ZERO,
        rate: Delta::ZERO,
        accel: Delta::ZERO,
    };

    /// Pure constant offset (most common for static bias).
    #[inline]
    pub const fn from_constant(c: Delta) -> Self {
        Self {
            constant: c,
            rate: Delta::ZERO,
            accel: Delta::ZERO,
        }
    }

    /// Constant offset + constant drift rate (very common for GNSS and probe clock steering).
    #[inline]
    pub const fn from_offset_and_rate(offset: Delta, rate: Delta) -> Self {
        Self {
            constant: offset,
            rate,
            accel: Delta::ZERO,
        }
    }

    /// Evaluates the polynomial at elapsed time `dt` (exact, using `DtBig`).
    ///
    /// All arithmetic is performed with full 36-digit precision.
    pub const fn evaluate(&self, dt: Delta) -> Delta {
        let dt_big = dt.to_big();
        let mqs = DtBig::from_u128(MICROQUECTOS_PER_SEC);
        let mut total = self.constant.to_big();

        if !self.rate.is_zero() || !self.accel.is_zero() {
            let rate_big = self.rate.to_big();
            let accel_big = self.accel.to_big();

            // Linear term: rate * dt / 10³⁶
            let rate_term = rate_big.wrapping_mul(dt_big).div_euclid(mqs);

            // Quadratic term: accel * dt² / 10⁷²
            // Computed in two safe steps to keep every intermediate inside 320 bits.
            let accel_dt = accel_big.wrapping_mul(dt_big).div_euclid(mqs);
            let accel_term = accel_dt.wrapping_mul(dt_big).div_euclid(mqs);

            total = total.wrapping_add(rate_term).wrapping_add(accel_term);
        }

        Delta::from_big(total)
    }

    /// Creates a `ClockDrift` using the standard first-order post-Newtonian
    /// weak-field approximation.
    ///
    /// ```math
    /// \delta \approx -\frac{v^2}{2c^2} - \frac{\Phi}{c^2}
    /// ```
    ///
    /// where \( \Phi = +GM/r > 0 \) (positive Newtonian potential, Sun + planets).
    #[inline]
    pub const fn from_weak_field_approximation(
        velocity_squared_over_2c2: f64,
        gravitational_potential_over_c2: f64,
    ) -> Self {
        let rate = -velocity_squared_over_2c2 - gravitational_potential_over_c2;
        Self::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(rate))
    }

    /// Convenience using physical SI units.
    #[inline]
    pub const fn from_velocity_and_potential(
        velocity_m_s: f64,
        gravitational_potential_m2_s2: f64, // Φ_total > 0 (multi-body OK)
    ) -> Self {
        let v2_over_2c2 = (velocity_m_s * velocity_m_s) / (2.0 * C_SQUARED);
        let phi_over_c2 = gravitational_potential_m2_s2 / C_SQUARED;
        Self::from_weak_field_approximation(v2_over_2c2, phi_over_c2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Delta;

    #[test]
    fn evaluate_zero_drift() {
        let drift = ClockDrift::ZERO;
        let dt = Delta::from_sec(1_234_567);
        assert_eq!(drift.evaluate(dt), Delta::ZERO);
    }

    #[test]
    fn evaluate_constant_only() {
        let drift = ClockDrift::from_constant(Delta::from_sec_f64(0.5));
        let dt = Delta::from_sec(1_000);
        assert_eq!(drift.evaluate(dt), Delta::from_sec_f64(0.5));
    }

    #[test]
    fn evaluate_rate_only() {
        let drift = ClockDrift::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(1e-9)); // 1 ns/s
        let dt = Delta::from_sec(1_000_000); // 1 million seconds
        assert_eq!(drift.evaluate(dt), Delta::from_sec_f64(0.001)); // 1 µs
    }

    #[test]
    fn evaluate_full_quadratic() {
        let drift = ClockDrift::new(
            Delta::from_sec(2),
            Delta::from_ns(1), // exactly 1e-9 s/s
            Delta::from_as(2), // exactly 2e-18 s/s²
        );
        let dt = Delta::from_sec(1_000_000);
        // 2 + (1e-9 * 1e6) + (2e-18 * 1e12) = 2.001002 exactly
        assert_eq!(drift.evaluate(dt), Delta::from_sec_f64(2.001002));
    }

    #[test]
    fn evaluate_negative_dt() {
        let drift = ClockDrift::new(
            Delta::from_sec(5),
            Delta::from_ns(1), // exactly 1e-9 s/s
            Delta::from_as(1), // exactly 1e-18 s/s²
        );
        let dt = Delta::from_sec(-500_000);

        // Exact mathematical result (no f64 loss)
        let expected = Delta::from_sec(4)
            .add(Delta::from_ms(999))
            .add(Delta::from_us(500))
            .add(Delta::from_ns(250));

        assert_eq!(drift.evaluate(dt), expected);
    }

    #[test]
    fn evaluate_large_dt_exact() {
        let drift = ClockDrift::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(1e-12));
        let dt = Delta::from_sec(1_000_000_000); // ~31.7 years
        assert_eq!(drift.evaluate(dt), Delta::from_sec_f64(0.001));
    }
}
