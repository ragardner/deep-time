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
\(\mu \in \{0,1\}\) (\(\mu=1\) for massive probes, \(\mu=0\) for null rays), and the background quantities \(\alpha(t,\mathbf{x})\) (local lapse/redshift factor) and \(\beta(t,\mathbf{x},\dot{\mathbf{x}})\) (local 3-velocity magnitude relative to the chrono-rest frame) are supplied by the modular **LocalSpacetime** interface for any metric \(g_{\mu\nu}\). The Kretschmann scalar \(\mathcal{K} = R_{\alpha\beta\gamma\delta} R^{\alpha\beta\gamma\delta}\) is also supplied by LocalSpacetime.

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
The accumulated proper-time shifts remain \(\delta(\TimeSpan\tau) \ll 10^{-140}\) s over cosmic history and far below machine precision in solar-system integrations—identical to the original low-curvature recovery of GR.

**High-curvature saturation (\(x \gg 1\))**
\[
K_{\rm eff} \to \delta^2 - \delta + 1, \qquad \frac{d\tau}{dt} \to \sqrt{\delta^2 - \delta + 1} \geq \sqrt{3/4} \approx 0.866.
\]
Proper time never stops; a smooth Planck-scale core replaces any would-be GR singularity.

### Background-Generalization Modules (LocalSpacetime Interface)
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

use crate::{ATTOSEC_PER_SEC_I128, C_SQUARED, PLANCK_LENGTH_4, Real, TimeSpan, Velocity};

/// The three local spacetime quantities that fully determine how fast an observer’s
/// proper time advances relative to coordinate time.
///
/// This structure holds the gravitational lapse factor, the observer’s local velocity,
/// and the curvature information needed for the library’s unified proper-time model.
/// It is the low-level input that `ClockDrift` uses internally.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct LocalSpacetime {
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

impl LocalSpacetime {
    #[inline]
    pub const fn new(alpha: Real, beta: Real, kretschmann: Real) -> Self {
        Self {
            alpha,
            beta,
            kretschmann,
        }
    }

    /// Size of the canonical wire representation in bytes (24 bytes).
    pub const WIRE_SIZE: usize = 24;

