use crate::{DtErrKind, DtError, TimeParts, ez_err};

impl TimeParts {
    /// Generalized CCSDS ASCII Time Code parser (A or B variant).
    /// Handles both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// All time components after the date portion are optional.
    pub fn from_ccsds_str(input: &str) -> Result<Self, DtError> {
        let cleaned = input.trim_end_matches(|c: char| c.to_ascii_uppercase() == 'Z');
        let bytes = cleaned.as_bytes();
        let len_ = bytes.len();

        let mut fmt_buf: [u8; 64] = [0; 64];
        let mut fmt_len: usize = 0;
        let mut pos: usize = 0;

        // Year (exactly 4 digits)
        if pos + 4 > len_ || !bytes[pos..pos + 4].iter().all(|&b| b.is_ascii_digit()) {
            return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 4-digit year"));
        }
        fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%Y");
        fmt_len += 2;
        pos += 4;

        // Required separator after year
        if pos < len_ && !bytes[pos].is_ascii_digit() {
            fmt_buf[fmt_len] = bytes[pos];
            fmt_len += 1;
            pos += 1;
        }

        // DOY vs calendar date
        let is_doy =
            pos + 3 == len_ || (pos + 3 < len_ && matches!(bytes[pos + 3], b' ' | b'T' | b't'));

        if is_doy {
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%j");
            fmt_len += 2;
            pos += 3;
        } else {
            // %m
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 2-digit month"));
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
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 2-digit day"));
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%d");
            fmt_len += 2;
            pos += 2;
        }

        // Date-time separator
        if pos < len_ {
            let c = bytes[pos];
            if matches!(c, b'T' | b't' | b' ') {
                fmt_buf[fmt_len] = c;
                fmt_len += 1;
                pos += 1;
            } else {
                return Err(ez_err!(
                    DtErrKind::CCSDSInputErr,
                    "expected T/t/space separator"
                ));
            }
        }

        // Optional time: %H [: %M [: %S [.%.f]]]

        if pos + 2 <= len_ {
            if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 2-digit hour"));
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
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 2-digit minute"));
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
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "expected 2-digit second"));
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
            } else {
                fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%f");
                fmt_len += 2;
            }
            while pos < len_ && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        let format = match core::str::from_utf8(&fmt_buf[0..fmt_len]) {
            Ok(f) => f,
            Err(_) => {
                return Err(ez_err!(DtErrKind::CCSDSInputErr, "from utf8"));
            }
        };

        TimeParts::from_str(format, cleaned, false, false, false)
    }
}
