use crate::{ClockType, DateComponents, DtErrKind, DtError, TimePoint, TimeZone};

/// Helper: converts days since 1958-01-01 (midnight) into Gregorian Y/M/D.
/// Pure integer arithmetic, fully self-contained, matches the exact CCSDS
/// Level 1 epoch (1958-01-01 00:00:00) used by both CUC and CDS.
fn days_since_1958_to_gregorian(days_since_epoch: i64) -> (i64, u8, u8) {
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

/// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
/// directly into [`DateComponents`].
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
/// A [`DateComponents`] with `clock_type = TAI` and `tz = Utc`.
///
/// # Errors
/// - [`DtErrKind::ExpectedUnixTimestamp`] if the input is too short or malformed.
/// - [`DtErrKind::UnsupportedDirective`] for non-Level-1 packets, invalid Code ID,
///   further P-field extension, or out-of-range field sizes.
pub fn parse_ccsds_c(input: &[u8]) -> Result<DateComponents, DtError> {
    if input.is_empty() {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
    }

    let p1 = input[0];
    let mut idx = 1usize;

    // ── Octet 1 ─────────────────────────────
    let extension = (p1 & 0b1000_0000) != 0;
    let code_id = (p1 >> 4) & 0b0111;
    if code_id != 0b001 {
        return Err(DtError::new(DtErrKind::UnsupportedDirective));
    }

    let base_coarse = (((p1 >> 2) & 0b0011) as usize) + 1;
    let base_frac = (p1 & 0b0011) as usize;

    // ── Octet 2 (if present) ─────────────────────────────
    let (n_coarse, n_frac) = if extension {
        if input.len() < 2 {
            return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
        }
        let p2 = input[1];
        idx += 1;

        // Further extension (3+ byte P-field) is not supported
        if (p2 & 0b1000_0000) != 0 {
            return Err(DtError::new(DtErrKind::UnsupportedDirective));
        }

        let add_coarse = ((p2 >> 5) & 0b0000_0011) as usize; // spec Bits 1-2 → u8 bits 6-5
        let add_frac = ((p2 >> 2) & 0b0000_0111) as usize; // spec Bits 3-5 → u8 bits 4-2

        (base_coarse + add_coarse, base_frac + add_frac)
    } else {
        (base_coarse, base_frac)
    };

    if n_coarse == 0 || input.len() < idx + n_coarse + n_frac {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
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

    let (year, month, day) = days_since_1958_to_gregorian(days_since_epoch);

    let hour = (sec_of_day / 3600) as u8;
    let minute = ((sec_of_day % 3600) / 60) as u8;
    let second = (sec_of_day % 60) as u8;

    // ── Build DateComponents ──────────────────────────────────────────────
    let pd = DateComponents {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        hour: Some(hour),
        minute: Some(minute),
        second: Some(second),
        attos: Some(frac_attos),
        clock_type: ClockType::TAI,
        tz: Some(TimeZone::Utc),
        ..DateComponents::default()
    };

    pd.finish()
}

/// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
/// directly into [`DateComponents`].
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
/// A [`DateComponents`] with `timescale = Utc` and `tz = Utc`.
///
/// # Errors
/// Same as [`parse_ccsds_c`], plus rejection of Level-2 packets.
pub fn parse_ccsds_d(input: &[u8]) -> Result<DateComponents, DtError> {
    if input.is_empty() {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
    }

    let p1 = input[0];
    let mut idx = 1usize;

    // ── 1-byte vs 2-byte P-field ─────────────────────────────
    let extension = (p1 & 0b1000_0000) != 0;
    if extension {
        if input.len() < 2 {
            return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
        }
        idx += 1;
    }

    // Code ID must be 100
    let code_id = (p1 >> 4) & 0b0111;
    if code_id != 0b100 {
        return Err(DtError::new(DtErrKind::UnsupportedDirective));
    }

    // Epoch bit (bit 4) must be 0 for Level 1
    if (p1 & 0b0000_1000) != 0 {
        return Err(DtError::new(DtErrKind::UnsupportedDirective));
    }

    // Day segment length (bit 5)
    let n_day = if (p1 & 0b0000_0100) == 0 { 2 } else { 3 };

    // Submillisecond resolution (bits 6-7)
    let sub_ms_code = p1 & 0b0000_0011;
    let n_subsec = match sub_ms_code {
        0b00 => 0,
        0b01 => 2,
        0b10 => 4,
        _ => return Err(DtError::new(DtErrKind::UnsupportedDirective)),
    };

    if input.len() < idx + n_day + 4 + n_subsec {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
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
    let (year, month, day) = days_since_1958_to_gregorian(days_since_epoch);

    let hour = (sec_of_day / 3600) as u8;
    let minute = ((sec_of_day % 3600) / 60) as u8;
    let second = (sec_of_day % 60) as u8;

    // ── Build DateComponents ──────────────────────────────────────────────
    let pd = DateComponents {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        hour: Some(hour),
        minute: Some(minute),
        second: Some(second),
        attos: Some(frac_attos as u64),
        clock_type: ClockType::UTC,
        tz: Some(TimeZone::Utc),
        ..DateComponents::default()
    };

    pd.finish()
}

/// Auto-detects and parses either a CCSDS C (CUC) or D (CDS) binary time code
/// based on the Code ID in the first P-field byte.
///
/// Convenience wrapper around [`parse_ccsds_c`] and [`parse_ccsds_d`].
pub fn parse_ccsds_binary(input: &[u8]) -> Result<DateComponents, DtError> {
    if input.is_empty() {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
    }
    let code_id = (input[0] >> 4) & 0b0111;
    match code_id {
        0b001 => parse_ccsds_c(input),
        0b100 => parse_ccsds_d(input),
        _ => Err(DtError::new(DtErrKind::UnsupportedDirective)),
    }
}

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
    pub fn ccsds_c_to_binary(
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
    pub fn ccsds_d_to_binary(
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

        let day_count = (total_utc_seconds / 86_400) as u64;
        let sec_of_day = (total_utc_seconds % 86_400) as u64;

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

#[test]
fn test_ccsds_c_direct_frac() {
    // P-field = 0x15 (binary 00010101)
    //   → 2 coarse bytes, 1 fractional byte
    // T-field = coarse 0x0001 (1 second) + frac 0x80 (0.5 seconds)
    let c_bytes = &[0x15u8, 0x00, 0x01, 0x80];

    let parsed = parse_ccsds_c(c_bytes).unwrap();

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

    let parsed = parse_ccsds_c(c_bytes).unwrap();

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

    let parsed = parse_ccsds_d(d_bytes).unwrap();

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

    let parsed = parse_ccsds_d(d_bytes).unwrap();

    assert_eq!(parsed.second, Some(0));
    assert_eq!(parsed.attos, Some(1_500_000_000_000_000)); // 1.5 ms
}

/// Exact inverse of `days_since_1958_to_gregorian`.
/// Pure integer arithmetic – guarantees perfect round-tripping with the parser
/// when the same Y/M/D values are supplied. Used only for the roundtrip tests.
#[cfg(test)]
fn gregorian_to_days_since_1958(year: i64, month: u8, day: u8) -> i64 {
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

#[test]
fn test_ccsds_c_roundtrip() {
    // Desired CCSDS C (TAI) civil time: 2025-04-17 14:30:45.123456789 TAI
    // We compute the *exact* total TAI seconds since the CCSDS epoch (1958-01-01 00:00:00 TAI)
    // using the same Gregorian logic as the parser. This is 100% independent of the library's
    // JD, leap-second table, or to_time_point path.
    let days_since_1958 = gregorian_to_days_since_1958(2025, 4, 17);
    let sec_of_day = (14 * 3600) + (30 * 60) + 45;
    let total_tai_seconds = days_since_1958 * 86_400 + sec_of_day;

    // Library-internal TAI representation (TAI zero = library epoch)
    const EPOCH_OFFSET: i64 = 1_325_419_167;
    let tai_sec = total_tai_seconds - EPOCH_OFFSET;

    let t = TimePoint::new(tai_sec, 123_456_789_000_000_000, ClockType::TAI);

    let (buf, len) = t.ccsds_c_to_binary(4, 3, false).unwrap();
    let parsed = parse_ccsds_c(&buf[0..len]).unwrap();

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
    let days_since_1958 = gregorian_to_days_since_1958(2025, 4, 17);
    let sec_of_day = (14 * 3600) + (30 * 60) + 45;
    let total_utc_seconds = days_since_1958 * 86_400 + sec_of_day;

    // Library-internal UTC representation
    const EPOCH_OFFSET: i64 = 1_325_419_135;
    let utc_sec = total_utc_seconds - EPOCH_OFFSET;

    let t = TimePoint::new(utc_sec, 400_000_000_000, ClockType::UTC);

    let (buf, len) = t.ccsds_d_to_binary(2, 1, false).unwrap();
    let parsed = parse_ccsds_d(&buf[0..len]).unwrap();

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
