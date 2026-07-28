use core::{fmt, str};

/// A fixed-capacity, stack-allocated byte buffer that can hold a UTF-8
/// string.
///
/// `BufStr<N>` stores its content in a `[u8; N]` array. The logical length is
/// tracked in [`len`](#structfield.len) (0..=`N`, stored as `u16`). Unused bytes
/// past `len` are zero-filled by the constructors. [`from_bytes`](#method.from_bytes)
/// treats the first `b'\0'` in the input as an end marker (C-style), so
/// zero-padded wire buffers deserialize with the correct length.
///
/// This type performs **no validation during construction**. UTF-8 validity is
/// only checked when the content is accessed via [`as_str`](#method.as_str),
/// [`Debug`], or serialization.
///
/// Both [`new`](#method.new) and [`from_bytes`](#method.from_bytes)
/// silently truncate input that exceeds the capacity `N`.
///
/// This type is intentionally minimal because each `BufStr<N>` is
/// monomorphized independently.
///
/// ## To get the length of the str
///
/// - **Byte length**: [`BufStr::len`](#method.len) or [`as_bytes`](#method.as_bytes)`.len()`
/// - **Unicode character count**: Use `as_str().chars().count()`
#[derive(Clone, PartialEq, Eq)]
pub struct BufStr<const N: usize> {
    /// Raw fixed-capacity byte storage (zero-filled past [`len`](#structfield.len)).
    pub bytes: [u8; N],
    /// Number of valid content bytes in [`bytes`](#structfield.bytes) (0..=`N`).
    pub len: u16,
}

impl<const N: usize> Default for BufStr<N> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> BufStr<N> {
    /// Capacity of the buffer in bytes (same as the const parameter `N`).
    pub const SIZE: usize = N;

    /// Creates a new `BufStr` from a `&str`.
    ///
    /// If the input is longer than `N` bytes, it is truncated at the nearest
    /// valid UTF-8 boundary.
    #[inline]
    pub fn new(s: &str) -> Self {
        let mut bytes = [0u8; N];
        let len = copy_str_prefix(&mut bytes, s, N) as u16;
        Self { bytes, len }
    }

    /// Creates a `BufStr<N>` from a byte slice.
    ///
    /// Copies up to `N` bytes from the input and zero-fills the remainder.
    /// If `bytes.len() > N`, the input is silently truncated.
    ///
    /// Logical length stops at the first `b'\0'` in the copied prefix (or the
    /// full copied length if there is no nul). This matches C-style /
    /// zero-padded wire buffers.
    ///
    /// No UTF-8 validation is performed.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut arr = [0u8; N];
        let copy_len = bytes.len().min(N);
        arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
        let len = arr[..copy_len]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(copy_len);
        Self {
            bytes: arr,
            len: len as u16,
        }
    }

    /// Returns the number of valid content bytes.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        let len = self.len as usize;
        if len < N { len } else { N }
    }

    /// Returns `true` if the buffer has no content.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the longest valid UTF-8 prefix of the content as a `&str`.
    ///
    /// - If the data is valid UTF-8, returns it directly.
    /// - If the data starts with invalid bytes, returns a single replacement
    ///   character (`�`).
    /// - Otherwise returns only the valid prefix up to the first invalid
    ///   sequence (everything after the first error is discarded).
    /// - This method is infallible.
    #[inline]
    pub fn as_str(&self) -> &str {
        match str::from_utf8(self.as_bytes()) {
            Ok(s) => s,
            Err(e) => handle_invalid_utf8(self.as_bytes(), e),
        }
    }

    /// Returns the content as a byte slice of length [`len`](#method.len).
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }
}

impl<const N: usize> fmt::Write for BufStr<N> {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let current = self.len();
        let remaining = N.saturating_sub(current);
        if remaining == 0 {
            return Ok(());
        }

        let written = copy_str_prefix(&mut self.bytes[current..], s, remaining);
        self.len = (current + written) as u16;
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

/// Copy up to `max_len` bytes of already-valid UTF-8 into `dst`.
///
/// Both call sites pass a `&str`, so a full `from_utf8` scan is unnecessary.
/// When capacity is smaller than `src`, walk back at most 3 bytes to a char
/// boundary, then one `copy_from_slice`.
#[inline(never)]
fn copy_str_prefix(dst: &mut [u8], src: &str, max_len: usize) -> usize {
    let bytes = src.as_bytes();
    let mut len = bytes.len().min(max_len).min(dst.len());
    while !src.is_char_boundary(len) {
        len -= 1;
    }
    if len != 0 {
        dst[..len].copy_from_slice(&bytes[..len]);
    }
    len
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
        let s = BufStr::<16>::new("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
        assert_eq!(BufStr::<8>::default().as_str(), "");
        assert!(BufStr::<8>::default().is_empty());
    }

    #[test]
    fn as_str_invalid_leading_byte() {
        let s = BufStr::<8>::from_bytes(&[0xFF, b'a']);
        assert_eq!(s.as_str(), "\u{FFFD}");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn as_str_valid_prefix_then_garbage() {
        let s = BufStr::<8>::from_bytes(&[b'h', b'i', 0xFF, b'!']);
        assert_eq!(s.as_str(), "hi");
        assert_eq!(s.len(), 4);
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
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn from_bytes_zero_padded() {
        // Wire-style: full capacity of zeros after content.
        let mut raw = [0u8; 8];
        raw[..5].copy_from_slice(b"hello");
        let s = BufStr::<8>::from_bytes(&raw);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn new_truncates_on_char_boundary() {
        // "€" is 3 bytes (E2 82 AC). Capacity 2 must drop the whole char.
        assert_eq!(BufStr::<2>::new("€").as_str(), "");
        assert_eq!(BufStr::<2>::new("€").len(), 0);
        assert_eq!(BufStr::<3>::new("€").as_str(), "€");
        assert_eq!(BufStr::<3>::new("€").len(), 3);
        assert_eq!(BufStr::<4>::new("a€b").as_str(), "a€");
        assert_eq!(BufStr::<4>::new("a€b").len(), 4);
        assert_eq!(BufStr::<5>::new("a€b").as_str(), "a€b");
        assert_eq!(BufStr::<5>::new("a€b").len(), 5);
    }

    #[test]
    fn write_str_appends_and_truncates() {
        use core::fmt::Write;
        let mut s = BufStr::<5>::default();
        write!(&mut s, "hi").unwrap();
        assert_eq!(s.len(), 2);
        write!(&mut s, "€").unwrap(); // 3 bytes → fills exactly
        assert_eq!(s.as_str(), "hi€");
        assert_eq!(s.len(), 5);
        write!(&mut s, "x").unwrap(); // full → no-op
        assert_eq!(s.as_str(), "hi€");
        assert_eq!(s.len(), 5);
    }
}
