use crate::{ClockDrift, Dt, LocalSpacetime, Real};

impl Dt {
    /// Computes the accumulated **proper time** (Δτ) experienced by a clock moving along a
    /// coordinate-time path from `self` to `end`.
    ///
    /// Proper time is the actual time measured by a real physical clock (onboard spacecraft
    /// clock, probe, etc.). This function evaluates the exact relativistic rate
    /// dτ/dt = √K_eff from the library’s unified master Lagrangian at each sample point
    /// and integrates using composite Simpson’s rule.
    ///
    /// Use this whenever velocity, gravitational potential, or spacetime curvature changes
    /// along the trajectory (e.g. planetary flybys, cislunar transfers, deep-space maneuvers,
    /// or strong-field regions). It automatically includes special-relativistic velocity
    /// effects, general-relativistic gravitational time dilation, and the built-in
    /// Planck-scale saturation term.
    ///
    /// # Parameters
    /// - `end` — the ending coordinate time of the interval.
    /// - `samples` — slice of `LocalSpacetime` snapshots evaluated at **uniformly spaced**
    ///   points along the path (must contain at least two entries). These samples can be
    ///   freely reused elsewhere (e.g. for light-time calculations in `ObserverState`).
    ///
    /// # Returns
    /// The accumulated proper-time interval Δτ (exact 36-digit precision).
    ///
    /// # Example
    /// ```rust
    /// use deep_time::{Scale, LocalSpacetime, Dt};
    ///
    /// let start = Dt::from_sec(0, Scale::TAI);
    /// let end   = Dt::from_sec(1000, Scale::TAI);
    ///
    /// // Constant metric example (α = 0.9 → dτ/dt = 0.9)
    /// let slow = LocalSpacetime::new(0.9, 0.0, 0.0);
    /// let samples = [slow; 2];
    ///
    /// let delta_tau = start.proper_time_interval_samples(end, &samples);
    /// assert_eq!(delta_tau, Dt::from_sec(900, Scale::TAI));
    ///
    /// // Update onboard proper time clock
    /// let onboard_tau = start.to(Scale::Custom).add(delta_tau);
    /// ```
    pub const fn proper_time_interval_samples(self, end: Dt, samples: &[LocalSpacetime]) -> Dt {
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

    /// Computes the relativistic correction (Δτ − Δt) using pre-computed samples.
    ///
    /// Returns how much the onboard clock has gained or lost relative to coordinate time.
    /// Positive values mean the clock ran fast; negative values mean it ran slow.
    ///
    /// # Parameters
    /// - `end` — ending coordinate time.
    /// - `samples` — uniformly spaced `LocalSpacetime` snapshots (see
    ///   [`proper_time_interval_samples`] for details and example).
    ///
    /// # Returns
    /// The relativistic correction as a `Dt`.
    pub const fn relativistic_correction_with_samples(
        self,
        end: Dt,
        samples: &[LocalSpacetime],
    ) -> Dt {
        let dtau = self.proper_time_interval_samples(end, samples);
        let dt = end.to_diff_raw(self);
        dtau.sub(dt)
    }

    /// Private helper: instantaneous proper-time rate dτ/dt from a `LocalSpacetime` snapshot.
    #[inline]
    const fn rate_from_local(spacetime: &LocalSpacetime) -> Real {
        let drift = ClockDrift::from_local_spacetime(spacetime);
        f!(1.0) + drift.rate().to_sec_f()
    }
}
