//! Intermediate "parts" of a civil date and time.
//!
//! [`Parts`] is produced by the parsers (string formats, ISO-like, CCSDS
//! binary and text, etc.). It is then typically converted into a
//! [`Dt`](../struct.Dt.html) or a type from `chrono`/`jiff`.
//!
//! It holds the individual components (various ways to express the date,
//! time-of-day down to attoseconds, offset, scale, weekday/week info, etc.).

mod from_bin_ccsds;
mod from_str;
mod from_str_iso;
mod to_bin_ccsds;
mod to_deep_time;

#[cfg(feature = "alloc")]
mod to_str_ccsds;

#[cfg(feature = "chrono")]
mod to_chrono;

#[cfg(feature = "jiff")]
mod to_jiff;

use crate::{LiteStr, Scale};

/// Intermediate representation of parsed civil date and time.
///
/// After parsing you typically convert the `Parts` to a final type
/// such as [`Dt`] or one from `chrono`/`jiff`.
///
/// ## Examples
///
/// ```rust
/// use deep_time::civil_parts::Parts;
///
/// let parts = Parts::from_str_iso("2024-06-20T14:30:00Z").unwrap();
///
/// // now you can convert to whichever type you need
/// let dt = parts.to_dt().unwrap();
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Parts {
    /// Year (can be negative for BCE dates).
    pub yr: Option<i64>,
    /// Month of the year (1–12).
    pub mo: Option<u8>,
    /// Day of the month (1–31).
    pub day: Option<u8>,
    /// Hour of the day (0–23).
    pub hr: u8,
    /// Minute of the hour (0–59).
    pub min: u8,
    /// Second of the minute (0–60). Value 60 is used for leap seconds.
    pub sec: u8,
    /// Attoseconds (0 ≤ value < 10¹⁸).
    pub attos: u64,
    /// Timezone offset from UTC.
    pub offset: Option<Offset>,
    /// IANA timezone name (e.g. `"America/New_York"`), stored as ASCII.
    pub iana_name: Option<LiteStr<49>>,
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
    /// Timestamp in seconds since a known epoch (`%s` = Unix 1970, `%J` = noon 2000 / J2000).
    pub timestamp: Option<Timestamp>,
}

/// Raw parsed components from a "seconds + optional fraction" string.
#[derive(Clone, Copy)]
pub(crate) struct SecF {
    pub(crate) negative: bool,
    /// Accumulated absolute integer part (u64::MAX on overflow during accumulation).
    pub(crate) int_u: u64,
    /// Fractional attoseconds, already left-padded to 18 digits.
    pub(crate) frac_attos: u64,
    pub(crate) scale: Scale,
}

impl Parts {
    #[inline(always)]
    pub fn new_utc() -> Parts {
        Self {
            scale: Scale::UTC,
            ..Default::default()
        }
    }

    /// Sets the IANA timezone name.
    #[inline(always)]
    pub fn set_iana_name(&mut self, name: Option<&str>) {
        self.iana_name = name.map(LiteStr::new);
    }
}

/// Used by [`Timestamp`]
///
/// Records the epoch of the timestamp.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Epoch {
    Unix,
    Noon2000,
}

/// Timestamp seconds relative to a specific epoch.
///
/// Used by the `%s` (Unix epoch) and `%J` (J2000.0 noon 2000-01-01 12:00 TAI) directives.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub attos: i128,
    pub epoch: Epoch,
}

/// AM / PM indicator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meridiem {
    #[default]
    AM,
    PM,
}

/// Day of the week. Default is set to Sunday.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
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
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Offset {
    #[default]
    None,
    /// Fixed offset in seconds
    Fixed(i32),
}

#[cfg(feature = "wire")]
impl Meridiem {
    pub const WIRE_SIZE: usize = 1;

    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        match self {
            Meridiem::AM => 0,
            Meridiem::PM => 1,
        }
    }

    #[inline]
    pub const fn from_wire_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Meridiem::AM),
            1 => Some(Meridiem::PM),
            _ => None,
        }
    }
}

#[cfg(feature = "wire")]
impl Offset {
    pub const WIRE_SIZE: usize = 5; // tag (1) + i32 (4)

    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        match self {
            Offset::None => buf[0] = 0,
            Offset::Fixed(offset) => {
                buf[0] = 1;
                buf[1..5].copy_from_slice(&offset.to_le_bytes());
            }
        }
        buf
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        match bytes[0] {
            0 => Some(Offset::None),
            1 => {
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Some(Offset::Fixed(offset))
            }
            _ => None,
        }
    }
}

#[cfg(feature = "wire")]
impl Weekday {
    pub const WIRE_SIZE: usize = 1;

    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        self.wkday_sun_0_based()
    }

    #[inline]
    pub const fn from_wire_byte(b: u8) -> Option<Self> {
        Self::from_sunday_0_based(b)
    }
}

#[cfg(feature = "wire")]
impl Parts {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Total size of the wire representation (120 bytes).
    /// The timestamp field now uses 17 bytes (tag + i128), using some of the previous slack space.
    pub const WIRE_SIZE: usize = 120;

    /// Serializes `Parts` into a fixed 120-byte buffer.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let mut offset = 1usize;

