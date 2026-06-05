use core::fmt::{self};
use core::str;

/// A fixed-capacity, stack-allocated buffer that can hold a UTF-8 string.
///
/// `LiteStr<N>` stores its content in a `[u8; N]` array using C-style nul
/// termination. The logical length is determined by the position of the first
/// `b'\0'` byte (or `N` if the buffer is completely filled without a nul).
///
/// This type performs **no validation during construction**. UTF-8 validity is
/// only checked when the content is accessed via [`as_str`], [`Debug`], or
/// serialization.
///
/// Both [`new`] and [`from_bytes`] silently truncate input that exceeds the
/// capacity `N`. This type is intentionally minimal because each `LiteStr<N>`
/// is monomorphized independently.
///
/// ## .len()
///
/// - **Byte length**: Use [`as_bytes()`][Self::as_bytes]`.len()`
/// - **Unicode character count**: Use [`as_str()`][Self::as_str]`.unwrap().len()`
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LiteStr<const N: usize> {
    pub bytes: [u8; N],
}

impl<const N: usize> Default for LiteStr<N> {
    #[inline(always)]
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const N: usize> LiteStr<N> {
    pub const SIZE: usize = N;

    /// Creates a new `LiteStr` from a `&str`.
    ///
    /// If the input is longer than `N` bytes, it is truncated at the nearest
    /// valid UTF-8 boundary.
    #[inline(always)]
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; N];
        copy_valid_utf8_prefix(&mut bytes, s.as_bytes(), N);
        Self { bytes }
    }

    /// Returns the content as a `&str`, validating that it is well-formed UTF-8.
    ///
    /// Finds the first nul byte and uses that as the end of the str, or if
    /// there isn't a nul byte then uses the whole len `N`.
    #[inline(always)]
    pub fn as_str(&self) -> Result<&str, LiteStrErr> {
        let end = find_first_nul(&self.bytes);
        str::from_utf8(&self.bytes[..end]).map_err(|_| LiteStrErr::CorruptedData)
    }

    /// Creates a `LiteStr<N>` from a byte slice.
    ///
    /// Copies up to `N` bytes from the input and zero-fills the remainder.
    /// If `bytes.len() > N`, the input is silently truncated.
    ///
    /// No UTF-8 validation is performed.
    #[inline(always)]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; N];
        let len = bytes.len().min(N);
        arr[..len].copy_from_slice(&bytes[..len]);
        Self { bytes: arr }
    }

    /// Returns the content as a byte slice (up to the first nul byte).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..find_first_nul(&self.bytes)]
    }
}

impl<const N: usize> fmt::Write for LiteStr<N> {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let current = self.as_bytes().len();
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

/// Errors that can occur when using a [`LiteStr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteStrErr {
    /// The content is not valid UTF-8.
    CorruptedData,
}

impl fmt::Display for LiteStrErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteStrErr::CorruptedData => f.write_str("content is not valid UTF-8"),
        }
    }
}

impl core::error::Error for LiteStrErr {}
