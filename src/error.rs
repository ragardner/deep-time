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
    /// %L clock type parsing failed
    InvalidClockType,
    /// Generic must start with
    MustStartWith,
    /// Any failure to parse a number, integer, or fractional part
    /// (no digits, parse::<i64> failed, bad UTF-8, empty fraction, too many decimals, etc.)
    InvalidNumber,
    InvalidItem,
    InvalidBytes,
    InvalidSyntax,

    FormatterErr,
    OutOfRange,
    TrailingCharacters,
    Incomplete,
    InvalidInput,
    CCSDSInputErr,
    CCSDSOutputErr,
    InternalErr,
    IOErr,
    JiffConversion,
    ChronoConversion,
}

pub type DtErr = AnErr<DtErrKind, 3, 29>;
