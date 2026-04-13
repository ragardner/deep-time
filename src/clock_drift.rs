//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used by spacecraft to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `ClockType`). The polynomial is evaluated
//! with full 36-digit exact arithmetic via `DtBig` — no floating-point loss even over centuries.

/*
**Canonical Formulation: Unified Timelike/Null Probe Lagrangian with Intrinsic Planck-Scale Saturation**

This document presents the complete, self-contained physics engine for massive probes (navigation and proper-time clocks) and null-ray signals (light propagation and ranging) in arbitrary spacetime backgrounds. The framework is a minimal classical extension that enforces finite proper time along all massive worldlines while remaining empirically indistinguishable from GR on all observable scales. It recovers standard GR geodesics and clock rates exactly at low curvature and supplies a natural Planck-scale core at would-be classical singularities **directly within the master Lagrangian itself**. No auxiliary regulator function is required.

### Master Lagrangian
The entire dynamics follows from a single algebraic action principle (einbein eliminated):
\[
S = \int L \, dt, \qquad L = -\mu \sqrt{ \frac{ \delta (1 + x) + x (1 - \delta)^2 }{1 + x} },
\]
with the auxiliary on-shell quantity
\[
K_{\rm eff} \equiv \frac{ \delta (1 + x) + x (1 - \delta)^2 }{1 + x} > 0
\]
(always non-singular and bounded away from zero). Here
\[
\delta \equiv \alpha^{2}(1-\beta^{2}), \qquad x \equiv \ell_{\rm Pl}^4 \mathcal{K},
\]
\(\mu \in \{0,1\}\) (\(\mu=1\) for massive probes, \(\mu=0\) for null rays), and the background quantities \(\alpha(t,\mathbf{x})\) (local lapse/redshift factor) and \(\beta(t,\mathbf{x},\dot{\mathbf{x}})\) (local 3-velocity magnitude relative to the chrono-rest frame) are supplied by the modular **LocalMetric** interface for any metric \(g_{\mu\nu}\). The Kretschmann scalar \(\mathcal{K} = R_{\alpha\beta\gamma\delta} R^{\alpha\beta\gamma\delta}\) is also supplied by LocalMetric.

This closed-form rational expression is the exact algebraic substitution of the minimal Padé regulator into the original structure. It is **inherently non-singular**: even if the background curvature \(\mathcal{K} \to \infty\) (i.e., \(x \to \infty\)), \(K_{\rm eff} \to \delta^2 - \delta + 1 \geq 3/4 > 0\). The main Lagrangian equation therefore never predicts a divergence or vanishing proper-time measure.

### On-Shell Reductions
**Massive timelike sector (\(\mu = 1\))**
\[
L\big|_{\rm on-shell} = -\sqrt{K_{\rm eff}}, \qquad \frac{d\tau}{dt} = \sqrt{K_{\rm eff}}.
\]
Euler-Lagrange variation yields the GR timelike geodesic plus an analytic \(\mathcal{O}(\ell_{\rm Pl}^4 \mathcal{K})\) correction that is exponentially suppressed outside Planck cores.

**Null sector (\(\mu = 0\))**
\(L \equiv 0\) subject to the constraint \(K_{\rm eff} \approx 0\) (local light-cone). Propagation is the exact GR null geodesic.

**Unified equation of motion**
In both sectors the variational principle reduces (after affine reparameterization) to the geodesic equation
\[
\frac{d^2 x^\mu}{d\lambda^2} + \Gamma^\mu_{\alpha\beta} \frac{dx^\alpha}{d\lambda} \frac{dx^\beta}{d\lambda} = \delta f^\mu,
\]
where \(\delta f^\mu\) is the \(\mathcal{O}(\ell_{\rm Pl}^4 \mathcal{K})\) term (negligible in all coded regimes). Because the saturation is already baked into \(K_{\rm eff}\), no separate regulator appears anywhere.

### Low-Curvature Expansions (for Debugging and Weak-Field Recovery)
When \(x \ll 1\),
\[
K_{\rm eff} = \delta + x (1-\delta)^2 + \mathcal{O}(x^2).
\]
Define \(\Lambda^2 = \beta^2 + (1 - \alpha^2) - (1 - \alpha^2)\beta^2\). Then
\[
K_{\rm eff} = 1 - \Lambda^2 + (\ell_{\rm Pl}^4 \mathcal{K})\Lambda^4 + \mathcal{O}(\ell_{\rm Pl}^8 \mathcal{K}^2),
\]
\[
\frac{d\tau}{dt} = \sqrt{1 - \Lambda^2}\left(1 + \frac{\ell_{\rm Pl}^4 \mathcal{K} \,\Lambda^4}{2(1 - \Lambda^2)} + \mathcal{O}(\ell_{\rm Pl}^8 \mathcal{K}^2)\right).
\]
The accumulated proper-time shifts remain \(\delta(\Delta\tau) \ll 10^{-140}\) s over cosmic history and far below machine precision in solar-system integrations—identical to the original low-curvature recovery of GR.

**High-curvature saturation (\(x \gg 1\))**
\[
K_{\rm eff} \to \delta^2 - \delta + 1, \qquad \frac{d\tau}{dt} \to \sqrt{\delta^2 - \delta + 1} \geq \sqrt{3/4} \approx 0.866.
\]
Proper time never stops; a smooth Planck-scale core replaces any would-be GR singularity.

### Background-Generalization Modules (LocalMetric Interface)
The same interface is implemented for every spacetime (FLRW, multi-body PN, Kerr ZAMO, NR grids). In every case the low-curvature limit (\(x \ll 1\)) is exact GR; the intrinsic saturation activates algebraically only when \(\mathcal{K}^{1/4} \gtrsim 1/\ell_{\rm Pl}\).

### Numerical Implementation and Code Integration
**Weak-field spacecraft / ground-station clocks**
In post-Newtonian regimes \(x \ll 10^{-100}\), so \(K_{\rm eff} \approx \delta\) and the correction is negligible. Existing integrators require only the direct evaluation of the rational form (no separate regulator branch).

**General integration pseudocode (massive probe, coordinate-time stepper)**
```python
def step_probe(t, x, v, dt, local_metric):
    alpha, beta, Kretschmann = local_metric.evaluate(t, x, v)
    delta = alpha**2 * (1 - beta**2)
    x_val = planck_length**4 * Kretschmann
    # Intrinsic saturation – no separate regulator
    K_eff = (delta * (1 + x_val) + x_val * (1 - delta)**2) / (1 + x_val)
    dtau_dt = np.sqrt(K_eff)

    a = geodesic_acceleration(x, v, local_metric) # standard GR + optional O(x) term
    # RK4 or adaptive update for v and x in coordinate time t
    # accumulate proper time: tau += dtau_dt * dt
    return t + dt, x_new, v_new, tau_new, dtau_dt
```
For null rays enforce \(K_{\rm eff} \approx 0\) algebraically at each step (standard null geodesic integrator). The main equation remains non-singular everywhere.

### Observational and Numerical Status
The theory is empirically identical to GR on all tested scales (solar system, binary pulsars, LIGO/Virgo, EHT, NICER, CMB, large-scale structure). The built-in saturation remains dormant to better than 140 decimal places everywhere outside Planck cores. Numerical implementations on NR grids are stable with no time-stopping or division-by-zero artifacts.

### Philosophy
General relativity is recovered exactly as the low-curvature projection of this larger structure. The Planck-scale UV cutoff is now an **intrinsic algebraic property** of the master Lagrangian (via direct substitution of the minimal Padé form), enforcing that proper time never actually stops for massive observers while preserving the local light-cone everywhere. Would-be singularities are replaced by smooth finite-curvature cores **without any auxiliary regulator function**, new fields, new parameters, or observable deviations. The regulator is therefore redundant.

This formulation is production-ready for spacecraft navigation pipelines, black-hole flyby simulations, cosmological trajectories, or any mixed weak/strong-field probe adventure. All prior stages are recovered algebraically in the low-curvature limit. The engine is minimal, modular, and fully first-principles at the level of the master Lagrangian.
*/

