/// A compact, `Copy`, zero-allocation error type that records a parallel stack
/// of error kinds **and** source locations.
///
/// # Example
///
/// ```ignore
/// use ez_error::{EzError, ez_err};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// pub enum MyKind {
///     Parse,
///     Io,
///     Validation,
/// }
///
/// pub type MyError = EzError<MyKind>;
///
/// fn parse() -> Result<(), MyError> {
///     Err(ez_err!(MyKind::Parse, "unexpected token at byte {}", 42))
/// }
///
/// fn load() -> Result<(), MyError> {
///     let inner = parse().map_err(|e| ez_err!(MyKind::Io, "while loading config", e))?;
///     Ok(())
/// }
/// ```
///
/// Printed output (new style):
///
/// ```text
/// --
/// EzError
/// • Reason : unexpected token at byte 42 -> while loading config
/// • Trace (2 levels):
///    1. Io    @ src/io.rs:42
///    2. Parse @ src/parser.rs:17
/// ```
use crate::AsciiStr;
use core::fmt::Write;
use core::panic::Location;

/// A compact, `Copy`, zero-allocation error type that records a parallel stack
/// of error kinds **and** source locations.
///
/// # Invariants (maintained by all constructors and `context`)
/// - `len` is always in `1..=DEPTH` after construction.
/// - For every `i` in `0..len`, `kinds[i]` and `locations[i]` are `Some`.
/// - `kinds` and `locations` are always kept in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "this error should be handled or converted to a different type"]
pub struct EzError<K, const DEPTH: usize = 4>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Parallel stack of error kinds (one per call-stack level).
    /// Only the first `len` entries are valid.
    pub kinds: [Option<K>; DEPTH],

    /// Parallel stack of source locations (`file:line`).
    /// Only the first `len` entries are valid.
    locations: [Option<&'static Location<'static>>; DEPTH],

    /// Current depth of the error trace (1 = original error).
    len: u8,

    /// Human-readable reason / context message (ASCII, max 127 bytes).
    pub reason: AsciiStr<127>,
}

impl<K, const DEPTH: usize> EzError<K, DEPTH>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Creates a new error with the given kind at the current call site.
    ///
    /// The reason is empty; use [`with_reason`](Self::with_reason) or
    /// [`with_fmt`](Self::with_fmt) if you need a message.
    #[inline]
    #[track_caller]
    pub fn new(kind: K) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        Self {
            kinds,
            locations: locs,
            len: 1,
            reason: AsciiStr::new(),
        }
    }

    /// Creates a new error with the given kind and a pre-built reason string.
    #[inline]
    #[track_caller]
    pub fn with_reason(kind: K, reason: AsciiStr<127>) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        Self {
            kinds,
            locations: locs,
            len: 1,
            reason,
        }
    }

    /// Creates a new error with the given kind and a formatted reason.
    ///
    /// The `args` are written into the internal `AsciiStr<127>` buffer.
    #[inline]
    #[track_caller]
    pub fn with_fmt(kind: K, args: core::fmt::Arguments<'_>) -> Self {
        let mut kinds = [None; DEPTH];
        let mut locs = [None; DEPTH];
        kinds[0] = Some(kind);
        locs[0] = Some(Location::caller());
        Self {
            kinds,
            locations: locs,
            len: 1,
            reason: AsciiStr::from_fmt(args),
        }
    }

    /// Returns the current depth of the error trace.
    ///
    /// - `1` = the original error (no `context` calls yet)
    /// - `2+` = after one or more `context` calls
    #[inline]
    pub fn depth(&self) -> u8 {
        self.len
    }

    /// Returns the current (outermost) error kind.
    ///
    /// This is the kind that was passed to the most recent `context` call,
    /// or the original kind if no chaining occurred.
    ///
    /// # Panics
    ///
    /// Never panics — the internal invariant guarantees `len >= 1` and the
    /// top slot is always `Some`.
    #[inline]
    pub fn kind(&self) -> K {
        // SAFETY: construction and `push` maintain the invariant that
        // for all i < len the corresponding slot is Some, and len >= 1.
        let idx = (self.len as usize) - 1;
        self.kinds[idx].expect("EzError internal invariant violated: top kind is None")
    }

    /// Private helper that appends one `(kind, location)` level if the stack
    /// has not yet reached its maximum depth of 4.
    ///
    /// Further calls after the stack is full are silently ignored.
    #[inline]
    fn push(&mut self, kind: K, loc: &'static Location<'static>) {
        if (self.len as usize) < DEPTH {
            self.kinds[self.len as usize] = Some(kind);
            self.locations[self.len as usize] = Some(loc);
            self.len += 1;
        }
    }

    /// Attaches additional context to an existing error.
    ///
    /// - The `new_reason` is appended to the existing reason with a ` → ` separator.
    /// - A new `(kind, location)` pair is pushed onto both stacks.
    /// - If the trace is already at maximum depth (`DEPTH`), the new level is dropped.
    #[inline]
    #[track_caller]
    pub fn context(self, kind: K, new_reason: AsciiStr<127>) -> Self {
        let mut reason = self.reason;
        if !reason.is_empty() {
            let _ = write!(
                &mut reason,
                " -> {}",
                new_reason.as_str().unwrap_or("<invalid ascii>")
            );
        } else {
            reason = new_reason;
        }

        let mut e = self;
        e.reason = reason;
        e.push(kind, Location::caller());
        e
    }
}