        // year (sentinel = i64::MIN)
        let year = self.yr.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&year.to_le_bytes());
        offset += 8;

        // month
        buf[offset] = self.mo.unwrap_or(u8::MAX);
        offset += 1;

        // day
        buf[offset] = self.day.unwrap_or(u8::MAX);
        offset += 1;

        // hour
        buf[offset] = self.hr;
        offset += 1;

        // minute
        buf[offset] = self.min;
        offset += 1;

        // second
        buf[offset] = self.sec;
        offset += 1;

        // attos
        let attos = self.attos;
        buf[offset..offset + 8].copy_from_slice(&attos.to_le_bytes());
        offset += 8;

        // offset (5 bytes)
        let offset_bytes = self.offset.unwrap_or_default().to_wire_bytes();
        buf[offset..offset + 5].copy_from_slice(&offset_bytes);
        offset += 5;

        // iana_name (49 bytes)
        if let Some(name) = &self.iana_name {
            let name_bytes = name.bytes;
            buf[offset..offset + 49].copy_from_slice(&name_bytes);
        }
        offset += 49;

        // scale
        buf[offset] = self.scale as u8;
        offset += 1;

        // weekday
        buf[offset] = self.wkday.map_or(255, |w| w.to_wire_byte());
        offset += 1;

        // day_of_year
        let doy = self.day_of_yr.unwrap_or(u16::MAX);
        buf[offset..offset + 2].copy_from_slice(&doy.to_le_bytes());
        offset += 2;

        // iso_week_year
        let iso_y = self.iso_wk_yr.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&iso_y.to_le_bytes());
        offset += 8;

        // iso_week
        buf[offset] = self.iso_wk.unwrap_or(u8::MAX);
        offset += 1;

        // week_sun
        buf[offset] = self.wk_sun.unwrap_or(u8::MAX);
        offset += 1;

        // week_mon
        buf[offset] = self.wk_mon.unwrap_or(u8::MAX);
        offset += 1;

        // meridiem
        buf[offset] = self.meridiem.map_or(255, |m| m.to_wire_byte());
        offset += 1;

        // timestamp: tag (1 byte) + i128 attos (16 bytes) = 17 bytes total
        // tag: 0 = none, 1 = Unix, 2 = Noon2000
        let (tag, attos) = match self.timestamp {
            None => (0u8, 0i128),
            Some(ts) => {
                let t = match ts.epoch {
                    Epoch::Unix => 1u8,
                    Epoch::Noon2000 => 2u8,
                };
                (t, ts.attos)
            }
        };
        buf[offset] = tag;
        offset += 1;
        buf[offset..offset + 16].copy_from_slice(&attos.to_le_bytes());
        // offset += 16;

        buf
    }

    /// Deserializes `Parts` from exactly `WIRE_SIZE` bytes.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let mut dc = Parts::default();
        let mut offset = 1usize;

        // year (8 bytes)
        let year = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if year != i64::MIN {
            dc.yr = Some(year);
        }
        offset += 8;

        // month (1 byte)
        let m = bytes[offset];
        if m != u8::MAX {
            dc.mo = Some(m);
        }
        offset += 1;

        // day (1 byte)
        let d = bytes[offset];
        if d != u8::MAX {
            dc.day = Some(d);
        }
        offset += 1;

        // hour (1 byte)
        dc.hr = bytes[offset];
        offset += 1;

        // minute (1 byte)
        dc.min = bytes[offset];
        offset += 1;

        // second (1 byte)
        dc.sec = bytes[offset];
        offset += 1;

        // attos (8 bytes)
        let attos = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        dc.attos = attos;
        offset += 8;

        // offset (5 bytes)
        if let Some(off) = Offset::from_wire_bytes(&bytes[offset..offset + 5]) {
            dc.offset = Some(off);
        }
        offset += 5;

        // iana_name (49 bytes)
        let iana_bytes = &bytes[offset..offset + 49];
        let name = LiteStr::<49>::from_bytes(iana_bytes);
        if !name.as_bytes().is_empty() {
            dc.iana_name = Some(name);
        }
        offset += 49;

        // scale (1 byte)
        dc.scale = Scale::from_u8(bytes[offset]);
        offset += 1;

        // weekday (1 byte)
        let wd_byte = bytes[offset];
        if wd_byte != 255
            && let Some(wd) = Weekday::from_wire_byte(wd_byte)
        {
            dc.wkday = Some(wd);
        }
        offset += 1;

        // day_of_year (2 bytes)
        let doy = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);
        if doy != u16::MAX {
            dc.day_of_yr = Some(doy);
        }
        offset += 2;

        // iso_week_year (8 bytes)
        let iso_y = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if iso_y != i64::MIN {
            dc.iso_wk_yr = Some(iso_y);
        }
        offset += 8;

        // iso_week (1 byte)
        let iw = bytes[offset];
        if iw != u8::MAX {
            dc.iso_wk = Some(iw);
        }
        offset += 1;

        // week_sun (1 byte)
        let ws = bytes[offset];
        if ws != u8::MAX {
            dc.wk_sun = Some(ws);
        }
        offset += 1;

        // week_mon (1 byte)
        let wm = bytes[offset];
        if wm != u8::MAX {
            dc.wk_mon = Some(wm);
        }
        offset += 1;

        // meridiem (1 byte)
        let mer_byte = bytes[offset];
        if mer_byte != 255
            && let Some(m) = Meridiem::from_wire_byte(mer_byte)
        {
            dc.meridiem = Some(m);
        }
        offset += 1;

        // timestamp: tag (1) + i128 attos (16)
        // tag: 0=none, 1=Unix, 2=Noon2000
        let tag = bytes[offset];
        offset += 1;

        if tag != 0 {
            let attos_arr: [u8; 16] = bytes[offset..offset + 16].try_into().ok()?;
            let attos = i128::from_le_bytes(attos_arr);
            // offset += 16;

            let epoch = match tag {
                1 => Epoch::Unix,
                2 => Epoch::Noon2000,
                _ => return None,
            };
            dc.timestamp = Some(Timestamp { attos, epoch });
        } else {
            // offset += 16;
        }

        Some(dc)
    }
}
