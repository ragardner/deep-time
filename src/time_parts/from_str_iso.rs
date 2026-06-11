use crate::{DtErr, DtErrKind, Offset, Scale, TimeParts, an_err};

impl TimeParts {
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
    pub fn from_str_iso(input: &str) -> Result<Self, DtErr> {
        let bytes = input.as_bytes();
        let len_ = bytes.len();

        let mut start = 0usize;
        while start < len_ {
            let b = bytes[start];
            if b.is_ascii_digit()
                || (matches!(b, b'+' | b'-')
                    && start + 1 < len_
                    && bytes[start + 1].is_ascii_digit())
            {
                break;
            }
            start += 1;
        }

        if start == len_ {
            return Err(an_err!(
                DtErrKind::ExpectedYear,
                "year start (digit or +/- and digit)"
            ));
        }

        let input = &input[start..];
        let bytes = input.as_bytes();
        let len_ = bytes.len();
        let mut pos: usize = 0;
        let mut tp = TimeParts::new_utc();

        // Year (manual accumulation, optional sign)
        let mut year: i64 = 0;
        let negative_year = pos < len_ && bytes[pos] == b'-';

        if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
            pos += 1;
        }

        let mut has_year_digit = false;
        while pos < len_ && bytes[pos].is_ascii_digit() {
            has_year_digit = true;
            year = year * 10 + (bytes[pos] - b'0') as i64;
            pos += 1;
        }
        if !has_year_digit {
            return Err(an_err!(
                DtErrKind::ExpectedYear,
                "year (digits after optional sign)"
            ));
        }
        if negative_year {
            year = -year;
        }
        tp.yr = Some(year);

        // Optional separator after year (consume only if present)
        if pos < len_ && !bytes[pos].is_ascii_digit() {
            pos += 1;
        }

        // DOY vs calendar detection, uses required datetime separator to detect
        let is_doy = pos + 3 == len_ || (pos + 3 < len_ && !bytes[pos + 3].is_ascii_digit());

