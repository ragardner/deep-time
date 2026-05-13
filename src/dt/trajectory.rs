use crate::{C_SQUARED, Drift, Dt, Real, Spacetime, Velocity};

impl Dt {
    /// Computes the accumulated proper time \(\Delta\tau\) experienced by a clock
    /// traveling along a trajectory given by a sequence of physical states.
    ///
    /// Each input triple \((t, v, \Phi)\) is automatically converted into the
    /// internal [`Spacetime`] representation required by the library’s
    /// unified master Lagrangian, then passed to [`proper_time_from_path`].
    ///
    /// Proper time \(\Delta\tau\) is the time actually measured by a real
    /// physical clock (e.g. an onboard spacecraft clock). It includes the
    /// relativistic effects of the clock's velocity and local gravitic conditions,
    /// plus the library’s built-in Planck-scale saturation term.
    ///
    /// # Parameters
    ///
    /// - `samples` — An iterator yielding tuples of the form
    ///   `(coordinate_time, velocity, Newtonian gravitational potential)`.
    ///   The coordinate time must be monotonically non-decreasing for physically
    ///   meaningful results.  
    ///   - `coordinate_time`: a [`Dt`] value (any [`Scale`] is accepted).  
    ///   - `velocity`: a [`Velocity`] in m/s.  
    ///   - `grav_potential`: \(\Phi\) in m² s⁻² (Newtonian potential; negative
    ///     for bound orbits). The library converts this to the lapse factor
    ///     \(\alpha = \sqrt{1 + 2\Phi/c^2}\).
    ///
    /// - `characteristic_length_scale` — A length in meters that controls whether
    ///   the weak-field or strong-field formulation is used.
    ///   - Pass `0.0` (the recommended default for solar-system, GNSS, and
    ///     cislunar work) to recover the classic general-relativistic clock rate
    ///     exactly as used by JPL, ESA, and SPICE pipelines.
    ///   - Supply a realistic non-zero value (e.g. the local scale height of
    ///     the gravitational field) only when operating near neutron stars,
    ///     black-hole horizons, or other regions where spacetime curvature
    ///     becomes extreme. This activates the Planck-scale saturation term
    ///     encoded in \(K_{\rm eff}\).
    ///
    /// # Returns
    ///
    /// The total accumulated proper-time interval \(\Delta\tau\) as a [`Dt`]
    /// value, computed with the library’s exact 36-digit arithmetic.
    ///
    /// # Examples
    ///
    /// **Basic solar-system usage (weak-field)**  
    /// ```rust
    /// use deep_time::{Dt, Velocity, Scale};
    ///
    /// let trajectory = vec![
    ///     (Dt::from_sec(0, Scale::TAI),   Velocity::from_speed(7800.0), -6.0e7),
    ///     (Dt::from_sec(3600, Scale::TAI), Velocity::from_speed(11000.0), -1.0e6),
    /// ];
    ///
    /// let delta_tau = Dt::proper_time_from_states(trajectory, 0.0);
    /// ```
    ///
    /// **Strong-field example (neutron-star vicinity)**  
    /// ```rust
    /// use deep_time::{Dt, Velocity, Scale};
    ///
    /// let trajectory = vec![
    ///     (Dt::from_sec(0, Scale::TAI),   Velocity::from_speed(7800.0), -6.0e7),
    ///     (Dt::from_sec(3600, Scale::TAI), Velocity::from_speed(11000.0), -1.0e6),
    /// ];
    ///
    /// let strong_scale = 1e4; // 10 km characteristic scale near neutron star
    /// let delta_tau = Dt::proper_time_from_states(trajectory, strong_scale);
    /// ```
    pub fn proper_time_from_states<I>(samples: I, characteristic_length_scale: Real) -> Self
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>, // (t, vel_m_s, Φ m²/s²)
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

