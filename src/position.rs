use crate::{C_SQUARED, Timestamp};

/// A 3-dimensional position vector expressed in Cartesian coordinates (x, y, z)
/// with units of meters (SI).
///
/// This type is designed for high-precision relativistic calculations in space
/// navigation, deep-space tracking, and interplanetary timing. Positions are
/// typically expressed in a heliocentric (Sun-centered) reference frame because
/// the dominant gravitational light-time correction—the Shapiro delay—is
/// calculated with respect to the Sun.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    /// Creates a new `Position` directly from its Cartesian components in meters.
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the zero vector, representing the origin of the coordinate system
    /// (commonly the center of the Sun).
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a `Position` from coordinates expressed in Astronomical Units (AU),
    /// converting them to meters using the exact IAU 2012 definition
    /// (1 AU = 149 597 870 700 m).
    ///
    /// Especially convenient when working with planetary ephemerides or solar-system
    /// models that are natively given in AU.
    #[inline]
    pub const fn from_au(x: f64, y: f64, z: f64) -> Self {
        const AU: f64 = 1.495978707e11; // exact IAU 2012 definition
        Self {
            x: x * AU,
            y: y * AU,
            z: z * AU,
        }
    }

    /// Returns the Euclidean norm (straight-line distance) of this position from
    /// the origin.
    ///
    /// When the position is Sun-centered, this is the radial distance from the Sun
    /// required for Shapiro-delay calculations.
    #[inline]
    pub fn norm(self) -> f64 {
        self.x.hypot(self.y).hypot(self.z)
    }

    /// Computes the straight-line (Euclidean) distance between this position and
    /// another `Position`.
    ///
    /// Together with the two radial distances from the Sun, this value supplies the
    /// three geometric inputs needed to evaluate the Shapiro delay.
    #[inline]
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx.hypot(dy).hypot(dz)
    }
}

/// A 3-dimensional velocity vector expressed in Cartesian coordinates (vx, vy, vz)
/// with units of meters per second (SI).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Velocity {
    /// Creates a new `Velocity` directly from its Cartesian components in m/s.
    #[inline]
    pub const fn new(vx: f64, vy: f64, vz: f64) -> Self {
        Self { vx, vy, vz }
    }

    /// Creates a `Velocity` from its scalar speed (magnitude) in m/s.
    ///
    /// Direction is set along the x-axis because only the speed matters
    /// for relativistic calculations (`beta()`, `norm_squared()`, etc.).
    /// This is the convenience constructor used by `ClockDrift::from_velocity_potential_and_scale`.
    #[inline]
    pub const fn from_speed(speed_m_s: f64) -> Self {
        Self::new(speed_m_s, 0.0, 0.0)
    }

    /// Returns the squared Euclidean norm (v²).
    #[inline]
    pub fn norm_squared(self) -> f64 {
        self.vx * self.vx + self.vy * self.vy + self.vz * self.vz
    }

    /// Speed in m/s (Euclidean magnitude).
    #[inline]
    pub fn speed(self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Dimensionless 3-velocity β = v/c relative to the local chrono-rest frame.
    /// This is exactly what the master Lagrangian and `LocalSpacetime` expect.
    #[inline]
    pub fn beta(self) -> f64 {
        (self.norm_squared() / C_SQUARED).sqrt()
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
    /// Timestamp of this state (any [`ClockType`] is accepted).
    pub time: Timestamp,
    /// Position in meters (typically barycentric or heliocentric).
    pub position: Position,
    /// Velocity in meters per second.
    pub velocity: Velocity,
    /// Local gravitational potential Φ in m² s⁻² (negative for bound orbits).
    /// Usually the sum of contributions from the Sun and planets.
    pub gravitational_potential_m2_s2: f64,
    /// Characteristic length scale (in meters) over which gravity varies
    /// significantly at the observer’s location.  
    /// Pass `0.0` (the default) for all solar-system, GNSS, and weak-field cases.
    pub characteristic_length_scale: f64,
}

impl ObserverState {
    /// Creates a new state for typical solar-system or GNSS use.
    #[inline]
    pub const fn new(
        time: Timestamp,
        position: Position,
        velocity: Velocity,
        gravitational_potential_m2_s2: f64,
    ) -> Self {
        Self {
            time,
            position,
            velocity,
            gravitational_potential_m2_s2,
            characteristic_length_scale: 0.0,
        }
    }

    /// Creates a new state when strong-field or gravimeter data is available.
    #[inline]
    pub const fn new_strong_field(
        time: Timestamp,
        position: Position,
        velocity: Velocity,
        gravitational_potential_m2_s2: f64,
        characteristic_length_scale: f64,
    ) -> Self {
        Self {
            time,
            position,
            velocity,
            gravitational_potential_m2_s2,
            characteristic_length_scale,
        }
    }
}
