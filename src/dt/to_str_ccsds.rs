use crate::{Dt, DtErr, Lang};
use alloc::string::String;

impl Dt {
    /// Returns this instant as a **CCSDS ASCII Time Code A** (month/day calendar).
    ///
    /// Conforms to **CCSDS 301.0-B-4 §3.5.1.1** (Level 1 subset of ISO 8601):
    ///
    /// - Example: **`"2025-04-17T14:30:45.123456789Z"`**
    /// - `T` separator and trailing `Z` (UTC / prime meridian)
    /// - Leading zeros on calendar and time subfields
    /// - Fractional seconds: up to 18 digits by default; **trailing zeros trimmed**,
    ///   and the decimal point omitted if the fraction is zero
    /// - Digits beyond the requested precision are **truncated** (not rounded)
    ///
    /// Round-trips with [`Dt::from_str`](../struct.Dt.html#method.from_str) /
    /// [`Parts::from_str`](../civil_parts/struct.Parts.html#method.from_str) for
    /// common ISO-like inputs.
    ///
    /// Uses this [`Dt`]'s `target` scale via civil formatting at offset 0.
    #[inline(always)]
    pub fn to_str_ccsds(&self) -> Result<String, DtErr> {
        self.to_str_ccsds_nf(18)
    }

    /// Same as [`to_str_ccsds`](Self::to_str_ccsds) with a maximum fractional digit
    /// count (`0`–`18`). Extra digits are truncated; trailing zeros are still trimmed.
    pub fn to_str_ccsds_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_in_offset(&fmt, 0, Lang::En)
    }

    /// Returns this instant as a **CCSDS ASCII Time Code B** (day-of-year).
    ///
    /// Conforms to **CCSDS 301.0-B-4 §3.5.1.2**.
    ///
    /// Example: **`"2025-107T14:30:45.123456789Z"`**
    ///
    /// Same fractional-digit rules as [`to_str_ccsds`](Self::to_str_ccsds).
    #[inline(always)]
    pub fn to_ccsds_doy_str(&self) -> Result<String, DtErr> {
        self.to_ccsds_doy_str_nf(18)
    }

    /// Same as [`to_ccsds_doy_str`](Self::to_ccsds_doy_str) with configurable
    /// maximum fractional precision (`0`–`18`, truncated).
    pub fn to_ccsds_doy_str_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        let fmt = alloc::format!("%Y-%jT%H:%M:%S%.{}~fZ", prec);
        self.to_str_in_offset(&fmt, 0, Lang::En)
    }
}
