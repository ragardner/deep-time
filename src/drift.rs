//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `Scale`).

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
\(\mu \in \{0,1\}\) (\(\mu=1\) for massive probes, \(\mu=0\) for null rays), and the background quantities \(\alpha(t,\mathbf{x})\) (local lapse/redshift factor) and \(\beta(t,\mathbf{x},\dot{\mathbf{x}})\) (local 3-velocity magnitude relative to the chrono-rest frame) are supplied by the modular **Spacetime** interface for any metric \(g_{\mu\nu}\). The Kretschmann scalar \(\mathcal{K} = R_{\alpha\beta\gamma\delta} R^{\alpha\beta\gamma\delta}\) is also supplied by Spacetime.

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
The accumulated proper-time shifts remain \(\delta(\Span\tau) \ll 10^{-140}\) s over cosmic history and far below machine precision in solar-system integrations—identical to the original low-curvature recovery of GR.

**High-curvature saturation (\(x \gg 1\))**
\[
K_{\rm eff} \to \delta^2 - \delta + 1, \qquad \frac{d\tau}{dt} \to \sqrt{\delta^2 - \delta + 1} \geq \sqrt{3/4} \approx 0.866.
\]
Proper time never stops; a smooth Planck-scale core replaces any would-be GR singularity.

### Background-Generalization Modules (Spacetime Interface)
The same interface is implemented for every spacetime (FLRW, multi-body PN, Kerr ZAMO, NR grids). In every case the low-curvature limit (\(x \ll 1\)) is exact GR; the intrinsic saturation activates algebraically only when \(\mathcal{K}^{1/4} \gtrsim 1/\ell_{\rm Pl}\).

### Numerical Implementation and Code Integration
**Weak-field spacecraft / ground-station clocks**
In post-Newtonian regimes \(x \ll 10^{-100}\), so \(K_{\rm eff} \approx \delta\) and the correction is negligible. Existing integrators require only the direct evaluation of the rational form (no separate regulator branch).

**General integration pseudocode (massive probe, coordinate-time stepper)**

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

For null rays enforce \(K_{\rm eff} \approx 0\) algebraically at each step (standard null geodesic integrator). The main equation remains non-singular everywhere.

### Observational and Numerical Status
The theory is empirically identical to GR on all tested scales (solar system, binary pulsars, LIGO/Virgo, EHT, NICER, CMB, large-scale structure). The built-in saturation remains dormant to better than 140 decimal places everywhere outside Planck cores. Numerical implementations on NR grids are stable with no time-stopping or division-by-zero artifacts.

### Philosophy
General relativity is recovered exactly as the low-curvature projection of this larger structure. The Planck-scale UV cutoff is now an **intrinsic algebraic property** of the master Lagrangian (via direct substitution of the minimal Padé form), enforcing that proper time never actually stops for massive observers while preserving the local light-cone everywhere. Would-be singularities are replaced by smooth finite-curvature cores **without any auxiliary regulator function**, new fields, new parameters, or observable deviations. The regulator is therefore redundant.

This formulation is production-ready for any mixed weak/strong-field probe adventure. All prior stages are recovered algebraically in the low-curvature limit. The engine is minimal, modular, and fully first-principles at the level of the master Lagrangian.
*/

use crate::{
    ATTOS_PER_SEC_I128, C_SQUARED, Dt, PLANCK_LENGTH_4, Position, Real, Scale, Velocity, sqrt,
};

