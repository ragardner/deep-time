use crate::{
    DtErrKind, DtError, TimePoint,
    parser::{ParsedDate, ParsedTimeScale, TimeZone},
};

/// Parses a **CCSDS C (CUC – Unsegmented Time Code)** binary time code
/// directly into [`ParsedDate`].
///
/// This function implements CCSDS 301.0-B-4 §3.2 (Level 1 only).
///
/// # Supported formats
/// - 1-byte or 2-byte P-field (extension bit is supported but the second byte is ignored for Level 1).
/// - Code ID must be `001` (1958-01-01 TAI epoch).
/// - `n_coarse`: 1–4 bytes for the coarse time field.
/// - `n_frac`:   0–3 bytes for the fractional field.
///
/// # Precision
/// Fractional seconds are converted to attoseconds with exact integer scaling.
/// The maximum quantization error depends on `n_frac`:
/// - 1 byte:  ~3.9 ms
/// - 2 bytes: ~15.3 µs
/// - 3 bytes: ~59.6 ns
///
/// # Returns
/// A [`ParsedDate`] with `timescale = TAI` and `tz = Utc`.
///
/// # Errors
/// - [`DtErrKind::ExpectedUnixTimestamp`] if the input is too short.
/// - [`DtErrKind::UnsupportedDirective`] for non-Level-1 packets or invalid P-field.
///
/// This function is designed for perfect round-tripping with [`TimePoint::ccsds_c_to_binary`]
/// when the same `n_coarse`/`n_frac` values are used.
pub fn parse_ccsds_c(input: &[u8]) -> Result<ParsedDate, DtError> {
    if input.is_empty() {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
    }

    let p1 = input[0];
    let mut idx = 1usize;

    // ── Handle 1-byte vs 2-byte P-field ─────────────────────────────
    let extension = (p1 & 0b1000_0000) != 0;
    if extension {
        if input.len() < 2 {
            return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
        }
        idx += 1; // consume the second P-field byte (we ignore its contents for now)
    }

    let code_id = (p1 >> 4) & 0b0111;
    if code_id != 0b001 {
        // Only Level 1 (1958-01-01 TAI) is supported
        return Err(DtError::new(DtErrKind::UnsupportedDirective));
    }

    let n_coarse = ((p1 >> 2) & 0b0011) as usize + 1; // bits 3-2
    let n_frac = (p1 & 0b0011) as usize; // bits 1-0

    if input.len() < idx + n_coarse + n_frac {
        return Err(DtError::new(DtErrKind::ExpectedUnixTimestamp));
    }

    // ── Read T-field (big-endian) ─────────────────────────────────────
    let mut coarse_sec: u64 = 0;
    for _ in 0..n_coarse {
        coarse_sec = (coarse_sec << 8) | u64::from(input[idx]);
        idx += 1;
    }

    let mut frac_raw: u64 = 0;
    for _ in 0..n_frac {
        frac_raw = (frac_raw << 8) | u64::from(input[idx]);
        idx += 1;
    }

    // Fractional bytes → attoseconds (exact)
    let frac_attos = if n_frac == 0 {
        0
    } else {
        let denom = 1u128 << (8 * n_frac);
        ((frac_raw as u128 * 1_000_000_000_000_000_000u128) / denom) as u64
    };

    // ── Convert TAI seconds since 1958-01-01 to Gregorian ─────────────
    let base_jdn = TimePoint::gregorian_jdn(1958, 1, 1);
    let days = (coarse_sec / 86400) as i64;
    let sec_of_day = (coarse_sec % 86400) as i64;

    let jdn = base_jdn + days;
    let (year, month, day) = TimePoint::jdn_to_gregorian(jdn);

    let hour = (sec_of_day / 3600) as u8;
    let minute = ((sec_of_day % 3600) / 60) as u8;
    let second = (sec_of_day % 60) as u8;

    // ── Build ParsedDate ──────────────────────────────────────────────
    let pd = ParsedDate {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        hour: Some(hour),
        minute: Some(minute),
        second: Some(second),
        attos: Some(frac_attos),
        timescale: ParsedTimeScale::Tai,
        tz: Some(TimeZone::Utc),
        ..ParsedDate::default()
    };

    pd.finish()
}

