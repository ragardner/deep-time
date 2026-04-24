use crate::{ClockType, TimeParts, DtErrKind, DtError, TimePoint, TimeZone};

// tests are in TimePoint to_ccsds_bin
impl TimeParts {
    /// Helper: converts days since 1958-01-01 (midnight) into Gregorian Y/M/D.
    /// Pure integer arithmetic, matches the exact CCSDS Level 1 epoch
    /// (1958-01-01 00:00:00) used by both CUC and CDS.
    pub fn days_since_1958_to_gregorian(days_since_epoch: i64) -> (i64, u8, u8) {
        let mut year = 1958i64;
        let mut remaining = days_since_epoch;

        while remaining >= 0 {
            let days_in_year = if TimePoint::is_leap_year(year) {
                366
            } else {
                365
            };
            if remaining < days_in_year {
                break;
            }
            remaining -= days_in_year;
            year += 1;
        }

        let month_days = if TimePoint::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut month = 0usize;
        let mut d = remaining as u32;
        while month < 12 {
            let days_in_month = month_days[month];
            if d < days_in_month {
                break;
            }
            d -= days_in_month;
            month += 1;
        }

        let day = d as u8 + 1;
        (year, month as u8 + 1, day)
    }

    /// Exact inverse of `days_since_1958_to_gregorian`.
    pub fn gregorian_to_days_since_1958(year: i64, month: u8, day: u8) -> i64 {
        let mut days = 0i64;
        let mut y = 1958i64;
        while y < year {
            days += if TimePoint::is_leap_year(y) { 366 } else { 365 };
            y += 1;
        }
        let month_days = if TimePoint::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        for m in 0..(month as usize - 1) {
            days += month_days[m] as i64;
        }
        days + (day as i64 - 1)
    }

