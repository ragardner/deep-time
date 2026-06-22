//! [`DtErrKind`] and main error type [`DtErr`].
//!
//! [`DtErr`] is a type alias to [`AnErr`] — a compact,
//! zero-allocation error that supports chaining with
//! source locations and short per-level reasons.

use crate::AnErr;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtErrKind {
    UnexpectedEnd,
    UnexpectedInputEnd,
    TruncatedDirective,
    UnknownItem,
    UnsupportedItem,
    BadFractional,
    MismatchedLiteral,
    ExpectedValue,
    ExpectedYear,
    ExpectedCentury,
    ExpectedMonth,
    ExpectedDay,
    ExpectedDayOfYear,
    ExpectedHour,
    ExpectedMinute,
    ExpectedSecond,
    ExpectedFractionalSeconds,
    ExpectedTimestamp,
    ExpectedWeekNumber,
    ExpectedWeekdayNumber,
    InvalidName,
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

/// Wrapper around [`AnErr`].
///
/// A [`DtErr`] object is 120 bytes.
pub type DtErr = AnErr<DtErrKind, 2, 49>;
