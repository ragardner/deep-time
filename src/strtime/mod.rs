pub mod parser;
pub mod printer;

use crate::error::{DtErr, DtErrKind};
use crate::{BufStr, Dt, Lang, Parts, STRTIME_SIZE, an_err};
use core::result::Result;
use core::str;

pub(crate) use parser::*;
pub(crate) use printer::*;

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

/// A pre-validated, reusable **parse** format for `strptime`-style directives.
///
/// Construct once with [`StrPTimeFmt::new`](../struct.StrPTimeFmt.html#method.new)
/// or [`Dt::parse_fmt`](../struct.Dt.html#method.parse_fmt). On success the format
/// is known to be syntactically valid for this crate’s parser, so later use will
/// not fail because the format itself is unusable.
///
/// ## What this type is for
///
/// - **Fail-fast format checking** — catch unknown, unsupported, or truncated
///   `%` directives at construction time (e.g. config load), not on every parse.
/// - **Reuse** — store the validated format and call
///   [`to_dt`](../struct.StrPTimeFmt.html#method.to_dt) (or reformat helpers)
///   many times without re-checking directive syntax.
///
/// ## Guarantee
///
/// After `new` succeeds, parsing with this value
/// ([`to_dt`](../struct.StrPTimeFmt.html#method.to_dt), and the parse step of
/// [`to_str`](../struct.StrPTimeFmt.html#method.to_str) /
/// [`to_str_b`](../struct.StrPTimeFmt.html#method.to_str_b)) will **not** fail
/// with format-structure errors such as [`DtErrKind::UnknownItem`],
/// [`DtErrKind::UnsupportedItem`], [`DtErrKind::TruncatedDirective`], or
/// [`DtErrKind::InvalidFractional`].
///
/// Parsing can still fail for **input** reasons: mismatched literals, missing
/// fields, out-of-range values, incomplete dates, trailing characters, bad
/// offsets, and so on. See
/// [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime).
///
/// ## What this type is *not*
///
/// - **Not a speed optimization.** Validation does not make strptime/strftime
///   faster; it only checks the format once and stores a copy.
/// - **Not a print-format type.** The stored string is the **input** (parse)
///   pattern. Methods that take an `output_fmt` argument do **not** pre-validate
///   that second format — only the format inside this struct is guaranteed.
/// - **Not the free-form auto-parser.** For guessing formats from text, use
///   [`Dt::from_str`](../struct.Dt.html#method.from_str) / the `parse` feature,
///   not this type.
///
/// ## Constraints
///
/// - Format must be **ASCII** only.
/// - Format length must be ≤
///   [`MAX_FMT_LEN`](../struct.StrPTimeFmt.html#associatedconstant.MAX_FMT_LEN)
///   (256 bytes).
/// - Bytes are copied into an owned fixed-size buffer (no heap allocation).
/// - Directives match the parse grammar documented on
///   [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime) (including
///   library extensions such as `%L`, `%*`, `%J`, `%Q`). Printer-only spellings
///   (e.g. trim flag `~` in `%.~f`) are **not** accepted here.
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, StrPTimeFmt};
///
/// // Prefer validating once (e.g. at startup) then reusing.
/// let fmt = StrPTimeFmt::new("%Y-%m-%d %H:%M:%S").unwrap();
/// // equivalent: Dt::parse_fmt("%Y-%m-%d %H:%M:%S").unwrap();
///
/// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
/// assert_eq!(dt.to_ymd().yr(), 2025);
///
/// // Construction fails on a broken format — before any input is parsed.
/// assert!(StrPTimeFmt::new("%Y-%m-%").is_err());
/// assert!(StrPTimeFmt::new("%c").is_err()); // unsupported locale directive
/// ```
///
/// ## See also
///
/// - [`StrPTimeFmt::new`](../struct.StrPTimeFmt.html#method.new)
/// - [`StrPTimeFmt::to_dt`](../struct.StrPTimeFmt.html#method.to_dt)
/// - [`StrPTimeFmt::to_str_b`](../struct.StrPTimeFmt.html#method.to_str_b)
/// - [`StrPTimeFmt::to_str`](../struct.StrPTimeFmt.html#method.to_str)
/// - [`Dt::parse_fmt`](../struct.Dt.html#method.parse_fmt)
#[derive(Debug, Clone)]
pub struct StrPTimeFmt {
    fmt: [u8; Self::MAX_FMT_LEN],
    len: usize,
}

impl StrPTimeFmt {
    /// Maximum length of a format string in bytes (not characters).
    ///
    /// Longer strings are rejected with [`DtErrKind::InvalidLen`] at construction.
    pub const MAX_FMT_LEN: usize = 256;