    /// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
    /// directly into [`TimeParts`].
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
    /// A [`TimeParts`] with `clock_type = TAI` and `tz = Utc`.
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinTooShort`] if the input is too short for the declared P-field / T-field sizes
    ///   or otherwise malformed.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] if the Code ID is not `001`.
    /// - [`DtErrKind::CCSDSBinInvalidPFieldExtension`] if the further-extension flag is set
    ///   (3+ byte P-field, unsupported).
    pub fn parse_ccsds_c(input: &[u8]) -> Result<TimeParts, DtError> {
        if input.is_empty() {
            return Err(DtError::new(DtErrKind::CCSDSBinEmpty));
        }

        let p1 = input[0];
        let mut idx = 1usize;

        // ── Octet 1 ─────────────────────────────
        let extension = (p1 & 0b1000_0000) != 0;
        let code_id = (p1 >> 4) & 0b0111;
        if code_id != 0b001 {
            return Err(DtError::new(DtErrKind::CCSDSBinInvalidCodeId));
        }

        let base_coarse = (((p1 >> 2) & 0b0011) as usize) + 1;
        let base_frac = (p1 & 0b0011) as usize;

        // ── Octet 2 (if present) ─────────────────────────────
        let (n_coarse, n_frac) = if extension {
            if input.len() < 2 {
                return Err(DtError::new(DtErrKind::CCSDSBinTooShort));
            }
            let p2 = input[1];
            idx += 1;

            // Further extension (3+ byte P-field) is not supported
            if (p2 & 0b1000_0000) != 0 {
                return Err(DtError::new(DtErrKind::CCSDSBinInvalidPFieldExtension));
            }

            let add_coarse = ((p2 >> 5) & 0b0000_0011) as usize; // spec Bits 1-2 → u8 bits 6-5
            let add_frac = ((p2 >> 2) & 0b0000_0111) as usize; // spec Bits 3-5 → u8 bits 4-2

            (base_coarse + add_coarse, base_frac + add_frac)
        } else {
            (base_coarse, base_frac)
        };

        if n_coarse == 0 || input.len() < idx + n_coarse + n_frac {
            return Err(DtError::new(DtErrKind::CCSDSBinTooShort));
        }

        // ── Read T-field (big-endian) ─────────────────────────────────────
        let mut coarse_sec: u64 = 0; // 7 bytes = 56 bits → fits in u64
        for _ in 0..n_coarse {
            coarse_sec = (coarse_sec << 8) | u64::from(input[idx]);
            idx += 1;
        }

        let mut frac_raw: u128 = 0; // up to 10 bytes = 80 bits
        for _ in 0..n_frac {
            frac_raw = (frac_raw << 8) | u128::from(input[idx]);
            idx += 1;
        }

        // Fractional bytes → attoseconds (exact)
        let frac_attos = if n_frac == 0 {
            0
        } else {
            let denom = 1u128 << (8 * n_frac as u32);
            ((frac_raw * 1_000_000_000_000_000_000u128) / denom) as u64
        };

        // ── Exact CCSDS CUC midnight epoch conversion ─────────────────────
        let days_since_epoch = (coarse_sec / 86400) as i64;
        let sec_of_day = (coarse_sec % 86400) as i64;

        let (year, month, day) = TimeParts::days_since_1958_to_gregorian(days_since_epoch);

        let hour = (sec_of_day / 3600) as u8;
        let minute = ((sec_of_day % 3600) / 60) as u8;
        let second = (sec_of_day % 60) as u8;

        // ── Build TimeParts ──────────────────────────────────────────────
        let pd = TimeParts {
            year: Some(year),
            month: Some(month),
            day: Some(day),
            hour: Some(hour),
            minute: Some(minute),
            second: Some(second),
            attos: Some(frac_attos),
            clock_type: ClockType::TAI,
            tz: Some(TimeZone::Utc),
            ..TimeParts::default()
        };

        pd.finish(false)
    }

    /// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
    /// directly into [`TimeParts`].
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
    /// A [`TimeParts`] with `timescale = Utc` and `tz = Utc`.
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinTooShort`] if the input is too short for the declared field sizes.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] if the Code ID is not `100`.
    /// - [`DtErrKind::CCSDSBinInvalidEpoch`] if the Epoch bit is set (non-Level-1 / non-1958 epoch).
    /// - [`DtErrKind::CCSDSBinInvalidSubMillisecondCode`] if bits 6-7 encode an unsupported value (0b11).
    pub fn parse_ccsds_d(input: &[u8]) -> Result<TimeParts, DtError> {
        if input.is_empty() {
            return Err(DtError::new(DtErrKind::CCSDSBinEmpty));
        }

        let p1 = input[0];
        let mut idx = 1usize;

        // ── 1-byte vs 2-byte P-field ─────────────────────────────
        let extension = (p1 & 0b1000_0000) != 0;
        if extension {
            if input.len() < 2 {
                return Err(DtError::new(DtErrKind::CCSDSBinTooShort));
            }
            idx += 1;
        }

        // Code ID must be 100
        let code_id = (p1 >> 4) & 0b0111;
        if code_id != 0b100 {
            return Err(DtError::new(DtErrKind::CCSDSBinInvalidCodeId));
        }

        // Epoch bit (bit 4) must be 0 for Level 1
        if (p1 & 0b0000_1000) != 0 {
            return Err(DtError::new(DtErrKind::CCSDSBinInvalidEpoch));
        }

        // Day segment length (bit 5)
        let n_day = if (p1 & 0b0000_0100) == 0 { 2 } else { 3 };

        // Submillisecond resolution (bits 6-7)
        let sub_ms_code = p1 & 0b0000_0011;
        let n_subsec = match sub_ms_code {
            0b00 => 0,
            0b01 => 2,
            0b10 => 4,
            _ => return Err(DtError::new(DtErrKind::CCSDSBinInvalidSubMillisecondCode)),
        };

        if input.len() < idx + n_day + 4 + n_subsec {
            return Err(DtError::new(DtErrKind::CCSDSBinTooShort));
        }

        // ── Read T-field ─────────────────────────────────────
        let mut day_count: u64 = 0;
        for _ in 0..n_day {
            day_count = (day_count << 8) | u64::from(input[idx]);
            idx += 1;
        }

        let mut millis_of_day: u64 = 0;
        for _ in 0..4 {
            millis_of_day = (millis_of_day << 8) | u64::from(input[idx]);
            idx += 1;
        }

        let mut frac_raw: u64 = 0;
        for _ in 0..n_subsec {
            frac_raw = (frac_raw << 8) | u64::from(input[idx]);
            idx += 1;
        }

        // ── Convert to attoseconds (CORRECT 10^15 scaling) ─────────────
        let sec_of_day = millis_of_day / 1000;
        let remaining_ms = (millis_of_day % 1000) as u128;

        let sub_ms_attos = if n_subsec == 0 {
            0
        } else if sub_ms_code == 0b01 {
            // 2 bytes → fraction of 1 ms (units of 1/65536)
            (frac_raw as u128 * 1_000_000_000_000_000) / 65_536
        } else {
            // 4 bytes → fraction of 1 ms (units of 2^-32)
            (frac_raw as u128 * 1_000_000_000_000_000) / (1u128 << 32)
        };

        let frac_attos = remaining_ms * 1_000_000_000_000_000 + sub_ms_attos;

        // ── Exact CCSDS CDS midnight epoch conversion (custom Gregorian) ─────
        let days_since_epoch = day_count as i64;
        let (year, month, day) = TimeParts::days_since_1958_to_gregorian(days_since_epoch);

        let hour = (sec_of_day / 3600) as u8;
        let minute = ((sec_of_day % 3600) / 60) as u8;
        let second = (sec_of_day % 60) as u8;

        // ── Build TimeParts ──────────────────────────────────────────────
        let pd = TimeParts {
            year: Some(year),
            month: Some(month),
            day: Some(day),
            hour: Some(hour),
            minute: Some(minute),
            second: Some(second),
            attos: Some(frac_attos as u64),
            clock_type: ClockType::UTC,
            tz: Some(TimeZone::Utc),
            ..TimeParts::default()
        };

        pd.finish(false)
    }

    /// Auto-detects and parses either a CCSDS C (CUC) or D (CDS) binary time code
    /// based on the Code ID in the first P-field byte.
    ///
    /// Convenience wrapper around [`parse_ccsds_c`] and [`parse_ccsds_d`].
    ///
    /// # Errors
    /// - [`DtErrKind::CCSDSBinEmpty`] if the input is empty.
    /// - [`DtErrKind::CCSDSBinInvalidCodeId`] for any Code ID other than `001` (CUC) or `100` (CDS).
    pub fn parse_ccsds_bin(input: &[u8]) -> Result<TimeParts, DtError> {
        if input.is_empty() {
            return Err(DtError::new(DtErrKind::CCSDSBinEmpty));
        }
        let code_id = (input[0] >> 4) & 0b0111;
        match code_id {
            0b001 => Self::parse_ccsds_c(input),
            0b100 => Self::parse_ccsds_d(input),
            _ => Err(DtError::new(DtErrKind::CCSDSBinInvalidCodeId)),
        }
    }
}
