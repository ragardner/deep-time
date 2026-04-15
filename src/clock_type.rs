//! Time scale definitions and relativistic support for spacecraft.
//!
//! # Spacecraft Usage Pattern
//!
//! ```ignore
//! use deep_time_core::{TimePoint, ClockType, ClockDrift, ClockModel};
//!
//! // Onboard clock is always tagged as Proper time
//! let onboard_tau = TimePoint::create_from_model(current_scale); // or .apply_new_model(...)
//!
//! // Latest relativistic model received from ground
//! let scale = ClockModel::proper(last_contact, current_poly);
//!
//! // One-line conversion to any Earth scale (TT, TDB, UTC, etc.)
//! let tt  = onboard_tau.convert_using_model(scale);
//! let tdb = tt.to_clock_type(ClockType::TDB);
//!
//! // Going the other direction (e.g. command timestamp in TT)
//! let command_in_tt = /* ... */;
//! let onboard_equivalent = command_in_tt.convert_back_using_model(scale);
//! ```
//!
//! This pattern provides:
//! - Fully self-describing proper time (no external state needed)
//! - Exact 36-digit quadratic corrections (velocity, gravity, clock drift)
//! - Zero-cost `const fn` everywhere
//! - Seamless round-tripping between Proper ↔ TT/TCB/etc.

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
