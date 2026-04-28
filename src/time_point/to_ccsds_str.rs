use crate::DtError;
use crate::TimePoint;
use alloc::string::String;

impl TimePoint {
    /// Returns this instant as a **CCSDS ASCII Time Code** (calendar variant A).
    ///
    /// Example: `"2025-04-17T14:30:45.123456789Z"`
    ///
    /// - Uses `T` separator and trailing `Z`.
    /// - Fractional seconds are trimmed (no trailing zeros, no dot if zero).
    /// - **Perfect round-trip** with `TimePoint::from_ccsds_str` / `TimeParts::from_ccsds_str`.
    #[inline(always)]
    pub fn to_ccsds_str(&self) -> Result<String, DtError> {
        self.to_ccsds_str_nf(18)
    }

    /// Same as [`to_ccsds_str`] but lets you control the maximum number of fractional digits (0–18).
    pub fn to_ccsds_str_nf(&self, max_precision: usize) -> Result<String, DtError> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_with_offset(&fmt, 0)
    }

    /// Returns this instant as a **CCSDS ASCII Time Code B** (day-of-year variant).
    ///
    /// Example: `"2025-107T14:30:45.123456789Z"`
    #[inline(always)]
    pub fn to_ccsds_doy_str(&self) -> Result<String, DtError> {
        self.to_ccsds_doy_str_nf(18)
    }

    /// Same as [`to_ccsds_doy_str`] but with configurable fractional precision.
    pub fn to_ccsds_doy_str_nf(&self, max_precision: usize) -> Result<String, DtError> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%jT%H:%M:%S%.{}~fZ", prec);
        self.to_str_with_offset(&fmt, 0)
    }
}
