use crate::AsciiStr;
use core::panic::Location;

/// Iterator over the error trace levels of an [`EzError`].
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
    error: &'a EzError<K, DEPTH, REASON_LEN>,
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
        Option<&'a AsciiStr<REASON_LEN>>,
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
/// `EzError` stores up to `DEPTH` levels of error context. Each level contains:
/// - an error kind of type `K`,
/// - the source location where the level was created,
/// - an optional reason specific to that level (`AsciiStr<REASON_LEN>`).
///
/// The kind enum provides the general error category while the per-level reason
/// carries concrete details (e.g. a bad value, file path, token, etc.).
///
/// The type implements `Copy` and performs no heap allocation. Default memory
/// footprint is small and fully controllable via the generic parameters.
///
/// # Type Parameters
///
/// - `K`: Error kind type. Must implement `Copy + Clone + Debug + PartialEq + Eq`.
/// - `DEPTH`: Maximum number of context levels (default `3`). Additional context
///   beyond this limit is silently discarded.
/// - `REASON_LEN`: Maximum length of each individual reason in bytes
///   (default `29`). Longer reasons are silently truncated.
///
/// # Construction
///
/// ```rust,ignore
/// use ez_error::{EzError, ez_err};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum MyKind {
///     Parse,
///     Io,
///     Validation,
/// }
///
/// pub type MyError = EzError<MyKind, 4, 64>;
///
/// fn parse() -> Result<(), MyError> {
///     Err(ez_err!(MyKind::Parse, "unexpected token at byte {}", 42))
/// }
///
/// fn load(path: &str) -> Result<(), MyError> {
///     let inner = parse()
///         .map_err(|e| ez_err!(MyKind::Io, "while loading config from {}", path => e))?;
///     Ok(())
/// }
/// ```
///
/// All constructors and the `context` method capture the call site via `#[track_caller]`.
///
/// # Display
///
/// The `Display` implementation produces output of the following form:
///
/// ```text
/// --
/// EzError
/// • Trace (2 levels):
///    1. Io    @ src/io.rs:42:10    while loading config from /etc/foo
///    2. Parse @ src/parser.rs:17:5  unexpected token at byte 42
/// ```
///
/// Each trace level shows its own reason (if present) immediately after the location.
///
/// # Invariants
///
/// Maintained by all constructors and `context`:
///
/// - `len` is always in `1..=DEPTH`.
/// - For every `i` in `0..len`, `kinds[i]` and `locations[i]` are `Some`.
/// - `reasons[i]` is `Some` only if a non-empty reason was supplied for that level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "this error should be handled or converted to a different type"]
pub struct EzError<K, const DEPTH: usize = 3, const REASON_LEN: usize = 29>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Per-level reasons. Only the first `len` entries are valid.
    /// `None` means no reason (or an empty reason) was provided for that level.
    pub reasons: [Option<AsciiStr<REASON_LEN>>; DEPTH],

    /// Parallel stack of source locations.
    /// Only the first `len` entries are valid.
    pub locations: [Option<&'static Location<'static>>; DEPTH],

    /// Parallel stack of error kinds (one per call-stack level).
    /// Only the first `len` entries are valid.
    pub kinds: [Option<K>; DEPTH],

    /// Current depth of the error trace (1 = original error).
    pub len: u8,
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> EzError<K, DEPTH, REASON_LEN>
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
    pub fn with_reason(kind: K, reason: AsciiStr<REASON_LEN>) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        let mut reasons = [None; DEPTH];

        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        reasons[0] = if reason.is_empty() {
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
        let reason = AsciiStr::from_fmt(args);
        reasons[0] = if reason.is_empty() {
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
    pub fn context(&mut self, kind: K, new_reason: AsciiStr<REASON_LEN>) {
        let idx = self.len as usize;
        if idx < DEPTH {
            self.reasons[idx] = if new_reason.is_empty() {
                None
            } else {
                Some(new_reason)
            };
            self.push(kind, Location::caller());
        }
    }

    /// Appends a new context level with a formatted reason.
    ///
    /// Used internally by the `ez_err!` macro. The formatted string is
    /// truncated if it exceeds `REASON_LEN` bytes.
    #[inline]
    #[track_caller]
    pub fn context_fmt(&mut self, kind: K, args: core::fmt::Arguments<'_>) {
        let idx = self.len as usize;
        if idx < DEPTH {
            let reason = AsciiStr::from_fmt(args);
            self.reasons[idx] = if reason.is_empty() {
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
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> From<K> for EzError<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Converts a kind into a new [`EzError`] with no reason.
    #[inline]
    #[track_caller]
    fn from(kind: K) -> Self {
        Self::new(kind)
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> core::fmt::Display
    for EzError<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f)?;
        writeln!(f, "--")?;
        writeln!(f, "EzError")?;

        writeln!(
            f,
            "• Trace ({} level{}):",
            self.len,
            if self.len == 1 { "" } else { "s" }
        )?;

        for (i, (kind, loc, reason_opt)) in self.trace().enumerate() {
            let num = i + 1;
            write!(
                f,
                "   {:>2}. {:?} @ {}:{}:{}",
                num,
                kind,
                loc.file(),
                loc.line(),
                loc.column()
            )?;

            if let Some(reason) = reason_opt {
                if let Ok(s) = reason.as_str() {
                    writeln!(f, "    {}", s)?;
                } else {
                    writeln!(f, "    <invalid ascii>")?;
                }
            } else {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl<K, const DEPTH: usize, const REASON_LEN: usize> core::error::Error
    for EzError<K, DEPTH, REASON_LEN>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
}

/// Ergonomic constructor and chaining macro for [`EzError`].
///
/// # Forms
///
/// | Form                                              | Equivalent to                                      |
/// |---------------------------------------------------|----------------------------------------------------|
/// | `ez_err!(Kind)`                                   | `EzError::new(Kind)`                               |
/// | `ez_err!(Kind, "reason")`                         | `EzError::with_fmt(Kind, ...)`                     |
/// | `ez_err!(Kind, "reason {}", arg, ...)`            | `EzError::with_fmt(Kind, ...)`                     |
/// | `ez_err!(Kind, "reason" => inner)`                | `inner.context(Kind, ...)`                         |
/// | `ez_err!(Kind, "reason {}", arg => inner)`        | `inner.context(Kind, ...)`                         |
///
/// All forms capture the call site via `#[track_caller]`.
#[macro_export]
macro_rules! ez_err {
    // New error, no reason
    ($kind:expr) => {
        $crate::EzError::new($kind)
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
        $crate::EzError::with_fmt($kind, format_args!($fmt $(, $arg)*))
    };
}
