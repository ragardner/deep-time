use crate::{DtErr, Parts};
use alloc::string::String;

impl Parts {
    /// Returns this instant as a **CCSDS ASCII Time Code A** (month/day).
    ///
    /// Converts via [`Parts::to_dt`](#method.to_dt), then
    /// [`Dt::to_str_ccsds`](../struct.Dt.html#method.to_str_ccsds)
    /// (CCSDS 301.0-B-4 §3.5.1.1).
    ///
    /// Example: `"2025-04-17T14:30:45.123456789Z"`
    #[inline]
    pub fn to_str_ccsds(&self) -> Result<String, DtErr> {
        self.to_dt()?.to_str_ccsds()
    }

    /// Same as [`Parts::to_str_ccsds`](#method.to_str_ccsds) with maximum
    /// fractional digits (`0`–`18`).
    pub fn to_str_ccsds_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        self.to_dt()?.to_str_ccsds_nf(max_precision)
    }

    /// Returns this instant as a **CCSDS ASCII Time Code B** (day-of-year).
    ///
    /// See [`Dt::to_ccsds_doy_str`](../struct.Dt.html#method.to_ccsds_doy_str)
    /// (CCSDS 301.0-B-4 §3.5.1.2).
    ///
    /// Example: `"2025-107T14:30:45.123456789Z"`
    #[inline]
    pub fn to_ccsds_doy_str(&self) -> Result<String, DtErr> {
        self.to_dt()?.to_ccsds_doy_str_nf(18)
    }

    /// Same as [`Parts::to_ccsds_doy_str`](#method.to_ccsds_doy_str) with
    /// configurable fractional precision.
    pub fn to_ccsds_doy_str_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        self.to_dt()?.to_ccsds_doy_str_nf(max_precision)
    }
}