/// Parses a **CCSDS D (CDS – Day Segmented Time Code)** binary time code
/// directly into [`ParsedDate`].
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
///   - `10`: 4 bytes (picoseconds of the millisecond)
///
/// # Precision
/// - The millisecond field is rounded to the nearest millisecond.
/// - With 2-byte sub-ms: maximum error ≈ ±7.6 ns.
/// - With 4-byte sub-ms: maximum error ≈ ±0.116 ps.
///
/// # Returns
/// A [`ParsedDate`] with `timescale = Utc` and `tz = Utc`.
///
/// # Errors
/// Same as [`parse_ccsds_c`], plus rejection of Level-2 packets.
pub fn parse_ccsds_d(input: &[u8]) -> Result<ParsedDate, DtError> {
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

    // ── Convert to attoseconds (correct scaling) ─────────────
    let sec_of_day = millis_of_day / 1000;
    let remaining_ms = (millis_of_day % 1000) as u128;

    let sub_ms_attos = if n_subsec == 0 {
        0
    } else if sub_ms_code == 0b01 {
        // 2 bytes → fraction of 1 ms in microseconds (0..65535)
        (frac_raw as u128 * 1_000_000_000_000) / 65_536
    } else {
        // 4 bytes → fraction of 1 ms in picoseconds
        (frac_raw as u128 * 1_000_000_000_000) / (1u128 << 32)
    };

    let frac_attos = remaining_ms * 1_000_000_000_000 + sub_ms_attos;

    // ── Gregorian conversion ───────────────────────────────
    let base_jdn = TimePoint::gregorian_jdn(1958, 1, 1);
    let jdn = base_jdn + (day_count as i64);
    let (year, month, day) = TimePoint::jdn_to_gregorian(jdn);

    let hour = (sec_of_day / 3600) as u8;
    let minute = ((sec_of_day % 3600) / 60) as u8;
    let second = (sec_of_day % 60) as u8;

    let pd = ParsedDate {
        year: Some(year),
        month: Some(month),
        day: Some(day),
        hour: Some(hour),
        minute: Some(minute),
        second: Some(second),
        attos: Some(frac_attos as u64),
        timescale: ParsedTimeScale::Utc,
        tz: Some(TimeZone::Utc),
        ..ParsedDate::default()
    };

    pd.finish()
}

