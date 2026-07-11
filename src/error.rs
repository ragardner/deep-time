//! [`DtErrKind`] and main error type [`DtErr`].
//! [`DtErr`] is a type alias to [`AnErr`].

use crate::AnErr;

/// Failure category inside [`DtErr`](crate::DtErr) — parse errors, out-of-range
/// values, missing features, and similar cases.
///
/// Almost every function in this crate that can fail returns a
/// [`DtErr`](crate::DtErr). That error is made of two parts:
///
/// 1. A **kind** — one of the variants of this enum (what went wrong).
/// 2. A short **reason** string — optional extra detail (up to 15 bytes).
///
/// Those two pieces live inside [`AnErr`](crate::AnErr). [`DtErr`](crate::DtErr)
/// is simply `AnErr<DtErrKind, 15>` — the whole error is 16 bytes.
///
/// Create one with the [`an_err!`](crate::an_err) macro, and read the kind
/// back with [`.kind()`](crate::AnErr::kind):
///
/// ```
/// use deep_time::{DtErr, DtErrKind, an_err};
///
/// let err: DtErr = an_err!(DtErrKind::YearOutOfRange, "year={}", 10_000);
/// assert_eq!(err.kind(), DtErrKind::YearOutOfRange);
/// ```
///
/// When printed, an error looks like `YearOutOfRange` or
/// `YearOutOfRange: year=10000`.
///
/// This enum is marked `#[non_exhaustive]`, so new variants may be added in
/// later releases. Always keep a catch-all arm (`_ => ...`) when matching.
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DtErrKind {
    UnexpectedEnd,
    TruncatedDirective,
    UnknownItem,
    MissingFeature,
    MissingRefTimeOrStd,
    MustStartWith,
    UnsupportedItem,
    MismatchedLiteral,
    ExpectedUnit,
    ExpectedValue,
    ExpectedYear,
    ExpectedCentury,
    ExpectedMonth,
    ExpectedDay,
    ExpectedDayOfYear,
    ExpectedHour,
    ExpectedMinute,
    ExpectedSecond,
    ExpectedFractional,
    ExpectedTimestamp,
    ExpectedWeekNumber,
    ExpectedWeekdayNumber,
    ExpectedDigits,
    ExpectedMonWeekday,
    ExpectedSunWeekday,
    ExpectedMonWeek,
    ExpectedSunWeek,
    MonWeekdayOutOfRange,
    SunWeekdayOutOfRange,
    InvalidCodeId,
    NonMonotonic,
    TFieldTooShort,
    PFieldTooShort,
    InvalidSubmillisecond,
    InvalidWeekdayName,
    InvalidMonthName,
    InvalidMeridiem,
    InvalidScale,
    InvalidDate,
    InvalidTime,
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    InvalidDayOfYear,
    InvalidIsoWeekYear,
    InvalidIsoWeek,
    InvalidSunWeek,
    InvalidMonWeek,
    InvalidHour,
    InvalidMinute,
    InvalidSecond,
    InvalidFractional,
    InvalidTimestamp,
    InvalidName,
    InvalidTimeZone,
    OffsetMissingSign,
    InvalidOffsetHour,
    InvalidOffsetMinute,
    InvalidOffsetSecond,
    InvalidOffsetColons,
    InvalidOffset,
    InvalidNumber,
    InvalidItem,
    InvalidBytes,
    InvalidSyntax,
    OutOfRange,
    MonthOutOfRange,
    DayOutOfRange,
    DayOfYearOutOfRange,
    HourOutOfRange,
    MinuteOutOfRange,
    SecondOutOfRange,
    WeekOutOfRange,
    IsoWeekOutOfRange,
    YearOutOfRange,
    FracOutOfRange,
    MjdOutOfRange,
    TrailingCharacters,
    Incomplete,
    InvalidInput,
    InvalidLen,
    InternalErr,
    ConversionFail,
    IOErr,
    Empty,
}

/// Wrapper around [`AnErr`].
///
/// A [`DtErr`] object is 16 bytes.
pub type DtErr = AnErr<DtErrKind, 15>;
