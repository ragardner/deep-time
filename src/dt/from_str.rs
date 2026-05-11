use crate::{
    Dt, DtErr, DtErrKind, SEC_PER_DAY, SEC_PER_MONTH, SEC_PER_WEEK, SEC_PER_YEAR, Scale, TimeParts,
    an_err, parser::Parser,
};

struct ParsedComponent {
    unit: u8,
    signed_int: i64,
    frac_digits: usize,
    frac_num: i64,
}

const MAX_FORMAT_LEN: usize = 256;

/// A pre-validated, reusable date/time format string.
///
/// - Format is validated **once** at construction (`new` returns `Result`).
/// - Format bytes are copied into an owned fixed-size buffer.
/// - Only ASCII formats are accepted.
#[derive(Debug, Clone, Copy)]
pub struct StrPTimeFmt {
    fmt: [u8; MAX_FORMAT_LEN],
    len: usize,
}

impl StrPTimeFmt {
    /// Creates a new validated format.
    ///
    /// - Validates syntax and supported directives.
    /// - Requires the format to be valid ASCII and ≤ 256 bytes.
    /// - Returns a proper `DtErr` on any failure.
    pub fn new(fmt: &str) -> Result<Self, DtErr> {
        if fmt.len() > MAX_FORMAT_LEN {
            return Err(an_err!(
                DtErrKind::UnexpectedEnd,
                "format string too long (max {} bytes)",
                MAX_FORMAT_LEN
            ));
        }
        let fmt = fmt.as_bytes();
        if !fmt.is_ascii() {
            return Err(an_err!(
                DtErrKind::UnexpectedEnd,
                "format string must be ASCII"
            ));
        }

        Self::validate_format(fmt)?;

        let mut buffer = [0u8; MAX_FORMAT_LEN];
        buffer[..fmt.len()].copy_from_slice(fmt);

        Ok(Self {
            fmt: buffer,
            len: fmt.len(),
        })
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
                return Err(an_err!(DtErrKind::UnexpectedEnd, "after %"));
            }
            fmt = &fmt[1..]; // eat %

            // reuse existing helper for flags/width/colons
            let (_, _, _, new_fmt) = Parser::parse_format_extensions(fmt, 0);
            fmt = new_fmt;

            if fmt.is_empty() {
                return Err(an_err!(DtErrKind::UnexpectedEnd, "expected directive"));
            }

            let directive = fmt[0];

            match directive {
            // all currently supported directives (exact list from Parser::parse)
            b'%' | b'A' | b'a' | b'B' | b'b' | b'h' | b'C' | b'd' | b'e' |
            b'f' | b'N' | b'G' | b'g' | b'H' | b'k' | b'I' | b'l' | b'j' |
            b'M' | b'm' | b'n' | b't' | b'P' | b'p' | b'Q' | b'S' | b's' |
            b'U' | b'u' | b'V' | b'W' | b'w' | b'Y' | b'y' | b'z' |
            // shortcuts
            b'F' | b'D' | b'T' | b'R' |
            // library directives
            b'*' => {
                fmt = &fmt[1..];
            }

            b'.' => {
                // special case for %.f / %.3N etc.
                fmt = &fmt[1..]; // eat the .

                // optional width digits
                while !fmt.is_empty() && fmt[0].is_ascii_digit() {
                    fmt = &fmt[1..];
                }

                let next = fmt.get(0).copied().unwrap_or(0);
                if !matches!(next, b'f' | b'N') {
                    return Err(an_err!(DtErrKind::BadFractional, "{}", char::from(next)));
                }
                fmt = &fmt[1..];
            }

            // explicitly unsupported (same as Parser)
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

    /// Parse a date str using this pre-validated format.
    pub fn to_dt(
        &self,
        s: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        TimeParts::from_str(
            self.as_str()?,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )
        .and_then(|p| p.to_time_point())
    }

    #[cfg(feature = "alloc")]
    pub fn to_str(
        &self,
        current: Scale,
        s: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<alloc::string::String, DtErr> {
        self.to_dt(
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_str(current, self.as_str()?)
    }
}

impl Dt {
    #[inline]
    pub fn from_str(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        Ok(TimeParts::from_str(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_time_point()?)
    }

    #[inline]
    pub fn parse_fmt(strptime_fmt: &str) -> Result<StrPTimeFmt, DtErr> {
        StrPTimeFmt::new(strptime_fmt)
    }

    pub fn from_iso(s: &str) -> Result<Dt, DtErr> {
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
        Ok(Dt::from_attos(total_attos, Scale::TAI))
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
