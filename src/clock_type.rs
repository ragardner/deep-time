//! # Time scales and reference epochs — the heart of the library
//!
//! This crate provides a **single `TimePoint` type** that can represent an
//! instant in **any supported time scale** while preserving exact physical
//! meaning across conversions.
//!
//! ## The single most important fact
//!
//! For **every built-in clock type except `Proper` and `Custom`**,
//! `TimePoint::new(0, 0, ClockType::XXX)` represents **the exact same physical
//! instant** — **2000-01-01 12:00:00 TAI**.
//!
//! - `new(0, 0, ClockType::TAI)` → exactly 2000-01-01 12:00:00 TAI
//! - `new(0, 0, ClockType::TT)`  → 2000-01-01 12:00:32.184 TT (J2000.0 TT)
//! - `new(0, 0, ClockType::UTC)` → the UTC instant corresponding to TAI 2000-01-01 12:00:00
//! - `new(0, 0, ClockType::GPS)` → 19 s after the TAI zero
//! - `new(0, 0, ClockType::TCG)` → the TCG instant that corresponds to the TAI zero
//!   (rate `L_G` integrated from 1977)
//! - `new(0, 0, ClockType::TCB)` → the TCB instant that corresponds to the TAI zero
//!   (rate `L_B` + `TDB0` integrated from 1977)
//! - `new(0, 0, ClockType::LTC)` → the LTC instant that corresponds to the TAI zero
//!   (rate `L_M` integrated from 1977)
//!
//! Only `Proper` and `Custom` have **user-chosen** reference epochs (via
//! `ClockModel`).
//!
//! ## Why TAI 2000-01-01 12:00:00 as the common anchor?
//!
//! TAI is the canonical internal hub used by all `to_tai`/`from_tai` conversions.
//! Anchoring every built-in scale at this exact TAI instant makes the zero point
//! simple and intuitive for engineering, GNSS, and most practical use cases
//! while still giving perfect round-tripping to the astronomical standard
//! J2000.0 TT (via the fixed +32.184 s TT–TAI offset).
//!
//! The relativistic scales (TCG, TCB, LTC) still integrate their linear rates
//! from the IAU 1977 reference epoch internally — only the storage zero point
//! is now aligned to TAI 2000-01-01 12:00:00.
//!
//! ## Exact zero points for every clock type
//!
//! | ClockType          | Zero point of `new(0, 0, ClockType::X)`                                      |
//! |--------------------|----------------------------------------------------------------------------------|
//! | TAI                | 2000-01-01 12:00:00 TAI                                                          |
//! | TT / ET            | 2000-01-01 12:00:32.184 TT (J2000.0 TT)                                         |
//! | UTC                | UTC instant corresponding to TAI 2000-01-01 12:00:00                             |
//! | GPS/QZSS/GST       | 2000-01-01 12:00:19 TAI (TAI zero + 19 s)                                       |
//! | BDT                | 2000-01-01 12:00:33 TAI (TAI zero + 33 s)                                       |
//! | TDB                | The TDB instant corresponding to the TAI zero                                    |
//! | TCG                | The TCG instant corresponding to the TAI zero (L_G integrated from 1977)        |
//! | TCB                | The TCB instant corresponding to the TAI zero (L_B + TDB0 integrated from 1977) |
//! | LTC                | The LTC instant corresponding to the TAI zero (L_M integrated from 1977)        |
//! | Proper             | User-chosen (via `ClockModel`)                                                   |
//! | Custom             | User-chosen (via `ClockModel`)                                                   |
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
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub enum ClockType {
    /// TAI is the representation of an Epoch internally.
    #[default]
    TAI,
    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT)).
    TT,
    /// Ephemeris Time as defined by NASA/NAIF SPICE (identical to TDB).
    ET,
    /// Barycentric Dynamical Time (TDB) — SPICE ephemeris time (ET is an alias for this).
    TDB,
    /// Universal Coordinated Time.
    UTC,
    /// GPS Time scale whose reference epoch is UTC midnight between 05 January and
    /// 06 January 1980.
    GPS,
    /// Galileo Time scale.
    GST,
    /// BeiDou Time scale.
    BDT,
    /// QZSS Time scale has the same properties as GPS but with dedicated clocks.
    QZSS,
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

impl ClockType {
    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 1;

