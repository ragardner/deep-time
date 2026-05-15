use crate::{
    C, C_SQUARED, Drift, Dt, Position, Real, Spacetime, TWO_GM_SUN_OVER_C3, Velocity, log,
};

impl Dt {
    /// Shapiro gravitational time scale for the Sun (`2 G M_☉ / c³`).
    ///
    /// This is the recommended value to pass as the `shapiro` parameter to
    /// [`ObserverState::one_way_relativistic_delay`] (and [`ObserverState::shapiro_delay_to`])
    /// for all normal solar-system work.
    ///
    /// It corresponds to the one-way Shapiro coefficient for the Sun.
    pub const SHAPIRO_SOLAR: Self = Self::from_sec_f(TWO_GM_SUN_OVER_C3);

    /// Creates the Shapiro delay scale for an arbitrary central body
    /// from its standard gravitational parameter `GM` (μ) in m³ s⁻².
    ///
    /// This produces the coefficient used in the Shapiro gravitational time delay
    /// formula. It is the recommended way to create a custom Shapiro scale for
    /// planets, stars, or other massive bodies.
    ///
    /// Pass the resulting value to [`ObserverState::one_way_relativistic_delay`]
    /// or [`ObserverState::shapiro_delay_to`].
    #[inline]
    pub const fn shapiro_from_grav_param(gm: Real) -> Self {
        let secs = 2.0 * gm / (C * C_SQUARED);
        Self::from_sec_f(secs)
    }

    /// Creates an [`ObserverState`] using this time value along with the
    /// provided position, velocity, and gravitational information.
    ///
    /// An `ObserverState` represents a complete snapshot of an observer
    /// (spacecraft, ground station, planet, etc.) at a specific moment.
    /// It bundles together the time, position, velocity, and local
    /// gravitational environment so that relativistic calculations
    /// (light time, clock rates, Shapiro delay, etc.) can be performed.
    ///
    /// This method is a convenience constructor. It is useful when you
    /// already have a [`Dt`] (a time value) and want to build an
    /// `ObserverState` directly from it, rather than calling
    /// [`ObserverState::new`] or [`ObserverState::new_strong_field`].
    ///
    /// # Parameters
    ///
    /// - `position`: The observer’s position in meters (typically expressed
    ///   in a barycentric or heliocentric frame).
    /// - `velocity`: The observer’s velocity in meters per second.
    /// - `grav_potential_m2_s2`: The total Newtonian gravitational potential
    ///   (Φ) at the observer’s location, in m²/s². This is usually negative
    ///   for bound orbits and is the sum of contributions from the Sun and
    ///   planets.
    /// - `characteristic_length_scale`: A length scale (in meters) over which
    ///   gravity varies significantly at this location. Use `0.0` for normal
    ///   solar-system and weak-field cases. Only provide a non-zero value when
    ///   working in strong gravitational fields.
    ///
    /// # When to use this method
    ///
    /// Use this method when you already have a time value as a [`Dt`] and
    /// want to construct an `ObserverState` in one step. It is especially
    /// convenient when working with time values that were previously
    /// computed or converted.
    ///
    /// For most normal use, [`ObserverState::new`] is simpler. Use
    /// [`ObserverState::new_strong_field`] instead if you need to specify
    /// a non-zero `characteristic_length_scale`.
    ///
    /// # Example
    /// ```ignore
    /// let t = Dt::from_sec(1234.5);
    ///
    /// let state = t.to_observer_state(
    ///     position,
    ///     velocity,
    ///     grav_potential,
    ///     0.0, // normal solar-system use
    /// );
    /// ```
    #[inline]
    pub const fn to_observer_state(
        self,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: Real,
        characteristic_length_scale: Real,
    ) -> ObserverState {
        ObserverState {
            time: self,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale,
        }
    }
}

