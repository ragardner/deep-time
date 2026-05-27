use crate::{DtErr, Scale, TimeParts};
use alloc::string::String;

impl TimeParts {
    /// Returns this instant as a **CCSDS ASCII Time Code** (calendar variant A).
    ///
    /// Example: `"2025-04-17T14:30:45.123456789Z"`
    ///
    /// - Uses `T` separator and trailing `Z`.
    /// - Fractional seconds are trimmed (no trailing zeros, no dot if zero).
    /// - Round-trips with `Dt::from_str_ccsds` / `TimeParts::from_str_ccsds`.
    #[inline]
    pub fn to_str_ccsds(&self, new: Scale) -> Result<String, DtErr> {
        self.to_dt()?.to_str_ccsds(self.scale, new)
    }

    /// Same as [`to_str_ccsds`] but lets you control the maximum number of fractional digits (0–18).
    pub fn to_str_ccsds_nf(&self, new: Scale, max_precision: usize) -> Result<String, DtErr> {
        self.to_dt()?
            .to_str_ccsds_nf(self.scale, new, max_precision)
    }

    /// Returns this instant as a **CCSDS ASCII Time Code B** (day-of-year variant).
    ///
    /// Example: `"2025-107T14:30:45.123456789Z"`
    #[inline]
    pub fn to_ccsds_doy_str(&self, new: Scale) -> Result<String, DtErr> {
        self.to_dt()?.to_ccsds_doy_str_nf(self.scale, new, 18)
    }

    /// Same as [`to_ccsds_doy_str`] but with configurable fractional precision.
    pub fn to_ccsds_doy_str_nf(&self, new: Scale, max_precision: usize) -> Result<String, DtErr> {
        self.to_dt()?
            .to_ccsds_doy_str_nf(self.scale, new, max_precision)
    }
}
