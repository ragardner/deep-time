use crate::{C, ClockDrift, Delta, Position, Timestamp};

impl Timestamp {
    /// Computes the relativistic correction (first-order post-Newtonian clock-rate
    /// effect using the trapezoidal average plus Shapiro delay) to be added to
    /// the Newtonian geometric light time `|r_rx − r_tx| / c`.
    ///
    /// # Layman explanation
    ///
    /// Your spacecraft and the receiver (e.g. Earth station) clocks tick at
    /// slightly different rates because of motion (special relativity) and
    /// gravity (general relativity). In addition, the radio signal itself
    /// takes a tiny bit longer to travel when it passes near the Sun because
    /// gravity curves spacetime (the Shapiro delay). This function returns the
    /// combined correction you add to the simple “distance divided by speed of
    /// light” calculation to achieve nanosecond-level accuracy.
    ///
    /// # Mathematics (first-order post-Newtonian)
    ///
    /// Clock-rate offset (weak-field approximation):
    /// ```math
    /// \delta \approx -\frac{v^2}{2c^2} - \frac{\Phi}{c^2} \quad (\Phi = +GM/r > 0)
    /// ```
    ///
    /// Drift correction (trapezoidal average):
    /// ```math
    /// \text{drift_correction} \approx \frac{\delta_\text{tx} + \delta_\text{rx}}{2} \times \Delta t
    /// ```
    ///
    /// Shapiro delay (Sun only):
    /// ```math
    /// \Delta t_\text{Shapiro} = \frac{2GM_\odot}{c^3} \ln\left( \frac{r_\text{tx} + r_\text{rx} + d}{r_\text{tx} + r_\text{rx} - d} \right)
    /// ```
    ///
    /// This formulation is consistent with the standard relativistic light-time
    /// model used in JPL’s Orbit Determination Program (Moyer, 2000) and modern
    /// deep-space navigation.
    ///
    /// # When to use
    /// - Routine interplanetary navigation: this function (fastest).
    /// - Production code requiring light-time consistency: `iterative_one_way_relativistic_delay`.
    /// - High-precision applications beyond ~50 AU or with picosecond-level clocks:
    ///   `one_way_relativistic_delay_integrated`.
    pub fn one_way_relativistic_delay(
        tx_time: Self,
        rx_time_approx: Self,
        tx_v2_over_2c2: f64,
        tx_phi_over_c2: f64,
        rx_v2_over_2c2: f64,
        rx_phi_over_c2: f64,
        tx_pos: Position,
        rx_pos: Position,
    ) -> Delta {
        let dt = rx_time_approx.duration_since(tx_time);

        let tx_drift = ClockDrift::from_weak_field_approximation(tx_v2_over_2c2, tx_phi_over_c2);
        let rx_drift = ClockDrift::from_weak_field_approximation(rx_v2_over_2c2, rx_phi_over_c2);

        let drift_correction = tx_drift.evaluate(dt).add(rx_drift.evaluate(dt)).div_by_2();

        let r_tx = tx_pos.norm();
        let r_rx = rx_pos.norm();
        let r_sep = tx_pos.distance_to(rx_pos);
        let shapiro = Self::shapiro_one_way_delay(r_tx, r_rx, r_sep);

        drift_correction.add(shapiro)
    }

    /// Iteratively solves for the receive time and relativistic correction until
    /// the solution is consistent to the requested tolerance.
    ///
    /// # Layman explanation
    ///
    /// The true arrival time depends on the very delay we are calculating.
    /// This function keeps refining its guess of the arrival time (including both
    /// the straight-line travel time and the relativistic effects), recomputes
    /// the correction, and stops when the answer stops changing. It typically
    /// converges in 3–5 steps to sub-nanosecond accuracy even for outer-solar-system
    /// distances.
    pub fn iterative_one_way_relativistic_delay<F>(
        tx_time: Self,
        tx_v2_over_2c2: f64,
        tx_phi_over_c2: f64,
        tx_pos: Position,
        mut rx_time_approx: Self,
        mut rx_state_provider: F,
        tolerance: Delta,
        max_iter: usize,
    ) -> (Delta, Self)
    where
        F: FnMut(Self) -> (f64, f64, Position),
    {
        let mut rel_correction = Delta::ZERO;
        for _ in 0..max_iter {
            let (rx_v2, rx_phi, rx_pos) = rx_state_provider(rx_time_approx);

            rel_correction = Self::one_way_relativistic_delay(
                tx_time,
                rx_time_approx,
                tx_v2_over_2c2,
                tx_phi_over_c2,
                rx_v2,
                rx_phi,
                tx_pos,
                rx_pos,
            );

            let r_sep = tx_pos.distance_to(rx_pos);
            let geometric = Delta::from_sec_f64(r_sep / C);

            let full_delay = geometric.add(rel_correction);
            let new_rx_time = tx_time + full_delay;
            let change = new_rx_time.duration_since(rx_time_approx);

            rx_time_approx = new_rx_time;
            if change < tolerance {
                return (rel_correction, rx_time_approx);
            }
        }
        (rel_correction, rx_time_approx) // best-effort result
    }

