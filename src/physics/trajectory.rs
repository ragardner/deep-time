//! Proper-time integration methods on [`Dt`] (see the public method docs).
//!
//! Overview and which-function guide:
//! [docs/trajectory.md](https://github.com/ragardner/deep-time/blob/main/docs/trajectory.md).

use crate::{
    C_SQUARED, Drift, Dt, DtErr, DtErrKind, Real, Spacetime, Velocity, an_err, from_sec_f,
};

impl Dt {
    /// Integrate proper time along samples of time, velocity, and gravitational potential.
    ///
    /// Walks a list of vehicle states and estimates how much time a clock on that
    /// path would accumulate over the **full** sample span (first time to last).
    /// For a named arc inside a longer file, use
    /// [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between).
    ///
    /// Guide: [docs/trajectory.md](https://github.com/ragardner/deep-time/blob/main/docs/trajectory.md).
    ///
    /// ## When to use it
    ///
    /// - Δτ over **exactly the samples you pass** (first sample to last).
    /// - Not a sub-interval of a longer arc (use
    ///   [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between)).
    ///
    /// ## Inputs
    ///
    /// Each sample is `(coordinate_time, velocity, gravitational_potential)`:
    ///
    /// - **time** — mission / ephemeris epoch as a [`Dt`]
    /// - **velocity** — m/s in the same frame convention you used for potential
    /// - **potential Φ** — SI units **m²/s²** (typically negative near a planet).
    ///   Do **not** pass Φ/c² here; this API divides by \(c^2\) internally.
    ///
    /// Times must be non-decreasing. Empty or single-point paths yield zero.
    /// Non-monotonic times yield [`DtErrKind::NonMonotonic`].
    ///
    /// ## `characteristic_length_scale`
    ///
    /// Pass **`0.0`** for Earth orbit, GNSS, cislunar, and similar work. That sets
    /// curvature to zero and uses the usual weak-field clock rate from Φ and \(v\).
    ///
    /// Pass a positive length in meters only if you intentionally want the
    /// library’s optional curvature estimate (see
    /// [`Spacetime::kretschmann_from_potential_and_scale`]).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Velocity};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(3600, Scale::TAI, Scale::TAI);
    /// // Example Earth-surface-scale |Φ| (m²/s²); use your model in production
    /// let phi = -6.25e7;
    /// let samples = [
    ///     (t0, Velocity::ZERO, phi),
    ///     (t1, Velocity::from_speed(0.0), phi),
    /// ];
    /// let dtau = Dt::proper_time_from_states(samples, 0.0).expect("monotonic");
    /// assert!(dtau.to_sec_f() > 0.0 && dtau.to_sec_f() < 3600.0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between) — named interval `[start, end]`
    /// - [`Dt::proper_time_drift_from_states`](#method.proper_time_drift_from_states) — gain/loss vs coordinate time
    /// - [`Dt::proper_time_from_path`](#method.proper_time_from_path) — same integral if you already have [`Spacetime`]
    pub fn proper_time_from_states<I>(
        samples: I,
        characteristic_length_scale: Real,
    ) -> Result<Self, DtErr>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        Self::proper_time_from_path(Self::states_to_path(samples, characteristic_length_scale))
    }

    /// Proper time Δτ on a named mission arc `[start, end]`.
    ///
    /// Same idea as [`Dt::proper_time_from_states`](#method.proper_time_from_states), but only the window
    /// `[start, end]` is integrated. Extra samples outside that window are
    /// ignored except as neighbors for interpolation at the endpoints.
    ///
    /// Example question: how much proper time has the onboard clock accumulated
    /// between two GET epochs when the trajectory file is longer than that arc.
    ///
    /// ## Coverage and errors
    ///
    /// Samples must **cover** `[start, end]`:
    /// - at least one sample at or before `start`, and
    /// - the path must reach at least as far as `end`.
    ///
    /// - [`DtErrKind::Incomplete`] — empty path (when `start ≠ end`) or incomplete coverage
    /// - [`DtErrKind::OutOfRange`] — `end < start`
    /// - [`DtErrKind::NonMonotonic`] — a later sample has an earlier time
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Velocity};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(10_000, Scale::TAI, Scale::TAI);
    /// // Flat spacetime via Φ = 0 → rate = 1
    /// let samples = [
    ///     (t0, Velocity::ZERO, 0.0),
    ///     (t1, Velocity::ZERO, 0.0),
    /// ];
    /// let start = Dt::from_sec(1000, Scale::TAI, Scale::TAI);
    /// let end = Dt::from_sec(4600, Scale::TAI, Scale::TAI);
    /// let dtau = Dt::proper_time_from_states_between(start, end, samples, 0.0)
    ///     .expect("samples cover the arc");
    /// assert_eq!(dtau, Dt::from_sec(3600, Scale::TAI, Scale::TAI));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::proper_time_drift_from_states`](#method.proper_time_drift_from_states) — same window, but Δτ − Δt
    /// - [`Dt::proper_time_from_path_between`](#method.proper_time_from_path_between) — if samples are already [`Spacetime`]
    pub fn proper_time_from_states_between<I>(
        start: Dt,
        end: Dt,
        states: I,
        characteristic_length_scale: Real,
    ) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        Self::proper_time_from_path_between(
            start,
            end,
            Self::states_to_path(states, characteristic_length_scale),
        )
    }

    /// Clock drift vs coordinate time on `[start, end]`: Δτ − (end − start).
    ///
    /// Did the vehicle clock run fast or slow compared to the mission timeline
    /// over a chosen interval?
    ///
    /// - **Positive** — clock accumulated more time than the coordinate interval
    ///   (ran fast).
    /// - **Negative** — clock accumulated less (ran slow).
    ///
    /// Algebraically \(\int_{start}^{end}(r - 1)\,dt\). Implemented as
    /// [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between) minus `(end − start)`.
    ///
    /// ## When to use it
    ///
    /// - Relativistic clock offset over an analysis arc
    /// - Comparing an integrated model to a coordinate-time reference
    /// - Not spacecraft-minus-ground (use
    ///   [`Dt::proper_time_differential_vs_rate`](#method.proper_time_differential_vs_rate) or
    ///   [`Dt::proper_time_differential_from_paths`](#method.proper_time_differential_from_paths))
    ///
    /// ## Inputs and errors
    ///
    /// Same sample layout as [`Dt::proper_time_from_states`](#method.proper_time_from_states):
    /// `(time, velocity m/s, Φ m²/s²)`. Pass `characteristic_length_scale = 0.0`
    /// for ordinary weak-field work. Coverage and error kinds match
    /// [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between). `start == end` returns zero
    /// without reading samples.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Velocity};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(86_400, Scale::TAI, Scale::TAI);
    /// let phi = -6.25e7_f64;
    /// let samples = [
    ///     (t0, Velocity::ZERO, phi),
    ///     (t1, Velocity::ZERO, phi),
    /// ];
    /// let drift = Dt::proper_time_drift_from_states(t0, t1, samples, 0.0).unwrap();
    /// // Stationary in a potential well → clock runs slow vs coordinate time
    /// assert!(drift.to_sec_f() < 0.0);
    /// ```
    pub fn proper_time_drift_from_states<I>(
        start: Dt,
        end: Dt,
        states: I,
        characteristic_length_scale: Real,
    ) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        if start.eq(&end) {
            return Ok(Dt::ZERO);
        }
        let dtau =
            Self::proper_time_from_states_between(start, end, states, characteristic_length_scale)?;
        Ok(dtau.sub(end.to_diff_raw(start)))
    }

    /// Integrate proper time along a path of [`Spacetime`] snapshots.
    ///
    /// Same as [`Dt::proper_time_from_states`](#method.proper_time_from_states), but each sample is already a
    /// full local state `(α, β, curvature)` instead of `(v, Φ)`.
    ///
    /// ## When to use it
    ///
    /// - You already built [`Spacetime`] values (tests, precomputed rates, custom α/β).
    /// - Prefer [`Dt::proper_time_from_states`](#method.proper_time_from_states) if you have velocity and potential.
    ///
    /// Integrates over the **full** sample span. For a named arc, use
    /// [`Dt::proper_time_from_path_between`](#method.proper_time_from_path_between).
    ///
    /// Empty path or a single point → [`Dt::ZERO`]. Non-monotonic times →
    /// [`DtErrKind::NonMonotonic`].
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Spacetime};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(1000, Scale::TAI, Scale::TAI);
    /// // α = 0.9, at rest → rate 0.9, Δτ = 900 s
    /// let slow = Spacetime::new(0.9, 0.0, 0.0);
    /// let dtau = Dt::proper_time_from_path([(t0, slow.clone()), (t1, slow)]).unwrap();
    /// assert_eq!(dtau, Dt::from_sec(900, Scale::TAI, Scale::TAI));
    /// ```
    pub fn proper_time_from_path<I>(path: I) -> Result<Self, DtErr>
    where
        I: IntoIterator<Item = (Self, Spacetime)>,
    {
        let mut iter = path.into_iter();

        let Some((mut prev_t, mut prev_ls)) = iter.next() else {
            return Ok(Self::ZERO);
        };

        let mut accumulated = Self::ZERO;

        for (t, ls) in iter {
            if t.lt(&prev_t) {
                return Err(an_err!(DtErrKind::NonMonotonic));
            }

            let rate0 = Self::rate_from_local(&prev_ls);
            let rate1 = Self::rate_from_local(&ls);
            accumulated = accumulated.add(Self::proper_time_segment(prev_t, rate0, t, rate1));

            prev_t = t;
            prev_ls = ls;
        }

        Ok(accumulated)
    }

    /// Proper time Δτ on `[start, end]` for a path of [`Spacetime`] samples.
    ///
    /// Like [`Dt::proper_time_from_path`](#method.proper_time_from_path), but only over a chosen time window.
    /// Between samples the clock rate is treated as linear (trapezoidal rule);
    /// if `start` or `end` falls between samples, the rate is interpolated.
    ///
    /// Use this when your pipeline already stores α, β, and curvature instead of
    /// raw Φ and \(v\). Coverage and error kinds match
    /// [`Dt::proper_time_from_states_between`](#method.proper_time_from_states_between).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Spacetime};
    ///
    /// let path = [
    ///     (Dt::from_sec(0, Scale::TAI, Scale::TAI), Spacetime::new(0.9, 0.0, 0.0)),
    ///     (Dt::from_sec(1000, Scale::TAI, Scale::TAI), Spacetime::new(0.9, 0.0, 0.0)),
    /// ];
    /// let start = Dt::from_sec(100, Scale::TAI, Scale::TAI);
    /// let end = Dt::from_sec(900, Scale::TAI, Scale::TAI);
    /// // 0.9 × 800 s = 720 s
    /// let dtau = Dt::proper_time_from_path_between(start, end, path).unwrap();
    /// assert_eq!(dtau, Dt::from_sec(720, Scale::TAI, Scale::TAI));
    /// ```
    pub fn proper_time_from_path_between<I>(start: Dt, end: Dt, path: I) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Spacetime)>,
    {
        let rates = path
            .into_iter()
            .map(|(t, ls)| (t, Self::rate_from_local(&ls)));
        Self::integrate_rates_between(start, end, rates)
    }

    /// Difference in proper time between two paths over the same interval.
    ///
    /// How much more (or less) time did clock A accumulate than clock B over
    /// `[start, end]`?
    ///
    /// Returns \(\Delta\tau_A - \Delta\tau_B\). Positive means A’s clock ran
    /// ahead of B’s over that coordinate interval.
    ///
    /// ## When to use it
    ///
    /// - Two vehicles or two reconstructed trajectories
    /// - Spacecraft path vs a **sampled** ground path (both as [`Spacetime`] series)
    ///
    /// For spacecraft vs a **fixed** ground rate (single number), prefer
    /// [`Dt::proper_time_differential_vs_rate`](#method.proper_time_differential_vs_rate).
    ///
    /// ## Errors
    ///
    /// Both paths must cover `[start, end]`. Same error kinds as
    /// [`Dt::proper_time_from_path_between`](#method.proper_time_from_path_between) (`Incomplete`, `OutOfRange`,
    /// `NonMonotonic`). `start == end` returns zero.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Spacetime};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(1000, Scale::TAI, Scale::TAI);
    /// let high = Spacetime::new(0.95, 0.0, 0.0); // less redshifted
    /// let low = Spacetime::new(0.90, 0.0, 0.0);
    /// let path_a = [(t0, high.clone()), (t1, high)];
    /// let path_b = [(t0, low.clone()), (t1, low)];
    /// let diff = Dt::proper_time_differential_from_paths(t0, t1, path_a, path_b).unwrap();
    /// // 950 − 900 = +50 s
    /// assert_eq!(diff, Dt::from_sec(50, Scale::TAI, Scale::TAI));
    /// ```
    pub fn proper_time_differential_from_paths<Ia, Ib>(
        start: Dt,
        end: Dt,
        path_a: Ia,
        path_b: Ib,
    ) -> Result<Dt, DtErr>
    where
        Ia: IntoIterator<Item = (Self, Spacetime)>,
        Ib: IntoIterator<Item = (Self, Spacetime)>,
    {
        if start.eq(&end) {
            return Ok(Dt::ZERO);
        }
        let dtau_a = Self::proper_time_from_path_between(start, end, path_a)?;
        let dtau_b = Self::proper_time_from_path_between(start, end, path_b)?;
        Ok(dtau_a.sub(dtau_b))
    }

    /// Proper time of a path minus a constant reference clock rate over `[start, end]`.
    ///
    /// How much did the spacecraft clock pull ahead of (or fall behind) a steady
    /// ground or reference clock?
    ///
    /// Returns \(\Delta\tau_{\mathrm{path}} - r_{\mathrm{ref}}\,(end - start)\).
    /// Positive means the path clock accumulated more proper time than the
    /// reference over the interval.
    ///
    /// ## When to use it
    ///
    /// - Onboard vs Earth-surface rate (mission clock differentials)
    /// - Satellite vs a fixed geoid rate
    /// - Any reference well modeled as **constant** \(r_{\mathrm{ref}}\)
    ///
    /// Get \(r_{\mathrm{ref}}\) from [`Spacetime::proper_time_rate`] for a
    /// stationary ground [`Spacetime`], or from a documented conventional value.
    ///
    /// ## Errors
    ///
    /// Path must cover `[start, end]`. Same error kinds as
    /// [`Dt::proper_time_from_path_between`](#method.proper_time_from_path_between). `start == end` returns zero.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Spacetime};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(100_000, Scale::TAI, Scale::TAI);
    /// // Slightly higher rate than a deeper potential well
    /// let sc = Spacetime::new(0.999_999_999_9, 0.0, 0.0);
    /// let ground = Spacetime::new(0.999_999_999_3, 0.0, 0.0);
    /// let path = [(t0, sc.clone()), (t1, sc)];
    /// let diff = Dt::proper_time_differential_vs_rate(
    ///     t0,
    ///     t1,
    ///     path,
    ///     ground.proper_time_rate(),
    /// )
    /// .unwrap();
    /// assert!(diff.to_sec_f() > 0.0);
    /// ```
    pub fn proper_time_differential_vs_rate<I>(
        start: Dt,
        end: Dt,
        path: I,
        ref_rate: Real,
    ) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Spacetime)>,
    {
        if start.eq(&end) {
            return Ok(Dt::ZERO);
        }
        let dtau = Self::proper_time_from_path_between(start, end, path)?;
        let ref_dtau = start.proper_time_between_constant_rate(end, ref_rate);
        Ok(dtau.sub(ref_dtau))
    }

    /// Proper time when the rate \(d\tau/dt\) is constant over an interval.
    ///
    /// If conditions do not change (same speed, same gravity), proper time is
    /// just **rate × elapsed coordinate time**. No sample list needed.
    ///
    /// ## When to use it
    ///
    /// - Fixed ground station
    /// - Circular orbit approximated as constant rate
    /// - Deep-space cruise with nearly constant \(v\) and Φ
    /// - Building the reference leg for
    ///   [`Dt::proper_time_differential_vs_rate`](#method.proper_time_differential_vs_rate)
    ///
    /// Called on the **start** time: `start.proper_time_between_constant_rate(end, rate)`.
    /// If `end` is before `self`, the result is negative.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, Spacetime};
    ///
    /// let t0 = Dt::from_sec(0, Scale::TAI, Scale::TAI);
    /// let t1 = Dt::from_sec(86_400, Scale::TAI, Scale::TAI);
    /// let ground = Spacetime::new(0.999_999_999_3, 0.0, 0.0);
    /// let dtau = t0.proper_time_between_constant_rate(t1, ground.proper_time_rate());
    /// assert!(dtau.to_sec_f() > 0.0 && dtau.to_sec_f() < 86_400.0);
    /// ```
    #[inline]
    pub const fn proper_time_between_constant_rate(self, end: Dt, dtau_dt: Real) -> Dt {
        let dt_sec = end.to_diff_raw(self).to_sec_f();
        from_sec_f!(dtau_dt * dt_sec)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Maps `(t, velocity, Φ)` states to `(t, Spacetime)` using the library rate model.
    fn states_to_path<I>(
        samples: I,
        characteristic_length_scale: Real,
    ) -> impl Iterator<Item = (Self, Spacetime)>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        samples.into_iter().map(move |(t, vel, phi)| {
            let phi_over_c2 = phi / C_SQUARED;
            let ls = Spacetime::from_potential_velocity_and_scale(
                phi_over_c2,
                vel,
                characteristic_length_scale,
            );
            (t, ls)
        })
    }

    /// Shared kernel: integrate a piecewise-linear proper-time rate series over
    /// the closed coordinate interval `[start, end]`.
    ///
    /// Returns absolute Δτ (not drift). Coverage and monotonicity rules match
    /// the public `*_between` methods.
    fn integrate_rates_between<I>(start: Dt, end: Dt, rates: I) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Real)>,
    {
        if start.eq(&end) {
            return Ok(Dt::ZERO);
        }
        if end.lt(&start) {
            return Err(an_err!(DtErrKind::OutOfRange));
        }

        let mut iter = rates.into_iter();

        let Some((mut prev_t, mut prev_rate)) = iter.next() else {
            return Err(an_err!(DtErrKind::Incomplete));
        };

        // Need a sample at or before `start` to evaluate the rate on the window.
        if prev_t.gt(&start) {
            return Err(an_err!(DtErrKind::Incomplete));
        }

        let mut accumulated = Self::ZERO;
        // Once true, `(prev_t, prev_rate)` is the left endpoint of an open
        // segment still inside the window (`start <= prev_t < end`).
        let mut active = false;

        for (t, rate) in iter {
            if t.lt(&prev_t) {
                return Err(an_err!(DtErrKind::NonMonotonic));
            }

            if !active {
                if t.lt(&start) {
                    // Entirely before the window; slide forward.
                    prev_t = t;
                    prev_rate = rate;
                    continue;
                }

                // prev_t <= start <= t
                let rate_start = if prev_t.eq(&start) {
                    prev_rate
                } else if t.eq(&start) {
                    rate
                } else {
                    Self::lerp_rate(prev_t, prev_rate, t, rate, start)
                };

                if t.lt(&end) {
                    accumulated =
                        accumulated.add(Self::proper_time_segment(start, rate_start, t, rate));
                    active = true;
                    prev_t = t;
                    prev_rate = rate;
                    continue;
                }

                // t >= end: the whole window lies inside this bracketing segment.
                let rate_end = if t.eq(&end) {
                    rate
                } else {
                    Self::lerp_rate(prev_t, prev_rate, t, rate, end)
                };
                accumulated =
                    accumulated.add(Self::proper_time_segment(start, rate_start, end, rate_end));
                return Ok(accumulated);
            }

            // active: integrate from prev toward end
            if t.lt(&end) {
                accumulated =
                    accumulated.add(Self::proper_time_segment(prev_t, prev_rate, t, rate));
                prev_t = t;
                prev_rate = rate;
                continue;
            }

            // t >= end
            let rate_end = if t.eq(&end) {
                rate
            } else {
                Self::lerp_rate(prev_t, prev_rate, t, rate, end)
            };
            accumulated =
                accumulated.add(Self::proper_time_segment(prev_t, prev_rate, end, rate_end));
            return Ok(accumulated);
        }

        // Exhausted samples without reaching `end`.
        Err(an_err!(DtErrKind::Incomplete))
    }

    /// Trapezoidal proper-time advance over one coordinate segment.
    ///
    /// Uses the compensated form
    /// \(\Delta\tau = \Delta t + \tfrac12(r_0 + r_1 - 2)\,\Delta t\)
    /// so that the large \(\approx 1\) part of the rate does not cancel against
    /// \(\Delta t\) in floating point. Supports a negative segment
    /// (`t1 < t0`) for symmetry; callers that enforce monotonic times only see
    /// non-negative \(\Delta t\).
    #[inline]
    const fn proper_time_segment(t0: Dt, rate0: Real, t1: Dt, rate1: Real) -> Dt {
        let dt = t1.to_diff_raw(t0);
        if dt.is_zero() {
            return Self::ZERO;
        }

        let sign = if dt.to_attos() < 0 { f!(-1.0) } else { f!(1.0) };
        let dt_pos = if sign < f!(0.0) { dt.neg() } else { dt };
        let dt_sec = dt_pos.to_sec_f();

        let integral = f!(0.5) * (rate0 + rate1 - f!(2.0)) * dt_sec;
        from_sec_f!(sign * (dt_sec + integral))
    }

    /// Linearly interpolates the proper-time rate at coordinate time `t`,
    /// assuming a piecewise-linear rate between `(t0, rate0)` and `(t1, rate1)`.
    ///
    /// Caller must ensure `t0 < t1` (non-zero span) and typically
    /// `t0 < t < t1`.
    #[inline]
    const fn lerp_rate(t0: Dt, rate0: Real, t1: Dt, rate1: Real, t: Dt) -> Real {
        let span = t1.to_diff_raw(t0).to_sec_f();
        let frac = t.to_diff_raw(t0).to_sec_f() / span;
        rate0 + frac * (rate1 - rate0)
    }

    /// Returns the instantaneous proper-time rate (dτ/dt) from a local
    /// spacetime state.
    #[inline]
    const fn rate_from_local(spacetime: &Spacetime) -> Real {
        let drift = Drift::from_spacetime(spacetime);
        f!(1.0) + drift.rate.to_sec_f()
    }
}
