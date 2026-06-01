use crate::{DtErr, DtErrKind, Scale, TimeParts, an_err};

impl TimeParts {
    /// Generalized CCSDS ASCII Time Code parser (A or B variant).
    /// - Handles both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// - Example formats:
    ///     - 2000-01-01T12:00:00
    ///     - 2000-001T12:00:00
    /// - All time components after the date portion are optional.
    /// - Optional time scale on the end also supported.
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
        let mut fmt_buf: [u8; 128] = [0; 128];
        let mut fmt_len: usize = 0;
        let mut pos: usize = 0;

        // Year: optional sign (+/-), then ≥1 digits.
        if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
            pos += 1;
        }
        let year_start = pos;
        while pos < len_ && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        let year_len = pos - year_start;
        if year_len == 0 {
            return Err(an_err!(
                DtErrKind::ExpectedValue,
                "year (digits after optional sign)"
            ));
        }
        fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%*");
        fmt_len += 2;

        // Required separator after year
        if pos < len_ && !bytes[pos].is_ascii_digit() {
            fmt_buf[fmt_len] = bytes[pos];
            fmt_len += 1;
            pos += 1;
        }

        // DOY vs calendar date
        let is_doy = pos + 3 == len_ || pos + 3 < len_ && !bytes[pos + 3].is_ascii_digit();

        if is_doy {
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%j");
            fmt_len += 2;
            pos += 3;
        } else {
            // %m
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedValue, "2-digit month"));
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%m");
            fmt_len += 2;
            pos += 2;

            if pos < len_ && !bytes[pos].is_ascii_digit() {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
                pos += 1;
            }

            // %d
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(an_err!(DtErrKind::ExpectedValue, "2-digit day"));
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%d");
            fmt_len += 2;
            pos += 2;
        }

        // Date-time separator
        if pos < len_ {
            let c = bytes[pos];
            if !c.is_ascii_digit() {
                // perhaps time scale and end, check if char after is digit
                if pos + 1 < len_ && bytes[pos + 1].is_ascii_digit() {
                    fmt_buf[fmt_len] = c;
                    fmt_len += 1;
                    pos += 1;
                }
            } else {
                return Err(an_err!(
                    DtErrKind::InvalidSyntax,
                    "expected time separator e.g. T"
                ));
            }
        }

        // Optional time: %H [: %M [: %S [.%.f]]]
        if pos < len_ && bytes[pos].is_ascii_digit() {
            if pos + 2 <= len_ {
                if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                    return Err(an_err!(DtErrKind::ExpectedValue, "2-digit hour"));
                }
                fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%H");
                fmt_len += 2;
                pos += 2;
            }

            if pos < len_ && !bytes[pos].is_ascii_digit() {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
                pos += 1;
            }

            if pos + 2 <= len_ {
                if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                    return Err(an_err!(DtErrKind::ExpectedValue, "2-digit minute"));
                }
                fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%M");
                fmt_len += 2;
                pos += 2;
            }

            if pos < len_ && !bytes[pos].is_ascii_digit() {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
                pos += 1;
            }

            if pos + 2 <= len_ {
                if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                    return Err(an_err!(DtErrKind::ExpectedValue, "2-digit second"));
                }
                fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%S");
                fmt_len += 2;
                pos += 2;
            }

            // fractional seconds
            if pos < len_ {
                if bytes[pos] == b'.' {
                    fmt_buf[fmt_len..fmt_len + 3].copy_from_slice(b"%.f");
                    fmt_len += 3;
                    pos += 1;
                } else if bytes[pos].is_ascii_digit() {
                    fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%f");
                    fmt_len += 2;
                }
                while pos < len_ && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }

            // skip optional Z
            if pos + 1 < len_ && matches!(bytes[pos], b'Z' | b'z') {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
                pos += 1;
            }
        }

        if pos < len_ {
            // skip optional whitespace separator
            if pos < len_ && !bytes[pos].is_ascii_alphabetic() {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
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
                if Scale::from_abbrev(&input[pos..end]).is_some() {
                    fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%L");
                    fmt_len += 2;
                    pos += end - pos;
                }
            }
        }

        let format = match core::str::from_utf8(&fmt_buf[0..fmt_len]) {
            Ok(f) => f,
            Err(_) => {
                return Err(an_err!(DtErrKind::InvalidBytes, "from utf8"));
            }
        };

        TimeParts::from_str(format, &input[..pos], true, true, false)
    }
}
