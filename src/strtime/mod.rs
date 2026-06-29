pub mod parser;
pub mod printer;

use crate::error::{DtErr, DtErrKind};
use crate::{Dt, Lang, LiteStr, Parts, STRTIME_SIZE, an_err};
use core::result::Result;
use core::str;

pub(crate) use parser::*;

/// Optional `%` directive extensions: flag, width, and colon count.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FmtExtensions {
    pub(crate) flag: FmtFlag,
    pub(crate) width: Option<u8>,
    pub(crate) colons: u8,
}

/// Flags that may appear immediately after `%` and before the directive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FmtFlag {
    #[default]
    None,
    PadSpace,
    PadZero,
    NoPad,
    Uppercase,
    Swapcase,
}

impl FmtFlag {
    #[inline(always)]
    pub(crate) fn from_byte(byte: u8) -> Self {
        match byte {
            b'_' => Self::PadSpace,
            b'0' => Self::PadZero,
            b'-' => Self::NoPad,
            b'^' => Self::Uppercase,
            b'#' => Self::Swapcase,
            _ => Self::None,
        }
    }

    /// Resolve the padding flag for numeric parsing.
    ///
    /// `None`, `Uppercase`, and `Swapcase` defer to the directive default;
    /// the three pad flags override it.
    #[inline(always)]
    pub(crate) fn resolve(self, default: FmtFlag) -> FmtFlag {
        match self {
            Self::None | Self::Uppercase | Self::Swapcase => default,
            pad => pad,
        }
    }
}

/// A pre-validated, reusable date/time format string.
///
/// - Format is validated **once** at construction (`new` returns `Result`).
/// - Format bytes are copied into an owned fixed-size buffer.
/// - Only ASCII formats are accepted.
///
/// ## See also
///
/// - [`StrPTimeFmt::new`]
/// - [`StrPTimeFmt::to_dt`]
/// - [`StrPTimeFmt::to_str`]
#[derive(Debug, Clone, Copy)]
pub struct StrPTimeFmt {
    fmt: [u8; Self::MAX_FORMAT_LEN],
    len: usize,
}

impl StrPTimeFmt {
    pub const MAX_FORMAT_LEN: usize = 256;

    /// Creates a new validated format.
    ///
    /// - Validates syntax and supported directives.
    /// - Requires the format to be valid ASCII and ≤ 256 bytes.
    /// - Returns a [`DtErr`] on any failure.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::InvalidLen`] if the format string is longer than 256 bytes.
    /// - [`DtErrKind::InvalidInput`] if the format string is not valid ASCII.
    /// - [`DtErrKind::TruncatedDirective`] if a `%` appears at the end of the format
    ///   with no directive character following it.
    /// - [`DtErrKind::UnexpectedEnd`] if a `%` is followed only by flags, width digits,
    ///   or colons, with no directive character after them.
    /// - [`DtErrKind::ExpectedFractional`] if a `%.` sequence is not followed by a
    ///   directive character.
    /// - [`DtErrKind::InvalidFractional`] if a `%.` sequence is followed by a character
    ///   other than `f` or `N`.
    /// - [`DtErrKind::UnsupportedItem`] if the format contains `%c`, `%r`, `%x`, `%X`,
    ///   or `%Z`.
    /// - [`DtErrKind::UnknownItem`] if the format contains any other unrecognized `%`
    ///   directive.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "parse")]
    /// # {
    /// use deep_time::{Dt, Lang, StrPTimeFmt};
    ///
    /// let fmt = Dt::parse_fmt("%F %T").unwrap();
    ///
    /// // parse a datetime
    /// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
    ///
    /// // change a datetimes format
    /// let s = fmt.to_str("2000-01-01 12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En).unwrap();
    ///
    /// assert_eq!(s, "01 01 2000 12:00:00");
    /// # }
    /// ```
    pub fn new(fmt: &str) -> Result<Self, DtErr> {
        if fmt.len() > Self::MAX_FORMAT_LEN {
            return Err(an_err!(DtErrKind::InvalidLen));
        }
        let fmt = fmt.as_bytes();
        if !fmt.is_ascii() {
            return Err(an_err!(DtErrKind::InvalidInput, "must be ascii"));
        }

        Self::validate_format(fmt)?;

        let mut buffer = [0u8; Self::MAX_FORMAT_LEN];
        buffer[..fmt.len()].copy_from_slice(fmt);

        Ok(Self {
            fmt: buffer,
            len: fmt.len(),
        })
    }

    /// Parses a date/time string using this pre-validated format.
    ///
    /// The four boolean flags control lenient parsing behavior — see
    /// [`Dt::from_str`](../struct.Dt.html#method.from_str) for full documentation.
    ///
    /// ## Parameters
    ///
    /// - `s`: The input string to parse.
    /// - `inp_can_end_before_fmt`: Allow input to end before format is fully consumed.
    /// - `fmt_can_end_before_inp`: Allow format to end before input is fully consumed.
    /// - `allow_partial_date`: Default missing month/day to `1` instead of erroring.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::InvalidBytes`] if `as_str()` fails to convert the stored format
    ///   back to `&str`.
    /// - Any error returned by `Parts::from_str` followed by `Parts::to_dt` (see the
    ///   error documentation on [`Dt::from_str`] for the complete list).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, StrPTimeFmt};
    ///
    /// let fmt = Dt::parse_fmt("%F %T").unwrap();
    /// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
    /// ```
    pub fn to_dt(
        &self,
        s: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        Parts::from_str(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )
        .and_then(|p| p.to_dt())
    }

