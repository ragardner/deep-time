use crate::{ClockDrift, Delta, ObserverState, Real, TimePoint};

// ──────────────────────────────────────────────────────────────
// RelativisticTrajectory trait
// ──────────────────────────────────────────────────────────────

/// A trajectory or ephemeris capable of computing the accumulated **proper time** (τ)
/// along a coordinate-time path using the library’s *unified master-Lagrangian*
/// formulation.
///
/// Proper time is the time actually experienced by a moving clock (spacecraft, probe,
/// planet, etc.). The implementation automatically uses the exact relativistic rate
/// `dτ/dt = √K_eff` from the master Lagrangian (with intrinsic Planck-scale saturation
/// when `characteristic_length_scale > 0`).
///
/// This is the recommended integration point for any relativistic navigation,
/// clock steering, or deep-space mission simulation.
pub trait RelativisticTrajectory {
    /// Returns the **complete relativistic state** at coordinate time `t`.
    ///
    /// This is the only method you must implement.
    /// Everything else (proper-time rate, interval, correction) has high-quality
    /// default implementations that use the unified Lagrangian.
    fn relativistic_state_at(&self, t: TimePoint) -> ObserverState;

    /// Instantaneous proper-time rate `dτ/dt` at time `t`.
    ///
    /// Returns a value ≈ 1.0 in weak fields. In strong gravity or high velocity
    /// it can be noticeably lower (and never reaches zero thanks to the built-in
    /// Planck-scale core).
    fn proper_time_rate_at(&self, t: TimePoint) -> Real {
        let state = self.relativistic_state_at(t);
        let drift = ClockDrift::from_velocity_potential_and_scale(
            state.velocity.speed(),
            state.grav_potential_m2_s2,
            state.characteristic_length_scale,
        );
        f!(1.0) + drift.time_diff_after(&Delta::ZERO).as_sec_f()
    }

    /// Computes the proper-time interval Δτ between two coordinate times.
    ///
    /// Uses composite Simpson’s rule (very high accuracy) when `num_samples > 2`.
    /// Falls back to trapezoidal rule for `num_samples ≤ 2`.
    /// Negative intervals are handled correctly.
    fn proper_time_interval(&self, start: TimePoint, end: TimePoint, num_samples: usize) -> Delta {
        let mut dt = end.duration_since(start);
        if dt.is_zero() {
            return Delta::ZERO;
        }

        // Forward interval for quadrature; sign restored at the end
        let sign = if dt.sec < 0 { f!(-1.0) } else { f!(1.0) };
        if sign < f!(0.0) {
            dt = dt.neg();
        }

        let dt_sec = dt.as_sec_f();

        if num_samples <= 2 {
            // Fast trapezoidal path
            let rate0 = self.proper_time_rate_at(start);
            let rate1 = self.proper_time_rate_at(end);
            let integral = f!(0.5) * (rate0 + rate1 - f!(2.0)) * dt_sec;
            return Delta::from_sec_f(sign * (dt_sec + integral));
        }

        // Simpson’s rule quadrature (high-order accuracy)
        let n = num_samples as Real;
        let h = dt_sec / n;
        let mut s = f!(0.0);

        for i in 0..=num_samples {
            let lambda = (i as Real) / n;
            let t_i = start.add(Delta::from_sec_f(lambda * dt_sec));
            let rate = self.proper_time_rate_at(t_i);

            let coeff = if i == 0 || i == num_samples {
                f!(1.0)
            } else if i % 2 == 0 {
                f!(2.0)
            } else {
                f!(4.0)
            };
            s += coeff * (rate - f!(1.0)); // only integrate the relativistic deviation
        }

        let integral = (h / f!(3.0)) * s;
        Delta::from_sec_f(sign * (dt_sec + integral))
    }

    /// Relativistic correction: how much the onboard clock has gained or lost
    /// relative to coordinate time (`Δτ − Δt`).
    fn relativistic_correction(
        &self,
        start: TimePoint,
        end: TimePoint,
        num_samples: usize,
    ) -> Delta {
        let dtau = self.proper_time_interval(start, end, num_samples);
        let dt = end.duration_since(start);
        dtau.sub(dt)
    }
}