/// A snapshot of an observer’s relativistic state at a specific instant.
///
/// `ObserverState` combines time, position, velocity, and local gravitational
/// information. It is the main input type used by relativistic light-time
/// methods in this library.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct ObserverState {
    /// The time of this state.
    ///
    /// Any [`Scale`] is accepted. This time is treated as coordinate time
    /// for light-time calculations.
    pub time: Dt,

    /// Position of the observer in meters.
    ///
    /// Typically expressed in a barycentric (solar-system barycenter) or
    /// heliocentric frame, depending on the application.
    pub position: Position,

    /// Velocity of the observer in meters per second.
    pub velocity: Velocity,

    /// Newtonian gravitational potential Φ at the observer’s location
    /// (in m² s⁻²).
    ///
    /// This value is usually negative for bound orbits. It should normally
    /// include contributions from the Sun and all relevant planets.
    pub grav_potential_m2_s2: Real,

    /// Characteristic length scale (in meters) over which the gravitational
    /// field varies significantly at this location.
    ///
    /// - Use `0.0` (the default) for all solar-system, GNSS, and weak-field
    ///   applications.
    /// - Provide a non-zero value only when working in strong gravitational
    ///   fields (e.g. near neutron stars or black holes), where the library’s
    ///   higher-order curvature terms become relevant.
    pub characteristic_length_scale: Real,
}

impl ObserverState {
    /// Creates a new `ObserverState` for typical solar-system, GNSS,
    /// or weak-field use.
    ///
    /// This is the recommended constructor for most applications.
    /// It sets the `characteristic_length_scale` to `0.0`, which disables
    /// higher-order curvature terms in the proper-time model.
    ///
    /// # Parameters
    /// - `time`: The time of the state.
    /// - `position`: Position in meters (usually barycentric or heliocentric).
    /// - `velocity`: Velocity in m/s.
    /// - `grav_potential_m2_s2`: Newtonian gravitational potential Φ
    ///   at the location (in m²/s²).
    #[inline]
    pub const fn new(
        time: Dt,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: Real,
    ) -> Self {
        Self {
            time,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale: 0.0,
        }
    }

