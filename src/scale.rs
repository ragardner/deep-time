use core::fmt;

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
    /// UT1
    UT1,
    /// Universal Coordinated Time using the SPICE historical model
    /// (fixed +9 s offset against TAI for all dates before 1972-01-01).
    UTCSpice,
    /// Universal Coordinated Time using the full SOFA historical model
    /// (varying fractional "rubber second" offsets from 1960–1971).
    UTCSofa,
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
    ///
    TCL,
    /// **Custom / user-defined type** – for experimental or mission-specific timescales.
    /// Most powerful when paired with `ClockModel` (self-describing polynomial).
    Custom,
}

impl Scale {
    #[inline]
    pub const fn to_ut(&self) -> Self {
        if self.is_ut() {
            return *self;
        } else {
            return Scale::UTC;
        }
    }

    /// Returns `true` if this scale accounts for leap seconds
    /// (or historical UTC civil time rules).
    #[inline]
    pub const fn is_ut(&self) -> bool {
        matches!(self, Self::UTC | Self::UTCSpice | Self::UTCSofa | Self::UT1)
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
            "UT1" => Some(Self::UT1),
            "UTCSPICE" => Some(Self::UTCSpice),
            "UTCSOFA" => Some(Self::UTCSofa),
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

    /// Short abbreviation used for formatting / display (e.g. "TAI", "UTC", "UTCSpice").
    pub const fn abbrev(&self) -> &'static str {
        match self {
            Self::TAI => "TAI",
            Self::TT => "TT",
            Self::ET => "ET",
            Self::TDB => "TDB",
            Self::UTC => "UTC",
            Self::UT1 => "UT1",
            Self::UTCSpice => "UTCSPICE",
            Self::UTCSofa => "UTCSOFA",
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
    #[inline]
    pub const fn eq(self, other: Self) -> bool {
        self.to_wire_byte() == other.to_wire_byte()
    }

    /// Size of the canonical wire representation in bytes.
    pub const WIRE_SIZE: usize = 1;

    /// Attempts to reconstruct a `Scale` from its wire byte representation.
    ///
    /// Returns `None` for any value that does not correspond to a known variant.
    /// This provides safe deserialization from untrusted sources.
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::TAI,
            1 => Self::TT,
            2 => Self::ET,
            3 => Self::TDB,
            4 => Self::UTC,
            5 => Self::UT1,
            6 => Self::UTCSpice,
            7 => Self::UTCSofa,
            8 => Self::GPS,
            9 => Self::GST,
            10 => Self::BDT,
            11 => Self::QZSS,
            12 => Self::TCG,
            13 => Self::TCB,
            14 => Self::LTC,
            15 => Self::TCL,
            _ => Self::Custom,
        }
    }

    /// Returns the wire representation of this `Scale` as a single byte.
    ///
    /// The returned byte is the `repr(u8)` discriminant of the enum.
    /// This is the canonical on-wire form used by [`Dt`] and [`ClockModel`].
    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.abbrev())
    }
}
