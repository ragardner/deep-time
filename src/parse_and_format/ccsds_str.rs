use crate::parser::{Error, ParsedDate, strptime};

/// Parses CCSDS ASCII Time Code **A** (calendar variant):
/// `YYYY-MM-DDThh:mm:ss.ddd...Z` (optional Z terminator, variable fractional seconds).
pub fn parse_ccsds_a(input: &str) -> Result<ParsedDate, Error> {
    // Optional trailing Z (case-insensitive, per common usage in telemetry)
    let cleaned = input.trim_end_matches(|c: char| c.to_ascii_uppercase() == 'Z');
    strptime("%Y-%m-%dT%H:%M:%S.%.f", cleaned, false)
}

/// Parses CCSDS ASCII Time Code **B** (day-of-year variant):
/// `YYYY-DDDThh:mm:ss.ddd...Z`
pub fn parse_ccsds_b(input: &str) -> Result<ParsedDate, Error> {
    let cleaned = input.trim_end_matches(|c: char| c.to_ascii_uppercase() == 'Z');
    strptime("%Y-%jT%H:%M:%S.%.f", cleaned, false)
}

/// Auto-detects A or B (tries A first).
pub fn parse_ccsds(input: &str) -> Result<ParsedDate, Error> {
    parse_ccsds_a(input).or_else(|_| parse_ccsds_b(input))
}
