mod from_ccsds_bin;
mod from_ccsds_str;
mod from_str;
mod to_ccsds_bin;
mod to_deep_time;

#[cfg(feature = "alloc")]
mod to_ccsds_str;

#[cfg(feature = "chrono")]
mod to_chrono;

#[cfg(feature = "jiff")]
mod to_jiff;

use crate::{AsciiStr, ClockType};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimeParts {
    pub year: Option<i64>,
    pub month: Option<u8>,  // 1-12
    pub day: Option<u8>,    // 1-31
    pub hour: Option<u8>,   // 0-23
    pub minute: Option<u8>, // 0-59
    pub second: Option<u8>, // 0-60
    pub attos: Option<u64>, // 0 ≤ value < 10¹⁸
    pub offset: Option<Offset>,
    pub iana_name: Option<AsciiStr<50>>,
    pub is_leap_second: bool,
    pub clock_type: ClockType,
    pub weekday: Option<Weekday>,
    pub day_of_year: Option<u16>,   // 1-366 (%j)
    pub iso_week_year: Option<i64>, // %G / %g
    pub iso_week: Option<u8>,       // 1-53 (%V)
    pub week_sun: Option<u8>,       // 0-53 (%U)
    pub week_mon: Option<u8>,       // 0-53 (%W)
    pub meridiem: Option<Meridiem>,
    pub unix_timestamp_seconds: Option<i64>, // %s
}

impl TimeParts {
    #[inline]
    pub fn new_utc() -> Self {
        Self {
            clock_type: ClockType::UTC,
            ..Default::default()
        }
    }

    /// Sets the IANA timezone name safely.
    ///
    /// Uses `AsciiStr::try_from_str` internally. If the name is non-ASCII
    /// or longer than 50 bytes it is silently dropped (no panics).
    #[inline]
    pub fn set_iana_name(&mut self, name: Option<&str>) -> &mut Self {
        self.iana_name = name.and_then(|s| AsciiStr::try_from_str(s).ok());
        self
    }

    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Total size of the wire representation (121 bytes).
    pub const WIRE_SIZE: usize = 121;

    /// Serializes `TimeParts` into a fixed 121-byte buffer.
    ///
    /// Layout:
    /// - Byte 0: Version (`WIRE_VERSION`)
    /// - Bytes 1..121: Data (120 bytes)
    #[cfg(feature = "wire")]
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let mut offset = 1usize;

