use crate::LiteStr;
use core::fmt;
use core::fmt::Write;
use core::panic::Location;

/// Iterator over the error trace levels of an [`AnErr`].
///
/// Yields `(kind, location, reason)` tuples **from most recent context to oldest**
/// (reverse chronological order). Only valid levels are returned.
///
/// The `reason` field is `Some` if a non-empty reason was supplied for that level,
/// otherwise `None`.
#[derive(Debug, Clone)]
pub struct TraceIter<'a, K, const DEPTH: usize, const REASON_LEN: usize>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    error: &'a AnErr<K, DEPTH, REASON_LEN>,
    pos: usize,
}

impl<'a, K, const DEPTH: usize, const REASON_LEN: usize> Iterator
    for TraceIter<'a, K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    type Item = (
        K,
        &'static Location<'static>,
        Option<&'a LiteStr<REASON_LEN>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.error.len as usize {
            return None;
        }

        let idx = (self.error.len as usize) - 1 - self.pos;
        let kind = self.error.kinds[idx]?;
        let loc = self.error.locations[idx]?;
        let reason = self.error.reasons[idx].as_ref();

        self.pos += 1;
        Some((kind, loc, reason))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.error.len as usize).saturating_sub(self.pos);
        (remaining, Some(remaining))
    }
}

impl<'a, K, const DEPTH: usize, const REASON_LEN: usize> ExactSizeIterator
    for TraceIter<'a, K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
}

