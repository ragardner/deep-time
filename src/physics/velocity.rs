//! Velocity vector in meters per second.

use crate::{C_SQUARED, Real, sqrt};

/// A 3-dimensional velocity vector expressed in Cartesian coordinates (vx, vy, vz)
/// with units of meters per second (SI).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Velocity {
    pub vx: Real,
    pub vy: Real,
    pub vz: Real,
}

impl Velocity {
    /// Creates a new `Velocity` directly from its Cartesian components in m/s.
    #[inline]
    pub const fn new(vx: Real, vy: Real, vz: Real) -> Velocity {
        Self { vx, vy, vz }
    }

    pub const ZERO: Self = Self::new(f!(0.0), f!(0.0), f!(0.0));

    /// Creates a `Velocity` from its scalar speed (magnitude) in m/s.
    ///
    /// Direction is set along the x-axis because only the speed matters
    /// for relativistic calculations (`beta()`, `norm_squared()`, etc.).
    /// This is the convenience constructor used by `Drift::from_velocity_potential_and_scale`.
    #[inline]
    pub const fn from_speed(speed_m_s: Real) -> Velocity {
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
    /// This is what the master Lagrangian and `Spacetime` expect.
    #[inline]
    pub const fn beta(self) -> Real {
        sqrt((self.norm_squared() / C_SQUARED).max(f!(0.0)))
    }
}
