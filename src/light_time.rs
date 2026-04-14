use crate::{
    C, C_SQUARED, ClockDrift, Delta, LocalSpacetime, Position, TWO_GM_SUN_OVER_C3, TimePoint,
    Velocity,
};

/// Configuration for the **Shapiro delay** — the extra time light (or radio signals)
/// takes to travel near a massive body because gravity curves spacetime.
///
/// In simple terms: when a light beam or radar pulse passes close to a star or planet,
/// general relativity makes the path take a tiny bit longer than it would in flat space.
/// This struct holds the single number that controls how strong that delay is for a
/// particular central body (the Sun, a planet, another star, etc.).
///
/// Used in high-precision light-time calculations, spacecraft ranging, pulsar timing,
/// and very-long-baseline interferometry (VLBI).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LightContext {
    /// 2GM/c³ in seconds — the characteristic gravitational time scale used in the
    /// one-way Shapiro delay formula.
    ///
    /// This is mathematically equal to `2 * GM / c³`, where:
    /// - `G` is Newton’s gravitational constant,
    /// - **M** is the mass of the central body,
    /// - `c` is the speed of light.
    ///
    /// “GM” (often written as a single value called the **standard gravitational parameter** or μ)
    /// is the product of G and M. In real-world astronomy it is measured very accurately as one
    /// combined number (in m³/s²), so we keep it together here.
    ///
    /// It tells the light-propagation code “how strong the gravitational slowing is”
    /// for this body. Bigger value = stronger delay effect.
    ///
    /// Most users never set this manually — just use `SOLAR` or `from_grav_param()`.
    pub two_grav_param_over_c3: f64,
}

impl LightContext {
    /// Ready-to-use value for our own Sun, based on the exact IAU 2015 recommended constants.
    ///
    /// Use this for any typical solar-system light-propagation calculation (radar to
    /// planets, spacecraft tracking, etc.).
    pub const SOLAR: Self = Self {
        two_grav_param_over_c3: TWO_GM_SUN_OVER_C3,
    };

    /// No gravitational delay at all (flat spacetime approximation).
    ///
    /// Perfect for:
    /// - interstellar or intergalactic distances,
    /// - quick “ignore gravity” calculations,
    /// - or when you want to apply your own custom gravitational model elsewhere.
    pub const FLAT: Self = Self {
        two_grav_param_over_c3: 0.0,
    };

    /// Creates a `LightContext` for any central body (planet, star, black hole, etc.).
    ///
    /// # Arguments
    ///
    /// * `grav_param` — the body’s **standard gravitational parameter** (GM or μ)
    ///   in m³/s². This is the product of Newton’s gravitational constant and the body’s mass.
    pub const fn from_grav_param(grav_param: f64) -> Self {
        Self {
            two_grav_param_over_c3: 2.0 * grav_param / (C * C_SQUARED),
        }
    }
}

/// A complete relativistic state of an observer (spacecraft, ground station,
/// planet, etc.) at a specific instant.
///
/// This is the natural input type for all relativistic light-time calculations
/// in the library. It bundles position, velocity, gravitational potential, and
/// an optional length scale in convenient SI units.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct ObserverState {
    /// TimePoint of this state (any [`ClockType`] is accepted).
    pub time: TimePoint,
    /// Position in meters (typically barycentric or heliocentric).
    pub position: Position,
    /// Velocity in meters per second.
    pub velocity: Velocity,
    /// Local gravitational potential Φ in m² s⁻² (negative for bound orbits).
    /// Usually the sum of contributions from the Sun and planets.
    pub grav_potential_m2_s2: f64,
    /// Characteristic length scale (in meters) over which gravity varies
    /// significantly at the observer’s location.  
    /// Pass `0.0` (the default) for all solar-system, GNSS, and weak-field cases.
    pub characteristic_length_scale: f64,
}

