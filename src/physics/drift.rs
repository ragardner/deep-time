//! Clock polynomial for the difference between proper time and a coordinate
//! time.
//!
//! Instantaneous proper-time rates from [`Spacetime`] use the 3+1 interval
//! \(d\tau/dt=\alpha\sqrt{1-\beta^2}\). With α and β from Φ and \(v\), the
//! \(O(c^{-2})\) expansion of that interval is IERS Conventions (2010)
//! eqs. (10.6)–(10.7) / Ashby (2003). A [`Drift`] can also hold measured clock
//! bias, aging, or other steering that is not that interval. See
//! [docs/relativity.md](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

use crate::{ATTOS_PER_SEC_I128, Dt, Real, Scale, dt};

use super::Spacetime;

/// Quadratic polynomial for the accumulated difference between an observer’s
/// proper time (what a real clock measures) and a chosen coordinate time such
/// as TT, TAI, or any other [`Scale`].
///
/// The form is \(\mathrm{offset} = a_0 + a_1 s + a_2 s^2\), where \(s\) is
/// elapsed coordinate time. The three coefficients are a fixed offset, a
/// constant fractional rate, and a quadratic term (aging, or a changing rate).
/// GNSS and spacecraft clock steering use this polynomial.
///
/// All three coefficients are stored as [`Dt`]. The rate coefficient is
/// dimensionless (seconds per second); the acceleration coefficient is seconds
/// per second squared. [`from_spacetime`](Self::from_spacetime) fills only the
/// linear term from the general-relativity interval.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Drift {
    /// Constant term \(a_0\), a fixed time offset between proper time and the
    /// chosen coordinate time.
    pub constant: Dt,

    /// Linear coefficient \(a_1\), a fractional rate in seconds per second
    /// (for example a clock that runs steadily fast or slow).
    pub rate: Dt,

    /// Quadratic coefficient \(a_2\), in seconds per second squared (aging, or
    /// a rate that itself changes). [`from_spacetime`](Self::from_spacetime)
    /// leaves this at zero.
    pub accel: Dt,
}

impl Drift {
    /// Creates a `Drift` polynomial from its three coefficients.
    ///
    /// `constant` is a time offset, `rate` is dimensionless (seconds per
    /// second), and `accel` is seconds per second squared. All three are
    /// stored as [`Dt`].
    #[inline]
    pub const fn new(constant: Dt, rate: Dt, accel: Dt) -> Drift {
        Self {
            constant,
            rate,
            accel,
        }
    }

    /// Polynomial with all coefficients zero, meaning no correction at all.
    ///
    /// Use this when the observer’s clock is already synchronized with the
    /// chosen coordinate time.
    pub const ZERO: Self = Self::new(Dt::ZERO, Dt::ZERO, Dt::ZERO);

    /// Creates a [`Drift`] consisting of a pure constant offset.
    ///
    /// This is the usual constructor when only a fixed time bias is known
    /// (for example after a one-time clock synchronization).
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

    /// Instantaneous rate \(d\tau/dt\) implied by this polynomial’s linear
    /// term (`1 + rate`, dimensionless).
    ///
    /// When this `Drift` was built with [`from_spacetime`](Self::from_spacetime),
    /// that term is the general-relativity interval. Otherwise it is the rate
    /// you stored (steering, a measured frequency offset, aging). `1.0` means
    /// the linear term ticks in step with the coordinate time the polynomial
    /// is written against. The constant and quadratic coefficients do not
    /// enter this value.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        f!(1.0) + self.rate.to_sec_f()
    }

    /// Evaluates the polynomial after `span` of coordinate time.
    ///
    /// The result is \(a_0 + a_1 s + a_2 s^2\) as a [`Dt`]. When this
    /// polynomial was built with [`from_spacetime`](Self::from_spacetime),
    /// that value is \(\Delta\tau - \Delta t\). Otherwise it is whatever
    /// offset, rate, and aging you stored (steering, a measured frequency
    /// offset, and so on).
    ///
    /// Arithmetic saturates like [`Dt`] add and mul. Scaled products
    /// \((a\cdot b)/10^{18}\) avoid wrapping or early-clamping the
    /// intermediate \(a\cdot b\) when it exceeds `i128` but the result still
    /// fits.
    pub const fn time_diff_after(&self, span: &Dt) -> Dt {
        let dt_attos = span.to_attos();
        let mut total_attos = self.constant.to_attos();

        if !self.rate.is_zero() || !self.accel.is_zero() {
            // Linear: rate * dt  →  (rate_attos * dt_attos) / 10¹⁸
            let rate_term = saturating_mul_div_attos_per_sec(self.rate.to_attos(), dt_attos);
            total_attos = total_attos.saturating_add(rate_term);

            // Quadratic: accel * dt²  →  two successive scaled multiplies
            let accel_dt = saturating_mul_div_attos_per_sec(self.accel.to_attos(), dt_attos);
            let accel_term = saturating_mul_div_attos_per_sec(accel_dt, dt_attos);
            total_attos = total_attos.saturating_add(accel_term);
        }

        dt!(total_attos)
    }

    /// Adds `stochastic_offset_sec` to the result of
    /// [`time_diff_after`](Self::time_diff_after).
    ///
    /// The polynomial is left as you stored it. Pass noise at evaluation time
    /// (measured residuals, a Monte-Carlo draw, and so on). Pass `0.0` to get
    /// the same result as [`time_diff_after`](Self::time_diff_after).
    #[inline]
    pub fn time_diff_after_with_noise(&self, span: &Dt, stochastic_offset_sec: Real) -> Dt {
        self.time_diff_after(span).add(Dt::from_sec_f(
            stochastic_offset_sec,
            Scale::TAI,
            Scale::TAI,
        ))
    }

    /// Builds a linear [`Drift`] from a [`Spacetime`] snapshot.
    ///
    /// The linear coefficient is the general-relativity tick-rate offset
    /// [`Spacetime::proper_time_rate_offset`] (\(d\tau/dt-1\)). The constant
    /// and quadratic terms are zero. Build the [`Spacetime`] from whichever
    /// potential, velocity, or metric lapse you have, then call this.
    #[inline]
    pub const fn from_spacetime(spacetime: &Spacetime) -> Drift {
        Self::from_offset_and_rate(
            Dt::ZERO,
            Dt::from_sec_f(spacetime.proper_time_rate_offset(), Scale::TAI, Scale::TAI),
        )
    }
}

