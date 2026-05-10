use crate::{AsciiStr, Dt, JD_2000_2_451_545, SEC_PER_DAYI64, Scale, Weekday};

mod to_str;

/// Combined Gregorian date + wall time with subsecond precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct YmdHms {
    pub yr: i64,
    pub mo: u8,
    pub day: u8,
    pub hr: u8,
    pub min: u8,
    pub sec: u8,    // 0–60 (60 only during leap seconds)
    pub attos: u64, // attoseconds (0 ≤ subsec < 10¹⁸)
}

/// UTC Civil calendar and time-of-day components of a `Dt`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GregorianTime {
    /// UNIX attoseconds counting from 1970 epoch
    pub(crate) unix_attosec: i128,
    /// Gregorian year (proleptic Gregorian calendar, supports negative years and year 0).
    pub(crate) yr: i64,
    /// Gregorian month in the range [1, 12].
    pub(crate) mo: u8,
    /// Gregorian day of the month in the range [1, 31].
    pub(crate) day: u8,
    /// Hour of the day in the range [0, 23].
    pub(crate) hr: u8,
    /// Minute in the range [0, 59].
    pub(crate) min: u8,
    /// Second in the range [0, 60] (60 only during UTC leap seconds).
    pub(crate) sec: u8,
    /// Fractional part of the second expressed in attoseconds (u64).
    pub(crate) attos: u64,
    /// ISO 8601 week year.
    pub(crate) iso_yr: i64,
    /// ISO 8601 week number in the range [1, 53].
    pub(crate) iso_wk: u8,
    /// ISO 8601 weekday enum e.g. Monday/Tuesday/...
    pub(crate) iso_wkday: Weekday,
    /// Ordinal day of the year (1-based).
    pub(crate) day_of_yr: u16,
    /// Weekday number (0 = Sunday … 6 = Saturday).
    pub(crate) wkday: u8,
    /// Sunday based week of year (Range: `0..=53`).
    pub(crate) wk_of_yr_sun: u8,
    /// Monday based week of year (Range: `0..=53`).
    pub(crate) wk_of_yr_mon: u8,
    /// Used for formatting (strftime).
    /// A stored offset in seconds, used within the crate.
    pub(crate) offset_sec: Option<i32>,
    /// A stored IANA name, used within the crate, %Q.
    pub(crate) tz: Option<AsciiStr<49>>,
    /// UTC, EST, %Z
    pub(crate) tz_abbrev: Option<AsciiStr<49>>,
}

impl GregorianTime {
    /// Creates a new `GregorianTime` with all fields specified.
    /// This isn't the recommended way to make a `GregorianTime`.
    /// It's safer to use `Dt::to_gregorian_time()`.
    #[inline]
    pub const fn new(
        unix_attosec: i128,
        yr: i64,
        mo: u8,
        day: u8,
        hr: u8,
        min: u8,
        sec: u8,
        attos: u64,
        iso_yr: i64,
        iso_wk: u8,
        iso_wkday: Weekday,
        day_of_yr: u16,
        wkday: u8,
        wk_of_yr_sun: u8,
        wk_of_yr_mon: u8,
    ) -> Self {
        Self {
            unix_attosec,
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos,
            iso_yr,
            iso_wk,
            iso_wkday,
            day_of_yr,
            wkday,
            wk_of_yr_sun,
            wk_of_yr_mon,
            offset_sec: None,
            tz: None,
            tz_abbrev: None,
        }
    }

    /// UNIX attoseconds since 1970 epoch
    #[inline]
    pub const fn unix_attosec(&self) -> i128 {
        self.unix_attosec
    }

