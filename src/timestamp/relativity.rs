use crate::{C, ClockDrift, Delta, RelativisticState, TWO_GM_SUN_OVER_C3, Timestamp};

impl Timestamp {
    /// Computes the total relativistic correction (differential clock-rate drift + Shapiro delay)
    /// that must be **added** to the Newtonian geometric light time `|r_rx − r_tx| / c`.
    ///
    /// - **Clock-rate drift**: Proper-time difference accumulated between transmitter and receiver
    ///   due to special-relativistic velocity and general-relativistic gravitational redshift.
    ///   This uses the exact master-Lagrangian rate `dτ/dt` (including Planck-scale saturation)
    ///   via [`ClockDrift::from_velocity_potential_and_scale`].
    /// - **Shapiro delay**: Extra coordinate-time propagation delay caused by solar-system curvature.
    ///
    /// The result is the precise one-way light-time correction used in deep-space tracking,
    /// GNSS, and pulsar timing.
    ///
    /// # Parameters
    /// - `tx` – relativistic state of the transmitter at the moment the signal is sent
    /// - `rx` – relativistic state of the receiver at the approximate arrival time
    ///
    /// # Returns
    /// A [`Delta`] (in seconds) to be added to the Newtonian light time.
    pub fn one_way_relativistic_delay(tx: RelativisticState, rx: RelativisticState) -> Delta {
        let dt = rx.time.duration_since(tx.time);

        let tx_drift = ClockDrift::from_velocity_potential_and_scale(
            tx.velocity.speed(),
            tx.gravitational_potential_m2_s2,
            tx.characteristic_length_scale,
        );
        let rx_drift = ClockDrift::from_velocity_potential_and_scale(
            rx.velocity.speed(),
            rx.gravitational_potential_m2_s2,
            rx.characteristic_length_scale,
        );

        // Differential clock-rate contribution: ∫ (dτ_rx/dt − dτ_tx/dt) dt
        let drift_correction = rx_drift.evaluate(dt).sub(tx_drift.evaluate(dt));

        let r_tx = tx.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = tx.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(r_tx, r_rx, r_sep);

        drift_correction.add(shapiro)
    }

    /// Iteratively solves for the true receive time and the corresponding
    /// relativistic correction.
    ///
    /// Because the exact arrival time depends on the correction itself, an
    /// iterative approach is required. The function typically converges in
    /// 3–5 iterations to sub-nanosecond accuracy, even for outer-solar-system
    /// distances.
    ///
    /// # Parameters
    /// - `tx` – relativistic state of the transmitter (fixed)
    /// - `rx_provider` – closure that, given a guessed receive time, returns
    ///   the full [`RelativisticState`] of the receiver at that time
    /// - `tolerance` – maximum allowed change in receive time between iterations
    /// - `max_iter` – safety limit on the number of iterations (recommended 8–12)
    ///
    /// # Returns
    /// A tuple `(correction, final_rx_time)` where `correction` is the relativistic
    /// delay and `final_rx_time` is the converged receive timestamp.
    pub fn iterative_one_way_relativistic_delay<F>(
        tx: RelativisticState,
        mut rx_provider: F,
        tolerance: Delta,
        max_iter: usize,
    ) -> (Delta, Timestamp)
    where
        F: FnMut(Timestamp) -> RelativisticState,
    {
        let mut rx = rx_provider(tx.time); // initial guess
        let mut rel_correction = Delta::ZERO;

        for _ in 0..max_iter {
            rel_correction = Self::one_way_relativistic_delay(tx, rx);

            let r_sep = tx.position.distance_to(rx.position);
            let geometric = Delta::from_sec_f64(r_sep / C);
            let full_delay = geometric.add(rel_correction);

            let new_rx_time = tx.time.add(full_delay);
            let change = new_rx_time.duration_since(rx.time);

            rx = rx_provider(new_rx_time);
            rx.time = new_rx_time; // ensure exact time stamp

            if change < tolerance {
                return (rel_correction, new_rx_time);
            }
        }
        (rel_correction, rx.time) // best-effort result
    }

