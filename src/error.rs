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
    /// Input ended before a complete token or value was parsed.
    UnexpectedEnd,
    /// A format `%` directive was cut off before its letter.
    TruncatedDirective,
    /// Unrecognized format directive or parse item.
    UnknownItem,
    /// Required Cargo feature is not enabled.
    MissingFeature,
    /// A reference time or the `std` feature is required for this operation.
    MissingRefTimeOrStd,
    /// Input must begin with a specific prefix or pattern.
    MustStartWith,
    /// Format item or value type is not supported in this context.
    UnsupportedItem,
    /// A required literal character in the format did not match input.
    MismatchedLiteral,
    /// Expected a duration unit (e.g. `s`, `ms`, `day`).
    ExpectedUnit,
    /// Expected a numeric value.
    ExpectedValue,
    /// Expected a year field.
    ExpectedYear,
    /// Expected a century field.
    ExpectedCentury,
    /// Expected a month field.
    ExpectedMonth,
    /// Expected a day-of-month field.
    ExpectedDay,
    /// Expected a day-of-year field.
    ExpectedDayOfYear,
    /// Expected an hour field.
    ExpectedHour,
    /// Expected a minute field.
    ExpectedMinute,
    /// Expected a second field.
    ExpectedSecond,
    /// Expected a fractional-seconds field.
    ExpectedFractional,
    /// Expected a Unix or epoch timestamp field.
    ExpectedTimestamp,
    /// Expected a week-number field.
    ExpectedWeekNumber,
    /// Expected a weekday-number field.
    ExpectedWeekdayNumber,
    /// Expected one or more decimal digits.
    ExpectedDigits,
    /// Expected a Monday-based weekday number.
    ExpectedMonWeekday,
    /// Expected a Sunday-based weekday number.
    ExpectedSunWeekday,
    /// Expected a Monday-based week number.
    ExpectedMonWeek,
    /// Expected a Sunday-based week number.
    ExpectedSunWeek,
    /// Monday-based weekday number is outside 1–7.
    MonWeekdayOutOfRange,
    /// Sunday-based weekday number is outside 0–6 (or 1–7).
    SunWeekdayOutOfRange,
    /// Invalid binary/wire code or format identifier.
    InvalidCodeId,
    /// Sequence is not strictly increasing (e.g. leap-second table).
    NonMonotonic,
    /// CCSDS or binary T-field is shorter than required.
    TFieldTooShort,
    /// CCSDS or binary P-field is shorter than required.
    PFieldTooShort,
    /// Sub-millisecond fractional part is invalid.
    InvalidSubmillisecond,
    /// Weekday name could not be recognized.
    InvalidWeekdayName,
    /// Month name could not be recognized.
    InvalidMonthName,
    /// AM/PM (meridiem) indicator is invalid.
    InvalidMeridiem,
    /// Time scale name or code is invalid.
    InvalidScale,
    /// Calendar date is not a valid civil date.
    InvalidDate,
    /// Time-of-day is not a valid civil time.
    InvalidTime,
    /// Year value is invalid for the operation.
    InvalidYear,
    /// Month value is invalid (not 1–12, or unusable in context).
    InvalidMonth,
    /// Day-of-month is invalid for the given month/year.
    InvalidDay,
    /// Day-of-year is invalid (not 1–365/366).
    InvalidDayOfYear,
    /// ISO week-year is invalid.
    InvalidIsoWeekYear,
    /// ISO week number is invalid.
    InvalidIsoWeek,
    /// Sunday-based week number is invalid.
    InvalidSunWeek,
    /// Monday-based week number is invalid.
    InvalidMonWeek,
    /// Hour is invalid for the civil time representation.
    InvalidHour,
    /// Minute is invalid (typically not 0–59).
    InvalidMinute,
    /// Second is invalid (including leap-second rules where applicable).
    InvalidSecond,
    /// Fractional seconds field is invalid.
    InvalidFractional,
    /// Timestamp value is invalid or could not be interpreted.
    InvalidTimestamp,
    /// Name (e.g. language or locale) is invalid.
    InvalidName,
    /// Time zone identifier could not be resolved.
    InvalidTimeZone,
    /// UTC offset is missing a required `+` or `-` sign.
    OffsetMissingSign,
    /// Hour component of a UTC offset is invalid.
    InvalidOffsetHour,
    /// Minute component of a UTC offset is invalid.
    InvalidOffsetMinute,
    /// Second component of a UTC offset is invalid.
    InvalidOffsetSecond,
    /// UTC offset colon separators are malformed.
    InvalidOffsetColons,
    /// UTC offset string or value is invalid overall.
    InvalidOffset,
    /// Numeric token could not be parsed.
    InvalidNumber,
    /// Format or parse item is invalid in this context.
    InvalidItem,
    /// Byte sequence is not valid for the expected encoding.
    InvalidBytes,
    /// Input syntax is invalid.
    InvalidSyntax,
    /// A value lies outside the allowed range for this operation.
    OutOfRange,
    /// Month is outside the valid range.
    MonthOutOfRange,
    /// Day is outside the valid range.
    DayOutOfRange,
    /// Day-of-year is outside the valid range.
    DayOfYearOutOfRange,
    /// Hour is outside the valid range.
    HourOutOfRange,
    /// Minute is outside the valid range.
    MinuteOutOfRange,
    /// Second is outside the valid range.
    SecondOutOfRange,
    /// Week number is outside the valid range.
    WeekOutOfRange,
    /// ISO week number is outside the valid range.
    IsoWeekOutOfRange,
    /// Year is outside the valid range.
    YearOutOfRange,
    /// Fractional part is outside the valid range.
    FracOutOfRange,
    /// Modified Julian Date is outside the valid range.
    MjdOutOfRange,
    /// Input has unexpected characters after a complete value.
    TrailingCharacters,
    /// Parse or conversion stopped before a complete value was formed.
    Incomplete,
    /// Input is invalid for a reason not covered by a more specific kind.
    InvalidInput,
    /// Buffer or field length is invalid.
    InvalidLen,
    /// Internal invariant failed (should not occur in normal use).
    InternalErr,
    /// Conversion between representations or crates failed.
    ConversionFail,
    /// I/O error while reading or writing data (e.g. EOP file).
    IOErr,
    /// Input is empty where a value was required.
    Empty,
}

/// Wrapper around [`AnErr`].
///
/// A [`DtErr`] object is 16 bytes.
pub type DtErr = AnErr<DtErrKind, 15>;
