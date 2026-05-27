use crate::{C_SQUARED, Drift, Dt, DtErr, DtErrKind, Real, Spacetime, Velocity, an_err};

impl Dt {
    /// Computes the relativistic clock drift (proper time minus coordinate time)
    /// over an interval.
    ///
    /// This returns how much a physical clock has gained or lost time compared
    /// with coordinate time between `start` and `end`.
    ///
    /// - A positive result means the onboard clock ran fast.
    /// - A negative result means the onboard clock ran slow.
    ///
    /// ## Parameters
    ///
    /// - `start`: Starting coordinate time of the interval.
    /// - `end`: Ending coordinate time of the interval.
    /// - `states`: Iterator of physical states. Coordinate times must be
    ///   monotonically non-decreasing. **It is the caller’s responsibility**
    ///   to ensure the provided states cover the time range from `start` to `end`.
    ///   The function integrates proper time over whatever states are supplied
    ///   and subtracts the requested coordinate interval (`end - start`).
    ///   Exact matching of the first and last state times to `start` and `end`
    ///   is **not** validated.
    /// - `characteristic_length_scale`: See [`proper_time_from_states`].
    ///
    /// ## Returns
    ///
    /// `Ok(drift)` — the accumulated drift (Δτ − Δt) as a [`Dt`].
    ///
    /// `Err(DtErr)` — if the states are not monotonically increasing in time.
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
    /// over an interval.
    ///
    /// This returns how much a physical clock has gained or lost time compared
    /// with coordinate time between `start` and `end`.
    ///
    /// - A positive result means the onboard clock ran fast.
    /// - A negative result means the onboard clock ran slow.
    ///
    /// ## Parameters
    ///
    /// - `start`: Starting coordinate time.
    /// - `end`: Ending coordinate time.
    /// - `states`: Iterator of physical states covering the interval.
    ///   Coordinate times must be monotonically non-decreasing.
    ///   It is the caller’s responsibility to ensure the states span
    ///   the requested interval (exact first/last time matching is not checked).
    /// - `characteristic_length_scale`: See [`proper_time_from_states`].
    ///
    /// ## Returns
    ///
    /// `Ok(drift)` — the accumulated drift (Δτ − Δt) as a [`Dt`].
    ///
    /// `Err(DtErr)` — if the states are not monotonically increasing in time.
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
    /// non-decreasing. It performs a single pass with no heap allocation.
    ///
    /// ## Parameters
    ///
    /// - `path`: An iterator of `(coordinate_time, Spacetime)` pairs.
    ///   Coordinate times must be monotonically non-decreasing.
    ///
    /// ## Returns
    ///
    /// `Ok(total_proper_time)` — the accumulated proper time as a [`Dt`].
    ///
    /// `Err(DtErr)` — if the path is empty or contains any decrease in
    /// coordinate time.
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
                return Err(an_err!(
                    DtErrKind::InvalidInput,
                    "proper_time_from_path requires monotonically non-decreasing coordinate times"
                ));
            }

            let dt = t.to_diff_raw(prev_t);
            if !dt.is_zero() {
                let sign = if dt.to_attos() < 0 { f!(-1.0) } else { f!(1.0) };
                let dt_pos = if sign < f!(0.0) { dt.neg() } else { dt };
                let dt_sec = dt_pos.to_sec_f();

                let rate0 = Self::rate_from_local(&prev_ls);
                let rate1 = Self::rate_from_local(&ls);

                let integral = f!(0.5) * (rate0 + rate1 - f!(2.0)) * dt_sec;
                let dtau_segment = Dt::from_sec_f(sign * (dt_sec + integral));

                accumulated = accumulated.add(dtau_segment);
            }

            prev_t = t;
            prev_ls = ls;
        }

        Ok(accumulated)
    }

    /// Computes proper time advance over an interval when the rate is constant.
    ///
    /// Use this for segments where conditions do not change, such as
    /// a ground station, a circular orbit, or a deep-space cruise phase
    /// with constant velocity and gravitational potential.
    ///
    /// This is mathematically equivalent to integrating a constant rate
    /// but is more efficient and expresses intent clearly.
    ///
    /// ## Parameters
    ///
    /// - `end`: Ending coordinate time.
    /// - `dtau_dt`: Constant proper-time rate (dimensionless, usually between 0 and 1).
    ///
    /// ## Returns
    ///
    /// The accumulated proper time advance as a [`Dt`].
    #[inline]
    pub const fn proper_time_between_constant_rate(self, end: Dt, dtau_dt: Real) -> Dt {
        let dt_sec = end.to_diff_raw(self).to_sec_f();
        Dt::from_sec_f(dtau_dt * dt_sec)
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
