use crate::{Dt, DtErr, Parts};

impl Dt {
    /// Parses a **CCSDS CCS (Calendar Segmented Time Code)** binary time code
    /// into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.4** (Level 1 only). Delegates to
    /// [`Parts::from_ccsds_ccs`](../civil_parts/struct.Parts.html#method.from_ccsds_ccs).
    ///
    /// ### P-field (exactly 1 byte)
    ///
    /// - Bit 7:     Extension flag → must be `0` (extensions rejected)
    /// - Bits 6-4:  Code ID = `101`
    /// - Bit 3:     Calendar type (`0` = Month/Day, `1` = Day-of-Year)
    /// - Bits 2-0:  Number of subsecond BCD octets (`0`–`6`)
    ///
    /// ### T-field (BCD, big-endian)
    ///
    /// - 2 bytes: Year (**0001–9999**)
    /// - 2 bytes: Month+Day (01–12, 01–31) **or** Day-of-Year (001–366; high nibble of
    ///   the first DOY octet is unused and should be zero)
    /// - 3 bytes: Hour (00–23), Minute (00–59), Second (00–60 for leap seconds)
    /// - 0–6 bytes: Fractional seconds (exactly 2 decimal digits per octet)
    ///
    /// ### Epoch / scale
    ///
    /// CCS is a **Gregorian calendar** code (year 1–9999), **UTC** civil time
    /// (CCSDS 301.0-B-4 §3.4 / §3.5). It does **not** use the 1958 CDS/CUC epoch.
    ///
    /// The returned [`Dt`] is converted to TAI (`scale = TAI`) with `target = UTC`.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_cuc`](../struct.Dt.html#method.from_ccsds_cuc)
    /// - [`Dt::from_ccsds_cds`](../struct.Dt.html#method.from_ccsds_cds)
    #[inline(always)]
    pub fn from_ccsds_ccs(input: &[u8]) -> Result<Dt, DtErr> {
        Parts::from_ccsds_ccs(input)?.to_dt()
    }

    /// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
    /// directly into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.2 (Level 1)**, including the extended
    /// 2-byte P-field from Issue 4. Delegates to
    /// [`Parts::from_ccsds_cuc`](../civil_parts/struct.Parts.html#method.from_ccsds_cuc).
    ///
    /// ## Supported formats (Level 1 only)
    ///
    /// - 1-byte or 2-byte P-field (further extension beyond 2 bytes is rejected).
    /// - Code ID must be `001` (1958-01-01 **TAI** epoch).
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
    ///   - Bit 7:     Further-extension flag (must be 0)
    ///   - Bits 6-5:  Additional coarse octets (0–3)
    ///   - Bits 4-2:  Additional fractional octets (0–7)
    ///   - Bits 1-0:  Reserved (ignored)
    ///
    /// ## T-field
    ///
    /// - Coarse time: seconds since **1958-01-01 00:00:00 TAI**.
    /// - Fine time: binary fraction of a second, decoded as
    ///   `floor(value × 10¹⁸ / 2^(8·n_frac))` attoseconds (inverse of the encoder’s
    ///   truncation).
    ///
    /// ## Returns
    ///
    /// A [`Dt`] with `scale = TAI` and `target = TAI`.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Empty`](../error/enum.DtErrKind.html#variant.Empty) if `input` is empty.
    /// - [`DtErrKind::InvalidCodeId`](../error/enum.DtErrKind.html#variant.InvalidCodeId) if the Code ID is not `001`.
    /// - [`DtErrKind::PFieldTooShort`](../error/enum.DtErrKind.html#variant.PFieldTooShort) if an extended P-field is indicated but missing.
    /// - [`DtErrKind::UnsupportedItem`](../error/enum.DtErrKind.html#variant.UnsupportedItem) if a third P-field octet is indicated.
    /// - [`DtErrKind::TFieldTooShort`](../error/enum.DtErrKind.html#variant.TFieldTooShort) if the T-field is shorter than declared.
    ///
    /// Errors from [`Parts::finish`](../civil_parts/struct.Parts.html#method.finish) and
    /// [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt) may also propagate.
    #[inline(always)]
    pub fn from_ccsds_cuc(input: &[u8]) -> Result<Dt, DtErr> {
        Parts::from_ccsds_cuc(input)?.to_dt()
    }

