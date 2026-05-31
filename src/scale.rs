use core::fmt;

/// Time scales supported for conversions.
///
/// This `#[non_exhaustive]` enum defines the complete set of time scales used by
/// the library for representing instants [`Dt`] and performing conversions
/// between them.
///
/// It covers atomic, dynamical, coordinate, civil/coordinated, GNSS, and emerging
/// lunar scales, plus a `Custom` variant for mission-specific or experimental use.
///
/// ## Overview
///
/// Time scales fall into several broad categories:
///
/// - **Atomic / proper time scales**: TAI (basis), TT, TDB/ET — continuous and
///   suitable for internal representation and dynamical modeling.
/// - **Coordinate time scales** (relativistic): TCG, TCB, **TCL** — defined in
///   specific reference frames (GCRS, BCRS, LCRS). Ideal for ephemeris
///   integration and high-accuracy modeling; not directly realized by clocks.
/// - **Coordinated / civil scales**: UTC (atomic time with leap seconds inserted
///   to keep it close to UT1), **UT1** (observed Earth rotation angle — does **not**
///   use leap seconds), and the lunar operational scale **LTC** (uses defined
///   secular rate offsets for traceability and cislunar operations).
/// - **GNSS / navigation scales**: GPS, GST, BDT, QZSS — tied to specific
///   satellite constellations.
/// - **Custom**: Fallback for custom scales.
///
/// The library's epoch when performing conversions between all scales is
/// 2000-01-01 noon.
///
/// ## Lunar Time Scales (LTC and TCL)
///
/// The library provides high-accuracy implementations of both lunar time scales
/// based on the **LTE440** model (Lu et al. 2025, A&A 704, A76):
///
/// - [`LTC`] (Coordinated Lunar Time): Applies the secular rate offset
///   (`L_M ≈ +56.02 µs/day`) **plus** the 13 dominant periodic terms from LTE440.
///   Conversions use fixed-point iteration for numerical stability.
///   Achieves sub-nanosecond accuracy (< 0.15 ns before 2050) when the periodic
///   terms are included.
/// - [`TCL`] (Lunar Coordinate Time): IAU-defined relativistic coordinate time
///   in the LCRS. The implementation includes the secular rate vs TDB, the same
///   LTE440 periodic terms, and a constant bias calibrated so that the model
///   reproduces the official LTE440 reference value at J2000.0 TDB.
///   Inverse conversion also uses fixed-point iteration.
///
/// See the documentation on the individual variants for rates, historical
/// models, and conversion notes.
///
/// ## Features
///
/// - `serde` — full serialization/deserialization support.
/// - `js` — TypeScript definitions via `tsify`.
///
/// ## Non-exhaustive
///
/// The enum is marked `#[non_exhaustive]` so new scales can be added in
/// future minor versions without breaking changes.
#[non_exhaustive]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub enum Scale {
    /// TAI is the representation of an Epoch internally.
    #[default]
    TAI,

    /// Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT)).
    TT,

    /// Ephemeris Time as defined by NASA/NAIF SPICE (identical to TDB).
    ET,

    /// Barycentric Dynamical Time (TDB) — SPICE ephemeris time (ET is an alias for this).
    TDB,

    /// Universal Coordinated Time using modern IERS leap second rules.
    UTC,

    /// Universal Coordinated Time using the SPICE historical model
    /// (fixed +9 s offset against TAI for all dates before 1972-01-01).
    /// Includes modern leap seconds for dates after 1972.
    UtcSpice,

    /// Universal Coordinated Time using the full SOFA historical model
    /// (varying fractional "rubber second" offsets from 1960–1972).
    /// Includes modern leap seconds for dates after 1972.
    ///
    /// Round tripping is not possible with this time scale, only convert
    /// once.
    UtcHist,

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

    /// **Coordinated Lunar Time (LTC)** – NASA’s operational lunar time scale
    /// for Artemis and cislunar operations (based on the NIST/Ashby & Patla
    /// relativistic framework).
    ///
    /// Implements the full **LTE440** model (Lu et al. 2025):
    /// - Secular rate: **+56.02 µs per Earth day** (`L_M = 6.48378 × 10^{-10}`)
    ///   relative to terrestrial time.
    /// - Plus the 13 dominant periodic terms (> 1 µs amplitude) from the LTE440
    ///   ephemeris.
    LTC,

    /// **Lunar Coordinate Time (TCL)** – IAU-defined (2024 Resolution II)
    /// relativistic coordinate time in the Lunar Celestial Reference System (LCRS).
    ///
    /// Directly analogous to **TCG**. This is the theoretical coordinate time
    /// at the Moon’s center of mass.
    ///
    /// The implementation follows the **LTE440** model (Lu et al. 2025):
    /// - Secular rate vs TDB (`L_D^M`).
    /// - The same 13-term LTE440 periodic series used for LTC.
    /// - A constant bias (`TCL_TDB_BIAS_SPAN`) calibrated so the model
    ///   reproduces the published LTE440 reference value at J2000.0 TDB.
    TCL,

    /// Custom / user-defined type.
    Custom,
}

