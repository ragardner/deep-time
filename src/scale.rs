use core::fmt;

/// Time scales supported by the library.
///
/// This `#[non_exhaustive]` enum defines all time scales that [`Dt`](crate::Dt) can represent.
/// Each [`Dt`](crate::Dt) instance stores its internal time value on the scale indicated by
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
/// | `LTC`       | Mean-selenoid lunar time: TCL scaled by \(L_m\) (like TT from TCG). ~+56 µs/day vs TT. Not a finalized international LTC standard. |
/// | `TCL`       | IAU Lunar Coordinate Time (approx.): \(L_D^M\) vs TDB from 1977 + 13-term series. Not the full LTE440 product. |
/// | `Custom`    | Custom time scale. Can be useful when a user doesn't want to use TAI but wants similar behavior in conversion functions. |
///
/// ## Lunar Time Scales (LTC / TCL)
///
/// - **`TCL`** — IAU LCRS coordinate time at the Moon’s center of mass. This crate
///   uses \(L_D^M\) (LTE440 rate) from the 1977 epoch plus the published 13-term
///   Fourier sketch of TCL−TDB. That is **not** the full LTE440 Chebyshev kernel.
/// - **`LTC`** — `LTC = TCL − L_m·(TCL − t₀)` with Ashby & Patla (2024) \(L_m\)
///   (same pattern as TT from TCG). Mean rate vs TT ≈ +56.02 µs/day. The name
///   matches common “coordinated lunar time” language but is **not** a claim to
///   implement a finished multi-agency LTC standard.
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
    /// ephemeris calculations, use [`TDB`](Scale::TDB).
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

    /// Mean-selenoid lunar time (library `LTC`).
    ///
    /// TCL scaled by \(L_m\) (Ashby & Patla 2024), like TT from TCG.
    /// Mean rate vs TT ≈ +56.02 µs/day. Not a finalized international LTC.
    LTC,

    /// Lunar Coordinate Time (TCL).
    ///
    /// IAU LCRS coordinate time at the Moon’s center of mass. Approximate
    /// model: \(L_D^M\) vs TDB from 1977 plus a 13-term series (not full LTE440).
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

    /// Parse scale from an ASCII abbreviation (e.g. `b"TAI"`, `b"UtcSpice"`).
    ///
    /// Reads up to 8 leading alphabetic bytes, case-insensitively. Stops at the
    /// first non-letter. Returns `None` if no known scale matches.
    pub fn from_abbrev(bytes: &[u8]) -> Option<Self> {
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

    /// Reconstructs a [`Scale`] from its single-byte wire form.
    ///
    /// Always succeeds. Known values map to the matching variant; any other
    /// byte becomes [`Scale::Custom`]. Safe for untrusted input.
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
    /// This is the canonical on-wire form used by [`Dt`](crate::Dt)
    /// (`0` = TAI, `1` = TT, … — the enum’s `repr(u8)` order).
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