    /// Computes the relativistic clock drift \(\Delta\tau - \Delta t\) for an
    /// onboard clock traveling along a trajectory given by a sequence of
    /// physical states.
    ///
    /// This function returns the net amount by which a real physical clock
    /// has gained or lost time relative to the coordinate time interval
    /// between `start` and `end`.
    ///
    /// # Parameters
    ///
    /// - `start` — The starting coordinate time of the interval (any [`Scale`]).
    /// - `end` — The ending coordinate time of the interval.
    /// - `states` — An iterator yielding tuples of the form
    ///   `(coordinate_time, velocity, Newtonian gravitational potential)`.
    ///   See [`proper_time_from_states`] for a detailed description of each
    ///   component.
    /// - `characteristic_length_scale` — A length in meters that controls
    ///   whether the weak-field or strong-field formulation is used.
    ///
    /// # Returns
    ///
    /// The total clock drift \(\Delta\tau - \Delta t\) as a [`Dt`] value,
    /// computed with the library’s exact 36-digit arithmetic.
    ///
    /// - A positive value means the onboard clock ran **fast** relative to
    ///   coordinate time.
    /// - A negative value means the onboard clock ran **slow** relative to
    ///   coordinate time.
    ///
    /// # Examples
    ///
    /// **Basic usage (weak-field solar-system trajectory)**
    /// ```rust
    /// use deep_time::{Dt, Velocity, Scale};
    ///
    /// let start = Dt::from_sec(0, Scale::TAI);
    /// let end   = Dt::from_sec(3600, Scale::TAI);
    ///
    /// let trajectory = vec![
    ///     (start, Velocity::from_speed(7800.0), -6.0e7),
    ///     (end,   Velocity::from_speed(11000.0), -1.0e6),
    /// ];
    ///
    /// let drift = Dt::proper_time_drift_from_states(
    ///     start,
    ///     end,
    ///     trajectory,
    ///     0.0,
    /// );
    /// ```
    pub fn proper_time_drift_from_states<I>(
        start: Dt,
        end: Dt,
        states: I,
        characteristic_length_scale: Real,
    ) -> Dt
    where
        I: IntoIterator<Item = (Self, Velocity, Real)>,
    {
        let dtau = Self::proper_time_from_states(states, characteristic_length_scale);
        let dt = end.to_diff_raw(start);
        dtau.sub(dt)
    }

    /// Computes the accumulated proper time \(\Delta\tau\) experienced by a clock
    /// traveling along an arbitrarily spaced trajectory in coordinate time.
    ///
    /// This is the core integration primitive of the library. It walks the supplied
    /// path segment-by-segment and integrates the instantaneous proper-time rate
    /// \(d\tau/dt = \sqrt{K_{\rm eff}}\) (derived from the library’s unified master
    /// Lagrangian). The result is the total signed proper-time interval from the
    /// first to the last point on the path.
    ///
    /// It accounts exactly for special-relativistic velocity time dilation,
    /// general-relativistic gravitational time dilation, and the intrinsic Planck-scale
    /// saturation term encoded in \(K_{\rm eff}\).
    ///
    /// The integration uses the same high-order quadrature already implemented in
    /// [`proper_time_between`]: composite Simpson’s rule for \(n \ge 2\) intervals
    /// or the trapezoidal rule for the two-point case.
    ///
    /// # Parameters
    ///
    /// - `path` — An iterator yielding at least two tuples of the form
    ///   `(coordinate_time, Spacetime)`. Each pair contains the coordinate
    ///   time at that instant and the corresponding [`Spacetime`] snapshot
    ///   (gravitational lapse factor \(\alpha\), local three-velocity magnitude
    ///   \(\beta\), and Kretschmann scalar \(\mathcal{K}\)).
    ///   The coordinate times must be monotonically non-decreasing for physically
    ///   consistent results.
    ///
    /// # Returns
    ///
    /// The total accumulated proper-time interval \(\Delta\tau\) as a [`Dt`] value.
    /// All final arithmetic is performed with the library’s exact 36-digit
    /// representation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Spacetime, Scale};
    ///
    /// // Sample non-uniform trajectory (e.g., denser sampling near periapsis
    /// // or during a gravity-assist flyby)
    /// let path: &[(Dt, Spacetime)] = &[
    ///     (Dt::from_sec(0, Scale::TAI),   Spacetime::new(0.999_000, 0.0, 0.0)),
    ///     (Dt::from_sec(120, Scale::TAI), Spacetime::new(0.998_500, 1.2e-4, 0.0)),
    ///     (Dt::from_sec(250, Scale::TAI), Spacetime::new(0.997_200, 3.5e-4, 0.0)), // higher density
    ///     (Dt::from_sec(400, Scale::TAI), Spacetime::new(0.999_200, 0.0, 0.0)),
    /// ];
    ///
    /// let delta_tau = Dt::proper_time_from_path(path.iter().copied());
    ///
    /// // Advance the onboard proper-time clock
    /// let onboard_tau = path[0].0.add(delta_tau);
    /// ```
    pub fn proper_time_from_path<I>(path: I) -> Self
    where
        I: IntoIterator<Item = (Self, Spacetime)>,
    {
        let mut iter = path.into_iter();

        let Some((mut prev_t, mut prev_ls)) = iter.next() else {
            return Self::ZERO;
        };

        let mut accumulated = Self::ZERO;

        for (t, ls) in iter {
            let segment = [prev_ls, ls];
            let dtau = prev_t.proper_time_between(t, &segment);

            accumulated = accumulated.add(dtau);

            prev_t = t;
            prev_ls = ls;
        }

        accumulated
    }