    /// Validates a `strptime`-style format and stores it for reuse.
    ///
    /// On success, the returned value carries the guarantee described on
    /// [`StrPTimeFmt`](../struct.StrPTimeFmt.html): the format will not fail later
    /// for structural reasons when used to **parse**.
    ///
    /// Accepts the same `%` directives and extensions as
    /// [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime) (one optional
    /// flag `-`/`_`/`0`/`^`/`#`, optional width digits, up to three colons for
    /// offsets, and `%.f` / `%.N` fractional forms).
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::InvalidLen`] — longer than
    ///   [`MAX_FMT_LEN`](../struct.StrPTimeFmt.html#associatedconstant.MAX_FMT_LEN).
    /// - [`DtErrKind::InvalidInput`] — not valid ASCII.
    /// - [`DtErrKind::TruncatedDirective`] — a bare `%` at end of format (no
    ///   directive letter).
    /// - [`DtErrKind::UnexpectedEnd`] — `%` followed only by flags, width, and/or
    ///   colons, with no directive letter.
    /// - [`DtErrKind::ExpectedFractional`] — a `%.` sequence with nothing after
    ///   optional digits (incomplete fractional directive).
    /// - [`DtErrKind::InvalidFractional`] — a `%.` sequence followed by a character
    ///   other than `f` or `N` (also rejects printer-only forms such as `%.~f`).
    /// - [`DtErrKind::UnsupportedItem`] — `%c`, `%r`, `%x`, `%X`, or `%Z`.
    /// - [`DtErrKind::UnknownItem`] — any other unrecognized `%` directive, or
    ///   more than three colons before a directive (e.g. `%::::z`).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::StrPTimeFmt;
    ///
    /// let fmt = StrPTimeFmt::new("%F %T").unwrap();
    /// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
    /// assert_eq!(dt.to_ymd().day(), 23);
    /// ```
    pub fn new(fmt: &str) -> Result<Self, DtErr> {
        if fmt.len() > Self::MAX_FMT_LEN {
            return Err(an_err!(DtErrKind::InvalidLen));
        }
        let fmt = fmt.as_bytes();
        if !fmt.is_ascii() {
            return Err(an_err!(DtErrKind::InvalidInput, "must be ascii"));
        }

        Self::validate_format(fmt)?;

        let mut buffer = [0u8; Self::MAX_FMT_LEN];
        buffer[..fmt.len()].copy_from_slice(fmt);

        Ok(Self {
            fmt: buffer,
            len: fmt.len(),
        })
    }

    /// Parses `s` with this format and converts the result to a [`Dt`].
    ///
    /// Equivalent to
    /// [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime)`(s, fmt, …)`
    /// where `fmt` is the string validated at construction — except the format
    /// has already been checked, so failures should only reflect the **input**
    /// (or post-parse conversion), not broken directive syntax.
    ///
    /// ## Parameters
    ///
    /// - `s`: The input string to parse.
    /// - `inp_can_end_before_fmt`: Allow input to end before the format is fully
    ///   consumed (parse what is present, ignore remaining directives).
    /// - `fmt_can_end_before_inp`: Allow the format to end before the input is
    ///   fully consumed (trailing characters in `s` are OK).
    /// - `allow_partial_date`: Default missing month/day to `1` instead of
    ///   returning [`DtErrKind::Incomplete`].
    ///
    /// Full directive list and flag semantics:
    /// [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime).
    ///
    /// ## Errors
    ///
    /// Same **input** and conversion errors as
    /// [`Dt::from_strptime`](../struct.Dt.html#method.from_strptime) (mismatch,
    /// expected fields, out of range, trailing characters, incomplete date,
    /// timezone issues, etc.). Format-structure errors for the stored pattern
    /// are not expected after a successful
    /// [`new`](../struct.StrPTimeFmt.html#method.new).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::StrPTimeFmt;
    ///
    /// let fmt = StrPTimeFmt::new("%F %T").unwrap();
    /// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
    /// assert_eq!(dt.to_ymd().hr(), 14);
    /// ```
    pub fn to_dt(
        &self,
        s: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        Parts::from_strptime(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )
        .and_then(|p| p.to_dt())
    }

    /// Parse `s` with this format, then format the resulting [`Dt`] with
    /// `output_fmt` into an owned [`String`](`alloc::string::String`).
    ///
    /// Requires the `alloc` feature. Prefer
    /// [`to_str_b`](../struct.StrPTimeFmt.html#method.to_str_b) when you want a
    /// fixed stack buffer and no allocator.
    ///
    /// **Important:** only the format stored in `self` is pre-validated.
    /// `output_fmt` is passed straight to formatting and may fail with
    /// format-structure errors (truncated `%`, unknown directives, etc.) if it
    /// is invalid. Validate it separately with
    /// [`StrPTimeFmt::new`](../struct.StrPTimeFmt.html#method.new) if you need
    /// the same guarantee for a parse pattern (some print-only spellings are
    /// still rejected by `new`).
    ///
    /// ## Parameters
    ///
    /// - `s`: Input datetime string, parsed with `self`.
    /// - `output_fmt`: Format string for the **output** (not pre-validated).
    /// - `inp_can_end_before_fmt`, `fmt_can_end_before_inp`, `allow_partial_date`:
    ///   passed through to [`to_dt`](../struct.StrPTimeFmt.html#method.to_dt).
    /// - `lang`: Language for weekday/month names in the output.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use deep_time::{Lang, StrPTimeFmt};
    ///
    /// let fmt = StrPTimeFmt::new("%Y-%m-%dT%H:%M:%S").unwrap();
    /// let s = fmt
    ///     .to_str("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En)
    ///     .unwrap();
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
        Parts::from_strptime(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_dt()?
        .to_str(output_fmt, lang)
    }

