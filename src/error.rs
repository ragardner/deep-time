use core::panic::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtErrKind {
    // strptime parser errors
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
    //
    StrCCSDSNoYear,
    StrCCSDSInvalidDate,
    StrCCSDSFromUtf8Err,
    StrCCSDSInvalidMonth,
    StrCCSDSInvalidDay,
    StrCCSDSInvalidHour,
    StrCCSDSInvalidMinute,
    StrCCSDSInvalidSecond,
    StrCCSDSInvalidRequiredTimeSeparator,
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