    /// Returns the Unix timestamp since 1970-01-01 00:00:00 UTC as a tuple of
    /// `(whole_seconds, attoseconds)`.
    ///
    /// - `whole_seconds` can be negative (for dates before 1970).
    /// - The fractional part (`attoseconds`) is always in the range `0..=999_999_999_999_999_999`.
    #[inline]
    pub const fn unix_timestamp(&self) -> (i64, u64) {
        const ATTOS_PER_SEC_I128: i128 = 1_000_000_000_000_000_000;
        let total = self.unix_attosec;
        let secs = (total / ATTOS_PER_SEC_I128) as i64;
        let frac = (total % ATTOS_PER_SEC_I128).unsigned_abs() as u64;
        (secs, frac)
    }

    /// Gregorian year (proleptic Gregorian calendar, supports negative years and year 0).
    #[inline]
    pub const fn yr(&self) -> i64 {
        self.yr
    }

    /// Gregorian month in the range [1, 12].
    #[inline]
    pub const fn mo(&self) -> u8 {
        self.mo
    }

    /// Gregorian day of the month in the range [1, 31].
    #[inline]
    pub const fn day(&self) -> u8 {
        self.day
    }

    /// Hour of the day in the range [0, 23].
    #[inline]
    pub const fn hr(&self) -> u8 {
        self.hr
    }

    /// Minute in the range [0, 59].
    #[inline]
    pub const fn min(&self) -> u8 {
        self.min
    }

    /// Second in the range [0, 60] (60 only during UTC leap seconds).
    #[inline]
    pub const fn sec(&self) -> u8 {
        self.sec
    }

    /// Fractional part of the second expressed in attoseconds (`0 ≤ attos < 10¹⁸`).
    #[inline]
    pub const fn attos(&self) -> u64 {
        self.attos
    }

    /// ISO 8601 week year.
    #[inline]
    pub const fn iso_yr(&self) -> i64 {
        self.iso_yr
    }

    /// ISO 8601 week number in the range [1, 53].
    #[inline]
    pub const fn iso_wk(&self) -> u8 {
        self.iso_wk
    }

    /// ISO 8601 weekday (Monday-based [`Weekday`] enum).
    #[inline]
    pub const fn iso_wkday(&self) -> Weekday {
        self.iso_wkday
    }

    /// Ordinal day of the year (1-based).
    #[inline]
    pub const fn day_of_yr(&self) -> u16 {
        self.day_of_yr
    }

    /// Weekday number (0 = Sunday … 6 = Saturday).
    #[inline]
    pub const fn wkday_sun(&self) -> u8 {
        self.wkday
    }

    /// ISO 8601 weekday (0 = Monday ... 6 = Sunday).
    #[inline]
    pub const fn wkday_mon(&self) -> u8 {
        self.iso_wkday.wk_mon()
    }

    /// Sunday based week of year (Range: `0..=53`).
    #[inline]
    pub const fn wk_of_yr_sun(&self) -> u8 {
        self.wk_of_yr_sun
    }

    /// Monday based week of year (Range: `0..=53`).
    #[inline]
    pub const fn wk_of_yr_mon(&self) -> u8 {
        self.wk_of_yr_mon
    }

    #[inline]
    pub const fn offset_sec(&self) -> Option<i32> {
        self.offset_sec
    }

    #[inline]
    pub const fn tz(&self) -> Option<&AsciiStr<49>> {
        self.tz.as_ref()
    }

    #[inline]
    pub const fn tz_abbrev(&self) -> Option<&AsciiStr<49>> {
        self.tz_abbrev.as_ref()
    }

    #[inline]
    pub(crate) fn set_offset(&mut self, offset_sec: Option<i32>) -> &mut Self {
        self.offset_sec = offset_sec;
        self
    }

    #[inline]
    pub(crate) fn set_tz(&mut self, tz: Option<&str>) -> &mut Self {
        self.tz = tz.and_then(|s| AsciiStr::try_from_str(s).ok());
        self
    }

    #[inline]
    pub(crate) fn set_tz_abbrev(&mut self, tz_abbrev: Option<&str>) -> &mut Self {
        self.tz_abbrev = tz_abbrev.and_then(|s| AsciiStr::try_from_str(s).ok());
        self
    }

