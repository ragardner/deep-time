mod arithmetic;
mod constructors;
mod conversions;
mod conversions_lunar;
mod conversions_mars;
mod decimal_year;
mod from_ccsds;
mod from_gps;
mod from_str;
mod gregorian;
mod julian_date;
mod ops;
mod to_ccsds_bin;
mod to_gps;
mod to_str;

pub mod numbers_traits;
pub mod trajectory;

#[cfg(feature = "alloc")]
mod formatting;
#[cfg(feature = "alloc")]
mod to_ccsds_str;

#[cfg(feature = "hifitime")]
mod hifitime;

#[cfg(feature = "chrono")]
mod chrono;

#[cfg(feature = "jiff")]
mod jiff;

use crate::ATTOS_PER_SEC;
use core::fmt;

/// Dt, and the library, is in the process of being switched from the sec
/// and subsec fields being related to the scale, TO the sec and subsec fields
/// always being TAI Epoch 2000-01-01 noon.
/// Much of the documentation is outdated and should be ignored.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct Dt {
    pub(crate) sec: i64,
    pub(crate) attos: u64,
}

impl Dt {
    /// Seconds field getter.
    #[inline]
    pub const fn sec(&self) -> i64 {
        self.sec
    }

    /// Subseconds field getter (attoseconds).
    #[inline]
    pub const fn attos(&self) -> u64 {
        self.attos
    }

    /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOS_PER_SEC)`.
    #[inline]
    pub const fn carry_over_mut(&mut self) -> &mut Self {
        if self.attos >= ATTOS_PER_SEC {
            self.sec = self.sec.saturating_add((self.attos / ATTOS_PER_SEC) as i64);
            self.attos %= ATTOS_PER_SEC;
        }
        self
    }

    /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOS_PER_SEC)`.
    #[inline]
    pub const fn carry_over(&self) -> Self {
        if self.attos < ATTOS_PER_SEC {
            return *self;
        }
        Self {
            sec: self.sec.saturating_add((self.attos / ATTOS_PER_SEC) as i64),
            attos: self.attos % ATTOS_PER_SEC,
        }
    }
}

impl Default for Dt {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sec = self.sec();
        let attos = self.attos();

        // Default to nanosecond precision (9 digits) — most useful for everyday use
        let precision = f.precision().unwrap_or(9);

        // Respect the `+` sign when the user writes {:+}
        if f.sign_plus() && sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", sec)?;

        if precision > 0 {
            let prec = precision.min(18);
            let scale = 10u64.pow(18 - prec as u32);
            let value = attos / scale;
            write!(f, ".{:0>width$}", value, width = prec)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Dt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dt")
            .field("sec", &self.sec())
            .field("attos", &self.attos())
            .finish()
    }
}

#[cfg(feature = "wire")]
impl Dt {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes (17 bytes).
    pub const WIRE_SIZE: usize = 17;

    /// Serializes this `Dt` into a fixed 17-byte little-endian buffer.
    ///
    /// # Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..9]`: `sec` as little-endian `i64`
    /// - Bytes `[9..17]`: `subsec` as little-endian `u64`
    ///
    /// This format is stable, portable, and suitable for network transmission,
    /// file storage, or FFI. The internal representation is always TAI.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..9].copy_from_slice(&self.sec.to_le_bytes());
        buf[9..17].copy_from_slice(&self.attos.to_le_bytes());
        buf
    }

    /// Deserializes a `Dt` from exactly 17 bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown.
    /// Any `subsec` value ≥ 10¹⁸ is automatically normalized using
    /// [`carry_over`](Self::carry_over) so the resulting `Dt`
    /// is always in canonical form.
    ///
    /// ## Security
    ///
    /// Safe to call with completely untrusted input. Fixed-size format,
    /// no allocation, no `unsafe`, and no possibility of code execution.
    /// Malicious data simply produces a normalized (but still valid) `Dt`.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let sec = i64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        let subsec = u64::from_le_bytes([
            bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16],
        ]);

        Some(Self::new(sec, subsec))
    }
}