impl<K, const DEPTH: usize> From<K> for EzError<K, DEPTH>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    /// Converts a bare error kind into an `EzError` (empty reason, current location).
    #[inline]
    #[track_caller]
    fn from(kind: K) -> Self {
        Self::new(kind)
    }
}

impl<K, const DEPTH: usize> core::fmt::Display for EzError<K, DEPTH>
where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f)?;
        writeln!(f, "--")?;
        writeln!(f, "EzError")?;

        if !self.reason.is_empty() {
            if let Ok(r) = self.reason.as_str() {
                writeln!(f, "• Reason : {}", r)?;
            } else {
                writeln!(f, "• Reason : <invalid ascii>")?;
            }
        }

        writeln!(
            f,
            "• Trace ({} level{}):",
            self.len,
            if self.len == 1 { "" } else { "s" }
        )?;

        // Print in reverse order so outermost error = level 1
        for (i, (kind_opt, loc_opt)) in self
            .kinds
            .iter()
            .zip(self.locations.iter())
            .take(self.len as usize)
            .rev()
            .enumerate()
        {
            let num = i + 1;
            match (kind_opt, loc_opt) {
                (Some(k), Some(l)) => {
                    writeln!(f, "   {:>2}. {:?} @ {}:{}", num, k, l.file(), l.line())?
                }
                (Some(k), None) => writeln!(f, "   {:>2}. {:?} @ <unknown location>", num, k)?,
                (None, Some(l)) => {
                    writeln!(f, "   {:>2}. <no kind> @ {}:{}", num, l.file(), l.line())?
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl<K, const DEPTH: usize> core::error::Error for EzError<K, DEPTH> where
    K: Copy + Clone + core::fmt::Debug + PartialEq + Eq
{
}

// ─────────────────────────────────────────────────────────────────────────────
// ez_err! macro
// ─────────────────────────────────────────────────────────────────────────────

/// Ergonomic constructor and chaining macro for [`EzError`].
///
/// # Arms
///
/// | Syntax                                      | Meaning                                      |
/// |---------------------------------------------|----------------------------------------------|
/// | `ez_err!(Kind)`                             | New error, empty reason, current location    |
/// | `ez_err!(Kind, "msg {}", x)`                | New error with formatted reason              |
/// | `ez_err!(NewKind, "ctx", inner)`            | Chain with literal reason (no extra args)    |
/// | `ez_err!(NewKind, "ctx {}", val, inner)`    | Chain with formatted reason + inner error    |
#[macro_export]
macro_rules! ez_err {
    // 1. New error, no reason
    ($kind:expr) => {
        $crate::EzError::new($kind)
    };

    // 2. Chaining (must come first so `=>` is recognized)
    //     (no type annotation on `inner` so the user's chosen DEPTH is preserved)
    ($kind:expr, $fmt:literal $(, $arg:expr)* => $inner:expr $(,)?) => {{
        let inner = $inner;
        inner.context($kind, $crate::AsciiStr::from_fmt(format_args!($fmt $(, $arg)*)))
    }};

    // 3. New error with reason (literal or formatted)
    ($kind:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        $crate::EzError::with_fmt($kind, format_args!($fmt $(, $arg)*))
    };
}