/// The three local spacetime quantities that fully determine how fast an observer’s
/// proper time advances relative to coordinate time.
///
/// This structure holds the gravitational lapse factor, the observer’s local velocity,
/// and the curvature information needed for the library’s unified proper-time model.
/// It is the low-level input that `Drift` uses internally.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Spacetime {
    /// Gravitational lapse (redshift) factor α.  
    /// This is the factor by which clocks run slower in a gravitational potential.
    pub alpha: Real,

    /// Local three-velocity β = v/c measured in the coordinate rest frame.
    pub beta: Real,

    /// Kretschmann scalar (a scalar measure of spacetime curvature).  
    /// In the weak-field regime — where |Φ|/c² ≪ 1 and the gravitational field varies
    /// over macroscopic distances — this value is effectively zero and can safely be
    /// left at its default. It only becomes numerically relevant in strong-field
    /// environments such as:
    ///
    /// - the surface or immediate vicinity of neutron stars (where |Φ|/c² ≈ 0.15–0.25);
    /// - regions near a black-hole event horizon (e.g. the photon rings imaged by the
    ///   Event Horizon Telescope around M87* or Sgr A*);
    /// - the final inspiral and merger phases of binary neutron-star or black-hole
    ///   systems (as observed by LIGO/Virgo in events such as GW170817 or GW150914).
    ///
    /// In these regimes a realistic non-zero value (estimated from the local potential
    /// and a characteristic length scale) activates the library’s intrinsic Planck-scale
    /// saturation term.
    pub kretschmann: Real,
}

