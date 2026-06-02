use crate::error::{DtErr, DtErrKind};
use crate::{Meridiem, Offset, Scale, TimeParts, Weekday, an_err};
use core::result::Result;
use core::str;

pub(crate) struct Parser<'f, 'i, 't> {
    pub(crate) fmt: &'f [u8], // remaining format string
    pub(crate) inp: &'i [u8], // remaining input string
    tm: &'t mut TimeParts,
    inp_can_end_before_fmt: bool,
}

impl<'f, 'i, 't> Parser<'f, 'i, 't> {
    pub(crate) fn new(
        fmt: &'f [u8],
        inp: &'i [u8],
        tm: &'t mut TimeParts,
        inp_can_end_before_fmt: bool,
    ) -> Self {
        Self {
            fmt,
            inp,
            tm,
            inp_can_end_before_fmt,
        }
    }

    #[inline]
    fn current_format_byte(&self) -> u8 {
        self.fmt[0]
    }

    #[inline]
    fn current_input_byte(&self) -> u8 {
        self.inp[0]
    }

    #[inline]
    fn advance_format(&mut self) -> bool {
        self.fmt = &self.fmt[1..];
        !self.fmt.is_empty()
    }

    #[inline]
    fn advance_input(&mut self) -> bool {
        self.inp = &self.inp[1..];
        !self.inp.is_empty()
    }

