use crate::{ClockDrift, ClockType, TimePoint};

/// A fully self-describing relativistic time scale.
///
/// Bundles a base `ClockType` (normally `Proper` or `Custom`) with the quadratic
/// polynomial and reference epoch needed for exact conversion to any other scale
/// (typically TT or TDB).
///
/// This is the recommended way to represent onboard proper time that carries
/// its own clock-drift / relativistic model.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct ClockModel {
    /// Base scale (usually `Proper` or `Custom`)
    pub base: ClockType,
    /// Epoch at which the polynomial was defined (e.g. last ground contact)
    pub reference: TimePoint,
    /// Quadratic correction model (exact 36-digit precision)
    pub drift: ClockDrift,
}

impl ClockModel {
    /// Creates a new self-describing scale (most common for Proper time).
    #[inline]
    pub const fn new(base: ClockType, reference: TimePoint, drift: ClockDrift) -> Self {
        Self {
            base,
            reference,
            drift,
        }
    }

    /// Returns a new `ClockModel` with the same base type and reference epoch,
    /// but with an updated `ClockDrift`.
    #[inline]
    pub const fn with_drift(self, new_drift: ClockDrift) -> Self {
        Self {
            base: self.base,
            reference: self.reference,
            drift: new_drift,
        }
    }
}

#[cfg(feature = "wire")]
impl ClockModel {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize =
        1 + ClockType::WIRE_SIZE + TimePoint::WIRE_SIZE + ClockDrift::WIRE_SIZE;

    /// Serializes this self-describing `ClockModel` into a fixed buffer.
    ///
    /// # Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Byte `1`: `base` (`ClockType`)
    /// - Bytes `2..20`: `reference` (`TimePoint`)
    /// - Bytes `20..71`: `drift` (`ClockDrift`)
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1] = self.base as u8;

        let tp = self.reference.to_wire_bytes();
        buf[2..2 + TimePoint::WIRE_SIZE].copy_from_slice(&tp);

        let cd = self.drift.to_wire_bytes();
        buf[2 + TimePoint::WIRE_SIZE..].copy_from_slice(&cd);

        buf
    }

    /// Deserializes a `ClockModel` from exactly `WIRE_SIZE` bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown or any nested component
    /// fails validation.
    ///
    /// ## Security
    ///
    /// This function is safe to call with arbitrary untrusted data because:
    /// - Fixed total size eliminates length-prefix vulnerabilities
    /// - Validation is performed at every layer
    /// - No allocation, no `unsafe`, no possibility of code execution
    /// - Returns `None` on any invalid or malicious input
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let base = ClockType::from_u8(bytes[1])?;
        let reference = TimePoint::from_wire_bytes(&bytes[2..2 + TimePoint::WIRE_SIZE])?;
        let drift = ClockDrift::from_wire_bytes(&bytes[2 + TimePoint::WIRE_SIZE..])?;

        Some(Self {
            base,
            reference,
            drift,
        })
    }
}