        // year (sentinel = i64::MIN)
        let year = self.year.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&year.to_le_bytes());
        offset += 8;

        // month
        buf[offset] = self.month.unwrap_or(u8::MAX);
        offset += 1;

        // day
        buf[offset] = self.day.unwrap_or(u8::MAX);
        offset += 1;

        // hour
        buf[offset] = self.hour.unwrap_or(u8::MAX);
        offset += 1;

        // minute
        buf[offset] = self.minute.unwrap_or(u8::MAX);
        offset += 1;

        // second
        buf[offset] = self.second.unwrap_or(u8::MAX);
        offset += 1;

        // attos
        let attos = self.attos.unwrap_or(u64::MAX);
        buf[offset..offset + 8].copy_from_slice(&attos.to_le_bytes());
        offset += 8;

        // offset (5 bytes)
        let offset_bytes = self.offset.unwrap_or_default().to_wire_bytes();
        buf[offset..offset + 5].copy_from_slice(&offset_bytes);
        offset += 5;

        // iana_name (50 bytes)
        if let Some(name) = &self.iana_name {
            let name_bytes = name.to_wire_bytes();
            buf[offset..offset + 50].copy_from_slice(&name_bytes);
        }
        offset += 50;

        // is_leap_second
        buf[offset] = if self.is_leap_second { 1 } else { 0 };
        offset += 1;

        // clock_type
        buf[offset] = self.clock_type as u8;
        offset += 1;

        // weekday
        buf[offset] = self.weekday.map_or(255, |w| w.to_wire_byte());
        offset += 1;

        // day_of_year
        let doy = self.day_of_year.unwrap_or(u16::MAX);
        buf[offset..offset + 2].copy_from_slice(&doy.to_le_bytes());
        offset += 2;

        // iso_week_year
        let iso_y = self.iso_week_year.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&iso_y.to_le_bytes());
        offset += 8;

        // iso_week
        buf[offset] = self.iso_week.unwrap_or(u8::MAX);
        offset += 1;

        // week_sun
        buf[offset] = self.week_sun.unwrap_or(u8::MAX);
        offset += 1;

        // week_mon
        buf[offset] = self.week_mon.unwrap_or(u8::MAX);
        offset += 1;

        // meridiem
        buf[offset] = self.meridiem.map_or(255, |m| m.to_wire_byte());
        offset += 1;

        // unix_timestamp_seconds
        let unix = self.unix_timestamp_seconds.unwrap_or(i64::MIN);
        buf[offset..offset + 8].copy_from_slice(&unix.to_le_bytes());

        buf
    }

    /// Deserializes `TimeParts` from exactly 121 bytes.
    ///
    /// Returns `None` if the version byte is unknown or the data is invalid.
    #[cfg(feature = "wire")]
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let mut dc = TimeParts::default();
        let mut offset = 1usize;

        // year (8 bytes)
        let year = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if year != i64::MIN {
            dc.year = Some(year);
        }
        offset += 8;

        // month (1 byte)
        let m = bytes[offset];
        if m != u8::MAX {
            dc.month = Some(m);
        }
        offset += 1;

        // day (1 byte)
        let d = bytes[offset];
        if d != u8::MAX {
            dc.day = Some(d);
        }
        offset += 1;

        // hour (1 byte)
        let h = bytes[offset];
        if h != u8::MAX {
            dc.hour = Some(h);
        }
        offset += 1;

        // minute (1 byte)
        let min = bytes[offset];
        if min != u8::MAX {
            dc.minute = Some(min);
        }
        offset += 1;

        // second (1 byte)
        let sec = bytes[offset];
        if sec != u8::MAX {
            dc.second = Some(sec);
        }
        offset += 1;

        // attos (8 bytes)
        let attos = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if attos != u64::MAX {
            dc.attos = Some(attos);
        }
        offset += 8;

        // offset (5 bytes) — already nice
        if let Some(offset) = Offset::from_wire_bytes(&bytes[offset..offset + 5]) {
            dc.offset = Some(offset);
        }
        offset += 5;

        // iana_name (50 bytes) — already nice
        let iana_bytes = &bytes[offset..offset + 50];
        if let Some(name) = AsciiStr::<50>::from_wire_bytes(iana_bytes) {
            if !name.is_empty() {
                dc.iana_name = Some(name);
            }
        }
        offset += 50;

        // is_leap_second (1 byte)
        dc.is_leap_second = bytes[offset] != 0;
        offset += 1;

        // clock_type (1 byte)
        if let Some(ct) = ClockType::from_u8(bytes[offset]) {
            dc.clock_type = ct;
        }
        offset += 1;

        // weekday (1 byte)
        let wd_byte = bytes[offset];
        if wd_byte != 255 {
            if let Some(wd) = Weekday::from_wire_byte(wd_byte) {
                dc.weekday = Some(wd);
            }
        }
        offset += 1;

        // day_of_year (2 bytes)
        let doy = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);
        if doy != u16::MAX {
            dc.day_of_year = Some(doy);
        }
        offset += 2;

        // iso_week_year (8 bytes)
        let iso_y = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if iso_y != i64::MIN {
            dc.iso_week_year = Some(iso_y);
        }
        offset += 8;

        // iso_week (1 byte)
        let iw = bytes[offset];
        if iw != u8::MAX {
            dc.iso_week = Some(iw);
        }
        offset += 1;

        // week_sun (1 byte)
        let ws = bytes[offset];
        if ws != u8::MAX {
            dc.week_sun = Some(ws);
        }
        offset += 1;

        // week_mon (1 byte)
        let wm = bytes[offset];
        if wm != u8::MAX {
            dc.week_mon = Some(wm);
        }
        offset += 1;

        // meridiem (1 byte)
        let mer_byte = bytes[offset];
        if mer_byte != 255 {
            if let Some(m) = Meridiem::from_wire_byte(mer_byte) {
                dc.meridiem = Some(m);
            }
        }
        offset += 1;

        // unix_timestamp_seconds (8 bytes)
        let unix = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        if unix != i64::MIN {
            dc.unix_timestamp_seconds = Some(unix);
        }

        Some(dc)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meridiem {
    #[default]
    AM,
    PM,
}

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
    pub const WIRE_SIZE: usize = 1;

    #[inline]
    pub const fn to_wire_byte(self) -> u8 {
        self.wk_sun()
    }

    #[inline]
    pub const fn from_wire_byte(b: u8) -> Option<Self> {
        Self::from_sunday_zero_offset(b as i8)
    }

    /// Converts a Sunday-based weekday number (0 = Sunday … 6 = Saturday) to `Weekday`.
    #[inline]
    pub const fn from_sunday_zero_offset(n: i8) -> Option<Self> {
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
    #[inline]
    pub const fn from_monday_one_offset(n: i8) -> Option<Self> {
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
    #[inline]
    pub const fn wk_sun(self) -> u8 {
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

    /// Monday-based weekday number (1 = Monday … 7 = Sunday).
    #[inline]
    pub const fn wk_mon(self) -> u8 {
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

impl Offset {
    pub const WIRE_SIZE: usize = 5; // tag (1) + i32 (4)

    #[cfg(feature = "wire")]
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        match self {
            Offset::Utc => buf[0] = 0,
            Offset::None => buf[0] = 1,
            Offset::Fixed(offset) => {
                buf[0] = 2;
                buf[1..5].copy_from_slice(&offset.to_le_bytes());
            }
        }
        buf
    }

    #[cfg(feature = "wire")]
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        match bytes[0] {
            0 => Some(Offset::Utc),
            1 => Some(Offset::None),
            2 => {
                let offset = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Some(Offset::Fixed(offset))
            }
            _ => None,
        }
    }
}
