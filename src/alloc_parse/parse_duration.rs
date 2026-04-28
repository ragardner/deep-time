use crate::error_alloc::DtAllocError;
use crate::{
    Lang, TimeSpan, looks_like_iso_duration, natural_duration_to_iso, natural_duration_to_span,
    parse_iso_duration_span,
};
use alloc::string::{String, ToString};

impl TimeSpan {
    /// Parses duration strings with three clean priority tiers:
    ///
    /// 1. Strict ISO 8601
    /// 2. Common natural-language formats
    /// 3. Legacy bare number, supports decimals → fractional milliseconds
    ///
    /// Returns deep_time::TimeSpan
    pub fn from_str(s: &str, lang: Lang) -> Result<TimeSpan, DtAllocError> {
        if s.is_empty() {
            return Err(DtAllocError::duration(
                s.to_string(),
                "Empty duration".to_string(),
                lang,
            ));
        }

        if looks_like_iso_duration(s) {
            return parse_iso_duration_span(s).map_err(|e| {
                DtAllocError::duration(
                    s.to_string(),
                    alloc::format!("Invalid ISO 8601 duration: {}", e),
                    lang,
                )
            });
        }

        if let Ok(dur) = natural_duration_to_span(s, lang, true) {
            return Ok(dur);
        }

        if let Ok(ms) = s.parse::<f64>() {
            if !ms.is_finite() {
                return Err(DtAllocError::duration(
                    s.to_string(),
                    "Value is not a finite number".to_string(),
                    lang,
                ));
            }
            let nanos = (ms * 1_000_000.0).round() as i128;
            if nanos > i64::MAX as i128 || nanos < i64::MIN as i128 {
                return Err(DtAllocError::duration(
                    s.to_string(),
                    "Value out of range (max ~292 years)".to_string(),
                    lang,
                ));
            }
            let span = TimeSpan::from_ns(nanos as i64);
            return Ok(span);
        }

        Err(DtAllocError::duration(
            s.to_string(),
            "Could not parse using any method".to_string(),
            lang,
        ))
    }

    pub fn natural_to_iso(s: &str, lang: Lang) -> Result<String, DtAllocError> {
        match natural_duration_to_iso(s, lang, true) {
            Ok(iso) => Ok(iso),
            Err(e) => Err(DtAllocError::simple(s.to_string(), e)),
        }
    }
}