impl ObserverState {
    /// Creates a new state for typical solar-system or GNSS use.
    #[inline]
    pub const fn new(
        time: TimePoint,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: f64,
    ) -> Self {
        Self {
            time,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale: 0.0,
        }
    }

    /// Creates a new state when strong-field or gravimeter data is available.
    #[inline]
    pub const fn new_strong_field(
        time: TimePoint,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: f64,
        characteristic_length_scale: f64,
    ) -> Self {
        Self {
            time,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale,
        }
    }

    /// Computes the total relativistic correction that must be added to the Newtonian
    /// geometric light time (`|r_rx − r_tx| / c`) for a one-way signal.
    ///
    /// This function accounts for two physical effects:
    /// - Differential clock-rate drift between transmitter and receiver (special-relativistic
    ///   velocity + general-relativistic gravitational time dilation) using the library’s
    ///   unified master-Lagrangian proper-time model.
    /// - Gravitational (Shapiro) delay caused by spacetime curvature near a central mass.
    ///
    /// Use cases include:
    /// - Deep-space tracking and ranging (DSN, ESA, JPL)
    /// - GNSS and satellite navigation
    /// - Pulsar timing arrays
    /// - Laser communication or ranging to distant spacecraft
    /// - Future interstellar missions where signals pass near other stars or black holes
    ///
    /// # Parameters
    /// - `self` – the transmitter’s full relativistic state at the moment the signal is sent
    /// - `rx` – the receiver’s relativistic state at the approximate arrival time
    /// - `context` – controls the gravitational (Shapiro) contribution. Use `LightContext::SOLAR`
    ///   for solar-system work, `LightContext::FLAT` when you want no central-mass delay,
    ///   or `LightContext::from_grav_param(your_gravitational_parameter)` for any other star, planet,
    ///   or black hole.
    ///
    /// # Returns
    /// A [`Delta`] (in seconds) to be **added** to the Newtonian geometric light time.
    ///
    /// # Examples
    ///
    /// Basic usage for a solar-system one-way light-time correction (e.g. Earth to Mars):
    ///
    /// ```no_run
    /// use deep_time_core::{
    ///     ObserverState, Position, Velocity, TimePoint, Delta, LightContext,
    ///     // Assume you have ephemeris functions or constants available
    /// };
    ///
    /// # let tx_time: TimePoint = todo!();
    /// # let tx_pos: Position = todo!();
    /// # let tx_vel: Velocity = todo!();
    /// # let tx_potential: f64 = todo!();
    /// # let rx_approx_time: TimePoint = todo!();
    /// # let rx_pos: Position = todo!();
    /// # let rx_vel: Velocity = todo!();
    /// # let rx_potential: f64 = todo!();
    ///
    /// let transmitter = ObserverState::new(
    ///     tx_time,
    ///     tx_pos,
    ///     tx_vel,
    ///     tx_potential,
    /// );
    ///
    /// let receiver_approx = ObserverState::new(
    ///     rx_approx_time,
    ///     rx_pos,
    ///     rx_vel,
    ///     rx_potential,
    /// );
    ///
    /// // Use SOLAR for Sun-centered calculations
    /// let correction: Delta = transmitter
    ///     .one_way_relativistic_delay_to(receiver_approx, LightContext::SOLAR);
    ///
    /// // The result should be added to the Newtonian geometric delay `r_sep / C`
    /// ```
    ///
    /// For a custom body (e.g. Jupiter):
    ///
    /// ```ignore
    /// let jupiter_context = LightContext::from_grav_param(jupiter_gm);  // GM in m³/s²
    /// let correction = tx.one_way_relativistic_delay_to(rx, jupiter_context);
    /// ```
    ///
    /// # Multi-body and exotic environments
    ///
    /// This function models the Shapiro delay from only a **single central mass**
    /// via the supplied `LightContext`. For signals that pass near multiple massive
    /// bodies (e.g. two stars, a star and a planet, or a binary black-hole system)
    /// or in highly dynamic/strong-field regimes, the single-body approximation may
    /// not be sufficient.
    ///
    /// In those cases consider using [`one_way_relativistic_delay_integrated`] instead,
    /// which lets you supply your own full spacetime model along the entire path.
    /// Alternatively, you can compute individual Shapiro contributions from each body
    /// (using the helper `shapiro_one_way_for_body` if you add it) and manually combine
    /// them with the result of this function.
    pub fn one_way_relativistic_delay_to(&self, rx: ObserverState, context: LightContext) -> Delta {
        let dt = rx.time.duration_since(self.time);

        let tx_drift = ClockDrift::from_velocity_potential_and_scale(
            self.velocity.speed(),
            self.grav_potential_m2_s2,
            self.characteristic_length_scale,
        );
        let rx_drift = ClockDrift::from_velocity_potential_and_scale(
            rx.velocity.speed(),
            rx.grav_potential_m2_s2,
            rx.characteristic_length_scale,
        );

        let drift_correction = rx_drift.evaluate(dt).sub(tx_drift.evaluate(dt));

        let r_tx = self.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = self.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(context, r_tx, r_rx, r_sep);

        drift_correction.add(shapiro)
    }