/// A compact, `Copy`, zero-allocation error type that records a parallel stack
/// of error kinds, source locations, and per-level human-readable reasons.
///
/// `AnErr` stores up to `DEPTH` levels of error context. Each level contains:
/// - an error kind of type `K`,
/// - the source location where the level was created,
/// - an optional reason specific to that level (`LiteStr<REASON_LEN>`).
///
/// The kind enum provides the general error category while the per-level reason
/// carries concrete details (e.g. a bad value, file path, token, etc.).
///
/// The type implements `Copy` and performs no heap allocation. Default memory
/// footprint is small and fully controllable via the generic parameters.
///
/// ## Type Parameters
///
/// - `K`: Error kind type. Must implement `Copy + Clone + Debug + PartialEq + Eq`.
/// - `DEPTH`: Maximum number of context levels (default `3`). Additional context
///   beyond this limit is silently discarded.
/// - `REASON_LEN`: Maximum length of each individual reason in bytes
///   (default `29`). Longer reasons are silently truncated.
///
/// ## Construction
///
/// ```rust,ignore
/// use an_error::{AnErr, an_err};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum MyKind {
///     Parse,
///     Io,
///     Validation,
/// }
///
/// pub type MyError = AnErr<MyKind, 4, 64>;
///
/// fn parse() -> Result<(), MyError> {
///     Err(an_err!(MyKind::Parse, "unexpected token at byte {}", 42))
/// }
///
/// fn load(path: &str) -> Result<(), MyError> {
///     let inner = parse()
///         .map_err(|e| an_err!(MyKind::Io, "while loading config from {}", path => e))?;
///     Ok(())
/// }
/// ```
///
/// All constructors and the `context` method capture the call site via `#[track_caller]`.
///
/// ## Display
///
/// The `Display` implementation produces output of the following form:
///
/// ```text
/// --
/// • Trace (2 levels):
///    1. Io    @ src/io.rs:42:10    while loading config from /etc/foo
///    2. Parse @ src/parser.rs:17:5  unexpected token at byte 42
/// ```
///
/// Each trace level shows its own reason (if present) immediately after the location.
///
/// ## Invariants
///
/// Maintained by all constructors and `context`:
///
/// - `len` is always in `1..=DEPTH`.
/// - For every `i` in `0..len`, `kinds[i]` and `locations[i]` are `Some`.
/// - `reasons[i]` is `Some` only if a non-empty reason was supplied for that level.
///
/// ## Accessing the stack
///
/// In addition to the top-level convenience methods (`kind()`, `location()`, `reason()`),
/// you can access any level directly or iterate the entire trace.
///
/// ### Direct access
///
/// ```rust,ignore
/// let top_kind     = err.kind();           // most recent
/// let top_loc      = err.location();
/// let top_reason   = err.reason();
///
/// let root_kind    = err.root_kind();      // original error
/// let root_loc     = err.root_location();
/// let root_reason  = err.root_reason();
///
/// if let Some((kind, loc, reason)) = err.get(1) {
///     // second level (index 0 = top, index 1 = next, ...)
/// }
/// ```
///
/// ### Iterating with `trace()`
///
/// The most common way to walk the full stack is with [`trace`](Self::trace):
///
/// ```rust,ignore
/// for (kind, location, reason) in err.trace() {
///     println!("{:?} @ {}:{}", kind, location.file(), location.line());
///
///     if let Some(r) = reason {
///         println!("    reason: {}", r);
///     }
/// }
/// ```
///
/// - Iteration order is **most recent → oldest** (same order as `Display`).
/// - The iterator implements `ExactSizeIterator`, so you can call `.len()`, use it in `for` loops, etc.
/// - No allocation — it just borrows the `AnErr`.
#[derive(Clone, Copy, PartialEq, Eq)]
#[must_use = "this error should be handled or converted to a different type e.g `pub type DtErr = AnErr<MyError, 2, 49>;`"]
pub struct AnErr<K, const DEPTH: usize = 3, const REASON_LEN: usize = 29>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Per-level reasons. Only the first `len` entries are valid.
    /// `None` means no reason (or an empty reason) was provided for that level.
    pub reasons: [Option<LiteStr<REASON_LEN>>; DEPTH],

    /// Parallel stack of source locations.
    /// Only the first `len` entries are valid.
    pub locations: [Option<&'static Location<'static>>; DEPTH],

    /// Parallel stack of error kinds (one per call-stack level).
    /// Only the first `len` entries are valid.
    pub kinds: [Option<K>; DEPTH],

    /// Current depth of the error trace (1 = original error).
    pub len: u8,
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Creates a new error with the given kind and no reason.
    #[inline]
    #[track_caller]
    pub fn new(kind: K) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        let reasons = [None; DEPTH];

        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());

        Self {
            kinds,
            locations: locs,
            reasons,
            len: 1,
        }
    }

    /// Creates a new error with the given kind and reason.
    ///
    /// If the reason is empty, it is stored as `None`.
    #[inline]
    #[track_caller]
    pub fn with_reason(kind: K, reason: LiteStr<REASON_LEN>) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        let mut reasons = [None; DEPTH];

        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        reasons[0] = if reason.len() == 0 {
            None
        } else {
            Some(reason)
        };

        Self {
            kinds,
            locations: locs,
            reasons,
            len: 1,
        }
    }

    /// Creates a new error with the given kind and a formatted reason.
    ///
    /// The formatted string is truncated if it exceeds `REASON_LEN` bytes.
    #[inline]
    #[track_caller]
    pub fn with_fmt(kind: K, args: core::fmt::Arguments<'_>) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        let mut reasons = [None; DEPTH];

        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        let mut reason = LiteStr::<REASON_LEN>::default();
        let _ = write!(&mut reason, "{}", args);
        reasons[0] = if reason.len() == 0 {
            None
        } else {
            Some(reason)
        };

        Self {
            kinds,
            locations: locs,
            reasons,
            len: 1,
        }
    }

    /// Returns the current depth of the error trace.
    #[inline]
    pub fn depth(&self) -> u8 {
        self.len
    }

    /// Returns the most recent error kind (the top of the trace).
    #[inline]
    pub fn kind(&self) -> Option<K> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.len as usize) - 1;
            self.kinds[idx]
        }
    }

    /// Appends a new context level and optional reason to this error.
    ///
    /// If `new_reason` is empty, no reason is stored for the new level.
    /// If the maximum depth is already reached, the call is a no-op.
    #[inline]
    #[track_caller]
    pub fn context(&mut self, kind: K, new_reason: LiteStr<REASON_LEN>) {
        let idx = self.len as usize;
        if idx < DEPTH {
            self.reasons[idx] = if new_reason.len() == 0 {
                None
            } else {
                Some(new_reason)
            };
            self.push(kind, Location::caller());
        }
    }

    /// Appends a new context level with a formatted reason.
    ///
    /// Used internally by the `an_err!` macro. The formatted string is
    /// truncated if it exceeds `REASON_LEN` bytes.
    #[inline]
    #[track_caller]
    pub fn context_fmt(&mut self, kind: K, args: core::fmt::Arguments<'_>) {
        let idx = self.len as usize;
        if idx < DEPTH {
            let mut reason = LiteStr::<REASON_LEN>::default();
            let _ = write!(&mut reason, "{}", args);

            self.reasons[idx] = if reason.len() == 0 {
                None
            } else {
                Some(reason)
            };
            self.push(kind, Location::caller());
        }
    }

    /// Returns an iterator over the error trace, from most recent context
    /// down to the original error.
    ///
    /// Each item is `(kind, location, reason)`. The iterator borrows `self`
    /// with zero copying.
    pub fn trace(&self) -> TraceIter<'_, K, DEPTH, REASON_LEN> {
        TraceIter {
            error: self,
            pos: 0,
        }
    }

    #[inline]
    fn push(&mut self, kind: K, loc: &'static Location<'static>) {
        if (self.len as usize) < DEPTH {
            let idx = self.len as usize;
            self.kinds[idx] = Some(kind);
            self.locations[idx] = Some(loc);
            self.len += 1;
        }
    }

    /// Returns the data for a specific level in the error trace.
    ///
    /// `index == 0` is the **most recent** context (top of the stack / newest `context!`).
    /// `index == self.depth() - 1` is the **root** (original) error.
    ///
    /// Returns `None` if `index >= self.depth()`.
    #[inline]
    pub fn get(
        &self,
        index: usize,
    ) -> Option<(K, &'static Location<'static>, Option<&LiteStr<REASON_LEN>>)> {
        let depth = self.len as usize;
        if index >= depth {
            return None;
        }
        let arr_idx = depth - 1 - index; // 0 in array = root, so we reverse
        Some((
            self.kinds[arr_idx]?,
            self.locations[arr_idx]?,
            self.reasons[arr_idx].as_ref(),
        ))
    }

    /// Returns the source location where the most recent error/context was created.
    #[inline]
    pub fn location(&self) -> Option<&'static Location<'static>> {
        self.get(0).map(|(_, loc, _)| loc)
    }

    /// Returns the reason (if any) attached to the most recent error/context.
    #[inline]
    pub fn reason(&self) -> Option<&LiteStr<REASON_LEN>> {
        self.get(0).and_then(|(_, _, r)| r)
    }

    /// Returns the original (root) error kind.
    #[inline]
    pub fn root_kind(&self) -> Option<K> {
        (self.len > 0).then(|| self.kinds[0]).flatten()
    }

    /// Returns the source location of the original (root) error.
    #[inline]
    pub fn root_location(&self) -> Option<&'static Location<'static>> {
        (self.len > 0).then(|| self.locations[0]).flatten()
    }

    /// Returns the reason (if any) attached to the root error.
    #[inline]
    pub fn root_reason(&self) -> Option<&LiteStr<REASON_LEN>> {
        (self.len > 0).then(|| self.reasons[0].as_ref()).flatten()
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> From<K> for AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Converts a kind into a new [`AnErr`] with no reason.
    #[inline]
    #[track_caller]
    fn from(kind: K) -> Self {
        Self::new(kind)
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> core::fmt::Display
    for AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f)?;
        writeln!(f, "--")?;
        writeln!(f, "Error:")?;

        for (i, (kind, loc, reason_opt)) in self.trace().enumerate() {
            let num = i + 1;

            write!(f, "  {:>2}. {:?}", num, kind)?;

            if let Some(reason) = reason_opt {
                if let Ok(s) = reason.as_str() {
                    write!(f, ": {}", s)?;
                } else {
                    write!(f, ": <invalid ascii>")?;
                }
            }

            writeln!(f, " @ {}:{}:{}", loc.file(), loc.line(), loc.column())?;
        }

        Ok(())
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> fmt::Debug for AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + fmt::Debug + PartialEq + Eq,
{
    /// Debug prints the same clean, human-readable trace as Display.
    /// This makes `unwrap()`, `dbg!()`, and panic messages readable instead of
    /// dumping giant byte arrays.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> core::error::Error
    for AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
}

/// Ergonomic constructor and chaining macro for [`AnErr`].
///
/// ## Forms
///
/// | Form                                              | Equivalent to                                      |
/// |---------------------------------------------------|----------------------------------------------------|
/// | `an_err!(Kind)`                                   | `AnErr::new(Kind)`                               |
/// | `an_err!(Kind, "reason")`                         | `AnErr::with_fmt(Kind, ...)`                     |
/// | `an_err!(Kind, "reason {}", arg, ...)`            | `AnErr::with_fmt(Kind, ...)`                     |
/// | `an_err!(Kind, "reason" => inner)`                | `inner.context(Kind, ...)`                         |
/// | `an_err!(Kind, "reason {}", arg => inner)`        | `inner.context(Kind, ...)`                         |
///
/// All forms capture the call site via `#[track_caller]`.
#[macro_export]
macro_rules! an_err {
    // New error, no reason
    ($kind:expr) => {
        $crate::AnErr::new($kind)
    };

    // Chaining form (must appear before the new-error form)
    ($kind:expr, $fmt:literal $(, $arg:expr)* => $inner:expr $(,)?) => {{
        let mut e = $inner;
        e.context_fmt(
            $kind,
            format_args!($fmt $(, $arg)*)
        );
        e
    }};

    // New error with reason (literal or formatted)
    ($kind:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::AnErr::with_fmt($kind, format_args!($fmt $(, $arg)*))
    };
}

#[cfg(feature = "wire")]
impl<K, const DEPTH: usize, const REASON_LEN: usize> AnErr<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Serialize this error into a fixed-size byte buffer for transmission.
    ///
    /// The caller must provide a buffer that is at least `Self::WIRE_SIZE::<PATH_LEN>()` bytes long.
    /// Returns the number of bytes actually written (always the same for a given `PATH_LEN`).
    ///
    /// Recommended usage:
    /// ```rust,ignore
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// #[repr(u8)]   // or #[repr(u16)] for >256 variants
    /// pub enum MyKind { ... }
    ///
    /// let mut buf = [0u8; AnErr::<MyKind, 3, 29>::wire_size::<80>()];
    /// let written = my_error.to_wire_bytes::<80>(|k| k as u16, &mut buf);
    /// let packet = &buf[..written];
    /// ```
    pub fn to_wire_bytes<const PATH_LEN: usize>(
        &self,
        kind_to_u16: impl Fn(K) -> u16,
        buf: &mut [u8],
    ) -> Result<usize, ()> {
        let needed = Self::wire_size::<PATH_LEN>();
        if buf.len() < needed {
            return Err(());
        }

        let mut offset = 0;

        // Header
        buf[offset] = 1; // wire format version
        offset += 1;
        buf[offset] = self.len;
        offset += 1;

        for i in 0..DEPTH {
            if i < self.len as usize {
                // 1. Kind as u16
                let kind_val = self.kinds[i].map_or(0, &kind_to_u16);
                buf[offset..offset + 2].copy_from_slice(&kind_val.to_le_bytes());
                offset += 2;

                // 2. Reason
                let defaultx = LiteStr::default();
                let reason = self.reasons[i].as_ref().unwrap_or(&defaultx);
                buf[offset..offset + REASON_LEN].copy_from_slice(&reason.to_bytes());
                offset += REASON_LEN;

                // 3. Location
                if let Some(loc) = self.locations[i] {
                    let file = LiteStr::<PATH_LEN>::new(loc.file());
                    buf[offset..offset + PATH_LEN].copy_from_slice(&file.to_bytes());
                    offset += PATH_LEN;

                    buf[offset..offset + 4].copy_from_slice(&loc.line().to_le_bytes());
                    offset += 4;
                    buf[offset..offset + 4].copy_from_slice(&loc.column().to_le_bytes());
                    offset += 4;
                } else {
                    offset += PATH_LEN + 8; // pad
                }
            } else {
                // pad remaining levels
                offset += 2 + REASON_LEN + PATH_LEN + 8;
            }
        }

        Ok(needed)
    }

    /// Compile-time size of the wire representation for a given `PATH_LEN`.
    pub const fn wire_size<const PATH_LEN: usize>() -> usize {
        2 + DEPTH * (2 + REASON_LEN + PATH_LEN + 8)
    }
}

