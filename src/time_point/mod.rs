mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod ops;
pub mod trajectory;
mod unix;

use crate::ClockType;

/// A high-precision point in time expressed in a specific [`ClockType`].
///
/// `TimePoint` represents an instant in time as **seconds + microquectoseconds**
/// (where 1 microquectosecond = 10⁻³⁶ s) since the reference epoch of the
/// associated ClockType.
///
/// - Precision: 10⁻³⁶ s
/// - Range: ±~5 × 10³⁰ years.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct TimePoint {
    /// Signed whole seconds since the reference epoch of the clock_type.
    pub(crate) sec: i128,
    /// Fractional part in microquectoseconds (`0 ≤ microquectos < 10³⁶`).
    pub(crate) subsec: u128,
    /// The time scale this instant belongs to.
    pub(crate) clock_type: ClockType,
}

impl TimePoint {
    #[inline(always)]
    pub const fn sec(&self) -> i128 {
        self.sec
    }

    #[inline(always)]
    pub const fn subsec(&self) -> u128 {
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