    /// Iteratively solves for the true receive time and the corresponding relativistic
    /// correction for a one-way signal.
    ///
    /// Because the exact arrival time depends on the relativistic correction itself,
    /// an iterative approach is required. The function typically converges in 3–5
    /// iterations to sub-nanosecond accuracy, even for outer-solar-system distances.
    ///
    /// # Parameters
    /// - `self` – the transmitter’s relativistic state (fixed)
    /// - `rx_provider` – a closure that, given a guessed receive [`TimePoint`], returns
    ///   the full [`ObserverState`] of the receiver at that time. You usually create
    ///   this closure by calling your ephemeris/orbit propagator.
    /// - `context` – gravitational context (`LightContext::SOLAR`, `LightContext::FLAT`,
    ///   or a custom value). See [`one_way_relativistic_delay_to`] for details.
    /// - `tolerance` – maximum allowed change in receive time between iterations
    ///   (recommended `Delta::from_ns(1)` or tighter)
    /// - `max_iter` – safety limit on the number of iterations (recommended 8–12)
    ///
    /// # Returns
    /// A tuple `(correction, final_rx_time)` where:
    /// - `correction` is the relativistic delay (same as returned by [`one_way_relativistic_delay_to`])
    /// - `final_rx_time` is the converged receive [`TimePoint`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use deep_time_core::{ObserverState, TimePoint, Delta, LightContext, Position, Velocity};
    ///
    /// # // Assume these exist in your code (e.g. from an ephemeris library)
    /// # let tx_time: TimePoint = todo!();
    /// # let tx_pos: Position = todo!();
    /// # let tx_vel: Velocity = todo!();
    /// # let tx_potential: f64 = todo!();
    /// # fn get_receiver_state_at(t: TimePoint) -> (Position, Velocity, f64) { todo!() }
    ///
    /// let transmitter = ObserverState::new(
    ///     tx_time,
    ///     tx_pos,
    ///     tx_vel,
    ///     tx_potential,
    /// );
    ///
    /// let (correction, final_rx_time) = transmitter.iterative_one_way_relativistic_delay_to(
    ///     |guessed_rx_time| {
    ///         // Call your ephemeris / orbit propagator here
    ///         let (pos, vel, potential) = get_receiver_state_at(guessed_rx_time);
    ///         ObserverState::new(guessed_rx_time, pos, vel, potential)
    ///     },
    ///     LightContext::SOLAR,
    ///     Delta::from_ns(1),   // 1 nanosecond tolerance (recommended)
    ///     12,                  // safety limit (recommended)
    /// );
    ///
    /// // `correction` is the total relativistic delay (clock drift + Shapiro)
    /// // to add to the Newtonian geometric light time.
    /// // `final_rx_time` is the accurately converged signal arrival time.
    /// ```
    ///
    /// Using a custom central body (e.g. near Jupiter) and a tighter tolerance:
    ///
    /// ```ignore
    /// let jupiter_gm = 1.26686534e17_f64; // m³/s²
    /// let context = LightContext::from_grav_param(jupiter_gm);
    ///
    /// let (correction, rx_time) = tx.iterative_one_way_relativistic_delay_to(
    ///     rx_provider, context, Delta::from_ns(0.1), 10
    /// );
    /// ```
    pub fn iterative_one_way_relativistic_delay_to<F>(
        &self,
        mut rx_provider: F,
        context: LightContext,
        tolerance: Delta,
        max_iter: usize,
    ) -> (Delta, TimePoint)
    where
        F: FnMut(TimePoint) -> ObserverState,
    {
        let mut rx = rx_provider(self.time);
        let mut rel_correction = Delta::ZERO;

        for _ in 0..max_iter {
            rel_correction = self.one_way_relativistic_delay_to(rx, context);

            let r_sep = self.position.distance_to(rx.position);
            let geometric = Delta::from_sec_f(r_sep / C);
            let full_delay = geometric.add(rel_correction);

            let new_rx_time = self.time.add(full_delay);
            let change = new_rx_time.duration_since(rx.time);

            rx = rx_provider(new_rx_time);
            rx.time = new_rx_time;

            if change < tolerance {
                return (rel_correction, new_rx_time);
            }
        }
        (rel_correction, rx.time)
    }

