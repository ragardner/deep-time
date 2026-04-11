//! Time scale definitions and relativistic support for spacecraft and probes.
//!
//! # Spacecraft / Probe Usage Pattern (Recommended)
//!
//! ```ignore
//! use deep_time_core::{Point, TimePov, TimePoly, TimePolyScale};
//!
//! // Onboard clock is always tagged as Proper time
//! let onboard_tau = Point::from_scale(current_scale);        // or .with_scale(...)
//!
//! // Latest relativistic model received from ground
//! let scale = TimePolyScale::proper(last_contact, current_poly);
//!
//! // One-line conversion to any Earth scale (TT, TDB, UTC, etc.)
//! let tt  = onboard_tau.to_pov_with_scale(scale);
//! let tdb = tt.to_pov(TimePov::TDB);
//!
//! // Going the other direction (e.g. command timestamp in TT)
//! let command_in_tt = /* ... */;
//! let onboard_equivalent = command_in_tt.from_pov_with_scale(scale);
//! ```
//!
//! This pattern gives you:
//! - Fully self-describing proper time (no external state needed)
//! - Exact 36-digit quadratic corrections (velocity, gravity, clock drift)
//! - Zero-cost `const fn` everywhere
//! - Seamless round-tripping between Proper ↔ TT/TCB/etc.

use crate::{Point, TimePoly};
use core::fmt;

/// Enum of the different time **points of view** (POVs) available.
///
/// - `Proper` – relativistic proper time (τ) experienced by a moving observer.
/// - `Custom` – user-defined / arbitrary time scale
#[non_exhaustive]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub enum TimePov {
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
    /// **Proper Time (τ)** – the relativistic proper time experienced by a moving
    /// observer (spacecraft, probe, etc.).  
    /// Onboard clocks run in this scale. Use `to_pov_with_poly(TimePov::TT, …)`  
    /// with a `TimePoly` to convert to Earth coordinate time.
    Proper,
    /// **Custom / user-defined scale** – for experimental or mission-specific timescales.
    /// Most powerful when paired with `TimePolyScale` (self-describing polynomial).
    Custom,
}

impl Default for TimePov {
    /// Default is `TAI`
    fn default() -> Self {
        Self::TAI
    }
}

impl TimePov {
    /// Returns `true` if this time POV accounts for leap seconds.
    pub const fn uses_leap_sec(&self) -> bool {
        matches!(self, Self::UTC)
    }

    /// Returns `true` if this time POV is based off a GNSS constellation.
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
            Self::Proper => "Proper",
            Self::Custom => "Custom",
        }
    }

    /// Returns the reference epoch (zero instant) of this time POV,
    /// expressed as a zero-duration [`Point`] in this exact POV.
    pub const fn reference_epoch(self) -> crate::Point {
        crate::Point::new(0, 0, self)
    }
}

impl fmt::Display for TimePov {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbreviation())
    }
}

/// A fully self-describing relativistic time scale for spacecraft and probes.
///
/// Bundles a base `TimePov` (normally `Proper` or `Custom`) with the quadratic
/// polynomial and reference epoch needed for exact conversion to any other scale
/// (typically TT or TDB).
///
/// This is the recommended way to represent onboard proper time that carries
/// its own clock-drift / relativistic model.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimePolyScale {
    /// Base scale (usually `Proper` or `Custom`)
    pub base: TimePov,
    /// Epoch at which the polynomial was defined (e.g. last ground contact)
    pub reference: Point,
    /// Quadratic correction model (exact 36-digit precision)
    pub poly: TimePoly,
}

impl TimePolyScale {
    /// Creates a new self-describing scale (most common for Proper time).
    #[inline]
    pub const fn new(base: TimePov, reference: Point, poly: TimePoly) -> Self {
        Self {
            base,
            reference,
            poly,
        }
    }

    /// Convenience constructor for a pure Proper-time scale with relativistic correction.
    #[inline]
    pub const fn proper(reference: Point, poly: TimePoly) -> Self {
        Self::new(TimePov::Proper, reference, poly)
    }

    /// Convenience constructor for a custom scale.
    #[inline]
    pub const fn custom(reference: Point, poly: TimePoly) -> Self {
        Self::new(TimePov::Custom, reference, poly)
    }

    /// Attaches this self-describing scale to an existing `Point`.
    ///
    /// Useful when you have a raw onboard reading and the latest polynomial update
    /// from ground control.
    #[inline]
    pub const fn attach_to(self, point: Point) -> Point {
        point.with_pov(self.base)
    }

    /// Convenience: creates a `Point` in this scale from a TAI instant.
    #[inline]
    pub const fn from_tai(self, tai: Point) -> Point {
        tai.with_pov(self.base)
    }
}
