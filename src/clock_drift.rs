//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used by spacecraft to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `ClockType`). The polynomial is evaluated
//! with full 36-digit exact arithmetic via `DtBig` — no floating-point loss even over centuries.

use crate::{
    C_SQUARED, Delta, DtBig, MICROQUECTOS_PER_SEC, Velocity, alpha_from_weak_field_potential,
    curvature_regulator, kretschmann_from_potential_and_scale,
};

/// Pre-resolved local spacetime metric quantities supplied by the caller.
///
/// - `alpha` comes from `alpha_from_weak_field_potential` (e.g. solar system use)
///   or from a full metric / onboard gravimeter.
/// - `beta` comes from `probe_velocity.beta()`.
/// - `kretschmann` is 0.0 in the solar system today (future gravimetric
///   hardware will supply the real value).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolvedMetric {
    pub alpha: f64,
    pub beta: f64,
    pub kretschmann: f64,
}

impl ResolvedMetric {
    #[inline(always)]
    pub const fn new(alpha: f64, beta: f64, kretschmann: f64) -> Self {
        Self {
            alpha,
            beta,
            kretschmann,
        }
    }

    /// Convenience for direct gravimeter / sensor paths.
    #[inline(always)]
    pub fn from_gravitic_and_velocity(alpha: f64, kretschmann: f64, velocity: Velocity) -> Self {
        Self::new(alpha, velocity.beta(), kretschmann)
    }

    /// Recommended constructor for most users.
    ///
    /// Computes both the gravitational lapse `α` **and** the Kretschmann scalar
    /// from the total local potential and the characteristic length scale.
    ///
    /// - Solar-system / GNSS users: pass `characteristic_length_scale = 0.0`
    ///   (returns exactly the same rate as the old weak-field path).
    /// - Strong-field users: `characteristic_length_scale`
    ///     — the typical length scale (in meters) over which the
    ///       gravitational field varies at the observer’s location.
    #[inline(always)]
    pub fn from_potential_velocity_and_scale(
        phi_over_c2: f64, // Φ/c² (total local potential)
        velocity: Velocity,
        characteristic_length_scale: f64,
    ) -> Self {
        let alpha = alpha_from_weak_field_potential(phi_over_c2);
        let kretschmann =
            kretschmann_from_potential_and_scale(phi_over_c2, characteristic_length_scale);
        Self::from_gravitic_and_velocity(alpha, kretschmann, velocity)
    }
}

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

    /// Creates a `ClockDrift` using the exact proper-time factor from the
    /// weak-field isotropic metric (higher-order accuracy).
    ///
    /// This is mathematically more precise than the linear approximation while
    /// remaining extremely fast and fully valid throughout the solar system.
    ///
    /// ```math
    /// \frac{d\tau}{dt} = \sqrt{(1 - 2\phi) - (1 + 2\phi)\beta^2}
    /// ```
    ///
    /// where \(\phi = \Phi/c^2 > 0\) and \(\beta^2 = v^2/c^2\).
    #[inline]
    pub fn from_weak_field_metric(
        velocity_squared_over_2c2: f64,
        gravitational_potential_over_c2: f64,
    ) -> Self {
        let phi = gravitational_potential_over_c2; // Φ/c² > 0
        let beta2 = 2.0 * velocity_squared_over_2c2; // v²/c
        let inside = (1.0 - 2.0 * phi) - (1.0 + 2.0 * phi) * beta2;
        let rate_factor = inside.sqrt(); // dτ/dt
        let rate_offset = rate_factor - 1.0;
        Self::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(rate_offset))
    }

    /// Convenience using physical SI units.
    #[inline]
    pub fn from_velocity_and_potential(
        velocity_m_s: f64,
        gravitational_potential_m2_s2: f64, // Φ_total > 0 (multi-body OK)
    ) -> Self {
        let v2_over_2c2 = (velocity_m_s * velocity_m_s) / (2.0 * C_SQUARED);
        let phi_over_c2 = gravitational_potential_m2_s2 / C_SQUARED;
        Self::from_weak_field_metric(v2_over_2c2, phi_over_c2)
    }

    /// Creates a `ClockDrift` from the probe's velocity and the total local gravitational potential.
    ///
    /// This is the main convenience function most users should call. It computes the relativistic
    /// rate at which a clock on your spacecraft runs relative to coordinate time, taking into
    /// account both its motion (special relativity) and the gravity it experiences (general relativity).
    ///
    /// - `velocity_m_s`: The speed of the probe in meters per second.
    /// - `gravitational_potential_m2_s2`: The total gravitational potential Φ (in m²/s²) at the
    ///   probe's location. This is usually negative and can include contributions from multiple bodies.
    /// - `characteristic_length_scale`: The characteristic length (in meters) over which gravity
    ///   varies significantly around the probe. For Earth orbit, GNSS, or solar-system navigation,
    ///   simply pass `0.0`. A non-zero value enables higher-order curvature effects and is only
    ///   needed for strong gravitational fields (e.g. neutron star or black hole flybys).
    ///
    /// Example:
    /// ```rust
    /// let drift = ClockDrift::from_velocity_potential_and_scale(
    ///     7800.0,      // velocity in m/s
    ///     -6.2e7,      // gravitational potential in m²/s²
    ///     0.0,         // 0.0 = standard weak-field case
    /// );
    /// ```
    #[inline]
    pub fn from_velocity_potential_and_scale(
        velocity_m_s: f64,
        gravitational_potential_m2_s2: f64,
        characteristic_length_scale: f64,
    ) -> Self {
        let phi = gravitational_potential_m2_s2 / C_SQUARED;
        let velocity = Velocity::from_speed(velocity_m_s);
        let resolved = ResolvedMetric::from_potential_velocity_and_scale(
            phi,
            velocity,
            characteristic_length_scale,
        );
        Self::from_resolved_metric(resolved)
    }

    /// Canonical low-level constructor — the single source of truth
    /// for the entire unified timelike/null probe Lagrangian.
    #[inline]
    pub fn from_unified_proper_time_rate(u: f64, kretschmann: f64) -> Self {
        let eps = curvature_regulator(kretschmann);
        let k = u + eps * (1.0 - u).powi(2);
        let rate_factor = k.sqrt().max(0.0);
        let rate_offset = rate_factor - 1.0;

        Self::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(rate_offset))
    }

    /// High-level entry point — the main function users will call.
    #[inline]
    pub fn from_resolved_metric(resolved: ResolvedMetric) -> Self {
        let u = resolved.alpha * resolved.alpha * (1.0 - resolved.beta * resolved.beta);
        Self::from_unified_proper_time_rate(u, resolved.kretschmann)
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
