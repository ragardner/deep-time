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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Position {
    /// X-coordinate in meters (SI).
    pub x: Real,
    /// Y-coordinate in meters (SI).
    pub y: Real,
    /// Z-coordinate in meters (SI).
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
    pub const fn norm(&self) -> Real {
        hypot(hypot(self.x, self.y), self.z)
    }

    /// Computes the straight-line (Euclidean) distance between this position and
    /// another `Position`.
    ///
    /// Together with the two radial distances from the Sun, this value supplies the
    /// three geometric inputs needed to evaluate the Shapiro delay.
    pub const fn distance_to(&self, other: &Self) -> Real {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        hypot(hypot(dx, dy), dz)
    }

    /// Returns a new position that lies a fraction `t` of the way along the straight
    /// line between `self` and `other`.
    ///
    /// This is known as linear interpolation (lerp). It is useful when you need
    /// an intermediate position along a straight-line segment between two known points.
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
    /// let midpoint = a.lerp(&b, 0.5);           // (5.0, 10.0, 15.0)
    /// let quarter   = a.lerp(&b, 0.25);         // (2.5, 5.0, 7.5)
    /// let beyond    = a.lerp(&b, 1.5);          // (15.0, 30.0, 45.0)
    /// ```
    #[inline]
    pub const fn lerp(&self, other: &Self, t: Real) -> Position {
        Self::new(
            self.x * (f!(1.0) - t) + other.x * t,
            self.y * (f!(1.0) - t) + other.y * t,
            self.z * (f!(1.0) - t) + other.z * t,
        )
    }
}

#[cfg(feature = "wire")]
impl Position {
    /// Size of the canonical wire representation in bytes (24 bytes).
    pub const WIRE_SIZE: usize = 24;

    /// Serializes this [[`Position`] into a fixed 24-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.x.to_le_bytes());
        buf[8..16].copy_from_slice(&self.y.to_le_bytes());
        buf[16..24].copy_from_slice(&self.z.to_le_bytes());
        buf
    }

    /// Deserializes a [`Position`] from exactly 24 bytes.
    ///
    /// ## Security
    ///
    /// Accepts any `f64` bit pattern (including `NaN`/`Inf`) to match the
    /// type’s own invariants. Fixed size makes it immune to length-based
    /// attacks. Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let x = Real::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let y = Real::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let z = Real::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        Some(Self { x, y, z })
    }
}
