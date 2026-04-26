use crate::{DtErrKind, DtError, TimeParts};

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
            return Err(DtErrKind::CCSDSStrNoYear.into());
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

        // 3 digits and a sep and time or end of input -> %j
        // (deliberately does NOT check digits here so space-padded DOY is supported)
        let is_doy =
            pos + 3 == len_ || (pos + 3 < len_ && matches!(bytes[pos + 3], b' ' | b'T' | b't'));

        if is_doy {
            // DOY variant
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%j");
            fmt_len += 2;
            pos += 3;
        } else {
            // Calendar variant

            // %m
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtErrKind::CCSDSStrInvalidMonth.into());
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%m");
            fmt_len += 2;
            pos += 2;

            // Required separator after month
            if pos < len_ && !bytes[pos].is_ascii_digit() {
                fmt_buf[fmt_len] = bytes[pos];
                fmt_len += 1;
                pos += 1;
            }

            // %d
            if pos + 2 > len_ || !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtErrKind::CCSDSStrInvalidDay.into());
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%d");
            fmt_len += 2;
            pos += 2;
        }

        // Date-time separator (T, t, or space)
        // Required if time is present
        // Optional if no time is present
        if pos < len_ {
            let c = bytes[pos];
            if matches!(c, b'T' | b't' | b' ') {
                fmt_buf[fmt_len] = c;
                fmt_len += 1;
                pos += 1;
            } else {
                return Err(DtErrKind::CCSDSStrInvalidRequiredTimeSeparator.into());
            }
        }

        // Optional time sections – %H [: %M [: %S [.%.f]]]

        // %H
        if pos + 2 <= len_ {
            if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtErrKind::CCSDSStrInvalidHour.into());
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%H");
            fmt_len += 2;
            pos += 2;
        }

        // Required separator
        if pos < len_ && !bytes[pos].is_ascii_digit() {
            fmt_buf[fmt_len] = bytes[pos];
            fmt_len += 1;
            pos += 1;
        }

        // %M
        if pos + 2 <= len_ {
            if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtErrKind::CCSDSStrInvalidMinute.into());
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%M");
            fmt_len += 2;
            pos += 2;
        }

        // Required separator
        if pos < len_ && !bytes[pos].is_ascii_digit() {
            fmt_buf[fmt_len] = bytes[pos];
            fmt_len += 1;
            pos += 1;
        }

        // %S
        if pos + 2 <= len_ {
            if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtErrKind::CCSDSStrInvalidSecond.into());
            }
            fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%S");
            fmt_len += 2;
            pos += 2;
        }

        if pos < len_ {
            if bytes[pos] == b'.' {
                // dot %.f
                fmt_buf[fmt_len..fmt_len + 3].copy_from_slice(b"%.f");
                fmt_len += 3;
                pos += 1;
            } else {
                // no dot %f
                fmt_buf[fmt_len..fmt_len + 2].copy_from_slice(b"%f");
                fmt_len += 2;
            }

            while pos < len_ && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        let format = match core::str::from_utf8(&fmt_buf[0..fmt_len]) {
            Ok(f) => f,
            Err(_) => return Err(DtErrKind::CCSDSStrFromUtf8Err.into()),
        };

        TimeParts::from_str(format, cleaned, false, false, false)
    }
}