    /// Returns the wire representation of this `ClockType` as a single byte.
    ///
    /// The returned byte is the `repr(u8)` discriminant of the enum.
    /// This is the canonical on-wire form used by [`TimePoint`] and [`ClockModel`].
    #[inline(always)]
    pub const fn to_wire_byte(self) -> u8 {
        self as u8
    }

    /// Attempts to reconstruct a `ClockType` from its wire byte representation.
    ///
    /// Returns `None` for any value that does not correspond to a known variant.
    /// This provides safe deserialization from untrusted sources.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::TAI),
            1 => Some(Self::TT),
            2 => Some(Self::ET),
            3 => Some(Self::TDB),
            4 => Some(Self::UTC),
            5 => Some(Self::GPS),
            6 => Some(Self::GST),
            7 => Some(Self::BDT),
            8 => Some(Self::QZSS),
            9 => Some(Self::TCG),
            10 => Some(Self::TCB),
            11 => Some(Self::LTC),
            12 => Some(Self::Proper),
            13 => Some(Self::Custom),
            _ => None,
        }
    }

    /// Returns `true` if this clock type accounts for leap seconds.
    pub const fn uses_leap_sec(&self) -> bool {
        matches!(self, Self::UTC)
    }

    /// Returns `true` if this clock type is based off a GNSS constellation.
    pub const fn is_gnss(&self) -> bool {
        matches!(self, Self::GPS | Self::GST | Self::BDT | Self::QZSS)
    }

    /// Parse clock type from abbreviation.
    /// Returns `None` for any non-ASCII input.
    #[inline]
    pub fn from_abbrev(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        // Reject non-ASCII input immediately (clock abbreviations must be ASCII)
        if !bytes.is_ascii() {
            return None;
        }
        // Safe to uppercase now
        let mut buf = [0u8; 8];
        let mut len = 0;
        for &byte in bytes {
            if len >= 8 {
                return None;
            }
            buf[len] = if byte.is_ascii_lowercase() {
                byte - 32
            } else {
                byte
            };
            len += 1;
        }
        let upper = core::str::from_utf8(&buf[..len]).ok()?;
        match upper {
            "TAI" => Some(Self::TAI),
            "TT" => Some(Self::TT),
            "ET" => Some(Self::ET),
            "TDB" => Some(Self::TDB),
            "UTC" => Some(Self::UTC),
            "GPS" => Some(Self::GPS),
            "GST" => Some(Self::GST),
            "BDT" => Some(Self::BDT),
            "QZSS" => Some(Self::QZSS),
            "TCG" => Some(Self::TCG),
            "TCB" => Some(Self::TCB),
            "LTC" => Some(Self::LTC),
            "PROPER" => Some(Self::Proper),
            "CUSTOM" => Some(Self::Custom),
            _ => None,
        }
    }

    /// Short abbreviation used for formatting / display (e.g. "TAI", "UTC", "Proper").
    pub const fn abbrev(&self) -> &'static str {
        match self {
            Self::TAI => "TAI",
            Self::TT => "TT",
            Self::ET => "ET",
            Self::TDB => "TDB",
            Self::UTC => "UTC",
            Self::TCG => "TCG",
            Self::TCB => "TCB",
            Self::GPS => "GPS",
            Self::GST => "GST",
            Self::BDT => "BDT",
            Self::QZSS => "QZSS",
            Self::LTC => "LTC",
            Self::Proper => "PROPER",
            Self::Custom => "CUSTOM",
        }
    }

    /// Const-friendly equality comparison (does **not** rely on `==` for the enum itself).
    ///
    /// Uses the underlying `u8` wire representation (via the already-existing `to_wire_byte`
    /// method). This works reliably in `const fn` contexts even on older Rust versions where
    /// the derived `PartialEq::eq` might not be `const fn` yet.
    ///
    /// Zero-cost and guaranteed to match the `repr(u8)` layout.
    #[inline(always)]
    pub const fn eq(self, other: Self) -> bool {
        self.to_wire_byte() == other.to_wire_byte()
    }

    /// Returns the reference epoch (zero instant) of this clock type,
    /// expressed as a zero-duration [`TimePoint`] in this exact clock type.
    pub const fn reference_epoch(self) -> TimePoint {
        TimePoint::new(0, 0, self)
    }
}

impl fmt::Display for ClockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbrev())
    }
}