    /// Computes the relativistic correction using numerical quadrature (Simpson’s rule)
    /// of the relative clock-rate offset along the entire straight-line light path.
    ///
    /// This is the most accurate method when the clock-rate offset varies continuously
    /// along the path (long baselines, interstellar distances, or strong-field regions).
    ///
    /// # Parameters
    /// - `self` – the transmitter’s relativistic state
    /// - `rx` – the receiver’s relativistic state
    /// - `context` – gravitational context for the Shapiro delay (see [`one_way_relativistic_delay_to`])
    /// - `samples` – a slice of [`LocalSpacetime`] snapshots uniformly spaced along the
    ///   path (λ ∈ [0, 1]). You build this slice by evaluating your spacetime model
    ///   at several points between transmitter and receiver. Even 9–21 samples give
    ///   excellent accuracy.
    ///
    /// # Returns
    /// A [`Delta`] containing the integrated clock-drift correction plus the Shapiro
    /// delay from the supplied `context`.
    ///
    /// # Example of building `samples`
    /// ```ignore
    /// let samples: Vec<LocalSpacetime> = (0..=15)
    ///     .map(|i| {
    ///         let lambda = i as f64 / 15.0;
    ///         let point = tx.position.lerp(rx.position, lambda);
    ///         let phi_over_c2 = compute_total_potential_at(point); // your model
    ///         LocalSpacetime::from_potential_velocity_and_scale(
    ///             phi_over_c2,
    ///             Velocity::ZERO,
    ///             0.0, // weak-field
    ///         )
    ///     })
    ///     .collect();
    /// ```
    ///
    /// # Examples
    ///
    /// Full usage example for high-accuracy one-way light-time correction
    /// (e.g. interstellar distances or strong gravitational fields):
    ///
    /// ```no_run
    /// use deep_time_core::{
    ///     ObserverState, LocalSpacetime, Position, Velocity, TimePoint,
    ///     Delta, LightContext,
    /// };
    ///
    /// # let tx_time: TimePoint = todo!();
    /// # let tx_pos: Position = todo!();
    /// # let tx_vel: Velocity = todo!();
    /// # let tx_potential: f64 = todo!();
    /// # let rx_time: TimePoint = todo!();
    /// # let rx_pos: Position = todo!();
    /// # let rx_vel: Velocity = todo!();
    /// # let rx_potential: f64 = todo!();
    /// # fn compute_total_potential_at(pos: Position) -> f64 { todo!() }
    ///
    /// let transmitter = ObserverState::new(
    ///     tx_time, tx_pos, tx_vel, tx_potential,
    /// );
    ///
    /// let receiver = ObserverState::new(
    ///     rx_time, rx_pos, rx_vel, rx_potential,
    /// );
    ///
    /// // Build uniformly spaced samples along the straight-line path.
    /// // 9–21 points are usually sufficient; use more for interstellar/strong-field cases.
    /// let samples: Vec<LocalSpacetime> = (0..=21)
    ///     .map(|i| {
    ///         let lambda = i as f64 / 21.0;
    ///         let point = transmitter.position.lerp(receiver.position, lambda);
    ///         let phi_over_c2 = compute_total_potential_at(point);
    ///
    ///         LocalSpacetime::from_potential_velocity_and_scale(
    ///             phi_over_c2,
    ///             Velocity::ZERO,   // light itself carries no rest-mass velocity
    ///             0.0,              // weak-field approximation
    ///         )
    ///     })
    ///     .collect();
    ///
    /// let total_correction: Delta = transmitter.one_way_relativistic_delay_integrated(
    ///     receiver,
    ///     LightContext::SOLAR,
    ///     &samples,
    /// );
    ///
    /// // `total_correction` is the integrated clock-drift + Shapiro delay
    /// // to be added to the Newtonian geometric light time.
    /// ```
    ///
    /// Using a custom central body (e.g. near another star or planet):
    ///
    /// ```ignore
    /// let custom_context = LightContext::from_grav_param(star_gm); // GM in m³/s²
    /// let correction = tx.one_way_relativistic_delay_integrated(
    ///     rx, custom_context, &samples
    /// );
    /// ```
    ///
    /// # Multi-body and exotic environments
    ///
    /// This function is the recommended choice when a signal passes near multiple
    /// massive bodies (two or more stars, a star and a planet, binary black holes,
    /// etc.) or when operating in strong gravitational fields or highly dynamic
    /// spacetimes. Because you supply your own [`LocalSpacetime`] snapshots, each
    /// sample can incorporate the combined gravitational potential, velocity, and
    /// curvature from every relevant body in your model.
    ///
    /// In contrast, the faster [`one_way_relativistic_delay_to`] function only
    /// supports a single central mass via `LightContext`. For complex geometries
    /// or high-fidelity simulations, this integrated method provides greater
    /// accuracy and flexibility.
    pub fn one_way_relativistic_delay_integrated(
        &self,
        rx: ObserverState,
        context: LightContext,
        samples: &[LocalSpacetime],
    ) -> Delta {
        if samples.is_empty() {
            return self.one_way_relativistic_delay_to(rx, context);
        }

        let dt_sec = rx.time.duration_since(self.time).as_sec_f();

        let num_samples = samples.len();
        let n = num_samples as f64;
        let h = 1.0 / n;
        let mut s = 0.0_f64;

        for i in 0..num_samples {
            let local = samples[i];
            let drift = ClockDrift::from_local_spacetime(local);
            let rate_offset = drift.rate.as_sec_f();

            let coeff = if i == 0 || i == num_samples - 1 {
                1.0
            } else if i % 2 == 0 {
                2.0
            } else {
                4.0
            };
            s += coeff * rate_offset;
        }

        let integrated_drift_sec = (h / 3.0) * s * dt_sec;

        let r_tx = self.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = self.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(context, r_tx, r_rx, r_sep);

        Delta::from_sec_f(integrated_drift_sec).add(shapiro)
    }

