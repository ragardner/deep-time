use crate::{Dt, DtErr, TimeParts};

impl Dt {
    /// Parses a **CCSDS CCS (Calendar Segmented Time Code)** binary time code
    /// into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.4** (Level 1 only).
    ///
    /// ### P-field (exactly 1 byte)
    ///
    /// - Bit 7:     Extension flag → must be `0` (we reject extensions)
    /// - Bits 6-4:  Code ID = `101`
    /// - Bit 3:     Calendar type (`0` = Month/Day, `1` = Day-of-Year)
    /// - Bits 2-0:  Number of subsecond BCD octets (`0`–`6`)
    ///
    /// ### T-field (BCD, big-endian)
    ///
    /// - 2 bytes: Year (0001–9999)
    /// - 2 bytes: Month+Day (01-12,01-31) **or** Day-of-Year (001–366)
    /// - 3 bytes: Hour (00-23), Minute (00-59), Second (00-60)
    /// - 0–6 bytes: Fractional seconds (exactly 2 decimal digits per byte)
    ///
    /// Epoch: 1958-01-01 00:00:00 **UTC** (identical to CDS).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_c`](../struct.Dt.html#method.from_ccsds_c)
    /// - [`Dt::from_ccsds_d`](../struct.Dt.html#method.from_ccsds_d)
    #[inline(always)]
    pub fn from_ccsds_ccs(input: &[u8]) -> Result<Dt, DtErr> {
        TimeParts::from_ccsds_ccs(input)?.to_dt()
    }

    /// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
    /// directly into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support
    /// for the extended 2-byte P-field defined in Issue 4.
    ///
    /// ## Supported formats (Level 1 only)
    ///
    /// - 1-byte or 2-byte P-field (further extension beyond 2 bytes is rejected).
    /// - Code ID must be `001` (1958-01-01 TAI epoch).
    /// - Coarse time: 1–7 octets total.
    /// - Fractional time: 0–10 octets total.
    ///
    /// ## P-field decoding
    ///
    /// - **First octet (P1)**:
    ///   - Bit 7:     Extension flag (1 = second P-field octet follows)
    ///   - Bits 6-4:  Code ID (must be `001`)
    ///   - Bits 3-2:  Coarse time octets minus 1 (0–3 → 1–4 octets)
    ///   - Bits 1-0:  Fractional time octets (0–3)
    ///
    /// - **Second octet (P2, when extension flag is set)**:
    ///   - Bit 7:     Further-extension flag (must be 0; 3+-byte P-fields are rejected)
    ///   - Bits 6-5:  Additional coarse octets (0–3)
    ///   - Bits 4-2:  Additional fractional octets (0–7)
    ///   - Bits 1-0:  Reserved (ignored)
    ///
    /// ## T-field
    ///
    /// - Coarse time is interpreted as seconds since **1958-01-01 00:00:00 TAI**.
    /// - Fractional time is converted to attoseconds using exact integer arithmetic
    ///   (`value × 10¹⁸ / 2^(8·n_frac)`).
    ///
    /// ## Returns
    ///
    /// A [`Dt`] with both `scale` and `target` set to [`Scale::TAI`].
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if `input` is empty.
    /// - [`DtErrKind::InvalidItem`] if the Code ID is not `001`.
    /// - [`DtErrKind::InvalidInput`] if the input is too short to contain the declared
    ///   extended P-field, or if the "further extension" flag (bit 7 of the second
    ///   P-field octet) is set.
    /// - [`DtErrKind::InvalidSyntax`] if the declared coarse + fractional field lengths
    ///   make the T-field longer than the remaining input bytes.
    ///
    /// Errors from [`TimeParts::finish`] and [`TimeParts::to_dt`] may also propagate.
    #[inline(always)]
    pub fn from_ccsds_c(input: &[u8]) -> Result<Dt, DtErr> {
        TimeParts::from_ccsds_c(input)?.to_dt()
    }