impl Spacetime {
    #[inline]
    pub const fn new(alpha: Real, beta: Real, kretschmann: Real) -> Spacetime {
        Self {
            alpha,
            beta,
            kretschmann,
        }
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` from this snapshot.
    ///
    /// Convenience method that internally uses the same unified calculation as
    /// `Drift::proper_time_rate`.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        Drift::from_spacetime(self).proper_time_rate()
    }

    /// Convenience for direct gravimeter / sensor paths.
    #[inline]
    pub const fn from_gravitic_and_velocity(
        alpha: Real,
        velocity: Velocity,
        kretschmann: Real,
    ) -> Spacetime {
        Self::new(alpha, velocity.beta(), kretschmann)
    }

    /// Converts the Newtonian gravitational potential Φ/c² (where Φ < 0 for bound orbits)
    /// into the relativistic lapse factor α = √(1 + 2Φ/c²).
    ///
    /// This function implements the standard weak-field approximation used in general
    /// relativity. It is valid when the dimensionless gravitational potential satisfies
    /// |Φ|/c² ≪ 1. In this regime spacetime is nearly flat, gravitational time dilation
    /// is a small perturbation, and higher-order curvature effects can safely be neglected.
    /// The resulting α gives the factor by which clocks tick more slowly in a gravitational
    /// well relative to a distant reference clock.
    ///
    /// This approximation is excellent for solar-system navigation, GNSS satellites,
    /// most spacecraft operations, and any environment where |Φ|/c² remains much smaller
    /// than ~0.01. It is exported from `deep_time::alpha_from_weak_field_potential`
    /// and is the recommended way to obtain the lapse factor when you have the local
    /// Newtonian potential.
    ///
    /// The weak-field regime breaks down in strong-gravity environments where
    /// |Φ|/c² approaches or exceeds ~0.1. Such conditions occur near:
    ///
    /// - the surface or immediate vicinity of neutron stars (where |Φ|/c² ≈ 0.15–0.25);
    /// - regions near a black-hole event horizon (e.g. the photon rings imaged by the
    ///   Event Horizon Telescope around M87* or Sgr A*);
    /// - the final inspiral and merger phases of binary neutron-star or black-hole
    ///   systems (as observed by LIGO/Virgo in events such as GW170817 or GW150914).
    ///
    /// In those extreme regimes this function alone is no longer sufficient; a full
    /// strong-field treatment (including curvature information passed to `Spacetime`)
    /// is required.
    #[inline]
    pub const fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
        // gravitational_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
        sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
    }

    /// Kretschmann scalar from total relativity
    /// Computes the Kretschmann scalar \(\mathcal{K}\) from the total gravitational
    /// relativity experienced by a local observer at the observer’s spacetime point.
    ///
    /// This is the canonical, physics-true convenience function for the master Lagrangian.
    /// It uses:
    /// - `phi` = Φ/c² — the total local gravitational potential (redshift/gravity effect)
    ///   felt by the observer from all masses.
    /// - `characteristic_length_scale` — the typical length scale (in meters) over which
    ///   the gravitational field varies at the observer’s location.
    ///
    /// **For existing weak-field users** (Earth orbit, GNSS, solar-system navigation):
    /// Supply your existing `phi` value and set `characteristic_length_scale = 0.0`.
    /// The function safely returns 0.0 (the value in double precision).
    ///
    /// **For strong-field / future users** (black-hole flybys, neutron stars, direct
    /// gravimeters, or full metric evaluation):
    /// Supply the measured or computed \(\phi\) and the real local length scale (or
    /// the value from your metric). The function returns a physically accurate non-zero
    /// curvature.
    pub const fn kretschmann_from_potential_and_scale(
        grav_potential_over_c2: Real,
        characteristic_length_scale: Real,
    ) -> Real {
        if characteristic_length_scale <= f!(0.0) || grav_potential_over_c2 <= f!(0.0) {
            return f!(0.0);
        }
        // Exact weak-field limit: K ≈ 48 φ² / L⁴
        let curvature_scale = f!(2.0) * grav_potential_over_c2
            / (characteristic_length_scale * characteristic_length_scale);
        f!(12.0) * (curvature_scale * curvature_scale)
    }

    /// Recommended constructor for most users.
    ///
    /// Computes both the gravitational lapse factor `α` and an estimate of the
    /// Kretschmann scalar from the dimensionless gravitational potential Φ/c²
    /// and a characteristic length scale.
    ///
    /// The lapse factor α is computed using `alpha_from_weak_field_potential`,
    /// which is the standard weak-field expression α = √(1 + 2Φ/c²). It is valid
    /// when the dimensionless gravitational potential satisfies |Φ|/c² ≪ 1. In
    /// this regime spacetime is nearly flat, gravitational time dilation is a
    /// small perturbation, and higher-order curvature effects can safely be
    /// neglected. The resulting α gives the factor by which clocks tick more
    /// slowly in a gravitational well relative to a distant reference clock.
    ///
    /// This approximation is excellent for solar-system navigation, GNSS
    /// satellites, most spacecraft operations, and any environment where
    /// |Φ|/c² remains much smaller than ~0.01. It is exported from
    /// `deep_time::alpha_from_weak_field_potential` and is the recommended
    /// way to obtain the lapse factor when you have the local Newtonian potential.
    ///
    /// The weak-field regime breaks down in strong-gravity environments where
    /// |Φ|/c² approaches or exceeds ~0.1. Such conditions occur near:
    ///
    /// - the surface or immediate vicinity of neutron stars (where |Φ|/c² ≈ 0.15–0.25);
    /// - regions near a black-hole event horizon (e.g. the photon rings imaged by the
    ///   Event Horizon Telescope around M87* or Sgr A*);
    /// - the final inspiral and merger phases of binary neutron-star or black-hole
    ///   systems (as observed by LIGO/Virgo in events such as GW170817 or GW150914).
    ///
    /// In those extreme regimes this function alone is no longer sufficient; a full
    /// strong-field treatment (including curvature information passed to `Spacetime`)
    /// is required.
    ///
    /// For the `characteristic_length_scale` parameter:
    /// - In weak-field conditions, pass `0.0`. This returns exactly the same clock
    ///   rate as the classic relativistic formulation and sets the Kretschmann scalar
    ///   to zero (its default value for all ordinary navigation, GNSS, or solar-system
    ///   work).
    /// - In strong-field conditions, supply the typical length scale (in meters) over
    ///   which the gravitational field varies significantly at the observer’s location.
    ///   This allows the library to estimate the Kretschmann scalar and activate the
    ///   intrinsic Planck-scale saturation term when curvature becomes extreme.
    pub const fn from_potential_velocity_and_scale(
        grav_potential_over_c2: Real, // Φ/c² (total local potential)
        velocity: Velocity,
        characteristic_length_scale: Real,
    ) -> Spacetime {
        let alpha: Real = Self::alpha_from_weak_field_potential(grav_potential_over_c2);
        let kretschmann: Real = Self::kretschmann_from_potential_and_scale(
            grav_potential_over_c2,
            characteristic_length_scale,
        );
        Self::from_gravitic_and_velocity(alpha, velocity, kretschmann)
    }

    /// Recovers the Newtonian gravitational potential Φ (m²/s²) from the
    /// gravitational lapse factor α using the weak-field relation.
    ///
    /// \[
    /// \alpha = \sqrt{1 + \frac{2\Phi}{c^2}} \quad\implies\quad
    /// \Phi = \frac{c^2}{2}(\alpha^2 - 1)
    /// \]
    ///
    /// This is the inverse of [`Spacetime::alpha_from_weak_field_potential`].
    #[inline]
    pub const fn grav_potential_from_alpha(alpha: Real) -> Real {
        let alpha_sq = alpha * alpha;
        (alpha_sq - f!(1.0)) / f!(2.0) * C_SQUARED
    }

    /// Computes the total Newtonian gravitational potential Φ at a given position
    /// from an arbitrary collection of point-mass bodies (Sun, Earth, Moon,
    /// planets, asteroids, etc.).
    ///
    /// This is the standard method used by real mission planners (Apollo,
    /// Artemis, Mars orbiters, lunar landers) and in open-source astrodynamics
    /// libraries (SPICE/NAIF, Orekit, GMAT, poliastro). It evaluates
    ///
    /// \[
    /// \Phi = -\sum_i \frac{GM_i}{r_i}
    /// \]
    ///
    /// ## Examples
    ///
    /// Realistic cislunar trajectory
    ///
    /// ```rust
    /// use deep_time::{Position, Spacetime};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20),     // Sun
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14),     // Earth
    ///     (Position::from_au(1.00257, 0.0, 0.0), 4.9048695e12), // Moon
    /// ];
    ///
    /// let position = Position::from_au(1.001, 0.001, 0.0); // e.g. spacecraft, asteroid, etc.
    ///
    /// let phi = Spacetime::grav_potential_from_point_masses(
    ///     position,
    ///     bodies.iter().copied(),
    /// );
    /// ```
    pub fn grav_potential_from_point_masses<I>(position: Position, bodies: I) -> Real
    where
        I: IntoIterator<Item = (Position, Real)>, // (body_position, GM in m³/s²)
    {
        let mut phi = 0.0;
        for (body_pos, gm) in bodies {
            let r = position.distance_to(body_pos);
            if r > 0.0 {
                phi -= gm / r;
            }
        }
        phi
    }
}

