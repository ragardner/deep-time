//! # Time scales and reference epochs — the heart of the library
//!
//! This crate provides a **single `TimePoint` type** that can represent an
//! instant in **any supported time scale** while preserving exact physical
//! meaning across conversions.
//!
//! ## The single most important fact
//!
//! For **every built-in clock type except `Proper` and `Custom`**,
//! `TimePoint::new(0, 0, ClockType::XXX)` represents the **exact same physical
//! instant** — the moment that corresponds to **J2000.0 Terrestrial Time**
//! (2000-01-01 12:00:00 TT, JD 2451545.0) when converted to TT.
//!
//! - `new(0, 0, ClockType::TT)` → directly J2000.0 TT
//! - `new(0, 0, ClockType::TAI)` → 32.184 s before J2000 TT
//! - `new(0, 0, ClockType::UTC)` → the UTC instant corresponding to the TAI zero
//! - `new(0, 0, ClockType::GPST)` → 19 s after the TAI zero
//! - `new(0, 0, ClockType::TCG)` → the TCG instant whose rate-corrected value
//!   equals J2000 TT (the rate `L_G` is integrated from the IAU 1977 reference epoch)
//! - `new(0, 0, ClockType::TCB)` → the TCB instant whose rate-corrected value
//!   equals J2000 TT (the rate `L_B` + `TDB0` is integrated from 1977)
//! - `new(0, 0, ClockType::LTC)` → the LTC instant whose rate-corrected value
//!   equals J2000 TT (the rate `L_M` is integrated from 1977)
//! - `new(0, 0, ClockType::TDB)` → the TDB instant that corresponds to J2000 TT
//!   after the annual sinusoidal correction
//!
//! Only `Proper` and `Custom` have **user-chosen** reference epochs (via
//! `ClockModel`).
//!
//! This is the design that makes perfect round-tripping and exact relativistic
//! math possible while keeping numbers small for modern dates.
//!
//! ## Why J2000.0 TT?
//!
//! J2000.0 TT is the modern standard epoch in astronomy and planetary science
//! (used by virtually all ephemerides, SPICE, DE440, etc.). By anchoring every
//! built-in scale so that its zero maps to J2000 TT, the library achieves:
//! - Small `sec` values for dates in the 21st century
//! - Exact integer `const fn` arithmetic everywhere
//! - Consistent behavior across all time scales
//!
//! The relativistic scales (TCG, TCB, LTC) use the IAU 1977 reference epoch
//! **only** as the starting point for integrating their linear rate (`L_G`,
//! `L_B`, `L_M`). Their actual zero point (i.e. what `new(0,0, ...)` means)
//! is still the instant that corresponds to J2000 TT after the rate correction.
//!
//! ## Exact zero points for every clock type
//!
//! | ClockType | Zero point of `new(0, 0, ClockType::X)` |
//! |-----------|-----------------------------------------|
//! | TT / ET   | 2000-01-01 12:00:00 TT (JD 2451545.0) |
//! | TAI       | 2000-01-01 11:59:27.816 TAI (J2000 TT − 32.184 s) |
//! | UTC       | ~2000-01-01 11:58:55 UTC (via leap-second table) |
//! | GPST/QZSST/GST | 2000-01-01 11:59:46.816 (TAI zero + 19 s) |
//! | BDT       | 2000-01-01 12:00:00.816 (TAI zero + 33 s) |
//! | TDB       | The TDB instant corresponding to J2000 TT |
//! | TCG       | The TCG instant corresponding to J2000 TT (L_G integrated from 1977) |
//! | TCB       | The TCB instant corresponding to J2000 TT (L_B + TDB0 from 1977) |
//! | LTC       | The LTC instant corresponding to J2000 TT (L_M integrated from 1977) |
//! | Proper    | User-chosen (via `ClockModel`) |
//! | Custom    | User-chosen (via `ClockModel`) |
//!
//! See `conversions.rs` for the exact implementation of each mapping.

