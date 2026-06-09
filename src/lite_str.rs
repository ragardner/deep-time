use core::fmt;
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
/// - **Unicode character count**: Use `as_str().chars().count()`
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
        let end = find_first_nul(&self.bytes);
        let slice = &self.bytes[..end];

        match str::from_utf8(slice) {
            Ok(s) => s,
            Err(e) => {
                let valid = e.valid_up_to();
                if valid == 0 {
                    "\u{FFFD}" // first bytes are garbage → just show �
                } else {
                    // SAFETY: valid_up_to is always a valid UTF-8 boundary
                    unsafe { str::from_utf8_unchecked(&slice[..valid]) }
                }
            }
        }
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

impl<const N: usize> fmt::Display for LiteStr<N> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for LiteStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for LiteStr<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
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
