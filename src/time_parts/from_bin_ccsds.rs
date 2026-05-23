use crate::{Dt, DtErr, DtErrKind, Offset, Scale, TimeParts, an_err};

impl TimeParts {
    /// Converts days since 1958-01-01 (midnight UTC/TAI) into Gregorian date.
    /// Pure integer arithmetic matching the CCSDS 301.0-B-4 Level 1 epoch.
    pub fn days_since_1958_to_gregorian(days_since_epoch: i64) -> (i64, u8, u8) {
        let mut year = 1958i64;
        let mut remaining = days_since_epoch;

        while remaining >= 0 {
            let days_in_year = if Dt::is_leap_yr(year) { 366 } else { 365 };
            if remaining < days_in_year {
                break;
            }
            remaining -= days_in_year;
            year += 1;
        }

        let month_days = if Dt::is_leap_yr(year) {
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
            days += if Dt::is_leap_yr(y) { 366 } else { 365 };
            y += 1;
        }
        let month_days = if Dt::is_leap_yr(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        for mday in month_days.iter().take(month as usize - 1) {
            days += *mday as i64;
        }
        days + (day as i64 - 1)
    }

    /// Parses a CCSDS Calendar Segmented Time Code (CCS) into [`TimeParts`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.4 (Level 1 only)**.
    ///
    /// This function accepts a single-byte P-field followed by a BCD-encoded T-field.
    /// It supports both calendar variants:
    /// - Month/Day format (most common)
    /// - Day-of-Year format
    ///
    /// ## P-field
    ///
    /// - Must not have the extension bit set (only 1-byte P-fields are supported).
    /// - Code ID must be `101`.
    /// - Subsecond resolution: 0 to 6 BCD octets (0–12 decimal digits).
    ///
    /// ## T-field
    ///
    /// - Year is encoded as 4 BCD digits (0001–9999).
    /// - Time of day uses BCD with leap second support (`second == 60`).
    /// - When a leap second is present, `second` is normalized to 59 and
    ///   `is_leap_second` is set to `true` in the returned [`TimeParts`].
    ///
    /// ## Epoch
    ///
    /// 1958-01-01 00:00:00 UTC (identical to CDS).
    ///
    /// ## Errors
    ///
    /// Returns an error if the P-field is extended, the Code ID is wrong,
    /// BCD digits are invalid, field lengths are insufficient, or any
    /// component (month, day, DOY, hour, minute, second) is out of range.
    ///
    /// The resulting [`TimeParts`] has `scale = UTC`.
    pub fn from_ccsds_ccs(input: &[u8]) -> Result<TimeParts, DtErr> {
        if input.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        let p1 = input[0];
        let mut idx = 1usize;

        if (p1 & 0b1000_0000) != 0 {
            return Err(an_err!(
                DtErrKind::InvalidInput,
                "p-field ext. not supported"
            ));
        }

        let code_id = (p1 >> 4) & 0b0111;
        if code_id != 0b101 {
            return Err(an_err!(DtErrKind::InvalidItem, "code id"));
        }

        let is_doy = ((p1 >> 3) & 1) != 0;
        let n_subsec = (p1 & 0b0000_0111) as usize;

        if n_subsec > 6 {
            return Err(an_err!(DtErrKind::InvalidItem, "subsecond count"));
        }

        let min_len = 1 + 2 + 2 + 3 + n_subsec;
        if input.len() < min_len {
            return Err(an_err!(DtErrKind::InvalidSyntax, "t-field too short"));
        }

        let bcd_byte = |b: u8| -> Result<u8, DtErr> {
            let hi = b >> 4;
            let lo = b & 0x0F;
            if hi > 9 || lo > 9 {
                Err(an_err!(DtErrKind::InvalidBytes, "invalid bcd digit"))
            } else {
                Ok(hi * 10 + lo)
            }
        };

        // Year
        let y1 = bcd_byte(input[idx])?;
        let y2 = bcd_byte(input[idx + 1])?;
        let year = (y1 as i64) * 100 + (y2 as i64);
        idx += 2;

        // Date field
        let (month, day, day_of_year) = if !is_doy {
            let mo = bcd_byte(input[idx])?;
            let d = bcd_byte(input[idx + 1])?;
            idx += 2;

            if !(1..=12).contains(&mo) {
                return Err(an_err!(DtErrKind::OutOfRange, "month"));
            }
            if !(1..=31).contains(&d) {
                return Err(an_err!(DtErrKind::OutOfRange, "day"));
            }
            (Some(mo), Some(d), None)
        } else {
            let d1 = bcd_byte(input[idx])?;
            let d2 = bcd_byte(input[idx + 1])?;
            idx += 2;
            let doy = (d1 as u16) * 100 + (d2 as u16);

            if doy == 0 || doy > 366 || (doy == 366 && !Dt::is_leap_yr(year)) {
                return Err(an_err!(DtErrKind::OutOfRange, "day of year"));
            }
            (None, None, Some(doy))
        };

        // Time
        let hour = bcd_byte(input[idx])?;
        let minute = bcd_byte(input[idx + 1])?;
        let mut second = bcd_byte(input[idx + 2])?;
        idx += 3;

        if hour > 23 {
            return Err(an_err!(DtErrKind::OutOfRange, "hour"));
        }
        if minute > 59 {
            return Err(an_err!(DtErrKind::OutOfRange, "minute"));
        }

        let is_leap_second = second == 60;
        if is_leap_second {
            second = 59;
        } else if second > 59 {
            return Err(an_err!(DtErrKind::OutOfRange, "second"));
        }

        // Subseconds (BCD → attoseconds)
        let mut frac_value: u128 = 0;
        for _ in 0..n_subsec {
            let b = input[idx];
            let hi = (b >> 4) as u128;
            let lo = (b & 0x0F) as u128;
            if hi > 9 || lo > 9 {
                return Err(an_err!(DtErrKind::InvalidBytes, "invalid subsecond bcd"));
            }
            frac_value = frac_value * 100 + hi * 10 + lo;
            idx += 1;
        }

        let attos = if n_subsec == 0 {
            0
        } else {
            let decimal_places = (2 * n_subsec) as u32;
            let denom = 10u128.pow(decimal_places);
            ((frac_value * 1_000_000_000_000_000_000u128) / denom) as u64
        };

        let mut pd = TimeParts {
            yr: Some(year),
            mo: month,
            day,
            day_of_yr: day_of_year,
            hr: Some(hour),
            min: Some(minute),
            sec: Some(second),
            attos: Some(attos),
            is_leap_sec: is_leap_second,
            scale: Scale::UTC,
            offset: Some(Offset::Utc),
            ..TimeParts::default()
        };

        pd.finish(false)?;
        Ok(pd)
    }

    /// Parses a CCSDS Unsegmented Time Code (CUC) into [`TimeParts`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.2 (Level 1)**, including full support
    /// for the extended 2-byte P-field defined in Issue 4.
    ///
    /// ## P-field
    ///
    /// - Supports both 1-byte and 2-byte P-fields.
    /// - Code ID must be `001` (1958-01-01 TAI epoch).
    /// - Coarse time: 1–7 octets total.
    /// - Fractional time: 0–10 octets total.
    /// - P-fields longer than 2 bytes are rejected.
    ///
    /// ## T-field
    ///
    /// - Coarse time is interpreted as seconds since the 1958 TAI epoch.
    /// - Fractional time is converted to attoseconds using exact integer scaling
    ///   (`value / 2^(8·n_frac)`).
    ///
    /// ## Epoch
    ///
    /// 1958-01-01 00:00:00 TAI.
    ///
    /// ## Errors
    ///
    /// Returns an error for empty input, insufficient length, invalid Code ID,
    /// unsupported further P-field extensions, or malformed T-field data.
    ///
    /// The resulting [`TimeParts`] has `scale = TAI`.
    pub fn from_ccsds_c(input: &[u8]) -> Result<TimeParts, DtErr> {
        if input.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        let p1 = input[0];
        let mut idx = 1usize;

        let extension = (p1 & 0b1000_0000) != 0;
        let code_id = (p1 >> 4) & 0b0111;
        if code_id != 0b001 {
            return Err(an_err!(DtErrKind::InvalidItem, "code id"));
        }

        let base_coarse = (((p1 >> 2) & 0b0011) as usize) + 1;
        let base_frac = (p1 & 0b0011) as usize;

        let (n_coarse, n_frac) = if extension {
            if input.len() < 2 {
                return Err(an_err!(DtErrKind::InvalidInput, "p-field too short"));
            }
            let p2 = input[1];
            idx += 1;

            if (p2 & 0b1000_0000) != 0 {
                return Err(an_err!(
                    DtErrKind::InvalidInput,
                    "further p-field ext. not supported"
                ));
            }

            let add_coarse = ((p2 >> 5) & 0b0000_0011) as usize;
            let add_frac = ((p2 >> 2) & 0b0000_0111) as usize;

            (base_coarse + add_coarse, base_frac + add_frac)
        } else {
            (base_coarse, base_frac)
        };

        if n_coarse == 0 || input.len() < idx + n_coarse + n_frac {
            return Err(an_err!(DtErrKind::InvalidSyntax, "t-field too short"));
        }

        // Read coarse time (big-endian)
        let mut coarse_sec: u64 = 0;
        for _ in 0..n_coarse {
            coarse_sec = (coarse_sec << 8) | u64::from(input[idx]);
            idx += 1;
        }

        // Read fractional time (big-endian)
        let mut frac_raw: u128 = 0;
        for _ in 0..n_frac {
            frac_raw = (frac_raw << 8) | u128::from(input[idx]);
            idx += 1;
        }

        let frac_attos = if n_frac == 0 {
            0
        } else {
            let denom = 1u128 << (8 * n_frac as u32);
            ((frac_raw * 1_000_000_000_000_000_000u128) / denom) as u64
        };

        // Convert to civil time using custom Gregorian conversion
        let days_since_epoch = (coarse_sec / 86400) as i64;
        let sec_of_day = (coarse_sec % 86400) as i64;

        let (year, month, day) = TimeParts::days_since_1958_to_gregorian(days_since_epoch);

        let hour = (sec_of_day / 3600) as u8;
        let minute = ((sec_of_day % 3600) / 60) as u8;
        let second = (sec_of_day % 60) as u8;

        let mut pd = TimeParts {
            yr: Some(year),
            mo: Some(month),
            day: Some(day),
            hr: Some(hour),
            min: Some(minute),
            sec: Some(second),
            attos: Some(frac_attos),
            scale: Scale::TAI,
            offset: Some(Offset::Utc),
            ..TimeParts::default()
        };
        pd.finish(false)?;
        Ok(pd)
    }

    /// Parses a CCSDS Day Segmented Time Code (CDS) into [`TimeParts`].
    ///
    /// Implements **CCSDS 301.0-B-4 §3.3 (Level 1)**.
    ///
    /// ## P-field
    ///
    /// - Supports optional 2-byte P-field.
    /// - Code ID must be `100`.
    /// - Epoch bit must be `0` (1958-01-01 UTC epoch only).
    /// - Day count: 2 or 3 bytes.
    /// - Sub-millisecond resolution: none, 2 bytes (µs), or 4 bytes (2⁻³² of a ms).
    ///
    /// ## T-field
    ///
    /// - Day count is days since 1958-01-01 UTC.
    /// - Milliseconds since midnight are always 4 bytes.
    /// - Sub-millisecond field (if present) is converted to attoseconds.
    ///
    /// ## Leap Second Handling
    ///
    /// This implementation correctly supports leap seconds. When `millis_of_day`
    /// represents 23:59:60 (i.e. ≥ 86,400,000 ms), `second` is set to 60 and
    /// `is_leap_second` is set to `true` in the returned [`TimeParts`].
    ///
    /// ## Epoch
    ///
    /// 1958-01-01 00:00:00 UTC.
    ///
    /// ## Errors
    ///
    /// Returns an error for empty input, wrong Code ID, non-Level-1 epoch,
    /// unsupported sub-millisecond code, insufficient length, or invalid data.
    ///
    /// The resulting [`TimeParts`] has `scale = UTC`.
    pub fn from_ccsds_d(input: &[u8]) -> Result<TimeParts, DtErr> {
        if input.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }

        let p1 = input[0];
        let mut idx = 1usize;

        let extension = (p1 & 0b1000_0000) != 0;
        if extension {
            if input.len() < 2 {
                return Err(an_err!(DtErrKind::InvalidInput, "p-field too short"));
            }
            idx += 1;
        }

        let code_id = (p1 >> 4) & 0b0111;
        if code_id != 0b100 {
            return Err(an_err!(DtErrKind::InvalidItem, "code id"));
        }

        if (p1 & 0b0000_1000) != 0 {
            return Err(an_err!(
                DtErrKind::InvalidItem,
                "non-level-1 epoch not supported"
            ));
        }

        let n_day = if (p1 & 0b0000_0100) == 0 { 2 } else { 3 };
        let sub_ms_code = p1 & 0b0000_0011;

        let n_subsec = match sub_ms_code {
            0b00 => 0,
            0b01 => 2,
            0b10 => 4,
            _ => return Err(an_err!(DtErrKind::InvalidItem, "sub-millisecond code")),
        };

        if input.len() < idx + n_day + 4 + n_subsec {
            return Err(an_err!(DtErrKind::InvalidSyntax, "t-field too short"));
        }

        // Read fields
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

        // === Leap second handling (robust) ===
        let total_sec_in_day = millis_of_day / 1000;
        let is_leap_second = total_sec_in_day == 86400;

        let effective_sec = if is_leap_second {
            86399
        } else {
            total_sec_in_day
        };

        let sec_of_day = effective_sec;
        let remaining_ms = (millis_of_day % 1000) as u128;

        // Sub-millisecond to attoseconds
        let sub_ms_attos = if n_subsec == 0 {
            0
        } else if sub_ms_code == 0b01 {
            (frac_raw as u128 * 1_000_000_000_000_000) / 65_536
        } else {
            (frac_raw as u128 * 1_000_000_000_000_000_000) / (1u128 << 32)
        };

        let frac_attos = remaining_ms * 1_000_000_000_000_000 + sub_ms_attos;

        // Convert day count to Gregorian
        let days_since_epoch = day_count as i64;
        let (year, month, day) = TimeParts::days_since_1958_to_gregorian(days_since_epoch);

        let hour = (sec_of_day / 3600) as u8;
        let minute = ((sec_of_day % 3600) / 60) as u8;
        let mut second = (sec_of_day % 60) as u8;

        if is_leap_second {
            second = 60;
        }

        let mut pd = TimeParts {
            yr: Some(year),
            mo: Some(month),
            day: Some(day),
            hr: Some(hour),
            min: Some(minute),
            sec: Some(second),
            attos: Some(frac_attos as u64),
            is_leap_sec: is_leap_second,
            scale: Scale::UTC,
            offset: Some(Offset::Utc),
            ..TimeParts::default()
        };
        pd.finish(false)?;
        Ok(pd)
    }

