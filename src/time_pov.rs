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
    /// Proper Time (relativistic proper time, often denoted τ).
    Proper,
    /// A user-defined / custom time scale.
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