    /// Parse `s` with this format, then format the resulting [`Dt`] with
    /// `output_fmt` into a stack [`BufStr`].
    ///
    /// Same pipeline as [`to_str`](../struct.StrPTimeFmt.html#method.to_str),
    /// without requiring `alloc`.
    ///
    /// **Important:** only the format stored in `self` is pre-validated.
    /// `output_fmt` is not checked at construction of this value and may fail
    /// if it contains invalid or unsupported directives. See
    /// [`to_str`](../struct.StrPTimeFmt.html#method.to_str) for details.
    ///
    /// ## Parameters
    ///
    /// - `s`: Input datetime string, parsed with `self`.
    /// - `output_fmt`: Format string for the **output** (not pre-validated).
    /// - `inp_can_end_before_fmt`, `fmt_can_end_before_inp`, `allow_partial_date`:
    ///   passed through to [`to_dt`](../struct.StrPTimeFmt.html#method.to_dt).
    /// - `lang`: Language for weekday/month names in the output.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Lang, StrPTimeFmt};
    ///
    /// let fmt = StrPTimeFmt::new("%Y-%m-%dT%H:%M:%S").unwrap();
    /// let s = fmt
    ///     .to_str_b("2000-01-01T12:00:00", "%d %m %Y %H:%M:%S", false, false, false, Lang::En)
    ///     .unwrap();
    /// assert_eq!(s.as_str(), "01 01 2000 12:00:00");
    /// ```
    pub fn to_str_b(
        &self,
        s: &str,
        output_fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
        lang: Lang,
    ) -> Result<BufStr<STRTIME_SIZE>, DtErr> {
        Parts::from_strptime(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_dt()?
        .to_str_b(output_fmt, lang)
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

            // Colons: match the parser — hard cap at 3. Extra `:` become the
            // directive byte and must be rejected as unknown (e.g. `%::::z`).
            let mut colons = 0u8;
            while colons < 3 && !fmt.is_empty() && fmt[0] == b':' {
                fmt = &fmt[1..];
                colons += 1;
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

impl Dt {
    /// Validates a `strptime`-style format into a reusable [`StrPTimeFmt`].
    ///
    /// Convenience alias for
    /// [`StrPTimeFmt::new`](../struct.StrPTimeFmt.html#method.new). The format is
    /// checked once for syntax and supported directives, then stored in a
    /// fixed-size buffer.
    ///
    /// After this succeeds, parsing with the returned value will not fail because
    /// the format itself is unusable — only because the **input** does not match
    /// (or conversion fails). See the guarantee on
    /// [`StrPTimeFmt`](../struct.StrPTimeFmt.html).
    ///
    /// This is **not** a performance optimization; it only validates and owns a
    /// copy of the format. Only ASCII formats up to
    /// [`StrPTimeFmt::MAX_FMT_LEN`](../struct.StrPTimeFmt.html#associatedconstant.MAX_FMT_LEN)
    /// bytes are accepted.
    ///
    /// ## Parameters
    ///
    /// - `strptime_fmt`: Parse format using `%` directives (e.g. `"%Y-%m-%d %H:%M:%S"`,
    ///   `"%F %T"`, `"%Y-%m-%dT%H:%M:%S%.3fZ"`). Same grammar as
    ///   [`from_strptime`](../struct.Dt.html#method.from_strptime).
    ///
    /// ## Errors
    ///
    /// Same as [`StrPTimeFmt::new`](../struct.StrPTimeFmt.html#method.new): overlong,
    /// non-ASCII, truncated/malformed directives, unknown or unsupported items
    /// (`%c`, `%r`, `%x`, `%X`, `%Z`, etc.).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let fmt = Dt::parse_fmt("%F %T").unwrap();
    /// let dt = fmt.to_dt("2025-05-23 14:30:00", false, false, false).unwrap();
    /// assert_eq!(dt.to_ymd().min(), 30);
    /// ```
    #[inline(always)]
    pub fn parse_fmt(strptime_fmt: &str) -> Result<StrPTimeFmt, DtErr> {
        StrPTimeFmt::new(strptime_fmt)
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
