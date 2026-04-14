mod arithmetic;
mod constructors;
mod formatting;
mod ops;
pub mod time_units;

/// A high-precision **duration** (time delta) expressed as **seconds + microquectoseconds**
/// (where 1 microquectosecond = 10⁻³⁶ s).
///
/// `Delta` is the delta counterpart of `TimePoint`. It does **not** carry a [`ClockType`]
/// because durations are scale-independent (they can be added to or subtracted from any
/// `TimePoint` regardless of its scale; any scale-specific adjustments like leap seconds
/// are handled by the `TimePoint` arithmetic).
///
/// - Precision: 10⁻³⁶ s
/// - Range: ±~5 × 10³⁰ years (identical to `TimePoint`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Delta {
    /// Signed whole seconds.
    pub(crate) sec: i128,
    /// Fractional part in microquectoseconds (`0 ≤ microquectos < 10³⁶`).
    pub(crate) subsec: u128,
}
