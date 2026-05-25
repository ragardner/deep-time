use core::fmt::{self};
use core::str;

/// Fixed-capacity, stack-only UTF-8 string stored in a single `[u8; N]` array.
///
/// - The string is stored as raw bytes with C-style nul termination.
/// - Its logical length is determined at runtime by the position of the first nul byte (`b'\0'`).
/// - All bytes after the string content are guaranteed to be zero.
/// - The type guarantees that the prefix up to the first nul is always valid UTF-8
///   (when constructed through the safe API).
/// - This type is **intentionally** kept very lightweight. When you use different sizes
///   (such as `LiteStr<16>`, `LiteStr<32>`, `LiteStr<64>`), Rust creates a full
///   separate copy of the type **and all of its methods** for each size. This is
///   why it is important to keep the implementation of `LiteStr` as minimal as possible.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LiteStr<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> Default for LiteStr<N> {
    #[inline(always)]
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const N: usize> LiteStr<N> {
    pub const SIZE: usize = N;

    /// Recommended ergonomic constructor – truncates at a UTF-8 boundary if necessary.
    #[inline(never)]
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; N];
        copy_valid_utf8_prefix(&mut bytes, s.as_bytes(), N);
        Self { bytes }
    }

    /// Returns the stored string as `&str`.
    #[inline(always)]
    pub fn as_str(&self) -> Result<&str, LiteStrErr> {
        str::from_utf8(&self.bytes[..find_first_nul(&self.bytes)])
            .map_err(|_| LiteStrErr::CorruptedData)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LiteStrErr> {
        if bytes.len() != N {
            return Err(LiteStrErr::WrongLen);
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(bytes);
        validate_filled_buffer(&arr)?;
        Ok(Self { bytes: arr })
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; N] {
        self.bytes
    }

    /// Returns the stored string as `&[u8]`.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..find_first_nul(&self.bytes)]
    }

    /// Returns the current len of the utf-8.
    #[allow(clippy::len_without_is_empty)]
    #[inline(always)]
    pub fn len(&self) -> usize {
        find_first_nul(&self.bytes)
    }
}

impl<const N: usize> fmt::Write for LiteStr<N> {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let current = self.len();
        let remaining = N.saturating_sub(current);
        if remaining == 0 {
            return Ok(());
        }

        copy_valid_utf8_prefix(&mut self.bytes[current..], s.as_bytes(), remaining);
        Ok(())
    }
}

impl<const N: usize> fmt::Debug for LiteStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => f.write_str("LiteStr(<invalid utf-8>)"),
        }
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for LiteStr<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for LiteStr<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        Ok(LiteStr::new(s))
    }
}

#[inline(never)]
fn find_first_nul(bytes: &[u8]) -> usize {
    bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
}

#[inline(never)]
fn validate_filled_buffer(bytes: &[u8]) -> Result<(), LiteStrErr> {
    let len = find_first_nul(bytes);

    if str::from_utf8(&bytes[..len]).is_err() {
        return Err(LiteStrErr::InvalidUtf8);
    }

    if len < bytes.len() && bytes[len..].iter().any(|&b| b != 0) {
        return Err(LiteStrErr::CorruptedData);
    }

    Ok(())
}

#[inline(never)]
fn copy_valid_utf8_prefix(dst: &mut [u8], src: &[u8], max_len: usize) -> usize {
    let len = src.len().min(max_len);
    match str::from_utf8(&src[..len]) {
        Ok(_) => {
            dst[..len].copy_from_slice(&src[..len]);
            len
        }
        Err(e) => {
            let valid = e.valid_up_to();
            dst[..valid].copy_from_slice(&src[..valid]);
            valid
        }
    }
}

/// Errors returned by [`LiteStr`] operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteStrErr {
    /// Input was not valid UTF-8.
    WrongLen,
    /// Input was not valid UTF-8.
    InvalidUtf8,
    /// Internal data is corrupted or violates the type invariant.
    CorruptedData,
}

impl fmt::Display for LiteStrErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteStrErr::WrongLen => f.write_str("input len does not match SIZE"),
            LiteStrErr::InvalidUtf8 => f.write_str("input is not valid UTF-8"),
            LiteStrErr::CorruptedData => {
                f.write_str("internal data is corrupted or violates the representation invariant")
            }
        }
    }
}

impl core::error::Error for LiteStrErr {}