    /// Creates a new `ObserverState` when strong-field effects or a
    /// non-zero characteristic length scale are relevant.
    ///
    /// Use this constructor when you have gravimeter data or are working
    /// in regions where spacetime curvature varies significantly over
    /// short distances (e.g. near compact objects). The
    /// `characteristic_length_scale` parameter controls whether the
    /// library activates higher-order terms in the proper-time calculation.
    ///
    /// For normal solar-system work, use [`Self::new`] instead.
    ///
    /// # Parameters
    /// - `time`: The time of the state.
    /// - `position`: Position in meters.
    /// - `velocity`: Velocity in m/s.
    /// - `grav_potential_m2_s2`: Newtonian gravitational potential Φ
    ///   at the location (in m²/s²).
    /// - `characteristic_length_scale`: Length scale (in meters) over which
    ///   gravity varies at this location. Must be positive to have an effect.
    #[inline]
    pub const fn new_strong_field(
        time: Dt,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: Real,
        characteristic_length_scale: Real,
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
    /// This value indicates how fast a physical clock located at this state
    /// would advance relative to the time used by this `ObserverState`.
    /// A returned value of `1.0` means the clock advances at the same rate
    /// as the state's time coordinate. Values are typically slightly different
    /// from `1.0` due to the effects of velocity and gravitational potential.
    ///
    /// This rate is computed using the library’s unified proper-time model.
    /// It is used internally for light-time corrections and Doppler calculations.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        Spacetime::from_potential_velocity_and_scale(
            self.grav_potential_m2_s2 / C_SQUARED,
            self.velocity,
            self.characteristic_length_scale,
        )
        .proper_time_rate()
    }

    /// Returns the ratio of proper time rates between the receiver and transmitter
    /// for a one-way signal.
    ///
    /// This method computes:
    ///
    /// ```text
    /// ratio = rx.proper_time_rate() / self.proper_time_rate()
    /// ```
    ///
    /// ### Interpretation
    ///
    /// - A value of `1.0` indicates that both clocks run at the same rate.
    /// - A value **less than `1.0`** means the receiver’s clock runs slower than
    ///   the transmitter’s clock. The receiver will observe a lower frequency
    ///   than was emitted.
    /// - A value **greater than `1.0`** means the receiver’s clock runs faster
    ///   than the transmitter’s clock. The receiver will observe a higher frequency
    ///   than was emitted.
    ///
    /// The ratio captures the combined effect of special-relativistic time dilation
    /// (due to velocity) and general-relativistic gravitational time dilation.
    ///
    /// ### Typical Usage (One-Way)
    ///
    /// This ratio is often combined with the classical kinematic Doppler term
    /// to estimate the total one-way frequency shift:
    ///
    /// ```text
    /// approximate_frequency_shift ≈ ratio * (1 - v_radial / C)
    /// ```
    ///
    /// where `v_radial` is the radial velocity (positive when the receiver is
    /// receding).
    ///
    /// ### Two-Way Usage
    ///
    /// For round-trip (two-way) measurements, square the one-way ratio:
    ///
    /// ```rust,ignore
    /// let one_way_ratio = transmitter.relativistic_clock_rate_ratio(receiver);
    /// let two_way_ratio = one_way_ratio * one_way_ratio;
    /// ```
    ///
    /// This pattern is commonly used when correcting two-way Doppler (range-rate)
    /// data for relativistic clock effects.
    ///
    /// ### Limitations
    ///
    /// - This method only accounts for the **difference in clock rates** between
    ///   the two ends.
    /// - It does **not** include Shapiro delay or higher-order relativistic effects
    ///   on signal propagation.
    /// - The combination with classical Doppler shown above is a first-order
    ///   approximation.
    ///
    /// # Parameters
    /// - `self` — Transmitter state at the time of transmission.
    /// - `rx`   — Receiver state at the approximate time of reception.
    ///
    /// # Example
    /// ```rust,ignore
    /// let ratio = transmitter.relativistic_clock_rate_ratio(receiver);
    ///
    /// let v_radial = ...; // m/s, positive if receding
    /// let classical_doppler = 1.0 - v_radial / C;
    ///
    /// let approx_frequency_shift = ratio * classical_doppler;
    /// ```
    #[inline]
    pub const fn relativistic_clock_rate_ratio(&self, rx: ObserverState) -> Real {
        rx.proper_time_rate() / self.proper_time_rate()
    }

    /// Computes the additional delay that must be added to the Newtonian
    /// geometric light time `|r_rx − r_tx| / c`.
    ///
    /// In simple terms, this method calculates the extra time delay caused by
    /// differences in clock rates between the transmitter and receiver (due to
    /// their relativistic states, including velocity and gravity) plus the
    /// gravitational Shapiro delay.
    ///
    /// The returned value is a **hybrid correction** consisting of two parts:
    ///
    /// - The difference in accumulated proper time between the transmitter and
    ///   receiver (clock-rate effects).
    /// - The gravitational (Shapiro) delay caused by spacetime curvature near
    ///   a central mass.
    ///
    /// This is the primary method for relativistic light-time calculations.
    ///
    /// ### Optional Endpoint States
    ///
    /// You may optionally supply a slice of [`Spacetime`] samples. When two or
    /// more samples are provided, only the first and last elements are used:
    /// the first becomes the effective transmitter state, and the last becomes
    /// the effective receiver state for the clock-rate correction.
    ///
    /// This is useful when you have more accurate information about the local
    /// spacetime conditions exactly at transmission and reception.
    ///
    /// If fewer than two samples are provided, the method falls back to using
    /// `self` and `rx` directly.
    ///
    /// **Note**: This is an endpoint-based model. It does **not** perform
    /// numerical integration of proper time along the signal path.
    ///
    /// ### Custom Shapiro / Propagation Delay
    ///
    /// You can provide your own delay value via `custom_shapiro_delay`. When
    /// supplied, this value is used directly instead of the internally computed
    /// Shapiro delay.
    ///
    /// This allows you to pass:
    /// - A delay computed with a different Shapiro model
    /// - A delay that includes additional propagation effects (such as solar plasma)
    /// - A delay obtained from an external calculation
    ///
    /// # Parameters
    ///
    /// * `rx` — Receiver state at the approximate arrival time.
    /// * `shapiro` — Shapiro scale factor. Use [`Dt::SHAPIRO_SOLAR`] for normal
    ///   solar-system work.
    /// * `samples` — Optional [`Spacetime`] samples. Only the first and last
    ///   are used when provided.
    /// * `custom_shapiro_delay` — Optional custom delay to use instead of the
    ///   standard Shapiro calculation.
    ///
    /// # Returns
    ///
    /// The total relativistic correction to be added to the geometric light time.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Basic usage
    /// let correction = tx.one_way_relativistic_delay(
    ///     rx_approx,
    ///     Dt::SHAPIRO_SOLAR,
    ///     &[],
    ///     None,
    /// );
    ///
    /// // With custom endpoint states and a custom delay
    /// let path_samples = vec![tx_spacetime, rx_spacetime];
    /// let custom_delay = compute_custom_delay(...);
    ///
    /// let correction = tx.one_way_relativistic_delay(
    ///     rx_approx,
    ///     Dt::SHAPIRO_SOLAR,
    ///     &path_samples,
    ///     Some(custom_delay),
    /// );
    /// ```
    pub const fn one_way_relativistic_delay(
        &self,
        rx: ObserverState,
        shapiro: Dt,
        samples: &[Spacetime],
        custom_shapiro_delay: Option<Dt>,
    ) -> Dt {
        // Compute the differential clock-rate correction
        let drift_correction = if samples.len() >= 2 {
            let tx_local = samples[0];
            let tx_drift = Drift::from_spacetime(&tx_local);

            let rx_local = samples[samples.len() - 1];
            let rx_drift = Drift::from_spacetime(&rx_local);

            let span = rx.time.to_diff_raw(self.time);

            rx_drift
                .time_diff_after(&span)
                .sub(tx_drift.time_diff_after(&span))
        } else {
            // Fallback to using self and rx directly
            self.compute_differential_clock_correction(rx)
        };

        // Determine which Shapiro value to use
        let shapiro_delay = match custom_shapiro_delay {
            Some(custom) => custom,
            None => self.shapiro_delay_to(rx, shapiro),
        };

        drift_correction.add(shapiro_delay)
    }

    /// Iteratively solves the one-way light-time equation including relativistic
    /// corrections until the receive time converges to the requested tolerance.
    ///
    /// This is the recommended high-precision solver for one-way light-time
    /// computations in modern deep-space navigation. It follows the formulation
    /// described in Moyer (2003) and works with any ephemeris source (SPICE
    /// kernels, numerical integrators, or analytical propagators).
    ///
    /// The solver performs a fixed-point iteration on the light-time equation:
    ///
    /// ```text
    /// t_rx = t_tx + |r_rx(t_rx) − r_tx(t_tx)| / c + Δt_rel(t_tx, t_rx)
    /// ```
    ///
    /// where `Δt_rel` is the relativistic correction returned by
    /// [`one_way_relativistic_delay`]. The iteration continues until the change
    /// in the estimated receive time falls below `tolerance`.
    ///
    /// # Parameters
    ///
    /// * `rx_provider` — A mutable closure that returns the full relativistic
    ///   state of the receiver at a given coordinate time. This allows the solver
    ///   to work with any ephemeris or propagator without requiring a specific
    ///   data structure.
    /// * `shapiro` — Controls the gravitational (Shapiro) contribution. Use
    ///   [`Dt::SHAPIRO_SOLAR`] for solar-system work or
    ///   [`Dt::shapiro_from_grav_param`] for other central bodies.
    /// * `tolerance` — Convergence tolerance on the change in receive time.
    ///   A typical value for high-precision work is `Dt::from_ns(1, Scale::TAI)`.
    /// * `max_iter` — Maximum number of iterations before falling back. A value
    ///   of 12–20 is usually sufficient for solar-system geometries.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `rel_correction` — The final relativistic correction (clock-rate
    ///   correction + Shapiro) evaluated at convergence.
    /// * `rx_time` — The converged receive time in the coordinate scale of
    ///   the transmitter.
    /// * `final_state` — The receiver state at the converged receive time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use deep_time::{Dt, ObserverState, Scale};
    ///
    /// # let tx = /* transmitter state */;
    /// let tolerance = Dt::from_ns(1, Scale::TAI);
    ///
    /// let (correction, rx_time, rx_state) = tx.iterative_one_way_relativistic_delay_to(
    ///     &mut |t| {
    ///         // Example: call into SPICE or your own propagator
    ///         get_receiver_state_at(t)
    ///     },
    ///     Dt::SHAPIRO_SOLAR,
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
    /// The solver uses simple fixed-point iteration. For most solar-system
    /// geometries, convergence occurs within 3–6 iterations. The function always
    /// returns a result. If `max_iter` is reached, the last computed values are
    /// returned. The returned `ObserverState` is guaranteed to be consistent with
    /// the final `rx_time`.
    pub fn iterative_one_way_relativistic_delay_to<F>(
        &self,
        rx_provider: &mut F,
        shapiro: Dt,
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

            rel_correction = self.one_way_relativistic_delay(rx, shapiro, &[], None);

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

    /// Computes the total relativistic correction for a two-way round-trip
    /// ranging measurement.
    ///
    /// This method solves the uplink and downlink legs **independently** using
    /// the iterative light-time solver. This is the modern approach recommended
    /// by Moyer (2003) and used by deep-space networks (DSN, ESA, JPL).
    ///
    /// Solving the legs separately is more accurate than older combined
    /// round-trip formulas when the two ends are in different gravitational
    /// environments or have significantly different velocities.
    ///
    /// The returned value must be **subtracted** from the raw measured
    /// round-trip time to recover the geometric light time.
    ///
    /// # Parameters
    ///
    /// * `rx_provider` — Closure that returns the relativistic state of the
    ///   remote body (planet, spacecraft, etc.) at a given coordinate time.
    ///   Used for both the uplink solution and to obtain the accurate state
    ///   at uplink arrival for the downlink leg.
    /// * `tx_provider` — Closure that returns the relativistic state of the
    ///   local transmitter at a given coordinate time (e.g. a moving ground
    ///   station or another spacecraft). Used for the downlink leg.
    /// * `shapiro` — Shapiro scale factor applied to both legs. Use
    ///   [`Dt::SHAPIRO_SOLAR`] for normal solar-system work.
    /// * `tolerance` — Convergence tolerance for the underlying iterative
    ///   solver (recommended: `Dt::from_ns(1, Scale::TAI)`).
    /// * `max_iter` — Maximum iterations per leg (typically 12–20).
    ///
    /// # Returns
    ///
    /// The total relativistic correction for the complete round trip.
    /// This value should be subtracted from the observed round-trip time.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use deep_time::{Dt, Scale};
    ///
    /// # let earth_station = /* local transmitter state */;
    /// let tolerance = Dt::from_ns(1, Scale::TAI);
    ///
    /// let correction = earth_station.round_trip_relativistic_correction(
    ///     &mut |t| get_spacecraft_state(t),      // remote body
    ///     &mut |t| get_earth_station_state(t),   // local transmitter
    ///     Dt::SHAPIRO_SOLAR,
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
    /// * Modern DSN and ESA ranging implementations.
    ///
    /// # Notes
    ///
    /// This method performs two independent iterative solutions. The downlink
    /// leg uses the precise receiver state obtained at the end of the uplink
    /// solution, ensuring consistency between the two legs.
    ///
    /// The method is suitable for Earth–Mars, Jupiter, Kuiper Belt, and
    /// interstellar-class distances.
    pub fn round_trip_relativistic_correction<RxF, TxF>(
        &self,
        mut rx_provider: RxF, // remote body (planet, spacecraft, etc.)
        mut tx_provider: TxF, // local transmitter for the return leg (can move)
        shapiro: Dt,
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
            shapiro,
            tolerance,
            max_iter,
        );

        // Downlink leg: receiver → transmitter
        let return_tx = rx_provider(rx_time); // accurate state at uplink arrival

        let (downlink_rel, _return_rx_time, _return_rx_state) = return_tx
            .iterative_one_way_relativistic_delay_to(
                &mut tx_provider,
                shapiro,
                tolerance,
                max_iter,
            );

        uplink_rel.add(downlink_rel)
    }

    /// Computes **only** the gravitational (Shapiro) delay for a one-way signal.
    ///
    /// This method returns the classical coordinate-time Shapiro delay caused by
    /// spacetime curvature near a central mass. It deliberately excludes any
    /// differential proper-time (clock-rate) correction.
    ///
    /// Use this method when you need only the traditional relativistic propagation
    /// delay used in most deep-space navigation, ranging, and orbit determination
    /// systems (consistent with Moyer 2003 DSN formulations).
    ///
    /// If you need both the Shapiro delay **and** the differential clock-rate
    /// correction, use [`Self::one_way_relativistic_delay`] instead.
    ///
    /// # Parameters
    /// - `rx` — Receiver state at the approximate arrival time.
    /// - `shapiro` — Shapiro scale factor. Use [`Dt::SHAPIRO_SOLAR`] for normal
    ///   solar-system work or [`Dt::shapiro_from_grav_param`] for other central
    ///   bodies. Pass `Dt::ZERO` to disable the Shapiro contribution.
    ///
    /// # Returns
    /// The Shapiro delay (`Dt`) to be added to the Newtonian geometric light time.
    ///
    /// # Example
    /// ```rust,ignore
    /// let shapiro_correction = transmitter.shapiro_delay_to(receiver_approx, Dt::SHAPIRO_SOLAR);
    /// ```
    #[inline]
    pub const fn shapiro_delay_to(&self, rx: ObserverState, shapiro: Dt) -> Dt {
        let r_tx = self.position.norm();
        let r_rx = rx.position.norm();
        let r_sep = self.position.distance_to(rx.position);
        Self::shapiro_one_way_delay(shapiro, r_tx, r_rx, r_sep)
    }

    /// Computes the first-order one-way Shapiro delay caused by a central point mass.
    ///
    /// This is an internal helper used by `shapiro_delay_to` and
    /// `one_way_relativistic_delay`.
    ///
    /// The implementation uses the standard analytic approximation:
    ///
    /// ```text
    /// Δt_Shapiro = (2GM/c³) × ln((r_tx + r_rx + r_sep) / (r_tx + r_rx - r_sep))
    /// ```
    ///
    /// # Numerical Notes
    ///
    /// - Returns `Dt::ZERO` if any of `r_tx`, `r_rx`, `r_sep`, or `shapiro` are
    ///   zero or negative.
    /// - Uses a small epsilon (`1e-6` m) as a safety floor on the denominator to
    ///   avoid division by zero or taking the logarithm of an invalid value.
    /// - When the signal path is nearly collinear with the central body (very
    ///   small impact parameter), the result can become large. This function does
    ///   **not** model the finite size of the Sun (or other body) nor higher-order
    ///   relativistic effects.
    /// - For high-precision work during deep solar conjunctions, consider using
    ///   a numerically integrated or impact-parameter-based Shapiro model instead.
    pub(crate) const fn shapiro_one_way_delay(
        shapiro: Dt,
        r_tx: Real,
        r_rx: Real,
        r_sep: Real,
    ) -> Dt {
        let shapiro_sec = shapiro.to_sec_f();

        if r_tx <= f!(0.0) || r_rx <= f!(0.0) || r_sep <= f!(0.0) || shapiro_sec == f!(0.0) {
            return Dt::ZERO;
        }

        // Safety floor to avoid division by zero or log of invalid argument
        // in near-grazing geometries.
        const EPS: Real = f!(1e-6);

        let denom = (r_tx + r_rx - r_sep).max(EPS);
        let arg = (r_tx + r_rx + r_sep) / denom;

        // Additional guard against invalid logarithm argument
        if arg <= f!(1.0) {
            return Dt::ZERO;
        }

        let delay_sec = shapiro_sec * log(arg);
        Dt::from_sec_f(delay_sec)
    }

    /// Computes only the differential clock-rate correction between `self`
    /// (transmitter) and `rx` (receiver). Does **not** include any Shapiro delay.
    ///
    /// Internal helper.
    pub const fn compute_differential_clock_correction(&self, rx: ObserverState) -> Dt {
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

        rx_drift
            .time_diff_after(&span)
            .sub(tx_drift.time_diff_after(&span))
    }
}