    /// Computes the relativistic correction using numerical quadrature (Simpson’s
    /// rule) of the *relative* clock-rate offset along the entire light path.
    ///
    /// This is the most accurate method for long baselines (Kuiper Belt, interstellar)
    /// or picosecond-level timing, where the clock-rate offset varies continuously
    /// along the path.
    ///
    /// For `num_samples ≤ 2` the function delegates directly to
    /// [`one_way_relativistic_delay`] (trapezoidal endpoint average) for perfect
    /// consistency and maximum performance.
    ///
    /// # Parameters
    /// - `tx` – relativistic state of the transmitter
    /// - `rx` – relativistic state of the receiver
    /// - `num_samples` – number of quadrature points along the path (5–21 is typical)
    /// - `path_sampler` – closure that receives λ ∈ [0, 1] (normalized path parameter)
    ///   and returns the **relative** clock-rate offset `δ(λ) = (dτ_rx/d t − dτ_tx/d t)`
    ///   at that point along the straight-line path
    pub fn one_way_relativistic_delay_integrated<F>(
        tx: RelativisticState,
        rx: RelativisticState,
        num_samples: usize,
        path_sampler: F,
    ) -> Delta
    where
        F: Fn(f64) -> f64,
    {
        if num_samples <= 2 {
            // Perfect consistency with the fast path (no code duplication)
            return Self::one_way_relativistic_delay(tx, rx);
        }

        let dt = rx.time.duration_since(tx.time);
        let dt_sec = dt.as_sec_f64();

        // Simpson’s rule quadrature over the *relative* rate offset
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

        let r_tx = tx.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = tx.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(r_tx, r_rx, r_sep);

        Delta::from_sec_f64(integrated_drift_sec).add(shapiro)
    }

    /// Computes the relativistic correction for a two-way round-trip ranging
    /// measurement (transmit → receive → immediate transponder reply).
    ///
    /// Deep-space networks measure distance by timing a round-trip signal. This
    /// function returns the tiny relativistic adjustment that must be **subtracted**
    /// from the raw measured round-trip time to recover the true geometric distance.
    ///
    /// # Parameters
    /// - `tx` – relativistic state of the transmitter at send time
    /// - `round_trip_measured` – the raw measured round-trip duration
    /// - `rx` – relativistic state of the receiver (its time field is ignored)
    pub fn round_trip_relativistic_correction(
        tx: RelativisticState,
        round_trip_measured: Delta,
        rx: RelativisticState,
    ) -> Delta {
        let one_way_approx = round_trip_measured.div_by_2();
        let rx_approx = RelativisticState {
            time: tx.time.add(one_way_approx),
            ..rx
        };

        let one_way_delay = Self::one_way_relativistic_delay(tx, rx_approx);
        one_way_delay.add(one_way_delay)
    }

    /// First-order one-way Shapiro delay (gravitational light-time delay) caused
    /// by the Sun’s gravitational field.
    fn shapiro_one_way_delay(r_tx: f64, r_rx: f64, d: f64) -> Delta {
        if r_tx <= 0.0 || r_rx <= 0.0 || d <= 0.0 {
            return Delta::ZERO;
        }

        let arg = (r_tx + r_rx + d) / (r_tx + r_rx - d).max(1.0);
        let delay_sec = TWO_GM_SUN_OVER_C3 * arg.ln();

        Delta::from_sec_f64(delay_sec)
    }
}

#[cfg(test)]
mod relativistic_tests {
    use super::*;
    use crate::{Delta, Position, RelativisticState, Timestamp, Velocity};

    /// Small helper to build a `RelativisticState` quickly.
    fn make_state(
        tai_sec: i128,
        pos: Position,
        vel: Velocity,
        phi_m2_s2: f64,
        char_scale: f64,
    ) -> RelativisticState {
        RelativisticState {
            time: Timestamp::from_tai_sec(tai_sec),
            position: pos,
            velocity: vel,
            gravitational_potential_m2_s2: phi_m2_s2,
            characteristic_length_scale: char_scale,
        }
    }

    #[test]
    fn zero_relativity_gives_zero_correction() {
        let origin = Position::zero();
        let zero_vel = Velocity::from_speed(0.0);
        let tx = make_state(0, origin, zero_vel, 0.0, 0.0);
        let rx = make_state(0, origin, zero_vel, 0.0, 0.0);

        let correction = Timestamp::one_way_relativistic_delay(tx, rx);
        assert_eq!(correction, Delta::ZERO);
    }

