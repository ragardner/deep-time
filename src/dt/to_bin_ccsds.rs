use crate::{Dt, DtErr, DtErrKind, SEC_PER_DAYI64, Scale, an_err};

impl Dt {
    /// Maximum size needed for a CCSDS C & D (CUC) binary packet (with extended P-field).
    pub const CCSDS_C_AND_D_MAX_SIZE: usize = 32;

    /// Formats this [`Dt`] as a **CCSDS C (CUC)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`from_ccsds_c`].
    /// Conforms to **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support for the
    /// extended P-field (second octet) when `n_coarse > 4` or `n_frac > 3`.
    ///
    /// ## Parameters
    ///
    /// - `n_coarse`: 1–7 (number of coarse-time octets)
    /// - `n_frac`:   0–10 (number of fractional octets)
    /// - `extension`: advisory flag (ignored when larger sizes force the second octet)
    pub fn to_ccsds_c(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if !(1..=7).contains(&n_coarse) {
            return Err(an_err!(DtErrKind::OutOfRange, "coarse: {}", n_coarse));
        } else if n_frac > 10 {
            return Err(an_err!(DtErrKind::OutOfRange, "frac: {}", n_frac));
        }

        const EPOCH_OFFSET: i64 = 1_325_419_167;

        let rem_attos = self.to_sec_ufrac();
        let total_tai_seconds = self.to_sec64().saturating_add(EPOCH_OFFSET);

        let frac_scaled = if n_frac == 0 {
            0u128
        } else {
            let scale = 1u128 << (8 * n_frac as u32);
            // Use the positive remainder (old behavior)
            ((rem_attos as u128) * scale + 500_000_000_000_000_000) / 1_000_000_000_000_000_000
        };

        let mut buf = [0u8; Self::CCSDS_C_AND_D_MAX_SIZE];
        let mut pos = 0usize;

        // Decide whether extension byte is needed
        let needs_extension = n_coarse > 4 || n_frac > 3 || extension;

        // Base values for Octet 1
        let base_coarse = if n_coarse <= 4 { n_coarse - 1 } else { 3 };
        let base_frac = if n_frac <= 3 { n_frac } else { 3 };

        // Build P-field Octet 1
        let mut p1 = 0b0001_0000u8; // Code ID = 001
        p1 |= (base_coarse << 2) & 0b0000_1100;
        p1 |= base_frac & 0b0000_0011;
        if needs_extension {
            p1 |= 0b1000_0000;
        }
        buf[pos] = p1;
        pos += 1;

        if needs_extension {
            // Build P-field Octet 2
            let add_coarse = n_coarse.saturating_sub(4); // 0–3
            let add_frac = n_frac.saturating_sub(3); // 0–7

            let mut p2 = 0u8;
            p2 |= (add_coarse & 0b11) << 5; // spec Bits 1-2 → u8 bits
            p2 |= (add_frac & 0b111) << 2; // spec Bits 3-5 → u8 bits 4-2
            // Bit 0 (further extension) = 0
            // Bits 6-7 reserved = 0
            buf[pos] = p2;
            pos += 1;
        }

        // ── Coarse time (big-endian) ─────────────────────────────
        let coarse = total_tai_seconds as u64;
        for i in (0..n_coarse).rev() {
            buf[pos] = (coarse >> (i as u32 * 8)) as u8;
            pos += 1;
        }

        // ── Fractional time (big-endian) ─────────────────────────────
        for i in (0..n_frac).rev() {
            buf[pos] = (frac_scaled >> (i as u32 * 8)) as u8;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Formats this [`Dt`] as a **CCSDS D (CDS)** binary time code.
    ///
    /// - Fully configurable for round-tripping with [`from_ccsds_d`].
    /// - Conforms to CCSDS 301.0-B-4 §3.3 (Level 1): UTC day count + ms-of-day
    ///   since 1958-01-01 UTC.
    pub fn to_ccsds_d(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if !matches!(n_day, 2 | 3) {
            return Err(an_err!(DtErrKind::InvalidNumber, "n_day: {}", n_day));
        } else if !matches!(sub_ms_code, 0..=2) {
            return Err(an_err!(DtErrKind::InvalidItem, "sub-millisecond code"));
        }

        let utc = self.to(Scale::UTC);

        const EPOCH_OFFSET: i64 = 1_325_419_135;
        let rem_attos = utc.to_sec_ufrac();
        let total_utc_seconds = (Dt::i128_to_i64(utc.to_sec())).saturating_add(EPOCH_OFFSET);

        let day_count = (total_utc_seconds / SEC_PER_DAYI64) as u64;
        let sec_of_day = (total_utc_seconds % SEC_PER_DAYI64) as u64;

        // Round to nearest millisecond (10¹⁵ attos = 1 ms)
        let additional_ms =
            ((rem_attos as u128 + 500_000_000_000_000) / 1_000_000_000_000_000) as u64;

        let millis_of_day = sec_of_day * 1000 + additional_ms;

        // Remaining attoseconds inside the current millisecond
        let remaining_attos_in_ms = (rem_attos as u128) % 1_000_000_000_000_000;

        let frac_scaled = match sub_ms_code {
            0 => 0u64,
            1 => ((remaining_attos_in_ms * 65_536u128) / 1_000_000_000_000_000u128) as u64,
            2 => {
                const PS_SCALE: u128 = 1u128 << 32;
                ((remaining_attos_in_ms * PS_SCALE) / 1_000_000_000_000_000u128) as u64
            }
            _ => return Err(an_err!(DtErrKind::InvalidItem, "sub-millisecond code")),
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
            _ => return Err(an_err!(DtErrKind::InvalidItem, "sub-millisecond code")),
        };
        for i in (0..n_frac).rev() {
            buf[pos] = (frac_scaled >> (i * 8)) as u8;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Maximum size needed for a CCSDS CCS binary packet (P-field + T-field).
    pub const CCSDS_CCS_MAX_SIZE: usize = 14; // 1 + 2(year) + 2(date) + 3(HMS) + 6(subsec)

    /// Formats this [`Dt`] as a **CCSDS CCS (Calendar Segmented Time Code)**.
    ///
    /// Implements **CCSDS 301.0-B-4 §3.4** (Level 1 only).
    ///
    /// ## Parameters
    ///
    /// - `use_doy`: `false` = Month/Day variant (most common), `true` = Day-of-Year variant
    /// - `n_subsec`: Number of subsecond BCD octets (`0`–`6`). Each octet holds 2 decimal digits.
    ///
    /// ## Returns
    ///
    /// `(buffer, written_len)` — the P-field + T-field (big-endian BCD).
    ///
    /// ## Precision & Rounding
    ///
    /// Fractional seconds are rounded to the nearest representable value at the chosen precision
    /// (exactly as `to_ccsds_d` does for milliseconds).
    pub fn to_ccsds_ccs(
        &self,
        use_doy: bool,
        n_subsec: u8,
    ) -> Result<([u8; Self::CCSDS_CCS_MAX_SIZE], usize), DtErr> {
        if n_subsec > 6 {
            return Err(an_err!(DtErrKind::OutOfRange, "n_subsec: {}", n_subsec));
        }

        // ── Convert to UTC civil time (CCS uses the same 1958-01-01 UTC epoch as CDS) ─────
        let ymd = self.target(Scale::UTC).to_ymd();

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
        let year = ymd.yr as u32;
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
            // Day-of-Year variant (high nibble of first byte is always 0)
            let doy = ymd.day_of_yr() as u32;
            buf[pos] = bcd(doy / 100); // high byte = 00–03 (but only 0-3 used)
            buf[pos + 1] = bcd(doy % 100);
        }
        pos += 2;

        // ── Hour / Minute / Second (BCD) ──────────────────────────────────────────────
        buf[pos] = bcd(ymd.hr as u32);
        buf[pos + 1] = bcd(ymd.min as u32);
        buf[pos + 2] = bcd(ymd.sec as u32); // leap second 60 is allowed by spec
        pos += 3;

        // ── Subsecond BCD (0–12 decimal digits, 2 per byte, rounded) ──────────────────
        if n_subsec > 0 {
            let decimal_places = (2 * n_subsec) as u32;
            let scale = 10u128.pow(decimal_places);

            // Round attos to nearest representable value at this precision
            let frac_scaled =
                (ymd.attos as u128 * scale + 500_000_000_000_000_000) / 1_000_000_000_000_000_000;

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

    /// Convenience method that automatically selects the most appropriate
    /// CCSDS binary time code based on `current` [`Scale`].
    ///
    /// - If the `current` [`Scale`] **uses leap seconds** then **ccsds_d is chosen**.
    /// - Otherwise ccsds_c is chosen.
    #[inline]
    pub fn to_ccsds_bin(&self) -> Result<([u8; Self::CCSDS_C_AND_D_MAX_SIZE], usize), DtErr> {
        if self.target.uses_leap_seconds() {
            self.to_ccsds_d(2, 1, false)
        } else {
            self.to_ccsds_c(4, 4, false)
        }
    }
}
