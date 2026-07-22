use crate::{Dt, DtErr, Parts};

impl Parts {
    /// Formats this [`Parts`] as a **CCSDS C (CUC)** binary time code.
    ///
    /// Converts via [`Parts::to_dt`](#method.to_dt), then
    /// [`Dt::to_ccsds_cuc`](../struct.Dt.html#method.to_ccsds_cuc).
    /// See that method for full parameters, P-field layout, and error conditions
    /// (CCSDS 301.0-B-4 §3.2 Level 1, including extended P-field).
    ///
    /// ## Parameters
    ///
    /// - `n_coarse`: 1–7 (coarse-time octets, seconds since 1958-01-01 TAI; must fit)
    /// - `n_frac`:   0–10 (fine-time octets; binary fraction of a second, truncated)
    /// - `extension`: force a second P-field octet even when sizes would not require it
    #[inline]
    pub fn to_ccsds_cuc(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; Dt::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        self.to_dt()?.to_ccsds_cuc(n_coarse, n_frac, extension)
    }

    /// Formats this [`Parts`] as a **CCSDS D (CDS)** binary time code.
    ///
    /// Converts via [`Parts::to_dt`](#method.to_dt), then
    /// [`Dt::to_ccsds_cds`](../struct.Dt.html#method.to_ccsds_cds).
    /// See that method for full parameters (CCSDS 301.0-B-4 §3.3 Level 1).
    ///
    /// ## Parameters
    ///
    /// - `n_day`: `2` or `3` (day-count octets since 1958-01-01 UTC)
    /// - `sub_ms_code`: `0` = ms only; `1` = µs-of-ms (0–999); `2` = ps-of-ms (0–999_999_999)
    /// - `extension`: emit a second P-field octet (content currently zero)
    #[inline]
    pub fn to_ccsds_cds(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; Dt::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        self.to_dt()?.to_ccsds_cds(n_day, sub_ms_code, extension)
    }

    /// Formats this [`Parts`] as a **CCSDS CCS (Calendar Segmented Time Code)**.
    ///
    /// Converts via [`Parts::to_dt`](#method.to_dt), then
    /// [`Dt::to_ccsds_ccs`](../struct.Dt.html#method.to_ccsds_ccs)
    /// (CCSDS 301.0-B-4 §3.4 Level 1).
    ///
    /// ## Parameters
    ///
    /// - `use_doy`: `false` = Month/Day; `true` = Day-of-Year
    /// - `n_subsec`: subsecond BCD octets (`0`–`6`); fractional digits are **truncated**
    ///
    /// ## Returns
    ///
    /// `(buffer, written_len)` — P-field + T-field (big-endian BCD).
    #[inline]
    pub fn to_ccsds_ccs(
        &self,
        use_doy: bool,
        n_subsec: u8,
    ) -> Result<([u8; Dt::CCSDS_CCS_MAX_SIZE], usize), DtErr> {
        self.to_dt()?.to_ccsds_ccs(use_doy, n_subsec)
    }

    /// Convenience CCSDS binary encode via [`Dt::to_ccsds_bin`](../struct.Dt.html#method.to_ccsds_bin).
    ///
    /// After [`Parts::to_dt`](#method.to_dt), selection uses the resulting
    /// [`Dt`] **`target`** scale:
    ///
    /// - Leap-second scales (UTC, UtcSpice, UtcHist) → **CDS** (2-day + µs-of-ms)
    /// - Otherwise (TAI, TT, GPS, …) → **CUC** (4 + 4 octets)
    ///
    /// Note: this is **not** “anything other than TAI → CDS”; e.g. TT uses CUC.
    #[inline]
    pub fn to_ccsds_bin(&self) -> Result<([u8; Dt::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        self.to_dt()?.to_ccsds_bin()
    }
}