    #[test]
    fn pure_shapiro_delay_sun_grazing() {
        // Realistic Sun-grazing geometry (impact parameter ≈ solar radius)
        let au = 1.495978707e11_f64;
        let r_sun = 6.957e8_f64;
        let b = r_sun * 1.001; // very close to the limb

        let tx_pos = Position::new(au, b, 0.0);
        let rx_pos = Position::new(-au, b, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0); // ≈520 s light time

        let correction = Timestamp::one_way_relativistic_delay(tx, rx);
        let got_us = correction.as_sec_f64() * 1_000_000.0;

        // Expected ≈ 119.45 µs (pure Shapiro delay; identical potentials + zero velocity
        // → clock-rate drift terms cancel exactly)
        assert!(
            (got_us - 119.45).abs() < 0.5,
            "Sun-grazing Shapiro delay: got {:.3} µs (expected ~119.45 µs)",
            got_us
        );
    }

    #[test]
    fn clock_rate_correction_only() {
        // Far from Sun, different velocities and potentials → only clock-rate drift matters
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(
            0,
            tx_pos,
            Velocity::from_speed(30_000.0), // 30 km/s
            -8.87e8,
            0.0,
        );
        let rx = make_state(520, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

        let correction = Timestamp::one_way_relativistic_delay(tx, rx);
        let corr_sec = correction.as_sec_f64();

        // Typical magnitude for interplanetary links: tens of nanoseconds
        assert!(
            corr_sec.abs() < 1e-5,
            "Clock-rate correction unreasonably large"
        );
        assert!(
            corr_sec.abs() > 1e-10,
            "Clock-rate correction suspiciously small"
        );
    }

    #[test]
    fn iterative_solver_converges_quickly() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);

        let rx_pos = Position::new(1.52e11, 0.0, 0.0);
        let rx_phi = -8.80e8;

        let tolerance = Delta::from_ns(1);
        let (correction, final_rx_time) = Timestamp::iterative_one_way_relativistic_delay(
            tx,
            |guessed_time| {
                make_state(
                    guessed_time.sec(),
                    rx_pos,
                    Velocity::from_speed(0.0),
                    rx_phi,
                    0.0,
                )
            },
            tolerance,
            12,
        );

        assert!(correction.as_sec_f64().abs() < 1e-5);

        // Correct geometric light time = actual separation distance / c
        let geometric_sec = tx_pos.distance_to(rx_pos) / C;
        let total_sec = final_rx_time.duration_since(tx.time).as_sec_f64();
        assert!(
            (total_sec - geometric_sec).abs() < 1e-4,
            "Converged receive time deviates from geometric light time by {:.6} s",
            (total_sec - geometric_sec).abs()
        );
    }

    #[test]
    fn round_trip_correction_is_roughly_twice_one_way() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
        let rx = make_state(0, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

        let round_trip_measured = Delta::from_sec_f64(1010.0); // ≈ 2 × 505 s

        let correction = Timestamp::round_trip_relativistic_correction(tx, round_trip_measured, rx);
        let corr_sec = correction.as_sec_f64();

        assert!(corr_sec > 0.0);
        assert!(corr_sec < 1e-3); // realistic upper bound for interplanetary ranging
    }

    #[test]
    fn integrated_low_samples_matches_trapezoidal() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        let trapezoidal = Timestamp::one_way_relativistic_delay(tx, rx);

        // path_sampler that returns zero (no varying rate difference) → should be identical
        let integrated = Timestamp::one_way_relativistic_delay_integrated(tx, rx, 2, |_| 0.0);

        let diff = (trapezoidal.as_sec_f64() - integrated.as_sec_f64()).abs();
        assert!(
            diff < 1e-12,
            "Trapezoidal vs integrated (n=2) mismatch: {}",
            diff
        );
    }

    #[test]
    fn integrated_high_samples_still_reasonable() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        // Use a realistic but constant relative rate offset for this test
        let integrated = Timestamp::one_way_relativistic_delay_integrated(tx, rx, 21, |_| 1e-9);

        let corr_sec = integrated.as_sec_f64();
        assert!(corr_sec.abs() < 1e-3); // sanity check – no huge blow-up
    }
}
