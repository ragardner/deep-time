mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod gregorian;
mod ops;
mod to_canonical;

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

// Full updated content for /home/workdir/attachments/mod.rs
// (only the TimePoint documentation was changed; the rest of the file is unchanged)

/// A high-precision instant in time, **typed by its time scale** ([`ClockType`]).
///
/// `TimePoint` stores a physical moment as **seconds + attoseconds (10⁻¹⁸ s)**
/// measured from the **reference epoch of its own `ClockType`**.
///
/// ### The single most important fact
///
/// For **every built-in clock type except `Proper` and `Custom`**,
/// `TimePoint::new(0, 0, ClockType::XXX)` represents **the exact same physical
/// instant** — **2000-01-01 12:00:00 TAI**.
///
/// Concretely:
/// - `new(0, 0, ClockType::TAI)` → exactly 2000-01-01 12:00:00 TAI
/// - `new(0, 0, ClockType::TT)`  → 2000-01-01 12:00:32.184 TT (J2000.0 TT)
/// - `new(0, 0, ClockType::UTC)` → the UTC instant that corresponds to TAI 2000-01-01 12:00:00
/// - `new(0, 0, ClockType::GPST)` → 19 s after the TAI zero
/// - `new(0, 0, ClockType::TCG)` → the TCG instant that corresponds to the TAI zero
///   (rate `L_G` integrated from the IAU 1977 reference epoch)
///
/// Only `Proper` and `Custom` have **user-chosen** reference epochs (via
/// [`ClockModel`]).
///
/// The library uses **TAI** as the canonical internal hub for all conversions
/// (`to_tai` / `from_tai`). All built-in scales are now anchored at the same
/// physical instant (TAI 2000-01-01 12:00:00) while still preserving perfect
/// round-tripping to the astronomical standard J2000.0 TT via the fixed
/// +32.184 s offset.
///
/// All high-level methods (`to_gregorian_date`, `to_rfc3339*`, formatting,
/// JD/MSD, etc.) automatically convert internally to TT when needed.
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