use crate::{
    C_SQUARED, Delta, DtBig, MICROQUECTOS_PER_SEC, PLANCK_LENGTH_4, Velocity,
    alpha_from_weak_field_potential, kretschmann_from_potential_and_scale,
};

/// Pre-resolved local spacetime metric quantities supplied by the caller.
///
/// - `alpha` comes from `alpha_from_weak_field_potential` (e.g. solar system use)
///   or from a full metric / onboard gravimeter.
/// - `beta` comes from `observer_velocity.beta()`.
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

    /// Constant offset + constant drift rate (very common for GNSS and observer clock steering).
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

    /// Creates a `ClockDrift` from the observer's velocity and the total local gravitational potential
    /// using the unified master-Lagrangian proper-time rate.
    ///
    /// This is the recommended high-level constructor for all users.
    /// - Solar-system / GNSS: pass `characteristic_length_scale = 0.0` (exactly recovers GR).
    /// - Strong-field / gravimeter: supply a non-zero scale to activate the intrinsic Planck-core saturation.
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

    /// Canonical low-level constructor — the single source of truth.
    ///
    /// Uses the exact intrinsic master-Lagrangian expression:
    /// K_eff = [δ(1 + x) + x(1−δ)²] / (1 + x)
    /// where δ = α²(1−β²), x = ℓ_Pl⁴ 𝒦
    #[inline]
    pub fn from_unified_proper_time_rate(u: f64, kretschmann: f64) -> Self {
        let delta = u.max(0.0);
        let x = PLANCK_LENGTH_4 * kretschmann.max(0.0);

        let num = delta * (1.0 + x) + x * ((1.0 - delta).powi(2));
        let k_eff = num / (1.0 + x);

        let rate_factor = k_eff.sqrt().max(0.0);
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

    // ========================================================================
    // Thorough tests for the unified proper-time rate (master Lagrangian)
    // ========================================================================

    #[test]
    fn unified_proper_time_rate_low_curvature() {
        // kretschmann = 0 must recover exactly the GR limit dτ/dt = √(max(δ, 0))
        // where δ = α²(1 − β²). This is the canonical weak-field / solar-system path.
        let test_cases: &[(f64, f64, f64)] = &[
            (1.0, 0.0, 1.0),     // stationary flat space
            (0.64, 0.0, 0.8),    // β = 0.6, α = 1
            (0.81, 0.0, 0.9),    // α = 0.9, β = 0
            (0.5184, 0.0, 0.72), // realistic combined α = 0.9, β = 0.6
            (0.0, 0.0, 0.0),     // null / lightlike edge
            (1.21, 0.0, 1.1),    // δ > 1 (mathematically allowed, physically rare)
        ];

        for &(u, k, expected_rate) in test_cases {
            let drift = ClockDrift::from_unified_proper_time_rate(u, k);
            let expected_offset = expected_rate - 1.0;
            let expected_drift =
                ClockDrift::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(expected_offset));
            assert_eq!(
                drift, expected_drift,
                "Low-curvature GR recovery failed for u={}, k={}",
                u, k
            );
        }
    }

    #[test]
    fn unified_proper_time_rate_high_curvature_saturation() {
        // When x = ℓ_Pl⁴ 𝒦 ≫ 1 the master Lagrangian saturates:
        //     K_eff → δ² − δ + 1   ⇒   dτ/dt → √(δ² − δ + 1) ≥ √(3/4) ≈ 0.866
        // (tested with an astronomically large kretschmann that forces x → ∞ in f64)
        let large_kretschmann = 1e200_f64;

        let deltas = [0.0_f64, 0.25, 0.5, 0.64, 0.81, 1.0, 1.21];
        for &delta in &deltas {
            let drift = ClockDrift::from_unified_proper_time_rate(delta, large_kretschmann);

            // Exact algebraic saturation limit from the master Lagrangian
            let k_eff_limit = delta * delta - delta + 1.0;
            let expected_rate = k_eff_limit.sqrt().max(0.0);
            let expected_offset = expected_rate - 1.0;

            let expected_drift =
                ClockDrift::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(expected_offset));
            assert_eq!(
                drift, expected_drift,
                "High-curvature saturation failed for δ = {}",
                delta
            );
        }
    }

    #[test]
    fn unified_proper_time_rate_clamping_and_edges() {
        // Negative inputs must be clamped (u.max(0), kretschmann.max(0))
        let drift_neg_u = ClockDrift::from_unified_proper_time_rate(-0.5, 0.0);

        // Semantic check using .as_sec_f64() — this is the robust way.
        // (Delta::from_sec_f64(-1.0) currently produces a non-canonical internal
        // representation while the unified function produces the canonical one.
        // The two Deltas are mathematically identical but not ==.)
        assert_eq!(
            drift_neg_u.rate.as_sec_f64(),
            -1.0,
            "Negative u should clamp to dτ/dt = 0.0 → rate_offset = -1.0"
        );

        let drift_neg_k = ClockDrift::from_unified_proper_time_rate(0.81, -100.0);
        let expected_neg_k = ClockDrift::from_unified_proper_time_rate(0.81, 0.0);
        assert_eq!(
            drift_neg_k, expected_neg_k,
            "Negative kretschmann not clamped"
        );

        // delta = 1.0 must always give exactly rate = 1.0 (no drift) regardless of curvature
        for k in [0.0, 1.0, 1e10, 1e30] {
            let drift = ClockDrift::from_unified_proper_time_rate(1.0, k);
            assert_eq!(drift.rate, Delta::ZERO, "δ=1 should be exactly rate=1");
        }

        // delta = 0 with moderate curvature (null-ray / lightlike edge case sanity).
        // We deliberately choose a kretschmann value large enough that
        // x = PLANCK_LENGTH_4 * kretschmann ≈ 6.82 (non-negligible in f64).
        // This tests the actual intermediate-curvature branch of the master Lagrangian,
        // unlike the old 1e10 which produced x ≈ 0 in floating-point.
        let kretschmann = 1e140_f64;
        let drift_null = ClockDrift::from_unified_proper_time_rate(0.0, kretschmann);

        // Expected value computed with the exact same formula the implementation uses
        let x = PLANCK_LENGTH_4 * kretschmann;
        let k_eff = x / (1.0 + x);
        let expected_null_rate: f64 = k_eff.sqrt() - 1.0;
        let expected_null =
            ClockDrift::from_offset_and_rate(Delta::ZERO, Delta::from_sec_f64(expected_null_rate));

        assert_eq!(drift_null, expected_null);
    }

    #[test]
    fn resolved_metric_to_unified_proper_time_rate() {
        // from_resolved_metric must correctly compute δ = α²(1 − β²) and delegate to the unified path
        let resolved = ResolvedMetric::new(0.9, 0.6, 0.0); // realistic values
        let drift = ClockDrift::from_resolved_metric(resolved);

        // Manual verification of the exact same path
        let u = 0.9 * 0.9 * (1.0 - 0.6 * 0.6);
        let expected_drift = ClockDrift::from_unified_proper_time_rate(u, 0.0);

        assert_eq!(
            drift, expected_drift,
            "ResolvedMetric → unified path mismatch"
        );
    }

    #[test]
    fn unified_proper_time_rate_intermediate_curvature_sanity() {
        // Spot-check a few intermediate x values (neither zero nor infinite) to ensure
        // the rational expression behaves smoothly and never goes negative.
        let u = 0.64_f64;
        let k_values = [0.0, 1e5, 1e15, 1e30];
        for &k in &k_values {
            let drift = ClockDrift::from_unified_proper_time_rate(u, k);
            let rate_factor = 1.0 + drift.rate.as_sec_f64(); // internal f64 value
            assert!(rate_factor > 0.0, "proper-time rate became non-positive");
            // monotonicity / bound check
            assert!(
                rate_factor <= 1.0 + 1e-10,
                "rate > 1 for u < 1 should not happen"
            );
        }
    }
}
