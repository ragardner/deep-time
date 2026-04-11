mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod ops;
mod point_pov;
pub mod traits;
mod unix;

use crate::TimePov;

/// A high-precision timestamp expressed in a specific [`TimePov`].
///
/// `Point` represents an instant in time as **seconds + microquectoseconds**
/// (where 1 microquectosecond = 10⁻³⁶ s) since the reference epoch of the
/// associated time scale.
///
/// - Precision: 10⁻³⁶ s
/// - Range: ±~5 × 10³⁰ years.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Point {
    /// Signed whole seconds since the reference epoch of the pov.
    pub(crate) sec: i128,
    /// Fractional part in microquectoseconds (`0 ≤ microquectos < 10³⁶`).
    pub(crate) subsec: u128,
    /// The time scale this instant belongs to.
    pub(crate) pov: TimePov,
}

impl Point {
    #[inline(always)]
    pub const fn sec(&self) -> i128 {
        self.sec
    }

    #[inline(always)]
    pub const fn subsec(&self) -> u128 {
        self.subsec
    }

    #[inline(always)]
    pub const fn pov(&self) -> TimePov {
        self.pov
    }
}
