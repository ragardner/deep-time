use crate::{
    C_SQUARED, Drift, Dt, DtErr, DtErrKind, Real, Spacetime, Velocity, an_err, from_sec_f,
};

impl Dt {
    /// Computes the accumulated proper time along a trajectory given a sequence
    /// of physical states.
    ///
    /// Accepts samples as `(coordinate_time, velocity, gravitational_potential)`
    /// and integrates proper time (Δτ) along the path. This is a convenience
    /// wrapper around [`Self::proper_time_from_path`].
    ///
    /// Integration uses the trapezoidal rule on the instantaneous proper-time
    /// rate between consecutive samples.
    ///
    /// A single sample, or multiple samples at identical times, produces a result
    /// of zero (no time has elapsed). An empty iterator also returns zero.
    ///
    /// For a named coordinate interval `[start, end]`, use
    /// [`Self::proper_time_from_states_between`] instead.
    ///
    /// ## Parameters
    ///
    /// - `samples`: Iterator yielding `(coordinate_time, velocity, gravitational_potential)`
    ///   triples. Potential is in SI units (m²/s²). Coordinate times must be
    ///   monotonically non-decreasing. This function does not check that the
    ///   first or last sample matches any particular start or end time.
    /// - `characteristic_length_scale`: Selects the weak-field vs strong-field
    ///   construction of the local spacetime state.
    ///
    ///   Pass `0.0` for ordinary weak-field work (Earth orbit, solar-system
    ///   navigation, and similar). The Kretschmann scalar is then set to zero,
    ///   and the rate reduces to the first-order weak-field form built from the
    ///   supplied potential and velocity. Accuracy depends on how complete that
    ///   potential is (multipoles, external bodies, etc. are the caller’s job).
    ///
    ///   Pass a positive length in meters only when estimating curvature for the
    ///   library’s optional strong-field term. Use a length scale over which the
    ///   field varies at the observer; this is relevant near compact objects, not
    ///   for typical solar-system work.
    ///
    /// ## Returns
    ///
    /// `Ok(total_proper_time)` — accumulated proper time (Δτ) for an observer
    /// following the supplied samples, as a [`Dt`].
    ///
    /// This is proper time along the path from the model’s rate at each sample
    /// (velocity and gravitational dilation from those inputs, plus the optional
    /// curvature term when active). It is **not** a drift relative to coordinate
    /// time. For Δτ − Δt over a named interval, use
    /// [`Self::proper_time_drift_from_states`].
    ///
    /// `Err(DtErr)` — if the coordinate times are not monotonically non-decreasing.
    pub fn proper_time_from_states<I>(
        samples: I,
        characteristic_length_scale: Real,
    ) -> Result<Self, DtErr>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        Self::proper_time_from_path(Self::states_to_path(samples, characteristic_length_scale))
    }

    /// Accumulated proper time (Δτ) over the coordinate interval `[start, end]`.
    ///
    /// Same physics as [`Self::proper_time_from_states`], but only the portion of
    /// the path that falls in `[start, end]` is integrated. Samples outside that
    /// window are ignored except as bracketing points for linear rate
    /// interpolation at the endpoints.
    ///
    /// ## Parameters
    ///
    /// - `start` / `end`: Interval bounds (`start` ≤ `end`).
    /// - `states`: `(coordinate_time, velocity, gravitational_potential)` samples
    ///   (potential in m²/s²). Must cover `[start, end]` (see errors below).
    /// - `characteristic_length_scale`: See [`Self::proper_time_from_states`].
    ///
    /// ## Returns
    ///
    /// `Ok(Δτ)` over `[start, end]`.
    ///
    /// `Ok(ZERO)` if `start == end`.
    ///
    /// `Err(DtErr)` — [`DtErrKind::OutOfRange`] if `end < start`;
    /// [`DtErrKind::NonMonotonic`] if times decrease;
    /// [`DtErrKind::Incomplete`] if samples do not cover the interval.
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

    /// Computes the relativistic clock drift (proper time minus coordinate time)
    /// over a specific interval `[start, end]`.
    ///
    /// This is \(\int_{start}^{end}(r-1)\,dt = \Delta\tau - (end - start)\), where
    /// \(r = d\tau/dt\).
    ///
    /// - A positive result means the onboard clock ran **fast** (it accumulated
    ///   more proper time than the coordinate interval).
    /// - A negative result means the onboard clock ran **slow** (it accumulated
    ///   less proper time than the coordinate interval).
    ///
    /// Implemented as
    /// [`Self::proper_time_from_states_between`]`(start, end, …)` minus
    /// `(end − start)`.
    ///
    /// ## Parameters
    ///
    /// - `start`: Starting coordinate time of the interval (must be ≤ `end`).
    /// - `end`: Ending coordinate time of the interval.
    /// - `states`: Iterator of physical states in the form
    ///   `(coordinate_time, velocity, gravitational_potential)`.
    ///   Coordinate times must be monotonically **non-decreasing**.
    ///   The samples must **cover** `[start, end]`: at least one sample at or
    ///   before `start`, and the path must reach at least as far as `end`
    ///   (a sample at or after `end`, possibly after interpolating across a
    ///   bracketing segment).
    /// - `characteristic_length_scale`: Weak-field vs strong-field construction
    ///   of local spacetime states (see [`Self::proper_time_from_states`]).
    ///   Pass `0.0` for ordinary weak-field work; a positive length (meters) only
    ///   when the optional curvature term is needed.
    ///
    /// ## Returns
    ///
    /// `Ok(drift)` — the accumulated drift (`Δτ − Δt`) over `[start, end]` as a [`Dt`].
    ///
    /// `Ok(ZERO)` — if `start == end` (no elapsed coordinate time), regardless
    /// of `states`.
    ///
    /// `Err(DtErr)` — on any of:
    /// - [`DtErrKind::OutOfRange`] if `end < start`
    /// - [`DtErrKind::NonMonotonic`] if coordinate times decrease
    /// - [`DtErrKind::Incomplete`] if `states` is empty (when `start != end`) or
    ///   does not cover `[start, end]`
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

    /// Computes accumulated proper time along an arbitrary trajectory.
    ///
    /// Core path integrator: walks the supplied samples segment by segment and
    /// applies the trapezoidal rule to the instantaneous proper-time rate.
    ///
    /// Coordinate times must be monotonically non-decreasing (equal times are
    /// allowed). The walk is a single pass with no heap allocation.
    ///
    /// For a named coordinate interval, use [`Self::proper_time_from_path_between`].
    ///
    /// ## Parameters
    ///
    /// - `path`: An iterator of `(coordinate_time, Spacetime)` pairs.
    ///   Coordinate times must be monotonically non-decreasing.
    ///
    /// ## Returns
    ///
    /// `Ok(total_proper_time)` — the accumulated proper time (Δτ) as a [`Dt`].
    ///   Returns `ZERO` if the iterator is empty (no time elapsed).
    ///
    /// `Err(DtErr)` — if the path contains any decrease in coordinate time
    ///   (i.e., a later sample has a strictly earlier coordinate time than a
    ///   previous sample).
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

    /// Accumulated proper time (Δτ) over the coordinate interval `[start, end]`.
    ///
    /// Same trapezoidal model as [`Self::proper_time_from_path`], restricted to
    /// `[start, end]`. The proper-time rate is treated as piecewise linear
    /// between samples; when `start` or `end` falls between samples, the rate at
    /// that boundary is linearly interpolated.
    ///
    /// ## Parameters
    ///
    /// - `start` / `end`: Interval bounds (`start` ≤ `end`).
    /// - `path`: `(coordinate_time, Spacetime)` samples covering `[start, end]`.
    ///
    /// ## Returns
    ///
    /// `Ok(Δτ)` over `[start, end]`.
    ///
    /// `Ok(ZERO)` if `start == end`.
    ///
    /// `Err(DtErr)` — [`DtErrKind::OutOfRange`] if `end < start`;
    /// [`DtErrKind::NonMonotonic`] if times decrease;
    /// [`DtErrKind::Incomplete`] if the path does not cover the interval.
    pub fn proper_time_from_path_between<I>(start: Dt, end: Dt, path: I) -> Result<Dt, DtErr>
    where
        I: IntoIterator<Item = (Self, Spacetime)>,
    {
        let rates = path
            .into_iter()
            .map(|(t, ls)| (t, Self::rate_from_local(&ls)));
        Self::integrate_rates_between(start, end, rates)
    }

    /// Differential proper time between two paths over `[start, end]`.
    ///
    /// Returns \(\Delta\tau_A - \Delta\tau_B\): how much more (or less) proper
    /// time path A accumulates than path B over the same coordinate interval.
    /// Positive means A’s clock ran ahead of B’s.
    ///
    /// Both paths must cover `[start, end]` (same coverage rules as
    /// [`Self::proper_time_from_path_between`]).
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

    /// Differential proper time of a path relative to a constant reference rate.
    ///
    /// Returns \(\Delta\tau_{\mathrm{path}} - r_{\mathrm{ref}}\,(end - start)\)
    /// over `[start, end]`. Typical use: spacecraft samples versus a fixed ground
    /// or geoid rate from [`Spacetime::proper_time_rate`] or a precomputed constant.
    ///
    /// Positive means the path clock accumulated more proper time than the
    /// reference over the interval.
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

    /// Computes proper time advance over an interval when the proper-time rate
    /// is constant.
    ///
    /// This method is intended for trajectory segments where the physical
    /// conditions remain unchanged, such as:
    ///
    /// - a fixed ground station,
    /// - a circular orbit, or
    /// - a deep-space cruise phase with constant velocity and gravitational potential.
    ///
    /// It is mathematically equivalent to integrating a constant rate using
    /// the trapezoidal rule in [`Self::proper_time_from_path`], but is more efficient
    /// and makes the caller's intent explicit.
    ///
    /// The method is called on the starting coordinate time (`self`). It
    /// calculates the coordinate time interval to `end` and multiplies it by
    /// the supplied constant rate `dtau_dt`.
    ///
    /// ## Parameters
    ///
    /// - `end`: Ending coordinate time of the interval.
    /// - `dtau_dt`: Constant proper-time rate (dimensionless). In relativistic
    ///   contexts this value is typically slightly less than `1.0`. The caller
    ///   is responsible for providing an appropriate rate (for example, from
    ///   `Drift::proper_time_rate` or a precomputed constant).
    ///
    /// ## Returns
    ///
    /// The accumulated proper time advance (Δτ) over the interval as a [`Dt`].
    ///
    /// If `end` occurs before `self`, the result will be negative.
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
                    accumulated = accumulated.add(Self::proper_time_segment(
                        start, rate_start, t, rate,
                    ));
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
    ///
    /// This is a private helper used by the integration routines.
    #[inline]
    const fn rate_from_local(spacetime: &Spacetime) -> Real {
        let drift = Drift::from_spacetime(spacetime);
        f!(1.0) + drift.rate.to_sec_f()
    }
}
