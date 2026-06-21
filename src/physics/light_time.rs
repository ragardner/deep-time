use crate::{
    C, C_SQUARED, Drift, Dt, Position, Real, Scale, Spacetime, TWO_GM_SUN_OVER_C3, Velocity, log,
};

impl Dt {
    /// Shapiro gravitational time scale for the Sun (`2 G M_☉ / c³`).
    ///
    /// Recommended value for the Sun when building the `bodies` slice passed to
    /// [`Observer::shapiro_delay`], [`Observer::shapiro_delay`],
    /// and related methods.
    pub const SHAPIRO_SOLAR: Self = Self::from_sec_f(TWO_GM_SUN_OVER_C3, Scale::TAI);

    /// Creates the Shapiro delay scale for an arbitrary central body
    /// from its standard gravitational parameter `GM` (μ) in m³ s⁻².
    ///
    /// This produces the coefficient used in the Shapiro gravitational time delay
    /// formula. It is the recommended way to create a custom Shapiro scale for
    /// planets, stars, or other massive bodies.
    ///
    /// The returned value is intended to be used for the `bodies` parameter
    /// when calling [`Observer::shapiro_delay`] or
    /// [`Observer::shapiro_delay`].
    #[inline]
    pub const fn shapiro_from_grav_param(gm: Real) -> Dt {
        let secs = 2.0 * gm / (C * C_SQUARED);
        Self::from_sec_f(secs, Scale::TAI)
    }

    /// Creates an [`Observer`] using this time value along with the
    /// provided position, velocity, and gravitational information.
    ///
    /// An [`Observer`] represents a complete snapshot of an observer
    /// (spacecraft, ground station, planet, person, etc.) at a
    /// specific moment.
    ///
    /// It bundles together the time, position, velocity, and local
    /// gravitational environment so that relativistic calculations
    /// (light time, clock rates, Shapiro delay, etc.) can be performed.
    ///
    /// This method is a convenience constructor. It is useful when you
    /// already have a [`Dt`] (a time value) and want to build an
    /// [`Observer`] directly from it.
    ///
    /// ## Parameters
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
    /// ## Examples
    ///
    /// ```
    /// use deep_time::{Dt, Position, Spacetime, Velocity};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20),     // Sun
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14),     // Earth
    ///     (Position::from_au(1.00257, 0.0, 0.0), 4.9048695e12), // Moon
    /// ];
    ///
    /// let position = Position::from_au(1.001, 0.001, 0.0); // e.g. spacecraft, asteroid, etc.
    ///
    /// let grav_potential = Spacetime::grav_potential_from_point_masses(
    ///     position,
    ///     bodies.iter().copied(),
    /// );
    ///
    /// let t = Dt::span_f(1234.5);
    ///
    /// let state = t.to_observer(
    ///     Position::ZERO,
    ///     Velocity::ZERO,
    ///     grav_potential,
    ///     0.0, // normal solar-system use
    /// );
    /// ```
    #[inline]
    pub const fn to_observer(
        self,
        position: Position,
        velocity: Velocity,
        grav_potential_m2_s2: Real,
        characteristic_length_scale: Real,
    ) -> Observer {
        Observer {
            time: self,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale,
        }
    }
}

/// An observer at a specific instant.
///
/// Combines time, position, velocity, and local gravitational
/// information. It is the main input type used by relativistic light-time
/// methods in this library.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Observer {
    /// The time of this observer.
    ///
    /// Any [`Scale`] is accepted. This time is treated as coordinate time.
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

