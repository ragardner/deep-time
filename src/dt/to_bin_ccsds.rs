use crate::{
    ATTOS_PER_MS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_U128, ATTOS_PER_US_I128, Dt, DtErr,
    DtErrKind, Scale, an_err,
};

impl Dt {
    /// Maximum size needed for a CCSDS **CUC or CDS** binary packet (with extended P-field).
    ///
    /// Sized for the largest supported CUC layout (2-byte P-field + 7 coarse + 10 fine = 19 octets)
    /// with headroom; also sufficient for CDS (≤ 2 + 3 day + 4 ms + 4 sub-ms = 13 octets).
    pub const CCSDS_C_AND_D_MAX_SIZE: usize = 32;

    /// Formats this [`Dt`] as a **CCSDS C (CUC – Unsegmented Time Code)** binary packet.
    ///
    /// Fully configurable for round-tripping with
    /// [`Dt::from_ccsds_cuc`](../struct.Dt.html#method.from_ccsds_cuc).
    /// Conforms to **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support for the
    /// extended 2-byte P-field.
    ///
    /// - The time is always encoded on the **TAI** timescale (Code ID `001`).
    /// - The `target` field of this [`Dt`] is ignored.
    ///
    /// ## Parameters
    ///
    /// - `n_coarse`: Number of bytes used for the coarse (integer) seconds since the
    ///   1958 epoch. Must be in `1..=7`. The value must be large enough to hold the
    ///   full second count; if the instant does not fit, encoding returns
    ///   [`DtErrKind::OutOfRange`] (no silent truncation). For example, a date in
    ///   2025 requires at least 4 bytes (`n_coarse >= 4`); 3 bytes only reach
    ///   roughly mid-1968.
    /// - `n_frac`: Number of bytes used for the fractional seconds. Must be in `0..=10`.
    ///   The fine field is the **binary fraction** of a second
    ///   (`floor(frac × 2^(8·n_frac))`), matching a free-running binary counter
    ///   (CCSDS 301.0-B-4 §3.2). Values are **truncated**, never rounded.
    /// - `extension`: If `true`, forces inclusion of the second P-field octet even
    ///   when it is not strictly required by the field sizes.
    ///
    /// ## Epoch
    ///
    /// Seconds are counted since **1958-01-01 00:00:00 TAI**.
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`Dt::CCSDS_C_AND_D_MAX_SIZE`](../struct.Dt.html#associatedconstant.CCSDS_C_AND_D_MAX_SIZE)
    /// and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::OutOfRange`] if `n_coarse` is not in `1..=7`, or if the
    ///   coarse second count does not fit in `n_coarse` octets.
    /// - [`DtErrKind::FracOutOfRange`] if `n_frac > 10`.
    /// - [`DtErrKind::YearOutOfRange`] if this instant is before
    ///   **1958-01-01 00:00:00 TAI** (the CUC epoch).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_cuc`](../struct.Dt.html#method.from_ccsds_cuc)
    pub fn to_ccsds_cuc(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if !(1..=7).contains(&n_coarse) {
            return Err(an_err!(DtErrKind::OutOfRange));
        } else if n_frac > 10 {
            return Err(an_err!(DtErrKind::FracOutOfRange));
        }

        let tai_since_1958 = self
            .target(Scale::TAI)
            .to_scale_and_diff(Self::CCSDS_EPOCH, false);

        let rem_attos = tai_since_1958.to_sec_ufrac();
        let total_tai_seconds = tai_since_1958.to_sec64_floor();

        if total_tai_seconds < 0 {
            return Err(an_err!(DtErrKind::YearOutOfRange, "<1958"));
        }

        let coarse = total_tai_seconds as u64;
        // Refuse silent truncation: coarse field must hold the full second count.
        let coarse_bits = 8u32 * u32::from(n_coarse);
        let max_coarse = if coarse_bits >= 64 {
            u64::MAX
        } else {
            (1u64 << coarse_bits) - 1
        };
        if coarse > max_coarse {
            return Err(an_err!(DtErrKind::OutOfRange, "n_coarse"));
        }

        // Binary-fraction fine time: state of a free-running counter
        // (CCSDS 301.0-B-4 §3.2). Pure truncation — never round.
        //
        // Build big-endian octets via successive ×256 / 10¹⁸ so that large
        // `n_frac` never needs an intermediate wider than u128.
        let mut frac_bytes = [0u8; 10];
        if n_frac > 0 {
            let mut rem = rem_attos as u128;
            for byte in frac_bytes.iter_mut().take(n_frac as usize) {
                rem *= 256;
                *byte = (rem / ATTOS_PER_SEC_U128) as u8;
                rem %= ATTOS_PER_SEC_U128;
            }
        }

        let mut buf = [0u8; Self::CCSDS_C_AND_D_MAX_SIZE];
        let mut pos = 0usize;

        // Decide whether we need the extended (2-byte) P-field.
        // Required when coarse > 4 octets, fractional > 3 octets, or caller forces it.
        let needs_extension = n_coarse > 4 || n_frac > 3 || extension;

        // Base values that fit in the first P-field octet.
        // Coarse: 2 bits (0-3) → actual octets = base + 1
        // Fractional: 2 bits (0-3)
        let base_coarse = if n_coarse <= 4 { n_coarse - 1 } else { 3 };
        let base_frac = if n_frac <= 3 { n_frac } else { 3 };

        // Build P-field octet 1
        // Bit 7     = extension flag
        // Bits 6-4  = Code ID (001 for CUC Level 1)
        // Bits 3-2  = base coarse (octets - 1)
        // Bits 1-0  = base fractional octets
        let mut p1 = 0b0001_0000u8; // Code ID = 001
        p1 |= (base_coarse << 2) & 0b0000_1100;
        p1 |= base_frac & 0b0000_0011;
        if needs_extension {
            p1 |= 0b1000_0000;
        }
        buf[pos] = p1;
        pos += 1;

        if needs_extension {
            // Build P-field octet 2 (extended P-field)
            // Bit 7     = further extension (must be 0)
            // Bits 6-5  = additional coarse octets (0-3)
            // Bits 4-2  = additional fractional octets (0-7)
            // Bits 1-0  = reserved
            let add_coarse = n_coarse - 4;
            let add_frac = n_frac.saturating_sub(3);

            let mut p2 = 0u8;
            p2 |= (add_coarse & 0b11) << 5;
            p2 |= (add_frac & 0b111) << 2;
            buf[pos] = p2;
            pos += 1;
        }

        // Write coarse time (big-endian)
        for i in (0..n_coarse).rev() {
            buf[pos] = (coarse >> (i as u32 * 8)) as u8;
            pos += 1;
        }

        // Write fractional time (big-endian, MSB already in frac_bytes[0])
        for &byte in frac_bytes.iter().take(n_frac as usize) {
            buf[pos] = byte;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Formats this [`Dt`] as a **CCSDS D (CDS – Day Segmented Time Code)** binary packet.
    ///
    /// Fully configurable for round-tripping with
    /// [`Dt::from_ccsds_cds`](../struct.Dt.html#method.from_ccsds_cds).
    /// Conforms to **CCSDS 301.0-B-4 §3.3 (Level 1)**.
    ///
    /// The time is always encoded on the **UTC** timescale (day count + milliseconds
    /// of day, with optional sub-millisecond segment). Leap-second handling follows
    /// civil UTC (`second == 60` maps into the 86_400_000 ms range of Annex A).
    ///
    /// ## Parameters
    ///
    /// - `n_day`: Number of day-count octets. Must be `2` or `3`.
    /// - `sub_ms_code`: Sub-millisecond resolution (cascaded unit counters):
    ///   - `0`: none (millisecond resolution only)
    ///   - `1`: 2 bytes — **microsecond-of-millisecond**, range **0–999** (Annex A)
    ///   - `2`: 4 bytes — **picosecond-of-millisecond**, range **0–999_999_999**
    /// - `extension`: If `true`, emits the second P-field octet.
    ///
    /// ## Segment counters (truncation)
    ///
    /// Each segment is a right-adjusted binary counter (CCSDS 301.0-B-4 §3.3).
    /// Sub-second fields are **truncated** to the segment unit — never rounded —
    /// so cascaded segments do not double-count residual time.
    ///
    /// ## Epoch
    ///
    /// Day count is calendar days since **1958-01-01 00:00:00 UTC**.
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`Dt::CCSDS_C_AND_D_MAX_SIZE`](../struct.Dt.html#associatedconstant.CCSDS_C_AND_D_MAX_SIZE)
    /// and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::InvalidNumber`] if `n_day` is not `2` or `3`.
    /// - [`DtErrKind::InvalidSubmillisecond`] if `sub_ms_code` is not in `0..=2`.
    /// - [`DtErrKind::YearOutOfRange`] if this instant is before
    ///   **1958-01-01 00:00:00 UTC** (the CDS Level 1 epoch).
    /// - [`DtErrKind::OutOfRange`] if the day count does not fit in `n_day` octets.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_cds`](../struct.Dt.html#method.from_ccsds_cds)
    pub fn to_ccsds_cds(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if !matches!(n_day, 2 | 3) {
            return Err(an_err!(DtErrKind::InvalidNumber, "n_day: {}", n_day));
        } else if !matches!(sub_ms_code, 0..=2) {
            return Err(an_err!(DtErrKind::InvalidSubmillisecond));
        }

        // Civil UTC so leap seconds map to Annex A ms-of-day ranges.
        let ymd = self.target(Scale::UTC).to_ymd();

        let day_count_i = Self::ymd_to_days_since_1958(ymd.yr, ymd.mo, ymd.day);
        if day_count_i < 0 {
            return Err(an_err!(DtErrKind::YearOutOfRange, "<1958"));
        }
        let day_count = day_count_i as u64;

        let max_day = if n_day == 2 {
            0xFFFF_u64
        } else {
            0xFF_FFFF_u64
        };
        if day_count > max_day {
            return Err(an_err!(DtErrKind::OutOfRange, "day count"));
        }

        // Cascaded counters, pure truncation (no rounding between segments).
        let attos = ymd.attos as u128;
        let sec_of_day = u64::from(ymd.hr) * 3600 + u64::from(ymd.min) * 60 + u64::from(ymd.sec);

        let ms_in_sec = (attos / ATTOS_PER_MS_I128 as u128) as u64; // 0..999
        // Civil fields already bound sec_of_day (≤ 86400 on leap second) and ms_in_sec.
        let millis_of_day = sec_of_day
            .checked_mul(1000)
            .and_then(|v| v.checked_add(ms_in_sec))
            .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "ms of day"))?;

        let attos_in_ms = attos % ATTOS_PER_MS_I128 as u128;

        let frac_scaled = match sub_ms_code {
            0 => 0u64,
            // Microsecond-of-millisecond: Annex A range 0..=999
            1 => (attos_in_ms / ATTOS_PER_US_I128 as u128) as u64,
            // Picosecond-of-millisecond: 0..=999_999_999
            2 => (attos_in_ms / ATTOS_PER_PS_I128 as u128) as u64,
            // sub_ms_code validated above
            _ => 0u64,
        };

        let mut buf = [0u8; Self::CCSDS_C_AND_D_MAX_SIZE];
        let mut pos = 0usize;

        let mut p1 = 0b0100_0000u8;
        if extension {
            p1 |= 0b1000_0000;
        }
        if n_day == 3 {
            p1 |= 0b0000_0100;
        }
        p1 |= sub_ms_code;
        buf[pos] = p1;
        pos += 1;

        if extension {
            buf[pos] = 0;
            pos += 1;
        }

        for i in (0..n_day).rev() {
            buf[pos] = (day_count >> (i * 8)) as u8;
            pos += 1;
        }

        for i in (0..4).rev() {
            buf[pos] = (millis_of_day >> (i * 8)) as u8;
            pos += 1;
        }

        let n_frac = match sub_ms_code {
            0 => 0,
            1 => 2,
            2 => 4,
            _ => 0,
        };
        for i in (0..n_frac).rev() {
            buf[pos] = (frac_scaled >> (i * 8)) as u8;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Maximum size needed for a CCSDS CCS binary packet (P-field + T-field).
    pub const CCSDS_CCS_MAX_SIZE: usize = 14; // 1 + 2(year) + 2(date) + 3(HMS) + 6(subsec)

    /// Formats this [`Dt`] as a **CCSDS CCS (Calendar Segmented Time Code)** binary packet.
    ///
    /// Fully configurable for round-tripping with
    /// [`Dt::from_ccsds_ccs`](../struct.Dt.html#method.from_ccsds_ccs).
    /// Conforms to **CCSDS 301.0-B-4 §3.4** (Level 1 only).
    ///
    /// Both CCS variants are **UTC-based** and use BCD encoding.
    /// Leap seconds are supported (`second = 60`).
    ///
    /// ## Parameters
    ///
    /// - `use_doy`: `false` = Month/Day variant (most common), `true` = Day-of-Year variant.
    /// - `n_subsec`: Number of subsecond BCD octets (`0`–`6`). Each octet holds two decimal digits
    ///   (so `n_subsec = 6` gives up to 12 decimal digits of subsecond precision).
    ///
    /// ## Subsecond digits
    ///
    /// Fractional seconds are **truncated** to the requested number of decimal digits
    /// (no rounding, no carry into the seconds field). This matches the cascaded-segment
    /// model used for CDS and avoids second-boundary overflow when the discarded
    /// residual would have rounded up.
    ///
    /// ## Year Range
    ///
    /// The year must be in the range **1 to 9999** (as defined by the CCSDS standard).
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`Dt::CCSDS_CCS_MAX_SIZE`](../struct.Dt.html#associatedconstant.CCSDS_CCS_MAX_SIZE)
    /// and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::FracOutOfRange`] if `n_subsec > 6`.
    /// - [`DtErrKind::YearOutOfRange`] if the year is outside `1..=9999`.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_ccs`](../struct.Dt.html#method.from_ccsds_ccs)
    pub fn to_ccsds_ccs(
        &self,
        use_doy: bool,
        n_subsec: u8,
    ) -> Result<([u8; Self::CCSDS_CCS_MAX_SIZE], usize), DtErr> {
        if n_subsec > 6 {
            return Err(an_err!(DtErrKind::FracOutOfRange));
        }

        // ── Convert to UTC civil time ─────────────────────────────────────────
        let ymd = self.target(Scale::UTC).to_ymd();

        let year = ymd.yr;
        if !(1..=9999).contains(&year) {
            return Err(an_err!(DtErrKind::YearOutOfRange));
        }

        let mut buf = [0u8; Self::CCSDS_CCS_MAX_SIZE];
        let mut pos = 0usize;

        // ── P-field (exactly 1 byte, no extension) ─────────────────────────────────────
        let mut p1 = 0b0101_0000u8; // bits 6-4 = 101 (Code ID)
        if use_doy {
            p1 |= 0b0000_1000; // bit 3 = 1 for DOY
        }
        p1 |= n_subsec & 0b0000_0111; // bits 2-0 = subsecond count
        buf[pos] = p1;
        pos += 1;

        // ── BCD encoder helper (2 decimal digits per byte) ─────────────────────────────
        let bcd = |val: u32| -> u8 {
            let hi = (val / 10) as u8;
            let lo = (val % 10) as u8;
            (hi << 4) | lo
        };

        // ── Year (4 BCD digits) ───────────────────────────────────────────────────────
        let year = year as u32;
        let y_hi = year / 100;
        let y_lo = year % 100;
        buf[pos] = bcd(y_hi);
        buf[pos + 1] = bcd(y_lo);
        pos += 2;

        // ── Date field (Month+Day or Day-of-Year) ─────────────────────────────────────
        if !use_doy {
            // Month/Day variant
            buf[pos] = bcd(ymd.mo as u32);
            buf[pos + 1] = bcd(ymd.day as u32);
        } else {
            // Day-of-Year variant
            let doy = ymd.day_of_yr() as u32;
            buf[pos] = bcd(doy / 100);
            buf[pos + 1] = bcd(doy % 100);
        }
        pos += 2;

        // ── Hour / Minute / Second (BCD) ──────────────────────────────────────────────
        buf[pos] = bcd(ymd.hr as u32);
        buf[pos + 1] = bcd(ymd.min as u32);
        buf[pos + 2] = bcd(ymd.sec as u32); // leap second 60 is allowed by spec
        pos += 3;

        // ── Subsecond BCD (0–12 decimal digits, 2 per byte, truncated) ─────────────────
        if n_subsec > 0 {
            let decimal_places = (2 * n_subsec) as u32;
            let scale = 10u128.pow(decimal_places);

            // Truncate attos to the requested decimal resolution (no rounding/carry).
            let frac_scaled = (ymd.attos as u128 * scale) / ATTOS_PER_SEC_U128;

            let mut remaining = frac_scaled;
            for i in (0..n_subsec).rev() {
                let pair = (remaining % 100) as u32;
                remaining /= 100;
                buf[pos + i as usize] = bcd(pair);
            }
            pos += n_subsec as usize;
        }

        Ok((buf, pos))
    }

    /// Convenience method that picks a default CCSDS binary time code from this
    /// [`Dt`]'s **`target`** [`Scale`].
    ///
    /// | Condition | Code | Defaults |
    /// |-----------|------|----------|
    /// | `target.uses_leap_seconds()` (UTC, UtcSpice, UtcHist) | **CDS** | 2-byte day, µs-of-ms (`sub_ms_code = 1`), no P-field extension |
    /// | otherwise (TAI, TT, GPS, …) | **CUC** | 4 coarse + 4 fine octets, no forced extension |
    ///
    /// For full control over field widths, call
    /// [`Dt::to_ccsds_cds`](../struct.Dt.html#method.to_ccsds_cds) or
    /// [`Dt::to_ccsds_cuc`](../struct.Dt.html#method.to_ccsds_cuc) directly.
    #[inline(always)]
    pub fn to_ccsds_bin(&self) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if self.target.uses_leap_seconds() {
            self.to_ccsds_cds(2, 1, false)
        } else {
            self.to_ccsds_cuc(4, 4, false)
        }
    }
}
