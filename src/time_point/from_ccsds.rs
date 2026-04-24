use crate::{ClockType, DtError, TimeParts, TimePoint};

impl TimePoint {
    /// Generalized CCSDS ASCII Time Code parser (A or B variant).
    /// Handles both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// All time components after the date portion are optional.
    #[inline(always)]
    pub fn from_ccsds_str(input: &str, clock_type: ClockType) -> Result<Self, DtError> {
        Ok(TimeParts::from_ccsds_str(input)?.to_time_point(clock_type)?)
    }

    /// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
    /// directly into [`TimePoint`].
    ///
    /// This function implements **CCSDS 301.0-B-4 §3.2** (Level 1 only) **with full support
    /// for the extended P-field** (second octet) as defined in the standard.
    ///
    /// # Supported formats (Level 1 only)
    /// - 1-byte or 2-byte P-field (further extension beyond 2 bytes is rejected).
    /// - Code ID must be `001` (1958-01-01 TAI epoch).
    /// - Coarse time: 1–7 octets (base 1–4 from Octet 1 + up to 3 additional from Octet 2).
    /// - Fractional time: 0–10 octets (base 0–3 from Octet 1 + up to 7 additional from Octet 2).
    ///
    /// # P-field decoding (when Bit 0 of Octet 1 = 1)
    /// - **Octet 2**:
    ///   - Bit 0:     Further-extension flag (must be 0; we reject 3+-byte P-fields).
    ///   - Bits 1-2:  Additional coarse octets (0–3).
    ///   - Bits 3-5:  Additional fractional octets (0–7).
    ///   - Bits 6-7:  Reserved for mission definition (ignored).
    ///
    /// # Precision
    /// Fractional seconds are converted to attoseconds with **exact** integer scaling
    /// (`value / 2^(8·n_frac)`). Larger `n_frac` gives higher resolution (down to ~2⁻⁸⁰ s
    /// with 10 fractional bytes).
    ///
    /// # Returns
    /// A [`TimePoint`] with `clock_type = TAI` and `tz = Utc`.
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinTooShort`] if the input is too short for the declared P-field / T-field sizes
    ///   or otherwise malformed.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] if the Code ID is not `001`.
    /// - [`DtErrKind::CCSDSBinInvalidPFieldExtension`] if the further-extension flag is set
    ///   (3+ byte P-field, unsupported).
    #[inline(always)]
    pub fn from_ccsds_c(input: &[u8], clock_type: ClockType) -> Result<TimePoint, DtError> {
        Ok(TimeParts::from_ccsds_c(input)?.to_time_point(clock_type)?)
    }

    /// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
    /// directly into [`TimePoint`].
    ///
    /// This function implements CCSDS 301.0-B-4 §3.3 (Level 1 only).
    ///
    /// # Supported formats
    /// - 1-byte or 2-byte P-field.
    /// - Code ID must be `100` and Epoch bit must be `0` (1958-01-01 UTC epoch).
    /// - `n_day`: 2 or 3 bytes for the day count.
    /// - Middle field is always 4 bytes of **milliseconds since midnight**.
    /// - Sub-millisecond field (bits 6-7 of P-field):
    ///   - `00`: no fractional field
    ///   - `01`: 2 bytes (microseconds of the millisecond, 0–65535)
    ///   - `10`: 4 bytes (2⁻³² of the millisecond)
    ///
    /// # Precision
    /// - The millisecond field is rounded to the nearest millisecond (in the encoder).
    /// - With 2-byte sub-ms: maximum quantization error ≈ ±7.63 ns.
    /// - With 4-byte sub-ms: maximum quantization error ≈ ±0.116 ps.
    ///
    /// # Returns
    /// A [`TimePoint`] with `timescale = Utc` and `tz = Utc`.
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinTooShort`] if the input is too short for the declared field sizes.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] if the Code ID is not `100`.
    /// - [`DtErrKind::CCSDSBinInvalidEpoch`] if the Epoch bit is set (non-Level-1 / non-1958 epoch).
    /// - [`DtErrKind::CCSDSBinInvalidSubMillisecondCode`] if bits 6-7 encode an unsupported value (0b11).
    #[inline(always)]
    pub fn from_ccsds_d(input: &[u8], clock_type: ClockType) -> Result<TimePoint, DtError> {
        Ok(TimeParts::from_ccsds_d(input)?.to_time_point(clock_type)?)
    }

    /// Auto-detects and parses either a CCSDS C (CUC) or D (CDS) binary time code
    /// based on the Code ID in the first P-field byte.
    ///
    /// Convenience wrapper around [`from_ccsds_c`] and [`from_ccsds_d`].
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] for any Code ID other than `001` (CUC) or `100` (CDS).
    #[inline(always)]
    pub fn from_ccsds_bin(input: &[u8], clock_type: ClockType) -> Result<TimePoint, DtError> {
        Ok(TimeParts::from_ccsds_bin(input)?.to_time_point(clock_type)?)
    }
}