    /// Computes the relativistic correction for a two-way round-trip ranging measurement
    /// (transmit → receive → immediate transponder reply).
    ///
    /// Deep-space networks measure distance by timing a round-trip signal. This function
    /// returns the tiny relativistic adjustment that must be **subtracted** from the raw
    /// measured round-trip time to recover the true geometric distance.
    ///
    /// # Parameters
    /// - `self` – the transmitter’s relativistic state at send time
    /// - `round_trip_measured` – the raw measured round-trip duration (in seconds)
    /// - `rx` – the receiver’s relativistic state (its `time` field is ignored)
    /// - `context` – gravitational context for the Shapiro delay (see [`one_way_relativistic_delay_to`])
    ///
    /// # Returns
    /// A [`Delta`] (in seconds) that must be **subtracted** from the measured round-trip time.
    ///
    /// # Examples
    ///
    /// Typical usage for deep-space two-way ranging (e.g. Earth to spacecraft or planet via DSN):
    ///
    /// ```no_run
    /// use deep_time_core::{
    ///     ObserverState, Position, Velocity, TimePoint, Delta, LightContext,
    /// };
    ///
    /// # let tx_time: TimePoint = todo!();
    /// # let tx_pos: Position = todo!();
    /// # let tx_vel: Velocity = todo!();
    /// # let tx_potential: f64 = todo!();
    /// # let rx_pos: Position = todo!();
    /// # let rx_vel: Velocity = todo!();
    /// # let rx_potential: f64 = todo!();
    /// # let measured_round_trip: Delta = todo!(); // from your ranging hardware / DSN
    ///
    /// let transmitter = ObserverState::new(
    ///     tx_time,
    ///     tx_pos,
    ///     tx_vel,
    ///     tx_potential,
    /// );
    ///
    /// // Receiver state at approximate arrival time (its `.time` field is ignored)
    /// let receiver_approx = ObserverState::new(
    ///     TimePoint::default(), // dummy time - will be ignored
    ///     rx_pos,
    ///     rx_vel,
    ///     rx_potential,
    /// );
    ///
    /// let relativistic_correction = transmitter.round_trip_relativistic_correction(
    ///     measured_round_trip,
    ///     receiver_approx,
    ///     LightContext::SOLAR,
    /// );
    ///
    /// // Correct the measured round-trip time:
    /// let corrected_round_trip = measured_round_trip.sub(relativistic_correction);
    ///
    /// // Then the true geometric one-way light time and distance can be computed from
    /// // `corrected_round_trip / 2`.
    /// ```
    ///
    /// Using a custom gravitational context (e.g. ranging to a probe near Jupiter):
    ///
    /// ```ignore
    /// let jupiter_context = LightContext::from_grav_param(jupiter_gm); // GM in m³/s²
    /// let correction = tx.round_trip_relativistic_correction(
    ///     measured, rx_approx, jupiter_context
    /// );
    /// ```
    pub fn round_trip_relativistic_correction(
        &self,
        round_trip_measured: Delta,
        rx: ObserverState,
        context: LightContext,
    ) -> Delta {
        let one_way_approx = round_trip_measured.div_by_2();
        let rx_approx = ObserverState {
            time: self.time.add(one_way_approx),
            ..rx
        };

        let one_way_delay = self.one_way_relativistic_delay_to(rx_approx, context);
        one_way_delay.add(one_way_delay)
    }