    /// Reconstructs a [`Dt`] from these **UTC** civil components.
    ///
    /// Round-tripping with `Dt::to_gregorian_time`.
    pub const fn to_time_point(self) -> Dt {
        let jdn = Dt::ymd_to_jdn(self.yr, self.mo, self.day);
        let days_since_j2000 = jdn - JD_2000_2_451_545;
        let seconds_from_noon =
            (self.hr as i64 - 12) * 3600i64 + (self.min as i64) * 60i64 + (self.sec as i64);
        let sec = days_since_j2000 * SEC_PER_DAYI64 + seconds_from_noon;
        Dt::from(sec, self.attos, Scale::UTC)

        // Dt::from_ymdhms(
        //     self.yr, self.mo, self.day, self.hr, self.min, self.sec, 0, scale,
        // )
    }
}

#[cfg(feature = "wire")]
impl GregorianTime {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes (158 bytes).
    pub const WIRE_SIZE: usize = 158;

    /// Serializes this `GregorianTime` into a fixed 158-byte buffer.
    ///
    /// # Wire Format (Version 1)
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `1..17`: `unix_attosec` (`i128`)
    /// - Bytes `17..25`: `yr` (`i64`)
    /// - Bytes `25..30`: `mo`, `day`, `hr`, `min`, `sec` (`u8` × 5)
    /// - Bytes `30..38`: `attos` (`u64`)
    /// - Bytes `38..46`: `iso_yr` (`i64`)
    /// - Bytes `46..48`: `iso_wk` + `iso_wkday` (`u8` × 2)
    /// - Bytes `48..50`: `day_of_yr` (`u16`)
    /// - Byte `50`: `wkday` (`u8`)
    /// - Bytes `51..53`: `wk_of_yr_sun` + `wk_of_yr_mon` (`u8` × 2)
    /// - Bytes `53..58`: `offset_sec` (tag byte + `i32`)
    /// - Bytes `58..108`: `tz` (tag byte + `AsciiStr<49>`)
    /// - Bytes `108..158`: `tz_abbrev` (tag byte + `AsciiStr<49>`)
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        let mut offset = 1usize;

        // unix_attosec (16 bytes)
        buf[offset..offset + 16].copy_from_slice(&self.unix_attosec.to_le_bytes());
        offset += 16;

        // yr (8 bytes)
        buf[offset..offset + 8].copy_from_slice(&self.yr.to_le_bytes());
        offset += 8;

        // mo, day, hr, min, sec (5 bytes)
        buf[offset] = self.mo;
        offset += 1;
        buf[offset] = self.day;
        offset += 1;
        buf[offset] = self.hr;
        offset += 1;
        buf[offset] = self.min;
        offset += 1;
        buf[offset] = self.sec;
        offset += 1;

        // attos (8 bytes)
        buf[offset..offset + 8].copy_from_slice(&self.attos.to_le_bytes());
        offset += 8;

        // iso_yr (8 bytes)
        buf[offset..offset + 8].copy_from_slice(&self.iso_yr.to_le_bytes());
        offset += 8;

        // iso_wk + iso_wkday (2 bytes)
        buf[offset] = self.iso_wk;
        offset += 1;
        buf[offset] = self.iso_wkday.to_wire_byte();
        offset += 1;

        // day_of_yr (2 bytes)
        buf[offset..offset + 2].copy_from_slice(&self.day_of_yr.to_le_bytes());
        offset += 2;

        // wkday (1 byte)
        buf[offset] = self.wkday;
        offset += 1;

        // wk_of_yr_sun + wk_of_yr_mon (2 bytes)
        buf[offset] = self.wk_of_yr_sun;
        offset += 1;
        buf[offset] = self.wk_of_yr_mon;
        offset += 1;

        // offset_sec (Option<i32>) — 5 bytes
        if let Some(val) = self.offset_sec {
            buf[offset] = 1;
            buf[offset + 1..offset + 5].copy_from_slice(&val.to_le_bytes());
        } else {
            buf[offset] = 0;
        }
        offset += 5;

