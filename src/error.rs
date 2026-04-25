use core::panic::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtErrKind {
    // from_str parser errors
    UnexpectedEndAfterPercent,
    InputExhaustedInStrictMode,
    UnexpectedEndAfterDot,
    ExpectedFOrNAfterDot,
    UnsupportedDirective,
    UnknownFormatDirective,
    ExpectedLiteralCharacter,
    ExpectedLiteralPercent,
    ExpectedYearPaddedDigits,
    ExpectedArbitraryYearDigit,
    ExpectedTwoDigitYear,
    ExpectedCenturyPaddedDigits,
    ExpectedIsoWeekYearPaddedDigits,
    ExpectedTwoDigitIsoWeekYearPaddedDigits,
    ExpectedMonthNumberPaddedDigits,
    MonthOutOfRange,
    ExpectedDayNumberPaddedDigits,
    DayOutOfRange,
    ExpectedDayOfYearPaddedDigits,
    DayOfYearOutOfRange,
    ExpectedHourNumberPaddedDigits,
    HourOutOfRange24,
    HourOutOfRange12,
    ExpectedMinuteNumberPaddedDigits,
    MinuteOutOfRange,
    ExpectedSecondNumberPaddedDigits,
    SecondOutOfRange,
    ExpectedFractionalSecondsDigit,
    ExpectedUnixTimestamp,
    NegativeTimestampTooLarge,
    ExpectedAbbrevMonthName,
    ExpectedFullMonthName,
    ExpectedAbbrevWeekdayName,
    ExpectedFullWeekdayName,
    ExpectedWeekdayMondayBased,
    WeekdayOutOfRangeMondayBased,
    ExpectedWeekdaySundayBased,
    WeekdayOutOfRangeSundayBased,
    ExpectedAmpmIndicator,
    ExpectedWeekNumberSundayBased,
    ExpectedIsoWeekNumber,
    IsoWeekOutOfRange,
    TimezoneOffsetMustStartWithPlusOrMinus,
    ExpectedTwoDigitMinutesInOffset,
    ExpectedTwoDigitMinutesAfterColon,
    ExpectedTwoDigitSecondsAfterSecondColon,
    ExpectedColonMinutesForColonZ,
    ExpectedAtLeastOneDigitForOffsetHours,
    TimezoneOffsetHourOutOfRange,
    ExpectedTwoDigitsForMinutesOrSecondsInOffset,
    InvalidMinutesSecondsValue,
    ExpectedIanaOrOffset,
    InternalUnexpectedColonCount,
    IncompleteDate,
    AssemblyFailed,
    TrailingCharacters,
    InvalidClockType,
    UnknownClockType,
    // chrono
    ChronoNaiveDate,
    ChronoNaiveTime,
    ChronoOffset,
    ChronoDateTime,
    Chrono,
    // jiff
    JiffBrokenDownTime,
    JiffTimestamp,
    JiffTimeZone,
    JiffToZoned,
    // crate
    TimePointIana,
    TimePointIanaFromBytes,
    TimePointTimeZone,
    TimePointYearIncompleteDate,
    TimePointJdnIncompleteDate,
    TimePointDayOfYearOutOfRange,
    TimePointIsoWeekOutOfRange,
    TimePointJdnIsNone,
    TimePointHourOutOfRange,
    TimePointInvalidDate,
    // output, formatter
    FormatterErr,
    // ccsds
    CCSDSStrNoYear,
    CCSDSStrInvalidDate,
    CCSDSStrFromUtf8Err,
    CCSDSStrInvalidMonth,
    CCSDSStrInvalidDay,
    CCSDSStrInvalidHour,
    CCSDSStrInvalidMinute,
    CCSDSStrInvalidSecond,
    CCSDSStrInvalidRequiredTimeSeparator,
    /// Input buffer was empty when a CCSDS CUC/CDS time code was expected.
    CCSDSBinEmpty,
    /// Input buffer was shorter than required by the P-field / T-field length fields
    /// (covers both missing extension octet and insufficient T-field bytes).
    CCSDSBinTooShort,
    /// The 3-bit Code ID in the P-field was neither `001` (CUC) nor `100` (CDS),
    /// or did not match the expected value for the specific parser.
    CCSDSBinInvalidCodeId,
    /// Bit 0 of the second P-field octet was set, indicating a 3+-byte P-field
    /// (further extension). Only 1- or 2-byte P-fields are supported for Level 1.
    CCSDSBinInvalidPFieldExtension,
    /// The Epoch bit (bit 4 of first P-field octet) was set in a CDS packet.
    /// Only Epoch=0 (1958-01-01 UTC) is supported for Level 1.
    CCSDSBinInvalidEpoch,
    /// Bits 6-7 of the first P-field octet in a CDS packet encoded 0b11,
    /// which is reserved / unsupported.
    CCSDSBinInvalidSubMillisecondCode,
}

#[derive(Debug)]
pub struct DtError {
    pub kind: DtErrKind,
    location: &'static Location<'static>,
}

impl DtError {
    #[inline]
    #[track_caller]
    pub fn new(kind: DtErrKind) -> Self {
        Self {
            kind,
            location: Location::caller(),
        }
    }
}

impl core::fmt::Display for DtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "--")?;
        writeln!(f, "deep-time Error")?;
        writeln!(f, "• Reason   : {:?}", self.kind)?;
        writeln!(
            f,
            "    at {}:{}",
            self.location.file(),
            self.location.line()
        )
    }
}

impl core::error::Error for DtError {}

impl From<DtErrKind> for DtError {
    #[track_caller]
    #[inline]
    fn from(kind: DtErrKind) -> Self {
        Self::new(kind)
    }
}
