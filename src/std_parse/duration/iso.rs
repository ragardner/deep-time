use crate::{
    SECONDS_PER_DAY, SECONDS_PER_MONTH, SECONDS_PER_WEEK, SECONDS_PER_YEAR, TimeSpan, str_err,
};
use std::string::String;

pub(crate) fn parse_iso_duration_span(s: &str) -> Result<TimeSpan, String> {
    let len = s.len();
    if len == 0 {
        return Err(str_err!("duration is empty"));
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
        return Err(str_err!("ISO duration must start with P, +P or -P"));
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
                return Err(str_err!(
                    "T separator present but no time components follow"
                ));
            }
            if b[pos + 1..].iter().any(|&c| matches!(c, b'T' | b't')) {
                return Err(str_err!("Multiple 'T' separators not allowed"));
            }
            (&b[i..pos], &b[pos + 1..])
        }
        None => (&b[i..], &[] as &[u8]),
    };

    let mut has_fraction = false;
    let mut total_nanos: i128 = 0;

    // Both date and time parts now use the same fixed-length logic
    parse_duration_part(date_part, &mut total_nanos, true, sign, &mut has_fraction)?;
    parse_duration_part(time_part, &mut total_nanos, false, sign, &mut has_fraction)?;

    // Convert accumulated nanoseconds to attoseconds and build TimeSpan
    let total_attos = total_nanos * 1_000_000_000i128;
    Ok(TimeSpan::from_total_attos(total_attos))
}

struct ParsedComponent {
    unit: u8,
    signed_int: i64,
    frac_digits: usize,
    frac_num: i64,
}

/// Parses a single component (number + optional fraction + unit) from the slice,
/// advancing the index `i`. Returns `None` when the slice is exhausted.
fn parse_next_component(
    chars: &[u8],
    i: &mut usize,
    sign: i64,
    has_fraction: &mut bool,
) -> Result<Option<ParsedComponent>, String> {
    if *i >= chars.len() {
        return Ok(None);
    }

    if *has_fraction {
        return Err(str_err!(
            "Could not parse duration: {:?} no components allowed after a fractional component",
            &chars[*i..]
        ));
    }

    // Parse integer part
    let start = *i;
    while *i < chars.len() && chars[*i].is_ascii_digit() {
        *i += 1;
    }
    if start == *i {
        return Err(str_err!(
            "Could not parse duration: {:?} expected a number",
            &chars[start..]
        ));
    }
    let int: i64 = std::str::from_utf8(&chars[start..*i])
        .unwrap()
        .parse()
        .map_err(|e: std::num::ParseIntError| str_err!("{}", e))?;

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
            return Err(str_err!(
                "Could not parse duration: {:?} empty fraction after decimal/comma",
                &chars[start..]
            ));
        }
        if frac_digits > 9 {
            return Err(str_err!(
                "Could not parse duration: {:?} too many decimal places (max 9 for nanosecond precision)",
                &chars[start..]
            ));
        }
        frac_num = std::str::from_utf8(&chars[frac_start..*i])
            .unwrap()
            .parse()
            .map_err(|e: std::num::ParseIntError| str_err!("{}", e))?;
    }

    // Unit must follow
    if *i >= chars.len() {
        return Err(str_err!(
            "Could not parse duration: {:?} Missing unit after number",
            &chars[start..]
        ));
    }
    let unit = chars[*i];
    *i += 1;

    // Only seconds support a fractional part
    if frac_digits > 0 {
        if !matches!(unit, b'S' | b's') {
            return Err(str_err!(
                "Could not parse duration: {:?} Fractional components are only supported for seconds",
                &chars[start..]
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
/// constants (the only sensible semantics for a pure `TimeSpan`).
fn parse_duration_part(
    chars: &[u8],
    total_nanos: &mut i128,
    is_date: bool,
    sign: i64,
    has_fraction: &mut bool,
) -> Result<(), String> {
    let mut i = 0;
    while let Some(comp) = parse_next_component(chars, &mut i, sign, has_fraction)? {
        let contrib_nanos = match (is_date, comp.unit) {
            (true, b'Y' | b'y') => {
                let total_secs = (comp.signed_int as i128)
                    .checked_mul(SECONDS_PER_YEAR)
                    .ok_or_else(|| str_err!("year value out of range"))?;
                total_secs * 1_000_000_000i128
            }
            (true, b'M' | b'm') => {
                let total_secs = (comp.signed_int as i128)
                    .checked_mul(SECONDS_PER_MONTH)
                    .ok_or_else(|| str_err!("month value out of range"))?;
                total_secs * 1_000_000_000i128
            }
            (true, b'W' | b'w') => {
                let total_secs = (comp.signed_int as i128)
                    .checked_mul(SECONDS_PER_WEEK)
                    .ok_or_else(|| str_err!("week value out of range"))?;
                total_secs * 1_000_000_000i128
            }
            (true, b'D' | b'd') => {
                let total_secs = (comp.signed_int as i128)
                    .checked_mul(SECONDS_PER_DAY)
                    .ok_or_else(|| str_err!("day value out of range"))?;
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
                return Err(str_err!(
                    "Could not parse duration: {:?} Unknown duration unit: {}",
                    chars,
                    comp.unit as char
                ));
            }
        };

        *total_nanos = total_nanos.saturating_add(contrib_nanos);
    }
    Ok(())
}

/// Accepts: `P1Y`, `-P2W`, `PT1.5H`, `P1DT2H30M`, `+P3D`, `p1y`, `P1,5S`, `PT0S`, etc.
/// Rejects: anything with whitespace, lone "P"/"-P"/"PT", "P123", "Please wait 5m",
///          "1.5h", "P1Yabc", "P1Y!", or **any string longer than 128 bytes**.
pub(crate) fn looks_like_iso_duration(s: &str) -> bool {
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
            b'Y' | b'y' | b'M' | b'm' | b'W' | b'w' | b'D' | b'd' | b'T' | b't' | b'H' | b'h'
            | b'S' | b's' => {
                has_designator = true;
            }
            _ => return false, // any other character = not ISO
        }

        i += 1;
    }
    // Must contain at least one digit *and* one designator after the initial P
    has_digit && has_designator
}
