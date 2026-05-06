use crate::ATTOS_PER_SEC;

mod arithmetic;
mod constructors;
mod formatting;
mod from_str;
mod ops;
pub mod time_units;

#[cfg(feature = "alloc")]
mod to_str;

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

/// A high-precision **duration** (time span) expressed as **seconds + attoseconds**
/// (where 1 attosecond = 10⁻¹⁸ s).
///
/// `TimeSpan` is the span counterpart of `TimePoint`. It does **not** carry a [`ClockType`]
/// because durations are scale-independent (they can be added to or subtracted from any
/// `TimePoint` regardless of its scale; any scale-specific adjustments like leap seconds
/// are handled by the `TimePoint` arithmetic).
///
/// - Precision: 10⁻¹⁸ s
/// - Range: ±~292 billion years (i64 seconds limit).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSpan {
    /// Signed whole seconds.
    pub sec: i64,
    /// Fractional part in attoseconds (`0 ≤ attos < 10¹⁸`).
    pub subsec: u64,
}

impl TimeSpan {
    /// Seconds field getter.
    #[inline]
    pub const fn sec(&self) -> i64 {
        self.sec
    }

    /// Subseconds field getter (attoseconds).
    #[inline]
    pub const fn subsec(&self) -> u64 {
        self.subsec
    }

    /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOS_PER_SEC)`.
    #[inline]
    pub const fn carry_over(&mut self) -> &mut Self {
        if self.subsec >= ATTOS_PER_SEC {
            self.sec += (self.subsec / ATTOS_PER_SEC) as i64;
            self.subsec %= ATTOS_PER_SEC;
        }
        self
    }
}

impl Default for TimeSpan {
    fn default() -> Self {
        Self::ZERO
    }
}

#[cfg(feature = "wire")]
impl TimeSpan {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes (17 bytes).
    pub const WIRE_SIZE: usize = 17;

    /// Serializes this `TimeSpan` into a fixed 17-byte little-endian buffer.
    ///
    /// # Wire Format
    ///
    /// - Byte `0`: Version (`WIRE_VERSION`)
    /// - Bytes `[1..9]`: `sec` as little-endian `i64`
    /// - Bytes `[9..17]`: `subsec` as little-endian `u64`
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;
        buf[1..9].copy_from_slice(&self.sec.to_le_bytes());
        buf[9..17].copy_from_slice(&self.subsec.to_le_bytes());
        buf
    }

    /// Deserializes a `TimeSpan` from exactly 17 bytes of wire data.
    ///
    /// Returns `None` if the version byte is unknown.
    /// Any `subsec` value ≥ 10¹⁸ is automatically normalized using
    /// [`carry_over`](Self::carry_over) so the resulting `TimeSpan`
    /// is always in canonical form.
    ///
    /// ## Security
    ///
    /// Safe to call with completely untrusted input. Fixed-size format,
    /// no allocation, no `unsafe`, and no possibility of code execution.
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

        Some(Self::new(sec, subsec))
    }
}