    /// First-order one-way Shapiro delay (gravitational light-time delay) caused by a
    /// central point mass.
    ///
    /// This is an internal helper used by the public delay functions. It implements the
    /// standard logarithmic formula used in solar-system navigation and pulsar timing.
    fn shapiro_one_way_delay(context: LightContext, r_tx: f64, r_rx: f64, r_sep: f64) -> Delta {
        if context.two_grav_param_over_c3 == 0.0 || r_tx <= 0.0 || r_rx <= 0.0 || r_sep <= 0.0 {
            return Delta::ZERO;
        }

        let arg = (r_tx + r_rx + r_sep) / (r_tx + r_rx - r_sep).max(1.0);
        let delay_sec = context.two_grav_param_over_c3 * libm::log(arg);

        Delta::from_sec_f(delay_sec)
    }
}

#[cfg(test)]
mod relativistic_tests {
    use super::*;
    use crate::{Delta, ObserverState, Position, TimePoint, Velocity};

    /// Small helper to build a `ObserverState` quickly.
    fn make_state(
        tai_sec: i128,
        pos: Position,
        vel: Velocity,
        phi_m2_s2: f64,
        char_scale: f64,
    ) -> ObserverState {
        ObserverState {
            time: TimePoint::from_tai_sec(tai_sec),
            position: pos,
            velocity: vel,
            grav_potential_m2_s2: phi_m2_s2,
            characteristic_length_scale: char_scale,
        }
    }

