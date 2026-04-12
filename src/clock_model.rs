use crate::{ClockDrift, ClockType, Timestamp};

/// A fully self-describing relativistic time scale for spacecraft and probes.
///
/// Bundles a base `ClockType` (normally `Proper` or `Custom`) with the quadratic
/// polynomial and reference epoch needed for exact conversion to any other scale
/// (typically TT or TDB).
///
/// This is the recommended way to represent onboard proper time that carries
/// its own clock-drift / relativistic model.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClockModel {
    /// Base scale (usually `Proper` or `Custom`)
    pub base: ClockType,
    /// Epoch at which the polynomial was defined (e.g. last ground contact)
    pub reference: Timestamp,
    /// Quadratic correction model (exact 36-digit precision)
    pub drift: ClockDrift,
}

impl ClockModel {
    /// Creates a new self-describing scale (most common for Proper time).
    #[inline]
    pub const fn new(base: ClockType, reference: Timestamp, drift: ClockDrift) -> Self {
        Self {
            base,
            reference,
            drift,
        }
    }

    /// Convenience constructor for a pure Proper-time scale with relativistic correction.
    #[inline]
    pub const fn proper(reference: Timestamp, drift: ClockDrift) -> Self {
        Self::new(ClockType::Proper, reference, drift)
    }

    /// Convenience constructor for a custom scale.
    #[inline]
    pub const fn custom(reference: Timestamp, drift: ClockDrift) -> Self {
        Self::new(ClockType::Custom, reference, drift)
    }

    /// Attaches this self-describing scale to an existing `Timestamp`.
    ///
    /// Useful when you have a raw onboard reading and the latest polynomial update
    /// from ground control.
    #[inline]
    pub const fn attach_to(self, point: Timestamp) -> Timestamp {
        point.with_clock_type(self.base)
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

    /// Convenience: creates a `Timestamp` in this scale from a TAI instant.
    #[inline]
    pub const fn from_tai(self, tai: Timestamp) -> Timestamp {
        tai.with_clock_type(self.base)
    }
}
