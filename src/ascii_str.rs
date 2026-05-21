use core::fmt::{self, Write};
use core::str;

/// Fixed-capacity, stack-only ASCII string stored in a single `[u8; N]` array.
///
/// The string is stored as raw bytes. Its logical length is determined at
/// runtime by the position of the first nul byte (`b'\0'`). All bytes after
/// the string content are guaranteed to be zero.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AsciiStr<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> fmt::Debug for AsciiStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{:?}", s),
            Err(_) => write!(f, "AsciiStr(<invalid ascii>)"),
        }
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> serde::Serialize for AsciiStr<N> {
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
impl<'de, const N: usize> serde::Deserialize<'de> for AsciiStr<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        AsciiStr::try_from_str(s).map_err(serde::de::Error::custom)
    }
}

/// Errors returned by [`AsciiStr`] operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiStrError {
    /// Input contained non-ASCII characters.
    InvalidAscii,
    /// Input exceeded the fixed capacity `N`.
    TooLong {
        /// Maximum capacity of this `AsciiStr`.
        capacity: usize,
        /// Length of the rejected input.
        length: usize,
    },
    /// Internal data is corrupted or violates the type invariant.
    ///
    /// This can occur when:
    /// - The bytes are not valid UTF-8 (should never happen for ASCII data).
    /// - Non-zero bytes appear after the first nul byte (violates the
    ///   "nul-terminated + trailing zeros" representation invariant).
    ///
    /// This variant exists only to keep the public API 100% panic-free.
    /// It is unreachable when the type is constructed through the safe API.
    CorruptedData,
}

// ─────────────────────────────────────────────────────────────────────────────
// Display implementation (required by serde::ser::Error::custom / de::Error::custom)
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Display for AsciiStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsciiStrError::InvalidAscii => f.write_str("input contained non-ASCII characters"),
            AsciiStrError::TooLong { capacity, length } => {
                write!(
                    f,
                    "input is too long: length {} exceeds capacity {}",
                    length, capacity
                )
            }
            AsciiStrError::CorruptedData => {
                f.write_str("internal data is corrupted or violates the representation invariant")
            }
        }
    }
}

impl<const N: usize> Default for AsciiStr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> AsciiStr<N> {
    /// Creates a new empty `AsciiStr` (all bytes zero).
    pub const fn new() -> Self {
        Self { bytes: [0; N] }
    }

    /// Size of the wire representation in bytes (always equal to the capacity `N`).
    pub const WIRE_SIZE: usize = N;

    pub const DEFAULT: Self = Self::new();

    /// Serializes this `AsciiStr` into a fixed-size byte array.
    ///
    /// The entire internal buffer is written (including trailing zeros after
    /// the logical string content). This preserves the exact representation.
    #[cfg(feature = "wire")]
    #[inline]
    pub fn to_wire_bytes(&self) -> [u8; N] {
        self.bytes
    }

    /// Deserializes an `AsciiStr<N>` from exactly `N` bytes.
    ///
    /// The input must be valid ASCII. Any bytes after the first nul byte
    /// must be zero (as required by the type invariant).
    ///
    /// Returns `None` if the input is not valid ASCII or violates the
    /// internal representation rules.
    #[cfg(feature = "wire")]
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != N {
            return None;
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(bytes);
        Self::try_from_filled_buffer(arr).ok()
    }

    /// Internal constructor used by the `strftime` formatter (and other
    /// trusted code paths).
    ///
    /// The caller **must** guarantee that:
    /// - The first `pos` bytes contain the formatted ASCII string.
    /// - All remaining bytes are zero (nul-terminated).
    ///
    /// For untrusted input use the safe [`try_from_filled_buffer`](Self::try_from_filled_buffer) instead.
    pub(crate) const fn from_filled_buffer(buffer: [u8; N]) -> Self {
        Self { bytes: buffer }
    }

    /// Attempts to create an `AsciiStr<N>` from a raw byte buffer **safely**.
    ///
    /// This is the public, validated counterpart to the internal
    /// [`from_filled_buffer`](Self::from_filled_buffer).
    ///
    /// It performs full validation:
    /// - All bytes must be valid ASCII.
    /// - Every byte after the first `b'\0'` must be zero (preserves the
    ///   nul-terminated + trailing-zeros invariant).
    ///
    /// Use this when you have untrusted or externally-supplied bytes
    /// (network packets, C `strftime` output, user input, etc.).
    ///
    /// **This method (and the entire public API) is completely panic-free.**
    /// All fallible operations return `Result` or `Option`.
    ///
    /// ## Errors
    ///
    /// - [`AsciiStrError::InvalidAscii`] if the buffer contains non-ASCII bytes.
    /// - [`AsciiStrError::CorruptedData`] if bytes after the first nul are
    ///   not all zero (violates the representation invariant).
    pub fn try_from_filled_buffer(buffer: [u8; N]) -> Result<Self, AsciiStrError> {
        if !buffer.is_ascii() {
            return Err(AsciiStrError::InvalidAscii);
        }

        if let Some(first_nul) = buffer.iter().position(|&b| b == 0)
            && buffer[first_nul..].iter().any(|&b| b != 0)
        {
            return Err(AsciiStrError::CorruptedData);
        }

        Ok(Self { bytes: buffer })
    }

