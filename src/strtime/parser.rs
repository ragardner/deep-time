use super::{FormatExtensions, FormatFlag};
use crate::error::{DtErr, DtErrKind};
use crate::locale::en::{EN_MONTHS_FULL, EN_WEEKDAYS_FULL};
use crate::{
    ATTOS_DIGITS, ATTOS_PER_SEC_I128, Epoch, Meridiem, Offset, Parts, Scale, Timestamp, Weekday,
    an_err,
};
use core::result::Result;
use core::str;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Sign {
    Positive,
    Negative,
}

pub(crate) struct Parser<'f, 'i, 't> {
    pub(crate) fmt: &'f [u8], // remaining format string
    pub(crate) inp: &'i [u8], // remaining input string
    tm: &'t mut Parts,
    /// When true, the parser will succeed regardless of whether
    /// the format string has not been fully consumed. Essentially
    /// parse what you can.
    inp_can_end_before_fmt: bool,
}

impl<'f, 'i, 't> Parser<'f, 'i, 't> {
    #[inline(always)]
    pub(crate) fn new(
        fmt: &'f [u8],
        inp: &'i [u8],
        tm: &'t mut Parts,
        inp_can_end_before_fmt: bool,
    ) -> Self {
        Self {
            fmt,
            inp,
            tm,
            inp_can_end_before_fmt,
        }
    }

    #[inline(always)]
    fn current_fmt_byte(&self) -> u8 {
        self.fmt[0]
    }

    #[inline(always)]
    fn current_inp_byte(&self) -> u8 {
        self.inp[0]
    }

    #[inline(always)]
    fn advance_fmt_checked(&mut self) -> bool {
        self.fmt = &self.fmt[1..];
        !self.fmt.is_empty()
    }

    #[inline(always)]
    fn advance_fmt(&mut self) {
        self.fmt = &self.fmt[1..];
    }

    #[inline(always)]
    fn advance_inp(&mut self) {
        self.inp = &self.inp[1..];
    }

