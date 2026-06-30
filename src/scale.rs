use core::fmt;

/// Time scales supported by the library.
///
/// This `#[non_exhaustive]` enum defines all time scales that [`Dt`](struct.Dt.html) can represent.
/// Each [`Dt`](struct.Dt.html) instance stores its internal time value on the scale indicated by
/// its `scale` field.
///
/// The reference epoch used for conversions between scales is **2000-01-01 12:00:00 TAI**.
///
/// ## UTC Variants and Leap Seconds
///
/// The library supports three UTC variants:
///
/// - **`UTC`** — Modern UTC using the built-in IERS leap second table (recommended for most uses).
/// - **`UtcSpice`** — SPICE-compatible model with a fixed +9 s offset before 1972-01-01.
/// - **`UtcHist`** — Historical SOFA model with piecewise linear offsets (“rubber seconds”) from 1961–1972.
///   Round-tripping is **not supported** for this variant.
///
/// ## Supported Time Scales
///
/// | Scale       | Description |
/// |-------------|-------------|
/// | `TAI`       | International Atomic Time. The primary internal continuous atomic time scale. |
/// | `TT`        | Terrestrial Time. Smooth atomic time used in astronomy and dynamics (TAI + 32.184 s). |
/// | `ET`        | Ephemeris Time using the **NAIF/SPICE simplified model** (~30 µs accuracy). Matches NASA/NAIF SPICE for interoperability. Use `TDB` for higher-fidelity. |
/// | `TDB`       | Barycentric Dynamical Time. High-fidelity relativistic ephemeris time (DE440/LTE440 + VSOP2013 tuned model). |
/// | `UTC`       | Coordinated Universal Time using modern IERS leap second rules. |
/// | `UtcSpice`  | Coordinated Universal Time using the SPICE historical model (fixed +9 s offset before 1972-01-01). |
/// | `UtcHist`   | Coordinated Universal Time using the historical SOFA model with “rubber seconds” (1961–1972). Round-tripping is not supported. |
/// | `GPS`       | GPS Time (used by the U.S. GPS navigation constellation). |
/// | `GST`       | Galileo Time (used by Europe’s Galileo navigation system). |
/// | `BDT`       | BeiDou Time (used by China’s BeiDou navigation system). |
/// | `QZSS`      | QZSS Time (used by Japan’s QZSS satellite system). |
/// | `TCG`       | Geocentric Coordinate Time. Relativistic time scale in the GCRS (Earth-centered). |
/// | `TCB`       | Barycentric Coordinate Time. Relativistic time scale in the BCRS (solar-system barycenter). |
/// | `LTC`       | Coordinated Lunar Time. Operational lunar time for cislunar use based on the LTE440 model. |
/// | `TCL`       | Lunar Coordinate Time. IAU relativistic coordinate time in the LCRS based on the LTE440 model. |
/// | `Custom`    | Custom time scale. Can be useful when a user doesn't want to use TAI but wants similar behavior in conversion functions. |
///
/// ## Lunar Time Scales (LTC / TCL)
///
/// Both `LTC` and `TCL` are based on the **LTE440** model (Lu et al. 2025):
///
/// - `LTC` (Coordinated Lunar Time) — Intended for operational cislunar use. Applies a secular rate of **+56.02 µs/day** relative to TT plus the dominant periodic terms.
/// - `TCL` (Lunar Coordinate Time) — Theoretical IAU relativistic coordinate time at the Moon’s center of mass. Includes the secular rate versus TDB, periodic terms, and a J2000 bias calibrated to published LTE440 values.
#[non_exhaustive]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Scale {
    /// International Atomic Time (TAI).
    #[default]
    TAI,

    /// Terrestrial Time (TT).
    ///
    /// A smooth, continuous atomic time scale used in astronomy and dynamics
    /// (TAI + 32.184 s constant offset).
    TT,

    /// Ephemeris Time (NAIF/SPICE simplified model).
    ///
    /// Uses the official NAIF simplified single-term model for interoperability
    /// with NASA/NAIF SPICE (~30 µs accuracy). For higher-fidelity relativistic
    /// ephemeris calculations, use [`TDB`](Scale::TDB) instead.
    ET,

    /// Barycentric Dynamical Time (TDB).
    ///
    /// High-fidelity relativistic ephemeris time tuned to DE440/LTE440 + VSOP2013.
    /// Used for precise planetary and spacecraft trajectory calculations.
    TDB,

    /// Coordinated Universal Time (UTC) using modern leap second rules.
    UTC,

    /// Coordinated Universal Time using the SPICE historical model
    /// (fixed +9 s offset before 1972-01-01).
    UtcSpice,

    /// Coordinated Universal Time using the historical SOFA model
    /// (with "rubber seconds" between 1961–1972).
    ///
    /// Round-tripping is not supported.
    UtcHist,

    /// GPS Time.
    ///
    /// The time scale used by the U.S. GPS satellite navigation system.
    GPS,

    /// Galileo Time.
    ///
    /// The time scale used by Europe’s Galileo satellite navigation system.
    GST,

    /// BeiDou Time.
    ///
    /// The time scale used by China’s BeiDou satellite navigation system.
    BDT,

    /// QZSS Time.
    ///
    /// The time scale used by Japan’s QZSS satellite system (similar to GPS).
    QZSS,

    /// Geocentric Coordinate Time (TCG).
    ///
    /// A relativistic time scale centered on Earth, used for high-precision
    /// work near Earth (e.g. satellite orbits).
    TCG,

    /// Barycentric Coordinate Time (TCB).
    ///
    /// A relativistic time scale for the entire solar system.
    TCB,

    /// Coordinated Lunar Time (LTC).
    ///
    /// Operational lunar time scale intended for cislunar operations.
    /// Based on the LTE440 model.
    LTC,

    /// Lunar Coordinate Time (TCL).
    ///
    /// Theoretical relativistic coordinate time at the Moon’s center of mass.
    /// Based on the LTE440 model.
    TCL,

    /// Custom / user-defined scale.
    ///
    /// Can be useful when a user doesn't want to use TAI, and instead wants their own
    /// time scale to mess about with.
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
        let mut buf = [0u8; 8];
        let mut len = 0;

        for &byte in bytes {
            if len >= 8 || !byte.is_ascii_alphabetic() {
                break;
            }
            buf[len] = byte.to_ascii_uppercase();
            len += 1;
        }

        match &buf[..len] {
            b"TAI" => Some(Self::TAI),
            b"TT" => Some(Self::TT),
            b"ET" => Some(Self::ET),
            b"TDB" => Some(Self::TDB),
            b"UTC" => Some(Self::UTC),
            b"UTCSPICE" => Some(Self::UtcSpice),
            b"UTCHIST" => Some(Self::UtcHist),
            b"GPS" => Some(Self::GPS),
            b"GST" => Some(Self::GST),
            b"BDT" => Some(Self::BDT),
            b"QZSS" => Some(Self::QZSS),
            b"TCG" => Some(Self::TCG),
            b"TCB" => Some(Self::TCB),
            b"LTC" => Some(Self::LTC),
            b"TCL" => Some(Self::TCL),
            b"CUSTOM" => Some(Self::Custom),
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
    /// This is the canonical on-wire form used by [`Dt`](struct.Dt.html).
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
