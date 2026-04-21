mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod gregorian;
mod ops;
mod unix;

pub mod trajectory;

#[cfg(feature = "chrono")]
pub mod from_chrono;
#[cfg(feature = "chrono")]
pub mod to_chrono;

#[cfg(feature = "std")]
pub mod from_jiff;
#[cfg(feature = "std")]
pub mod to_jiff;

use crate::ClockType;

/// A high-precision point in time expressed in a specific [`ClockType`].
///
/// `TimePoint` represents an instant in time as **seconds + attoseconds**
/// (where 1 attosecond = 10⁻¹⁸ s) **since the reference epoch** of the
/// associated [`ClockType`].
///
/// Every clock type has its own zero point — the exact physical moment when
/// its internal counter reads `sec = 0, subsec = 0`. This library anchors
/// almost everything to **J2000.0 TT** (`2000-01-01 12:00:00 TT`, JD 2451545.0)
/// so that numbers stay small, math stays fast, and relativistic corrections
/// remain perfectly exact.
///
/// The full explanation of every clock type’s reference epoch — including
/// why we chose J2000-centric zeros, the exact offsets and IAU/NIST rates
/// (`L_G`, `L_B`, `L_M`), leap-second handling, and how Proper/Custom work —
/// is in the module-level documentation of the [`ClockType`] enum.
///
/// - Precision: 10⁻¹⁸ s (attosecond)
/// - Range: ±~292 billion years (i64 seconds limit)
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct TimePoint {
    /// Signed whole seconds since the reference epoch of the clock_type.
    pub(crate) sec: i64,
    /// Fractional part in attoseconds (`0 ≤ attos < 10¹⁸`).
    pub(crate) subsec: u64,
    /// The time scale this instant belongs to.
    pub(crate) clock_type: ClockType,
}

impl TimePoint {
    #[inline(always)]
    pub const fn sec(&self) -> i64 {
        self.sec
    }

    #[inline(always)]
    pub const fn subsec(&self) -> u64 {
        self.subsec
    }

    #[inline(always)]
    pub const fn clock_type(&self) -> ClockType {
        self.clock_type
    }
}

impl Default for TimePoint {
    fn default() -> Self {
        Self::ZERO
    }
}