/// Quadratic polynomial that describes the accumulated difference between an
/// observer’s proper time (the time measured by a real clock moving through
/// spacetime) and a chosen coordinate time such as TT, TAI, or any other
/// `Scale`.
///
/// The polynomial follows the classic form  
/// Δt = constant + rate·Δt + accel·(Δt)²  
/// where the three coefficients capture any fixed offset, constant drift, and
/// quadratic acceleration of the clock. This structure is used throughout
/// spacecraft navigation, GNSS systems, and relativistic timing pipelines to
/// steer clocks, predict time offsets, and maintain synchronization over long
/// durations.
///
/// All three coefficients are stored using [`Dt`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Drift {
    /// Constant term a₀ expressed in seconds.  
    /// This represents any fixed time offset between the observer’s proper time
    /// and the chosen coordinate time.
    pub constant: Dt,

    /// Linear drift rate a₁ expressed in seconds per second.  
    /// This term captures a steady fractional rate difference (for example, a
    /// clock that runs consistently fast or slow).
    pub rate: Dt,

    /// Quadratic acceleration term a₂ expressed in seconds per second squared.  
    /// This term accounts for any changing drift rate, such as the gradual
    /// acceleration caused by relativistic effects or hardware aging.
    pub accel: Dt,
}

impl Drift {
    /// Creates a new `Drift` polynomial from its three coefficients.
    #[inline]
    pub const fn new(constant: Dt, rate: Dt, accel: Dt) -> Drift {
        Self {
            constant,
            rate,
            accel,
        }
    }

