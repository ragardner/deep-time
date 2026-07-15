//! Quadratic polynomial for relativistic corrections, clock drift, and custom timescale steering.
//!
//! Used to model the accumulated difference between Proper time (τ)
//! and a coordinate time such as TT (or any other `Scale`).
//!
//! Information on the underlying physical model (the master Lagrangian, different
//! regimes of behavior, and its relationship to general relativity) can be found
//! [here](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

use crate::{
    ATTOS_PER_SEC_I128, C_SQUARED, Dt, PLANCK_LENGTH_4, Real, Scale, Spacetime, Velocity, dt, sqrt,
};

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

        dt!(total_attos)
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
        self.time_diff_after(span).add(Dt::from_sec_f(
            stochastic_offset_sec,
            Scale::TAI,
            Scale::TAI,
        ))
    }

    /// Build a linear-rate [`Drift`] from speed (m/s) and SI potential Φ (m²/s²).
    ///
    /// Given how fast you move and how deep you sit in gravity, return a
    /// [`Drift`] whose rate term matches the library’s proper-time model
    /// (special-relativistic and gravitational effects). Useful when you want
    /// the rate as a polynomial coefficient rather than integrating a path.
    ///
    /// ## `characteristic_length_scale`
    ///
    /// Pass **`0.0`** for ordinary weak-field work (Earth orbit, solar system):
    /// Kretschmann is zero and the rate is the first-order weak-field form.
    /// Pass a positive length (meters) only if you want the optional curvature
    /// estimate (see [`Spacetime::kretschmann_from_potential_and_scale`]).
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

        Self::from_offset_and_rate(
            Dt::ZERO,
            Dt::from_sec_f(rate_offset, Scale::TAI, Scale::TAI),
        )
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

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// derived from the supplied `Spacetime` model.
    ///
    /// - This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// - For a local hardware proper-time clock, use the plain `add` methods instead.
    #[inline]
    pub const fn adjusted_advance(&mut self, elapsed: &Dt, spacetime: &Spacetime) {
        let dtau = elapsed.add(Drift::from_spacetime(spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// from a pre-computed `Drift` value.
    ///
    /// - This is an optimized variant of [`Dt::adjusted_advance`](../struct.Dt.html#method.adjusted_advance)
    ///   for callers that already hold a [`Drift`] instance.
    /// - This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// - For a local hardware proper-time clock, use the plain `add` methods instead.
    #[inline]
    pub const fn adjusted_advance_using_drift(&mut self, elapsed: &Dt, drift: &Drift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Converts this instant to any other [`Scale`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`Drift`] model relative to a reference instant.
    pub const fn convert_using_drift(self, reference: Dt, drift: &Drift) -> Dt {
        let span = self.to_diff_raw(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Performs the inverse conversion of [`Dt::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    pub const fn convert_back_using_drift(self, reference: Dt, drift: &Drift) -> Dt {
        if drift.rate.is_zero() && drift.accel.is_zero() {
            return self.sub(drift.constant);
        }
        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            let span = guess.to_diff_raw(reference);
            let correction = drift.time_diff_after(&span);
            guess = self.sub(correction);
            i += 1;
        }
        guess
    }
}

#[cfg(feature = "wire")]
impl Drift {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 3 * Dt::WIRE_SIZE;

    /// Serializes this [`Drift`] polynomial into a fixed buffer.
    ///
    /// The layout is the concatenation of the three `Dt` fields.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        let c = self.constant.to_wire_bytes();
        let r = self.rate.to_wire_bytes();
        let a = self.accel.to_wire_bytes();

        buf[0..Dt::WIRE_SIZE].copy_from_slice(&c);
        buf[Dt::WIRE_SIZE..2 * Dt::WIRE_SIZE].copy_from_slice(&r);
        buf[2 * Dt::WIRE_SIZE..].copy_from_slice(&a);
        buf
    }

    /// Deserializes a [`Drift`] from exactly `WIRE_SIZE` bytes of wire data.
    ///
    /// Returns `None` if any nested `Dt` fails validation or if the version
    /// byte is unknown.
    ///
    /// ## Security
    ///
    /// Composes the safety guarantees of
    /// [`Dt::from_wire_bytes`](../struct.Dt.html#method.from_wire_bytes).
    ///
    /// Fixed size and layered validation make it safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let constant = Dt::from_wire_bytes(&bytes[0..Dt::WIRE_SIZE])?;
        let rate = Dt::from_wire_bytes(&bytes[Dt::WIRE_SIZE..2 * Dt::WIRE_SIZE])?;
        let accel = Dt::from_wire_bytes(&bytes[2 * Dt::WIRE_SIZE..])?;

        Some(Self::new(constant, rate, accel))
    }
}
