//! 3D position vector (in meters) for relativistic calculations.

use crate::{Real, hypot};

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
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Position {
    pub x: Real,
    pub y: Real,
    pub z: Real,
}

impl Position {
    /// Creates a new `Position` directly from its Cartesian components in meters.
    #[inline]
    pub const fn new(x: Real, y: Real, z: Real) -> Position {
        Self { x, y, z }
    }

    /// The zero vector, representing the origin of the coordinate system
    /// (commonly the center of the Sun).
    pub const ZERO: Self = Self::new(f!(0.0), f!(0.0), f!(0.0));

    /// Creates a `Position` from coordinates expressed in Astronomical Units (AU),
    /// converting them to meters using the IAU 2012 definition
    /// (1 AU = 149 597 870 700 m).
    ///
    /// Especially convenient when working with planetary ephemerides or solar-system
    /// models that are natively given in AU.
    #[inline]
    pub const fn from_au(x: Real, y: Real, z: Real) -> Position {
        const AU: Real = f!(1.495978707e11);
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
    pub const fn norm(self) -> Real {
        hypot(hypot(self.x, self.y), self.z)
    }

    /// Computes the straight-line (Euclidean) distance between this position and
    /// another `Position`.
    ///
    /// Together with the two radial distances from the Sun, this value supplies the
    /// three geometric inputs needed to evaluate the Shapiro delay.
    pub const fn distance_to(self, other: Self) -> Real {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        hypot(hypot(dx, dy), dz)
    }

    /// Returns a new position that lies a fraction `t` of the way along the straight
    /// line between `self` and `other`.
    ///
    /// This is known as linear interpolation (lerp). It is most commonly used when
    /// you need to generate evenly spaced sample points along a path — for example,
    /// when building the `samples` slice for [`Observer::one_way_relativistic_delay_integrated`].
    ///
    /// ## Parameters
    ///
    /// - `other` – the ending position
    /// - `t` – interpolation parameter (0.0 = start point, 1.0 = end point).
    ///   Values outside [0, 1] are allowed and will extrapolate.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Position;
    ///
    /// let a = Position::new(0.0, 0.0, 0.0);
    /// let b = Position::new(10.0, 20.0, 30.0);
    ///
    /// let midpoint = a.lerp(b, 0.5);           // (5.0, 10.0, 15.0)
    /// let quarter   = a.lerp(b, 0.25);         // (2.5, 5.0, 7.5)
    /// let beyond    = a.lerp(b, 1.5);          // (15.0, 30.0, 45.0)
    /// ```
    #[inline]
    pub const fn lerp(self, other: Self, t: Real) -> Position {
        Self::new(
            self.x * (f!(1.0) - t) + other.x * t,
            self.y * (f!(1.0) - t) + other.y * t,
            self.z * (f!(1.0) - t) + other.z * t,
        )
    }
}
