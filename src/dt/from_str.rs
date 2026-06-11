use crate::{
    Dt, DtErr, DtErrKind, SEC_PER_DAY, SEC_PER_MONTH, SEC_PER_WEEK, SEC_PER_YEAR, StrPTimeFmt,
    TimeParts, an_err,
};
use core::str::FromStr;

#[cfg(feature = "parse")]
impl FromStr for Dt {
    type Err = DtErr;

    #[inline]
    fn from_str(s: &str) -> Result<Self, DtErr> {
        Dt::from_str_parse(s, &None)
    }
}

#[cfg(not(feature = "parse"))]
impl FromStr for Dt {
    type Err = DtErr;

    #[inline]
    fn from_str(s: &str) -> Result<Self, DtErr> {
        Self::from_str_iso(s)
    }
}

struct ParsedComponent {
    unit: u8,
    signed_int: i64,
    frac_digits: usize,
    frac_num: i64,
}

impl Dt {
    /// Parses a date/time string.
    ///
    /// - When the `parse` feature is enabled: uses the smart auto-parser.
    /// - When the `parse` feature is disabled: falls back to CCSDS format.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // uses impl FromStr but Dt::parse provides the same functionality
    /// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
    ///
    /// let ymd = x.to_ymd();
    /// assert_eq!(ymd.yr(), 2000);
    /// assert_eq!(ymd.mo(), 1);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse)
    /// - [`Dt::from_str_iso`](../struct.Dt.html#method.from_str_iso)
    #[inline(always)]
    pub fn parse(s: &str) -> Result<Self, DtErr> {
        #[cfg(feature = "parse")]
        {
            Self::from_str_parse(s, &None)
        }
        #[cfg(not(feature = "parse"))]
        {
            Self::from_str_iso(s)
        }
    }

