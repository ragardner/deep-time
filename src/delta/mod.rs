mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod ops;
pub mod time_units;

#[cfg(feature = "chrono")]
pub mod from_chrono;
#[cfg(feature = "chrono")]
pub mod to_chrono;

#[cfg(feature = "jiff")]
pub mod from_jiff;
#[cfg(feature = "jiff")]
pub mod to_jiff;

/// A high-precision **duration** (time delta) expressed as **seconds + attoseconds**
/// (where 1 attosecond = 10⁻¹⁸ s).
///
/// `Delta` is the delta counterpart of `TimePoint`. It does **not** carry a [`ClockType`]
/// because durations are scale-independent (they can be added to or subtracted from any
/// `TimePoint` regardless of its scale; any scale-specific adjustments like leap seconds
/// are handled by the `TimePoint` arithmetic).
///
/// - Precision: 10⁻¹⁸ s
/// - Range: ±~292 billion years (i64 seconds limit).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delta {
    /// Signed whole seconds.
    pub(crate) sec: i64,
    /// Fractional part in attoseconds (`0 ≤ attos < 10¹⁸`).
    pub(crate) subsec: u64,
}

impl Delta {
    #[inline(always)]
    pub const fn sec(&self) -> i64 {
        self.sec
    }

    #[inline(always)]
    pub const fn subsec(&self) -> u64 {
        self.subsec
    }
}

impl Default for Delta {
    fn default() -> Self {
        Self::ZERO
    }
}