    #[inline(always)]
    pub(crate) fn parse(&mut self) -> Result<(), DtErr> {
        while !self.fmt.is_empty() {
            // When true, we are allowed to stop as soon as the input is exhausted.
            if self.inp_can_end_before_fmt && self.inp.is_empty() {
                return Ok(());
                // When false we fall through and produce normal "expected X" / mismatch errors.
            }

            if self.current_fmt_byte() != b'%' {
                self.parse_literal_character()?;
                continue;
            }
            if !self.advance_fmt_checked() {
                return Err(an_err!(DtErrKind::TruncatedDirective));
            }

            let ext = self.parse_fmt_extensions();

            let directive = match self.fmt.first() {
                Some(b) => *b,
                None => return Err(an_err!(DtErrKind::TruncatedDirective)),
            };

            match directive {
                b'Y' => {
                    self.parse_full_year(ext)?;
                    self.advance_fmt();
                }
                b'm' => {
                    self.parse_month_number(ext)?;
                    self.advance_fmt();
                }
                b'd' | b'e' => {
                    self.parse_day_of_month(ext)?;
                    self.advance_fmt();
                }
                b'H' | b'k' => {
                    self.parse_hour24(ext)?;
                    self.advance_fmt();
                }
                b'M' => {
                    self.parse_minute(ext)?;
                    self.advance_fmt();
                }
                b'S' => {
                    self.parse_second(ext)?;
                    self.advance_fmt();
                }
                b'Q' => self.parse_iana_or_offset(ext.colons)?,
                b'z' => self.parse_timezone_offset(ext.colons)?,
                b'A' => self.parse_weekday_full()?,
                b'a' => self.parse_weekday_abbrev()?,
                b'B' => self.parse_month_name_full()?,
                b'b' | b'h' => self.parse_month_name_abbrev()?,
                b'f' | b'N' => {
                    self.parse_fractional_seconds(ext.width)?;
                    self.advance_fmt();
                }
                b'I' | b'l' => self.parse_hour12(ext)?,
                b'y' => {
                    self.parse_two_digit_year(ext)?;
                    self.advance_fmt();
                }
                b'.' => {
                    if !self.advance_fmt_checked() {
                        return Err(an_err!(DtErrKind::ExpectedFractional));
                    }

                    let width = if !self.fmt.is_empty() && self.current_fmt_byte().is_ascii_digit()
                    {
                        let start = self.fmt;
                        while !self.fmt.is_empty() && self.current_fmt_byte().is_ascii_digit() {
                            self.advance_fmt();
                        }
                        core::str::from_utf8(&start[..start.len() - self.fmt.len()])
                            .ok()
                            .and_then(|s| s.parse::<u8>().ok())
                    } else {
                        None
                    };

                    let next: u8 = self.fmt.first().copied().unwrap_or(0);
                    if !matches!(next, b'f' | b'N') {
                        return Err(an_err!(DtErrKind::InvalidFractional));
                    }
                    self.advance_fmt();

                    self.parse_optional_dot_fractional(width)?;
                }
                b'P' | b'p' => self.parse_ampm()?,
                b'j' => self.parse_day_of_year(ext)?,
                b'C' => self.parse_century(ext)?,
                b'G' => self.parse_iso_week_year(ext)?,
                b'g' => self.parse_two_digit_iso_week_year(ext)?,
                b'n' | b't' => self.skip_whitespace(),
                b's' => self.parse_timestamp_sec(ext, Epoch::Unix)?,
                b'U' => self.parse_week_number_sunday_based(ext)?,
                b'u' => self.parse_weekday_number_monday_based(ext)?,
                b'V' => self.parse_week_iso(ext)?,
                b'W' => self.parse_week_number_monday_based(ext)?,
                b'w' => self.parse_weekday_number_sunday_based(ext)?,
                b'%' => self.parse_percent_sign()?,
                // shortcuts
                b'F' => self.parse_iso_date()?,
                b'D' => self.parse_us_date_shortcut()?,
                b'T' => self.parse_time_with_seconds_shortcut()?,
                b'R' => self.parse_time_without_seconds_shortcut()?,
                // Library directives
                b'J' => self.parse_timestamp_sec(ext, Epoch::Noon2000)?,
                b'*' => self.parse_unbounded_year()?,
                b'L' => self.parse_scale()?,
                b'c' | b'r' | b'X' | b'x' | b'Z' => {
                    return Err(an_err!(
                        DtErrKind::UnsupportedItem,
                        "{}",
                        char::from(directive)
                    ));
                }
                _ => {
                    return Err(an_err!(DtErrKind::UnknownItem, "{}", char::from(directive)));
                }
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn parse_literal_character(&mut self) -> Result<(), DtErr> {
        let c = self.current_fmt_byte();
        if c.is_ascii_whitespace() {
            while self.inp.get(0).map_or(false, |b| b.is_ascii_whitespace()) {
                self.inp = &self.inp[1..];
            }
        } else {
            if self.inp.is_empty() {
                return Err(an_err!(DtErrKind::UnexpectedEnd));
            } else if self.current_inp_byte() != c {
                return Err(an_err!(
                    DtErrKind::MismatchedLiteral,
                    "{} got: {}",
                    char::from(c),
                    char::from(self.current_inp_byte())
                ));
            }

            self.advance_inp();
        }
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn skip_whitespace(&mut self) {
        self.inp = self.inp.trim_ascii_start();
        self.advance_fmt();
    }

    #[inline(always)]
    fn parse_percent_sign(&mut self) -> Result<(), DtErr> {
        if self.inp.is_empty() {
            return Err(an_err!(DtErrKind::UnexpectedEnd));
        } else if self.current_inp_byte() != b'%' {
            return Err(an_err!(
                DtErrKind::MismatchedLiteral,
                "%% got: {}",
                char::from(self.current_inp_byte())
            ));
        } else {
            self.advance_inp();
            self.advance_fmt();
        }
        Ok(())
    }

    #[inline(always)]
    fn parse_optional_dot_fractional(&mut self, width: Option<u8>) -> Result<(), DtErr> {
        // dot is optional in the input for %.f
        // (also supports explicit literal dot before %.f, e.g. %S.%.f)
        if !self.inp.is_empty() && self.current_inp_byte() == b'.' {
            self.advance_inp();
        }
        self.parse_fractional_seconds(width)?;
        Ok(())
    }

    #[inline(always)]
    fn parse_full_year(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (y, _sign, remaining) = match ext.parse_number(4, FormatFlag::PadZero, self.inp, false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedYear)),
        };
        self.inp = remaining;
        self.tm.yr = Some(y);
        Ok(())
    }

    #[inline(always)]
    fn parse_unbounded_year(&mut self) -> Result<(), DtErr> {
        // %* is special (arbitrary width year), use FormatExtensions for number parse
        let ext = FormatExtensions::default();
        let (y, _sign, remaining) = match ext.parse_number(0, FormatFlag::PadZero, self.inp, true) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedYear)),
        };
        self.inp = remaining;
        self.tm.yr = Some(y);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_two_digit_year(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (y, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedYear)),
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.yr = Some(year);
        Ok(())
    }

    #[inline(always)]
    fn parse_century(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (c, _sign, remaining) = match ext.parse_number(2, FormatFlag::PadSpace, self.inp, false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedCentury)),
        };
        self.inp = remaining;
        self.tm.yr = Some(c * 100);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_iso_week_year(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (y, _sign, remaining) = match ext.parse_number(4, FormatFlag::PadZero, self.inp, false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedYear)),
        };
        self.inp = remaining;
        self.tm.iso_wk_yr = Some(y);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_two_digit_iso_week_year(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (y, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedYear));
            }
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.iso_wk_yr = Some(year);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_month_number(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (m, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedMonth)),
        };
        self.inp = remaining;
        if !(1..=12).contains(&m) {
            return Err(an_err!(DtErrKind::MonthOutOfRange));
        }
        self.tm.mo = Some(m);
        Ok(())
    }

    #[inline(always)]
    fn parse_day_of_month(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (d, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedDay)),
        };
        self.inp = remaining;
        if !(1..=31).contains(&d) {
            return Err(an_err!(DtErrKind::DayOutOfRange));
        }
        self.tm.day = Some(d);
        Ok(())
    }

    #[inline(always)]
    fn parse_day_of_year(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (n, _sign, remaining) = match ext.parse_number(3, FormatFlag::PadZero, self.inp, false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedDayOfYear)),
        };
        self.inp = remaining;
        let day = n as u16;
        if !(1..=366).contains(&day) {
            return Err(an_err!(DtErrKind::DayOfYearOutOfRange));
        }
        self.tm.day_of_yr = Some(day);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_hour24(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (h, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedHour)),
        };
        self.inp = remaining;
        if h > 23 {
            return Err(an_err!(DtErrKind::HourOutOfRange));
        }
        self.tm.hr = h;
        Ok(())
    }

    #[inline(always)]
    fn parse_hour12(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (h, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedHour)),
        };
        self.inp = remaining;
        if !(1..=12).contains(&h) {
            return Err(an_err!(DtErrKind::HourOutOfRange));
        }
        self.tm.hr = h;
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_minute(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (m, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedMinute)),
        };
        self.inp = remaining;
        if m > 59 {
            return Err(an_err!(DtErrKind::MinuteOutOfRange));
        }
        self.tm.min = m;
        Ok(())
    }

    #[inline(always)]
    fn parse_second(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (s, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedSecond)),
        };
        self.inp = remaining;
        if s > 60 {
            return Err(an_err!(DtErrKind::SecondOutOfRange));
        }
        self.tm.sec = s;
        Ok(())
    }

    #[inline(always)]
    fn parse_fractional_seconds(&mut self, width: Option<u8>) -> Result<(), DtErr> {
        // Make %f, %N, %3N, %6N, etc. also accept an optional leading '.'
        // (symmetric with the %.f case handled in parse_optional_dot_fractional)
        if !self.inp.is_empty() && self.current_inp_byte() == b'.' {
            self.advance_inp();
        }
        let max_digits = width.map(|w| w as usize).unwrap_or(usize::MAX);

        let mut frac: u64 = 0;
        let mut digits_read = 0usize;
        while !self.inp.is_empty()
            && self.current_inp_byte().is_ascii_digit()
            && digits_read < max_digits
        {
            if digits_read < ATTOS_DIGITS {
                let d = (self.current_inp_byte() - b'0') as u64;
                frac = frac * 10 + d;
            }
            self.advance_inp();
            digits_read += 1;
        }
        if digits_read == 0 {
            return Err(an_err!(
                DtErrKind::ExpectedFractional,
                "%f or %N frac seconds"
            ));
        }
        let attos = if digits_read >= ATTOS_DIGITS {
            frac
        } else {
            let multiplier = 10u64.pow((ATTOS_DIGITS - digits_read) as u32);
            frac * multiplier
        };
        self.tm.attos = attos;
        Ok(())
    }

    #[inline(always)]
    fn parse_timestamp_sec(&mut self, ext: FormatExtensions, epoch: Epoch) -> Result<(), DtErr> {
        let (sec_val, sign, remaining) =
            match ext.parse_number(19, FormatFlag::PadSpace, self.inp, false) {
                Ok(v) => v,
                Err(_) => return Err(an_err!(DtErrKind::ExpectedTimestamp)),
            };
        self.inp = remaining;

        // Start with integer seconds converted to attoseconds
        let mut total_attos: i128 = (sec_val as i128) * ATTOS_PER_SEC_I128;

        // Greedily consume optional fractional seconds (e.g. 1712345678.123456789012345678)
        // This allows a single %s (or %J) directive to parse a full high-precision timestamp
        // without requiring a separate %.f / %N directive afterward.
        if !self.inp.is_empty()
            && self.current_inp_byte() == b'.'
            && self.inp.len() >= 2
            && self.inp[1].is_ascii_digit()
        {
            // Only consume the dot if it's followed by at least one digit

            self.advance_inp(); // eat '.'

            let mut frac: u64 = 0;
            let mut digits_read: usize = 0;

            while digits_read < ATTOS_DIGITS
                && !self.inp.is_empty()
                && self.current_inp_byte().is_ascii_digit()
            {
                let d = (self.current_inp_byte() - b'0') as u64;
                frac = frac * 10 + d;
                self.advance_inp();
                digits_read += 1;
            }

            if digits_read > 0 {
                let frac_attos = if digits_read >= ATTOS_DIGITS {
                    frac
                } else {
                    let multiplier = 10u64.pow((ATTOS_DIGITS - digits_read) as u32);
                    frac * multiplier
                };

                if sign == Sign::Negative {
                    total_attos -= frac_attos as i128;
                } else {
                    total_attos += frac_attos as i128;
                }
            }
            // dot with no following digit → leave it in input (do not consume)
        }

        let ts = Timestamp {
            attos: total_attos,
            epoch,
        };
        self.tm.timestamp = Some(ts);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_month_name_abbrev(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 3 {
            return Err(an_err!(DtErrKind::InvalidMonthName));
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
                return Err(an_err!(DtErrKind::InvalidMonthName));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.mo = Some(index + 1);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_month_name_full(&mut self) -> Result<(), DtErr> {
        let (index, remaining) = match Self::match_from_choice_list(self.inp, &EN_MONTHS_FULL) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::InvalidMonthName)),
        };
        self.inp = remaining;
        self.tm.mo = Some(index + 1);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_weekday_abbrev(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 3 {
            return Err(an_err!(DtErrKind::InvalidWeekdayName));
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
                return Err(an_err!(DtErrKind::InvalidWeekdayName));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.wkday = Some(
            Weekday::from_sunday_0_based(index)
                .ok_or_else(|| an_err!(DtErrKind::InvalidWeekdayName))?,
        );
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_weekday_full(&mut self) -> Result<(), DtErr> {
        let (index, remaining) = match Self::match_from_choice_list(self.inp, &EN_WEEKDAYS_FULL) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::InvalidWeekdayName)),
        };
        self.inp = remaining;
        self.tm.wkday = Some(
            Weekday::from_sunday_0_based(index)
                .ok_or_else(|| an_err!(DtErrKind::InvalidWeekdayName))?,
        );
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_weekday_number_monday_based(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (w, remaining) = match ext.parse_u8(1, FormatFlag::PadSpace, self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedMonWeekday));
            }
        };
        self.inp = remaining;
        let wd = Weekday::from_monday_1_based(w)
            .ok_or_else(|| an_err!(DtErrKind::MonWeekdayOutOfRange))?;
        self.tm.wkday = Some(wd);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_weekday_number_sunday_based(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (w, remaining) = match ext.parse_u8(1, FormatFlag::PadSpace, self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedSunWeekday));
            }
        };
        self.inp = remaining;
        let wd = Weekday::from_sunday_0_based(w)
            .ok_or_else(|| an_err!(DtErrKind::SunWeekdayOutOfRange))?;
        self.tm.wkday = Some(wd);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_ampm(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 2 {
            return Err(an_err!(DtErrKind::InvalidMeridiem));
        }
        let slice = &self.inp[..2];
        self.tm.meridiem = Some(if slice.eq_ignore_ascii_case(b"am") {
            Meridiem::AM
        } else if slice.eq_ignore_ascii_case(b"pm") {
            Meridiem::PM
        } else {
            return Err(an_err!(DtErrKind::InvalidMeridiem));
        });
        if self.tm.hr == 0 {
            self.tm.hr = 12;
        }
        self.inp = &self.inp[2..];
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_week_number_sunday_based(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (w, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedSunWeek));
            }
        };
        self.inp = remaining;
        self.tm.wk_sun = Some(w);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_week_number_monday_based(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (w, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedMonWeek));
            }
        };
        self.inp = remaining;
        self.tm.wk_mon = Some(w);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_week_iso(&mut self, ext: FormatExtensions) -> Result<(), DtErr> {
        let (w, remaining) = match ext.parse_u8(2, FormatFlag::PadZero, self.inp) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedWeekNumber)),
        };
        self.inp = remaining;
        if !(1..=53).contains(&w) {
            return Err(an_err!(DtErrKind::IsoWeekOutOfRange));
        }
        self.tm.iso_wk = Some(w);
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_timezone_offset(&mut self, colons: u8) -> Result<(), DtErr> {
        let sign = match self.inp.first() {
            Some(b'+') => 1i32,
            Some(b'-') => -1i32,
            _ => {
                return Err(an_err!(DtErrKind::MustStartWith));
            }
        };
        self.advance_inp();

        let mut total_seconds = self.parse_offset_hours()? * 3600;

        match colons {
            0 => {
                let minutes = if self.inp.len() >= 2
                    && self.inp[0].is_ascii_digit()
                    && self.inp[1].is_ascii_digit()
                {
                    self.parse_offset_mm_ss()?
                } else {
                    0
                };
                total_seconds += minutes * 60;

                if self.inp.len() >= 2
                    && let Ok(seconds) = self.parse_offset_mm_ss()
                {
                    total_seconds += seconds;
                }
            }
            1..=3 => {
                let minutes_required = colons != 3;
                if self.inp.first() == Some(&b':') {
                    self.advance_inp();
                    let minutes = match self.parse_offset_mm_ss() {
                        Ok(m) => m,
                        Err(_) => {
                            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z minutes"));
                        }
                    };
                    total_seconds += minutes * 60;
                    if self.inp.first() == Some(&b':') {
                        self.advance_inp();
                        let seconds = match self.parse_offset_mm_ss() {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(an_err!(
                                    DtErrKind::InvalidTimezoneOffset,
                                    "%z seconds",
                                ));
                            }
                        };
                        total_seconds += seconds;
                    } else if colons == 2 {
                        return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z num colons"));
                    }
                } else if minutes_required {
                    return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z num colons"));
                }
            }
            _ => {
                return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z num colons"));
            }
        }

        // Store the fixed offset (in seconds) in our core TimeZone type.
        self.tm.offset = Some(Offset::Fixed(sign * total_seconds));
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_offset_hours(&mut self) -> Result<i32, DtErr> {
        let mut n = 0i32;
        let mut digits = 0;
        while digits < 2 && !self.inp.is_empty() && self.current_inp_byte().is_ascii_digit() {
            n = n * 10 + (self.current_inp_byte() - b'0') as i32;
            self.advance_inp();
            digits += 1;
        }
        if digits == 0 {
            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z hour"));
        }
        if n > 23 {
            return Err(an_err!(
                DtErrKind::InvalidTimezoneOffset,
                "%z hour (0..=23): {}",
                n
            ));
        }
        Ok(n)
    }

    fn parse_offset_mm_ss(&mut self) -> Result<i32, DtErr> {
        if self.inp.len() < 2 {
            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z mins or secs"));
        }

        let a = self.inp[0];
        let b = self.inp[1];

        // Must be two ASCII digits
        if !a.is_ascii_digit() || !b.is_ascii_digit() {
            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z mins or secs"));
        }

        let n = ((a - b'0') as i32) * 10 + (b - b'0') as i32;

        if !(0..=59).contains(&n) {
            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "%z mins or secs"));
        }

        self.inp = &self.inp[2..];
        Ok(n)
    }

    #[inline(always)]
    fn parse_iana_or_offset(&mut self, colons: u8) -> Result<(), DtErr> {
        if !self.inp.is_empty() && matches!(self.current_inp_byte(), b'+' | b'-') {
            return self.parse_timezone_offset(colons);
        }
        let (iana_str, remaining) = match Self::parse_iana(self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::InvalidTimezoneOffset));
            }
        };
        let name_to_use = if iana_str.len() > 50 {
            &iana_str[0..50]
        } else {
            iana_str
        };
        self.tm.set_iana_name(Some(name_to_use));
        self.tm.offset = Some(Offset::None);
        self.inp = remaining;
        self.advance_fmt();
        Ok(())
    }

    #[inline(always)]
    fn parse_scale(&mut self) -> Result<(), DtErr> {
        if self.inp.is_empty() || !self.inp[0].is_ascii_alphabetic() {
            return Err(an_err!(DtErrKind::InvalidScale));
        }
        let start = self.inp;
        let mut pos = 0usize;
        while pos < start.len() && pos < 8 && start[pos].is_ascii_alphanumeric() {
            pos += 1;
        }
        let abbrev =
            core::str::from_utf8(&start[..pos]).map_err(|_| an_err!(DtErrKind::InvalidScale))?;
        self.inp = &start[pos..];
        self.advance_fmt();
        if let Some(ct) = Scale::from_abbrev(abbrev) {
            self.tm.scale = ct;
            Ok(())
        } else {
            Err(an_err!(DtErrKind::InvalidScale))
        }
    }

    #[inline(always)]
    fn parse_iso_date(&mut self) -> Result<(), DtErr> {
        self.parse_full_year(FormatExtensions::default())?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_month_number(FormatExtensions::default())?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_day_of_month(FormatExtensions::default())?;
        self.advance_fmt(); // eat %F
        Ok(())
    }

    #[inline(always)]
    fn parse_us_date_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_month_number(FormatExtensions::default())?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_day_of_month(FormatExtensions::default())?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_two_digit_year(FormatExtensions::default())?;
        self.advance_fmt(); // eat %D
        Ok(())
    }

    #[inline(always)]
    fn parse_time_with_seconds_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_hour24(FormatExtensions::default())?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(FormatExtensions::default())?;
        self.parse_literal_character_byte(b':')?;
        self.parse_second(FormatExtensions::default())?;
        self.advance_fmt(); // eat %T
        Ok(())
    }

    #[inline(always)]
    fn parse_time_without_seconds_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_hour24(FormatExtensions::default())?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(FormatExtensions::default())?;
        self.advance_fmt(); // eat %R
        Ok(())
    }

    #[inline(always)]
    fn parse_literal_character_byte(&mut self, expected: u8) -> Result<(), DtErr> {
        if self.inp.is_empty() || self.current_inp_byte() != expected {
            return Err(an_err!(
                DtErrKind::MismatchedLiteral,
                "expected literal: {}",
                expected,
            ));
        }
        self.advance_inp();
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn parse_fmt_extensions(&mut self) -> FormatExtensions {
        let first = self.fmt[0];

        // Early exit for the common case (no flag, no width, no colons)
        if !matches!(first, b'-' | b'_' | b'0' | b'^' | b'#' | b'0'..=b'9' | b':') {
            return FormatExtensions::default();
        }

        let mut flag = FormatFlag::None;
        let mut width = None;
        let mut colons = 0u8;

        // Flag
        let f = FormatFlag::from_byte(first);
        if f != FormatFlag::None {
            flag = f;
            self.fmt = &self.fmt[1..];
            if self.fmt.is_empty() {
                return FormatExtensions {
                    flag,
                    width: None,
                    colons: 0,
                };
            }
        }

        // Width
        if !self.fmt.is_empty() && self.fmt[0].is_ascii_digit() {
            let mut w: u16 = u16::from(self.fmt[0] - b'0');
            self.fmt = &self.fmt[1..];
            let mut digits = 1u8;

            while digits < 3 && !self.fmt.is_empty() && self.fmt[0].is_ascii_digit() {
                w = w * 10 + u16::from(self.fmt[0] - b'0');
                self.fmt = &self.fmt[1..];
                digits += 1;
            }

            // Consume any extra digits beyond 3
            while !self.fmt.is_empty() && self.fmt[0].is_ascii_digit() {
                self.fmt = &self.fmt[1..];
            }

            if w <= u8::MAX as u16 {
                width = Some(w as u8);
            }
        }

        // Colons (hard cap at 3)
        while colons < 3 && !self.fmt.is_empty() && self.fmt[0] == b':' {
            colons += 1;
            self.fmt = &self.fmt[1..];
        }

        FormatExtensions {
            flag,
            width,
            colons,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    fn parse_iana(inp: &[u8]) -> Result<(&str, &[u8]), ()> {
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
                if pos >= inp.len() || !matches!(inp[pos], b'_' | b'.' | b'A'..=b'Z' | b'a'..=b'z')
                {
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
}

impl FormatExtensions {
    #[inline(always)]
    fn parse_u8<'i>(
        &self,
        default_pad_width: usize,
        default_flag: FormatFlag,
        mut inp: &'i [u8],
    ) -> Result<(u8, &'i [u8]), ()> {
        // Strip leading whitespace. This only copies the fat pointer (ptr+len),
        // never the underlying input bytes.
        while inp.get(0).map_or(false, |b| b.is_ascii_whitespace()) {
            inp = &inp[1..];
        }
        if inp.is_empty() {
            return Err(());
        }

        let max_d = if self.flag == FormatFlag::None && self.width.is_none() {
            default_pad_width
        } else {
            match self.flag.resolve(default_flag) {
                FormatFlag::PadSpace | FormatFlag::NoPad => default_pad_width,
                _ => self.width.map_or(default_pad_width, usize::from),
            }
        }
        .min(3);

        let len = inp.len();
        let mut consumed = 0usize;
        let mut acc: u8 = 0;

        while consumed < max_d && consumed < len {
            let b = inp[consumed];
            if !b.is_ascii_digit() {
                break;
            }
            acc = acc
                .checked_mul(10)
                .and_then(|a| a.checked_add(b - b'0'))
                .ok_or(())?;

            consumed += 1;
        }

        Ok((acc, &inp[consumed..]))
    }

    #[inline(always)]
    fn parse_number<'i>(
        &self,
        default_pad_width: usize,
        default_flag: FormatFlag,
        mut inp: &'i [u8],
        arbitrary: bool,
    ) -> Result<(i64, Sign, &'i [u8]), ()> {
        // Strip leading whitespace
        while inp.get(0).map_or(false, |b| b.is_ascii_whitespace()) {
            inp = &inp[1..];
        }
        if inp.is_empty() {
            return Err(());
        }

        // Fast path: no sign, no format extensions, not arbitrary
        if self.flag == FormatFlag::None
            && self.width.is_none()
            && !arbitrary
            && !matches!(inp[0], b'+' | b'-')
        {
            if !inp[0].is_ascii_digit() {
                return Err(());
            }

            let mut consumed = 0usize;
            let mut acc: i64 = 0;

            // Skip leading zeros
            while consumed < default_pad_width && consumed < inp.len() && inp[consumed] == b'0' {
                consumed += 1;
            }

            // Accumulate significant digits
            while consumed < default_pad_width && consumed < inp.len() {
                let b = inp[consumed];
                if !b.is_ascii_digit() {
                    break;
                }
                let digit = (b - b'0') as i64;

                acc = acc
                    .checked_mul(10)
                    .and_then(|a| a.checked_add(digit))
                    .ok_or(())?;

                consumed += 1;
            }

            return Ok((acc, Sign::Positive, &inp[consumed..]));
        }

        // Normal path with optional sign
        let (sign, inp) = match inp.first() {
            Some(b'-') => (Sign::Negative, &inp[1..]),
            Some(b'+') => (Sign::Positive, &inp[1..]),
            _ => (Sign::Positive, inp),
        };

        if inp.is_empty() || !inp[0].is_ascii_digit() {
            return Err(());
        }

        let max_digits = if arbitrary {
            19
        } else {
            let zero_pad_width = match self.flag.resolve(default_flag) {
                FormatFlag::PadSpace | FormatFlag::NoPad => 0,
                _ => self.width.map_or(0, usize::from),
            };
            zero_pad_width.max(default_pad_width)
        };

        let mut consumed = 0usize;
        let mut acc: i64 = 0; // always stores non-negative magnitude

        // Skip leading zeros
        while consumed < max_digits && consumed < inp.len() && inp[consumed] == b'0' {
            consumed += 1;
        }

        // Accumulate significant digits (always positive)
        while consumed < max_digits && consumed < inp.len() {
            let b = inp[consumed];
            if !b.is_ascii_digit() {
                break;
            }
            let digit = (b - b'0') as i64;

            acc = acc
                .checked_mul(10)
                .and_then(|a| a.checked_add(digit))
                .ok_or(())?;

            consumed += 1;
        }

        // Apply sign at the very end
        let n = if sign == Sign::Negative {
            acc.checked_neg().ok_or(())?
        } else {
            acc
        };

        Ok((n, sign, &inp[consumed..]))
    }
}