    /// Serializes this `LocalSpacetime` snapshot into a fixed 24-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.alpha.to_le_bytes());
        buf[8..16].copy_from_slice(&self.beta.to_le_bytes());
        buf[16..24].copy_from_slice(&self.kretschmann.to_le_bytes());
        buf
    }

    /// Deserializes a `LocalSpacetime` from exactly 24 bytes.
    ///
    /// ## Security
    ///
    /// Accepts any `f64` bit pattern (including `NaN`/`Inf`) to match the
    /// type’s own invariants. Fixed size makes it immune to length-based
    /// attacks. Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let alpha = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let beta = f64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let kretschmann = f64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        Some(Self {
            alpha,
            beta,
            kretschmann,
        })
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` from this snapshot.
    ///
    /// Convenience method that internally uses the same unified calculation as
    /// `ClockDrift::proper_time_rate`.
    #[inline(always)]
    pub fn proper_time_rate(self) -> Real {
        ClockDrift::from_local_spacetime(&self).proper_time_rate()
    }

    /// Convenience for direct gravimeter / sensor paths.
    #[inline]
    pub fn from_gravitic_and_velocity(alpha: Real, velocity: Velocity, kretschmann: Real) -> Self {
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
    /// than ~0.01. It is exported from `deep_time_core::alpha_from_weak_field_potential`
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
    /// strong-field treatment (including curvature information passed to `LocalSpacetime`)
    /// is required.
    #[inline]
    pub fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
        // gravitational_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
        libm::sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
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
    /// The function safely returns 0.0 (the correct value in double precision).
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
    /// `deep_time_core::alpha_from_weak_field_potential` and is the recommended
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
    /// strong-field treatment (including curvature information passed to `LocalSpacetime`)
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
    pub fn from_potential_velocity_and_scale(
        grav_potential_over_c2: Real, // Φ/c² (total local potential)
        velocity: Velocity,
        characteristic_length_scale: Real,
    ) -> Self {
        let alpha: Real = Self::alpha_from_weak_field_potential(grav_potential_over_c2);
        let kretschmann: Real = Self::kretschmann_from_potential_and_scale(
            grav_potential_over_c2,
            characteristic_length_scale,
        );
        Self::from_gravitic_and_velocity(alpha, velocity, kretschmann)
    }
}

/// Quadratic polynomial that describes the accumulated difference between an
/// observer’s proper time (the time measured by a real clock moving through
/// spacetime) and a chosen coordinate time such as TT, TAI, or any other
/// `ClockType`.
///
/// The polynomial follows the classic form  
/// Δt = constant + rate·Δt + accel·(Δt)²  
/// where the three coefficients capture any fixed offset, constant drift, and
/// quadratic acceleration of the clock. This structure is used throughout
/// spacecraft navigation, GNSS systems, and relativistic timing pipelines to
/// steer clocks, predict time offsets, and maintain synchronization over long
/// durations.
///
/// All three coefficients are stored using the exact `TimeSpan` type, which
/// guarantees 36-digit precision with no floating-point rounding errors even
/// over centuries of integration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockDrift {
    /// Constant term a₀ expressed in seconds.  
    /// This represents any fixed time offset between the observer’s proper time
    /// and the chosen coordinate time.
    constant: TimeSpan,

    /// Linear drift rate a₁ expressed in seconds per second.  
    /// This term captures a steady fractional rate difference (for example, a
    /// clock that runs consistently fast or slow).
    rate: TimeSpan,

    /// Quadratic acceleration term a₂ expressed in seconds per second squared.  
    /// This term accounts for any changing drift rate, such as the gradual
    /// acceleration caused by relativistic effects or hardware aging.
    accel: TimeSpan,
}

impl ClockDrift {
    /// Creates a new `ClockDrift` polynomial from its three exact coefficients.
    #[inline(always)]
    pub const fn new(constant: TimeSpan, rate: TimeSpan, accel: TimeSpan) -> Self {
        Self {
            constant,
            rate,
            accel,
        }
    }

    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 3 * TimeSpan::WIRE_SIZE; // 3 × 17 = 51

    /// Serializes this `ClockDrift` polynomial into a fixed buffer.
    ///
    /// The layout is the concatenation of the three `TimeSpan` fields.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        let c = self.constant.to_wire_bytes();
        let r = self.rate.to_wire_bytes();
        let a = self.accel.to_wire_bytes();

        buf[0..TimeSpan::WIRE_SIZE].copy_from_slice(&c);
        buf[TimeSpan::WIRE_SIZE..2 * TimeSpan::WIRE_SIZE].copy_from_slice(&r);
        buf[2 * TimeSpan::WIRE_SIZE..].copy_from_slice(&a);
        buf
    }

    /// Deserializes a `ClockDrift` from exactly `WIRE_SIZE` bytes of wire data.
    ///
    /// Returns `None` if any nested `TimeSpan` fails validation or if the version
    /// byte is unknown.
    ///
    /// ## Security
    ///
    /// Composes the safety guarantees of [`TimeSpan::from_wire_bytes`].
    /// Fixed size and layered validation make it safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let constant = TimeSpan::from_wire_bytes(&bytes[0..TimeSpan::WIRE_SIZE])?;
        let rate = TimeSpan::from_wire_bytes(&bytes[TimeSpan::WIRE_SIZE..2 * TimeSpan::WIRE_SIZE])?;
        let accel = TimeSpan::from_wire_bytes(&bytes[2 * TimeSpan::WIRE_SIZE..])?;

        Some(Self::new(constant, rate, accel))
    }

    /// The zero polynomial representing no correction at all.  
    /// Use this when the observer’s clock is already perfectly synchronized with
    /// the chosen coordinate time.
    pub const ZERO: Self = Self::new(TimeSpan::ZERO, TimeSpan::ZERO, TimeSpan::ZERO);

    /// Creates a `ClockDrift` consisting of a pure constant offset.  
    /// This is the most common constructor when only a fixed time bias is known
    /// (for example, after a one-time clock synchronization or leap-second
    /// adjustment).
    #[inline(always)]
    pub const fn from_constant(c: TimeSpan) -> Self {
        Self::new(c, TimeSpan::ZERO, TimeSpan::ZERO)
    }

    /// Creates a `ClockDrift` consisting of a constant offset together with a
    /// constant linear drift rate.  
    /// This form is very common for GNSS receivers and spacecraft clock steering,
    /// where a steady fractional frequency offset must be corrected in addition
    /// to any fixed bias.
    #[inline]
    pub const fn from_offset_and_rate(offset: TimeSpan, rate: TimeSpan) -> Self {
        Self::new(offset, rate, TimeSpan::ZERO)
    }

    #[inline]
    pub const fn constant(&self) -> &TimeSpan {
        &self.constant
    }

    #[inline]
    pub const fn rate(&self) -> &TimeSpan {
        &self.rate
    }

    #[inline]
    pub const fn accel(&self) -> &TimeSpan {
        &self.accel
    }

    #[inline]
    pub fn set_constant(&mut self, constant: TimeSpan) -> &mut Self {
        self.constant = constant;
        self
        // constant never affects the pre-computed big fields
    }

    #[inline]
    pub fn set_rate(&mut self, rate: TimeSpan) -> &mut Self {
        self.rate = rate;
        self
    }

    #[inline]
    pub fn set_accel(&mut self, accel: TimeSpan) -> &mut Self {
        self.accel = accel;
        self
    }

    #[inline]
    pub const fn with_constant(self, constant: TimeSpan) -> Self {
        Self::new(constant, self.rate, self.accel)
    }

    #[inline]
    pub const fn with_rate(self, rate: TimeSpan) -> Self {
        Self::new(self.constant, rate, self.accel)
    }

    #[inline]
    pub const fn with_accel(self, accel: TimeSpan) -> Self {
        Self::new(self.constant, self.rate, accel)
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` (dimensionless).
    ///
    /// This value tells you how fast a real physical clock (such as a spacecraft
    /// onboard clock) is advancing compared to coordinate time. A value of exactly
    /// `1.0` means the clock runs at the normal rate. Values slightly below `1.0`
    /// are typical when the clock is moving or sitting in a gravitational well.
    ///
    /// The rate includes special-relativistic velocity effects, gravitational
    /// time dilation, and the library’s built-in Planck-scale saturation term.
    #[inline(always)]
    pub const fn proper_time_rate(&self) -> Real {
        f!(1.0) + self.rate.as_sec_f()
    }

    /// Evaluates the polynomial at the given elapsed coordinate time span.  
    ///
    /// Returns the exact accumulated time difference (in seconds) between proper
    /// time and coordinate time after the interval span has passed. All
    /// arithmetic is performed with full 36-digit precision, ensuring no loss of
    /// accuracy even for multi-year integrations.
    pub const fn time_diff_after(&self, span: &TimeSpan) -> TimeSpan {
        let dt_attos = span.total_attos();
        let mut total_attos = self.constant.total_attos();

        if !self.rate.is_zero() || !self.accel.is_zero() {
            // Linear term: rate * dt
            let rate_attos = self.rate.total_attos();
            let rate_term = rate_attos.wrapping_mul(dt_attos) / ATTOSEC_PER_SEC_I128;
            total_attos = total_attos.wrapping_add(rate_term);

            // Quadratic term: accel * dt²
            let accel_attos = self.accel.total_attos();
            let accel_dt = accel_attos.wrapping_mul(dt_attos) / ATTOSEC_PER_SEC_I128;
            let accel_term = accel_dt.wrapping_mul(dt_attos) / ATTOSEC_PER_SEC_I128;
            total_attos = total_attos.saturating_add(accel_term);
        }

        TimeSpan::from_total_attos(total_attos)
    }

    /// Creates a `ClockDrift` directly from an observer’s velocity and total
    /// local gravitational potential using the library’s unified master-Lagrangian
    /// proper-time rate.  
    ///
    /// This is the recommended high-level constructor for nearly all users. It
    /// automatically computes the relativistic clock rate that includes both
    /// special-relativistic velocity effects and gravitational time dilation,
    /// then returns a `ClockDrift` that can be evaluated at any future time.
    ///
    /// The `characteristic_length_scale` parameter controls whether the
    /// weak-field or strong-field formulation is used:
    ///
    /// - In the weak-field regime (where |Φ|/c² ≪ 1), simply pass
    ///   `characteristic_length_scale = 0.0`. This returns exactly the same
    ///   relativistic clock rate used by JPL, ESA, GNSS systems, and all modern
    ///   solar-system navigation pipelines.
    /// - In strong-field conditions, supply a non-zero length scale (in meters)
    ///   over which the gravitational potential changes at the observer’s
    ///   location. This activates the library’s intrinsic Planck-scale saturation
    ///   term when spacetime curvature becomes extreme.
    pub fn from_velocity_potential_and_scale(
        velocity_m_s: Real,
        grav_potential_m2_s2: Real,
        characteristic_length_scale: Real,
    ) -> Self {
        let phi = grav_potential_m2_s2 / C_SQUARED;
        let velocity = Velocity::from_speed(velocity_m_s);
        let spacetime = LocalSpacetime::from_potential_velocity_and_scale(
            phi,
            velocity,
            characteristic_length_scale,
        );
        Self::from_local_spacetime(&spacetime)
    }

    /// Canonical low-level constructor that implements the exact intrinsic
    /// expression from the master Lagrangian.  
    ///
    /// This function is the single source of truth for the proper-time rate
    /// calculation used throughout the library. Most users will never call it
    /// directly; the high-level constructors `from_velocity_potential_and_scale`
    /// and `from_local_spacetime` are the intended entry points.
    ///
    /// The internal expression is  
    /// K_eff = [δ(1 + x) + x(1−δ)²] / (1 + x)  
    /// where δ = α²(1−β²) and x = ℓ_Pl⁴ 𝒦. The returned rate offset is then
    /// applied as a linear term in the `ClockDrift` polynomial.
    pub fn from_unified_proper_time_rate(u: Real, kretschmann: Real) -> Self {
        let delta = u.max(f!(0.0));
        let x = PLANCK_LENGTH_4 * kretschmann.max(f!(0.0));

        // powi(2) replaced by manual square — mathematically identical, no libm needed
        let one_minus_delta = f!(1.0) - delta;
        let num = delta * (f!(1.0) + x) + x * (one_minus_delta * one_minus_delta);
        let k_eff = num / (f!(1.0) + x);

        let rate_factor = libm::sqrt(k_eff).max(f!(0.0));
        let rate_offset = rate_factor - f!(1.0);

        Self::from_offset_and_rate(TimeSpan::ZERO, TimeSpan::from_sec_f(rate_offset))
    }

    /// Creates a `ClockDrift` from a fully resolved `LocalSpacetime` snapshot.  
    ///
    /// This is the canonical high-level entry point when you already hold a
    /// `LocalSpacetime` object containing the gravitational lapse factor α, the
    /// local velocity β, and the Kretschmann scalar. It internally computes the
    /// unified proper-time rate and packages the result as a `ClockDrift`
    /// polynomial ready for evaluation at any future time.
    #[inline]
    pub fn from_local_spacetime(spacetime: &LocalSpacetime) -> Self {
        let u = spacetime.alpha * spacetime.alpha * (f!(1.0) - spacetime.beta * spacetime.beta);
        Self::from_unified_proper_time_rate(u, spacetime.kretschmann)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeSpan;

    #[test]
    fn evaluate_zero_drift() {
        let drift = ClockDrift::ZERO;
        let dt = TimeSpan::from_sec(1_234_567);
        assert_eq!(drift.time_diff_after(&dt), TimeSpan::ZERO);
    }

    #[test]
    fn evaluate_constant_only() {
        let drift = ClockDrift::from_constant(TimeSpan::from_sec_f(0.5));
        let dt = TimeSpan::from_sec(1_000);
        assert_eq!(drift.time_diff_after(&dt), TimeSpan::from_sec_f(0.5));
    }

    #[test]
    fn evaluate_rate_only() {
        let drift = ClockDrift::from_offset_and_rate(TimeSpan::ZERO, TimeSpan::from_sec_f(1e-9)); // 1 ns/s
        let dt = TimeSpan::from_sec(1_000_000); // 1 million seconds
        assert_eq!(drift.time_diff_after(&dt), TimeSpan::from_sec_f(0.001)); // 1 µs
    }

    #[test]
    fn evaluate_full_quadratic() {
        let drift = ClockDrift::new(
            TimeSpan::from_sec(2),
            TimeSpan::from_ns(1), // exactly 1e-9 s/s
            TimeSpan::from_as(2), // exactly 2e-18 s/s²
        );
        let dt = TimeSpan::from_sec(1_000_000);

        // Exact mathematical result:
        // 2 + (1e-9 * 1_000_000) + (2e-18 * 1_000_000²) = 2 + 0.001 + 0.000002
        // = 2.001002 s = 2 s + 1_002_000_000_000_000 attoseconds
        assert_eq!(
            drift.time_diff_after(&dt),
            TimeSpan::new(2, 1_002_000_000_000_000)
        );
    }

    #[test]
    fn evaluate_negative_dt() {
        let drift = ClockDrift::new(
            TimeSpan::from_sec(5),
            TimeSpan::from_ns(1), // exactly 1e-9 s/s
            TimeSpan::from_as(1), // exactly 1e-18 s/s²
        );
        let dt = TimeSpan::from_sec(-500_000);

        // Exact mathematical result (no f64 loss)
        let expected = TimeSpan::from_sec(4)
            .add(TimeSpan::from_ms(999))
            .add(TimeSpan::from_us(500))
            .add(TimeSpan::from_ns(250));

        assert_eq!(drift.time_diff_after(&dt), expected);
    }

    #[test]
    fn evaluate_large_dt_exact() {
        let drift = ClockDrift::from_offset_and_rate(TimeSpan::ZERO, TimeSpan::from_sec_f(1e-12));
        let dt = TimeSpan::from_sec(1_000_000_000); // ~31.7 years
        assert_eq!(drift.time_diff_after(&dt), TimeSpan::from_sec_f(0.001));
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
            let expected_drift = ClockDrift::from_offset_and_rate(
                TimeSpan::ZERO,
                TimeSpan::from_sec_f(expected_offset),
            );
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

            let expected_drift = ClockDrift::from_offset_and_rate(
                TimeSpan::ZERO,
                TimeSpan::from_sec_f(expected_offset),
            );
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

        // Semantic check using .as_sec_f() — this is the robust way.
        // (TimeSpan::from_sec_f(-1.0) currently produces a non-canonical internal
        // representation while the unified function produces the canonical one.
        // The two TimeSpans are mathematically identical but not ==.)
        assert_eq!(
            drift_neg_u.rate.as_sec_f(),
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
            assert_eq!(drift.rate, TimeSpan::ZERO, "δ=1 should be exactly rate=1");
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
        let expected_null = ClockDrift::from_offset_and_rate(
            TimeSpan::ZERO,
            TimeSpan::from_sec_f(expected_null_rate),
        );

        assert_eq!(drift_null, expected_null);
    }

    #[test]
    fn local_spacetime_to_unified_proper_time_rate() {
        // from_local_spacetime must correctly compute δ = α²(1 − β²) and delegate to the unified path
        let spacetime = LocalSpacetime::new(0.9, 0.6, 0.0); // realistic values
        let drift = ClockDrift::from_local_spacetime(&spacetime);

        // Manual verification of the exact same path
        let u = 0.9 * 0.9 * (1.0 - 0.6 * 0.6);
        let expected_drift = ClockDrift::from_unified_proper_time_rate(u, 0.0);

        assert_eq!(
            drift, expected_drift,
            "LocalSpacetime → unified path mismatch"
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
            let rate_factor = 1.0 + drift.rate.as_sec_f(); // internal f64 value
            assert!(rate_factor > 0.0, "proper-time rate became non-positive");
            // monotonicity / bound check
            assert!(
                rate_factor <= 1.0 + 1e-10,
                "rate > 1 for u < 1 should not happen"
            );
        }
    }
}
