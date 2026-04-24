use crate::{TimeParts, DtErrKind, DtError};

impl TimeParts {
    /// Generalized CCSDS ASCII Time Code parser (A or B variant).
    /// Handles both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// All time components after the date portion are optional.
    pub fn parse_ccsds(input: &str) -> Result<Self, DtError> {
        let cleaned = input.trim_end_matches(|c: char| c.to_ascii_uppercase() == 'Z');
        let bytes = cleaned.as_bytes();
        let len_ = bytes.len();

        let mut fmt_buf: [u8; 64] = [0; 64];
        let mut fmt_len: usize = 0;
        let mut pos: usize = 0;

        // Year (exactly 4 digits)
        if pos + 4 > len_ || !bytes[pos..pos + 4].iter().all(|&b| b.is_ascii_digit()) {
            return Err(DtError::new(DtErrKind::CCSDSStrNoYear));
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
                return Err(DtError::new(DtErrKind::CCSDSStrInvalidMonth));
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
                return Err(DtError::new(DtErrKind::CCSDSStrInvalidDay));
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
                return Err(DtError::new(
                    DtErrKind::CCSDSStrInvalidRequiredTimeSeparator,
                ));
            }
        }

        // Optional time sections – %H [: %M [: %S [.%.f]]]

        // %H
        if pos + 2 <= len_ {
            if !bytes[pos..pos + 2].iter().all(|&b| b.is_ascii_digit()) {
                return Err(DtError::new(DtErrKind::CCSDSStrInvalidHour));
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
                return Err(DtError::new(DtErrKind::CCSDSStrInvalidMinute));
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
                return Err(DtError::new(DtErrKind::CCSDSStrInvalidSecond));
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
            Err(_) => return Err(DtError::new(DtErrKind::CCSDSStrFromUtf8Err)),
        };

        TimeParts::strptime(format, cleaned, false, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Small helper for tests (strptime already calls .finish() internally on full consumption)
    fn parse(s: &str) -> TimeParts {
        let x = TimeParts::parse_ccsds(s);
        match x {
            Ok(x) => {
                return x;
            }
            Err(_) => {
                panic!("parse_ccsds should succeed on valid CCSDS input")
            }
        }
    }

    #[test]
    fn test_ccsds_calendar_variants() {
        // Full calendar with fractional seconds + trailing Z
        let dt = parse("2024-04-18T14:30:25.123456789Z");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.day_of_year, None);
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(25));
        assert!(dt.attos.is_some()); // fractional seconds parsed
        assert!(!dt.is_leap_second);

        // Calendar with seconds, no fraction
        let dt = parse("2024-04-18T14:30:25");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(25));
        assert!(dt.attos.is_some()); // defaults to 0

        // Calendar with only minutes
        let dt = parse("2024-04-18T14:30");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(0));

        // Calendar with only hour
        let dt = parse("2024-04-18T14");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(0));
        assert_eq!(dt.second, Some(0));

        // Calendar date-only
        let dt = parse("2024-04-18");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.day_of_year, None);
        assert_eq!(dt.hour, Some(0));
        assert_eq!(dt.minute, Some(0));
        assert_eq!(dt.second, Some(0));
    }

    #[test]
    fn test_ccsds_doy_variants() {
        // DOY with fractional seconds + Z
        let dt = parse("2024-109T14:30:25.5Z");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.day_of_year, Some(109));
        assert_eq!(dt.month, None);
        assert_eq!(dt.day, None);
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(25));
        assert!(dt.attos.is_some());

        // DOY date-only
        let dt = parse("2024-001");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.day_of_year, Some(1));
        assert_eq!(dt.month, None);
        assert_eq!(dt.day, None);

        // DOY with seconds only (no fraction)
        let dt = parse("2024-366T23:59:59");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.day_of_year, Some(366));
        assert_eq!(dt.hour, Some(23));
        assert_eq!(dt.minute, Some(59));
        assert_eq!(dt.second, Some(59));
    }

    #[test]
    fn test_ccsds_separators_and_z() {
        // Space instead of T
        let dt = parse("2024-04-18 14:30:25");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(25));

        // Lowercase t
        let dt = parse("2024-109t14:30");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.day_of_year, Some(109));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));

        // Trailing Z (case-insensitive) is stripped and still works
        let dt = parse("2024-04-18T14:30:25Z");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(4));
        assert_eq!(dt.day, Some(18));
        assert_eq!(dt.hour, Some(14));
        assert_eq!(dt.minute, Some(30));
        assert_eq!(dt.second, Some(25));
    }

    #[test]
    fn test_ccsds_fractional_seconds_various_lengths() {
        // 1 digit
        let dt = parse("2024-04-18T14:30:25.1");
        assert!(dt.attos.is_some());

        // 3 digits
        let dt = parse("2024-04-18T14:30:25.123");
        assert!(dt.attos.is_some());

        // 6 digits
        let dt = parse("2024-04-18T14:30:25.123456");
        assert!(dt.attos.is_some());

        // 9 digits (full nanos)
        let dt = parse("2024-04-18T14:30:25.123456789");
        assert!(dt.attos.is_some());
    }

    #[test]
    fn test_ccsds_leap_second() {
        let dt = parse("2024-06-30T23:59:60Z");
        assert_eq!(dt.year, Some(2024));
        assert_eq!(dt.month, Some(6));
        assert_eq!(dt.day, Some(30));
        assert_eq!(dt.second, Some(60));
        assert!(dt.is_leap_second);
    }

    #[test]
    fn test_ccsds_doy_vs_calendar_detection() {
        // Must be detected as DOY (exactly 3 digits after year separator, next char is not a digit)
        let doy = parse("2024-123T12:00:00");
        assert_eq!(doy.day_of_year, Some(123));
        assert_eq!(doy.month, None);
        assert_eq!(doy.day, None);

        // Must be detected as calendar date
        let cal = parse("2024-12-03T12:00:00");
        assert_eq!(cal.month, Some(12));
        assert_eq!(cal.day, Some(3));
        assert_eq!(cal.day_of_year, None);
    }
}
