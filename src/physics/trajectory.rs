use crate::{
    C_SQUARED, Drift, Dt, DtErr, DtErrKind, Real, Spacetime, Velocity, an_err, from_sec_f,
};

impl Dt {
    /// Computes the accumulated proper time along a trajectory given a sequence
    /// of physical states.
    ///
    /// This function accepts samples expressed in terms of directly observable
    /// quantities — coordinate time, velocity, and gravitational potential —
    /// and integrates the proper time (Δτ) along the path. It is a convenience
    /// wrapper around the core [`Self::proper_time_from_path`] routine.
    ///
    /// The integration is performed using the trapezoidal rule applied to the
    /// instantaneous proper-time rate between consecutive samples. This approach
    /// is standard for high-precision clock modeling in astrodynamics and
    /// relativistic timing applications.
    ///
    /// A single sample, or multiple samples at identical times, produces a result
    /// of zero (no time has elapsed). An empty iterator also returns zero.
    ///
    /// ## Parameters
    ///
    /// - `samples`: Iterator yielding `(coordinate_time, velocity, gravitational_potential)`
    ///   triples. The coordinate times must be monotonically non-decreasing.
    ///   **It is the caller’s responsibility** to supply samples that cover the
    ///   desired time interval. The function does not validate that the first or
    ///   last sample exactly matches any particular start or end time.
    /// - `characteristic_length_scale`: Controls whether the weak-field or
    ///   strong-field formulation is used when constructing the local spacetime
    ///   state.
    ///
    ///   Pass `0.0` (the normal choice) for all conventional weak-field work
    ///   (Earth orbit, GNSS, solar-system navigation, most spacecraft). This
    ///   produces exactly the classic relativistic clock rate used by JPL, ESA,
    ///   and GNSS systems, with the Kretschmann scalar set to zero.
    ///
    ///   Supply a positive value (in meters) only when you need the library’s
    ///   intrinsic Planck-scale saturation term. The value should represent the
    ///   characteristic length scale over which the gravitational field varies
    ///   significantly at the observer’s location. This is intended for strong-field
    ///   regimes such as the vicinity of neutron stars or black-hole event horizons.
    ///
    /// ## Returns
    ///
    /// `Ok(total_proper_time)` — the total proper time (Δτ) that has accumulated
    /// for an observer following the trajectory defined by the supplied samples,
    /// returned as a [`Dt`].
    ///
    /// This value represents the actual time that would have elapsed on a physical
    /// clock moving along the path, including all relativistic effects (velocity
    /// and gravitational time dilation, plus the Planck-scale saturation term when
    /// active). It is **not** a drift or difference relative to coordinate time.
    /// If you need the difference between proper time and coordinate time
    /// (Δτ − Δt), use [`Self::proper_time_drift_from_states`] instead.
    ///
    /// `Err(DtErr)` — if the coordinate times are not monotonically non-decreasing.
    pub fn proper_time_from_states<I>(
        samples: I,
        characteristic_length_scale: Real,
    ) -> Result<Self, DtErr>
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        let path_iter = samples.into_iter().map(|(t, vel, phi)| {
            let phi_over_c2 = phi / C_SQUARED;
            let ls = Spacetime::from_potential_velocity_and_scale(
                phi_over_c2,
                vel,
                characteristic_length_scale,
            );
            (t, ls)
        });

        Self::proper_time_from_path(path_iter)
    }

    /// Computes the relativistic clock drift (proper time minus coordinate time)
    /// over a specific interval.
    ///
    /// This returns how much a physical clock has gained or lost time compared
    /// with coordinate time between `start` and `end`.
    ///
    /// - A positive result means the onboard clock ran **fast** (it accumulated
    ///   more proper time than the coordinate interval).
    /// - A negative result means the onboard clock ran **slow** (it accumulated
    ///   less proper time than the coordinate interval).
    ///
    /// This is the higher-level function most callers should use when they need
    /// the net drift over a well-defined time interval. It internally calls
    /// [`Self::proper_time_from_states`] to integrate proper time along the supplied
    /// trajectory and then subtracts the requested coordinate time span.
    ///
    /// ## Parameters
    ///
    /// - `start`: Starting coordinate time of the interval.
    /// - `end`: Ending coordinate time of the interval.
    /// - `states`: Iterator of physical states in the form
    ///   `(coordinate_time, velocity, gravitational_potential)`.
    ///   Coordinate times must be monotonically **non-decreasing**.
    ///   **It is the caller’s responsibility** to ensure the provided states
    ///   cover the time range from `start` to `end`. The function integrates
    ///   proper time over whatever samples are supplied and subtracts the
    ///   requested coordinate interval (`end - start`). Exact matching of the
    ///   first and last state times to `start` and `end` is **not** validated.
    /// - `characteristic_length_scale`: Controls the weak-field vs strong-field
    ///   formulation when constructing local spacetime states (see
    ///   [`Self::proper_time_from_states`] for full details). Pass `0.0` for all normal
    ///   weak-field work (GNSS, Earth orbit, solar-system navigation). Supply a
    ///   positive length (in meters) only when strong-field Planck-scale
    ///   saturation effects are required.
    ///
    /// ## Returns
    ///
    /// `Ok(drift)` — the accumulated drift (`Δτ − Δt`) as a [`Dt`].
    ///
    /// `Err(DtErr)` — if the coordinate times in `states` are not monotonically
    /// non-decreasing.
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
        let dtau = Self::proper_time_from_states(states, characteristic_length_scale)?;
        Ok(dtau.sub(end.to_diff_raw(start)))
    }

    /// Computes accumulated proper time along an arbitrary trajectory.
    ///
    /// This is the core integration function of the library. It walks the
    /// supplied path segment by segment and applies the trapezoidal rule
    /// to the instantaneous proper-time rate at each step.
    ///
    /// This approach is commonly used when integrating clock rates along
    /// sampled trajectories in astrodynamics and high-precision timing work.
    ///
    /// The function enforces that coordinate times are monotonically
    /// non-decreasing (equal times are allowed). It performs a single pass
    /// with no heap allocation.
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

            let dt = t.to_diff_raw(prev_t);
            if !dt.is_zero() {
                let sign = if dt.to_attos() < 0 { f!(-1.0) } else { f!(1.0) };
                let dt_pos = if sign < f!(0.0) { dt.neg() } else { dt };
                let dt_sec = dt_pos.to_sec_f();

                let rate0 = Self::rate_from_local(&prev_ls);
                let rate1 = Self::rate_from_local(&ls);

                let integral = f!(0.5) * (rate0 + rate1 - f!(2.0)) * dt_sec;
                let dtau_segment = from_sec_f!(sign * (dt_sec + integral));

                accumulated = accumulated.add(dtau_segment);
            }

            prev_t = t;
            prev_ls = ls;
        }

        Ok(accumulated)
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
        crate::from_sec_f!(dtau_dt * dt_sec)
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
