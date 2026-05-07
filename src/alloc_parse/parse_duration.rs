use crate::{
    DtErrKind, DtErr, Lang, TSpan, an_err, natural_duration_to_iso, natural_duration_to_span,
};
use alloc::string::String;

impl TSpan {
    /// Parses duration strings with three clean priority tiers:
    ///
    /// 1. Strict ISO 8601
    /// 2. Common natural-language formats
    /// 3. Legacy bare number, supports decimals → fractional milliseconds
    ///
    /// Returns deep_time::TSpan
    pub fn from_str(s: &str, lang: Lang) -> Result<TSpan, DtErr> {
        if s.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        if TSpan::looks_like_iso(s) {
            return TSpan::from_iso(s).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidInput,
                    "iso: {}",
                    s => e
                )
            });
        }

        if let Ok(dur) = natural_duration_to_span(s, lang, true) {
            return Ok(dur);
        }

        if let Ok(ms) = s.parse::<f64>() {
            if !ms.is_finite() {
                return Err(an_err!(DtErrKind::OutOfRange, "{}", s));
            }
            let nanos = (ms * 1_000_000.0).round() as i128;
            let span = TSpan::from_ns(nanos);
            return Ok(span);
        }

        Err(an_err!(DtErrKind::InvalidInput, "{}", s))
    }

    pub fn natural_to_iso(s: &str, lang: Lang) -> Result<String, DtErr> {
        match natural_duration_to_iso(s, lang, true) {
            Ok(iso) => Ok(iso),
            Err(e) => Err(an_err!(
                DtErrKind::InvalidInput,
                "{}",
                s => e
            )),
        }
    }
}
