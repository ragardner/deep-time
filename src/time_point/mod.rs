mod arithmetic;
mod constructors;
mod conversions;
mod formatting;
mod from_canonical;
mod from_ccsds;
mod from_gps;
mod gregorian;
mod gregorian_historical;
mod ops;
mod to_canonical;
mod to_ccsds_bin;
mod to_ccsds_str;
mod to_gps;
mod to_str;
mod trajectory;

#[cfg(feature = "hifitime")]
mod from_hifitime;
#[cfg(feature = "hifitime")]
mod to_hifitime;

#[cfg(feature = "chrono")]
mod from_chrono;
#[cfg(feature = "chrono")]
mod to_chrono;

#[cfg(feature = "jiff")]
mod from_jiff;
#[cfg(feature = "jiff")]
mod to_jiff;

use crate::ClockType;

/// A high-precision instant in time, **typed by its time scale** ([`ClockType`]).
///
/// `TimePoint` stores a physical moment as **seconds + attoseconds (10⁻¹⁸ s)**
/// measured from the **reference epoch of its own `ClockType`**.
///
/// ### The single most important fact
///
/// For **every built-in clock type except `Proper` and `Custom`**,
/// `TimePoint::new(0, 0, ClockType::XXX)` represents **the exact same physical
/// instant** — **2000-01-01 12:00:00 TAI**.
///
/// Concretely:
/// - `new(0, 0, ClockType::TAI)` → exactly 2000-01-01 12:00:00 TAI
/// - `new(0, 0, ClockType::TT)`  → 2000-01-01 12:00:32.184 TT (J2000.0 TT)
/// - `new(0, 0, ClockType::UTC)` → the UTC instant that corresponds to TAI 2000-01-01 12:00:00
/// - `new(0, 0, ClockType::GPST)` → 19 s after the TAI zero
/// - `new(0, 0, ClockType::TCG)` → the TCG instant that corresponds to the TAI zero
///   (rate `L_G` integrated from the IAU 1977 reference epoch)
///
/// Only `Proper` and `Custom` have **user-chosen** reference epochs (via
/// [`ClockModel`]).
///
/// The library uses **TAI** as the canonical internal hub for all conversions
/// (`to_tai` / `from_tai`). All built-in scales are now anchored at the same
/// physical instant (TAI 2000-01-01 12:00:00) while still preserving perfect
/// round-tripping to the astronomical standard J2000.0 TT via the fixed
/// +32.184 s offset.
///
/// All high-level methods (`to_gregorian_date`, `to_rfc3339*`, formatting,
/// JD/MSD, etc.) automatically convert internally to TT when needed.
///
/// See the [`ClockType`] module documentation for the exact zero point of
/// every scale.
///
/// - **Precision**: 10⁻¹⁸ s (attosecond)
/// - **Range**: ±~292 billion years (i64 seconds)
/// - **Correctness**: All conversions preserve the exact physical instant
///   using TAI as the canonical hub + proper leap-second and IAU relativistic
///   handling.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
pub struct TimePoint {
    /// Signed whole seconds since the reference epoch of the clock_type.
    pub(crate) sec: i64,
    /// Fractional part in attoseconds (`0 ≤ attos < 10¹⁸`).
    pub(crate) subsec: u64,
    /// The time scale this instant belongs to.
    pub(crate) clock_type: ClockType,
}

impl TimePoint {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes (18 bytes).
    pub const WIRE_SIZE: usize = 18;

    /// Serializes this `TimePoint` into a fixed 18-byte little-endian buffer.
    ///
    /// # Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..9]`: `sec` as little-endian `i64`
    /// - Bytes `[9..17]`: `subsec` as little-endian `u64`
    /// - Byte `17`: `ClockType` as `u8`
    ///
    /// This format is stable, portable, and suitable for network transmission,
    /// file storage, or FFI.
    #[inline]
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..9].copy_from_slice(&self.sec.to_le_bytes());
        buf[9..17].copy_from_slice(&self.subsec.to_le_bytes());
        buf[17] = self.clock_type as u8;
        buf
    }

    /// Deserializes a `TimePoint` from exactly 18 bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown.
    /// Any `subsec` value ≥ 10¹⁸ is automatically normalized using
    /// [`carry_over`](Self::carry_over) so the resulting `TimePoint`
    /// is always in canonical form.
    ///
    /// ## Security
    ///
    /// Safe to call with completely untrusted input. Fixed-size format,
    /// no allocation, no `unsafe`, and no possibility of code execution.
    /// Malicious data simply produces a normalized (but still valid) `TimePoint`.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        // Version check for future compatibility
        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let sec = i64::from_le_bytes([
            bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        ]);
        let subsec = u64::from_le_bytes([
            bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15], bytes[16],
        ]);
        let clock_type = ClockType::from_u8(bytes[17])?;

        Some(Self::new(sec, subsec, clock_type))
    }

    #[inline(always)]
    pub const fn clock_type(&self) -> ClockType {
        self.clock_type
    }
}

impl Default for TimePoint {
    fn default() -> Self {
        Self::ZERO
    }
}
