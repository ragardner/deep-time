use crate::{Dt, DtErr, DtErrKind, Lang, Scale, an_err, natural_duration_to_iso};
use alloc::string::String;

impl Dt {
    /// Parses duration strings, tries formats in the following order:
    ///
    /// 1. Strict ISO 8601 e.g. **`P1DT2H30M`**
    /// 2. Common natural-language formats e.g. **`2 wks, 3 days, and 2 mins`**
    /// 3. Media duration format e.g. **`1:07:54:30`**
    /// 4. Numerical milliseconds, decimals counted as fractional milliseconds
    ///
    /// Returns a [`Dt`].
    pub fn from_str_duration(s: &str, lang: Lang) -> Result<Dt, DtErr> {
        if s.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        if Dt::looks_like_iso(s) {
            return Dt::from_iso_duration(s).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidInput,
                    "iso: {}",
                    s => e
                )
            });
        }

        let lower = s.to_lowercase();
        if let Ok(dur) = Dt::from_natural_duration(&lower, lang, true) {
            return Ok(dur);
        }

        if let Ok(dur) = Dt::from_str_media_duration(s) {
            return Ok(dur);
        }

        if let Ok(ms) = s.parse::<f64>() {
            if !ms.is_finite() {
                return Err(an_err!(DtErrKind::OutOfRange, "{}", s));
            }
            let nanos = (ms * 1_000_000.0).round() as i128;
            let span = Dt::from_ns(nanos, Scale::TAI);
            return Ok(span);
        }

        Err(an_err!(DtErrKind::InvalidInput, "{}", s))
    }

    /// Converts a natural language duration into an ISO duration.
    pub fn natural_to_iso(s: &str, lang: Lang) -> Result<String, DtErr> {
        let lower = s.to_lowercase();
        match natural_duration_to_iso(&lower, lang, true) {
            Ok(iso) => Ok(iso),
            Err(e) => Err(an_err!(
                DtErrKind::InvalidInput,
                "{}",
                s => e
            )),
        }
    }

    /// Accepts: `P1Y`, `-P2W`, `PT1.5H`, `P1DT2H30M`, `+P3D`, `p1y`, `P1,5S`, `PT0S`, etc.
    /// Rejects: anything with whitespace, lone "P"/"-P"/"PT", "P123", "Please wait 5m",
    ///          "1.5h", "P1Yabc", "P1Y!", or **any string longer than 128 bytes**.
    fn looks_like_iso(s: &str) -> bool {
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
