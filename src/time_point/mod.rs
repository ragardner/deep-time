mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
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
/// (where 1 attosecond = 10⁻¹⁸ s) since the reference epoch of the
/// associated ClockType.
///
/// - Precision: 10⁻¹⁸ s
/// - Range: ±~292 billion years (i64 seconds limit).
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
