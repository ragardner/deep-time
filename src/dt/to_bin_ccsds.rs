use crate::{Dt, DtErr, DtErrKind, SEC_PER_DAYI64, Scale, an_err};

impl Dt {
    /// Maximum size needed for a CCSDS C & D (CUC) binary packet (with extended P-field).
    pub const CCSDS_C_AND_D_MAX_SIZE: usize = 32;

    /// Formats this [`Dt`] as a **CCSDS C (CUC – Unsegmented Time Code)** binary packet.
    ///
    /// Fully configurable for round-tripping with [`from_ccsds_c`](Self::from_ccsds_c).
    /// Conforms to **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support for the
    /// extended 2-byte P-field.
    ///
    /// The time is always encoded on the **TAI** timescale (Code ID `001`).
    /// The `target` field of this [`Dt`] is ignored.
    ///
    /// ## Parameters
    ///
    /// - `n_coarse`: Number of coarse-time octets. Must be in `1..=7`.
    /// - `n_frac`: Number of fractional-time octets. Must be in `0..=10`.
    /// - `extension`: If `true`, forces the second P-field octet even when not
    ///   strictly required by the field sizes.
    ///
    /// ## Epoch
    ///
    /// Seconds are counted since **1958-01-01 00:00:00 TAI**.
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`CCSDS_C_AND_D_MAX_SIZE`] and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::OutOfRange`] if `n_coarse` is not in `1..=7`, `n_frac > 10`,
    ///   or if this instant is before **1958-01-01 00:00:00 TAI** (the CUC epoch).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_c`](Self::from_ccsds_c)
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

        let tai_since_1958 = self
            .target(Scale::TAI)
            .to_scale_and_diff(Self::CCSDS_EPOCH, false);

        let rem_attos = tai_since_1958.to_sec_ufrac();
        let total_tai_seconds = tai_since_1958.to_sec64();

        if total_tai_seconds < 0 {
            return Err(an_err!(
                DtErrKind::OutOfRange,
                "time before 1958-01-01 TAI (CUC epoch)"
            ));
        }

        // Convert positive attosecond remainder to the requested fractional
        // binary representation. The +0.5 term implements round-to-nearest.
        let frac_scaled = if n_frac == 0 {
            0u128
        } else {
            let scale = 1u128 << (8 * n_frac as u32);
            ((rem_attos as u128) * scale + 500_000_000_000_000_000) / 1_000_000_000_000_000_000
        };

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
            let add_coarse = n_coarse.saturating_sub(4);
            let add_frac = n_frac.saturating_sub(3);

            let mut p2 = 0u8;
            p2 |= (add_coarse & 0b11) << 5;
            p2 |= (add_frac & 0b111) << 2;
            buf[pos] = p2;
            pos += 1;
        }

        // Write coarse time (big-endian)
        let coarse = total_tai_seconds as u64;
        for i in (0..n_coarse).rev() {
            buf[pos] = (coarse >> (i as u32 * 8)) as u8;
            pos += 1;
        }

        // Write fractional time (big-endian)
        for i in (0..n_frac).rev() {
            buf[pos] = (frac_scaled >> (i as u32 * 8)) as u8;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Formats this [`Dt`] as a **CCSDS D (CDS – Day Segmented Time Code)** binary packet.
    ///
    /// Fully configurable for round-tripping with [`from_ccsds_d`](Self::from_ccsds_d).
    /// Conforms to **CCSDS 301.0-B-4 §3.3 (Level 1)**.
    ///
    /// The time is always encoded on the **UTC** timescale (day count + milliseconds
    /// since midnight UTC). Leap-second handling follows the library’s conversion rules.
    ///
    /// ## Parameters
    ///
    /// - `n_day`: Number of day-count octets. Must be `2` or `3`.
    /// - `sub_ms_code`: Sub-millisecond resolution:
    ///   - `0`: none
    ///   - `1`: 2 bytes (microseconds within the millisecond)
    ///   - `2`: 4 bytes (fraction of a millisecond as 2⁻³²)
    /// - `extension`: If `true`, emits the second P-field octet.
    ///
    /// ## Epoch
    ///
    /// Day count is days since **1958-01-01 00:00:00 UTC**.
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`CCSDS_C_AND_D_MAX_SIZE`] and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::InvalidNumber`] if `n_day` is not `2` or `3`.
    /// - [`DtErrKind::InvalidItem`] if `sub_ms_code` is not in `0..=2`.
    /// - [`DtErrKind::OutOfRange`] if this instant is before **1958-01-01 00:00:00 UTC**
    ///   (the CDS Level 1 epoch).
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_d`](Self::from_ccsds_d)
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

        let utc_since_1958 = self
            .target(Scale::UTC)
            .to_scale_and_diff(Self::CCSDS_EPOCH, false);

        let rem_attos = utc_since_1958.to_sec_ufrac();
        let total_utc_seconds = utc_since_1958.to_sec64();

        if total_utc_seconds < 0 {
            return Err(an_err!(
                DtErrKind::OutOfRange,
                "time before 1958-01-01 UTC (CDS epoch)"
            ));
        }

        let day_count = (total_utc_seconds / SEC_PER_DAYI64) as u64;
        let sec_of_day = (total_utc_seconds % SEC_PER_DAYI64) as u64;

        // Round to nearest millisecond
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
    /// Fully configurable for round-tripping with [`from_ccsds_ccs`](Self::from_ccsds_ccs).
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
    /// ## Year Range
    ///
    /// The year must be in the range **1 to 9999** (as defined by the CCSDS standard).
    ///
    /// ## Returns
    ///
    /// `(buffer, len)` where `buffer` is a fixed-size array of length
    /// [`CCSDS_CCS_MAX_SIZE`] and `len` is the number of bytes written.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::OutOfRange`] if `n_subsec > 6` or if the year is outside `1..=9999`.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ccsds_ccs`](Self::from_ccsds_ccs)
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

        let year = ymd.yr;
        if !(1..=9999).contains(&year) {
            return Err(an_err!(DtErrKind::OutOfRange, "year: {}", year));
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
    /// CCSDS binary time code based on this [`Dt`]'s `target` time [`Scale`].
    ///
    /// - If the `target` [`Scale`] **uses leap seconds** then **ccsds_d is chosen**.
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
