use crate::error_std::DtStdError;
use crate::{
    Delta, Lang, looks_like_iso_duration, natural_duration_to_delta, natural_duration_to_iso,
    parse_iso_duration_delta,
};
use std::string::{String, ToString};

/// Parses duration strings with three clean priority tiers:
///
/// 1. Strict ISO 8601
/// 2. Common natural-language formats
/// 3. Legacy bare number, supports decimals → fractional milliseconds
///
/// Returns deep_time::Delta
pub fn parse_duration(s: &str, lang: Lang) -> Result<Delta, DtStdError> {
    if s.is_empty() {
        return Err(DtStdError::duration(
            s.to_string(),
            "Empty duration".to_string(),
            lang,
        ));
    }

    if looks_like_iso_duration(s) {
        return parse_iso_duration_delta(s).map_err(|e| {
            DtStdError::duration(
                s.to_string(),
                std::format!("Invalid ISO 8601 duration: {}", e),
                lang,
            )
        });
    }

    if let Ok(dur) = natural_duration_to_delta(s, lang, true) {
        return Ok(dur);
    }

    if let Ok(ms) = s.parse::<f64>() {
        if !ms.is_finite() {
            return Err(DtStdError::duration(
                s.to_string(),
                "Value is not a finite number".to_string(),
                lang,
            ));
        }
        let nanos = (ms * 1_000_000.0).round() as i128;
        if nanos > i64::MAX as i128 || nanos < i64::MIN as i128 {
            return Err(DtStdError::duration(
                s.to_string(),
                "Value out of range (max ~292 years)".to_string(),
                lang,
            ));
        }
        let delta = Delta::from_ns(nanos as i64);
        return Ok(delta);
    }

    Err(DtStdError::duration(
        s.to_string(),
        "Could not parse using any method".to_string(),
        lang,
    ))
}

pub fn duration_to_iso(s: &str, lang: Lang) -> Result<String, DtStdError> {
    match natural_duration_to_iso(s, lang, true) {
        Ok(iso) => Ok(iso),
        Err(e) => Err(DtStdError::simple(s.to_string(), e)),
    }
}