    /// Formats a [`Dt`] into a string using this pre-validated format and a given
    /// output format.
    ///
    /// Effectively parses a [`str`] with the contained format, then outputs a
    /// [`String`](`alloc::string::String`) with a new given format.
    ///
    /// Requires the `alloc` feature.
    ///
    /// ## Parameters
    ///
    /// - `s`: datetime input [`str`].
    /// - `output_fmt`: The new format to output the datetime as.
    /// - The remaining three flags are passed through to the internal `to_dt` call.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use deep_time::{Dt, Lang, StrPTimeFmt};
    ///
    /// let fmt = Dt::parse_fmt("%Y-%m-%dT%H:%M:%S").unwrap();
    /// let s = fmt.to_str("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En).unwrap();
    ///
    /// assert_eq!(s, "01 01 2000 12:00:00");
    /// # }
    /// ```
    #[cfg(feature = "alloc")]
    pub fn to_str(
        &self,
        s: &str,
        output_fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
        lang: Lang,
    ) -> Result<alloc::string::String, DtErr> {
        let parts = Parts::from_str(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?;
        parts.to_dt()?.to_str(output_fmt, lang)
    }

    /// Formats a [`Dt`] into a [`LiteStr`] using this pre-validated format and a given
    /// output format.
    ///
    /// Effectively parses a [`str`] with the contained format, then outputs a
    /// [`LiteStr`] with a new given format.
    ///
    /// ## Parameters
    ///
    /// - `s`: datetime input [`str`].
    /// - `output_fmt`: The new format to output the datetime as.
    /// - The remaining three flags are passed through to the internal `to_dt` call.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Lang, StrPTimeFmt};
    ///
    /// let fmt = Dt::parse_fmt("%Y-%m-%dT%H:%M:%S").unwrap();
    /// let s = fmt.to_str_lite("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En).unwrap();
    ///
    /// assert_eq!(s.as_str(), "01 01 2000 12:00:00");
    /// ```
    pub fn to_str_lite(
        &self,
        s: &str,
        output_fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
        lang: Lang,
    ) -> Result<LiteStr<STRTIME_SIZE>, DtErr> {
        let parts = Parts::from_str(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?;
        parts.to_dt()?.to_str_lite(output_fmt, lang)
    }

    fn validate_format(mut fmt: &[u8]) -> Result<(), DtErr> {
        while !fmt.is_empty() {
            if fmt[0] != b'%' {
                // literal character (including whitespace) — always valid
                fmt = &fmt[1..];
                continue;
            }

            // lone % at end of format
            if fmt.len() == 1 {
                return Err(an_err!(DtErrKind::TruncatedDirective));
            }
            fmt = &fmt[1..]; // eat %

            // Skip format extensions (flag / width / colons)
            // Flag (at most one)
            if !fmt.is_empty() {
                match fmt[0] {
                    b'-' | b'_' | b'0' | b'^' | b'#' => {
                        fmt = &fmt[1..];
                    }
                    _ => {}
                }
            }

            // Width: consume all consecutive digits (parser consumes any number of digits)
            while !fmt.is_empty() && fmt[0].is_ascii_digit() {
                fmt = &fmt[1..];
            }

            // Colons: consume all consecutive colons
            while !fmt.is_empty() && fmt[0] == b':' {
                fmt = &fmt[1..];
            }

            if fmt.is_empty() {
                return Err(an_err!(DtErrKind::UnexpectedEnd));
            }

            let directive = fmt[0];

            match directive {
            // all currently supported directives
            b'%' | b'A' | b'a' | b'B' | b'b' | b'h' | b'C' | b'd' | b'e' |
            b'f' | b'N' | b'G' | b'g' | b'H' | b'k' | b'I' | b'l' | b'j' |
            b'J' | b'M' | b'm' | b'n' | b't' | b'P' | b'p' | b'Q' | b'S' | b's' |
            b'U' | b'u' | b'V' | b'W' | b'w' | b'Y' | b'y' | b'z' |
            // shortcuts
            b'F' | b'D' | b'T' | b'R' |
            // library directives
            b'L' | b'*' => {
                fmt = &fmt[1..];
            }

            b'.' => {
                // special case for %.f / %.3N / %-.3f etc.
                fmt = &fmt[1..]; // eat the .

                // optional width/precision digits (e.g. 3 in %.3N)
                while !fmt.is_empty() && fmt[0].is_ascii_digit() {
                    fmt = &fmt[1..];
                }

                if fmt.is_empty() {
                    return Err(an_err!(DtErrKind::ExpectedFractional));
                }
                let next = fmt[0];
                if !matches!(next, b'f' | b'N') {
                    return Err(an_err!(DtErrKind::InvalidFractional, "{}", char::from(next)));
                }
                fmt = &fmt[1..];
            }

            // explicitly unsupported
            b'c' | b'r' | b'X' | b'x' | b'Z' => {
                return Err(an_err!(
                    DtErrKind::UnsupportedItem,
                    "{}",
                    char::from(directive)
                ));
            }

            _ => {
                return Err(an_err!(DtErrKind::UnknownItem));
            }
        }
        }

        Ok(())
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.fmt[..self.len]
    }

    #[inline]
    fn as_str(&self) -> Result<&str, DtErr> {
        match core::str::from_utf8(self.as_bytes()) {
            Ok(f) => Ok(f),
            Err(e) => Err(an_err!(DtErrKind::InvalidBytes, "{}", e)),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for StrPTimeFmt {
    fn format(&self, f: defmt::Formatter) {
        match self.as_str() {
            Ok(fmt) => defmt::write!(f, "{}", fmt),
            Err(_) => defmt::write!(f, "StrPTimeFmt<invalid utf8>"),
        }
    }
}