/// Portable location for wire transmission.
#[cfg(feature = "wire")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireLocation<const N: usize> {
    pub file: LiteStr<N>,
    pub line: u32,
    pub column: u32,
}

/// Fully portable, zero-allocation error for transmission/reception.
#[cfg(feature = "wire")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireErr<const DEPTH: usize = 3, const REASON_LEN: usize = 29, const FILE_LEN: usize = 80>
{
    pub len: u8,
    pub kinds: [Option<u16>; DEPTH],
    pub reasons: [Option<LiteStr<REASON_LEN>>; DEPTH],
    pub locations: [Option<WireLocation<FILE_LEN>>; DEPTH],
}

#[cfg(feature = "wire")]
impl<const DEPTH: usize, const REASON_LEN: usize, const FILE_LEN: usize>
    WireErr<DEPTH, REASON_LEN, FILE_LEN>
{
    /// Fixed wire size (exactly matches `AnErr::wire_size::<FILE_LEN>()`).
    pub const fn wire_size() -> usize {
        const fn compute_size<const D: usize, const R: usize, const F: usize>() -> usize {
            2 + D * (2 + R + F + 8)
        }
        compute_size::<DEPTH, REASON_LEN, FILE_LEN>()
    }

    /// Parse a wire buffer from `AnErr` into a `WireErr`.
    ///
    /// Returns `None` on any corruption, wrong size, unknown version,
    /// or invalid `LiteStr` data.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::wire_size() {
            return None;
        }

        let mut offset = 0;

        // Version
        let version = bytes[offset];
        if version != 1 {
            return None; // unknown wire format
        }
        offset += 1;

        let len = bytes[offset];
        if len == 0 || len as usize > DEPTH {
            return None;
        }
        offset += 1;

        let mut kinds = [None; DEPTH];
        let mut reasons = [None; DEPTH];
        let mut locations = [None; DEPTH];

        for i in 0..(len as usize) {
            // kind (u16)
            let kind_bytes = <[u8; 2]>::try_from(&bytes[offset..offset + 2]).ok()?;
            kinds[i] = Some(u16::from_le_bytes(kind_bytes));
            offset += 2;

            // reason
            let reason_bytes = &bytes[offset..offset + REASON_LEN];
            reasons[i] = LiteStr::from_bytes(reason_bytes).ok();
            offset += REASON_LEN;

            // location
            let file_bytes = &bytes[offset..offset + FILE_LEN];
            let file = LiteStr::from_bytes(file_bytes).ok()?;

            offset += FILE_LEN;

            let line_bytes = <[u8; 4]>::try_from(&bytes[offset..offset + 4]).ok()?;
            let line = u32::from_le_bytes(line_bytes);
            offset += 4;

            let col_bytes = <[u8; 4]>::try_from(&bytes[offset..offset + 4]).ok()?;
            let column = u32::from_le_bytes(col_bytes);
            offset += 4;

            locations[i] = Some(WireLocation { file, line, column });
        }

        // remaining bytes are padding (we already checked total length)

        Some(WireErr {
            len,
            kinds,
            reasons,
            locations,
        })
    }
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec::Vec;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum TestKind {
        Root,
        Context1,
        Context2,
        Parse,
        Io,
    }

    /// Helper for creating `LiteStr` reasons (turbofish required for const generic).
    fn r<const N: usize>(s: &str) -> LiteStr<N> {
        LiteStr::new(s)
    }

    // Use the crate's exact *default* parameters so the an_err! macro + constructors
    // match perfectly and inference is unambiguous.
    type E3 = AnErr<TestKind, 3, 29>;

    #[test]
    fn test_new_from_and_basic_properties() {
        let e1: E3 = AnErr::new(TestKind::Root);
        let e2: E3 = TestKind::Root.into();

        // NOTE: We cannot use assert_eq!(e1, e2) because #[track_caller]
        // captures different source locations (different lines in this test).
        // The rest of the data is identical.
        assert_eq!(e1.depth(), e2.depth());
        assert_eq!(e1.kind(), e2.kind());
        assert_eq!(e1.depth(), 1);
        assert_eq!(e1.kind(), Some(TestKind::Root));

        let mut trace = e1.trace();
        let (kind, _loc, reason) = trace.next().unwrap();
        assert_eq!(kind, TestKind::Root);
        assert!(reason.is_none());
        assert!(trace.next().is_none());
    }

    #[test]
    fn test_with_reason_and_with_fmt() {
        // Explicit type fixes const-generic inference (DEPTH cannot be inferred from LiteStr alone)
        let e: E3 = AnErr::with_reason(TestKind::Parse, r::<29>("bad token"));
        assert_eq!(e.depth(), 1);

        let items: Vec<_> = e.trace().collect();
        assert_eq!(items[0].2.unwrap().as_str().unwrap(), "bad token");

        let e2: E3 = AnErr::with_fmt(
            TestKind::Io,
            format_args!("file not found: {}", "config.toml"),
        );
        let items2: Vec<_> = e2.trace().collect();
        assert_eq!(
            items2[0].2.unwrap().as_str().unwrap(),
            "file not found: config.toml"
        );
    }

    #[test]
    fn test_an_err_macro_all_forms() {
        let e1: E3 = an_err!(TestKind::Root);
        assert_eq!(e1.kind(), Some(TestKind::Root));

        let e2: E3 = an_err!(TestKind::Parse, "unexpected {}", "EOF");
        assert_eq!(
            e2.trace().next().unwrap().2.unwrap().as_str().unwrap(),
            "unexpected EOF"
        );

        // Chaining form
        let inner: E3 = an_err!(TestKind::Parse, "bad data");
        let outer: E3 = an_err!(TestKind::Io, "while reading file" => inner);

        assert_eq!(outer.depth(), 2);
        let mut t = outer.trace();
        let (k1, _, r1) = t.next().unwrap();
        assert_eq!(k1, TestKind::Io);
        assert_eq!(r1.unwrap().as_str().unwrap(), "while reading file");

        let (k2, _, r2) = t.next().unwrap();
        assert_eq!(k2, TestKind::Parse);
        assert_eq!(r2.unwrap().as_str().unwrap(), "bad data");
    }

    #[test]
    fn test_context_and_context_fmt() {
        let mut e: E3 = an_err!(TestKind::Root, "initial");
        e.context(TestKind::Context1, r::<29>("level 1"));
        e.context_fmt(TestKind::Context2, format_args!("level {}", 2));

        assert_eq!(e.depth(), 3);

        let trace: Vec<_> = e.trace().collect();
        // Most recent first
        assert_eq!(trace[0].0, TestKind::Context2);
        assert_eq!(trace[1].0, TestKind::Context1);
        assert_eq!(trace[2].0, TestKind::Root);

        assert_eq!(trace[0].2.unwrap().as_str().unwrap(), "level 2");
        assert_eq!(trace[1].2.unwrap().as_str().unwrap(), "level 1");
        assert_eq!(trace[2].2.unwrap().as_str().unwrap(), "initial");
    }

    #[test]
    fn test_max_depth_is_no_op() {
        let mut e: E3 = an_err!(TestKind::Root);
        for i in 0..10 {
            e.context(TestKind::Context1, r::<29>(&format!("extra {i}")));
        }
        assert_eq!(e.depth(), 3); // DEPTH limit reached, further calls ignored

        let trace: Vec<_> = e.trace().collect();
        assert_eq!(trace.len(), 3);
        assert_eq!(trace[0].0, TestKind::Context1); // last successful context
    }

    #[test]
    fn test_empty_reason_becomes_none() {
        let e: E3 = an_err!(TestKind::Parse, "");
        let (_, _, reason) = e.trace().next().unwrap();
        assert!(reason.is_none());

        let mut e2: E3 = an_err!(TestKind::Root);
        e2.context(TestKind::Io, r::<29>("")); // empty literal -> None
        let items: Vec<_> = e2.trace().collect();
        assert!(items[0].2.is_none());
    }

    #[test]
    fn test_trace_iter_order_exact_size_and_size_hint() {
        let e: E3 = an_err!(TestKind::Root, "a" => an_err!(TestKind::Io, "b" => an_err!(TestKind::Parse, "c")));

        let trace = e.trace();
        assert_eq!(trace.len(), 3); // ExactSizeIterator
        assert_eq!(trace.size_hint(), (3, Some(3)));

        let collected: Vec<_> = trace.collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].0, TestKind::Root); // most recent
        assert_eq!(collected[1].0, TestKind::Io);
        assert_eq!(collected[2].0, TestKind::Parse); // original
    }

    #[test]
    fn test_kind_returns_most_recent() {
        let mut e: E3 = an_err!(TestKind::Parse);
        e.context(TestKind::Context1, r::<29>("ctx1"));
        e.context(TestKind::Context2, r::<29>("ctx2"));

        assert_eq!(e.kind(), Some(TestKind::Context2)); // top of the trace
    }

    #[test]
    fn test_display() {
        let inner: E3 = an_err!(TestKind::Parse, "bad syntax");
        let e: E3 = an_err!(TestKind::Io, "while loading config" => inner);

        let display = format!("{}", e);
        assert!(display.contains("--"));
        assert!(display.contains("Error:"));
        assert!(display.contains("Io"));
        assert!(display.contains("while loading config"));
        assert!(display.contains("Parse"));
        assert!(display.contains("bad syntax"));
    }

    #[cfg(feature = "wire")]
    type E4 = AnErr<TestKind, 4, 29>;
    #[cfg(feature = "wire")]
    use alloc::vec;

    #[cfg(feature = "wire")]
    #[test]
    fn test_wire_roundtrip() {
        let inner: E4 = an_err!(TestKind::Parse, "unexpected char");
        let e: E4 = an_err!(TestKind::Io, "while processing file" => inner);

        const FILE_LEN: usize = 64;
        let wire_size = E4::wire_size::<FILE_LEN>();
        let mut buf = vec![0u8; wire_size];

        // Fixed: turbofish required for the const generic PATH_LEN
        let written = e.to_wire_bytes::<FILE_LEN>(|k| k as u16, &mut buf).unwrap();
        assert_eq!(written, wire_size);

        let wire_err = WireErr::<4, 29, FILE_LEN>::from_wire_bytes(&buf[..written]).unwrap();

        assert_eq!(wire_err.len, 2);

        // Wire stores levels oldest-first (index 0 = root)
        assert_eq!(wire_err.kinds[0], Some(TestKind::Parse as u16));
        assert_eq!(wire_err.kinds[1], Some(TestKind::Io as u16));

        assert_eq!(
            wire_err.reasons[0].as_ref().unwrap().as_str().unwrap(),
            "unexpected char"
        );
        assert_eq!(
            wire_err.reasons[1].as_ref().unwrap().as_str().unwrap(),
            "while processing file"
        );
    }

    #[cfg(feature = "wire")]
    #[test]
    fn test_wire_invalid_cases() {
        // Wrong size
        assert!(WireErr::<3, 29, 64>::from_wire_bytes(&[0u8; 10]).is_none());

        // Bad version
        let mut buf = vec![0u8; E4::wire_size::<64>()];
        buf[0] = 99; // invalid version
        assert!(WireErr::<4, 29, 64>::from_wire_bytes(&buf).is_none());
    }
}
