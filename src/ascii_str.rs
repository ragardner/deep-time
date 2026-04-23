use core::fmt;
use core::str; // ← added for Display

/// Fixed-capacity, stack-only ASCII string stored in a single `[u8; N]` array.
///
/// The string is stored as raw bytes. Its logical length is determined at
/// runtime by the position of the first nul byte (`b'\0'`). All bytes after
/// the string content are guaranteed to be zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsciiStr<const N: usize> {
    bytes: [u8; N],
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
    /// Internal data is not valid UTF-8.
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
            AsciiStrError::CorruptedData => f.write_str("internal data is not valid UTF-8"),
        }
    }
}

impl<const N: usize> AsciiStr<N> {
    /// Creates a new empty `AsciiStr` (all bytes zero).
    pub const fn new() -> Self {
        Self { bytes: [0; N] }
    }

    /// Internal constructor used by the `strftime` formatter.
    ///
    /// The formatter guarantees that:
    /// - The first `pos` bytes contain the formatted ASCII string.
    /// - All remaining bytes are zero (nul-terminated).
    pub(crate) const fn from_filled_buffer(buffer: [u8; N]) -> Self {
        Self { bytes: buffer }
    }

    /// Attempts to create an `AsciiStr<N>` from a string slice.
    ///
    /// # Errors
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
    /// # Errors
    /// Returns [`AsciiStrError::CorruptedData`] only if the internal data
    /// has become invalid UTF-8 (unreachable via safe constructors).
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
}

impl<const N: usize> TryFrom<&str> for AsciiStr<N> {
    type Error = AsciiStrError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AsciiStr::try_from_str(s)
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