    pub(crate) fn parse(&mut self) -> Result<(), DtErr> {
        while !self.fmt.is_empty() {
            if self.current_format_byte() != b'%' {
                self.parse_literal_character()?;
                continue;
            }
            if !self.advance_format() {
                return Err(an_err!(DtErrKind::UnexpectedEnd, "after %"));
            }

            let (flag, width, colons, new_fmt) = Self::parse_format_extensions(self.fmt, 0);
            self.fmt = new_fmt;

            let directive = self.fmt.first().copied().unwrap_or(0);

            if self.inp.is_empty() {
                if self.inp_can_end_before_fmt {
                    if !matches!(directive, b'.' | b'f' | b'N') {
                        return Err(an_err!(DtErrKind::UnexpectedEnd, "input exhausted"));
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
                b'z' => self.parse_timezone_offset(flag, width, colons)?,
                b'.' => {
                    if !self.advance_format() {
                        return Err(an_err!(DtErrKind::UnexpectedEnd, "after ."));
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

                    let next: u8 = self.fmt.first().copied().unwrap_or(0);
                    if !matches!(next, b'f' | b'N') {
                        return Err(an_err!(DtErrKind::BadFractional, "{}", char::from(next)));
                    }
                    self.advance_format();

                    self.parse_optional_dot_fractional(flag, width, colons)?;
                }
                // shortcuts
                b'F' => self.parse_iso_date()?,
                b'D' => self.parse_us_date_shortcut()?,
                b'T' => self.parse_time_with_seconds_shortcut()?,
                b'R' => self.parse_time_without_seconds_shortcut()?,
                // Library directives
                b'*' => self.parse_unbounded_year()?,
                b'L' => self.parse_scale()?,
                b'c' | b'r' | b'X' | b'x' | b'Z' => {
                    return Err(an_err!(
                        DtErrKind::UnsupportedDirective,
                        "{}",
                        char::from(directive)
                    ));
                }
                _ => {
                    return Err(an_err!(DtErrKind::UnknownDirective));
                }
            }
        }
        Ok(())
    }

    fn parse_literal_character(&mut self) -> Result<(), DtErr> {
        let c = self.current_format_byte();
        if c.is_ascii_whitespace() {
            while !self.inp.is_empty() && self.current_input_byte().is_ascii_whitespace() {
                self.advance_input();
            }
        } else if self.inp.is_empty() || self.current_input_byte() != c {
            return Err(an_err!(DtErrKind::MismatchedLiteral, "literal"));
        } else {
            self.advance_input();
        }
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn skip_whitespace(&mut self) -> Result<(), DtErr> {
        while !self.inp.is_empty() && self.current_input_byte().is_ascii_whitespace() {
            self.advance_input();
        }
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_percent_sign(&mut self) -> Result<(), DtErr> {
        if self.inp.is_empty() || self.current_input_byte() != b'%' {
            return Err(an_err!(
                DtErrKind::MismatchedLiteral,
                "% got: {}",
                char::from(self.current_input_byte())
            ));
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
    ) -> Result<(), DtErr> {
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
    ) -> Result<(), DtErr> {
        let (y, remaining) = match Self::parse_padded_number(self.inp, flag, width, 4, b'0', false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "year")),
        };
        self.tm.yr = Some(y);
        self.inp = remaining;
        if advance {
            self.advance_format();
        }
        Ok(())
    }

    #[inline]
    fn parse_unbounded_year(&mut self) -> Result<(), DtErr> {
        let (y, remaining) = match Self::parse_padded_number(self.inp, None, None, 0, b'0', true) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "year")),
        };
        self.tm.yr = Some(y);
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
    ) -> Result<(), DtErr> {
        let (y, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "year")),
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.yr = Some(year);
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
    ) -> Result<(), DtErr> {
        let (c, remaining) = match Self::parse_padded_number(self.inp, flag, width, 2, b'_', false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "century")),
        };
        self.tm.yr = Some(c * 100);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_iso_week_year(
        &mut self,
        flag: Option<u8>,
        width: Option<u8>,
        _colons: u8,
    ) -> Result<(), DtErr> {
        let (y, remaining) = match Self::parse_padded_number(self.inp, flag, width, 4, b'0', false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "iso week year")),
        };
        self.tm.iso_wk_yr = Some(y);
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
    ) -> Result<(), DtErr> {
        let (y, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(DtErrKind::ExpectedValue, "iso week year"));
            }
        };
        self.inp = remaining;
        let year = if y <= 68 {
            2000i64 + (y as i64)
        } else {
            1900i64 + (y as i64)
        };
        self.tm.iso_wk_yr = Some(year);
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
    ) -> Result<(), DtErr> {
        let (m, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "digit month")),
        };
        if !(1..=12).contains(&m) {
            return Err(an_err!(DtErrKind::OutOfRange, "month (1..=12): {}", m));
        }
        self.tm.mo = Some(m);
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
    ) -> Result<(), DtErr> {
        let (d, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "day")),
        };
        if !(1..=31).contains(&d) {
            return Err(an_err!(DtErrKind::OutOfRange, "day (1..=31): {}", d));
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
    ) -> Result<(), DtErr> {
        let (n, remaining) = match Self::parse_padded_number(self.inp, flag, width, 3, b'0', false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "day of year")),
        };
        let day = n as u16;
        if !(1..=366).contains(&day) {
            return Err(an_err!(
                DtErrKind::OutOfRange,
                "day of year (1..=366): {}",
                day
            ));
        }
        self.tm.day_of_yr = Some(day);
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
    ) -> Result<(), DtErr> {
        let (h, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "hour")),
        };
        if h > 23 {
            return Err(an_err!(DtErrKind::OutOfRange, "hour (0..=23): {}", h));
        }
        self.tm.hr = Some(h);
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
    ) -> Result<(), DtErr> {
        let (h, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "hour")),
        };
        if !(1..=12).contains(&h) {
            return Err(an_err!(DtErrKind::OutOfRange, "hour (1..=12): {}", h));
        }
        self.tm.hr = Some(h);
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
    ) -> Result<(), DtErr> {
        let (m, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "minute")),
        };
        if m > 59 {
            return Err(an_err!(DtErrKind::OutOfRange, "minute (0..=59): {}", m));
        }
        self.tm.min = Some(m);
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
    ) -> Result<(), DtErr> {
        let (s, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "seconds")),
        };
        if s > 60 {
            return Err(an_err!(DtErrKind::OutOfRange, "seconds (0..=60): {}", s));
        }
        self.tm.sec = Some(s);
        self.tm.is_leap_sec = s == 60;
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
    ) -> Result<(), DtErr> {
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
            return Err(an_err!(DtErrKind::ExpectedValue, "frac seconds"));
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
    ) -> Result<(), DtErr> {
        let (n, remaining) = match Self::parse_padded_number(self.inp, flag, width, 19, b' ', false)
        {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "timestamp")),
        };
        self.tm.unix_timestamp_seconds = Some(n);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_month_name_abbrev(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 3 {
            return Err(an_err!(DtErrKind::InvalidName, "abbrev. month name"));
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
                return Err(an_err!(DtErrKind::InvalidName, "abbrev. month name"));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.mo = Some(index + 1);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_month_name_full(&mut self) -> Result<(), DtErr> {
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
        let (index, remaining) = match Self::match_from_choice_list(self.inp, CHOICES) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::InvalidName, "month name")),
        };
        self.inp = remaining;
        self.tm.mo = Some(index + 1);
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_abbrev(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 3 {
            return Err(an_err!(DtErrKind::InvalidName, "abbrev. weekday"));
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
                return Err(an_err!(DtErrKind::InvalidName, "abbrev. weekday"));
            }
        };
        self.inp = &self.inp[3..];
        self.tm.wkday = Some(
            Weekday::from_sunday_zero_offset(index)
                .ok_or_else(|| an_err!(DtErrKind::InvalidName, "abbrev. weekday"))?,
        );
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_weekday_full(&mut self) -> Result<(), DtErr> {
        static CHOICES: &[&[u8]] = &[
            b"Sunday",
            b"Monday",
            b"Tuesday",
            b"Wednesday",
            b"Thursday",
            b"Friday",
            b"Saturday",
        ];
        let (index, remaining) = match Self::match_from_choice_list(self.inp, CHOICES) {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::InvalidName, "weekday")),
        };
        self.inp = remaining;
        self.tm.wkday = Some(
            Weekday::from_sunday_zero_offset(index)
                .ok_or_else(|| an_err!(DtErrKind::InvalidName, "weekday"))?,
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
    ) -> Result<(), DtErr> {
        let (w, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 1, b'_') {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(
                    DtErrKind::ExpectedValue,
                    "monday based weekday number"
                ));
            }
        };
        let wd = Weekday::from_monday_one_offset(w)
            .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "monday based weekday number"))?;
        self.tm.wkday = Some(wd);
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
    ) -> Result<(), DtErr> {
        let (w, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 1, b'_') {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(
                    DtErrKind::ExpectedValue,
                    "sunday based weekday number"
                ));
            }
        };
        let wd = Weekday::from_sunday_zero_offset(w)
            .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "sunday based weekday number"))?;
        self.tm.wkday = Some(wd);
        self.inp = remaining;
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_ampm(&mut self) -> Result<(), DtErr> {
        if self.inp.len() < 2 {
            return Err(an_err!(DtErrKind::InvalidName, "am/pm"));
        }
        let slice = &self.inp[..2];
        self.tm.meridiem = Some(if slice.eq_ignore_ascii_case(b"am") {
            Meridiem::AM
        } else if slice.eq_ignore_ascii_case(b"pm") {
            Meridiem::PM
        } else {
            return Err(an_err!(DtErrKind::InvalidName, "am/pm"));
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
    ) -> Result<(), DtErr> {
        let (w, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(
                    DtErrKind::ExpectedValue,
                    "week number sunday based"
                ));
            }
        };
        self.tm.wk_sun = Some(w);
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
    ) -> Result<(), DtErr> {
        let (w, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(
                    DtErrKind::ExpectedValue,
                    "week number monday based"
                ));
            }
        };
        self.tm.wk_mon = Some(w);
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
    ) -> Result<(), DtErr> {
        let (w, remaining) = match Self::parse_u8_padded(self.inp, flag, width, 2, b'0') {
            Ok(v) => v,
            Err(_) => return Err(an_err!(DtErrKind::ExpectedValue, "iso week")),
        };
        if !(1..=53).contains(&w) {
            return Err(an_err!(DtErrKind::OutOfRange, "iso week (1..=53): {}", w));
        }
        self.tm.iso_wk = Some(w);
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
    ) -> Result<(), DtErr> {
        let sign = match self.inp.first() {
            Some(b'+') => 1i32,
            Some(b'-') => -1i32,
            _ => {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "must start with + or -"
                ));
            }
        };
        self.advance_input();

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
                    self.advance_input();
                    let minutes = match self.parse_offset_mm_ss() {
                        Ok(m) => m,
                        Err(_) => {
                            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "minutes"));
                        }
                    };
                    total_seconds += minutes * 60;
                    if self.inp.first() == Some(&b':') {
                        self.advance_input();
                        let seconds = match self.parse_offset_mm_ss() {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "seconds",));
                            }
                        };
                        total_seconds += seconds;
                    } else if colons == 2 {
                        return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "num colons"));
                    }
                } else if minutes_required {
                    return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "num colons"));
                }
            }
            _ => {
                return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "num colons"));
            }
        }

        // Store the fixed offset (in seconds) in our core TimeZone type.
        self.tm.offset = Some(Offset::Fixed(sign * total_seconds));
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_offset_hours(&mut self) -> Result<i32, DtErr> {
        let mut n = 0i32;
        let mut digits = 0;
        while digits < 2 && !self.inp.is_empty() && self.current_input_byte().is_ascii_digit() {
            n = n * 10 + (self.current_input_byte() - b'0') as i32;
            self.advance_input();
            digits += 1;
        }
        if digits == 0 {
            return Err(an_err!(DtErrKind::InvalidTimezoneOffset, "hour"));
        }
        if n > 23 {
            return Err(an_err!(
                DtErrKind::InvalidTimezoneOffset,
                "hour (0..=23): {}",
                n
            ));
        }
        Ok(n)
    }

    #[inline]
    fn parse_offset_mm_ss(&mut self) -> Result<i32, DtErr> {
        if self.inp.len() < 2 {
            return Err(an_err!(
                DtErrKind::InvalidTimezoneOffset,
                "minutes or seconds"
            ));
        }
        let slice = &self.inp[..2];
        let n = match core::str::from_utf8(slice)
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|&n| (0..=59).contains(&n))
        {
            Some(n) => n,
            None => {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "minutes or seconds"
                ));
            }
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
    ) -> Result<(), DtErr> {
        if !self.inp.is_empty() && matches!(self.current_input_byte(), b'+' | b'-') {
            return self.parse_timezone_offset(_flag, _width, colons);
        }
        let (iana_str, remaining) = match Self::parse_iana(self.inp) {
            Ok(v) => v,
            Err(_) => {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "expected iana or offset"
                ));
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
        self.advance_format();
        Ok(())
    }

    #[inline]
    fn parse_scale(&mut self) -> Result<(), DtErr> {
        if self.inp.is_empty() || !self.inp[0].is_ascii_alphabetic() {
            return Err(an_err!(DtErrKind::InvalidItem, "invalid scale"));
        }
        let start = self.inp;
        let mut pos = 0usize;
        // Use `start[pos]`, not `self.inp[pos]`
        while pos < start.len() && pos < 8 && start[pos].is_ascii_alphanumeric() {
            pos += 1;
        }
        let abbrev = core::str::from_utf8(&start[..pos])
            .map_err(|_| an_err!(DtErrKind::InvalidItem, "invalid scale"))?;
        self.inp = &start[pos..];
        self.advance_format();
        if let Some(ct) = Scale::from_abbrev(abbrev) {
            self.tm.scale = ct;
            Ok(())
        } else {
            Err(an_err!(DtErrKind::InvalidItem, "invalid scale"))
        }
    }

    #[inline]
    fn parse_iso_date(&mut self) -> Result<(), DtErr> {
        self.parse_full_year(None, None, 0, false)?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_month_number(None, None, 0, false)?;
        self.parse_literal_character_byte(b'-')?;
        self.parse_day_of_month(None, None, 0, false)?;
        self.advance_format(); // eat %F
        Ok(())
    }

    #[inline]
    fn parse_us_date_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_month_number(None, None, 0, false)?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_day_of_month(None, None, 0, false)?;
        self.parse_literal_character_byte(b'/')?;
        self.parse_two_digit_year(None, None, 0, false)?;
        self.advance_format(); // eat %D
        Ok(())
    }

    #[inline]
    fn parse_time_with_seconds_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_hour24(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_second(None, None, 0, false)?;
        self.advance_format(); // eat %T
        Ok(())
    }

    #[inline]
    fn parse_time_without_seconds_shortcut(&mut self) -> Result<(), DtErr> {
        self.parse_hour24(None, None, 0, false)?;
        self.parse_literal_character_byte(b':')?;
        self.parse_minute(None, None, 0, false)?;
        self.advance_format(); // eat %R
        Ok(())
    }

    #[inline]
    fn parse_literal_character_byte(&mut self, expected: u8) -> Result<(), DtErr> {
        if self.inp.is_empty() || self.current_input_byte() != expected {
            return Err(an_err!(
                DtErrKind::MismatchedLiteral,
                "Expected literal char"
            ));
        }
        self.advance_input();
        Ok(())
    }

    #[inline]
    pub(crate) fn parse_format_extensions(
        fmt: &[u8],
        mut pos: usize,
    ) -> (Option<u8>, Option<u8>, u8, &[u8]) {
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
        if let Some(b'-') = inp.first() {
            (-1, &inp[1..])
        } else if let Some(b'+') = inp.first() {
            (1, &inp[1..])
        } else {
            (1, inp)
        }
    }

    fn parse_padded_number(
        input: &[u8],
        flag: Option<u8>,
        width: Option<u8>,
        default_pad_width: usize,
        default_flag: u8,
        arbitrary: bool,
    ) -> Result<(i64, &[u8]), ()> {
        // Skip leading ASCII whitespace
        let mut pos = 0;
        while pos < input.len() && input[pos].is_ascii_whitespace() {
            pos += 1;
        }

        let bytes = &input[pos..];
        if bytes.is_empty() {
            return Err(());
        }

        let effective_flag = match flag {
            Some(b'^') | Some(b'#') => default_flag,
            Some(f) => f,
            None => default_flag,
        };

        // Sign handling
        let (sign, after_sign) = Self::parse_optional_sign(bytes);
        if after_sign.is_empty() || !after_sign[0].is_ascii_digit() {
            return Err(());
        }

        let is_negative = sign < 0;

        let zero_pad_width = if arbitrary || matches!(effective_flag, b'_' | b' ' | b'-') {
            0
        } else {
            width.map_or(0, usize::from)
        };

        let max_digits = if arbitrary {
            usize::MAX
        } else {
            zero_pad_width.max(default_pad_width)
        };

        // Padding zero skip
        let mut consumed = 0usize;
        while consumed < zero_pad_width
            && consumed < after_sign.len()
            && after_sign[consumed] == b'0'
        {
            consumed += 1;
        }

        // Unified robust accumulation
        let max_allowed: u64 = if is_negative {
            (i64::MAX as u64).wrapping_add(1)
        } else {
            i64::MAX as u64
        };

        let mut acc: u64 = if consumed < after_sign.len() {
            (after_sign[consumed] - b'0') as u64
        } else {
            0
        };
        if consumed < after_sign.len() {
            consumed += 1;
        }

        while consumed < max_digits && consumed < after_sign.len() {
            let b = after_sign[consumed];
            if !b.is_ascii_digit() {
                break;
            }
            let d = (b - b'0') as u64;

            if acc > max_allowed / 10 || (acc == max_allowed / 10 && d > max_allowed % 10) {
                return Err(());
            }

            acc = acc * 10 + d;
            consumed += 1;
        }

        let n = if is_negative {
            if acc == max_allowed {
                i64::MIN
            } else {
                -(acc as i64)
            }
        } else {
            acc as i64
        };

        Ok((n, &after_sign[consumed..]))
    }

    #[inline]
    fn parse_u8_padded(
        inp: &[u8],
        flag: Option<u8>,
        width: Option<u8>,
        default_pad_width: usize,
        default_flag: u8,
    ) -> Result<(u8, &[u8]), ()> {
        let (n, remaining) =
            Self::parse_padded_number(inp, flag, width, default_pad_width, default_flag, false)?;
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
