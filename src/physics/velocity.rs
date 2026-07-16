//! Velocity vector in meters per second.

use crate::{C_SQUARED, Real, sqrt};

/// A 3-dimensional velocity vector expressed in Cartesian coordinates (vx, vy, vz)
/// with units of meters per second (SI).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Velocity {
    /// X-component of velocity in meters per second (SI).
    pub vx: Real,
    /// Y-component of velocity in meters per second (SI).
    pub vy: Real,
    /// Z-component of velocity in meters per second (SI).
    pub vz: Real,
}

impl Velocity {
    /// Creates a new `Velocity` directly from its Cartesian components in m/s.
    #[inline]
    pub const fn new(vx: Real, vy: Real, vz: Real) -> Velocity {
        Self { vx, vy, vz }
    }

    /// The zero velocity vector (at rest in the coordinate frame).
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

#[cfg(feature = "wire")]
impl Velocity {
    /// Size of the canonical wire representation in bytes (24 bytes).
    pub const WIRE_SIZE: usize = 24;

    /// Serializes this [`Velocity`] into a fixed 24-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.vx.to_le_bytes());
        buf[8..16].copy_from_slice(&self.vy.to_le_bytes());
        buf[16..24].copy_from_slice(&self.vz.to_le_bytes());
        buf
    }

    /// Deserializes a [`Velocity`] from exactly 24 bytes.
    ///
    /// ## Security
    ///
    /// Accepts any [`Real`] bit pattern (including `NaN`/`Inf`) to match the
    /// type’s own invariants. Fixed size makes it immune to length-based
    /// attacks. Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let vx = Real::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let vy = Real::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let vz = Real::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        Some(Self { vx, vy, vz })
    }
}