/// Auto-detects and parses either a CCSDS C (CUC) or D (CDS) binary time code
/// based on the Code ID in the first P-field byte.
///
/// Convenience wrapper around [`parse_ccsds_c`] and [`parse_ccsds_d`].
pub fn parse_ccsds_binary(input: &[u8]) -> Result<ParsedDate, DtError> {
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
    /// Maximum size needed for a CCSDS C (CUC) binary packet.
    const CCSDS_C_MAX_SIZE: usize = 32;

    /// Formats this [`TimePoint`] as a **CCSDS C (CUC)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`parse_ccsds_c`].
    ///
    /// # Parameters
    /// - `n_coarse`: 1–4 bytes for the coarse (integer seconds) field.
    /// - `n_frac`:   0–3 bytes for the fractional field.
    /// - `extension`: if `true`, a 2-byte P-field is emitted (second byte is always 0 for Level 1).
    ///
    /// # Precision
    /// Fractional seconds are rounded to the nearest representable value for the chosen `n_frac`.
    /// The maximum quantization error is half the step size of the selected resolution.
    ///
    /// # Returns
    /// A fixed-size buffer containing the binary packet and the number of bytes written.
    ///
    /// # Errors
    /// [`DtErrKind::UnsupportedDirective`] if `n_coarse` or `n_frac` are out of range.
    pub fn ccsds_c_to_binary(
        &self,
        n_coarse: u8,
        n_frac: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_C_MAX_SIZE], usize), DtErrKind> {
        if !(1..=4).contains(&n_coarse) || n_frac > 3 {
            return Err(DtErrKind::UnsupportedDirective);
        }

        let base_jdn = Self::gregorian_jdn(1958, 1, 1);
        let (year, month, day) = self.to_gregorian_date();
        let current_jdn = Self::gregorian_jdn(year, month, day);
        let days_since_epoch = current_jdn - base_jdn;

        let (hour, minute, second, subsec_attos) = self.to_hms_subsec();
        let sec_of_day = (hour as i64) * 3600 + (minute as i64) * 60 + (second as i64);
        let total_tai_seconds = days_since_epoch * 86400 + sec_of_day;

        let frac_scaled = if n_frac == 0 {
            0u64
        } else {
            let scale = 1u128 << (8 * n_frac as u32);
            ((subsec_attos as u128 * scale + 500_000_000_000_000_000) / 1_000_000_000_000_000_000)
                as u64
        };

        let mut buf = [0u8; Self::CCSDS_C_MAX_SIZE];
        let mut pos = 0usize;

        let mut p1 = 0b0001_0000u8;
        p1 |= (n_coarse - 1) << 2;
        p1 |= n_frac;
        if extension {
            p1 |= 0b1000_0000;
        }
        buf[pos] = p1;
        pos += 1;

        if extension {
            buf[pos] = 0;
            pos += 1;
        }

        let coarse = total_tai_seconds as u64;
        for i in (0..n_coarse).rev() {
            buf[pos] = (coarse >> (i * 8)) as u8;
            pos += 1;
        }

        for i in (0..n_frac).rev() {
            buf[pos] = (frac_scaled >> (i * 8)) as u8;
            pos += 1;
        }

        Ok((buf, pos))
    }

    /// Maximum size needed for a CCSDS D (CDS) binary packet.
    const CCSDS_D_MAX_SIZE: usize = 32;

    /// Formats this [`TimePoint`] as a **CCSDS D (CDS)** binary time code.
    ///
    /// Fully configurable for round-tripping with [`parse_ccsds_d`].
    ///
    /// # Parameters
    /// - `n_day`: 2 or 3 bytes for the day count field.
    /// - `sub_ms_code`:
    ///   - `0`: no sub-millisecond field
    ///   - `1`: 2 bytes (microseconds of the millisecond)
    ///   - `2`: 4 bytes (picoseconds of the millisecond)
    /// - `extension`: if `true`, a 2-byte P-field is emitted.
    ///
    /// # Precision
    /// - Milliseconds-of-day are rounded to the nearest millisecond.
    /// - The sub-millisecond field is rounded to the nearest representable value.
    /// - With `sub_ms_code = 1`: max error ≈ ±7.6 ns
    /// - With `sub_ms_code = 2`: max error ≈ ±0.116 ps
    ///
    /// # Returns
    /// Same as [`ccsds_c_to_binary`].
    pub fn ccsds_d_to_binary(
        &self,
        n_day: u8,
        sub_ms_code: u8,
        extension: bool,
    ) -> Result<([u8; Self::CCSDS_D_MAX_SIZE], usize), DtErrKind> {
        if !matches!(n_day, 2 | 3) || !matches!(sub_ms_code, 0 | 1 | 2) {
            return Err(DtErrKind::UnsupportedDirective);
        }

        let (hour, minute, second, subsec_attos) = self.to_hms_subsec();

        let seconds_of_day = (hour as u64) * 3600 + (minute as u64) * 60 + second as u64;

        // Round to nearest millisecond
        let additional_ms = ((subsec_attos as u128 + 500_000_000_000) / 1_000_000_000_000) as u64;
        let millis_of_day = seconds_of_day * 1000 + additional_ms;

        // Remaining attoseconds inside the current millisecond
        let remaining_attos_in_ms = (subsec_attos as u128) % 1_000_000_000_000;

        // Fractional scaling with explicit u128 constants to avoid inference issues
        let frac_scaled = match sub_ms_code {
            0 => 0u64,
            1 => ((remaining_attos_in_ms * 65_536u128) / 1_000_000_000_000u128) as u64,
            2 => {
                const PS_SCALE: u128 = 1u128 << 32;
                ((remaining_attos_in_ms * PS_SCALE) / 1_000_000_000_000u128) as u64
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

        let day_count = {
            let base_jdn = Self::gregorian_jdn(1958, 1, 1);
            let (year, month, day) = self.to_gregorian_date();
            let current_jdn = Self::gregorian_jdn(year, month, day);
            (current_jdn - base_jdn) as u64
        };

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
    assert_eq!(parsed.timescale, ParsedTimeScale::Tai);
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
    assert_eq!(parsed.attos, Some(1_000_000_000_000)); // 1 ms
    assert_eq!(parsed.timescale, ParsedTimeScale::Utc);
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
    assert_eq!(parsed.attos, Some(1_500_000_000_000)); // 1 ms + 0.5 ms = 1.5 ms
}

#[test]
fn test_ccsds_c_roundtrip() {
    use crate::Delta;

    // Helper to create a TimePoint at the requested civil time (TT scale).
    // The library uses standard astronomical JD (day changes at *noon*).
    // This helper now correctly converts civil (midnight-based) time to that representation.
    fn tp(y: i64, m: u8, d: u8, h: u8, min: u8, s: u8, attos: u64) -> TimePoint {
        let jd_noon = TimePoint::gregorian_jdn(y, m, d);
        let seconds_from_noon = (h as i64 - 12) * 3600 + (min as i64) * 60 + (s as i64);
        let (jd_days, delta_sec) = if seconds_from_noon >= 0 {
            (jd_noon, seconds_from_noon)
        } else {
            (jd_noon - 1, seconds_from_noon + 86400)
        };
        TimePoint::from_jd_tt_exact(jd_days, Delta::new(delta_sec, attos))
    }
    let t = tp(2025, 4, 17, 14, 30, 45, 123_456_789_000_000_000);

    // Use safe defaults for long-term dates: 4 coarse bytes + 3 fractional bytes
    let (buf, len) = t.ccsds_c_to_binary(4, 3, false).unwrap();

    let parsed = parse_ccsds_c(&buf[0..len]).unwrap();

    assert_eq!(parsed.year, Some(2025));
    assert_eq!(parsed.month, Some(4));
    assert_eq!(parsed.day, Some(17));
    assert_eq!(parsed.hour, Some(14));
    assert_eq!(parsed.minute, Some(30));
    assert_eq!(parsed.second, Some(45));
    assert_eq!(parsed.timescale, ParsedTimeScale::Tai);

    // Allow up to ~60 ns error (correct quantization bound for n_frac=3)
    let diff = (parsed.attos.unwrap() as i64 - 123_456_789_000_000_000i64).abs();
    assert!(
        diff < 60_000_000_000,
        "Fractional error too large: {} attos",
        diff
    );
}

#[test]
fn test_ccsds_d_roundtrip() {
    use crate::Delta;

    fn tp(y: i64, m: u8, d: u8, h: u8, min: u8, s: u8, attos: u64) -> TimePoint {
        let jd_noon = TimePoint::gregorian_jdn(y, m, d);
        let seconds_from_noon = (h as i64 - 12) * 3600 + (min as i64) * 60 + (s as i64);
        let (jd_days, delta_sec) = if seconds_from_noon >= 0 {
            (jd_noon, seconds_from_noon)
        } else {
            (jd_noon - 1, seconds_from_noon + 86400)
        };
        TimePoint::from_jd_tt_exact(jd_days, Delta::new(delta_sec, attos))
    }

    let t = tp(2025, 4, 17, 14, 30, 45, 400_000_000_000);

    let (buf, len) = t.ccsds_d_to_binary(2, 1, false).unwrap();

    let parsed = parse_ccsds_d(&buf[0..len]).unwrap();

    assert_eq!(parsed.year, Some(2025));
    assert_eq!(parsed.month, Some(4));
    assert_eq!(parsed.day, Some(17));
    assert_eq!(parsed.hour, Some(14));
    assert_eq!(parsed.minute, Some(30));
    assert_eq!(parsed.second, Some(45));
    assert_eq!(parsed.timescale, ParsedTimeScale::Utc);

    let diff = (parsed.attos.unwrap() as i64 - 400_000_000_000i64).abs();
    assert!(
        diff < 10_000_000,
        "Fractional error too large: {} attos",
        diff
    );
}
