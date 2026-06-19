use core::fmt;

/// Time scales supported by the library.
///
/// This `#[non_exhaustive]` enum defines all time scales that [`Dt`] can represent.
/// Each [`Dt`] instance stores its internal time value on the scale indicated by
/// its `scale` field.
///
/// ## UTC and Leap Seconds
///
/// The library supports three UTC variants:
///
/// - **`UTC`** — Modern UTC using the library’s built-in IERS leap second table.
///   Basic functions for run-time loading of leap seconds tables also available.
///
/// - **`UtcSpice`** — SPICE-compatible model. Uses a fixed +9 s offset before
///   1972-01-01 and modern leap seconds afterward.
///
/// - **`UtcHist`** — Full historical SOFA model with piecewise linear frequency
///   offsets ("rubber seconds") from 1961–1972. This variant does **not support
///   round-tripping**.
///
/// ## Scale Categories
///
/// | Category                    | Scales                          | Purpose |
/// |-----------------------------|---------------------------------|---------|
/// | **Atomic / Proper**         | `TAI`, `TT`, `TDB` (alias `ET`) | Continuous atomic time. `TAI` is the primary internal scale. |
/// | **Relativistic Coordinate** | `TCG`, `TCB`, `TCL`             | Time scales defined in specific relativistic reference frames (GCRS, BCRS, LCRS). |
/// | **Civil / Coordinated**     | `UTC`, `UtcSpice`, `UtcHist`    | Real-world civil time, with support for leap seconds and historical behavior. |
/// | **GNSS / Navigation**       | `GPS`, `GST`, `BDT`, `QZSS`     | Time scales used by satellite navigation constellations. |
/// | **Lunar**                   | `LTC`, `TCL`                    | Lunar time scales based on the LTE440 model (Lu et al. 2025). |
/// | **Special**                 | `Custom`                        | User-defined or experimental scales. |
///
/// The reference epoch used for conversions between scales is **2000-01-01 12:00:00 TAI**.
///
/// ## Lunar Time Scales
///
/// Both `LTC` and `TCL` implement the **LTE440** model (Lu et al. 2025):
///
/// - **`LTC`** (Coordinated Lunar Time): Operational lunar time for cislunar operations.
///   Applies a secular rate of **+56.02 µs per Earth day** relative to TT, plus the
///   13 dominant periodic terms from the model.
///
/// - **`TCL`** (Lunar Coordinate Time): IAU-defined relativistic coordinate time
///   in the Lunar Celestial Reference System (LCRS). Includes the secular rate
///   versus TDB, the same periodic terms, and a constant bias calibrated to match
///   the published LTE440 reference value at J2000.0 TDB.
#[non_exhaustive]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum Scale {
    /// International Atomic Time (TAI).
    #[default]
    TAI,

    /// Terrestrial Time (TT).
    ///
    /// A smooth, continuous atomic time scale used in astronomy and dynamics.
    TT,

    /// Ephemeris Time (alias for TDB).
    ET,

    /// Barycentric Dynamical Time (TDB).
    ///
    /// Used for planetary and spacecraft calculations.
    TDB,

    /// Coordinated Universal Time (UTC) using modern leap second rules.
    UTC,

    /// Coordinated Universal Time using the SPICE historical model
    /// (fixed +9 s offset before 1972).
    UtcSpice,

    /// Coordinated Universal Time using the full historical SOFA model
    /// (with "rubber seconds" between 1960–1972).
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
