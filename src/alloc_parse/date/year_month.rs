use crate::{ClockType, J2000_JD_TT, SEC_PER_DAYI64, SEC_PER_HALF_DAYI64, TimeParts, TimePoint};

/// 6-digit legacy date: YYMMDD (e.g. "240315")
#[inline(always)]
pub(crate) fn parse_yymmdd(input: &str) -> Option<TimePoint> {
    let parsed = TimeParts::from_str("%y%m%d", input, true, true, false).ok()?;
    parsed.to_time_point(Some(ClockType::UTC)).ok()
}

/// Parses year-month formats with flexible separators and optional sign:
/// "2024-03", "2024/3", "2024.03", "-2024-03", "-2024/3", "-2025.1", "+2024-05", etc.
pub(crate) fn parse_yyyy_mm(bytes: &[u8]) -> Option<TimePoint> {
    let len = bytes.len();

    // Parse optional leading sign for the year
    let (sign, mut pos) = match bytes.get(0) {
        Some(b'+') => (1i32, 1),
        Some(b'-') => (-1i32, 1),
        _ => (1i32, 0),
    };

    // Parse year digits (at least one required)
    let mut year = 0i32;
    let year_start = pos;
    while pos < len && bytes[pos].is_ascii_digit() {
        year = year * 10 + (bytes[pos] - b'0') as i32;
        pos += 1;
    }
    if pos == year_start || pos == len {
        return None;
    }

    // Must be followed by a valid separator
    if !matches!(bytes.get(pos), Some(b'-' | b'/' | b'.')) {
        return None;
    }
    pos += 1;

    // only valid cases (1 or 2 digits)
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

    if month == 0 || month > 12 {
        return None;
    }

    year *= sign;
    if year < crate::MIN_YEAR || year > crate::MAX_YEAR {
        return None;
    }

    // Build TimePoint at day 1, 00:00:00 UTC using the same J2000 logic
    let jdn = TimePoint::ymd_to_jdn(year as i64, month as u8, 1);
    let days_since_j2000 = jdn - J2000_JD_TT;
    let sec = days_since_j2000 * SEC_PER_DAYI64 - SEC_PER_HALF_DAYI64; // midnight = -12h from noon

    Some(TimePoint::new(sec, 0, ClockType::UTC))
}

/// 6-digit year-month: "202403" or "-202403"
pub(crate) fn parse_yyyymm(s: &str) -> Option<TimePoint> {
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
                TimeParts::from_str("%Y%m", s.trim_start_matches('-'), true, true, true).ok()?;
            return parsed.to_time_point(Some(ClockType::UTC)).ok();
        }
    }
    None
}
