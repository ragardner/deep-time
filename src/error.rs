use crate::AnErr;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtErrKind {
    /// Format string or input ended unexpectedly (%, ., exhausted input, etc.)
    UnexpectedEnd,
    /// Unknown % directive (the `_` case)
    UnknownDirective,
    /// %c, %r, %X, %x, %Z etc. (explicitly unsupported library directives)
    UnsupportedDirective,
    /// The `.` was followed by something other than f/N
    BadFractional,
    /// Literal character or % sign in input didn't match format
    MismatchedLiteral,
    /// Generic "could not parse expected integer" (year, month, day, hour, …)
    ExpectedValue,
    /// Month name, weekday name, or AM/PM failed to parse
    InvalidName,
    /// Anything wrong with a timezone offset (+HH:MM:SS syntax)
    InvalidTimezoneOffset,
    MustStartWith,
    InvalidNumber,
    InvalidItem,
    InvalidBytes,
    InvalidSyntax,
    OutOfRange,
    TrailingCharacters,
    Incomplete,
    InvalidInput,
    InternalErr,
    IOErr,
}

// 120 bytes
pub type DtErr = AnErr<DtErrKind, 2, 49>;