impl Scale {
    /// Returns `true` if this scale is TAI.
    #[inline]
    pub const fn is_tai(&self) -> bool {
        matches!(self, Self::TAI)
    }

    /// Converts this [`Scale`] to UTC.
    /// - If the scale is already one of the UTC variants
    ///   including historical UTC then no change occurs.
    #[inline]
    pub const fn to_utc(&self) -> Scale {
        if self.uses_leap_seconds() {
            *self
        } else {
            Scale::UTC
        }
    }

    /// Returns `true` if this scale accounts for leap seconds
    /// (or historical UTC civil time rules).
    #[inline]
    pub const fn uses_leap_seconds(&self) -> bool {
        matches!(self, Self::UTC | Self::UtcSpice | Self::UtcHist)
    }

    /// Returns `true` if this scale is based off a GNSS constellation.
    #[inline]
    pub const fn is_gnss(&self) -> bool {
        matches!(self, Self::GPS | Self::GST | Self::BDT | Self::QZSS)
    }

    /// Parse scale from abbreviation.
    /// Returns `None` for any non-ASCII input.
    pub fn from_abbrev(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if !bytes.is_ascii() {
            return None;
        }
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
            "UTCSPICE" => Some(Self::UtcSpice),
            "UTCHIST" => Some(Self::UtcHist),
            "GPS" => Some(Self::GPS),
            "GST" => Some(Self::GST),
            "BDT" => Some(Self::BDT),
            "QZSS" => Some(Self::QZSS),
            "TCG" => Some(Self::TCG),
            "TCB" => Some(Self::TCB),
            "LTC" => Some(Self::LTC),
            "TCL" => Some(Self::TCL),
            "CUSTOM" => Some(Self::Custom),
            _ => None,
        }
    }

    /// Short abbreviation used for formatting / display (e.g. "TAI", "UTC", "UtcSpice").
    pub const fn abbrev(&self) -> &'static str {
        match self {
            Self::TAI => "TAI",
            Self::TT => "TT",
            Self::ET => "ET",
            Self::TDB => "TDB",
            Self::UTC => "UTC",
            Self::UtcSpice => "UTCSPICE",
            Self::UtcHist => "UTCHIST",
            Self::TCG => "TCG",
            Self::TCB => "TCB",
            Self::GPS => "GPS",
            Self::GST => "GST",
            Self::BDT => "BDT",
            Self::QZSS => "QZSS",
            Self::LTC => "LTC",
            Self::TCL => "TCL",
            Self::Custom => "CUSTOM",
        }
    }

    /// Const-friendly equality comparison.
    #[inline(always)]
    pub const fn eq(self, other: Self) -> bool {
        self.to_u8() == other.to_u8()
    }

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 1;

    /// Attempts to reconstruct a `Scale` from its wire byte representation.
    ///
    /// - Returns `Custom` for any value that does not correspond to a known variant.
    /// - This provides safe deserialization from untrusted sources.
    pub const fn from_u8(v: u8) -> Scale {
        match v {
            0 => Self::TAI,
            1 => Self::TT,
            2 => Self::ET,
            3 => Self::TDB,
            4 => Self::UTC,
            5 => Self::UtcSpice,
            6 => Self::UtcHist,
            7 => Self::GPS,
            8 => Self::GST,
            9 => Self::BDT,
            10 => Self::QZSS,
            11 => Self::TCG,
            12 => Self::TCB,
            13 => Self::LTC,
            14 => Self::TCL,
            _ => Self::Custom,
        }
    }

    /// Returns the wire representation of this `Scale` as a single byte.
    ///
    /// The returned byte is the `repr(u8)` discriminant of the enum.
    /// This is the canonical on-wire form used by [`Dt`].
    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbrev())
    }
}