    /// Computes the accumulated proper time \(\Delta\tau\) experienced by a clock
    /// moving along a coordinate-time path from `self` to `end`.
    ///
    /// Proper time is the actual time measured by a real physical clock
    /// (onboard spacecraft clock, probe, etc.). This function evaluates the
    /// exact relativistic rate \(d\tau/dt = \sqrt{K_{\rm eff}}\) from the
    /// library’s unified master Lagrangian at each sample point and integrates
    /// the result using composite Simpson’s rule (or the trapezoidal rule for
    /// the two-point case).
    ///
    /// Use this when you have a fixed number of uniformly spaced
    /// [`Spacetime`] snapshots and need the integrated proper time over a
    /// single interval.
    ///
    /// # Parameters
    ///
    /// - `end` — The ending coordinate time of the interval (any [`Scale`]).
    /// - `samples` — A slice of [`Spacetime`] snapshots evaluated at
    ///   **uniformly spaced** points along the path. The slice must contain
    ///   at least two entries. These samples can be freely reused elsewhere
    ///   (e.g. for light-time calculations in [`ObserverState`]).
    ///
    /// # Returns
    ///
    /// The accumulated proper-time interval \(\Delta\tau\) as a [`Dt`] value,
    /// computed with the library’s exact 36-digit arithmetic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::{Scale, Spacetime, Dt};
    ///
    /// let start = Dt::from_sec(0, Scale::TAI);
    /// let end   = Dt::from_sec(1000, Scale::TAI);
    ///
    /// // Constant metric example (α = 0.9 → dτ/dt = 0.9)
    /// let slow = Spacetime::new(0.9, 0.0, 0.0);
    /// let samples = [slow; 2];
    ///
    /// let delta_tau = start.proper_time_between(end, &samples);
    /// assert_eq!(delta_tau, Dt::from_sec(900, Scale::TAI));
    ///
    /// // Update onboard proper time clock
    /// let onboard_tau = start.add(delta_tau);
    /// ```
    pub const fn proper_time_between(self, end: Dt, samples: &[Spacetime]) -> Dt {
        if samples.len() < 2 || self.eq(&end) {
            return Dt::ZERO;
        }

        let mut dt = end.to_diff_raw(self);
        let sign = if dt.sec < 0 { f!(-1.0) } else { f!(1.0) };
        if sign < f!(0.0) {
            dt = dt.neg();
        }

        let dt_sec = dt.to_sec_f();
        let num_intervals = samples.len() - 1;

        if num_intervals <= 1 {
            // Fast trapezoidal rule for constant-rate cases
            let rate0 = Self::rate_from_local(&samples[0]);
            let rate1 = Self::rate_from_local(&samples[samples.len() - 1]);
            let integral = f!(0.5) * (rate0 + rate1 - f!(2.0)) * dt_sec;
            return Dt::from_sec_f(sign * (dt_sec + integral));
        }

        // Simpson’s rule quadrature (high-order accuracy)
        let n = f!(num_intervals);
        let h = dt_sec / n;
        let mut s = f!(0.0);

        let mut i = 0;
        while i <= num_intervals {
            let local = &samples[i];
            let rate = Self::rate_from_local(local);

            let coeff = if i == 0 || i == num_intervals {
                f!(1.0)
            } else if i % 2 == 0 {
                f!(2.0)
            } else {
                f!(4.0)
            };
            s += coeff * (rate - f!(1.0));

            i += 1;
        }

        let integral = (h / f!(3.0)) * s;
        Dt::from_sec_f(sign * (dt_sec + integral))
    }

