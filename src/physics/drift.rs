//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `Scale`).
//!
//! The underlying physical model (master Lagrangian, regimes, relationship to GR)
//! is documented in `docs/relativity.md`.

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

    /// Canonical low-level constructor that implements the library's general
    /// relativity formula.
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
    /// Builds a clock-drift model in which this [`Dt`] is treated as the
    /// initial fixed time difference between the observer’s proper time and
    /// the chosen coordinate time.
    ///
    /// In practice you often compute or measure a one-time offset (for example
    /// after a clock synchronization or a leap-second jump) and then want to
    /// combine it with a steady rate difference and any quadratic change.
    /// This method lets you do that directly from a [`Dt`] without having to
    /// call the more verbose [`Drift::new`].
    ///
    /// The other two arguments describe how the difference between the two
    /// clocks will evolve:
    /// - `rate` — the constant fractional speed difference (how much faster or
    ///   slower one clock runs compared with the other).
    /// - `accel` — how quickly that speed difference itself is changing (for
    ///   example because the spacecraft is moving through a varying gravitational
    ///   field).
    ///
    /// See [`Drift`] and [`Drift::from_offset_and_rate`] for more background on
    /// why these three numbers are used to model real clocks.
    #[inline]
    pub const fn to_drift_as_constant(self, rate: Dt, accel: Dt) -> Drift {
        Drift::new(self, rate, accel)
    }

    /// Builds a clock-drift model in which this [`Dt`] supplies the constant
    /// fractional rate difference between the observer’s proper time and the
    /// chosen coordinate time.
    ///
    /// If you have already calculated (or measured) a steady rate offset as a
    /// [`Dt`], you can use this method to attach an initial time offset and a
    /// quadratic term and obtain a complete [`Drift`] polynomial.
    ///
    /// Physically, the rate term captures the fact that two clocks that are
    /// moving at different velocities or sitting at different gravitational
    /// potentials will accumulate a steadily growing time difference. The
    /// other two parameters let you also describe any starting bias and any
    /// change in that rate over time.
    ///
    /// See the documentation on [`Drift`] for the meaning of the three
    /// coefficients in a relativistic timing context.
    #[inline]
    pub const fn to_drift_as_rate(self, constant: Dt, accel: Dt) -> Drift {
        Drift::new(constant, self, accel)
    }

    /// Builds a clock-drift model in which this [`Dt`] supplies the quadratic
    /// term that describes how the rate difference itself is changing.
    ///
    /// Some situations (a spacecraft on a highly elliptical orbit, a clock
    /// whose frequency is aging, or a trajectory that takes it through regions
    /// of changing gravitational potential) cause the *rate* at which two
    /// clocks diverge to change over time. If you have computed that changing
    /// rate as a [`Dt`], this method lets you combine it with an initial offset
    /// and a base rate to form a full [`Drift`].
    ///
    /// The other two arguments are:
    /// - `constant` — any fixed time bias present at the start.
    /// - `rate` — the base fractional rate difference that will itself be
    ///   modified by the quadratic term supplied by `self`.
    ///
    /// See [`Drift`] for more explanation of why a quadratic model is used for
    /// relativistic clock predictions.
    #[inline]
    pub const fn to_drift_as_accel(self, constant: Dt, rate: Dt) -> Drift {
        Drift::new(constant, rate, self)
    }
}
