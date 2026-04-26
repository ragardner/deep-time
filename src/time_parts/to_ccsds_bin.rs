use crate::{DtError, TimeParts, TimePoint};

impl TimeParts {
    /// Formats this [`TimeParts`] as a **CCSDS C (CUC)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`from_ccsds_c`].
    /// Conforms to **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support for the
    /// extended P-field (second octet) when `n_coarse > 4` or `n_frac > 3`.
    ///
    /// # Parameters
    /// - `n_coarse`: 1–7 (number of coarse-time octets)
    /// - `n_frac`:   0–10 (number of fractional octets)
    /// - `extension`: advisory flag (ignored when larger sizes force the second octet)
    #[inline]
    pub fn to_ccsds_c(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; TimePoint::CCSDS_C_AND_D_MAX_SIZE], usize), DtError> {
        self.to_time_point(Some(self.clock_type))?
            .to_ccsds_c(n_coarse, n_frac, extension)
    }

    /// Formats this [`TimeParts`] as a **CCSDS D (CDS)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`from_ccsds_d`].
    /// Conforms to CCSDS 301.0-B-4 §3.3 (Level 1): UTC day count + ms-of-day since 1958-01-01 UTC.
    #[inline]
    pub fn to_ccsds_d(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; TimePoint::CCSDS_C_AND_D_MAX_SIZE], usize), DtError> {
        self.to_time_point(Some(self.clock_type))?
            .to_ccsds_d(n_day, sub_ms_code, extension)
    }

    /// Formats this [`TimeParts`] as a **CCSDS CCS (Calendar Segmented Time Code)**.
    ///
    /// Implements **CCSDS 301.0-B-4 §3.4** (Level 1 only).
    ///
    /// # Parameters
    /// - `use_doy`: `false` = Month/Day variant (most common), `true` = Day-of-Year variant
    /// - `n_subsec`: Number of subsecond BCD octets (`0`–`6`). Each octet holds 2 decimal digits.
    ///
    /// # Returns
    /// `(buffer, written_len)` — the P-field + T-field (big-endian BCD).
    ///
    /// # Precision & Rounding
    /// Fractional seconds are rounded to the nearest representable value at the chosen precision
    /// (exactly as `to_ccsds_d` does for milliseconds).
    #[inline]
    pub fn to_ccsds_ccs(
        &self,
        use_doy: bool,
        n_subsec: u8,
    ) -> Result<([u8; TimePoint::CCSDS_CCS_MAX_SIZE], usize), DtError> {
        self.to_time_point(Some(self.clock_type))?
            .to_ccsds_ccs(use_doy, n_subsec)
    }

    /// Convenience method that automatically selects the most appropriate
    /// CCSDS binary time code based on this `TimeParts`’s [`ClockType`].
    ///
    /// # Automatic selection (matches common mission practice)
    /// - `ClockType::TAI` → **CUC** (4 coarse + 4 fractional bytes)
    /// - Any other `ClockType` (UTC, TT, GPST, TCG, …) → converted to UTC and uses **CDS**
    ///   (2 day bytes + 4 ms bytes + 2-byte sub-ms)
    #[inline(always)]
    pub fn to_ccsds_bin(
        &self,
    ) -> Result<([u8; TimePoint::CCSDS_C_AND_D_MAX_SIZE], usize), DtError> {
        self.to_time_point(Some(self.clock_type))?.to_ccsds_bin()
    }
}