    /// Computes the relativistic correction using numerical quadrature of the
    /// clock-rate offset along the signal path (Simpson’s rule). For low sample
    /// counts the routine falls back to the same trapezoidal average used by
    /// `one_way_relativistic_delay`.
    ///
    /// # Layman explanation
    ///
    /// For very long journeys (Kuiper Belt, interstellar precursors) or
    /// ultra-precise clocks, simply averaging the two endpoints is no longer
    /// perfect. This version samples the relativistic rate at many points
    /// *along the light path* and integrates the difference properly — the
    /// gold-standard approach for the deepest solar-system and future missions.
    pub fn one_way_relativistic_delay_integrated<F>(
        tx_time: Self,
        rx_time_approx: Self,
        tx_v2_over_2c2: f64,
        tx_phi_over_c2: f64,
        rx_v2_over_2c2: f64,
        rx_phi_over_c2: f64,
        num_samples: usize, // 5–21 samples suffice; ≤ 2 falls back to trapezoidal
        path_sampler: F,    // λ ∈ [0,1] → relative δ(λ) = δ_rx(λ) − δ_tx(λ)
        tx_pos: Position,
        rx_pos: Position,
    ) -> Delta
    where
        F: Fn(f64) -> f64,
    {
        let dt = rx_time_approx.duration_since(tx_time);
        let dt_sec = dt.as_sec_f64();

        let tx_drift = ClockDrift::from_weak_field_approximation(tx_v2_over_2c2, tx_phi_over_c2);
        let rx_drift = ClockDrift::from_weak_field_approximation(rx_v2_over_2c2, rx_phi_over_c2);

        if num_samples <= 2 {
            let drift_correction = tx_drift.evaluate(dt).add(rx_drift.evaluate(dt)).div_by_2();
            let r_tx = tx_pos.norm();
            let r_rx = rx_pos.norm();
            let r_sep = tx_pos.distance_to(rx_pos);
            let shapiro = Self::shapiro_one_way_delay(r_tx, r_rx, r_sep);
            return drift_correction.add(shapiro);
        }

        // Simpson’s rule quadrature over normalized path parameter λ ∈ [0,1]
        let n = num_samples as f64;
        let h = 1.0 / n;
        let mut s = path_sampler(0.0) + path_sampler(1.0);
        for i in 1..num_samples {
            let lambda = i as f64 * h;
            let f = path_sampler(lambda);
            let coeff = if i % 2 == 0 { 2.0 } else { 4.0 };
            s += coeff * f;
        }
        let integrated_drift_sec = (h / 3.0) * s * dt_sec;

        let r_tx = tx_pos.norm();
        let r_rx = rx_pos.norm();
        let r_sep = tx_pos.distance_to(rx_pos);
        let shapiro = Self::shapiro_one_way_delay(r_tx, r_rx, r_sep);

        Delta::from_sec_f64(integrated_drift_sec).add(shapiro)
    }

    /// Computes the relativistic correction for a two-way round-trip ranging
    /// measurement (transmit → receive → immediate transponder reply).
    ///
    /// # Layman explanation
    ///
    /// Deep-space networks measure distance by sending a signal to a spacecraft
    /// and timing how long the reply takes. This function returns the tiny
    /// relativistic adjustment you must subtract from the raw measured round-trip
    /// time to recover the true geometric distance.
    pub fn round_trip_relativistic_correction(
        tx_time: Self,
        round_trip_measured: Delta,
        tx_v2_over_2c2: f64,
        tx_phi_over_c2: f64,
        rx_v2_over_2c2: f64,
        rx_phi_over_c2: f64,
        tx_pos: Position,
        rx_pos: Position,
    ) -> Delta {
        let one_way_approx = round_trip_measured.div_by_2();

        let one_way_delay = Self::one_way_relativistic_delay(
            tx_time,
            tx_time.add(one_way_approx),
            tx_v2_over_2c2,
            tx_phi_over_c2,
            rx_v2_over_2c2,
            rx_phi_over_c2,
            tx_pos,
            rx_pos,
        );

        one_way_delay.add(one_way_delay)
    }

    /// First-order one-way Shapiro delay (gravitational light-time delay) in the Sun’s field.
    ///
    /// # Layman explanation
    ///
    /// Radio signals passing close to the Sun are delayed by a few microseconds
    /// because spacetime is curved. This is the famous Shapiro time delay first
    /// measured in 1964 and now a routine correction in deep-space navigation.
    fn shapiro_one_way_delay(r_tx: f64, r_rx: f64, d: f64) -> Delta {
        if r_tx <= 0.0 || r_rx <= 0.0 || d <= 0.0 {
            return Delta::ZERO;
        }

        let arg = (r_tx + r_rx + d) / (r_tx + r_rx - d).max(1.0);
        let delay_sec = crate::TWO_GM_SUN_OVER_C3 * arg.ln();

        Delta::from_sec_f64(delay_sec)
    }
}