    /// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
    /// directly into a [`Dt`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.3 (Level 1)**. Delegates to
    /// [`Parts::from_ccsds_cds`](../civil_parts/struct.Parts.html#method.from_ccsds_cds).
    ///
    /// ## Supported formats (Level 1 only)
    ///
    /// - 1-byte or 2-byte P-field.
    /// - Code ID must be `100` and the Epoch bit must be `0` (1958-01-01 UTC epoch).
    /// - Day count: 2 or 3 bytes (calendar days since the epoch).
    /// - Milliseconds of day: always 4 bytes (Annex A: 0–86_399_999, or up to
    ///   86_400_999 during a positive leap second).
    /// - Sub-millisecond field (bits 1-0 of P-field) — **cascaded unit counters**:
    ///   - `00`: absent (millisecond resolution)
    ///   - `01`: 2 bytes — microsecond-of-millisecond, range **0–999** (Annex A)
    ///   - `10`: 4 bytes — picosecond-of-millisecond, range **0–999_999_999**
    ///   - `11`: rejected (reserved)
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
    /// - Day count is **calendar days** since **1958-01-01 00:00:00 UTC**.
    /// - Milliseconds of day are always present (4 bytes).
    /// - Sub-millisecond segments use unit scaling (µs × 10¹² attos, ps × 10⁶ attos),
    ///   not binary fractions of the millisecond.
    ///
    /// ## Leap-second handling
    ///
    /// When `millis_of_day` is in `[86_400_000, 86_400_999]`, civil `second` is `60`
    /// (positive leap second). Values above `86_400_999` are rejected.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] with `scale = TAI` and `target = UTC` (via
    /// [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt)).
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Empty`](../error/enum.DtErrKind.html#variant.Empty) if `input` is empty.
    /// - [`DtErrKind::PFieldTooShort`](../error/enum.DtErrKind.html#variant.PFieldTooShort) if an extended P-field is indicated but missing.
    /// - [`DtErrKind::InvalidCodeId`](../error/enum.DtErrKind.html#variant.InvalidCodeId) if the Code ID is not `100`.
    /// - [`DtErrKind::ExpectedValue`](../error/enum.DtErrKind.html#variant.ExpectedValue) if the Epoch bit is set (non–Level-1).
    /// - [`DtErrKind::InvalidSubmillisecond`](../error/enum.DtErrKind.html#variant.InvalidSubmillisecond) if the code is `0b11`, or µs/ps exceed their ranges.
    /// - [`DtErrKind::TFieldTooShort`](../error/enum.DtErrKind.html#variant.TFieldTooShort) if the T-field is shorter than declared.
    /// - [`DtErrKind::OutOfRange`](../error/enum.DtErrKind.html#variant.OutOfRange) if `millis_of_day > 86_400_999`.
    ///
    /// Errors from [`Parts::finish`](../civil_parts/struct.Parts.html#method.finish) and
    /// [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt) may also propagate.
    #[inline(always)]
    pub fn from_ccsds_cds(input: &[u8]) -> Result<Dt, DtErr> {
        Parts::from_ccsds_cds(input)?.to_dt()
    }

    /// Auto-detects and parses a CCSDS binary time code (CUC, CDS, or CCS)
    /// based on the Code ID in the first P-field byte, then returns a [`Dt`].
    ///
    /// Convenience wrapper around
    /// [`Parts::from_ccsds_bin`](../civil_parts/struct.Parts.html#method.from_ccsds_bin).
    ///
    /// Dispatches as follows:
    /// - Code ID `001` → [`Dt::from_ccsds_cuc`](../struct.Dt.html#method.from_ccsds_cuc)
    /// - Code ID `100` → [`Dt::from_ccsds_cds`](../struct.Dt.html#method.from_ccsds_cds)
    /// - Code ID `101` → [`Dt::from_ccsds_ccs`](../struct.Dt.html#method.from_ccsds_ccs)
    ///
    /// Prefer a specific `from_ccsds_*` when the format is known.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] whose `scale` / `target` follow the decoded format after
    /// [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt):
    /// - CUC (`001`): `scale = TAI`, `target = TAI`
    /// - CDS (`100`): `scale = TAI`, `target = UTC`
    /// - CCS (`101`): `scale = TAI`, `target = UTC`
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Empty`](../error/enum.DtErrKind.html#variant.Empty) if `input` is empty.
    /// - [`DtErrKind::InvalidCodeId`](../error/enum.DtErrKind.html#variant.InvalidCodeId) if the Code ID is not `001`, `100`, or `101`.
    ///
    /// Any error from the dispatched parser is also propagated.
    #[inline(always)]
    pub fn from_ccsds_bin(input: &[u8]) -> Result<Dt, DtErr> {
        Parts::from_ccsds_bin(input)?.to_dt()
    }
}
