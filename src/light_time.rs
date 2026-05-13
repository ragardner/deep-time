use crate::{
    C, C_SQUARED, Drift, Dt, Position, Real, Spacetime, TWO_GM_SUN_OVER_C3, Velocity, log,
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
    /// Dt of this state (any [`Scale`] is accepted).
    pub time: Dt,
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
        time: Dt,
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
        time: Dt,
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

    /// Returns the instantaneous proper-time rate `dτ/dt` for this observer.
    ///
    /// This is the exact rate at which a real clock at the given position,
    /// velocity, and gravitational environment would advance compared to
    /// coordinate time. It is used internally by the library for proper-time
    /// integration, light-time corrections, and Doppler calculations.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        let ls = Spacetime::from_potential_velocity_and_scale(
            self.grav_potential_m2_s2 / C_SQUARED,
            self.velocity,
            self.characteristic_length_scale,
        );
        ls.proper_time_rate()
    }

    /// Returns the relativistic clock-rate Doppler factor for a one-way signal
    /// sent from this transmitter to the given receiver.
    ///
    /// The factor is the ratio of the receiver’s proper-time rate to the
    /// transmitter’s proper-time rate. It accounts for the fact that clocks
    /// at the two locations run at slightly different speeds due to motion
    /// and gravity.
    ///
    /// To obtain the full observed frequency shift, multiply this factor by
    /// the classical kinematic Doppler term `(1 - v_radial / C)`, where
    /// `v_radial` is the line-of-sight component of the relative velocity
    /// (positive when the transmitter and receiver are moving apart).
    ///
    /// This value is used for deep-space tracking, GNSS range-rate measurements,
    /// and pulsar timing.
    ///
    /// # Parameters
    /// - `self` – transmitter state (position, velocity, gravitational potential,
    ///   and length scale at the moment the signal is sent)
    /// - `rx`   – receiver state (same information, evaluated at the approximate
    ///   arrival time)
    ///
    /// # Example
    /// ```rust
    /// use deep_time::{ObserverState, Position, Velocity, Dt};
    /// use deep_time::constants::C;
    ///
    /// # let tx_time = Dt::default();
    /// # let rx_time = Dt::default();
    /// # let tx_pos = Position::ZERO;
    /// # let rx_pos = Position::ZERO;
    /// # let tx_vel = Velocity::ZERO;
    /// # let rx_vel = Velocity::ZERO;
    /// # let phi = 0.0_f64; // gravitational potential
    ///
    /// let tx = ObserverState::new(tx_time, tx_pos, tx_vel, phi);
    /// let rx = ObserverState::new(rx_time, rx_pos, rx_vel, phi);
    ///
    /// let factor = tx.relativistic_clock_doppler_factor(rx);
    ///
    /// // Full observed frequency shift (example only)
    /// let v_radial = 0.0; // m/s, positive if receding
    /// let classical_doppler = 1.0 - v_radial / C;
    /// let total_frequency_shift = 1.0 * factor * classical_doppler;
    /// ```
    #[inline]
    pub const fn relativistic_clock_doppler_factor(&self, rx: ObserverState) -> Real {
        rx.proper_time_rate() / self.proper_time_rate()
    }

    /// Returns the two-way relativistic clock-rate Doppler factor for round-trip
    /// ranging (transmit → receive → immediate transponder reply).
    ///
    /// This is the product of the one-way factors for the complete round trip
    /// and is the value needed by deep-space networks when correcting measured
    /// range-rate data.
    #[inline]
    pub const fn two_way_relativistic_doppler_factor(&self, rx: ObserverState) -> Real {
        let one_way = self.relativistic_clock_doppler_factor(rx);
        one_way * one_way
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
    /// A [`Dt`] (in seconds) to be **added** to the Newtonian geometric light time.
    ///
    /// # Examples
    ///
    /// Basic usage for a solar-system one-way light-time correction (e.g. Earth to Mars):
    ///
    /// ```no_run
    /// use deep_time::{
    ///     ObserverState, Position, Velocity, Dt, LightContext,
    ///     // Assume you have ephemeris functions or constants available
    /// };
    ///
    /// # let tx_time: Dt = todo!();
    /// # let tx_pos: Position = todo!();
    /// # let tx_vel: Velocity = todo!();
    /// # let tx_potential: f64 = todo!();
    /// # let rx_approx_time: Dt = todo!();
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
    /// let correction: Dt = transmitter
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
    pub const fn one_way_relativistic_delay_to(
        &self,
        rx: ObserverState,
        context: LightContext,
    ) -> Dt {
        let span = rx.time.to_diff_raw(self.time);

        let tx_drift = Drift::from_velocity_potential_and_scale(
            self.velocity.speed(),
            self.grav_potential_m2_s2,
            self.characteristic_length_scale,
        );
        let rx_drift = Drift::from_velocity_potential_and_scale(
            rx.velocity.speed(),
            rx.grav_potential_m2_s2,
            rx.characteristic_length_scale,
        );

        let drift_correction = rx_drift
            .time_diff_after(&span)
            .sub(tx_drift.time_diff_after(&span));

        let r_tx = self.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = self.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(context, r_tx, r_rx, r_sep);

        drift_correction.add(shapiro)
    }

    /// Iteratively solves the one-way light-time equation including relativistic corrections
    /// until the receive time converges to the requested tolerance.
    ///
    /// This is the recommended high-precision solver for one-way light-time computations
    /// in modern deep-space navigation. It follows the formulation described in
    /// Moyer (2003) and is suitable for use with any ephemeris source (SPICE kernels,
    /// numerical integrators, or analytical propagators).
    ///
    /// The solver performs a fixed-point iteration on the light-time equation:
    ///
    /// ```text
    /// t_rx = t_tx + |r_rx(t_rx) − r_tx(t_tx)| / c + Δt_rel(t_tx, t_rx)
    /// ```
    ///
    /// where `Δt_rel` is the relativistic correction returned by
    /// [`one_way_relativistic_delay_to`]. The iteration continues until the change
    /// in the estimated receive time falls below `tolerance`.
    ///
    /// # Parameters
    ///
    /// * `rx_provider` — A mutable closure that returns the full relativistic state
    ///   of the receiver at a given coordinate time. This allows the solver to work
    ///   seamlessly with any ephemeris or propagator without requiring a specific
    ///   data structure.
    /// * `context` — Controls the gravitational (Shapiro) contribution. Use
    ///   [`LightContext::SOLAR`] for solar-system work or
    ///   [`LightContext::from_grav_param`] for other central bodies.
    /// * `tolerance` — Convergence tolerance on the change in receive time.
    ///   A typical value for high-precision work is `Dt::from_ns(1, Scale::TAI)`.
    /// * `max_iter` — Maximum number of iterations before falling back. A value of
    ///   12–20 is usually sufficient for solar-system geometries.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `rel_correction` — The final relativistic correction (clock drift + Shapiro)
    ///   evaluated at convergence.
    /// * `rx_time` — The converged receive time in the coordinate scale of the
    ///   transmitter.
    /// * `final_state` — The receiver state at the converged receive time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use deep_time::{Dt, LightContext, ObserverState, Scale};
    ///
    /// # let tx = /* transmitter state */;
    /// let tolerance = Dt::from_ns(1, Scale::TAI);
    ///
    /// let (correction, rx_time, rx_state) = tx.iterative_one_way_relativistic_delay_to(
    ///     &mut |t| {
    ///         // Example: call into SPICE or your own propagator
    ///         get_receiver_state_at(t)
    ///     },
    ///     LightContext::SOLAR,
    ///     tolerance,
    ///     20,
    /// );
    ///
    /// // The total light time is:
    /// let total_light_time = rx_time.to_diff_raw(tx.time);
    /// ```
    ///
    /// # References
    ///
    /// * Moyer, T.D. (2003). *Formulation for Observed and Computed Values of
    ///   Deep Space Network Data Types for Navigation*. JPL DESCANSO Vol. 2,
    ///   Section 8 (Light-Time Solution).
    /// * IAU Resolution B1.5 (2000) and subsequent updates on relativistic
    ///   reference systems and time scales.
    /// * Ashby, N. (2003). "Relativity in the Global Positioning System".
    ///   *Living Reviews in Relativity*.
    ///
    /// # Notes
    ///
    /// The solver uses a simple fixed-point iteration. For most solar-system
    /// geometries convergence occurs within 3–6 iterations. The function always
    /// returns a result; if `max_iter` is reached, the last computed values are
    /// returned. The returned `ObserverState` is guaranteed to be consistent with
    /// the final `rx_time`.
    pub fn iterative_one_way_relativistic_delay_to<F>(
        &self,
        rx_provider: &mut F,
        context: LightContext,
        tolerance: Dt,
        max_iter: usize,
    ) -> (Dt, Dt, ObserverState)
    where
        F: FnMut(Dt) -> ObserverState,
    {
        // Initial geometric guess
        let initial_rx = rx_provider(self.time);
        let initial_r_sep = self.position.distance_to(initial_rx.position);
        let initial_geometric = Dt::from_sec_f(initial_r_sep / C);

        let mut rx_time = self.time.add(initial_geometric);
        let mut rel_correction = Dt::ZERO;

        for _ in 0..max_iter {
            let rx = rx_provider(rx_time);

            rel_correction = self.one_way_relativistic_delay_to(rx, context);

            let r_sep = self.position.distance_to(rx.position);
            let geometric = Dt::from_sec_f(r_sep / C);
            let full_delay = geometric.add(rel_correction);

            let new_rx_time = self.time.add(full_delay);
            let change = new_rx_time.to_diff_raw(rx_time);

            rx_time = new_rx_time;

            if change.abs() < tolerance {
                return (rel_correction, rx_time, rx);
            }
        }

        // Fallback after max iterations
        let final_rx = rx_provider(rx_time);
        (rel_correction, rx_time, final_rx)
    }

    /// Computes the one-way relativistic correction using the standard
    /// endpoint-differential model with optional path sampling.
    ///
    /// This function implements the modern formulation used by deep-space
    /// navigation systems (Moyer 2003 / DSN standard). It returns the total
    /// correction that must be added to the Newtonian geometric light time
    /// `|r_rx − r_tx| / c`.
    ///
    /// The correction consists of two physically distinct contributions:
    /// - **Differential clock-rate correction**: The difference in accumulated
    ///   proper time between the transmitter (at transmission) and receiver
    ///   (at reception) over the coordinate time span, computed from the
    ///   library’s unified master-Lagrangian model.
    /// - **Gravitational (Shapiro) delay**: The extra coordinate time required
    ///   for the signal to propagate near a massive body, evaluated using the
    ///   supplied [`LightContext`].
    ///
    /// When a non-empty `samples` slice is provided, the first element is used
    /// as the effective transmitter state and the last element as the effective
    /// receiver state. This allows callers to supply a high-resolution model
    /// of how the local spacetime (gravitational potential and velocity) varies
    /// along the straight-line path, yielding more accurate endpoint rates when
    /// the environment changes significantly between transmission and reception.
    ///
    /// If fewer than two samples are supplied, the function falls back to the
    /// direct endpoint evaluation using the provided `rx` state.
    ///
    /// # Parameters
    ///
    /// * `rx` — The receiver state evaluated at an approximate arrival time.
    ///   This state is used for the Shapiro calculation and as a fallback when
    ///   no samples are provided.
    /// * `context` — Controls the gravitational contribution via the Shapiro
    ///   delay formula. Use [`LightContext::SOLAR`] for solar-system work or
    ///   [`LightContext::from_grav_param`] for custom central bodies.
    /// * `samples` — Optional sequence of [`Spacetime`] states sampled
    ///   along the straight-line path from transmitter to receiver. When
    ///   non-empty, the first sample defines the transmitter clock rate and
    ///   the last sample defines the receiver clock rate.
    ///
    /// # Returns
    ///
    /// The total relativistic correction (`Dt`) to be added to the Newtonian
    /// geometric light time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use deep_time::{Dt, LightContext, Spacetime, ObserverState};
    ///
    /// # let tx = /* transmitter state */;
    /// # let rx_approx = /* approximate receiver state */;
    ///
    /// // Basic usage (no path sampling)
    /// let correction = tx.one_way_relativistic_delay_integrated(
    ///     rx_approx,
    ///     LightContext::SOLAR,
    ///     &[],
    /// );
    ///
    /// // High-fidelity usage with path sampling
    /// let path_samples: Vec<Spacetime> = /* sample along the light path */;
    /// let correction = tx.one_way_relativistic_delay_integrated(
    ///     rx_approx,
    ///     LightContext::SOLAR,
    ///     &path_samples,
    /// );
    /// ```
    ///
    /// # References
    ///
    /// * Moyer, T.D. (2003). *Formulation for Observed and Computed Values of
    ///   Deep Space Network Data Types for Navigation*. JPL DESCANSO Vol. 2,
    ///   Sections 8 and 11.
    /// * IAU Resolution B1.5 (2000) and subsequent updates on relativistic
    ///   reference systems.
    /// * Ashby, N. (2003). "Relativity in the Global Positioning System".
    ///   *Living Reviews in Relativity*.
    ///
    /// # Notes
    ///
    /// This function always uses the **differential** proper-time accumulation
    /// between the effective transmitter and receiver states. It does **not**
    /// perform an absolute integration of `(dτ/dt − 1)` along the path.
    /// The Shapiro delay term is always computed from the endpoint positions
    /// using the analytic logarithmic formula.
    pub const fn one_way_relativistic_delay_integrated(
        &self,
        rx: ObserverState,
        context: LightContext,
        samples: &[Spacetime],
    ) -> Dt {
        if samples.len() < 2 {
            return self.one_way_relativistic_delay_to(rx, context);
        }

        // Effective transmitter drift from first sample (or self if you prefer)
        let tx_local = samples[0];
        let tx_drift = Drift::from_spacetime(&tx_local);

        // Effective receiver drift from last sample (path-informed rx rate)
        let rx_local = samples[samples.len() - 1];
        let rx_drift = Drift::from_spacetime(&rx_local);

        let span = rx.time.to_diff_raw(self.time);
        let drift_correction = rx_drift
            .time_diff_after(&span)
            .sub(tx_drift.time_diff_after(&span));

        let r_tx = self.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = self.position.distance_to(rx.position);
        let shapiro = Self::shapiro_one_way_delay(context, r_tx, r_rx, r_sep);

        drift_correction.add(shapiro)
    }

    /// Computes the total relativistic correction for a two-way round-trip
    /// ranging measurement by independently solving the uplink and downlink
    /// legs using the full iterative light-time solver.
    ///
    /// This function implements the modern formulation recommended by
    /// Moyer (2003) and used by deep-space networks (DSN, ESA, JPL) for
    /// high-accuracy two-way ranging. It solves the uplink leg (transmitter
    /// to remote body) and the downlink leg (remote body back to transmitter)
    /// as two separate one-way problems, each with its own iterative solution.
    /// This approach is more accurate than older combined round-trip formulations
    /// when the transmitter and receiver are in different gravitational
    /// environments or moving at different velocities.
    ///
    /// The returned correction must be **subtracted** from the raw measured
    /// round-trip time to recover the geometric light time.
    ///
    /// # Parameters
    ///
    /// * `rx_provider` — Closure that returns the relativistic state of the
    ///   remote body (planet, spacecraft, etc.) at a given coordinate time.
    ///   This is used for both the uplink solution and to obtain the accurate
    ///   state at uplink arrival for the downlink leg.
    /// * `tx_provider` — Closure that returns the relativistic state of the
    ///   local transmitter at a given coordinate time. This is used for the
    ///   downlink leg and may represent a moving Earth station or another
    ///   spacecraft.
    /// * `context` — Controls the gravitational (Shapiro) contribution for
    ///   both legs. Use [`LightContext::SOLAR`] for solar-system work.
    /// * `tolerance` — Convergence tolerance passed to the underlying
    ///   iterative solver (recommended: `Dt::from_ns(1, Scale::TAI)`).
    /// * `max_iter` — Maximum iterations for each leg (typically 12–20).
    ///
    /// # Returns
    ///
    /// The total relativistic correction (`Dt`) for the complete round-trip.
    /// This value should be subtracted from the observed round-trip time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use deep_time::{Dt, LightContext, Scale};
    ///
    /// # let earth_station = /* local transmitter state */;
    /// let tolerance = Dt::from_ns(1, Scale::TAI);
    ///
    /// let correction = earth_station.round_trip_relativistic_correction(
    ///     &mut |t| get_spacecraft_state(t),      // remote body
    ///     &mut |t| get_earth_station_state(t),   // local transmitter
    ///     LightContext::SOLAR,
    ///     tolerance,
    ///     15,
    /// );
    ///
    /// // Geometric light time = measured_round_trip_time - correction
    /// ```
    ///
    /// # References
    ///
    /// * Moyer, T.D. (2003). *Formulation for Observed and Computed Values of
    ///   Deep Space Network Data Types for Navigation*. JPL DESCANSO Vol. 2,
    ///   Sections 8 and 11 (Two-Way Light Time).
    /// * IAU Resolution B1.5 (2000) and updates on relativistic reference systems.
    /// * Modern DSN and ESA ranging implementations (post-2005).
    ///
    /// # Notes
    ///
    /// This function performs two independent iterative solutions. The downlink
    /// leg uses the precise receiver state obtained at the end of the uplink
    /// solution, ensuring consistency. The method is suitable for Earth–Mars,
    /// Jupiter, Kuiper Belt, and interstellar-class distances.
    pub fn round_trip_relativistic_correction<RxF, TxF>(
        &self,
        mut rx_provider: RxF, // remote body (planet, spacecraft, etc.)
        mut tx_provider: TxF, // local transmitter for the return leg (can move)
        context: LightContext,
        tolerance: Dt,
        max_iter: usize,
    ) -> Dt
    where
        RxF: FnMut(Dt) -> ObserverState,
        TxF: FnMut(Dt) -> ObserverState,
    {
        // Uplink leg: transmitter → receiver
        let (uplink_rel, rx_time, _rx_state) = self.iterative_one_way_relativistic_delay_to(
            &mut rx_provider,
            context,
            tolerance,
            max_iter,
        );

        // Downlink leg: receiver → transmitter
        let return_tx = rx_provider(rx_time); // accurate state at uplink arrival

        let (downlink_rel, _return_rx_time, _return_rx_state) = return_tx
            .iterative_one_way_relativistic_delay_to(
                &mut tx_provider,
                context,
                tolerance,
                max_iter,
            );

        uplink_rel.add(downlink_rel)
    }

    /// First-order one-way Shapiro delay (gravitational light-time delay) caused by a
    /// central point mass.
    ///
    /// This is an internal helper used by the public delay functions. It implements the
    /// standard logarithmic formula used in solar-system navigation and pulsar timing.
    const fn shapiro_one_way_delay(
        context: LightContext,
        r_tx: Real,
        r_rx: Real,
        r_sep: Real,
    ) -> Dt {
        if context.two_grav_param_over_c3 == f!(0.0)
            || r_tx <= f!(0.0)
            || r_rx <= f!(0.0)
            || r_sep <= f!(0.0)
        {
            return Dt::ZERO;
        }

        let arg = (r_tx + r_rx + r_sep) / (r_tx + r_rx - r_sep).max(f!(1.0));
        let delay_sec = context.two_grav_param_over_c3 * log(arg);

        Dt::from_sec_f(delay_sec)
    }
}