    /// Computes the accumulated proper time \(\Delta\tau\) when the instantaneous
    /// rate \(d\tau/dt\) is known to be constant.
    ///
    /// This is the fastest and clearest way to accumulate proper time for
    /// segments where the metric is unchanging, such as a ground station,
    /// circular orbit, or deep-space cruise phase.
    ///
    /// It is mathematically equivalent to calling [`proper_time_between`] with
    /// a two-element slice containing the same rate at both endpoints, but
    /// requires no allocation and expresses the intent more directly.
    ///
    /// # Parameters
    ///
    /// - `end` — The ending coordinate time of the interval (any [`Scale`]).
    /// - `dtau_dt` — The constant proper-time rate \(d\tau/dt\) (dimensionless).
    ///   Typical sources are:
    ///   - [`Spacetime::proper_time_rate`]
    ///   - [`Drift::proper_time_rate`]
    ///   - Any precomputed value of \(\sqrt{K_{\rm eff}}\) from the master Lagrangian.
    ///
    /// # Returns
    ///
    /// The accumulated proper-time interval \(\Delta\tau\) as a [`Dt`] value,
    /// computed with the library’s exact 36-digit arithmetic.
    #[inline]
    pub const fn proper_time_between_constant_rate(
        self,
        end: Dt,
        // can come from Drift::proper_time_rate() or Spacetime::proper_time_rate()
        dtau_dt: Real,
    ) -> Dt {
        let dt_sec = end.to_diff_raw(self).to_sec_f();
        Dt::from_sec_f(dtau_dt * dt_sec)
    }

    /// Computes the relativistic clock drift \(\Delta\tau - \Delta t\) using
    /// pre-computed, uniformly spaced [`Spacetime`] samples.
    ///
    /// This function returns the difference between the proper time \(\Delta\tau\)
    /// accumulated by a real physical clock and the elapsed coordinate time
    /// interval from `self` to `end`. Coordinate time is the reference time
    /// used in the ephemeris or simulation data to label each set of positions,
    /// velocities, and gravitational potentials.
    ///
    /// # Parameters
    ///
    /// - `end` — The ending coordinate time of the interval (any [`Scale`]).
    /// - `samples` — A slice of [`Spacetime`] snapshots evaluated at
    ///   **uniformly spaced** points along the path. The slice must contain
    ///   at least two entries. See [`proper_time_between`] for details on
    ///   the sampling requirements and usage examples.
    ///
    /// # Returns
    ///
    /// The total clock drift \(\Delta\tau - \Delta t\) as a [`Dt`] value,
    /// computed with the library’s exact 36-digit arithmetic.
    ///
    /// - A positive value means the onboard clock ran **fast** relative to
    ///   coordinate time.
    /// - A negative value means the onboard clock ran **slow** relative to
    ///   coordinate time.
    pub const fn relativistic_correction_between(self, end: Dt, samples: &[Spacetime]) -> Dt {
        let dtau = self.proper_time_between(end, samples);
        let dt = end.to_diff_raw(self);
        dtau.sub(dt)
    }

    /// Private helper: instantaneous proper-time rate dτ/dt from a `Spacetime` snapshot.
    #[inline]
    const fn rate_from_local(spacetime: &Spacetime) -> Real {
        let drift = Drift::from_spacetime(spacetime);
        f!(1.0) + drift.rate().to_sec_f()
    }
}
