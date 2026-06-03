use core::fmt::{self};
use core::str;

/// Fixed-capacity, stack-only buffer holding a UTF-8 string in a `[u8; N]` array.
///
/// - The string is stored as raw bytes with C-style nul termination.
/// - Logical length is the position of the first `b'\0'` (or `N` if there is none).
/// - This type performs **no validation on construction**.
///   Validity is only checked when calling `as_str()`, `Debug`, or during serialization.
/// - `new()` truncates at a UTF-8 boundary if the input is too long.
/// - `from_bytes()` only checks that the input fits in `N`.
/// - This type is intentionally kept minimal because each `LiteStr<N>` is monomorphized separately.
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

    /// Creates a `LiteStr` from a `&str`, truncating at a UTF-8 boundary if necessary.
    #[inline(never)]
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; N];
        copy_valid_utf8_prefix(&mut bytes, s.as_bytes(), N);
        Self { bytes }
    }

    /// Returns the content as `&str`, validating UTF-8.
    #[inline(always)]
    pub fn as_str(&self) -> Result<&str, LiteStrErr> {
        str::from_utf8(&self.bytes[..find_first_nul(&self.bytes)])
            .map_err(|_| LiteStrErr::CorruptedData)
    }

    /// Creates a `LiteStr<N>` from a byte slice.
    ///
    /// - Copies up to `N` bytes from the input into the buffer and zero-fills the rest.
    /// - If `bytes.len() > N`, the input is **silently truncated** to the first `N` bytes.
    /// - **No UTF-8 validation is performed.**
    #[inline(always)]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; N];
        let len = bytes.len().min(N);
        arr[..len].copy_from_slice(&bytes[..len]);
        Self { bytes: arr }
    }

    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; N] {
        self.bytes
    }

    /// Returns the content as `&[u8]` (up to the first nul).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..find_first_nul(&self.bytes)]
    }

    /// Returns the length of the content (position of first nul or `N`).
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
    /// Input was too long for this `LiteStr<N>`.
    WrongLen,
    /// The content is not valid UTF-8 (only returned by `as_str`).
    CorruptedData,
}

impl fmt::Display for LiteStrErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteStrErr::WrongLen => f.write_str("input length exceeds SIZE"),
            LiteStrErr::CorruptedData => f.write_str("content is not valid UTF-8"),
        }
    }
}

impl core::error::Error for LiteStrErr {}
