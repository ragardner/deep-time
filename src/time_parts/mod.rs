mod from_bin_ccsds;
mod from_str;
mod from_str_ccsds;
mod to_bin_ccsds;
mod to_deep_time;

#[cfg(feature = "alloc")]
mod to_str_ccsds;

#[cfg(feature = "chrono")]
mod to_chrono;

#[cfg(feature = "jiff")]
mod to_jiff;

use crate::{LiteStr, Scale};

/// A flexible, partially-filled representation of a civil datetime.
///
/// [`TimeParts`] is the central intermediate type used throughout the library
/// for parsing, formatting, and converting between different time representations
/// (CCSDS, ISO-like strings, `chrono`, `jiff`, `Dt`, etc.).
///
/// Most fields are optional, allowing partial dates/times. It also carries
/// metadata such as the time `scale`, IANA zone name, leap-second flag,
/// and various weekday/week-number representations.
///
/// - Convert to [`Dt`] using [`TimeParts::to_dt`].
/// - Conversions to types from other crates require relevant features to
///   be enabled.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimeParts {
    /// Year (can be negative for BCE dates).
    pub yr: Option<i64>,
    /// Month of the year (1–12).
    pub mo: Option<u8>,
    /// Day of the month (1–31).
    pub day: Option<u8>,
    /// Hour of the day (0–23).
    pub hr: Option<u8>,
    /// Minute of the hour (0–59).
    pub min: Option<u8>,
    /// Second of the minute (0–60). Value 60 is used for leap seconds.
    pub sec: Option<u8>,
    /// Attoseconds (0 ≤ value < 10¹⁸).
    pub attos: Option<u64>,
    /// Timezone offset from UTC.
    pub offset: Option<Offset>,
    /// IANA timezone name (e.g. `"America/New_York"`), stored as ASCII.
    pub iana_name: Option<LiteStr<49>>,
    /// Whether this instant represents a leap second.
    pub is_leap_sec: bool,
    /// The time scale this value belongs to (TAI, UTC, etc.).
    pub scale: Scale,
    /// Day of the week.
    pub wkday: Option<Weekday>,
    /// Day of the year (1–366), corresponding to `%j`.
    pub day_of_yr: Option<u16>,
    /// ISO week year (`%G` / `%g`).
    pub iso_wk_yr: Option<i64>,
    /// ISO week number (1–53), corresponding to `%V`.
    pub iso_wk: Option<u8>,
    /// Week number with Sunday as first day of week (0–53), `%U`.
    pub wk_sun: Option<u8>,
    /// Week number with Monday as first day of week (0–53), `%W`.
    pub wk_mon: Option<u8>,
    /// AM / PM indicator.
    pub meridiem: Option<Meridiem>,
    /// Unix timestamp in seconds (`%s`).
    pub unix_timestamp_seconds: Option<i64>,
}

impl TimeParts {
    #[inline]
    pub fn new_utc() -> TimeParts {
        Self {
            scale: Scale::UTC,
            ..Default::default()
        }
    }

    /// Sets the IANA timezone name.
    #[inline]
    pub fn set_iana_name(&mut self, name: Option<&str>) -> &mut Self {
        self.iana_name = name.map(LiteStr::new);
        self
    }
}

/// AM / PM indicator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meridiem {
    #[default]
    AM,
    PM,
}

/// Day of the week. Default is set to Sunday.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weekday {
    #[default]
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    /// Converts a Sunday-based weekday number (0 = Sunday … 6 = Saturday) to `Weekday`.
    pub const fn from_sunday_0_based(n: u8) -> Option<Self> {
        match n {
            0 => Some(Weekday::Sunday),
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            _ => None,
        }
    }

    /// Converts a Monday-based weekday number (1 = Monday … 7 = Sunday) to `Weekday`.
    pub const fn from_monday_1_based(n: u8) -> Option<Self> {
        match n {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }

    /// Sunday-based weekday number (0 = Sunday … 6 = Saturday).
    pub const fn wkday_sun_0_based(self) -> u8 {
        match self {
            Weekday::Sunday => 0,
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
        }
    }

    /// Sunday-based weekday number (1 = Sunday … 7 = Saturday).
    pub const fn wkday_sun_1_based(self) -> u8 {
        match self {
            Weekday::Sunday => 1,
            Weekday::Monday => 2,
            Weekday::Tuesday => 3,
            Weekday::Wednesday => 4,
            Weekday::Thursday => 5,
            Weekday::Friday => 6,
            Weekday::Saturday => 7,
        }
    }

    /// Monday-based weekday number (0 = Monday … 6 = Sunday).
    pub const fn wkday_mon_0_based(self) -> u8 {
        match self {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }

    /// Monday-based weekday number (1 = Monday … 7 = Sunday).
    pub const fn wkday_mon_1_based(self) -> u8 {
        match self {
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
            Weekday::Sunday => 7,
        }
    }
}

/// Timezone offset representation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Offset {
    #[default]
    Utc,
    None,
    /// Fixed offset from UTC in seconds
    Fixed(i32),
}
