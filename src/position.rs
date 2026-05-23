use crate::{C_SQUARED, Real, hypot, sqrt};

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
    pub x: Real,
    pub y: Real,
    pub z: Real,
}

impl Position {
    /// Creates a new `Position` directly from its Cartesian components in meters.
    #[inline]
    pub const fn new(x: Real, y: Real, z: Real) -> Self {
        Self { x, y, z }
    }

    /// The zero vector, representing the origin of the coordinate system
    /// (commonly the center of the Sun).
    pub const ZERO: Self = Self::new(f!(0.0), f!(0.0), f!(0.0));

    /// Creates a `Position` from coordinates expressed in Astronomical Units (AU),
    /// converting them to meters using the exact IAU 2012 definition
    /// (1 AU = 149 597 870 700 m).
    ///
    /// Especially convenient when working with planetary ephemerides or solar-system
    /// models that are natively given in AU.
    #[inline]
    pub const fn from_au(x: Real, y: Real, z: Real) -> Self {
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
    /// when building the `samples` slice for [`ObserverState::one_way_relativistic_delay_integrated`].
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
    pub const fn lerp(self, other: Self, t: Real) -> Self {
        Self::new(
            self.x * (f!(1.0) - t) + other.x * t,
            self.y * (f!(1.0) - t) + other.y * t,
            self.z * (f!(1.0) - t) + other.z * t,
        )
    }
}

/// A 3-dimensional velocity vector expressed in Cartesian coordinates (vx, vy, vz)
/// with units of meters per second (SI).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Velocity {
    pub vx: Real,
    pub vy: Real,
    pub vz: Real,
}

impl Velocity {
    /// Creates a new `Velocity` directly from its Cartesian components in m/s.
    #[inline]
    pub const fn new(vx: Real, vy: Real, vz: Real) -> Self {
        Self { vx, vy, vz }
    }

    pub const ZERO: Self = Self::new(f!(0.0), f!(0.0), f!(0.0));

    /// Creates a `Velocity` from its scalar speed (magnitude) in m/s.
    ///
    /// Direction is set along the x-axis because only the speed matters
    /// for relativistic calculations (`beta()`, `norm_squared()`, etc.).
    /// This is the convenience constructor used by `Drift::from_velocity_potential_and_scale`.
    #[inline]
    pub const fn from_speed(speed_m_s: Real) -> Self {
        Self::new(speed_m_s, f!(0.0), f!(0.0))
    }

    /// Returns the squared Euclidean norm (v²).
    #[inline]
    pub const fn norm_squared(self) -> Real {
        self.vx * self.vx + self.vy * self.vy + self.vz * self.vz
    }

    /// Speed in m/s (Euclidean magnitude).
    #[inline]
    pub const fn speed(self) -> Real {
        sqrt(self.norm_squared().max(f!(0.0)))
    }

    /// Dimensionless 3-velocity β = v/c relative to the local chrono-rest frame.
    /// This is exactly what the master Lagrangian and `Spacetime` expect.
    #[inline]
    pub const fn beta(self) -> Real {
        sqrt((self.norm_squared() / C_SQUARED).max(f!(0.0)))
    }
}
