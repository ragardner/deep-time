//! This pattern provides:
//! - Fully self-describing proper time (no external state needed)
//! - Exact 36-digit quadratic corrections (velocity, gravity, clock drift)
//! - Zero-cost `const fn` everywhere
//! - Seamless round-tripping between Proper ↔ TT/TCB/etc.
//!
//! ## Reference Epochs for Every Clock Type
//!
//! Every `TimePoint` stores its value as **whole seconds + attoseconds (10⁻¹⁸ s)**
//! since a specific "reference epoch" — the exact physical instant when that
//! clock type’s counter reads zero. You can think of it like the odometer on
//! different cars: each clock type has its own starting line. When you write
//! `TimePoint::new(0, 0, ClockType::XXX)` or call `clock_type.reference_epoch()`,
//! you’re asking “what real-world moment does this clock consider its zero point?”
//!
//! This library deliberately anchors almost every clock type near the year 2000
//! (specifically **J2000.0 Terrestrial Time**) instead of using the historical
//! start dates of each scale (1958 for TAI, 1980 for GPS, etc.). The reason is
//! practical: using the old zeros would make the `sec` field for a modern
//! spacecraft around 2.1 trillion — huge numbers that waste bits and slow down
//! math. By resetting the zeros close to J2000.0, numbers stay small and fast
//! while relativistic corrections stay perfectly exact with pure integer `const fn`
//! arithmetic. The single master anchor is:
//!
//! > **2000 January 1, 12:00:00 TT** exactly  
//! > Julian Date **2451545.0 TT**
//!
//! Every other clock type’s zero point is mathematically defined so that
//! converting it to TT (or the reverse) always gives exactly the same physical
//! moment, using the rules implemented in `conversions.rs`.
//!
//! ### TAI — International Atomic Time
//! TAI is the pure, continuous count of cesium-atom vibrations that serves as
//! the hidden “master clock” behind all Earth-based time scales. It has no leap
//! seconds and ignores relativity. Its zero point is defined as exactly 32.184
//! seconds before J2000.0 TT — i.e. **2000-01-01 11:59:27.816 TAI** — via the
//! constant `TT_TAI_OFFSET_DELTA`. All `to_tai()` and `from_tai(ClockType::TT)`
//! conversions simply add or subtract this fixed offset.
//!
//! ### TT (and ET) — Terrestrial Time / Ephemeris Time
//! TT is the smooth, ideal coordinate time that astronomers and spacecraft
//! teams use to predict exactly where planets and probes will be. It runs on
//! Earth’s geoid with no leap seconds or orbital wobbles. Its zero point is
//! literally J2000.0 itself: **2000-01-01 12:00:00 TT** (JD 2451545.0). This
//! is the only scale where `sec == 0` directly corresponds to the J2000 anchor,
//! which is why all Gregorian dates, Julian Day calculations, and ISO week
//! formatting start here. ET is treated as an identical alias for TT for
//! legacy SPICE compatibility.
//!
//! ### UTC — Coordinated Universal Time
//! UTC is the everyday “wall-clock time” you see on phones and watches. It
//! inserts leap seconds every now and then so that noon stays roughly aligned
//! with the Sun overhead. Its zero point is whatever UTC instant converts to
//! the TAI zero above using the library’s leap-second table (`leap_seconds_before`).
//! Around the year 2000 that lands at roughly 11:58:55 UTC (32 seconds behind
//! TAI at that epoch). Only this scale returns `true` from `uses_leap_sec()`.
//!
//! ### TDB — Barycentric Dynamical Time
//! TDB is the high-precision time scale used for the most accurate solar-system
//! ephemerides. It includes tiny periodic corrections caused by Earth’s
//! slightly elliptical orbit around the Sun. Its zero point is defined so that
//! `tai_to_tdb` / `tdb_to_tai` (which evaluate the IAU annual sinusoidal term
//! at J2000 using `tdb_minus_tt`) produce exactly J2000.0 TT. The conversion
//! uses an 8-iteration fixed-point solver around the coefficients 0.001658 and
//! 0.000022 evaluated relative to `J2000_SECONDS_PER_CENTURY`.
//!
//! ### TCG — Geocentric Coordinate Time
//! TCG is the coordinate time experienced by objects in Earth orbit (satellites,
//! ISS, etc.). Because they sit higher in Earth’s gravitational well, their
//! clocks run very slightly faster than clocks on the ground. Its zero point is
//! defined by integrating the exact IAU rate `L_G = 6.969290134 × 10^{-10}`
//! (`LG_NUM / LG_DEN`) from the 1977-01-01.0 TAI reference epoch
//! (`TCG_TCB_REF_JD_INT`, `TCG_TCB_REF_TOD_SEC`, `TCG_TCB_REF_TOD_SUBSEC`).
//! The helpers `elapsed_attos_since_ref` + `mul_lg` do the integration so that
//! converting the TCG zero to TT yields exactly J2000.0 TT.
//!
//! ### TCB — Barycentric Coordinate Time
//! TCB is the coordinate time used for deep-space missions far from Earth
//! (Voyagers, New Horizons, future interstellar probes). It accounts for the
//! Sun’s gravity and the solar-system barycenter. Its zero point is defined
//! using the larger IAU rate `L_B = 1.550519768 × 10^{-8}` (`LB_NUM / LB_DEN`)
//! plus the constant `TDB0_ATTOS = −65.5 µs`, all integrated from the same
//! 1977 reference epoch via `mul_lb` and `elapsed_attos_since_ref`. Converting
//! the TCB zero through the TCB ↔ TDB ↔ TAI chain lands exactly on J2000.0 TT.
//!
//! ### LTC — Coordinated Lunar Time
//! LTC is NASA’s official lunar coordinate time (adopted for Artemis and
//! cislunar operations). Lunar clocks run faster than terrestrial clocks by
//! the secular rate `L_M = 6.48378 × 10^{-10}` because the Moon sits in a
//! weaker gravitational field. Its zero point uses the exact same 1977
//! reference epoch and `elapsed_attos_since_ref` + `mul_lm` machinery as TCG
//! and TCB. (The small periodic variation due to the Moon’s elliptical orbit
//! is intentionally left out of the base conversion; you can add it with
//! `ClockDrift` when you need sub-microsecond precision.)
//!
//! ### GPST, QZSST, GST — GPS Time, QZSS Time, Galileo Time
//! These are the continuous time scales broadcast by the major GNSS
//! constellations. They run 19 seconds behind TAI and never insert leap seconds.
//! Their zero point is exactly 19 seconds after the TAI zero
//! (**2000-01-01 11:59:46.816 TAI**), implemented with the constant
//! `Delta::SEC_19`. The `is_gnss()` method returns `true` for all three.
//!
//! ### BDT — BeiDou Time
//! BeiDou Time is China’s GNSS time scale. It runs 33 seconds behind TAI. Its
//! zero point is exactly 33 seconds after the TAI zero
//! (**2000-01-01 12:00:00.816 TAI**) using the constant `Delta::SEC_33`.
//!
//! ### Proper — Relativistic Proper Time (τ)
//! Proper time is the actual time experienced by a moving spacecraft. Because
//! of velocity and varying gravity, its clock ticks at a different rate than
//! any Earth clock. There is **no fixed universal zero** — you (or your ground
//! team) choose it, usually at launch or the last ground contact. You create
//! a `ClockModel::proper(last_contact, current_poly)` that encodes a constant
//! offset plus linear rate plus quadratic acceleration. Then
//! `convert_using_drift` / `convert_back_using_drift` (or the convenience
//! `convert_using_model`) gives exact round-tripping with at most 16 fixed-point
//! iterations for the inverse.
//!
//! ### Custom — User-defined time scale
//! Custom is a blank slate for experimental or mission-specific timescales.
//! It works exactly like Proper: no fixed zero, fully defined by a `ClockModel`
//! and the same drift-conversion machinery.
//!
//! ### How All the Conversions Work
//!
//! All paths ultimately go through TAI (the library’s common language).
//! `to_tai()` applies the right rule — fixed GNSS offsets, leap-second table,
//! or relativistic integration — and `from_tai()` does the inverse. The
//! relativistic scales (TCG, TCB, LTC) rely on `elapsed_attos_since_ref` plus
//! the exact fixed-point multipliers `mul_lg` / `mul_lb` / `mul_lm`. Proper
//! and Custom apply the quadratic `ClockDrift` polynomial. Because everything
//! is `const fn` and uses only integer arithmetic, you can even compute any
//! reference epoch at compile time.
//!
//! See `constants.rs` for the exact magic numbers (`J2000_JD_TT`, `LG_NUM`,
//! `LM_NUM`, `TCG_TCB_REF_*`, etc.) and `conversions.rs` for the full
//! implementation details. The `gregorian.rs` module then turns any instant
//! into human-readable dates using the TT anchor.

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
