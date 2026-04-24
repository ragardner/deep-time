use crate::{ClockType, DtErrKind, SEC_PER_DAYI64, TimePoint};

impl TimePoint {
    /// Maximum size needed for a CCSDS C (CUC) binary packet (with extended P-field).
    const CCSDS_C_MAX_SIZE: usize = 32;

    /// Formats this [`TimePoint`] as a **CCSDS C (CUC)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`parse_ccsds_c`].
    /// Conforms to **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support for the
    /// extended P-field (second octet) when `n_coarse > 4` or `n_frac > 3`.
    ///
    /// # Parameters
    /// - `n_coarse`: 1–7 (number of coarse-time octets)
    /// - `n_frac`:   0–10 (number of fractional octets)
    /// - `extension`: advisory flag (ignored when larger sizes force the second octet)
    pub fn to_ccsds_c_bin(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_MAX_SIZE], usize), DtErrKind> {
        if !(1..=7).contains(&n_coarse) || n_frac > 10 {
            return Err(DtErrKind::UnsupportedDirective);
        }

        let tai = self.to_clock_type(ClockType::TAI);

        const EPOCH_OFFSET: i64 = 1_325_419_167;
        let total_tai_seconds = tai.sec + EPOCH_OFFSET;

        let frac_scaled = if n_frac == 0 {
            0u128
        } else {
            let scale = 1u128 << (8 * n_frac as u32);
            (tai.subsec as u128 * scale + 500_000_000_000_000_000) / 1_000_000_000_000_000_000
        };

        let mut buf = [0u8; Self::CCSDS_C_MAX_SIZE];
        let mut pos = 0usize;

        // Decide whether extension byte is needed
        let needs_extension = n_coarse > 4 || n_frac > 3 || extension;

        // Base values for Octet 1
        let base_coarse = if n_coarse <= 4 { n_coarse - 1 } else { 3 };
        let base_frac = if n_frac <= 3 { n_frac } else { 3 };

        // ── Build P-field Octet 1 ─────────────────────────────
        let mut p1 = 0b0001_0000u8; // Code ID = 001
        p1 |= (base_coarse << 2) & 0b0000_1100;
        p1 |= base_frac & 0b0000_0011;
        if needs_extension {
            p1 |= 0b1000_0000;
        }
        buf[pos] = p1;
        pos += 1;

        if needs_extension {
            // ── Build P-field Octet 2 ─────────────────────────────
            let add_coarse = n_coarse.saturating_sub(4); // 0–3
            let add_frac = n_frac.saturating_sub(3); // 0–7

            let mut p2 = 0u8;
            p2 |= (add_coarse & 0b11) << 5; // spec Bits 1-2 → u8 bits 6-5
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

    /// Maximum size needed for a CCSDS D (CDS) binary packet.
    const CCSDS_D_MAX_SIZE: usize = 32;

    /// Formats this [`TimePoint`] as a **CCSDS D (CDS)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`parse_ccsds_d`].
    /// Conforms to CCSDS 301.0-B-4 §3.3 (Level 1): UTC day count + ms-of-day since 1958-01-01 UTC.
    pub fn to_ccsds_d_bin(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_D_MAX_SIZE], usize), DtErrKind> {
        if !matches!(n_day, 2 | 3) || !matches!(sub_ms_code, 0 | 1 | 2) {
            return Err(DtErrKind::UnsupportedDirective);
        }

        let utc = self.to_clock_type(ClockType::UTC);

        // UTC seconds since 1958-01-01 00:00:00 UTC (exact offset to library UTC zero,
        // accounting for all leap seconds up to the library epoch)
        const EPOCH_OFFSET: i64 = 1_325_419_135;
        let total_utc_seconds = utc.sec + EPOCH_OFFSET;

        let day_count = (total_utc_seconds / SEC_PER_DAYI64) as u64;
        let sec_of_day = (total_utc_seconds % SEC_PER_DAYI64) as u64;

        // Round to nearest millisecond (CORRECT 10^15 scaling)
        let additional_ms =
            ((utc.subsec as u128 + 500_000_000_000_000) / 1_000_000_000_000_000) as u64;
        let millis_of_day = sec_of_day * 1000 + additional_ms;

        // Remaining attoseconds inside the current millisecond
        let remaining_attos_in_ms = (utc.subsec as u128) % 1_000_000_000_000_000;

        let frac_scaled = match sub_ms_code {
            0 => 0u64,
            1 => ((remaining_attos_in_ms * 65_536u128) / 1_000_000_000_000_000u128) as u64,
            2 => {
                const PS_SCALE: u128 = 1u128 << 32;
                ((remaining_attos_in_ms * PS_SCALE) / 1_000_000_000_000_000u128) as u64
            }
            _ => unreachable!(),
        };

        let mut buf = [0u8; Self::CCSDS_D_MAX_SIZE];
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeParts;

    #[test]
    fn test_ccsds_c_direct_frac() {
        // P-field = 0x15 (binary 00010101)
        //   → 2 coarse bytes, 1 fractional byte
        // T-field = coarse 0x0001 (1 second) + frac 0x80 (0.5 seconds)
        let c_bytes = &[0x15u8, 0x00, 0x01, 0x80];

        let parsed = TimeParts::parse_ccsds_c(c_bytes).unwrap();

        assert_eq!(parsed.year, Some(1958));
        assert_eq!(parsed.month, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hour, Some(0));
        assert_eq!(parsed.minute, Some(0));
        assert_eq!(parsed.second, Some(1));
        assert!(parsed.attos.unwrap() > 499_000_000_000_000_000); // ~0.5 s
        assert_eq!(parsed.clock_type, ClockType::TAI);
    }

    #[test]
    fn test_ccsds_c_2byte_pfield() {
        // P-field = 0x90 (extension=1) + second byte 0x00
        // n_coarse=1, n_frac=0, T-field = 0x64 → 100 seconds
        let c_bytes = &[0x90u8, 0x00, 0x64];

        let parsed = TimeParts::parse_ccsds_c(c_bytes).unwrap();

        assert_eq!(parsed.year, Some(1958));
        assert_eq!(parsed.month, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hour, Some(0));
        assert_eq!(parsed.minute, Some(1));
        assert_eq!(parsed.second, Some(40));
    }

    #[test]
    fn test_ccsds_d_direct() {
        // P-field = 0x40 (n_day=2, sub_ms=00)
        // Day = 0x0000
        // Millis-of-day = 0x00000001 → 1 ms → 0 seconds + 1 ms
        let d_bytes = &[0x40u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];

        let parsed = TimeParts::parse_ccsds_d(d_bytes).unwrap();

        assert_eq!(parsed.year, Some(1958));
        assert_eq!(parsed.month, Some(1));
        assert_eq!(parsed.day, Some(1));
        assert_eq!(parsed.hour, Some(0));
        assert_eq!(parsed.minute, Some(0));
        assert_eq!(parsed.second, Some(0));
        assert_eq!(parsed.attos, Some(1_000_000_000_000_000)); // 1 ms
        assert_eq!(parsed.clock_type, ClockType::UTC);
    }

    #[test]
    fn test_ccsds_d_direct_frac() {
        // P-field = 0x41 (n_day=2, sub_ms=01 → 2 bytes)
        // Day count = 0x0000
        // Millis-of-day = 0x00000001 → 1 ms
        // Sub-ms = 0x8000 → exactly 0.5 ms
        let d_bytes = &[0x41u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00];

        let parsed = TimeParts::parse_ccsds_d(d_bytes).unwrap();

        assert_eq!(parsed.second, Some(0));
        assert_eq!(parsed.attos, Some(1_500_000_000_000_000)); // 1.5 ms
    }

    #[test]
    fn test_ccsds_c_roundtrip() {
        // Desired CCSDS C (TAI) civil time: 2025-04-17 14:30:45.123456789 TAI
        // We compute the *exact* total TAI seconds since the CCSDS epoch (1958-01-01 00:00:00 TAI)
        // using the same Gregorian logic as the parser. This is 100% independent of the library's
        // JD, leap-second table, or to_time_point path.
        let days_since_1958 = TimeParts::gregorian_to_days_since_1958(2025, 4, 17);
        let sec_of_day = (14 * 3600) + (30 * 60) + 45;
        let total_tai_seconds = days_since_1958 * SEC_PER_DAYI64 + sec_of_day;

        // Library-internal TAI representation (TAI zero = library epoch)
        const EPOCH_OFFSET: i64 = 1_325_419_167;
        let tai_sec = total_tai_seconds - EPOCH_OFFSET;

        let t = TimePoint::new(tai_sec, 123_456_789_000_000_000, ClockType::TAI);

        let (buf, len) = t.to_ccsds_c_bin(4, 3, false).unwrap();
        let parsed = TimeParts::parse_ccsds_c(&buf[0..len]).unwrap();

        assert_eq!(parsed.year, Some(2025));
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(17));
        assert_eq!(parsed.hour, Some(14));
        assert_eq!(parsed.minute, Some(30));
        assert_eq!(parsed.second, Some(45));
        assert_eq!(parsed.clock_type, ClockType::TAI);

        // 3 fractional bytes → max ~59.6 ns quantization error
        let diff = (parsed.attos.unwrap() as i64 - 123_456_789_000_000_000i64).abs();
        assert!(
            diff < 60_000_000_000,
            "Fractional error too large: {} attos",
            diff
        );
    }

    #[test]
    fn test_ccsds_d_roundtrip() {
        // Desired CCSDS D (CDS) civil time: 2025-04-17 14:30:45.000400000 UTC
        // Same pure-CCSDS-epoch calculation as above (no library conversions).
        let days_since_1958 = TimeParts::gregorian_to_days_since_1958(2025, 4, 17);
        let sec_of_day = (14 * 3600) + (30 * 60) + 45;
        let total_utc_seconds = days_since_1958 * SEC_PER_DAYI64 + sec_of_day;

        // Library-internal UTC representation
        const EPOCH_OFFSET: i64 = 1_325_419_135;
        let utc_sec = total_utc_seconds - EPOCH_OFFSET;

        let t = TimePoint::new(utc_sec, 400_000_000_000, ClockType::UTC);

        let (buf, len) = t.to_ccsds_d_bin(2, 1, false).unwrap();
        let parsed = TimeParts::parse_ccsds_d(&buf[0..len]).unwrap();

        assert_eq!(parsed.year, Some(2025));
        assert_eq!(parsed.month, Some(4));
        assert_eq!(parsed.day, Some(17));
        assert_eq!(parsed.hour, Some(14));
        assert_eq!(parsed.minute, Some(30));
        assert_eq!(parsed.second, Some(45));
        assert_eq!(parsed.clock_type, ClockType::UTC);

        let diff = (parsed.attos.unwrap() as i64 - 400_000_000_000i64).abs();
        assert!(
            diff < 16_000_000_000, // ~16 ns tolerance (2-byte sub-ms resolution)
            "Fractional error too large: {} attos",
            diff
        );
    }
}
