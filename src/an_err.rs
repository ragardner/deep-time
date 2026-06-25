use crate::LiteStr;
use core::fmt;
use core::fmt::Write;

/// A compact, `Copy`, zero-allocation error type consisting of a single
/// error kind and a human-readable reason string.
///
/// When context is added via `context`, `context_fmt`, or the `=>` form of
/// `an_err!`, the new reason text is appended to the existing reason.
///
/// The total is silently truncated to `REASON_LEN`
/// bytes if necessary.
#[derive(Clone, Copy, PartialEq, Eq)]
#[must_use = "this error should be handled or converted to a different type e.g. `pub type DtErr = AnErr<MyKind, 49>;`"]
pub struct AnErr<K, const REASON_LEN: usize = 29>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// The error kind.
    pub kind: K,

    /// Accumulated reason string (controlled by `REASON_LEN`).
    /// Can be empty.
    pub reason: LiteStr<REASON_LEN>,
}

impl<K, const REASON_LEN: usize> AnErr<K, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Creates a new error with the given kind and empty reason.
    #[inline(always)]
    pub fn new(kind: K) -> Self {
        Self {
            kind,
            reason: LiteStr::default(),
        }
    }

    /// Creates a new error with the given kind and reason.
    #[inline(always)]
    pub fn with_reason(kind: K, reason: LiteStr<REASON_LEN>) -> Self {
        Self { kind, reason }
    }

    /// Creates a new error with the given kind and a formatted reason.
    ///
    /// The formatted text is truncated if it exceeds `REASON_LEN` bytes.
    #[inline]
    pub fn with_fmt(kind: K, args: core::fmt::Arguments<'_>) -> Self {
        let mut reason = LiteStr::<REASON_LEN>::default();
        let _ = write!(&mut reason, "{}", args);
        Self { kind, reason }
    }

    /// Appends context by appending the given reason text to the accumulated
    /// reason. Truncates if the total would exceed `REASON_LEN` bytes.
    #[inline(always)]
    pub fn context(&mut self, new_reason: LiteStr<REASON_LEN>) {
        self.append_reason(new_reason);
    }

    /// Appends context using a formatted reason string.
    #[inline]
    pub fn context_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        let mut new_reason = LiteStr::<REASON_LEN>::default();
        let _ = write!(&mut new_reason, "{}", args);
        self.append_reason(new_reason);
    }

    #[inline(always)]
    fn append_reason(&mut self, new_reason: LiteStr<REASON_LEN>) {
        let _ = write!(&mut self.reason, "{}", new_reason.as_str());
    }

    /// Returns the current error kind.
    #[inline(always)]
    pub fn kind(&self) -> K {
        self.kind
    }

    /// Returns the accumulated reason.
    #[inline(always)]
    pub fn reason(&self) -> &LiteStr<REASON_LEN> {
        &self.reason
    }
}

impl<K, const REASON_LEN: usize> From<K> for AnErr<K, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    #[inline]
    fn from(kind: K) -> Self {
        Self::new(kind)
    }
}

impl<K, const REASON_LEN: usize> core::fmt::Display for AnErr<K, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.kind)?;

        if !self.reason.as_bytes().is_empty() {
            write!(f, ": {}", self.reason.as_str())?;
        }

        Ok(())
    }
}

impl<K, const REASON_LEN: usize> fmt::Debug for AnErr<K, REASON_LEN>
where
    K: Copy + Clone + fmt::Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<K, const REASON_LEN: usize> core::error::Error for AnErr<K, REASON_LEN> where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq
{
}

/// Ergonomic constructor and chaining macro for [`AnErr`].
///
/// ## Forms
///
/// | Form                                           | Equivalent to                                      |
/// |------------------------------------------------|----------------------------------------------------|
/// | `an_err!(Kind)`                                | `AnErr::new(Kind)`                                 |
/// | `an_err!(Kind, "reason")`                      | `AnErr::with_fmt(Kind, ...)`                       |
/// | `an_err!(Kind, "reason {}", arg, ...)`         | `AnErr::with_fmt(Kind, ...)`                       |
/// | `an_err!("reason" => inner)`                   | `inner.context(...)` (appends to reason only)      |
/// | `an_err!("reason {}", arg => inner)`           | `inner.context_fmt(...)` (appends to reason only)  |
#[macro_export]
macro_rules! an_err {
    ($kind:expr) => {
        $crate::AnErr::new($kind)
    };

    ($fmt:literal $(, $arg:expr)* => $inner:expr $(,)?) => {{
        let mut e = $inner;
        e.context_fmt(format_args!($fmt $(, $arg)*));
        e
    }};

    ($kind:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::AnErr::with_fmt($kind, format_args!($fmt $(, $arg)*))
    };
}

#[cfg(feature = "wire")]
impl<K, const REASON_LEN: usize> AnErr<K, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Serialize this error to a fixed-size byte buffer for transmission.
    ///
    /// The provided buffer must be at least `Self::wire_size()` bytes long.
    /// Returns the number of bytes written.
    pub fn to_wire_bytes(
        &self,
        kind_to_u16: impl Fn(K) -> u16,
        buf: &mut [u8],
    ) -> Result<usize, ()> {
        let needed = Self::wire_size();
        if buf.len() < needed {
            return Err(());
        }

        let mut offset = 0;
        buf[offset] = 1; // version
        offset += 1;

        let kind_val = kind_to_u16(self.kind);
        buf[offset..offset + 2].copy_from_slice(&kind_val.to_le_bytes());
        offset += 2;

        buf[offset..offset + REASON_LEN].copy_from_slice(&self.reason.bytes);

        Ok(needed)
    }

    /// Returns the exact size (in bytes) of the wire representation.
    pub const fn wire_size() -> usize {
        1 + 2 + REASON_LEN
    }

    /// Deserialize from a wire buffer directly into an `AnErr`.
    ///
    /// Requires a closure that maps the stored `u16` back to your concrete `K`.
    /// Returns `None` on corruption, wrong size, unknown version, or mapping failure.
    pub fn from_wire_bytes(bytes: &[u8], u16_to_kind: impl Fn(u16) -> Option<K>) -> Option<Self> {
        if bytes.len() != Self::wire_size() {
            return None;
        }

        let mut offset = 0;
        if bytes[offset] != 1 {
            return None;
        }
        offset += 1;

        let kind_bytes = <[u8; 2]>::try_from(&bytes[offset..offset + 2]).ok()?;
        let kind_u16 = u16::from_le_bytes(kind_bytes);
        let kind = u16_to_kind(kind_u16)?;

        offset += 2;

        let reason_bytes = &bytes[offset..offset + REASON_LEN];
        let reason = LiteStr::from_bytes(reason_bytes);

        Some(Self { kind, reason })
    }
}

#[cfg(feature = "wire")]
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum TestKind {
        Foo,
    }

    #[test]
    fn test_wire_roundtrip_with_append() {
        let err: AnErr<TestKind, 15> = an_err!("bar" => an_err!(TestKind::Foo, "foo"));

        let size = AnErr::<TestKind, 15>::wire_size();
        let mut buf = [0u8; 32];

        let written = err.to_wire_bytes(|k| k as u16, &mut buf).unwrap();
        assert_eq!(written, size);

        let decoded = AnErr::<TestKind, 15>::from_wire_bytes(&buf[..written], |v| {
            if v == 0 { Some(TestKind::Foo) } else { None }
        })
        .unwrap();

        assert_eq!(decoded.kind(), TestKind::Foo);
        assert_eq!(decoded.reason.as_str(), "foobar");
    }
}