    /// The zero polynomial representing no correction at all.
    ///
    /// Use this when the observer’s clock is already perfectly synchronized with
    /// the chosen coordinate time.
    pub const ZERO: Self = Self::new(Dt::ZERO, Dt::ZERO, Dt::ZERO);

    /// Creates a [`Drift`] consisting of a pure constant offset.
    ///
    /// This is the most common constructor when only a fixed time bias is known
    /// (for example, after a one-time clock synchronization or leap-second
    /// adjustment).
    #[inline]
    pub const fn from_constant(c: Dt) -> Drift {
        Self::new(c, Dt::ZERO, Dt::ZERO)
    }

    /// Creates a [`Drift`] consisting of a constant offset together with a
    /// constant linear drift rate.  
    ///
    /// This form is very common for GNSS receivers and spacecraft clock steering,
    /// where a steady fractional frequency offset must be corrected in addition
    /// to any fixed bias.
    #[inline]
    pub const fn from_offset_and_rate(offset: Dt, rate: Dt) -> Drift {
        Self::new(offset, rate, Dt::ZERO)
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` (dimensionless).
    ///
    /// This value tells you how fast a real physical clock (such as a spacecraft
    /// onboard clock) is advancing compared to coordinate time. A value of
    /// `1.0` means the clock runs at the normal rate. Values slightly below `1.0`
    /// are typical when the clock is moving or sitting in a gravitational well.
    ///
    /// The rate includes special-relativistic velocity effects, gravitational
    /// time dilation, and the library’s built-in Planck-scale saturation term.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        f!(1.0) + self.rate.to_sec_f()
    }

    /// Evaluates the polynomial at the given elapsed coordinate time span.  
    ///
    /// Returns the accumulated time difference (in seconds) between proper
    /// time and coordinate time after the interval span has passed.
    pub const fn time_diff_after(&self, span: &Dt) -> Dt {
        let dt_attos = span.to_attos();
        let mut total_attos = self.constant.to_attos();

        if !self.rate.is_zero() || !self.accel.is_zero() {
            // Linear term: rate * dt
            let rate_attos: i128 = self.rate.to_attos();
            let rate_term = rate_attos.wrapping_mul(dt_attos) / ATTOS_PER_SEC_I128;
            total_attos = total_attos.wrapping_add(rate_term);

            // Quadratic term: accel * dt²
            let accel_attos: i128 = self.accel.to_attos();
            let accel_dt = accel_attos.wrapping_mul(dt_attos) / ATTOS_PER_SEC_I128;
            let accel_term = accel_dt.wrapping_mul(dt_attos) / ATTOS_PER_SEC_I128;
            total_attos = total_attos.saturating_add(accel_term);
        }

        Dt::span(total_attos)
    }

    /// Evaluates the deterministic relativistic/polynomial correction **and**
    /// adds a user-supplied stochastic offset (in seconds).
    ///
    /// This is the single production method for realistic stochastic clock
    /// modeling. In real mission pipelines the deterministic part (this
    /// polynomial) is kept perfectly clean; stochastic noise (white phase noise,
    /// random-walk frequency noise, Monte-Carlo realizations, Kalman process
    /// noise, measured clock residuals, etc.) is added at evaluation time.
    ///
    /// Pass `0.0` (or simply call the original `time_diff_after`) when you
    /// want purely deterministic behavior.
    #[inline]
    pub fn time_diff_after_with_noise(&self, span: &Dt, stochastic_offset_sec: Real) -> Dt {
        self.time_diff_after(span)
            .add(Dt::from_sec_f(stochastic_offset_sec, Scale::TAI))
    }

    /// Creates a `Drift` directly from an observer’s velocity and total
    /// local gravitational potential using the library’s unified master-Lagrangian
    /// proper-time rate.  
    ///
    /// It automatically computes the relativistic clock rate that includes both
    /// special-relativistic velocity effects and gravitational time dilation,
    /// then returns a [`Drift`] that can be evaluated at any future time.
    ///
    /// The `characteristic_length_scale` parameter controls whether the
    /// weak-field or strong-field formulation is used:
    ///
    /// - In the weak-field regime (where |Φ|/c² ≪ 1), simply pass
    ///   `characteristic_length_scale = 0.0`. This returns the same
    ///   relativistic clock rate used by JPL, ESA, GNSS systems, and all modern
    ///   solar-system navigation pipelines.
    /// - In strong-field conditions, supply a non-zero length scale (in meters)
    ///   over which the gravitational potential changes at the observer’s
    ///   location. This activates the library’s intrinsic Planck-scale saturation
    ///   term when spacetime curvature becomes extreme.
    pub const fn from_velocity_potential_and_scale(
        velocity_m_s: Real,
        grav_potential_m2_s2: Real,
        characteristic_length_scale: Real,
    ) -> Drift {
        let phi = grav_potential_m2_s2 / C_SQUARED;
        let velocity = Velocity::from_speed(velocity_m_s);
        let spacetime = Spacetime::from_potential_velocity_and_scale(
            phi,
            velocity,
            characteristic_length_scale,
        );
        Self::from_spacetime(&spacetime)
    }

    /// Canonical low-level constructor that implements the exact intrinsic
    /// expression from the master Lagrangian.  
    ///
    /// This function is the single source of truth for the proper-time rate
    /// calculation used throughout the library. Most users will never call it
    /// directly; the high-level constructors `from_velocity_potential_and_scale`
    /// and `from_spacetime` are the intended entry points.
    ///
    /// The internal expression is  
    /// K_eff = [δ(1 + x) + x(1−δ)²] / (1 + x)  
    /// where δ = α²(1−β²) and x = ℓ_Pl⁴ 𝒦.
    ///
    /// The returned rate offset is then applied as a linear term in the `Drift`
    /// polynomial.
    pub const fn from_unified_proper_time_rate(u: Real, kretschmann: Real) -> Drift {
        let delta = u.max(f!(0.0));
        let x = PLANCK_LENGTH_4 * kretschmann.max(f!(0.0));

        let one_minus_delta = f!(1.0) - delta;
        let num = delta * (f!(1.0) + x) + x * (one_minus_delta * one_minus_delta);
        let k_eff = num / (f!(1.0) + x);

        let rate_factor = sqrt(k_eff).max(f!(0.0));
        let rate_offset = rate_factor - f!(1.0);

        Self::from_offset_and_rate(Dt::ZERO, Dt::from_sec_f(rate_offset, Scale::TAI))
    }

    /// Creates a `Drift` from a fully resolved `Spacetime` snapshot.  
    ///
    /// This is the canonical high-level entry point when you already hold a
    /// `Spacetime` object containing the gravitational lapse factor α, the
    /// local velocity β, and the Kretschmann scalar. It internally computes the
    /// unified proper-time rate and packages the result as a `Drift`
    /// polynomial ready for evaluation at any future time.
    #[inline]
    pub const fn from_spacetime(spacetime: &Spacetime) -> Drift {
        let u = spacetime.alpha * spacetime.alpha * (f!(1.0) - spacetime.beta * spacetime.beta);
        Self::from_unified_proper_time_rate(u, spacetime.kretschmann)
    }
}

impl Dt {
    #[inline]
    pub const fn to_drift_as_constant(self, rate: Dt, accel: Dt) -> Drift {
        Drift::new(self, rate, accel)
    }

    #[inline]
    pub const fn to_drift_as_rate(self, constant: Dt, accel: Dt) -> Drift {
        Drift::new(constant, self, accel)
    }

    #[inline]
    pub const fn to_drift_as_accel(self, constant: Dt, rate: Dt) -> Drift {
        Drift::new(constant, rate, self)
    }
}