        // tz (Option<AsciiStr<49>>) — 50 bytes
        if let Some(tz) = &self.tz {
            buf[offset] = 1;
            let tz_bytes = tz.to_wire_bytes();
            buf[offset + 1..offset + 1 + AsciiStr::<49>::WIRE_SIZE].copy_from_slice(&tz_bytes);
        } else {
            buf[offset] = 0;
        }
        offset += 1 + AsciiStr::<49>::WIRE_SIZE;

        // tz_abbrev (Option<AsciiStr<49>>) — 50 bytes
        if let Some(abbrev) = &self.tz_abbrev {
            buf[offset] = 1;
            let abbrev_bytes = abbrev.to_wire_bytes();
            buf[offset + 1..offset + 1 + AsciiStr::<49>::WIRE_SIZE].copy_from_slice(&abbrev_bytes);
        } else {
            buf[offset] = 0;
        }

        buf
    }

    /// Deserializes a `GregorianTime` from exactly 158 bytes of wire data.
    ///
    /// Returns `None` if the version is unknown or any field is invalid.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed-size format with strict validation.
    /// No allocation or `unsafe` code used.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let mut offset = 1usize;

        // unix_attosec (16 bytes)
        let unix_attosec = i128::from_le_bytes(bytes[offset..offset + 16].try_into().ok()?);
        offset += 16;

        // yr (8 bytes)
        let yr = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // mo, day, hr, min, sec (5 bytes)
        let mo = bytes[offset];
        offset += 1;
        let day = bytes[offset];
        offset += 1;
        let hr = bytes[offset];
        offset += 1;
        let min = bytes[offset];
        offset += 1;
        let sec = bytes[offset];
        offset += 1;

        // attos (8 bytes)
        let attos = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // iso_yr (8 bytes)
        let iso_yr = i64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;

        // iso_wk + iso_wkday (2 bytes)
        let iso_wk = bytes[offset];
        offset += 1;
        let iso_wkday = Weekday::from_wire_byte(bytes[offset])?;
        offset += 1;

        // day_of_yr (2 bytes)
        let day_of_yr = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?);
        offset += 2;

        // wkday (1 byte)
        let wkday = bytes[offset];
        offset += 1;

        // wk_of_yr_sun + wk_of_yr_mon (2 bytes)
        let wk_of_yr_sun = bytes[offset];
        offset += 1;
        let wk_of_yr_mon = bytes[offset];
        offset += 1;

        // offset_sec (Option<i32>) — 5 bytes
        let offset_sec = if bytes[offset] == 1 {
            Some(i32::from_le_bytes(
                bytes[offset + 1..offset + 5].try_into().ok()?,
            ))
        } else {
            None
        };
        offset += 5;

        // tz (Option<AsciiStr<49>>) — 50 bytes
        let tz = if bytes[offset] == 1 {
            AsciiStr::<49>::from_wire_bytes(
                &bytes[offset + 1..offset + 1 + AsciiStr::<49>::WIRE_SIZE],
            )
        } else {
            None
        };
        offset += 1 + AsciiStr::<49>::WIRE_SIZE;

        // tz_abbrev (Option<AsciiStr<49>>) — 50 bytes
        let tz_abbrev = if bytes[offset] == 1 {
            AsciiStr::<49>::from_wire_bytes(
                &bytes[offset + 1..offset + 1 + AsciiStr::<49>::WIRE_SIZE],
            )
        } else {
            None
        };

        Some(Self {
            unix_attosec,
            yr,
            mo,
            day,
            hr,
            min,
            sec,
            attos,
            iso_yr,
            iso_wk,
            iso_wkday,
            day_of_yr,
            wkday,
            wk_of_yr_sun,
            wk_of_yr_mon,
            offset_sec,
            tz,
            tz_abbrev,
        })
    }
}