impl Observer {
    /// Creates a new `Observer` for typical solar-system, GNSS,
    /// or weak-field use.
    ///
    /// This is the recommended constructor for most applications.
    /// It sets the `characteristic_length_scale` to `0.0`, which disables
    /// higher-order curvature terms in the proper-time model.
    ///
    /// ## Parameters
    ///
    /// - `time`: The time of the observer.
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
    ) -> Observer {
        Self {
            time,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale: 0.0,
        }
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` for this observer.
    ///
    /// This value indicates how fast a physical clock located at this observer
    /// would advance relative to the time used by this `Observer`.
    /// A returned value of `1.0` means the clock advances at the same rate
    /// as the observer's time coordinate. Values are typically slightly different
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
    /// ```rust
    /// use deep_time::{Dt, Observer, Position, Spacetime, Velocity};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20), // Sun
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14), // Earth
    /// ];
    ///
    /// let tx_pos = Position::from_au(1.0, 0.0, 0.0);
    /// let rx_pos = Position::from_au(1.00257, 0.0, 0.0);
    ///
    /// let grav_potential_tx = Spacetime::grav_potential_from_point_masses(tx_pos, bodies.iter().copied());
    /// let grav_potential_rx = Spacetime::grav_potential_from_point_masses(rx_pos, bodies.iter().copied());
    ///
    /// let transmitter = Observer::new(
    ///     Dt::span_f(0.0),
    ///     tx_pos,
    ///     Velocity::ZERO,
    ///     grav_potential_tx,
    /// );
    ///
    /// let receiver = Observer::new(
    ///     Dt::span_f(0.0),
    ///     rx_pos,
    ///     Velocity::from_speed(800.0),
    ///     grav_potential_rx,
    /// );
    ///
    /// let one_way_ratio = transmitter.relativistic_clock_rate_ratio(receiver);
    /// let two_way_ratio = one_way_ratio * one_way_ratio;
    /// ```
    ///
    /// **Note:** Squaring the one-way ratio is a common first-order approximation.
    /// For higher precision (especially during flybys or when uplink and downlink
    /// geometries differ significantly), consider using
    /// [`round_trip_light_time_correction`](Self::round_trip_light_time_correction)
    /// instead.
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
    /// ## Parameters
    ///
    /// - `self` — Transmitter state at the time of transmission.
    /// - `rx`   — Receiver state at the approximate time of reception.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Observer, Position, Spacetime, Velocity, constants::C};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20), // Sun
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14), // Earth
    /// ];
    ///
    /// let tx_pos = Position::from_au(1.0, 0.0, 0.0);
    /// let rx_pos = Position::from_au(1.002, 0.0, 0.0);
    ///
    /// let grav_potential_tx = Spacetime::grav_potential_from_point_masses(tx_pos, bodies.iter().copied());
    /// let grav_potential_rx = Spacetime::grav_potential_from_point_masses(rx_pos, bodies.iter().copied());
    ///
    /// let transmitter = Observer::new(
    ///     Dt::span_f(0.0),
    ///     tx_pos,
    ///     Velocity::ZERO,
    ///     grav_potential_tx,
    /// );
    ///
    /// // Receiver receding at ~1.2 km/s (example spacecraft)
    /// let receiver = Observer::new(
    ///     Dt::span_f(0.0),
    ///     rx_pos,
    ///     Velocity::from_speed(1200.0),
    ///     grav_potential_rx,
    /// );
    ///
    /// let ratio = transmitter.relativistic_clock_rate_ratio(receiver);
    ///
    /// let v_radial = 1200.0; // m/s, positive if receding
    /// let classical_doppler = 1.0 - v_radial / C;
    ///
    /// let approx_frequency_shift = ratio * classical_doppler;
    /// ```
    #[inline]
    pub const fn relativistic_clock_rate_ratio(&self, rx: Observer) -> Real {
        rx.proper_time_rate() / self.proper_time_rate()
    }

    /// Computes the combined one-way relativistic correction for a signal
    /// traveling from this observer (the transmitter) to a receiver.
    ///
    /// This value is the **total extra time** you should add to the Newtonian
    /// geometric light travel time (`distance / speed of light`). It includes
    /// **two** separate relativistic effects:
    ///
    /// 1. The gravitational propagation delay (Shapiro delay) caused by the
    ///    Sun and other bodies slowing the signal.
    /// 2. The differential clock-rate correction caused by the transmitter
    ///    and receiver having slightly different proper-time rates (due to
    ///    their velocities and gravitational potentials).
    ///
    /// In other words, this method gives you **propagation delay + clock-rate
    /// correction** in one convenient call.
    ///
    /// **Important:** This is a convenience method. It is provided so you can
    /// get the full one-way relativistic correction quickly. If you need
    /// strict separation of the two effects (for example, to apply them at
    /// different stages of your calculation), call
    /// [`Self::shapiro_delay`] and [`Self::compute_differential_clock_correction`]
    /// individually and add the results yourself.
    ///
    /// ## When to use this method
    ///
    /// Use this when you need the complete relativistic correction for
    /// one-way light time in a single step — for example when:
    /// - Computing high-precision one-way range or Doppler observables
    /// - Building simplified navigation or orbit determination models
    /// - You want the total effect without manually combining the pieces
    ///
    /// ## The `bodies` parameter – which masses to include
    ///
    /// Pass a slice of `(shapiro_coefficient, body_position)` pairs:
    ///
    /// - `shapiro_coefficient`: How strong the delay from this body should be.
    ///   It equals `2GM / c³`. Use [`Dt::SHAPIRO_SOLAR`] for the Sun, or
    ///   [`Dt::shapiro_from_grav_param(gm)`] for any other body.
    /// - `body_position`: Where the center of that body is located at the
    ///   relevant time.
    ///
    /// **Important: All positions must be measured the same way**
    ///
    /// The transmitter position (`self.position`), the receiver position
    /// (`rx.position`), and every `body_position` you provide must all be
    /// measured from the **same point in space**, and they must all use
    /// the **same directions** for their X, Y, and Z axes.
    ///
    /// For example, if your transmitter position is measured from the center
    /// of the solar system, then the receiver and body positions must also
    /// be measured from the center of the solar system using the same
    /// pointing directions for the coordinate axes.
    ///
    /// In most solar-system work, people use positions from JPL ephemerides
    /// (which are measured from the center of the solar system).
    ///
    /// Pass an empty slice (`&[]`) to turn off the Shapiro (gravitational)
    /// part of the correction.
    ///
    /// ## Parameters
    ///
    /// * `rx` — Receiver state at the approximate time the signal arrives.
    /// * `bodies` — List of bodies that should contribute to the gravitational
    ///   propagation delay.
    ///
    /// ## Returns
    ///
    /// The total one-way relativistic correction (Shapiro propagation delay
    /// plus differential clock-rate correction), expressed as a `Dt` in the
    /// same time scale as `self.time`.
    ///
    /// This value should normally be **added** to the Newtonian geometric
    /// light time.
    pub const fn one_way_relativistic_delay(&self, rx: Observer, bodies: &[(Dt, Position)]) -> Dt {
        let prop = self.shapiro_delay(rx, bodies);
        let drift = self.compute_differential_clock_correction(rx);
        prop.add(drift)
    }

    /// Iteratively solves the one-way light-time equation in coordinate time,
    /// including relativistic propagation corrections, until convergence.
    ///
    /// This solver computes the receive epoch `t_rx` such that:
    ///
    /// ```text
    /// t_rx = t_tx + |r_rx(t_rx) − r_tx(t_tx)| / c + Δt_shapiro(t_tx, t_rx)
    /// ```
    ///
    /// It performs fixed-point iteration using the propagation delay returned by
    /// [`Self::shapiro_delay`]. Clock-rate and proper-time effects
    /// are **not** included in the iteration; they should be applied separately
    /// when converting between coordinate time and proper time or when forming
    /// observables.
    ///
    /// The solver is suitable for high-precision one-way light-time calculations
    /// and works with any ephemeris source via the provided closure.
    ///
    /// ## Parameters
    ///
    /// * `rx_provider` — Closure returning the full [`Observer`] of the
    ///   receiver at a given coordinate time.
    /// * `bodies` — Slice of `(shapiro_coefficient, body_position)` pairs
    ///   controlling the Shapiro contribution. Use `&[(Dt::SHAPIRO_SOLAR, sun_pos)]`
    ///   for solar-system work; include additional bodies for higher precision.
    ///   Pass `&[]` to disable Shapiro.
    /// * `tolerance` — Maximum allowed change in receive time per iteration
    ///   before declaring convergence (e.g. `Dt::from_ns(1, Scale::TAI)`).
    /// * `max_iter` — Maximum number of iterations. Typical values are 12–20
    ///   for solar-system geometries.
    ///
    /// ## Returns
    ///
    /// A tuple `(prop_correction, rx_time, final_state)` where:
    /// - `prop_correction` is the converged Shapiro propagation delay,
    /// - `rx_time` is the converged receive time (same scale as `self.time`),
    /// - `final_state` is the receiver state at `rx_time`.
    pub fn iterative_one_way_light_time_to<F>(
        &self,
        rx_provider: &mut F,
        bodies: &[(Dt, Position)],
        tolerance: Dt,
        max_iter: usize,
    ) -> (Dt, Dt, Observer)
    where
        F: FnMut(Dt) -> Observer,
    {
        // Initial geometric guess
        let initial_rx = rx_provider(self.time);
        let initial_r_sep = self.position.distance_to(initial_rx.position);
        let initial_geometric = Dt::from_sec_f(initial_r_sep / C, Scale::TAI);

        let mut rx_time = self.time.add(initial_geometric);
        let mut prop_correction = Dt::ZERO;

        for _ in 0..max_iter {
            let rx = rx_provider(rx_time);

            prop_correction = self.shapiro_delay(rx, bodies);

            let r_sep = self.position.distance_to(rx.position);
            let geometric = Dt::from_sec_f(r_sep / C, Scale::TAI);
            let full_delay = geometric.add(prop_correction);

            let new_rx_time = self.time.add(full_delay);
            let change = new_rx_time.to_diff_raw(rx_time);

            rx_time = new_rx_time;

            if change.abs() < tolerance {
                return (prop_correction, rx_time, rx);
            }
        }

        // Fallback after max iterations
        let final_rx = rx_provider(rx_time);
        (prop_correction, rx_time, final_rx)
    }

    /// Computes the total Shapiro (gravitational propagation) delay for a
    /// complete round-trip (two-way) signal.
    ///
    /// This method solves the uplink and downlink legs *separately and
    /// independently* using the iterative light-time solver. This approach
    /// is more accurate than older combined round-trip formulas when the
    /// two ends have significantly different velocities or are in different
    /// gravitational environments.
    ///
    /// The returned value is the **sum of the uplink and downlink Shapiro
    /// delays only**. It does **not** include clock-rate or proper-time
    /// corrections.
    ///
    /// ## When to use this method
    ///
    /// Use this when you need the total gravitational propagation correction
    /// for two-way (round-trip) measurements, for example:
    /// - Two-way range or range-rate (Doppler) data
    /// - Transponded signals from spacecraft
    /// - Any high-precision two-way light-time calculation
    ///
    /// For one-way signals, use [`Self::shapiro_delay`] or
    /// [`Self::one_way_relativistic_delay`] instead.
    ///
    /// ## How the calculation works
    ///
    /// 1. Solves the uplink leg (from `self` to the remote receiver) using
    ///    the `rx_provider` closure.
    /// 2. Obtains the accurate receiver state at the uplink arrival time.
    /// 3. Solves the downlink leg (from the receiver back to the local
    ///    transmitter) using the `tx_provider` closure.
    ///
    /// ## The `bodies` parameter – which masses to include
    ///
    /// Pass a slice of `(shapiro_coefficient, body_position)` pairs (the
    /// same slice is used for both legs). See [`Self::shapiro_delay`] for
    /// details on how to build this slice.
    ///
    /// **Important: All states returned by the providers must be consistent**
    /// with the same reference frame (same origin and same coordinate axes).
    ///
    /// ## Parameters
    ///
    /// * `rx_provider` — Closure that returns the full [`Observer`] of
    ///   the remote receiver (planet, spacecraft, etc.) at any given
    ///   coordinate time.
    /// * `tx_provider` — Closure that returns the full [`Observer`] of
    ///   the local transmitter at any given coordinate time (used only for
    ///   the downlink leg).
    /// * `bodies` — Slice of `(shapiro_coefficient, body_position)` pairs
    ///   describing the gravitating bodies.
    /// * `tolerance` — Convergence tolerance for each leg’s iterative solver
    ///   (e.g. `Dt::from_ns(1, Scale::TAI)`).
    /// * `max_iter` — Maximum number of iterations allowed per leg
    ///   (typical values are 12–20).
    ///
    /// ## Returns
    ///
    /// The total round-trip Shapiro propagation delay (uplink + downlink)
    /// as a `Dt`, in the same time scale as `self.time`.
    ///
    /// This value should normally be **added** to the Newtonian geometric
    /// round-trip light time. Clock-rate corrections must still be applied
    /// separately (e.g. by squaring the one-way clock-rate ratio).
    pub fn round_trip_light_time_correction<RxF, TxF>(
        &self,
        mut rx_provider: RxF, // remote body (planet, spacecraft, etc.)
        mut tx_provider: TxF, // local transmitter for the return leg (can move)
        bodies: &[(Dt, Position)],
        tolerance: Dt,
        max_iter: usize,
    ) -> Dt
    where
        RxF: FnMut(Dt) -> Observer,
        TxF: FnMut(Dt) -> Observer,
    {
        // Uplink leg: transmitter → receiver
        let (uplink_prop, rx_time, _rx_state) =
            self.iterative_one_way_light_time_to(&mut rx_provider, bodies, tolerance, max_iter);

        // Downlink leg: receiver → transmitter
        let return_tx = rx_provider(rx_time); // accurate state at uplink arrival

        let (downlink_prop, _return_rx_time, _return_rx_state) = return_tx
            .iterative_one_way_light_time_to(&mut tx_provider, bodies, tolerance, max_iter);

        uplink_prop.add(downlink_prop)
    }

    /// Computes the one-way gravitational propagation delay (Shapiro delay)
    /// caused by massive bodies between this observer (the transmitter) and
    /// a receiver.
    ///
    /// This value is the **extra time** a radio signal takes to travel because
    /// gravity from the Sun and planets slightly slows it down. You normally
    /// add this delay to the ordinary geometric light travel time
    /// (`distance / speed of light`) to get a more accurate total one-way
    /// signal travel time.
    ///
    /// **Important:** This method returns **only** the gravitational
    /// propagation delay. It does **not** include clock-rate differences
    /// between the transmitter and receiver caused by velocity or gravity.
    /// Those effects are available separately through
    /// [`Self::compute_differential_clock_correction`],
    /// [`Self::proper_time_rate`], and [`Self::relativistic_clock_rate_ratio`].
    ///
    /// ## When to use this method
    ///
    /// Use this when you need the gravitational (Shapiro) contribution to
    /// one-way light time — for example when building high-precision range,
    /// Doppler, or orbit determination models.
    ///
    /// ## The `bodies` parameter – which masses to include
    ///
    /// Pass a slice of `(shapiro_coefficient, body_position)` pairs:
    ///
    /// - `shapiro_coefficient`: How strong the delay from this body should be.
    ///   It equals `2GM / c³`. Use [`Dt::SHAPIRO_SOLAR`] for the Sun, or
    ///   [`Dt::shapiro_from_grav_param(gm)`] for any other body.
    /// - `body_position`: Where the center of that body is located at the
    ///   relevant time.
    ///
    /// **Important: All positions must be measured the same way**
    ///
    /// The transmitter position (`self.position`), the receiver position
    /// (`rx.position`), and every `body_position` you provide must all be
    /// measured from the **same point in space**, and they must all use
    /// the **same directions** for their X, Y, and Z axes.
    ///
    /// For example, if the transmitter position is measured from the center
    /// of the solar system, then the receiver and body positions must also
    /// be measured from the center of the solar system, using the same
    /// pointing directions for the coordinate axes.
    ///
    /// If the positions come from different measurement systems, the
    /// calculated delay will be wrong.
    ///
    /// In most solar-system work, people use positions from JPL ephemerides
    /// (which are measured from the center of the solar system).
    ///
    /// Pass an empty slice (`&[]`) to turn off Shapiro delay entirely.
    ///
    /// ## Parameters
    ///
    /// * `rx` — Receiver state at the approximate time the signal arrives.
    /// * `bodies` — List of bodies that should contribute to the delay.
    ///
    /// ## Returns
    ///
    /// The total one-way Shapiro gravitational propagation delay, in the
    /// same time scale as `self.time`. This value should normally be
    /// **added** to the Newtonian geometric light time.
    pub const fn shapiro_delay(&self, rx: Observer, bodies: &[(Dt, Position)]) -> Dt {
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
    ///
    ///   ln((r_tx + r_rx + r_sep) / (r_tx + r_rx - r_sep))
    ///   ≡ 2·ln(num) − ln(denom_term)
    ///
    ///
    /// where denom_term is computed from the dot-product identity
    /// (r_tx + r_rx)² − r_sep² = 2(r_tx·r_rx + p_tx · p_rx).
    /// This avoids the dangerous subtraction that loses precision when
    /// the signal path passes close to the body.
    ///
    /// The result is equivalent (within floating-point) to the
    /// classic Moyer/DSN-style formula while being far more stable.
    /// Contributions from multiple bodies are summed at a higher level.
    ///
    /// ## Safety / Guards
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
        Dt::from_sec_f(delay_sec, Scale::TAI)
    }

    /// Computes the differential proper-time correction between `self`
    /// (transmitter) and `rx` (receiver) over the interval between their
    /// time tags.
    ///
    /// This returns the difference in proper time advance between the two
    /// observers. It does **not** include Shapiro propagation delay.
    ///
    /// The result can be added to the output of [`Self::shapiro_delay`]
    /// or [`Self::iterative_one_way_light_time_to`] when a combined
    /// relativistic correction (propagation + clock rate) is required.
    ///
    /// ## Parameters
    ///
    /// * `rx` — Receiver state at the approximate time of reception.
    ///
    /// ## Returns
    ///
    /// The differential clock-rate correction (`rx_proper_advance − tx_proper_advance`).
    pub const fn compute_differential_clock_correction(&self, rx: Observer) -> Dt {
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