    #[test]
    fn zero_relativity_gives_zero_correction() {
        let origin = Position::zero();
        let zero_vel = Velocity::from_speed(0.0);
        let tx = make_state(0, origin, zero_vel, 0.0, 0.0);
        let rx = make_state(0, origin, zero_vel, 0.0, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::FLAT);
        assert_eq!(correction, Delta::ZERO);
    }

    #[test]
    fn pure_shapiro_delay_sun_grazing() {
        let au = 1.495978707e11_f64;
        let r_sun = 6.957e8_f64;
        let b = r_sun * 1.001;

        let tx_pos = Position::new(au, b, 0.0);
        let rx_pos = Position::new(-au, b, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let got_us = correction.as_sec_f() * 1_000_000.0;

        assert!(
            (got_us - 119.45).abs() < 0.5,
            "Sun-grazing Shapiro delay: got {:.3} µs (expected ~119.45 µs)",
            got_us
        );
    }

    #[test]
    fn clock_rate_correction_only() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

        let correction = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);
        let corr_sec = correction.as_sec_f();

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
        let (correction, final_rx_time) = tx.iterative_one_way_relativistic_delay_to(
            |guessed_time| {
                make_state(
                    guessed_time.sec(),
                    rx_pos,
                    Velocity::from_speed(0.0),
                    rx_phi,
                    0.0,
                )
            },
            LightContext::SOLAR,
            tolerance,
            12,
        );

        assert!(correction.as_sec_f().abs() < 1e-5);

        let geometric_sec = tx_pos.distance_to(rx_pos) / C;
        let total_sec = final_rx_time.duration_since(tx.time).as_sec_f();
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

        let round_trip_measured = Delta::from_sec_f(1010.0);

        let correction =
            tx.round_trip_relativistic_correction(round_trip_measured, rx, LightContext::SOLAR);
        let corr_sec = correction.as_sec_f();

        assert!(corr_sec > 0.0);
        assert!(corr_sec < 1e-3);
    }

    #[test]
    fn integrated_low_samples_matches_trapezoidal() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        let trapezoidal = tx.one_way_relativistic_delay_to(rx, LightContext::SOLAR);

        let samples = [LocalSpacetime::new(1.0, 0.0, 0.0); 2];

        let integrated =
            tx.one_way_relativistic_delay_integrated(rx, LightContext::SOLAR, &samples);

        let diff = (trapezoidal.as_sec_f() - integrated.as_sec_f()).abs();
        assert!(
            diff < 2e-7, // small numerical difference is expected when going through LocalSpacetime
            "n=2 mismatch: {}",
            diff
        );
    }

    #[test]
    fn integrated_high_samples_still_reasonable() {
        let tx_pos = Position::new(1.5e11, 0.0, 0.0);
        let rx_pos = Position::new(1.52e11, 0.0, 0.0);

        let tx = make_state(0, tx_pos, Velocity::from_speed(0.0), -8.87e8, 0.0);
        let rx = make_state(520, rx_pos, Velocity::from_speed(0.0), -8.80e8, 0.0);

        // 21 samples with zero relative drift (sanity check – no huge blow-up)
        let samples = [LocalSpacetime::new(1.0, 0.0, 0.0); 21];

        let integrated = tx.one_way_relativistic_delay_integrated(rx, LightContext::FLAT, &samples);

        let corr_sec = integrated.as_sec_f();
        assert!(corr_sec.abs() < 1e-3);
    }
}
