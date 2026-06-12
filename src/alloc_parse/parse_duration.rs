use crate::{
    Dt, DtErr, DtErrKind, Lang, Scale, an_err, natural_duration_to_iso, natural_duration_to_span,
};
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

        if let Ok(dur) = natural_duration_to_span(s, lang, true) {
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
