use core::fmt;
use core::str;

/// A fixed-capacity, stack-allocated **byte buffer** that can hold a UTF-8 string.
///
/// `BufStr<N>` stores its content in a `[u8; N]` array using C-style nul
/// termination. The logical length is determined by the position of the first
/// `b'\0'` byte (or `N` if the buffer is completely filled without a nul).
///
/// This type performs **no validation during construction**. UTF-8 validity is
/// only checked when the content is accessed via [`as_str`](#method.as_str), [`Debug`], or
/// serialization.
///
/// Both [`new`](#method.new) and [`from_bytes`](#method.from_bytes) silently truncate input that exceeds the
/// capacity `N`. This type is intentionally minimal because each `BufStr<N>`
/// is monomorphized independently.
///
/// ## .len()
///
/// - **Byte length**: [`BufStr::as_bytes`](#method.as_bytes) (then `.len()`)
/// - **Unicode character count**: Use `as_str().chars().count()`
#[derive(Clone, PartialEq, Eq)]
pub struct BufStr<const N: usize> {
    pub bytes: [u8; N],
}

impl<const N: usize> Default for BufStr<N> {
    #[inline(always)]
    fn default() -> Self {
        Self { bytes: [0; N] }
    }
}

impl<const N: usize> BufStr<N> {
    pub const SIZE: usize = N;

    /// Creates a new `BufStr` from a `&str`.
    ///
    /// If the input is longer than `N` bytes, it is truncated at the nearest
    /// valid UTF-8 boundary.
    #[inline(always)]
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; N];
        copy_valid_utf8_prefix(&mut bytes, s.as_bytes(), N);
        Self { bytes }
    }

    /// Creates a `BufStr<N>` from a byte slice.
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

    /// Returns the longest valid UTF-8 prefix of the content as a `&str`.
    ///
    /// - If the data is valid UTF-8, returns it directly.
    /// - If the data starts with invalid bytes, returns a single replacement
    ///   character (`�`).
    /// - Otherwise returns only the valid prefix up to the first invalid
    ///   sequence (everything after the first error is discarded).
    ///
    /// This method is infallible and never allocates.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        let slice = &self.bytes[..find_first_nul(&self.bytes)];
        match str::from_utf8(slice) {
            Ok(s) => s,
            Err(e) => handle_invalid_utf8(slice, e),
        }
    }

    /// Returns the content as a byte slice (up to the first nul byte).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..find_first_nul(&self.bytes)]
    }
}

impl<const N: usize> fmt::Write for BufStr<N> {
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

impl<const N: usize> fmt::Display for BufStr<N> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for BufStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for BufStr<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for BufStr<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        Ok(BufStr::new(s))
    }
}

#[cfg(feature = "defmt")]
impl<const N: usize> defmt::Format for BufStr<N> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", self.as_str());
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

#[cold]
#[inline(never)]
fn handle_invalid_utf8(slice: &[u8], e: core::str::Utf8Error) -> &str {
    let valid = e.valid_up_to();
    if valid == 0 {
        "\u{FFFD}"
    } else {
        str::from_utf8(&slice[..valid]).unwrap_or("\u{FFFD}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_valid() {
        assert_eq!(BufStr::<16>::new("hello").as_str(), "hello");
        assert_eq!(BufStr::<8>::default().as_str(), "");
    }

    #[test]
    fn as_str_invalid_leading_byte() {
        let s = BufStr::<8>::from_bytes(&[0xFF, b'a']);
        assert_eq!(s.as_str(), "\u{FFFD}");
    }

    #[test]
    fn as_str_valid_prefix_then_garbage() {
        let s = BufStr::<8>::from_bytes(&[b'h', b'i', 0xFF, b'!']);
        assert_eq!(s.as_str(), "hi");
    }

    #[test]
    fn as_str_truncated_multibyte_at_start() {
        // incomplete U+20AC (euro sign)
        let s = BufStr::<8>::from_bytes(&[0xE2, 0x82]);
        assert_eq!(s.as_str(), "\u{FFFD}");
    }

    #[test]
    fn as_str_truncated_multibyte_after_valid_prefix() {
        let s = BufStr::<8>::from_bytes(&[b'h', b'i', 0xE2, 0x82]);
        assert_eq!(s.as_str(), "hi");
    }

    #[test]
    fn as_str_stops_at_nul() {
        let s = BufStr::<8>::from_bytes(b"ab\0cd");
        assert_eq!(s.as_str(), "ab");
        assert_eq!(s.as_bytes(), b"ab");
    }
}