    /// High-level parser equivalent to C `strptime` (and Python `strptime`).
    ///
    /// Parses the input string `s` according to the supplied format string `fmt`
    /// and returns a [`Dt`] directly. This is a convenience wrapper around
    /// [`TimeParts::from_str`](../struct.TimeParts.html#method.from_str)
    /// followed by [`TimeParts::to_dt`](../struct.TimeParts.html#method.to_dt).
    ///
    /// It supports the same set of `%` directives as the low-level parser, pretty
    /// much the same as jiff.
    ///
    /// ## Parameters
    ///
    /// - `s`: The date/time string to parse.
    /// - `fmt`: The format string containing `%` directives (must be valid ASCII).
    /// - `inp_can_end_before_fmt`: If `true`, the input may end before the format
    ///   string is fully consumed (extra format specifiers are ignored).
    /// - `fmt_can_end_before_inp`: If `true`, the format may end before the input
    ///   is fully consumed (trailing characters in the input are allowed).
    /// - `allow_partial_date`: If `true`, a missing month/day will be defaulted
    ///   to `1` instead of returning an [`Incomplete`] error.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] for:
    /// - Parse failures (`InvalidFormat`, `OutOfRange`, `UnknownItem`, etc.)
    /// - Incomplete data when `allow_partial_date` is `false`
    /// - Trailing characters (when `fmt_can_end_before_inp` is `false`)
    ///
    /// See [`TimeParts::from_str`] for the complete list of supported directives
    /// and detailed parsing semantics.
    #[inline(always)]
    pub fn from_str(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        TimeParts::from_str(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_dt()
    }

    /// Parses and validates a `strptime`-style format string into a reusable [`StrPTimeFmt`].
    ///
    /// The format is checked once for syntax errors and unsupported directives,
    /// then stored in a compact fixed-size buffer. The resulting `StrPTimeFmt` is
    /// `Copy`, cheap to clone, and can be used repeatedly with [`StrPTimeFmt::to_dt`]
    /// and [`StrPTimeFmt::to_str`] without re-validating.
    ///
    /// Only ASCII formats up to 256 bytes are accepted.
    ///
    /// ## Parameters
    ///
    /// - `strptime_fmt`: The format string using `%` directives (e.g. `"%Y-%m-%d %H:%M:%S"`,
    ///   `"%F %T"`, `"%Y-%m-%dT%H:%M:%S%.3fZ"`).
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format is:
    /// - Longer than 256 bytes
    /// - Not valid ASCII
    /// - Contains unknown, unsupported, or malformed directives
    #[inline(always)]
    pub fn parse_fmt(strptime_fmt: &str) -> Result<StrPTimeFmt, DtErr> {
        StrPTimeFmt::new(strptime_fmt)
    }

    /// Generalized ISO / CCSDS ASCII Time Code parser (A or B variant).
    /// - Parses e.g. **`+2000-01-01T17:00:00 -0500 [America/New_York] TAI`**.
    /// - Only supports ASCII characters.
    /// - If a time is included then some kind of date-time separator e.g. `T` is
    ///   required.
    /// - Supports both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// - Supported **optional** components:
    ///     - Time components after a date e.g. `T12:00:00`.
    ///     - Offset after time components or directly after the date e.g. `+0200` or
    ///       `2023-01-01+05:00`.
    ///     - Timezone name, **requires square brackets** and requires `jiff-tz` feature,
    ///       after time or offset e.g. `T12:00:00 [America/New_York]`.
    ///     - Library time scale right on the end of the input, e.g. `TAI`.
    #[inline(always)]
    pub fn from_str_iso(input: &str) -> Result<Self, DtErr> {
        TimeParts::from_str_iso(input)?.to_dt()
    }

    /// Parses an ISO 8601 duration string into a [`Dt`] representing a pure time interval.
    ///
    /// Supports the full `PnYnMnDTnHnMnS` format (case-insensitive), including:
    /// - Optional leading `+` or `-` sign
    /// - `P` / `p` prefix (required)
    /// - Optional `T` / `t` separator between date and time parts
    /// - Weeks (`W` / `w`)
    /// - Fractional seconds with up to 18 digits of precision (attosecond resolution)
    ///
    /// The returned [`Dt`] is a **duration** (signed interval) on the TAI scale.
    /// It can be added to/subtracted from other `Dt` values, multiplied/divided,
    /// rounded, etc.
    ///
    /// ## Not Reference-Time Aware
    ///
    /// This parser is **not reference-time aware**. Calendar units (`Y`, `M`) are
    /// converted to a fixed number of seconds using standard average lengths
    /// rather than being resolved against a specific date. This makes parsing
    /// fast and allocation-free, but `P1M` always represents exactly the same
    /// duration regardless of context.
    ///
    /// ## Parameters
    ///
    /// - `s`: The ISO 8601 duration string (e.g. `"P1Y2M3DT4H5M6.123456789012345678S"`,
    ///   `"-PT30M"`, `"P7W"`, `"+P1DT12H"`).
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] for:
    /// - Empty string
    /// - Missing `P` prefix
    /// - Invalid syntax (`T` with no time part, multiple `T`s, etc.)
    /// - Unknown unit designators
    /// - Numeric values that are out of range or cause overflow
    pub fn from_iso_duration(s: &str) -> Result<Dt, DtErr> {
        let len = s.len();
        if len == 0 {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        let b = s.as_bytes();
        let mut i = 0usize;

        // Optional leading sign (+ or -)
        let mut sign: i64 = 1;
        if i < len && matches!(b[i], b'+' | b'-') {
            if b[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        // Must start with P/p
        if i >= len || !matches!(b[i], b'P' | b'p') {
            return Err(an_err!(DtErrKind::MustStartWith, "P"));
        }
        i += 1;

        // Find the (single) T/t separator
        let t_pos = b[i..]
            .iter()
            .position(|&c| matches!(c, b'T' | b't'))
            .map(|p| i + p);

        let (date_part, time_part) = match t_pos {
            Some(pos) => {
                if pos == len - 1 {
                    return Err(an_err!(DtErrKind::InvalidSyntax, "T with no time"));
                }
                if b[pos + 1..].iter().any(|&c| matches!(c, b'T' | b't')) {
                    return Err(an_err!(DtErrKind::InvalidSyntax, "multiple T"));
                }
                (&b[i..pos], &b[pos + 1..])
            }
            None => (&b[i..], &[] as &[u8]),
        };

        let mut has_fraction = false;
        let mut total_nanos: i128 = 0;

        // Both date and time parts now use the same fixed-length logic
        Self::parse_duration_part(date_part, &mut total_nanos, true, sign, &mut has_fraction)?;
        Self::parse_duration_part(time_part, &mut total_nanos, false, sign, &mut has_fraction)?;

        // Convert accumulated nanoseconds to attoseconds and build Dt
        let total_attos = total_nanos * 1_000_000_000i128;
        Ok(Dt::span(total_attos))
    }

    /// Parses a single component (number + optional fraction + unit) from the slice,
    /// advancing the index `i`. Returns `None` when the slice is exhausted.
    fn parse_next_component(
        chars: &[u8],
        i: &mut usize,
        sign: i64,
        has_fraction: &mut bool,
    ) -> Result<Option<ParsedComponent>, DtErr> {
        if *i >= chars.len() {
            return Ok(None);
        }

        if *has_fraction {
            return Err(an_err!(DtErrKind::InvalidSyntax, "components after frac"));
        }

        // Parse integer part
        let start = *i;
        while *i < chars.len() && chars[*i].is_ascii_digit() {
            *i += 1;
        }
        if start == *i {
            return Err(an_err!(DtErrKind::ExpectedValue, "number"));
        }

        let int_str = core::str::from_utf8(&chars[start..*i])
            .map_err(|_| an_err!(DtErrKind::InvalidNumber, "invalid utf8 in int"))?;
        let int: i64 = int_str.parse().map_err(|e: core::num::ParseIntError| {
            an_err!(DtErrKind::InvalidNumber, "{}: {}", int_str, e)
        })?;

        // Parse optional fraction
        let mut frac_num: i64 = 0;
        let mut frac_digits: usize = 0;
        if *i < chars.len() && matches!(chars[*i], b'.' | b',') {
            *i += 1;
            let frac_start = *i;
            while *i < chars.len() && chars[*i].is_ascii_digit() {
                *i += 1;
            }
            frac_digits = *i - frac_start;
            if frac_digits == 0 {
                return Err(an_err!(DtErrKind::ExpectedValue, "empty frac after ."));
            }
            if frac_digits > 9 {
                return Err(an_err!(DtErrKind::OutOfRange, "frac >9"));
            }

            let frac_str = core::str::from_utf8(&chars[frac_start..*i])
                .map_err(|_| an_err!(DtErrKind::InvalidNumber, "invalid utf8 in frac"))?;
            frac_num = frac_str.parse().map_err(|e: core::num::ParseIntError| {
                an_err!(DtErrKind::InvalidNumber, "{}: {}", frac_str, e)
            })?;
        }

        // Unit must follow
        if *i >= chars.len() {
            return Err(an_err!(
                DtErrKind::InvalidSyntax,
                "missing unit after number"
            ));
        }
        let unit = chars[*i];
        *i += 1;

        // Only seconds support a fractional part
        if frac_digits > 0 {
            if !matches!(unit, b'S' | b's') {
                return Err(an_err!(
                    DtErrKind::InvalidSyntax,
                    "frac only supported for seconds"
                ));
            }
            *has_fraction = true;
        }

        let signed_int = (int as i128 * sign as i128) as i64;

        Ok(Some(ParsedComponent {
            unit,
            signed_int,
            frac_digits,
            frac_num,
        }))
    }

    /// Helper that parses **one section** of an ISO duration (date or time part)
    /// and accumulates nanoseconds into `total_nanos`.
    ///
    /// Years, months, weeks, and days are converted using the fixed-length
    /// constants (the only sensible semantics for a pure `Dt`).
    fn parse_duration_part(
        chars: &[u8],
        total_nanos: &mut i128,
        is_date: bool,
        sign: i64,
        has_fraction: &mut bool,
    ) -> Result<(), DtErr> {
        let mut i = 0;
        while let Some(comp) = Self::parse_next_component(chars, &mut i, sign, has_fraction)? {
            let contrib_nanos = match (is_date, comp.unit) {
                (true, b'Y' | b'y') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_YEAR)
                        .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "year"))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'M' | b'm') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_MONTH)
                        .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "month"))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'W' | b'w') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_WEEK as i128)
                        .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "week"))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'D' | b'd') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_DAY)
                        .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "day"))?;
                    total_secs * 1_000_000_000i128
                }
                (false, b'H' | b'h') => (comp.signed_int as i128) * 3_600_000_000_000i128,
                (false, b'M' | b'm') => (comp.signed_int as i128) * 60_000_000_000i128,
                (false, b'S' | b's') => {
                    let mut sec_nanos = (comp.signed_int as i128) * 1_000_000_000i128;
                    if comp.frac_digits > 0 {
                        let frac_ns = (comp.frac_num as i128 * sign as i128 * 1_000_000_000i128)
                            / 10i128.pow(comp.frac_digits as u32);
                        sec_nanos += frac_ns;
                    }
                    sec_nanos
                }
                _ => {
                    return Err(an_err!(DtErrKind::InvalidItem, "{}", comp.unit as char));
                }
            };

            *total_nanos = total_nanos.saturating_add(contrib_nanos);
        }
        Ok(())
    }

    /// Accepts: `P1Y`, `-P2W`, `PT1.5H`, `P1DT2H30M`, `+P3D`, `p1y`, `P1,5S`, `PT0S`, etc.
    /// Rejects: anything with whitespace, lone "P"/"-P"/"PT", "P123", "Please wait 5m",
    ///          "1.5h", "P1Yabc", "P1Y!", or **any string longer than 128 bytes**.
    pub fn looks_like_iso(s: &str) -> bool {
        let len = s.len();
        if matches!(len, 0 | 1) {
            return false;
        }
        let b = s.as_bytes();
        let mut i = 0usize;
        // Optional leading sign
        if matches!(b[0], b'+' | b'-') {
            i += 1;
        }
        // Must start with P/p after optional sign
        if !matches!(b[i], b'P' | b'p') {
            return false;
        }
        i += 1;
        let mut has_digit = false;
        let mut has_designator = false;
        while i < len {
            match b[i] {
                b'0'..=b'9' => has_digit = true,
                b'.' | b',' => {} // decimal separators allowed by ISO 8601
                b'Y' | b'y' | b'M' | b'm' | b'W' | b'w' | b'D' | b'd' | b'T' | b't' | b'H'
                | b'h' | b'S' | b's' => {
                    has_designator = true;
                }
                _ => return false, // any other character = not ISO
            }

            i += 1;
        }
        // Must contain at least one digit *and* one designator after the initial P
        has_digit && has_designator
    }
}