use crate::TimePoint;
use core::fmt;

/// Enum of the different time systems/clocks available.
///
/// - `Proper` – relativistic proper time (τ) experienced by a moving observer.
/// - `Custom` – user-defined / arbitrary time scale
#[non_exhaustive]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub enum ClockType {
    /// TAI is the representation of an Epoch internally.
    TAI,
    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT)).
    TT,
    /// Ephemeris Time as defined by SPICE (slightly different from true TDB).
    ET,
    /// Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time).
    TDB,
    /// Universal Coordinated Time.
    UTC,
    /// GPS Time scale whose reference epoch is UTC midnight between 05 January and
    /// 06 January 1980.
    GPST,
    /// Galileo Time scale.
    GST,
    /// BeiDou Time scale.
    BDT,
    /// QZSS Time scale has the same properties as GPST but with dedicated clocks.
    QZSST,
    /// **Geocentric Coordinate Time (TCG)** – relativistic coordinate time in the
    /// Geocentric Celestial Reference System (GCRS).
    TCG,
    /// **Barycentric Coordinate Time (TCB)** – relativistic coordinate time in the
    /// Barycentric Celestial Reference System (BCRS).
    TCB,
    /// **Coordinated Lunar Time (LTC)** – NASA’s official lunar coordinate time scale
    /// (analogous to TCG). Defined from the NIST/Ashby & Patla (2024) relativistic
    /// framework adopted for Artemis and cislunar operations.
    ///
    /// Lunar clocks on the selenoid run faster than terrestrial clocks by a
    /// constant secular rate of **+56.02 µs per Earth day** (L_M = 6.48378 × 10^{-10}).
    /// A small additional periodic variation exists due to lunar orbital eccentricity
    /// (±0.108 µs/day in instantaneous rate, ~±0.75 µs accumulated over one orbit).
    /// The periodic term is **not** part of the defining LTC conversion; it is
    /// handled via `ClockModel` / `ClockDrift` when utmost precision is required.
    LTC,
    /// **Proper Time (τ)** – the relativistic proper time experienced by a moving
    /// observer (spacecraft, etc.).  
    /// Onboard clocks run this type. Use `convert_using_drift(ClockType::TT, …)`  
    /// with a `ClockDrift` to convert to Earth coordinate time.
    Proper,
    /// **Custom / user-defined type** – for experimental or mission-specific timescales.
    /// Most powerful when paired with `ClockModel` (self-describing polynomial).
    Custom,
}

impl Default for ClockType {
    /// Default is `TAI`
    fn default() -> Self {
        Self::TAI
    }
}

impl ClockType {
    /// Returns `true` if this clock type accounts for leap seconds.
    pub const fn uses_leap_sec(&self) -> bool {
        matches!(self, Self::UTC)
    }

    /// Returns `true` if this clock type is based off a GNSS constellation.
    pub const fn is_gnss(&self) -> bool {
        matches!(self, Self::GPST | Self::GST | Self::BDT | Self::QZSST)
    }

    /// Short abbreviation used for formatting / display (e.g. "TAI", "UTC", "Proper").
    pub const fn abbreviation(&self) -> &'static str {
        match self {
            Self::TAI => "TAI",
            Self::TT => "TT",
            Self::ET => "ET",
            Self::TDB => "TDB",
            Self::UTC => "UTC",
            Self::TCG => "TCG",
            Self::TCB => "TCB",
            Self::GPST => "GPST",
            Self::GST => "GST",
            Self::BDT => "BDT",
            Self::QZSST => "QZSST",
            Self::LTC => "LTC",
            Self::Proper => "Proper",
            Self::Custom => "Custom",
        }
    }

    /// Returns the reference epoch (zero instant) of this clock type,
    /// expressed as a zero-duration [`TimePoint`] in this exact clock type.
    pub const fn reference_epoch(self) -> TimePoint {
        TimePoint::new(0, 0, self)
    }
}

impl fmt::Display for ClockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbreviation())
    }
}