    /// Attempts to create an `AsciiStr<N>` from a string slice.
    ///
    /// ## Errors
    ///
    /// - [`AsciiStrError::InvalidAscii`] if the input is not ASCII.
    /// - [`AsciiStrError::TooLong`] if the input exceeds capacity `N`.
    pub fn try_from_str(s: &str) -> Result<Self, AsciiStrError> {
        if !s.is_ascii() {
            return Err(AsciiStrError::InvalidAscii);
        }
        if s.len() > N {
            return Err(AsciiStrError::TooLong {
                capacity: N,
                length: s.len(),
            });
        }
        let mut bytes = [0u8; N];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self { bytes })
    }

    /// Attempts to create an `AsciiStr<N>` from a string slice, **uppercasing** the input.
    ///
    /// This is a convenience wrapper around [`try_from_str`](Self::try_from_str)
    /// that converts the input to ASCII uppercase before storing it.
    ///
    /// # Errors
    /// - [`AsciiStrError::InvalidAscii`] if the input is not ASCII.
    /// - [`AsciiStrError::TooLong`] if the input exceeds capacity `N`.
    pub fn try_from_str_upper(s: &str) -> Result<Self, AsciiStrError> {
        if !s.is_ascii() {
            return Err(AsciiStrError::InvalidAscii);
        }
        if s.len() > N {
            return Err(AsciiStrError::TooLong {
                capacity: N,
                length: s.len(),
            });
        }
        let mut bytes = [0u8; N];
        let src = s.as_bytes();
        bytes[..src.len()].copy_from_slice(src);
        bytes[..src.len()].make_ascii_uppercase();
        Ok(Self { bytes })
    }

    /// Returns the stored string as `&str`.
    ///
    /// The length is computed by locating the first nul byte.
    ///
    /// ## Errors
    ///
    /// - Returns [`AsciiStrError::CorruptedData`] only if the internal data
    ///   has become invalid UTF-8 (unreachable via safe constructors).
    pub fn as_str(&self) -> Result<&str, AsciiStrError> {
        let len = self
            .bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.bytes.len());
        str::from_utf8(&self.bytes[..len]).map_err(|_| AsciiStrError::CorruptedData)
    }

    /// Returns the raw bytes of the stored string (excluding the trailing nul).
    pub fn as_bytes(&self) -> &[u8] {
        let len = self
            .bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.bytes.len());
        &self.bytes[..len]
    }

    /// Returns the current logical length of the string.
    pub fn len(&self) -> usize {
        self.bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.bytes.len())
    }

    /// Returns `true` if the string is empty.
    pub const fn is_empty(&self) -> bool {
        self.bytes[0] == 0
    }

    /// Returns the fixed maximum capacity of this type (always `N`).
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Creates an `AsciiStr` from a `&str`, **truncating** if it exceeds capacity `N`.
    ///
    /// Non-ASCII characters are allowed.
    pub fn from_str_truncate(s: &str) -> Self {
        let mut bytes = [0u8; N];
        let len = s.len().min(N);
        bytes[..len].copy_from_slice(&s.as_bytes()[..len]);
        Self { bytes }
    }

    /// Creates an `AsciiStr` from any type that implements `Display`.
    /// The output is truncated if it exceeds capacity `N`.
    ///
    /// Very useful for embedding numbers, paths, etc. into errors.
    pub fn from_display<T: core::fmt::Display>(value: T) -> Self {
        let mut s = Self::new();
        let _ = write!(&mut s, "{}", value);
        s
    }

    /// Convenience: create from a format string (most ergonomic for errors)
    pub fn from_fmt(args: core::fmt::Arguments<'_>) -> Self {
        let mut s = Self::new();
        let _ = write!(&mut s, "{}", args);
        s
    }
}

impl<const N: usize> TryFrom<&str> for AsciiStr<N> {
    type Error = AsciiStrError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AsciiStr::try_from_str(s)
    }
}

impl<const N: usize> TryFrom<[u8; N]> for AsciiStr<N> {
    type Error = AsciiStrError;

    /// Attempts to create an `AsciiStr<N>` from a filled buffer.
    ///
    /// This is the idiomatic, **completely panic-free** way to construct
    /// from a byte array using the `?` operator or `.unwrap_or_else()`.
    fn try_from(buffer: [u8; N]) -> Result<Self, Self::Error> {
        AsciiStr::try_from_filled_buffer(buffer)
    }
}

impl<const N: usize> core::fmt::Write for AsciiStr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if !s.is_ascii() {
            return Err(core::fmt::Error);
        }

        let current_len = self.len();
        let remaining = N.saturating_sub(current_len);

        // Nothing space to write
        if remaining == 0 {
            return Ok(());
        }

        // Copy as much as possible (truncate if necessary)
        let to_copy = s.len().min(remaining);

        self.bytes[current_len..current_len + to_copy].copy_from_slice(&s.as_bytes()[..to_copy]);

        Ok(())
    }
}
