use crate::{Dt, DtErr, Lang};
use alloc::string::String;

impl Dt {
    /// Returns this instant as a **CCSDS ASCII Time Code** (calendar variant A).
    ///
    /// Example: `"2025-04-17T14:30:45.123456789Z"`
    ///
    /// - Uses `T` separator and trailing `Z`.
    /// - Fractional seconds are trimmed (no trailing zeros, no dot if zero).
    /// - **Perfect round-trip** with `Dt::from_str_iso` / `TimeParts::from_str_iso`.
    #[inline]
    pub fn to_str_ccsds(&self) -> Result<String, DtErr> {
        self.to_str_ccsds_nf(18)
    }

    /// Same as [`to_str_ccsds`] but lets you control the maximum number of fractional digits (0–18).
    pub fn to_str_ccsds_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_in_offset(&fmt, 0, Lang::En)
    }

    /// Returns this instant as a **CCSDS ASCII Time Code B** (day-of-year variant).
    ///
    /// Example: `"2025-107T14:30:45.123456789Z"`
    #[inline]
    pub fn to_ccsds_doy_str(&self) -> Result<String, DtErr> {
        self.to_ccsds_doy_str_nf(18)
    }

    /// Same as [`to_ccsds_doy_str`] but with configurable fractional precision.
    pub fn to_ccsds_doy_str_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%jT%H:%M:%S%.{}~fZ", prec);
        self.to_str_in_offset(&fmt, 0, Lang::En)
    }
}
