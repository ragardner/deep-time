use crate::{DtErr, DtErrKind, Scale, TimeParts, an_err};

impl TimeParts {
    /// Generalized CCSDS ASCII Time Code parser (A or B variant).
    /// - Handles both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// - All time components after the date portion are optional.
    /// - Optional time scale on the end is also supported.
    pub fn from_str_ccsds(input: &str) -> Result<Self, DtErr> {
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
                DtErrKind::ExpectedValue,
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
                DtErrKind::ExpectedValue,
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

        // DOY vs calendar detection
        let is_doy = pos + 3 == len_ || (pos + 3 < len_ && !bytes[pos + 3].is_ascii_digit());

        if is_doy {
            // 3-digit day of year
            if pos + 3 > len_ || !bytes[pos..pos + 3].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedValue, "3-digit day of year"));
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
                return Err(an_err!(DtErrKind::ExpectedValue, "2-digit month"));
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
                return Err(an_err!(DtErrKind::ExpectedValue, "2-digit day"));
            }
            let mut day: u8 = 0;
            for _ in 0..2 {
                day = day * 10 + (bytes[pos] - b'0');
                pos += 1;
            }
            tp.day = Some(day);
        }

        // Optional date-time separator (only consume if followed by a digit)
        if pos < len_ {
            let c = bytes[pos];
            if !c.is_ascii_digit() {
                if pos + 1 < len_ && bytes[pos + 1].is_ascii_digit() {
                    pos += 1;
                }
            } else {
                return Err(an_err!(
                    DtErrKind::InvalidSyntax,
                    "expected time separator e.g. T"
                ));
            }
        }

        // Optional time components
        if pos < len_ && bytes[pos].is_ascii_digit() {
            // Hour (2 digits)
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedValue, "2-digit hour"));
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
                    return Err(an_err!(DtErrKind::ExpectedValue, "2-digit minute"));
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
                    return Err(an_err!(DtErrKind::ExpectedValue, "2-digit second"));
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

        // Optional trailing scale (e.g. TAI, UTC)
        if pos < len_ {
            if pos < len_ && !bytes[pos].is_ascii_alphabetic() {
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