    /// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
    /// directly into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.3 (Level 1)**.
    ///
    /// ## Supported formats (Level 1 only)
    ///
    /// - 1-byte or 2-byte P-field.
    /// - Code ID must be `100` and the Epoch bit must be `0` (1958-01-01 UTC epoch).
    /// - Day count: 2 or 3 bytes.
    /// - Milliseconds since midnight: always 4 bytes.
    /// - Sub-millisecond field (bits 1-0 of P-field):
    ///   - `00`: no fractional field
    ///   - `01`: 2 bytes (microseconds within the millisecond, 0–65535)
    ///   - `10`: 4 bytes (fractional part of the millisecond as 2⁻³²)
    ///   - `11`: rejected (unsupported)
    ///
    /// ## P-field bit layout (first octet)
    ///
    /// - Bit 7:     Extension flag (1 = second P-field octet follows)
    /// - Bits 6-4:  Code ID (must be `100`)
    /// - Bit 3:     Epoch (must be `0` for Level 1 / 1958 epoch)
    /// - Bit 2:     Day count size (`0` = 2 bytes, `1` = 3 bytes)
    /// - Bits 1-0:  Sub-millisecond code (see above)
    ///
    /// ## T-field
    ///
    /// - Day count is days since **1958-01-01 00:00:00 UTC**.
    /// - Milliseconds since midnight are always present (4 bytes).
    /// - Sub-millisecond data (if present) is converted to attoseconds with
    ///   exact integer scaling.
    ///
    /// ## Leap-second handling
    ///
    /// Correctly supports leap seconds. When the millisecond-of-day value
    /// represents 23:59:60 (i.e. `millis_of_day >= 86_400_000`), `sec` is set
    /// to `60` in the resulting time.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] with `scale = TAI` and `target = UTC`.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if `input` is empty.
    /// - [`DtErrKind::InvalidInput`] if the P-field indicates an extended second
    ///   octet but the input is too short to contain it.
    /// - [`DtErrKind::InvalidItem`] if the Code ID is not `100`, the Epoch bit is
    ///   set (non-Level-1 epoch), or the sub-millisecond code is `0b11`.
    /// - [`DtErrKind::InvalidSyntax`] if the declared field lengths make the
    ///   T-field longer than the remaining input bytes.
    ///
    /// Errors from [`TimeParts::finish`] and [`TimeParts::to_dt`] may also propagate.
    #[inline(always)]
    pub fn from_ccsds_d(input: &[u8]) -> Result<Dt, DtErr> {
        TimeParts::from_ccsds_d(input)?.to_dt()
    }

    /// Auto-detects and parses a CCSDS binary time code (CUC, CDS, or CCS)
    /// based on the Code ID in the first P-field byte, then returns a [`Dt`].
    ///
    /// Convenience wrapper around [`TimeParts::from_ccsds_bin`].
    ///
    /// Dispatches as follows:
    /// - Code ID `001` → [`from_ccsds_c`](Self::from_ccsds_c) (CUC – Unsegmented)
    /// - Code ID `100` → [`from_ccsds_d`](Self::from_ccsds_d) (CDS – Day Segmented)
    /// - Code ID `101` → [`from_ccsds_ccs`](Self::from_ccsds_ccs) (CCS – Calendar Segmented)
    ///
    /// For stricter control or when the format is known in advance, prefer calling
    /// the specific `from_ccsds_*` function directly.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] whose `scale` and `target` depend on the detected format:
    /// - CUC (`001`): `scale = TAI`, `target = TAI`
    /// - CDS (`100`): `scale = TAI`, `target = UTC`
    /// - CCS (`101`): depends on the CCS parser implementation
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if `input` is empty.
    /// - [`DtErrKind::InvalidItem`] if the Code ID is not one of the three
    ///   recognized Level 1 values (`001`, `100`, or `101`).
    ///
    /// Any error returned by the dispatched parser is also propagated.
    #[inline(always)]
    pub fn from_ccsds_bin(input: &[u8]) -> Result<Dt, DtErr> {
        TimeParts::from_ccsds_bin(input)?.to_dt()
    }
}
