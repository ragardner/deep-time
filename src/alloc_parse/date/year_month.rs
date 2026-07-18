use crate::{Dt, MAX_YEAR, MIN_YEAR, Parts, Scale};

/// 6-digit legacy date: YYMMDD (e.g. "240315")
#[inline]
pub(crate) fn parse_yymmdd(input: &str) -> Option<Dt> {
    let parsed = Parts::from_str("%y%m%d", input, true, true, false).ok()?;
    parsed.to_dt().ok()
}

/// Parses year-month (no day) with flexible separators:
/// - year-first: `"2024-03"`, `"2024/3"`, `"2024.03"`, `"-2024-03"`, `"+2024-05"`, …
/// - month-first: `"03/2024"`, `"3-2024"`, `"12.2024"` (4-digit year only)
///
/// Day defaults to the 1st. Month-first requires a 4-digit year so `"03/24"` stays
/// ambiguous/rejected rather than guessing century.
pub(crate) fn parse_yyyy_mm(bytes: &[u8]) -> Option<Dt> {
    parse_yyyy_mm_year_first(bytes).or_else(|| parse_mm_yyyy(bytes))
}

#[inline]
fn ymd_first_of_month(year: i32, month: u32) -> Option<Dt> {
    if month == 0 || month > 12 {
        return None;
    }
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    Some(Dt::from_ymd(
        year as i64,
        month as u8,
        1,
        Scale::UTC,
        0,
        0,
        0,
        0,
    ))
}

/// Year-first: optional sign, year digits, separator, 1–2 digit month.
fn parse_yyyy_mm_year_first(bytes: &[u8]) -> Option<Dt> {
    let len = bytes.len();

    let (sign, mut pos) = match bytes.first() {
        Some(b'+') => (1i32, 1),
        Some(b'-') => (-1i32, 1),
        _ => (1i32, 0),
    };

    let mut year = 0i32;
    let year_start = pos;
    while pos < len && bytes[pos].is_ascii_digit() {
        year = year * 10 + (bytes[pos] - b'0') as i32;
        pos += 1;
    }
    if pos == year_start || pos == len {
        return None;
    }

    if !matches!(bytes.get(pos), Some(b'-' | b'/' | b'.')) {
        return None;
    }
    pos += 1;

    let month = match len - pos {
        1 => {
            let b = bytes[pos];
            if !b.is_ascii_digit() {
                return None;
            }
            (b - b'0') as u32
        }
        2 => {
            let b1 = bytes[pos];
            let b2 = bytes[pos + 1];
            if !b1.is_ascii_digit() || !b2.is_ascii_digit() {
                return None;
            }
            10 * (b1 - b'0') as u32 + (b2 - b'0') as u32
        }
        _ => return None,
    };

    ymd_first_of_month(year * sign, month)
}

/// Month-first: 1–2 digit month, separator, exactly 4 digit year.
fn parse_mm_yyyy(bytes: &[u8]) -> Option<Dt> {
    let len = bytes.len();
    let mut pos = 0usize;

    let month_start = pos;
    while pos < len && bytes[pos].is_ascii_digit() {
        pos += 1;
        if pos - month_start > 2 {
            return None;
        }
    }
    let month_digits = pos - month_start;
    if month_digits == 0 {
        return None;
    }
    let month = match month_digits {
        1 => (bytes[month_start] - b'0') as u32,
        2 => 10 * (bytes[month_start] - b'0') as u32 + (bytes[month_start + 1] - b'0') as u32,
        _ => return None,
    };

    if !matches!(bytes.get(pos), Some(b'-' | b'/' | b'.')) {
        return None;
    }
    pos += 1;

    // Exactly four year digits keeps MM/YY out of this path.
    if len - pos != 4 {
        return None;
    }
    let mut year = 0i32;
    for &b in &bytes[pos..pos + 4] {
        if !b.is_ascii_digit() {
            return None;
        }
        year = year * 10 + (b - b'0') as i32;
    }

    ymd_first_of_month(year, month)
}

/// 6-digit year-month: "202403" or "-202403"
pub(crate) fn parse_yyyymm(s: &str) -> Option<Dt> {
    let (y_str, m_str) = if let Some(rest) = s.strip_prefix('-') {
        if rest.len() != 6 {
            return None;
        }
        (&rest[0..4], &rest[4..6])
    } else {
        if s.len() != 6 {
            return None;
        }
        (&s[0..4], &s[4..6])
    };

    if let (Ok(mut y), Ok(m)) = (y_str.parse::<i32>(), m_str.parse::<u32>()) {
        if s.starts_with('-') {
            y = -y;
        }
        if (1..=12).contains(&m) && (crate::MIN_YEAR..=crate::MAX_YEAR).contains(&y) {
            let parsed =
                Parts::from_str("%Y%m", s.trim_start_matches('-'), true, true, true).ok()?;
            return parsed.to_dt().ok();
        }
    }
    None
}
