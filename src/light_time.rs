use crate::{
    C, C_SQUARED, Drift, Dt, Position, Real, Spacetime, TWO_GM_SUN_OVER_C3, Velocity, log,
};

impl Dt {
    /// Shapiro gravitational time scale for the Sun (`2 G M_☉ / c³`).
    ///
    /// This is the recommended value to use when constructing the `bodies`
    /// slice passed to [`ObserverState::one_way_relativistic_delay`],
    /// [`ObserverState::shapiro_delay`], etc.
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
    /// The returned value is intended to be used inside a `bodies` slice when
    /// calling [`ObserverState::one_way_relativistic_delay`] or
    /// [`ObserverState::shapiro_delay`].
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
    /// This method returns a **hybrid relativistic correction** consisting of:
    ///
    /// - Differential clock-rate effects (proper time difference between
    ///   transmitter and receiver).
    /// - Total gravitational Shapiro delay summed from all provided bodies.
    ///
    /// This is the main high-level method for one-way relativistic light time
    /// calculations.
    ///
    /// ### The `bodies` parameter
    ///
    /// The `bodies` parameter is a slice of `(shapiro_coefficient, body_position)`
    /// tuples. The **Shapiro coefficient** is the value `(2GM / c³)` for that body,
    /// expressed as a `Dt`.
    ///
    /// #### How to obtain the Shapiro coefficient (`Dt`)
    ///
    /// - **For the Sun** (most common):
    ///   ```rust,ignore
    ///   Dt::SHAPIRO_SOLAR
    ///   ```
    ///
    /// - **For any other body** (Earth, Moon, planets, asteroids, etc.):
    ///   ```rust,ignore
    ///   Dt::shapiro_from_grav_param(gm)
    ///   ```
    ///   where `gm` is the **standard gravitational parameter** (`GM` or `μ`)
    ///   of the body in **m³ s⁻²** (this is the value you get from ephemerides
    ///   such as JPL DE, SPICE, or IAU constants).
    ///
    /// You can mix multiple bodies:
    /// ```rust,ignore
    /// let bodies = &[
    ///     (Dt::SHAPIRO_SOLAR, sun_position),
    ///     (Dt::shapiro_from_grav_param(earth_gm), earth_position),
    ///     (Dt::shapiro_from_grav_param(moon_gm), moon_position),
    /// ];
    /// ```
    ///
    /// Pass an empty slice (`&[]`) to disable Shapiro delay entirely.
    ///
    /// ### Optional Endpoint States
    ///
    /// You may optionally supply a slice of [`Spacetime`] samples. When two or
    /// more samples are provided, only the first and last elements are used for
    /// the clock-rate correction. This is useful when you have more accurate
    /// local spacetime information at the exact transmission and reception events.
    ///
    /// If fewer than two samples are provided, the method falls back to using
    /// `self` and `rx` directly.
    ///
    /// **Note**: This is an endpoint-based model. It does **not** integrate
    /// proper time along the signal path.
    ///
    /// ### Custom Shapiro Delay
    ///
    /// You can override the internally computed Shapiro delay by passing a value
    /// via `custom_shapiro_delay`. This is useful when you want to use a different
    /// Shapiro model, include solar plasma, or inject an externally computed delay.
    ///
    /// # Parameters
    ///
    /// * `rx` — Receiver state at the approximate arrival time.
    /// * `bodies` — Slice of `(shapiro_coefficient, body_position)` pairs. See
    ///   above for how to construct the coefficients.
    /// * `samples` — Optional [`Spacetime`] samples (only first and last used).
    /// * `custom_shapiro_delay` — Optional override for the Shapiro delay.
    ///
    /// # Returns
    ///
    /// The total relativistic correction (clock-rate + Shapiro) to be added to
    /// the geometric light time.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Basic usage — Sun only
    /// let correction = tx.one_way_relativistic_delay(
    ///     rx_approx,
    ///     &[(Dt::SHAPIRO_SOLAR, sun_position)],
    ///     &[],
    ///     None,
    /// );
    ///
    /// // Multi-body + custom endpoint states
    /// let bodies = &[
    ///     (Dt::SHAPIRO_SOLAR, sun_pos),
    ///     (Dt::shapiro_from_grav_param(earth_gm), earth_pos),
    /// ];
    /// let path_samples = vec![tx_spacetime, rx_spacetime];
    ///
    /// let correction = tx.one_way_relativistic_delay(
    ///     rx_approx,
    ///     bodies,
    ///     &path_samples,
    ///     None,
    /// );
    /// ```
    pub const fn one_way_relativistic_delay(
        &self,
        rx: ObserverState,
        bodies: &[(Dt, Position)],
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
            self.compute_differential_clock_correction(rx)
        };