        if is_doy {
            // 3-digit day of year
            if pos + 3 > len_ || !bytes[pos..pos + 3].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedDayOfYear, "3-digit day of year"));
            }
            let mut doy: u16 = 0;
            for _ in 0..3 {
                doy = doy * 10 + (bytes[pos] - b'0') as u16;
                pos += 1;
            }
            tp.day_of_yr = Some(doy);
        } else {
            // 2-digit month
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedMonth, "2-digit month"));
            }
            let mut mo: u8 = 0;
            for _ in 0..2 {
                mo = mo * 10 + (bytes[pos] - b'0');
                pos += 1;
            }
            tp.mo = Some(mo);

            // Optional separator after month
            if pos < len_ && !bytes[pos].is_ascii_digit() {
                pos += 1;
            }

            // 2-digit day
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedDay, "2-digit day"));
            }
            let mut day: u8 = 0;
            for _ in 0..2 {
                day = day * 10 + (bytes[pos] - b'0');
                pos += 1;
            }
            tp.day = Some(day);
        }

        // date-time separator
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < len_ {
            let c = bytes[pos];
            // push past a T
            if !c.is_ascii_digit() {
                if pos + 1 < len_ && !matches!(c, b'+' | b'-') {
                    if bytes[pos + 1].is_ascii_digit() {
                        pos += 1;
                    } else if bytes[pos + 1].is_ascii_whitespace() {
                        pos += 1;
                        while pos < len_ && bytes[pos].is_ascii_whitespace() {
                            pos += 1;
                        }
                    }
                }
            }
        }

        // Optional time components
        if pos < len_ && bytes[pos].is_ascii_digit() {
            // Hour (2 digits)
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedHour, "2-digit hour"));
            }
            let mut hr: u8 = 0;
            for _ in 0..2 {
                hr = hr * 10 + (bytes[pos] - b'0');
                pos += 1;
            }
            tp.hr = hr;

            if pos < len_ && !bytes[pos].is_ascii_digit() {
                pos += 1;
            }

            // Minute (2 digits, if present)
            if pos + 2 <= len_ {
                if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                    return Err(an_err!(DtErrKind::ExpectedMinute, "2-digit minute"));
                }
                let mut min: u8 = 0;
                for _ in 0..2 {
                    min = min * 10 + (bytes[pos] - b'0');
                    pos += 1;
                }
                tp.min = min;
            }

            if pos < len_ && !bytes[pos].is_ascii_digit() {
                pos += 1;
            }

            // Second (2 digits, if present)
            if pos + 2 <= len_ {
                if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                    return Err(an_err!(DtErrKind::ExpectedSecond, "2-digit second"));
                }
                let mut sec: u8 = 0;
                for _ in 0..2 {
                    sec = sec * 10 + (bytes[pos] - b'0');
                    pos += 1;
                }
                tp.sec = sec;
            }

            // Fractional seconds (with or without leading dot)
            if pos < len_ {
                let has_dot = bytes[pos] == b'.';
                if has_dot {
                    pos += 1;
                }

                if pos < len_ && bytes[pos].is_ascii_digit() {
                    let mut attos: u64 = 0;
                    let mut digits_seen: usize = 0;

                    while pos < len_ && bytes[pos].is_ascii_digit() {
                        if digits_seen < 18 {
                            attos = attos * 10 + (bytes[pos] - b'0') as u64;
                            digits_seen += 1;
                        }
                        // Ignore any digits beyond the first 18
                        pos += 1;
                    }

                    if digits_seen > 0 {
                        tp.attos = attos * 10u64.pow(18u32.saturating_sub(digits_seen as u32));
                    }
                }
            }

            // Optional trailing Z/z
            if pos < len_ && matches!(bytes[pos], b'Z' | b'z') {
                pos += 1;
            }
        }

        // Skip any whitespace
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        // Optional offset
        if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
            let sign: i64 = if bytes[pos] == b'+' { 1 } else { -1 };
            pos += 1;

            // Parse hours (up to 2 digits). "+05:30"/"+0530"
            let mut hours: i64 = 0;
            let mut h_digits = 0usize;
            while pos < len_ && bytes[pos].is_ascii_digit() && h_digits < 2 {
                hours = hours
                    .saturating_mul(10)
                    .saturating_add((bytes[pos] - b'0') as i64);
                pos += 1;
                h_digits += 1;
            }

            if h_digits > 0 {
                // Optional ':' separator before minutes
                if pos < len_ && bytes[pos] == b':' {
                    pos += 1;
                }

                // Parse minutes (up to 2 digits; optional)
                let mut minutes: i64 = 0;
                let mut m_digits = 0usize;
                while pos < len_ && bytes[pos].is_ascii_digit() && m_digits < 2 {
                    minutes = minutes
                        .saturating_mul(10)
                        .saturating_add((bytes[pos] - b'0') as i64);
                    pos += 1;
                    m_digits += 1;
                }

                let total_sec_i64 = sign.saturating_mul(
                    hours
                        .saturating_mul(3600)
                        .saturating_add(minutes.saturating_mul(60)),
                );
                let total_seconds: i32 =
                    total_sec_i64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                tp.offset = Some(Offset::Fixed(total_seconds));
            }
        }

        // Skip any whitespace before IANA name or scale
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        // Optional IANA timezone name in square brackets, e.g. [America/New_York]
        // Must be explicitly wrapped in [] so we don't mistake a scale for a zone.
        if pos < len_ && bytes[pos] == b'[' {
            pos += 1; // skip '['

            let name_start = pos;

            while pos < len_ && bytes[pos] != b']' {
                pos += 1;
            }

            if pos >= len_ {
                return Err(an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "unclosed IANA tz name (missing ']')"
                ));
            }

            // pos is now at ']'
            let iana_bytes = &bytes[name_start..pos];

            let iana = core::str::from_utf8(iana_bytes).map_err(|_| {
                an_err!(
                    DtErrKind::InvalidBytes,
                    "IANA tz name contains invalid UTF-8"
                )
            })?;

            tp.set_iana_name(Some(iana));
            pos += 1; // consume ']'
        }

        // Optional trailing scale (e.g. TAI, UTC)
        if pos < len_ {
            while pos < len_ && !bytes[pos].is_ascii_alphabetic() {
                pos += 1;
            }
            if pos < len_ {
                let end = {
                    let mut i = pos;
                    while i < len_ && bytes[i].is_ascii_alphabetic() {
                        i += 1;
                        if i - pos > 8 {
                            break;
                        }
                    }
                    i
                };
                if let Some(sc) = Scale::from_abbrev(&input[pos..end]) {
                    tp.scale = sc;
                    // pos += end - pos;
                }
            }
        }

        Ok(tp)
    }
}