impl Dt {
    /// Builds a clock-drift model in which this [`Dt`] is treated as the
    /// initial fixed time difference between the observer’s proper time and
    /// the chosen coordinate time.
    ///
    /// In practice you often compute or measure a one-time offset (for example
    /// after a clock synchronization) and then want to combine it with a
    /// steady rate difference and any quadratic change.
    /// This method lets you do that directly from a [`Dt`] without having to
    /// call the more verbose [`Drift::new`].
    ///
    /// The other two arguments describe how the difference between the two
    /// clocks will evolve:
    /// - `rate` — the constant fractional rate difference (how much faster or
    ///   slower one clock ticks compared with the other).
    /// - `accel` — how quickly that rate difference itself is changing (for
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
    /// Physically, the rate term captures the fact that two clocks with
    /// different spatial velocities or different gravitational potentials
    /// will accumulate a steadily growing time difference. The other two
    /// parameters let you also describe any starting bias and any change in
    /// that rate over time.
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
    /// whose frequency is aging, or a path that takes it through regions
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

    /// Advances this instant by the proper time that elapses during the
    /// coordinate interval `elapsed` at `spacetime`.
    ///
    /// Adds \(\Delta\tau = \Delta t + (r-1)\Delta t\), where \(r\) is
    /// [`Spacetime::proper_time_rate`]. For a clock that already ticks proper
    /// time, use the plain `add` methods instead.
    #[inline]
    pub const fn adjusted_advance(&mut self, elapsed: &Dt, spacetime: &Spacetime) {
        let dtau = elapsed.add(Drift::from_spacetime(spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this instant by `elapsed` plus the [`Drift`] polynomial
    /// evaluated at `elapsed`.
    ///
    /// When `drift` came from [`Drift::from_spacetime`], this matches
    /// [`adjusted_advance`](Self::adjusted_advance). A polynomial with a
    /// constant or quadratic term is applied in full. The constant term is
    /// added on every call, so a stepping loop should use a polynomial whose
    /// constant is zero, or [`adjusted_advance`](Self::adjusted_advance).
    #[inline]
    pub const fn adjusted_advance_using_drift(&mut self, elapsed: &Dt, drift: &Drift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Adds the [`Drift`] polynomial evaluated at `(self − reference)` to this
    /// instant.
    ///
    /// GNSS broadcast clock corrections and other quadratic steering use this.
    pub const fn convert_using_drift(self, reference: Dt, drift: &Drift) -> Dt {
        let span = self.to_diff_raw(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Inverse of [`convert_using_drift`](Self::convert_using_drift).
    ///
    /// Recovers the instant that would produce `self` after adding the
    /// polynomial relative to `reference`. A fixed-point iteration (at most 16
    /// steps) solves the implicit equation. If the polynomial is a pure
    /// constant offset, the result is returned immediately.
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

/// Fixed-point product `(a * b) / ATTOS_PER_SEC`, saturating on true result overflow.
///
/// Drift coefficients and spans are both attosecond-scaled, so applying rate or
/// accel needs `(a·b)/10¹⁸`. The raw product `a·b` can exceed `i128` even when
/// that scaled result still fits; this helper avoids wrapping or early clamp.
///
/// 1. Uses `checked_mul` when the intermediate product fits (common path).
/// 2. Otherwise splits `a = a_hi·D + a_lo` so
///    `(a·b)/D = a_hi·b + (a_lo·b)/D`, with a second split on `b` if needed.
/// 3. Combines parts with saturating arithmetic so extreme inputs clamp like
///    the rest of [`Dt`] rather than wrapping.
const fn saturating_mul_div_attos_per_sec(a: i128, b: i128) -> i128 {
    if a == 0 || b == 0 {
        return 0;
    }

    if let Some(product) = a.checked_mul(b) {
        return product / ATTOS_PER_SEC_I128;
    }

    // a = a_hi * D + a_lo  (Rust truncating division; identity holds for negatives)
    let a_hi = a / ATTOS_PER_SEC_I128;
    let a_lo = a % ATTOS_PER_SEC_I128;
    // (a_hi * D + a_lo) * b / D = a_hi * b + (a_lo * b) / D
    let hi = a_hi.saturating_mul(b);

    let lo = match a_lo.checked_mul(b) {
        Some(product) => product / ATTOS_PER_SEC_I128,
        None => {
            // |a_lo| < D; split b the same way:
            // a_lo * b / D = a_lo * b_hi + (a_lo * b_lo) / D
            // |a_lo * b_lo| < D² = 10³⁶ < i128::MAX, so the cross term is exact.
            let b_hi = b / ATTOS_PER_SEC_I128;
            let b_lo = b % ATTOS_PER_SEC_I128;
            let cross = (a_lo * b_lo) / ATTOS_PER_SEC_I128;
            a_lo.saturating_mul(b_hi).saturating_add(cross)
        }
    };

    hi.saturating_add(lo)
}

#[cfg(feature = "wire")]
impl Drift {
    /// Wire format version for this type’s outer envelope.
    ///
    /// Independent of nested
    /// [`Dt::WIRE_VERSION`](../struct.Dt.html#associatedconstant.WIRE_VERSION).
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    ///
    /// One version byte plus three
    /// [`Dt::WIRE_SIZE`](../struct.Dt.html#associatedconstant.WIRE_SIZE)
    /// records (`constant`, `rate`, `accel`).
    pub const WIRE_SIZE: usize = 1 + 3 * Dt::WIRE_SIZE;

    /// Serializes this polynomial into a fixed buffer.
    ///
    /// ## Wire format
    ///
    /// - Byte `0`: [`WIRE_VERSION`](Self::WIRE_VERSION)
    /// - Next [`Dt::WIRE_SIZE`](../struct.Dt.html#associatedconstant.WIRE_SIZE)
    ///   bytes: `constant`
    /// - Next [`Dt::WIRE_SIZE`](../struct.Dt.html#associatedconstant.WIRE_SIZE)
    ///   bytes: `rate`
    /// - Next [`Dt::WIRE_SIZE`](../struct.Dt.html#associatedconstant.WIRE_SIZE)
    ///   bytes: `accel`
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let n = Dt::WIRE_SIZE;
        let c = self.constant.to_wire_bytes();
        let r = self.rate.to_wire_bytes();
        let a = self.accel.to_wire_bytes();

        buf[1..1 + n].copy_from_slice(&c);
        buf[1 + n..1 + 2 * n].copy_from_slice(&r);
        buf[1 + 2 * n..1 + 3 * n].copy_from_slice(&a);
        buf
    }

    /// Deserializes from exactly [`WIRE_SIZE`](Self::WIRE_SIZE) bytes.
    ///
    /// ## Errors
    ///
    /// Returns `None` only when:
    /// - `bytes` is not exactly [`WIRE_SIZE`](Self::WIRE_SIZE) long,
    /// - the version byte is not [`WIRE_VERSION`](Self::WIRE_VERSION), or
    /// - any nested
    ///   [`Dt::from_wire_bytes`](../struct.Dt.html#method.from_wire_bytes)
    ///   fails (unknown
    ///   [`Dt::WIRE_VERSION`](../struct.Dt.html#associatedconstant.WIRE_VERSION)).
    ///
    /// Nested scale/target bytes never fail decode (see
    /// [`Dt::from_wire_bytes`](../struct.Dt.html#method.from_wire_bytes)).
    ///
    /// ## Security
    ///
    /// Composes the safety guarantees of
    /// [`Dt::from_wire_bytes`](../struct.Dt.html#method.from_wire_bytes).
    /// Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let n = Dt::WIRE_SIZE;
        let constant = Dt::from_wire_bytes(&bytes[1..1 + n])?;
        let rate = Dt::from_wire_bytes(&bytes[1 + n..1 + 2 * n])?;
        let accel = Dt::from_wire_bytes(&bytes[1 + 2 * n..1 + 3 * n])?;

        Some(Self::new(constant, rate, accel))
    }
}