    /// Auto-detects and parses a CCSDS binary time code (CUC, CDS, or CCS).
    ///
    /// Examines the Code ID in the first P-field byte and dispatches to the
    /// appropriate parser:
    /// - `001` → [`TimeParts::from_ccsds_c`](../struct.TimeParts.html#method.from_ccsds_c) (CUC)
    /// - `100` → [`TimeParts::from_ccsds_d`](../struct.TimeParts.html#method.from_ccsds_d) (CDS)
    /// - `101` → [`TimeParts::from_ccsds_ccs`](../struct.TimeParts.html#method.from_ccsds_ccs)
    ///   (CCS)
    ///
    /// This is a convenience wrapper. For stricter control or when the format
    /// is known in advance, prefer calling the specific `from_ccsds_*` function directly.
    ///
    /// ## Errors
    /// Returns an error if the input is empty or the Code ID is not one of the
    /// three recognized Level 1 values.
    pub fn from_ccsds_bin(input: &[u8]) -> Result<TimeParts, DtErr> {
        if input.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        }
        let code_id = (input[0] >> 4) & 0b0111;
        match code_id {
            0b001 => Self::from_ccsds_c(input),
            0b100 => Self::from_ccsds_d(input),
            0b101 => Self::from_ccsds_ccs(input),
            _ => Err(an_err!(DtErrKind::InvalidItem, "unknown code id")),
        }
    }
}
