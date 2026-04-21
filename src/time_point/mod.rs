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

/// A high-precision instant in time, **typed by its time scale** ([`ClockType`]).
///
/// `TimePoint` stores a physical moment as **seconds + attoseconds (10⁻¹⁸ s)**
/// measured from the **reference epoch of its own `ClockType`**.
///
/// ### The single most important fact
///
/// For **every built-in clock type except `Proper` and `Custom`**,
/// `TimePoint::new(0, 0, ClockType::XXX)` represents the **exact same physical
/// instant** — the moment that corresponds to **J2000.0 Terrestrial Time**
/// (2000-01-01 12:00:00 TT, JD 2451545.0) when converted to TT.
///
/// Examples:
/// - `new(0, 0, ClockType::TT)` → directly J2000.0 TT
/// - `new(0, 0, ClockType::TAI)` → 32.184 s before J2000 TT
/// - `new(0, 0, ClockType::UTC)` → the UTC instant corresponding to the TAI zero
/// - `new(0, 0, ClockType::GPST)` → 19 s after the TAI zero
/// - `new(0, 0, ClockType::TCG)` → the TCG instant whose rate-corrected value
///   equals J2000 TT (rate integrated from the IAU 1977 reference epoch)
///
/// Only `Proper` and `Custom` have **user-chosen** reference epochs (via
/// `ClockModel`).
///
/// This design gives exact round-tripping and relativistic corrections while
/// keeping numbers small for modern dates. All high-level methods
/// (`to_gregorian_date`, `to_rfc3339*`, formatting, JD/MSD, etc.) convert
/// internally to TT. You almost never need to look at raw `.sec()` unless
/// doing low-level work.
///
/// See the [`ClockType`] module documentation for the exact zero point of
/// every scale.
///
/// - **Precision**: 10⁻¹⁸ s (attosecond)
/// - **Range**: ±~292 billion years (i64 seconds)
/// - **Correctness**: All conversions preserve the exact physical instant
///   using TAI as the canonical hub + proper leap-second and IAU relativistic
///   handling.
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