        // Determine Shapiro delay
        let shapiro_delay = match custom_shapiro_delay {
            Some(custom) => custom,
            None => self.shapiro_delay(rx, bodies),
        };

        drift_correction.add(shapiro_delay)
    }

    /// Iteratively solves the one-way light-time equation including relativistic
    /// corrections until the receive time converges to the requested tolerance.
    ///
    /// This is the recommended high-precision solver for one-way light-time
    /// computations. It follows the formulation described in Moyer (2003) and
    /// works with any ephemeris source (SPICE kernels, numerical integrators,
    /// or analytical propagators).
    ///
    /// The solver performs fixed-point iteration on the light-time equation:
    ///
    /// ```text
    /// t_rx = t_tx + |r_rx(t_rx) − r_tx(t_tx)| / c + Δt_rel(t_tx, t_rx)
    /// ```
    ///
    /// where `Δt_rel` is the relativistic correction returned by
    /// [`one_way_relativistic_delay`]. Iteration continues until the change
    /// in the estimated receive time falls below `tolerance`.
    ///
    /// # Parameters
    ///
    /// * `rx_provider` — A mutable closure that returns the full relativistic
    ///   state of the receiver at a given coordinate time. This allows the solver
    ///   to work with any ephemeris or propagator without requiring a specific
    ///   data structure.
    /// * `bodies` — Slice of `(shapiro_coefficient, body_position)` pairs that
    ///   control the gravitational (Shapiro) contribution.
    ///   - Use `&[(Dt::SHAPIRO_SOLAR, sun_position)]` for normal solar-system work.
    ///   - Include additional bodies (planets, Moon, etc.) when higher precision
    ///     is required.
    ///   - Pass an empty slice (`&[]`) to disable the Shapiro contribution.
    /// * `tolerance` — Convergence tolerance on the change in receive time.
    ///   A typical value for high-precision work is `Dt::from_ns(1, Scale::TAI)`.
    /// * `max_iter` — Maximum number of iterations before falling back.
    ///   A value of 12–20 is usually sufficient for solar-system geometries.
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
    /// ```ignore
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
    ///     &[(Dt::SHAPIRO_SOLAR, sun_position)],
    ///     tolerance,
    ///     20,
    /// );
    ///
    /// // The total light time is:
    /// let total_light_time = rx_time.to_diff_raw(tx.time);
    /// ```
    pub fn iterative_one_way_relativistic_delay_to<F>(
        &self,
        rx_provider: &mut F,
        bodies: &[(Dt, Position)],
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

            rel_correction = self.one_way_relativistic_delay(rx, bodies, &[], None);

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
    /// * `bodies` — Slice of `(shapiro_coefficient, body_position)` pairs.
    ///   The same bodies list is used for both uplink and downlink legs.
    ///   Use `&[(Dt::SHAPIRO_SOLAR, sun_position)]` for normal solar-system work.
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
    /// ```ignore
    /// use deep_time::{Dt, Scale};
    ///
    /// # let earth_station = /* local transmitter state */;
    /// let tolerance = Dt::from_ns(1, Scale::TAI);
    ///
    /// let correction = earth_station.round_trip_relativistic_correction(
    ///     &mut |t| get_spacecraft_state(t),      // remote body
    ///     &mut |t| get_earth_station_state(t),   // local transmitter
    ///     &[(Dt::SHAPIRO_SOLAR, sun_position)],
    ///     tolerance,
    ///     15,
    /// );
    ///
    /// // Geometric light time = measured_round_trip_time - correction
    /// ```
    pub fn round_trip_relativistic_correction<RxF, TxF>(
        &self,
        mut rx_provider: RxF, // remote body (planet, spacecraft, etc.)
        mut tx_provider: TxF, // local transmitter for the return leg (can move)
        bodies: &[(Dt, Position)],
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
            bodies,
            tolerance,
            max_iter,
        );

        // Downlink leg: receiver → transmitter
        let return_tx = rx_provider(rx_time); // accurate state at uplink arrival

        let (downlink_rel, _return_rx_time, _return_rx_state) = return_tx
            .iterative_one_way_relativistic_delay_to(&mut tx_provider, bodies, tolerance, max_iter);

        uplink_rel.add(downlink_rel)
    }

    /// Computes the total gravitational (Shapiro) delay for a one-way signal,
    /// summed across all provided gravitating bodies.
    ///
    /// This is the recommended method for obtaining Shapiro delay.
    ///
    /// It uses the modern numerically stable formulation for each body and returns
    /// the sum of all contributions.
    ///
    /// ### When to use this method
    ///
    /// - When you only need the gravitational propagation delay (Shapiro term).
    /// - For multi-body calculations (Sun + planets + Moon, etc.).
    ///
    /// ### Parameters
    ///
    /// - `rx`: Receiver state at the approximate time of signal arrival.
    /// - `bodies`: Slice of `(shapiro_coefficient, body_position)` pairs.
    ///     - Use [`Dt::SHAPIRO_SOLAR`](../struct.Dt.html#associatedconstant.SHAPIRO_SOLAR) +
    ///       the Sun’s position for normal solar-system work.
    ///     - Add additional bodies (planets, Moon, etc.) when higher precision is needed.
    ///     - Pass an empty slice (`&[]`) to disable the Shapiro contribution entirely.
    ///
    /// The positions must be in the **same reference frame** as `self.position` and `rx.position`.
    ///
    /// ### Notes
    ///
    /// - This method computes **only** the Shapiro gravitational delay.
    /// - It does **not** include differential clock-rate corrections.
    /// - If you need both the Shapiro delay **and** clock-rate effects, use
    ///   [`Self::one_way_relativistic_delay`] instead.
    /// - Internally uses the stable algebraic formulation (equivalent to the classic
    ///   Moyer/DSN form but numerically robust near conjunctions).
    ///
    /// ### Example
    ///
    /// ```ignore
    /// // Sun only (common case)
    /// let shapiro = tx.shapiro_delay(rx_approx, &[(Dt::SHAPIRO_SOLAR, sun_position)]);
    ///
    /// // Multi-body example
    /// let bodies = &[
    ///     (Dt::SHAPIRO_SOLAR, sun_pos),
    ///     (earth_shapiro, earth_pos),
    ///     (moon_shapiro, moon_pos),
    /// ];
    /// let total_shapiro = tx.shapiro_delay(rx_approx, bodies);
    /// ```
    pub const fn shapiro_delay(&self, rx: ObserverState, bodies: &[(Dt, Position)]) -> Dt {
        let mut total = Dt::ZERO;
        let mut i = 0;

        while i < bodies.len() {
            let (shapiro_coeff, body_pos) = bodies[i];
            total = total.add(Self::shapiro_one_way_delay(
                shapiro_coeff,
                self.position,
                rx.position,
                body_pos,
            ));
            i += 1;
        }

        total
    }

    /// Computes the first-order one-way Shapiro gravitational time delay
    /// due to a single central body using a numerically stable formulation.
    ///
    /// This is the **core low-level implementation** (pub(crate) const fn).
    /// It replaces the classic radial formula with an algebraically equivalent
    /// but cancellation-free form that is robust even for small impact parameters
    /// (near-grazing / conjunction geometries).
    ///
    /// The algorithm uses the identity:
    ///
    /// ```ignore
    ///   ln((r_tx + r_rx + r_sep) / (r_tx + r_rx - r_sep))
    ///   ≡ 2·ln(num) − ln(denom_term)
    /// ```
    ///
    /// where denom_term is computed from the dot-product identity
    /// (r_tx + r_rx)² − r_sep² = 2(r_tx·r_rx + p_tx · p_rx).
    /// This avoids the dangerous subtraction that loses precision when
    /// the signal path passes close to the body.
    ///
    /// The result is **exactly equivalent** (within floating-point) to the
    /// classic Moyer/DSN-style formula while being far more stable.
    /// Contributions from multiple bodies are summed at a higher level.
    ///
    /// # Safety / Guards
    ///
    /// - Returns [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)
    ///   for any non-positive distance or zero Shapiro coefficient.
    /// - Protects against invalid logarithm argument (`arg <= 1.0`).
    /// - Designed for weak-field solar-system / cislunar use (monopole, straight-line approx).
    pub(crate) const fn shapiro_one_way_delay(
        shapiro: Dt,
        tx_pos: Position,
        rx_pos: Position,
        body_pos: Position,
    ) -> Dt {
        let shapiro_sec = shapiro.to_sec_f();

        // Distances relative to *this specific gravitating body*
        let r_tx = tx_pos.distance_to(body_pos);
        let r_rx = rx_pos.distance_to(body_pos);
        let r_sep = tx_pos.distance_to(rx_pos);

        if r_tx <= f!(0.0) || r_rx <= f!(0.0) || r_sep <= f!(0.0) || shapiro_sec == f!(0.0) {
            return Dt::ZERO;
        }

        let s = r_tx + r_rx;
        let num = s + r_sep; // (r_tx + r_rx + r_sep)

        if num <= f!(0.0) {
            return Dt::ZERO;
        }

        // Stable computation of (r_tx + r_rx)^2 − r_sep^2
        // = 2 × (r_tx r_rx + \vec{p_tx} · \vec{p_rx})
        let dot_term = (r_tx * r_tx + r_rx * r_rx - r_sep * r_sep) / f!(2.0);
        let denom_term = f!(2.0) * (r_tx * r_rx + dot_term);

        if denom_term <= f!(0.0) {
            return Dt::ZERO;
        }

        let arg = (num * num) / denom_term;

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
