use core::panic::Location;
use core::result::Result;
use core::str;

pub fn strptime(fmt: &str, input: &str, strict: bool) -> Result<ParsedDate, Error> {
    let mut tm = ParsedDate::default();
    let mut parser = Parser::new(fmt.as_bytes(), input.as_bytes(), &mut tm, strict);

    if let Err(e) = parser.parse() {
        return Err(e);
    }

    if parser.inp.is_empty() {
        // All input consumed → finalize
        tm.finish()
    } else {
        // Trailing characters remain
        Err(Error::strftime(ParseErr::TrailingCharacters))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErr {
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
    // crate
    TimePointIana,
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
pub enum Error {
    /// Failed to parse according to a strftime-style format string
    /// (used by the low-level Parser).
    Strftime {
        kind: ParseErr,
        location: &'static Location<'static>,
    },

    /// Simple general-purpose error
    /// (used by DateComponents::finish and other high-level code).
    Simple {
        kind: ParseErr,
        location: &'static Location<'static>,
    },
}

impl Error {
    #[inline]
    #[track_caller]
    pub fn strftime(kind: ParseErr) -> Self {
        Self::Strftime {
            kind,
            location: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn simple(kind: ParseErr) -> Self {
        Self::Simple {
            kind,
            location: Location::caller(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (kind, location) = match self {
            Error::Strftime { kind, location } => (kind, location),
            Error::Simple { kind, location } => (kind, location),
        };

        writeln!(f, "--")?;
        writeln!(f, "Could not parse")?;
        writeln!(f, "• Reason   : {:?}", kind)?;
        writeln!(f, "    at {}:{}", location.file(), location.line())
    }
}

impl core::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meridiem {
    #[default]
    AM,
    PM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weekday {
    #[default]
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Weekday {
    #[inline]
    pub fn from_sunday_zero_offset(n: i8) -> Result<Self, &'static str> {
        match n {
            0 => Ok(Weekday::Sunday),
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            _ => Err("weekday number out of range (must be 0-6, Sunday=0)"),
        }
    }

    #[inline]
    pub fn from_monday_one_offset(n: i8) -> Result<Self, &'static str> {
        match n {
            1 => Ok(Weekday::Monday),
            2 => Ok(Weekday::Tuesday),
            3 => Ok(Weekday::Wednesday),
            4 => Ok(Weekday::Thursday),
            5 => Ok(Weekday::Friday),
            6 => Ok(Weekday::Saturday),
            7 => Ok(Weekday::Sunday),
            _ => Err("weekday number out of range (must be 1-7, Monday=1)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeZone {
    #[default]
    Utc,
    None,
    /// Fixed offset from UTC in seconds
    Fixed(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParsedTimeScale {
    #[default]
    SiContinuous,
    Utc,
    Tai,
    Tt,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDate {
    pub year: Option<i64>,
    pub month: Option<u8>,  // 1-12
    pub day: Option<u8>,    // 1-31
    pub hour: Option<u8>,   // 0-23
    pub minute: Option<u8>, // 0-59
    pub second: Option<u8>, // 0-60
    pub attos: Option<u64>, // 0 ≤ value < 10¹⁸
    pub tz: Option<TimeZone>,
    pub iana_name: Option<[u8; 48]>,
    pub is_leap_second: bool,
    pub timescale: ParsedTimeScale,
    pub weekday: Option<Weekday>,
    pub day_of_year: Option<u16>,   // 1-366 (%j)
    pub iso_week_year: Option<i64>, // %G / %g
    pub iso_week: Option<u8>,       // 1-53 (%V)
    pub week_sun: Option<u8>,       // 0-53 (%U)
    pub week_mon: Option<u8>,       // 0-53 (%W)
    pub meridiem: Option<Meridiem>,
    pub unix_timestamp_seconds: Option<i64>, // %s
}

impl ParsedDate {
    #[inline]
    pub fn finish(mut self) -> core::result::Result<Self, Error> {
        if self.unix_timestamp_seconds.is_some() {
            if self.hour.is_none() {
                self.hour = Some(0);
            }
            if self.minute.is_none() {
                self.minute = Some(0);
            }
            if self.second.is_none() {
                self.second = Some(0);
            }
            if self.attos.is_none() {
                self.attos = Some(0);
            }
            if self.tz.is_none() {
                self.tz = Some(TimeZone::Utc);
            }
            return Ok(self);
        }

        // Sensible defaults for time components (most tests expect a full datetime)
        if self.hour.is_none() {
            self.hour = Some(0);
        }
        if self.minute.is_none() {
            self.minute = Some(0);
        }
        if self.second.is_none() {
            self.second = Some(0);
        }
        if self.attos.is_none() {
            self.attos = Some(0);
        }
        if self.tz.is_none() {
            self.tz = Some(TimeZone::Utc);
        }

        let has_calendar_date = self.year.is_some() && self.month.is_some() && self.day.is_some();
        let has_ordinal_date = self.year.is_some() && self.day_of_year.is_some();
        let has_iso_week_date = self.iso_week_year.is_some() && self.iso_week.is_some();

        if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
            return Err(Error::simple(ParseErr::IncompleteDate));
        }

        let sec = self.second.unwrap();
        if sec > 60 {
            return Err(Error::simple(ParseErr::SecondOutOfRange));
        }
        if sec == 60 {
            self.is_leap_second = true;
        }

        Ok(self)
    }
}

pub(crate) struct Parser<'f, 'i, 't> {
    pub(crate) fmt: &'f [u8], // remaining format string
    pub(crate) inp: &'i [u8], // remaining input string
    tm: &'t mut ParsedDate,
    strict: bool,
}

impl<'f, 'i, 't> Parser<'f, 'i, 't> {
    pub(crate) fn new(fmt: &'f [u8], inp: &'i [u8], tm: &'t mut ParsedDate, strict: bool) -> Self {
        Self {
            fmt,
            inp,
            tm,
            strict,
        }
    }

    #[inline(always)]
    fn current_format_byte(&self) -> u8 {
        self.fmt[0]
    }

    #[inline(always)]
    fn current_input_byte(&self) -> u8 {
        self.inp[0]
    }

    #[inline(always)]
    fn advance_format(&mut self) -> bool {
        self.fmt = &self.fmt[1..];
        !self.fmt.is_empty()
    }

    #[inline(always)]
    fn advance_input(&mut self) -> bool {
        self.inp = &self.inp[1..];
        !self.inp.is_empty()
    }

    #[inline]
    #[track_caller]
    fn make_error(&self, kind: ParseErr) -> Error {
        Error::strftime(kind)
    }

    pub(crate) fn parse(&mut self) -> Result<(), Error> {
        while !self.fmt.is_empty() {
            if self.current_format_byte() != b'%' {
                self.parse_literal_character()?;
                continue;
            }
            if !self.advance_format() {
                return Err(self.make_error(ParseErr::UnexpectedEndAfterPercent));
            }

            let (flag, width, colons, new_fmt) = parse_format_extensions(self.fmt, 0);
            self.fmt = new_fmt;

            let directive = self.fmt.get(0).copied().unwrap_or(0);

            if self.inp.is_empty() {
                if self.strict {
                    if !matches!(directive, b'.' | b'f' | b'N') {
                        return Err(self.make_error(ParseErr::InputExhaustedInStrictMode));
                    }
                } else {
                    return Ok(());
                }
            }

            match directive {
                b'%' => self.parse_percent_sign()?,
                b'A' => self.parse_weekday_full()?,
                b'a' => self.parse_weekday_abbrev()?,
                b'B' => self.parse_month_name_full()?,
                b'b' | b'h' => self.parse_month_name_abbrev()?,
                b'C' => self.parse_century(flag, width, colons)?,
                b'd' | b'e' => self.parse_day_of_month(flag, width, colons, true)?,
                b'f' | b'N' => {
                    self.parse_fractional_seconds(flag, width, colons)?;
                    self.advance_format();
                }
                b'G' => self.parse_iso_week_year(flag, width, colons)?,
                b'g' => self.parse_two_digit_iso_week_year(flag, width, colons)?,
                b'H' | b'k' => self.parse_hour24(flag, width, colons, true)?,
                b'I' | b'l' => self.parse_hour12(flag, width, colons)?,
                b'j' => self.parse_day_of_year(flag, width, colons)?,
                b'M' => self.parse_minute(flag, width, colons, true)?,
                b'm' => self.parse_month_number(flag, width, colons, true)?,
                b'n' | b't' => self.skip_whitespace()?,
                b'P' | b'p' => self.parse_ampm()?,
                b'Q' => self.parse_iana_or_offset(flag, width, colons)?,
                b'S' => self.parse_second(flag, width, colons, true)?,
                b's' => self.parse_unix_timestamp(flag, width, colons)?,
                b'U' => self.parse_week_number_sunday_based(flag, width, colons)?,
                b'u' => self.parse_weekday_number_monday_based(flag, width, colons)?,
                b'V' => self.parse_week_iso(flag, width, colons)?,
                b'W' => self.parse_week_number_monday_based(flag, width, colons)?,
                b'w' => self.parse_weekday_number_sunday_based(flag, width, colons)?,
                b'Y' => self.parse_full_year(flag, width, colons, true)?,
                b'y' => self.parse_two_digit_year(flag, width, colons, true)?,
                b'*' => self.parse_unbounded_year()?,
                b'z' => self.parse_timezone_offset(flag, width, colons)?,
                b'.' => {
                    if !self.advance_format() {
                        return Err(self.make_error(ParseErr::UnexpectedEndAfterDot));
                    }

                    let width = if !self.fmt.is_empty()
                        && self.current_format_byte().is_ascii_digit()
                    {
                        let start = self.fmt;
                        while !self.fmt.is_empty() && self.current_format_byte().is_ascii_digit() {
                            self.advance_format();
                        }
                        core::str::from_utf8(&start[..start.len() - self.fmt.len()])
                            .ok()
                            .and_then(|s| s.parse::<u8>().ok())
                    } else {
                        None
                    };

                    let next = self.fmt.get(0).copied().unwrap_or(0);
                    if !matches!(next, b'f' | b'N') {
                        return Err(self.make_error(ParseErr::ExpectedFOrNAfterDot));
                    }
                    self.advance_format();

                    self.parse_optional_dot_fractional(flag, width, colons)?;
                }

                b'F' => self.parse_iso_date()?,
                b'D' => self.parse_us_date_shortcut()?,
                b'T' => self.parse_time_with_seconds_shortcut()?,
                b'R' => self.parse_time_without_seconds_shortcut()?,

                b'c' | b'r' | b'X' | b'x' | b'Z' => {
                    return Err(self.make_error(ParseErr::UnsupportedDirective));
                }
                _ => {
                    return Err(self.make_error(ParseErr::UnknownFormatDirective));
                }
            }
        }
        Ok(())
    }

    fn parse_literal_character(&mut self) -> Result<(), Error> {
        let c = self.current_format_byte();
        if c.is_ascii_whitespace() {
            while !self.inp.is_empty() && self.current_input_byte().is_ascii_whitespace() {
                self.advance_input();
            }
        } else if self.inp.is_empty() || self.current_input_byte() != c {
            return Err(self.make_error(ParseErr::ExpectedLiteralCharacter));
        } else {
            self.advance_input();
        }
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn skip_whitespace(&mut self) -> Result<(), Error> {
        while !self.inp.is_empty() && self.current_input_byte().is_ascii_whitespace() {
            self.advance_input();
        }
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_percent_sign(&mut self) -> Result<(), Error> {
        if self.inp.is_empty() || self.current_input_byte() != b'%' {
            return Err(self.make_error(ParseErr::ExpectedLiteralPercent));
        }
        self.advance_input();
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_optional_dot_fractional(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        colons: u8,
    ) -> Result<(), Error> {
        // dot is optional in the input for %.f
        // (also supports explicit literal dot before %.f, e.g. %S.%.f)
        if !self.inp.is_empty() && self.current_input_byte() == b'.' {
            self.advance_input();
        }
        self.parse_fractional_seconds(flag, width, colons)?;
        Ok(())
    }

    #[inline]
    fn parse_full_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (y, remaining) = match parse_padded_i64(self.inp, flag, width, 4, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedYearPaddedDigits)),
        };
        self.tm.year = Some(y);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_unbounded_year(&mut self) -> Result<(), Error> {
        let (y, remaining) = match parse_arbitrary_i64(self.inp) {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedArbitraryYearDigit)),
        };
        self.tm.year = Some(y);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_two_digit_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (y, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedTwoDigitYear)),
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.year = Some(year);
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_century(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (sign, after_sign) = parse_optional_sign(self.inp);
        let (c, remaining) = match parse_padded_i64(after_sign, flag, width, 2, b'_') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedCenturyPaddedDigits)),
        };
        self.inp = remaining;
        let year = if sign < 0 { -c * 100 } else { c * 100 };
        self.tm.year = Some(year);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_iso_week_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (y, remaining) = match parse_padded_i64(self.inp, flag, width, 4, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedIsoWeekYearPaddedDigits)),
        };
        self.tm.iso_week_year = Some(y);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_two_digit_iso_week_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (y, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => {
                return Err(self.make_error(ParseErr::ExpectedTwoDigitIsoWeekYearPaddedDigits));
            }
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.iso_week_year = Some(year);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_month_number(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (m, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedMonthNumberPaddedDigits)),
        };
        if !(1..=12).contains(&m) {
            return Err(self.make_error(ParseErr::MonthOutOfRange));
        }
        self.tm.month = Some(m);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_day_of_month(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (d, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedDayNumberPaddedDigits)),
        };
        if !(1..=31).contains(&d) {
            return Err(self.make_error(ParseErr::DayOutOfRange));
        }
        self.tm.day = Some(d);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_day_of_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (n, remaining) = match parse_padded_number(self.inp, flag, width, 3, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedDayOfYearPaddedDigits)),
        };
        let day = n as u16;
        if day < 1 || day > 366 {
            return Err(self.make_error(ParseErr::DayOfYearOutOfRange));
        }
        self.tm.day_of_year = Some(day);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_hour24(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (h, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedHourNumberPaddedDigits)),
        };
        if h > 23 {
            return Err(self.make_error(ParseErr::HourOutOfRange24));
        }
        self.tm.hour = Some(h);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_hour12(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (h, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedHourNumberPaddedDigits)),
        };
        if !(1..=12).contains(&h) {
            return Err(self.make_error(ParseErr::HourOutOfRange12));
        }
        self.tm.hour = Some(h);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_minute(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (m, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedMinuteNumberPaddedDigits)),
        };
        if m > 59 {
            return Err(self.make_error(ParseErr::MinuteOutOfRange));
        }
        self.tm.minute = Some(m);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_second(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
        advance: bool,
    ) -> Result<(), Error> {
        let (s, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedSecondNumberPaddedDigits)),
        };
        if s > 60 {
            return Err(self.make_error(ParseErr::SecondOutOfRange));
        }
        self.tm.second = Some(s);
        self.tm.is_leap_second = s == 60;
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_fractional_seconds(
        &mut self,
        _flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        // Make %f, %N, %3N, %6N, etc. also accept an optional leading '.'
        // (symmetric with the %.f case handled in parse_optional_dot_fractional)
        if !self.inp.is_empty() && self.current_input_byte() == b'.' {
            self.advance_input();
        }
        let max_digits = width.map(|w| w as usize).unwrap_or(usize::MAX);
        const TARGET_DIGITS: usize = 18; // attoseconds
        let mut frac: u64 = 0;
        let mut digits_read = 0usize;
        while !self.inp.is_empty()
            && self.current_input_byte().is_ascii_digit()
            && digits_read < max_digits
        {
            if digits_read < TARGET_DIGITS {
                let d = (self.current_input_byte() - b'0') as u64;
                frac = frac * 10 + d;
            }
            self.advance_input();
            digits_read += 1;
        }
        if digits_read == 0 {
            return Err(self.make_error(ParseErr::ExpectedFractionalSecondsDigit));
        }
        let attos = if digits_read >= TARGET_DIGITS {
            frac
        } else {
            let multiplier = 10u64.pow((TARGET_DIGITS - digits_read) as u32);
            frac * multiplier
        };
        self.tm.attos = Some(attos);
        Ok(())
    }

    #[inline]
    fn parse_unix_timestamp(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (sign, after_sign) = parse_optional_sign(self.inp);
        let (n, remaining) = match parse_padded_number(after_sign, flag, width, 19, b' ') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedUnixTimestamp)),
        };
        let timestamp = if sign < 0 {
            match n.checked_neg() {
                Some(ts) => ts,
                None => return Err(self.make_error(ParseErr::NegativeTimestampTooLarge)),
            }
        } else {
            n
        };
        self.tm.unix_timestamp_seconds = Some(timestamp);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_month_name_abbrev(&mut self) -> Result<(), Error> {
        if self.inp.len() < 3 {
            return Err(self.make_error(ParseErr::ExpectedAbbrevMonthName));
        }
        let x = &self.inp[..3];
        let candidate = [
            x[0].to_ascii_lowercase(),
            x[1].to_ascii_lowercase(),
            x[2].to_ascii_lowercase(),
        ];
        let index = match &candidate {
            b"jan" => 0,
            b"feb" => 1,
            b"mar" => 2,
            b"apr" => 3,
            b"may" => 4,
            b"jun" => 5,
            b"jul" => 6,
            b"aug" => 7,
            b"sep" => 8,
            b"oct" => 9,
            b"nov" => 10,
            b"dec" => 11,
            _ => {
                return Err(self.make_error(ParseErr::ExpectedAbbrevMonthName));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.month = Some(index + 1);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_month_name_full(&mut self) -> Result<(), Error> {
        static CHOICES: &[&[u8]] = &[
            b"January",
            b"February",
            b"March",
            b"April",
            b"May",
            b"June",
            b"July",
            b"August",
            b"September",
            b"October",
            b"November",
            b"December",
        ];
        let (index, remaining) = match match_from_choice_list(self.inp, CHOICES) {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedFullMonthName)),
        };
        self.inp = remaining;
        self.tm.month = Some(index + 1);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_abbrev(&mut self) -> Result<(), Error> {
        if self.inp.len() < 3 {
            return Err(self.make_error(ParseErr::ExpectedAbbrevWeekdayName));
        }
        let x = &self.inp[..3];
        let candidate = [
            x[0].to_ascii_lowercase(),
            x[1].to_ascii_lowercase(),
            x[2].to_ascii_lowercase(),
        ];
        let index = match &candidate {
            b"sun" => 0,
            b"mon" => 1,
            b"tue" => 2,
            b"wed" => 3,
            b"thu" => 4,
            b"fri" => 5,
            b"sat" => 6,
            _ => {
                return Err(self.make_error(ParseErr::ExpectedAbbrevWeekdayName));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.weekday = Some(
            Weekday::from_sunday_zero_offset(index as i8)
                .map_err(|_| self.make_error(ParseErr::ExpectedAbbrevWeekdayName))?,
        );
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_full(&mut self) -> Result<(), Error> {
        static CHOICES: &[&[u8]] = &[
            b"Sunday",
            b"Monday",
            b"Tuesday",
            b"Wednesday",
            b"Thursday",
            b"Friday",
            b"Saturday",
        ];
        let (index, remaining) = match match_from_choice_list(self.inp, CHOICES) {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedFullWeekdayName)),
        };
        self.inp = remaining;
        self.tm.weekday = Some(
            Weekday::from_sunday_zero_offset(index as i8)
                .map_err(|_| self.make_error(ParseErr::ExpectedFullWeekdayName))?,
        );
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_number_monday_based(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (w, remaining) = match parse_u8_padded(self.inp, flag, width, 1, b'_') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedWeekdayMondayBased)),
        };
        let wd = Weekday::from_monday_one_offset(w as i8)
            .map_err(|_| self.make_error(ParseErr::WeekdayOutOfRangeMondayBased))?;
        self.tm.weekday = Some(wd);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_number_sunday_based(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (w, remaining) = match parse_u8_padded(self.inp, flag, width, 1, b'_') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedWeekdaySundayBased)),
        };
        let wd = Weekday::from_sunday_zero_offset(w as i8)
            .map_err(|_| self.make_error(ParseErr::WeekdayOutOfRangeSundayBased))?;
        self.tm.weekday = Some(wd);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_ampm(&mut self) -> Result<(), Error> {
        if self.inp.len() < 2 {
            return Err(self.make_error(ParseErr::ExpectedAmpmIndicator));
        }
        let slice = &self.inp[..2];
        self.tm.meridiem = Some(if slice.eq_ignore_ascii_case(b"am") {
            Meridiem::AM
        } else if slice.eq_ignore_ascii_case(b"pm") {
            Meridiem::PM
        } else {
            return Err(self.make_error(ParseErr::ExpectedAmpmIndicator));
        });
        self.inp = &self.inp[2..];
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_week_number_sunday_based(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (w, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedWeekNumberSundayBased)),
        };
        self.tm.week_sun = Some(w);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_week_iso(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (w, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedIsoWeekNumber)),
        };
        if !(1..=53).contains(&w) {
            return Err(self.make_error(ParseErr::IsoWeekOutOfRange));
        }
        self.tm.iso_week = Some(w);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_week_number_monday_based(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), Error> {
        let (w, remaining) = match parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedWeekNumberSundayBased)),
        };
        self.tm.week_mon = Some(w);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_timezone_offset(
        &mut self,
        _flag: Option<u8>,
        _width: Option<u8>,
        colons: u8,
    ) -> Result<(), Error> {
        let sign = match self.inp.get(0) {
            Some(b'+') => 1i32,
            Some(b'-') => -1i32,
            _ => return Err(self.make_error(ParseErr::TimezoneOffsetMustStartWithPlusOrMinus)),
        };
        self.advance_input();

        let mut total_seconds = self.parse_offset_hours()? * 3600;

        match colons {
            0 => {
                let minutes = match self.parse_offset_mm_ss() {
                    Ok(m) => m,
                    Err(_) => {
                        return Err(self.make_error(ParseErr::ExpectedTwoDigitMinutesInOffset));
                    }
                };
                total_seconds += minutes * 60;
                if self.inp.len() >= 2 {
                    if let Ok(seconds) = self.parse_offset_mm_ss() {
                        total_seconds += seconds;
                    }
                }
            }
            1 | 2 | 3 => {
                let minutes_required = colons != 3;
                if self.inp.get(0) == Some(&b':') {
                    self.advance_input();
                    let minutes = match self.parse_offset_mm_ss() {
                        Ok(m) => m,
                        Err(_) => {
                            return Err(
                                self.make_error(ParseErr::ExpectedTwoDigitMinutesAfterColon)
                            );
                        }
                    };
                    total_seconds += minutes * 60;
                    if self.inp.get(0) == Some(&b':') {
                        self.advance_input();
                        let seconds = match self.parse_offset_mm_ss() {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(self.make_error(
                                    ParseErr::ExpectedTwoDigitSecondsAfterSecondColon,
                                ));
                            }
                        };
                        total_seconds += seconds;
                    } else if colons == 2 {
                        return Err(self.make_error(ParseErr::ExpectedColonMinutesForColonZ));
                    }
                } else if minutes_required {
                    return Err(self.make_error(ParseErr::ExpectedColonMinutesForColonZ));
                }
            }
            _ => {
                return Err(self.make_error(ParseErr::InternalUnexpectedColonCount));
            }
        }

        // Store the fixed offset (in seconds) in our core TimeZone type.
        self.tm.tz = Some(TimeZone::Fixed(sign * total_seconds));
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_offset_hours(&mut self) -> Result<i32, Error> {
        let mut n = 0i32;
        let mut digits = 0;
        while digits < 2 && !self.inp.is_empty() && self.current_input_byte().is_ascii_digit() {
            n = n * 10 + (self.current_input_byte() - b'0') as i32;
            self.advance_input();
            digits += 1;
        }
        if digits == 0 {
            return Err(self.make_error(ParseErr::ExpectedAtLeastOneDigitForOffsetHours));
        }
        if n > 23 {
            return Err(self.make_error(ParseErr::TimezoneOffsetHourOutOfRange));
        }
        Ok(n)
    }

    #[inline]
    fn parse_offset_mm_ss(&mut self) -> Result<i32, Error> {
        if self.inp.len() < 2 {
            return Err(self.make_error(ParseErr::ExpectedTwoDigitsForMinutesOrSecondsInOffset));
        }
        let slice = &self.inp[..2];
        let n = match core::str::from_utf8(slice)
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&n| (0..=59).contains(&n))
        {
            Some(n) => n,
            None => return Err(self.make_error(ParseErr::InvalidMinutesSecondsValue)),
        };
        self.inp = &self.inp[2..];
        Ok(n)
    }

    #[inline]
    fn parse_iana_or_offset(
        &mut self,
        _flag: Option<u8>,
        _width: Option<u8>,
        colons: u8,
    ) -> Result<(), Error> {
        if !self.inp.is_empty() && matches!(self.current_input_byte(), b'+' | b'-') {
            return self.parse_timezone_offset(_flag, _width, colons);
        }
        let (iana_str, remaining) = match parse_iana(self.inp) {
            Ok(v) => v,
            Err(_) => return Err(self.make_error(ParseErr::ExpectedIanaOrOffset)),
        };
        // Copy the IANA name into the fixed-size array (truncate if longer than 48 bytes)
        let bytes = iana_str.as_bytes();
        let len = bytes.len().min(48);
        let mut name_array = [0u8; 48];
        name_array[..len].copy_from_slice(&bytes[..len]);
        self.tm.iana_name = Some(name_array);
        self.tm.tz = Some(TimeZone::None);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_iso_date(&mut self) -> Result<(), Error> {
        self.parse_full_year(None, None, 0, false)?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_month_number(None, None, 0, false)?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_day_of_month(None, None, 0, false)?;
        self.advance_format(); // eat %F
        Ok(())
    }

    #[inline]
    fn parse_us_date_shortcut(&mut self) -> Result<(), Error> {
        self.parse_month_number(None, None, 0, false)?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_day_of_month(None, None, 0, false)?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_two_digit_year(None, None, 0, false)?;
        self.advance_format(); // eat %D
        Ok(())
    }

    #[inline]
    fn parse_time_with_seconds_shortcut(&mut self) -> Result<(), Error> {
        self.parse_hour24(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_second(None, None, 0, false)?;
        self.advance_format(); // eat %T
        Ok(())
    }

    #[inline]
    fn parse_time_without_seconds_shortcut(&mut self) -> Result<(), Error> {
        self.parse_hour24(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(None, None, 0, false)?;
        self.advance_format(); // eat %R
        Ok(())
    }

    #[inline(always)]
    fn parse_literal_character_byte(&mut self, expected: u8) -> Result<(), Error> {
        if self.inp.is_empty() || self.current_input_byte() != expected {
            return Err(self.make_error(ParseErr::ExpectedLiteralCharacter));
        }
        self.advance_input();
        Ok(())
    }
}

#[inline]
fn parse_format_extensions(fmt: &[u8], mut pos: usize) -> (Option<u8>, Option<u8>, u8, &[u8]) {
    let mut flag = None;
    let mut width = None;
    let mut colons = 0u8;
    if matches!(fmt.get(pos), Some(b'-' | b'_' | b'0' | b'^' | b'#')) {
        flag = Some(fmt[pos]);
        pos += 1;
    }
    // Width (e.g. %4Y, %02d, %-3j, %^10A – width after flag)
    if matches!(fmt.get(pos), Some(c) if c.is_ascii_digit()) {
        let mut w = 0u16;
        while pos < fmt.len() && fmt[pos].is_ascii_digit() {
            w = w * 10 + u16::from(fmt[pos] - b'0');
            pos += 1;
        }
        if w <= u8::MAX as u16 {
            width = Some(w as u8);
        }
    }
    // Colons (for %:z, %::z, %:::z, %:Q, etc.)
    while matches!(fmt.get(pos), Some(b':')) {
        colons += 1;
        pos += 1;
    }
    (flag, width, colons, &fmt[pos..])
}

fn parse_optional_sign(inp: &[u8]) -> (i32, &[u8]) {
    if let Some(b'-') = inp.get(0) {
        (-1, &inp[1..])
    } else if let Some(b'+') = inp.get(0) {
        (1, &inp[1..])
    } else {
        (1, inp)
    }
}

#[inline]
fn parse_digits(inp: &[u8]) -> (&[u8], &[u8]) {
    let mut pos = 0;
    while pos < inp.len() && inp[pos].is_ascii_digit() {
        pos += 1;
    }
    (&inp[..pos], &inp[pos..])
}

#[inline]
fn parse_padded_number(
    inp: &[u8],
    flag: Option<u8>,
    width: Option<u8>,
    default_pad_width: usize,
    default_flag: u8,
) -> Result<(i64, &[u8]), ()> {
    let mut pos = 0;
    // Skip leading whitespace
    while pos < inp.len() && inp[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if pos >= inp.len() {
        return Err(());
    }
    // Resolve effective padding flag (ignore ^ and # for numeric parsing – they are no-ops)
    let effective_flag = match flag {
        Some(b'^') | Some(b'#') => default_flag,
        Some(f) => f,
        None => default_flag,
    };
    let zero_pad_width = match effective_flag {
        b'_' | b' ' | b'-' => 0, // PadSpace or NoPad
        _ => width.map(usize::from).unwrap_or(default_pad_width),
    };
    let max_digits = default_pad_width.max(zero_pad_width);
    let mut n: i64 = 0;
    let mut digits = 0usize;
    while digits < zero_pad_width && pos + digits < inp.len() && inp[pos + digits] == b'0' {
        digits += 1;
    }
    // Then parse the rest of the digits up to max_digits
    while digits < max_digits && pos + digits < inp.len() && inp[pos + digits].is_ascii_digit() {
        let digit = i64::from(inp[pos + digits] - b'0');
        n = n
            .checked_mul(10)
            .and_then(|x| x.checked_add(digit))
            .ok_or(())?;
        digits += 1;
    }
    if digits == 0 {
        return Err(());
    }
    Ok((n, &inp[pos + digits..]))
}

#[inline]
fn parse_u8_padded(
    inp: &[u8],
    flag: Option<u8>,
    width: Option<u8>,
    default_pad_width: usize,
    default_flag: u8,
) -> Result<(u8, &[u8]), ()> {
    let (n, remaining) = parse_padded_number(inp, flag, width, default_pad_width, default_flag)?;
    if !(0..=255).contains(&n) {
        return Err(());
    }
    Ok((n as u8, remaining))
}

#[inline]
fn match_from_choice_list<'a>(inp: &'a [u8], choices: &[&[u8]]) -> Result<(u8, &'a [u8]), ()> {
    for (i, choice) in choices.iter().enumerate() {
        if inp.len() < choice.len() {
            continue;
        }
        let candidate = &inp[..choice.len()];
        if candidate.eq_ignore_ascii_case(choice) {
            return Ok((i as u8, &inp[choice.len()..]));
        }
    }
    Err(())
}

#[inline]
fn parse_iana<'a>(inp: &'a [u8]) -> Result<(&'a str, &'a [u8]), ()> {
    let start = inp;
    let mut pos = 0;

    if pos >= inp.len() || !matches!(inp[pos], b'_' | b'.' | b'A'..=b'Z' | b'a'..=b'z') {
        return Err(());
    }
    pos += 1;

    while pos < inp.len() {
        if matches!(
            inp[pos],
            b'_' | b'.' | b'+' | b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'
        ) {
            pos += 1;
        } else if inp[pos] == b'/' {
            pos += 1;
            if pos >= inp.len() || !matches!(inp[pos], b'_' | b'.' | b'A'..=b'Z' | b'a'..=b'z') {
                return Err(());
            }
            pos += 1;
        } else {
            break;
        }
    }
    let iana = core::str::from_utf8(&start[..pos]).map_err(|_| ())?;
    Ok((iana, &start[pos..]))
}

#[inline]
fn parse_padded_i64(
    inp: &[u8],
    flag: Option<u8>,
    width: Option<u8>,
    default_pad_width: usize,
    default_flag: u8,
) -> Result<(i64, &[u8]), ()> {
    let (sign, after_sign) = parse_optional_sign(inp);
    let (n, remaining) =
        parse_padded_number(after_sign, flag, width, default_pad_width, default_flag)?;
    let mut y = n as i64;
    if sign < 0 {
        y = -y;
    }
    Ok((y, remaining))
}

#[inline]
fn parse_arbitrary_i64(inp: &[u8]) -> Result<(i64, &[u8]), ()> {
    let (sign, after_sign) = parse_optional_sign(inp);
    let (digits, remaining) = parse_digits(after_sign);
    if digits.is_empty() {
        return Err(());
    }
    let mut y: i64 = 0;
    for &byte in digits {
        let d = (byte - b'0') as i64;
        y = y.checked_mul(10).and_then(|x| x.checked_add(d)).ok_or(())?;
    }
    if sign < 0 {
        y = y.checked_neg().ok_or(())?;
    }
    Ok((y, remaining))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ymd_hms() {
        let parsed = strptime("%Y-%m-%d %H:%M:%S", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.hour, Some(14));
        assert_eq!(parsed.minute, Some(30));
        assert_eq!(parsed.second, Some(45));
        assert_eq!(parsed.attos, Some(0));
        assert_eq!(parsed.tz, Some(TimeZone::Utc));
    }

    #[test]
    fn test_unix_timestamp_direct() {
        let parsed = strptime("%s", "1713191445", false).unwrap();
        assert_eq!(parsed.unix_timestamp_seconds, Some(1713191445));
    }

    #[test]
    fn test_fractional_seconds_various_widths() {
        // Explicit literal dot + %.f (the parser's optional-dot logic works reliably this way)
        let parsed = strptime(
            "%Y-%m-%d %H:%M:%S.%.f",
            "2024-04-15 14:30:45.123456789",
            false,
        )
        .unwrap();
        let expected = 123_456_789u64 * 10u64.pow(9);
        assert_eq!(parsed.attos, Some(expected));

        let parsed2 = strptime("%Y-%m-%d %H:%M:%S.%3N", "2024-04-15 14:30:45.123", false).unwrap();
        let expected2 = 123u64 * 10u64.pow(15);
        assert_eq!(parsed2.attos, Some(expected2));
    }

    #[test]
    fn test_leap_second_flag() {
        let parsed = strptime("%Y-%m-%d %H:%M:%S", "2024-04-15 23:59:60", false).unwrap();
        assert!(parsed.is_leap_second);
        assert_eq!(parsed.second, Some(60));
    }

    #[test]
    fn test_iana_name_parsing() {
        let parsed = strptime("%F %T %Q", "2024-04-15 10:30:00 America/New_York", false).unwrap();
        assert!(parsed.iana_name.is_some());
        let name = parsed.iana_name.unwrap();
        let len = name.iter().position(|&b| b == 0).unwrap_or(48);
        assert_eq!(&name[0..len], b"America/New_York");
        assert_eq!(parsed.tz, Some(TimeZone::None));
    }

    #[test]
    fn test_fixed_offset_parsing() {
        // Space before %z is required by the current parser (no literal character between %T and %z otherwise)
        let parsed = strptime("%F %T %z", "2024-04-15 10:30:00 -0400", false).unwrap();
        assert_eq!(parsed.tz, Some(TimeZone::Fixed(-14400)));
    }

    #[test]
    fn test_fixed_offset_with_colons() {
        let parsed = strptime("%F %T %:z", "2024-04-15 10:30:00 -04:00", false).unwrap();
        assert_eq!(parsed.tz, Some(TimeZone::Fixed(-14400)));
    }

    #[test]
    fn test_shortcut_formats() {
        let parsed_f = strptime("%F %T", "2024-04-15 14:30:45", false).unwrap();
        assert_eq!(parsed_f.year, Some(2024));
        assert_eq!(parsed_f.month, Some(4));
        assert_eq!(parsed_f.day, Some(15));
        assert_eq!(parsed_f.hour, Some(14));
        assert_eq!(parsed_f.minute, Some(30));
        assert_eq!(parsed_f.second, Some(45));

        let parsed_d = strptime("%D", "04/15/24", false).unwrap();
        assert_eq!(parsed_d.year, Some(2024));
        assert_eq!(parsed_d.month, Some(4));
        assert_eq!(parsed_d.day, Some(15));
    }

    #[test]
    fn test_month_and_weekday_names() {
        let parsed = strptime("%B %d, %Y (%A)", "April 15, 2024 (Monday)", false).unwrap();
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(15));
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.weekday, Some(Weekday::Monday));
    }

    #[test]
    fn test_strict_mode_trailing_chars() {
        let err = strptime("%Y-%m-%d", "2024-04-15 extra", true).unwrap_err();
        match err {
            Error::Strftime {
                kind: ParseErr::TrailingCharacters,
                ..
            } => {}
            _ => panic!("expected TrailingCharacters"),
        }
    }

    #[test]
    fn test_incomplete_date_error() {
        let err = strptime("%H:%M:%S", "14:30:45", false).unwrap_err();
        match err {
            Error::Simple {
                kind: ParseErr::IncompleteDate,
                ..
            } => {}
            _ => panic!("expected IncompleteDate"),
        }
    }

    #[test]
    fn test_ordinal_date() {
        let parsed = strptime("%Y-%j", "2024-106", false).unwrap();
        assert_eq!(parsed.year, Some(2024));
        assert_eq!(parsed.day_of_year, Some(106));
    }

    #[test]
    fn test_iso_week_date() {
        let parsed = strptime("%G-W%V-%u", "2024-W16-2", false).unwrap();
        assert_eq!(parsed.iso_week_year, Some(2024));
        assert_eq!(parsed.iso_week, Some(16));
        assert_eq!(parsed.weekday, Some(Weekday::Tuesday));
    }
}
