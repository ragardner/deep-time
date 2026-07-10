//! Observer state at an instant.

mod light_time;

use crate::{C_SQUARED, Dt, Position, Real, Spacetime, Velocity};

/// Snapshot of one observer’s state at a single instant: time, position,
/// velocity, and local gravity.
///
/// This is the main input type used by relativistic light-time methods
/// in this library.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Observer {
    /// The time of this observer.
    ///
    /// Any [`Scale`](../enum.Scale.html) is accepted. This time is treated as coordinate time.
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
    /// let grav_potential_tx = Spacetime::grav_potential_from_point_masses(&tx_pos, bodies.iter().cloned());
    /// let grav_potential_rx = Spacetime::grav_potential_from_point_masses(&rx_pos, bodies.iter().cloned());
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
    /// let one_way_ratio = transmitter.relativistic_clock_rate_ratio(&receiver);
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
    #[inline]
    pub const fn relativistic_clock_rate_ratio(&self, rx: &Observer) -> Real {
        rx.proper_time_rate() / self.proper_time_rate()
    }
}

#[cfg(feature = "wire")]
impl Observer {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 1 + Dt::WIRE_SIZE + Position::WIRE_SIZE + Velocity::WIRE_SIZE + 16;

    /// Serializes this [`Observer`] into a fixed buffer.
    ///
    /// Layout:
    /// - Byte 0: Version
    /// - Bytes [1..]: time (Dt wire) + position (24) + velocity (24) + grav_potential (8) + char_length_scale (8)
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let mut offset = 1usize;

        let time = self.time.to_wire_bytes();
        buf[offset..offset + Dt::WIRE_SIZE].copy_from_slice(&time);
        offset += Dt::WIRE_SIZE;

        let pos = self.position.to_wire_bytes();
        buf[offset..offset + Position::WIRE_SIZE].copy_from_slice(&pos);
        offset += Position::WIRE_SIZE;

        let vel = self.velocity.to_wire_bytes();
        buf[offset..offset + Velocity::WIRE_SIZE].copy_from_slice(&vel);
        offset += Velocity::WIRE_SIZE;

        buf[offset..offset + 8].copy_from_slice(&self.grav_potential_m2_s2.to_le_bytes());
        offset += 8;

        buf[offset..offset + 8].copy_from_slice(&self.characteristic_length_scale.to_le_bytes());

        buf
    }

    /// Deserializes an [`Observer`] from exactly `WIRE_SIZE` bytes.
    ///
    /// Returns `None` if the version is unknown or any component is invalid.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed size with layered validation of inner types.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let mut offset = 1usize;

        let time = Dt::from_wire_bytes(&bytes[offset..offset + Dt::WIRE_SIZE])?;
        offset += Dt::WIRE_SIZE;

        let position = Position::from_wire_bytes(&bytes[offset..offset + Position::WIRE_SIZE])?;
        offset += Position::WIRE_SIZE;

        let velocity = Velocity::from_wire_bytes(&bytes[offset..offset + Velocity::WIRE_SIZE])?;
        offset += Velocity::WIRE_SIZE;

        let grav_potential_m2_s2 = Real::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let characteristic_length_scale = Real::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);

        Some(Self {
            time,
            position,
            velocity,
            grav_potential_m2_s2,
            characteristic_length_scale,
        })
    }
}
