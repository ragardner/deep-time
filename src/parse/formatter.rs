use crate::{ClockType, TimePoint, error::DtErrKind, parser::Weekday};

/// Fixed UTC offset in seconds (positive = east of UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcOffset(i32);

impl UtcOffset {
    pub const ZERO: Self = Self(0);
    pub const UTC: Self = Self(0);

    #[inline]
    pub const fn from_seconds(seconds: i32) -> Self {
        Self(seconds)
    }

    #[inline]
    pub const fn seconds(self) -> i32 {
        self.0
    }

    #[inline]
    pub const fn from_hms(hours: i32, minutes: i32, seconds: i32) -> Self {
        Self(hours * 3600 + minutes * 60 + seconds)
    }

    /// Returns `(is_negative, hours, minutes)`.
    #[inline]
    pub const fn as_hhmm(self) -> (bool, u8, u8) {
        let total = self.0.abs();
        let hours = (total / 3600) as u8;
        let minutes = ((total % 3600) / 60) as u8;
        (self.0 < 0, hours, minutes)
    }
}

impl TimePoint {
    /// Core zero-allocation formatter (no_std compatible).
    pub fn format_u8_with_offset(
        &self,
        fmt: &str,
        dest: &mut [u8],
        offset: UtcOffset,
    ) -> Result<usize, DtErrKind> {
        let mut internal_buf = [0u8; Self::BUFFER_SIZE];
        let mut pos = 0usize;

        // Just forward the bytes – format_to_buffer stays exactly as-is
        Self::format_to_buffer(self, fmt.as_bytes(), &mut internal_buf, &mut pos, offset)?;

        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Convenience version that assumes UTC.
    #[inline(always)]
    pub fn format_u8(&self, fmt: &str, dest: &mut [u8]) -> Result<usize, DtErrKind> {
        self.format_u8_with_offset(fmt, dest, UtcOffset::UTC)
    }

    /// High-level alloc version (defaults to UTC).
    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub fn format(&self, fmt: &str) -> Result<alloc::string::String, DtErrKind> {
        self.format_with_offset(fmt, UtcOffset::UTC)
    }

    /// High-level alloc version with explicit offset.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn format_with_offset(
        &self,
        fmt: &str,
        offset: UtcOffset,
    ) -> Result<alloc::string::String, DtErrKind> {
        let mut internal_buf = [0u8; Self::BUFFER_SIZE];
        let n = self.format_u8_with_offset(fmt, &mut internal_buf, offset)?;

        // Safe: everything emitted is valid ASCII/UTF-8
        Ok(alloc::string::String::from_utf8_lossy(&internal_buf[0..n]).into_owned())
    }

    fn format_to_buffer(
        &self,
        fmt: &[u8],
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        offset: UtcOffset,
    ) -> Result<(), DtErrKind> {
        let mut i = 0usize;

        while i < fmt.len() {
            let byte = fmt[i];

            if byte != b'%' {
                Self::write_bytes(buf, pos, &[byte]);
                i += 1;
                continue;
            }

            i += 1; // skip '%'

            if i >= fmt.len() {
                return Err(DtErrKind::UnexpectedEndAfterPercent);
            }

            // %% → literal percent
            if fmt[i] == b'%' {
                Self::write_bytes(buf, pos, b"%");
                i += 1;
                continue;
            }

            // ── Parse optional flags (- 0 _) ───────────────────────
            let mut flag = b'0'; // temporary default; many directives override it via pad param
            while i < fmt.len() {
                match fmt[i] {
                    b'-' | b'0' | b'_' => {
                        flag = fmt[i];
                        i += 1;
                    }
                    _ => break,
                }
            }

            // ── Parse optional width ───────────────────────────────
            let mut width: Option<u8> = None;
            let width_start = i;
            while i < fmt.len() && fmt[i].is_ascii_digit() {
                i += 1;
            }
            if i > width_start {
                if let Ok(s) = core::str::from_utf8(&fmt[width_start..i]) {
                    if let Ok(w) = s.parse::<u8>() {
                        width = Some(w);
                    }
                }
            }

            // ── Parse optional colons (: :: :::) ───────────────────
            let mut colons: u8 = 0;
            while i < fmt.len() && fmt[i] == b':' {
                colons += 1;
                i += 1;
            }

            if i >= fmt.len() {
                return Err(DtErrKind::UnexpectedEndAfterPercent);
            }

            let directive = fmt[i];
            i += 1;

            // ── Special case: %.f or %.9f etc. ─────────────────────
            if directive == b'.' {
                let mut frac_width: Option<u8> = None;
                let frac_start = i;
                while i < fmt.len() && fmt[i].is_ascii_digit() {
                    i += 1;
                }
                if i > frac_start {
                    if let Ok(s) = core::str::from_utf8(&fmt[frac_start..i]) {
                        if let Ok(w) = s.parse::<u8>() {
                            frac_width = Some(w);
                        }
                    }
                }

                if i >= fmt.len() {
                    return Err(DtErrKind::ExpectedFOrNAfterDot);
                }

                let next = fmt[i];
                i += 1;

                if matches!(next, b'f' | b'N') {
                    // FIXED: Only print the dot for %f when width > 0
                    let width_val = frac_width.unwrap_or(18);
                    let add_dot = (next == b'f') && (width_val > 0);

                    if add_dot {
                        Self::write_bytes(buf, pos, b".");
                    }
                    self.write_fractional_seconds(buf, pos, flag, frac_width, colons);
                    continue;
                } else {
                    return Err(DtErrKind::ExpectedFOrNAfterDot);
                }
            }

            // ── Normal directives (exact match to your original parser) ──
            match directive {
                b'A' => self.write_weekday_full(buf, pos),
                b'a' => self.write_weekday_abbrev(buf, pos),
                b'B' => self.write_month_name_full(buf, pos),
                b'b' | b'h' => self.write_month_name_abbrev(buf, pos),
                b'C' => self.write_century(buf, pos, flag, width, colons),
                b'd' | b'e' => self.write_day_of_month(buf, pos, flag, width, colons, true),
                b'f' | b'N' => self.write_fractional_seconds(buf, pos, flag, width, colons),
                b'G' => self.write_iso_week_year(buf, pos, flag, width, colons),
                b'g' => self.write_two_digit_iso_week_year(buf, pos, flag, width, colons),
                b'H' | b'k' => self.write_hour24(buf, pos, flag, width, colons, true),
                b'I' | b'l' => self.write_hour12(buf, pos, flag, width, colons),
                b'j' => self.write_day_of_year(buf, pos, flag, width, colons),
                b'M' => self.write_minute(buf, pos, flag, width, colons, true),
                b'm' => self.write_month_number(buf, pos, flag, width, colons, true),
                b'n' => self.write_whitespace(buf, pos, b'n'),
                b't' => self.write_whitespace(buf, pos, b't'),
                b'P' => self.write_ampm(buf, pos, false),
                b'p' => self.write_ampm(buf, pos, true),
                b'Q' => self.write_iana_or_offset(buf, pos, flag, width, colons, offset),
                b'S' => self.write_second(buf, pos, flag, width, colons, true),
                b's' => self.write_unix_timestamp(buf, pos, flag, width, colons),
                b'U' => self.write_week_number_sunday_based(buf, pos, flag, width, colons),
                b'u' => self.write_weekday_number_monday_based(buf, pos, flag, width, colons),
                b'V' => self.write_week_iso(buf, pos, flag, width, colons),
                b'W' => self.write_week_number_monday_based(buf, pos, flag, width, colons),
                b'w' => self.write_weekday_number_sunday_based(buf, pos, flag, width, colons),
                b'Y' => self.write_full_year(buf, pos, flag, width, colons, true),
                b'y' => self.write_two_digit_year(buf, pos, flag, width, colons, true),
                b'*' => self.write_unbounded_year(buf, pos, flag, width, colons),
                b'z' => self.write_timezone_offset(buf, pos, flag, width, colons, offset),
                b'F' => self.write_iso_date(buf, pos),
                b'D' => self.write_us_date_shortcut(buf, pos),
                b'T' => self.write_time_with_seconds_shortcut(buf, pos),
                b'R' => self.write_time_without_seconds_shortcut(buf, pos),

                b'c' | b'r' | b'X' | b'x' | b'Z' => self.write_unsupported(buf, pos),
                _ => return Err(DtErrKind::UnknownFormatDirective),
            }
        }

        Ok(())
    }

    // ──────────────────────────────────────────────────────────────
    // Constants (byte slices → zero-allocation, no_std)
    // ──────────────────────────────────────────────────────────────
    const WEEKDAYS_FULL: [&[u8]; 7] = [
        b"Sunday",
        b"Monday",
        b"Tuesday",
        b"Wednesday",
        b"Thursday",
        b"Friday",
        b"Saturday",
    ];
    const WEEKDAYS_ABBR: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
    const MONTHS_FULL: [&[u8]; 12] = [
        b"January",
        b"February",
        b"March",
        b"April",
        b"May",
        b"June",
        b"July",
        b"August",
        b"September",
        b"October",
        b"November",
        b"December",
    ];
    const MONTHS_ABBR: [&[u8]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];

    // ──────────────────────────────────────────────────────────────
    // Extraction helpers (exact, const where possible)
    // ──────────────────────────────────────────────────────────────
    #[inline]
    pub const fn jdn_to_gregorian(jdn: i64) -> (i64, u8, u8) {
        // Use i128 internally to avoid overflow on full i64 JDN range
        let j = jdn as i128;
        let a = j + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (b * 146097) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;
        let day = (e - (153 * m + 2) / 5 + 1) as u8;
        let month = (m + 3 - 12 * (m / 10)) as u8;
        let year = b * 100 + d - 4800 + (m / 10);
        (year as i64, month, day)
    }

    #[inline]
    pub const fn to_gregorian_date(self) -> (i64, u8, u8) {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let jdn = if frac.sec >= 43200 {
            jd_days + 1
        } else {
            jd_days
        };
        Self::jdn_to_gregorian(jdn)
    }

    #[inline]
    pub const fn weekday(self) -> u8 {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let jdn = if frac.sec >= 43200 {
            jd_days + 1
        } else {
            jd_days
        };
        Self::jdn_to_weekday(jdn)
    }

    #[inline]
    pub const fn to_hms_subsec(self) -> (u8, u8, u8, u64) {
        let tt = self.to_clock_type(ClockType::TT);
        let (_, frac) = tt.to_jd_tt_exact();
        let seconds_since_midnight = if frac.sec >= 43200 {
            frac.sec - 43200
        } else {
            frac.sec + 43200
        };
        let hour = (seconds_since_midnight / 3600) as u8;
        let minute = ((seconds_since_midnight % 3600) / 60) as u8;
        let second = (seconds_since_midnight % 60) as u8;
        (hour, minute, second, frac.subsec)
    }

    #[inline]
    pub const fn day_of_year(self) -> u16 {
        let (year, month, day) = self.to_gregorian_date();
        let jdn = Self::gregorian_jdn(year, month, day);
        let jdn_jan1 = Self::gregorian_jdn(year, 1, 1);
        (jdn - jdn_jan1 + 1) as u16
    }

    pub const fn to_iso_week_date(self) -> (i64, u8, Weekday) {
        let (year, month, day) = self.to_gregorian_date();
        let jdn = Self::gregorian_jdn(year, month, day);
        let wd = Self::jdn_to_weekday(jdn);
        let wd_iso = if wd == 0 { 7 } else { wd };

        let jan4_jdn = Self::gregorian_jdn(year, 1, 4);
        let wd_jan4 = Self::jdn_to_weekday(jan4_jdn);
        let days_to_monday = (wd_jan4 + 6) % 7;
        let monday_week1 = jan4_jdn - (days_to_monday as i64);

        let days_since = jdn - monday_week1;

        let week = if days_since < 0 {
            0u8
        } else {
            ((days_since / 7) + 1) as u8
        };

        let iso_year = if week == 0 {
            year - 1
        } else if (week == 53 || week > 53) && !Self::has_iso_week_53(year) {
            year + 1
        } else {
            year
        };

        let iso_week = if week == 0 {
            if Self::has_iso_week_53(year - 1) {
                53
            } else {
                52
            }
        } else if week == 53 && !Self::has_iso_week_53(year) {
            1
        } else if week > 53 {
            1
        } else {
            week
        };

        let weekday_enum = match wd_iso {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => Weekday::Sunday,
        };

        (iso_year, iso_week, weekday_enum)
    }

    // ──────────────────────────────────────────────────────────────
    // Low-level no-alloc write helpers (fixed 512-byte buffer)
    // ──────────────────────────────────────────────────────────────
    const BUFFER_SIZE: usize = 512;

    #[inline]
    fn write_bytes(buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize, bytes: &[u8]) {
        let len = bytes.len();
        if *pos + len > Self::BUFFER_SIZE {
            return;
        }
        buf[*pos..*pos + len].copy_from_slice(bytes);
        *pos += len;
    }

    fn write_u32_padded(
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        mut value: u32,
        flag: u8,
        width: Option<u8>,
        default_pad: u8,
    ) {
        let w = width.unwrap_or(2) as usize;

        // ── strftime semantics ─────────────────────────────────────
        // -  = no padding (minimal width)
        // 0  = zero-pad on the left
        // _  = space-pad on the left
        // (no flag) = use the caller's default_pad (usually '0')
        let pad_char = match flag {
            b'0' => b'0',
            b'_' => b' ',
            _ => default_pad,
        };
        let pad_left = flag != b'-';

        let mut digits = [0u8; 20];
        let mut i = 0usize;

        if value == 0 {
            digits[0] = b'0';
            i = 1;
        } else {
            while value > 0 {
                digits[i] = b'0' + (value % 10) as u8;
                value /= 10;
                i += 1;
            }
        }
        let num_digits = i;
        let pad_len = if pad_left && num_digits < w {
            w - num_digits
        } else {
            0
        };

        if *pos + num_digits + pad_len > Self::BUFFER_SIZE {
            return;
        }

        if pad_left {
            for _ in 0..pad_len {
                buf[*pos] = pad_char;
                *pos += 1;
            }
        }

        for j in (0..num_digits).rev() {
            buf[*pos] = digits[j];
            *pos += 1;
        }
        // No right-padding ever (strftime does not do this for date fields)
    }

    #[allow(unused_mut)]
    fn write_i64(mut buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize, value: i64) {
        if value == 0 {
            Self::write_bytes(buf, pos, b"0");
            return;
        }

        let negative = value < 0;
        let mut v = if negative {
            value.wrapping_neg()
        } else {
            value
        };

        let mut digits = [0u8; 20];
        let mut i = 0usize;
        while v > 0 {
            digits[i] = b'0' + (v % 10) as u8;
            v /= 10;
            i += 1;
        }

        if negative {
            if *pos >= Self::BUFFER_SIZE {
                return;
            }
            buf[*pos] = b'-';
            *pos += 1;
        }

        if *pos + i > Self::BUFFER_SIZE {
            return;
        }
        for j in (0..i).rev() {
            buf[*pos] = digits[j];
            *pos += 1;
        }
    }

    fn write_i64_padded(
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        value: i64,
        flag: u8,
        width: Option<u8>,
        default_pad: u8,
    ) {
        let w = width.unwrap_or(4) as usize; // %Y and %G default to 4 digits

        let negative = value < 0;
        let abs_val = if negative {
            value.wrapping_neg()
        } else {
            value
        };

        let mut digits = [0u8; 20];
        let mut i = 0usize;

        let mut v = abs_val;
        if v == 0 {
            digits[0] = b'0';
            i = 1;
        } else {
            while v > 0 {
                digits[i] = b'0' + (v % 10) as u8;
                v /= 10;
                i += 1;
            }
        }

        let num_digits = i;
        let pad_char = match flag {
            b'-' => b' ',
            b'0' => b'0',
            b'_' => b' ',
            _ => default_pad,
        };
        let pad_left = flag != b'-';
        let pad_len = if pad_left && num_digits < w {
            w - num_digits
        } else {
            0
        };

        if *pos + (if negative { 1 } else { 0 }) + num_digits + pad_len > Self::BUFFER_SIZE {
            return;
        }

        if negative {
            buf[*pos] = b'-';
            *pos += 1;
        }

        if pad_left {
            for _ in 0..pad_len {
                buf[*pos] = pad_char;
                *pos += 1;
            }
        }

        for j in (0..num_digits).rev() {
            buf[*pos] = digits[j];
            *pos += 1;
        }
    }

    fn write_fractional(
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        subsec: u64,
        width: Option<u8>,
    ) {
        let w = width.unwrap_or(18).min(18) as usize;
        if w == 0 {
            return;
        }

        let mut n = subsec;
        let mut digits = [b'0'; 18];
        for i in (0..18).rev() {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        Self::write_bytes(buf, pos, &digits[0..w]);
    }

    // ──────────────────────────────────────────────────────────────
    // Individual write_ functions – one per parser directive
    // ──────────────────────────────────────────────────────────────

    pub(crate) fn write_weekday_full(&self, buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize) {
        let name = Self::WEEKDAYS_FULL[self.weekday() as usize];
        Self::write_bytes(buf, pos, name);
    }

    pub(crate) fn write_weekday_abbrev(&self, buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize) {
        let name = Self::WEEKDAYS_ABBR[self.weekday() as usize];
        Self::write_bytes(buf, pos, name);
    }

    pub(crate) fn write_month_name_full(&self, buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize) {
        let (_, month, _) = self.to_gregorian_date();
        let name = Self::MONTHS_FULL[(month as usize) - 1];
        Self::write_bytes(buf, pos, name);
    }

    pub(crate) fn write_month_name_abbrev(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
    ) {
        let (_, month, _) = self.to_gregorian_date();
        let name = Self::MONTHS_ABBR[(month as usize) - 1];
        Self::write_bytes(buf, pos, name);
    }

    pub(crate) fn write_century(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        _colons: u8,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        // Floor division → -123 becomes -2 (exactly matches parse_century)
        let century = year.div_euclid(100);
        Self::write_i64(buf, pos, century);
    }

    pub(crate) fn write_day_of_month(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let (_, _, day) = self.to_gregorian_date();
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, day as u32, flag, width, default_pad);
    }

    pub(crate) fn write_fractional_seconds(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        _flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (_, _, _, subsec) = self.to_hms_subsec();
        Self::write_fractional(buf, pos, subsec, width);
    }

    pub(crate) fn write_iso_week_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (iso_year, _, _) = self.to_iso_week_date();
        Self::write_i64_padded(buf, pos, iso_year, flag, width, b'0');
    }

    pub(crate) fn write_two_digit_iso_week_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (iso_year, _, _) = self.to_iso_week_date();
        let yy = (iso_year % 100).abs() as u32;
        Self::write_u32_padded(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_hour24(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let (hour, _, _, _) = self.to_hms_subsec();
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, hour as u32, flag, width, default_pad);
    }

    pub(crate) fn write_hour12(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (hour24, _, _, _) = self.to_hms_subsec();
        let hour12 = if hour24 == 0 {
            12
        } else if hour24 > 12 {
            hour24 - 12
        } else {
            hour24
        };
        Self::write_u32_padded(buf, pos, hour12 as u32, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_day_of_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let doy = self.day_of_year();
        Self::write_u32_padded(buf, pos, doy as u32, flag, width.or(Some(3)), b'0');
    }

    pub(crate) fn write_minute(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let (_, minute, _, _) = self.to_hms_subsec();
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, minute as u32, flag, width, default_pad);
    }

    pub(crate) fn write_month_number(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let (_, month, _) = self.to_gregorian_date();
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, month as u32, flag, width, default_pad);
    }

    pub(crate) fn write_whitespace(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        ch: u8,
    ) {
        let bytes = if ch == b'n' { b"\n" } else { b"\t" };
        Self::write_bytes(buf, pos, bytes);
    }

    pub(crate) fn write_ampm(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        upper: bool,
    ) {
        let (hour, _, _, _) = self.to_hms_subsec();
        let bytes = if hour < 12 {
            if upper { b"AM" } else { b"am" }
        } else if upper {
            b"PM"
        } else {
            b"pm"
        };
        Self::write_bytes(buf, pos, bytes);
    }

    pub(crate) fn write_iana_or_offset(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        colons: u8,
        offset: UtcOffset,
    ) {
        // For now we treat %Q as "UTC or numeric offset" (common pattern).
        // When you add full IANA/tzdb support later, just replace the zero case.
        if offset == UtcOffset::ZERO {
            Self::write_bytes(buf, pos, b"UTC");
        } else {
            self.write_timezone_offset(buf, pos, flag, width, colons, offset);
        }
    }

    pub(crate) fn write_second(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let (_, _, second, _) = self.to_hms_subsec();
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, second as u32, flag, width, default_pad);
    }

    pub(crate) fn write_unix_timestamp(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        _colons: u8,
    ) {
        let utc = self.to_clock_type(ClockType::UTC);
        let (jd_days, frac) = utc.to_jd_tt_exact();
        let unix_days = jd_days - 2440587i64;
        let seconds = unix_days * 86400i64 + (frac.sec as i64) - 43200i64;
        Self::write_i64(buf, pos, seconds);
    }

    pub(crate) fn write_week_number_sunday_based(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        let jdn_jan1 = Self::gregorian_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jdn_jan1);
        let days_to_first_sunday = (7 - wd_jan1) % 7;
        let first_sunday_jdn = jdn_jan1 + days_to_first_sunday as i64;
        let current_jdn =
            Self::gregorian_jdn(year, self.to_gregorian_date().1, self.to_gregorian_date().2);
        let days_since = current_jdn - first_sunday_jdn;
        let week = if days_since < 0 {
            0u8
        } else {
            ((days_since / 7) + 1) as u8
        };
        Self::write_u32_padded(buf, pos, week as u32, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_weekday_number_monday_based(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let wd = self.weekday();
        let monday_based = if wd == 0 { 7 } else { wd };
        Self::write_u32_padded(buf, pos, monday_based as u32, flag, width.or(Some(1)), b'0');
    }

    pub(crate) fn write_week_iso(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (_, week, _) = self.to_iso_week_date();
        Self::write_u32_padded(buf, pos, week as u32, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_week_number_monday_based(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        let jdn_jan1 = Self::gregorian_jdn(year, 1, 1);
        let wd_jan1 = Self::jdn_to_weekday(jdn_jan1);
        let days_to_first_monday = (1i64 - wd_jan1 as i64).rem_euclid(7);
        let first_monday_jdn = jdn_jan1 + days_to_first_monday;
        let current_jdn =
            Self::gregorian_jdn(year, self.to_gregorian_date().1, self.to_gregorian_date().2);
        let days_since = current_jdn - first_monday_jdn;
        let week = if days_since < 0 {
            0u8
        } else {
            ((days_since / 7) + 1) as u8
        };
        Self::write_u32_padded(buf, pos, week as u32, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_weekday_number_sunday_based(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let wd = self.weekday();
        Self::write_u32_padded(buf, pos, wd as u32, flag, width.or(Some(1)), b'0');
    }

    pub(crate) fn write_full_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        _pad: bool,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        Self::write_i64_padded(buf, pos, year, flag, width, b'0');
    }

    pub(crate) fn write_two_digit_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        _pad: bool,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        let yy = (year % 100).abs() as u32;
        Self::write_u32_padded(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    pub(crate) fn write_unbounded_year(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let (year, _, _) = self.to_gregorian_date();
        Self::write_i64_padded(buf, pos, year, flag, width, b'0');
    }

    pub(crate) fn write_timezone_offset(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        colons: u8,
        offset: UtcOffset,
    ) {
        let (negative, hours, minutes) = offset.as_hhmm();
        let sign = if negative { b'-' } else { b'+' };

        // seconds component — only used by %::z
        let seconds = ((offset.seconds().abs() % 3600) % 60) as u8;

        match colons {
            0 => {
                // %z     → +HHMM
                let mut tmp = [0u8; 5];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b'0' + minutes / 10;
                tmp[4] = b'0' + minutes % 10;
                Self::write_bytes(buf, pos, &tmp);
            }
            1 => {
                // %:z    → +HH:MM
                let mut tmp = [0u8; 6];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b':';
                tmp[4] = b'0' + minutes / 10;
                tmp[5] = b'0' + minutes % 10;
                Self::write_bytes(buf, pos, &tmp);
            }
            2 => {
                // %::z   → +HH:MM:SS
                let mut tmp = [0u8; 9];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b':';
                tmp[4] = b'0' + minutes / 10;
                tmp[5] = b'0' + minutes % 10;
                tmp[6] = b':';
                tmp[7] = b'0' + seconds / 10;
                tmp[8] = b'0' + seconds % 10;
                Self::write_bytes(buf, pos, &tmp);
            }
            _ => Self::write_bytes(buf, pos, b"+0000"),
        }
    }

    pub(crate) fn write_iso_date(&self, buf: &mut [u8; Self::BUFFER_SIZE], pos: &mut usize) {
        let (y, m, d) = self.to_gregorian_date();
        // Improved: now uses write_i64_padded so negative years are correctly signed and padded
        Self::write_i64_padded(buf, pos, y, b'0', Some(4), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32_padded(buf, pos, m as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32_padded(buf, pos, d as u32, b'0', Some(2), b'0');
    }

    pub(crate) fn write_us_date_shortcut(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
    ) {
        let (y, m, d) = self.to_gregorian_date();
        Self::write_u32_padded(buf, pos, m as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32_padded(buf, pos, d as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32_padded(buf, pos, (y % 100).abs() as u32, b'0', Some(2), b'0');
    }

    pub(crate) fn write_time_with_seconds_shortcut(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
    ) {
        let (h, m, s, _) = self.to_hms_subsec();
        Self::write_u32_padded(buf, pos, h as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, m as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, s as u32, b'0', Some(2), b'0');
    }

    pub(crate) fn write_time_without_seconds_shortcut(
        &self,
        buf: &mut [u8; Self::BUFFER_SIZE],
        pos: &mut usize,
    ) {
        let (h, m, _, _) = self.to_hms_subsec();
        Self::write_u32_padded(buf, pos, h as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, m as u32, b'0', Some(2), b'0');
    }

    pub(crate) fn write_unsupported(&self, _buf: &mut [u8; Self::BUFFER_SIZE], _pos: &mut usize) {
        // no-op (parser already errors)
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
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

    #[test]
    fn test_basic_formatting() {
        let t = tp(2025, 4, 16, 14, 30, 45, 123_456_789_000_000_000);

        let mut buf = [0u8; 512];

        // ISO date + time + fractional (now full attosecond precision)
        let n = t.format_u8("%Y-%m-%d %H:%M:%S.%f", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16 14:30:45.123456789000000000");

        // Shortcuts
        let n = t.format_u8("%F", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16");

        let n = t.format_u8("%T", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"14:30:45");

        let n = t.format_u8("%R", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = tp(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000);

        let mut buf = [0u8; 512];

        // %f and %N now default to 18 attosecond digits
        let n = t.format_u8("%f", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        let n = t.format_u8("%N", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        // Custom width
        let n = t.format_u8("%.3f", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b".123");

        let n = t.format_u8("%.6N", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"123456");
    }

    #[test]
    fn test_iso_week_fix() {
        let mut buf = [0u8; 512];

        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = tp(2000, 1, 1, 12, 0, 0, 0);
        let n = t2000.format_u8("%G-W%V-%u", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = tp(2000, 1, 3, 12, 0, 0, 0);
        let n = t2000_monday.format_u8("%G-W%V-%u", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2000-W01-1");

        // Year with 53 weeks (2015-12-28 is Monday of week 53 of 2015)
        let t_week53 = tp(2015, 12, 28, 12, 0, 0, 0);
        let n = t_week53.format_u8("%G-W%V", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = tp(2025, 4, 16, 14, 30, 45, 0);
        let mut buf = [0u8; 512];

        let utc = UtcOffset::UTC;
        let ny = UtcOffset::from_hms(-5, 0, 0); // New York
        let la = UtcOffset::from_hms(-8, 0, 0); // Los Angeles
        let positive = UtcOffset::from_hms(2, 30, 0);

        // %z with different colon counts
        let n = t.format_u8_with_offset("%z", &mut buf, utc).unwrap();
        assert_eq!(&buf[0..n], b"+0000");

        let n = t.format_u8_with_offset("%:z", &mut buf, ny).unwrap();
        assert_eq!(&buf[0..n], b"-05:00");

        let n = t.format_u8_with_offset("%::z", &mut buf, la).unwrap();
        assert_eq!(&buf[0..n], b"-08:00:00");

        let n = t.format_u8_with_offset("%z", &mut buf, positive).unwrap();
        assert_eq!(&buf[0..n], b"+0230");

        // %Q
        let n = t.format_u8_with_offset("%Q", &mut buf, utc).unwrap();
        assert_eq!(&buf[0..n], b"UTC");

        let n = t.format_u8_with_offset("%Q", &mut buf, ny).unwrap();
        assert_eq!(&buf[0..n], b"-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = tp(2025, 4, 5, 3, 9, 7, 0);
        let mut buf = [0u8; 512];

        // Default zero padding
        let n = t.format_u8("%d %H %M %S", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"05 03 09 07");

        // Space padding
        let n = t.format_u8("%_d %_H", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b" 5  3");

        // No padding
        let n = t.format_u8("%-d %-H", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"5 3");

        // Zero padding explicit
        let n = t.format_u8("%0d %0H", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = tp(2025, 4, 16, 0, 0, 0, 0); // Wednesday
        let mut buf = [0u8; 512];

        let n = t.format_u8("%A, %B %d, %Y", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"Wednesday, April 16, 2025");

        let n = t.format_u8("%a %b %d", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"Wed Apr 16");
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = tp(1970, 1, 1, 0, 0, 0, 0); // Unix epoch
        let mut buf = [0u8; 512];

        let n = t.format_u8("%s", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"0");

        let n = t.format_u8("%j", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        let mut buf = [0u8; 512];

        // ── Negative & zero years ─────────────────────────────────────
        let t_neg = tp(-123, 6, 15, 9, 30, 45, 0);
        let n = t_neg.format_u8("%Y-%m-%d", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"-0123-06-15");

        let n = t_neg.format_u8("%C", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"-2"); // century

        let t_zero = tp(0, 1, 1, 0, 0, 0, 0);
        let n = t_zero.format_u8("%Y", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"0000");

        // ── ISO week year-boundary cases (now fixed correctly) ───────
        // 2024-12-30 (Mon) → belongs to 2025 week 1
        let t_2024_dec30 = tp(2024, 12, 30, 12, 0, 0, 0);
        let n = t_2024_dec30.format_u8("%G-W%V-%u", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-1");

        // 2024-12-31 (Tue) → still 2025-W01-2
        let t_2024_dec31 = tp(2024, 12, 31, 12, 0, 0, 0);
        let n = t_2024_dec31.format_u8("%G-W%V-%u", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-2");

        // 2025-01-01 (Wed) → 2025-W01-3
        let t_2025_jan1 = tp(2025, 1, 1, 12, 0, 0, 0);
        let n = t_2025_jan1.format_u8("%G-W%V-%u", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-3");

        // Year with 53 weeks
        let t_2015_dec28 = tp(2015, 12, 28, 12, 0, 0, 0);
        let n = t_2015_dec28.format_u8("%G-W%V", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");

        // ── Week numbers %U / %W edge cases ───────────────────────────
        // 2000-01-01 was Saturday → %U = 0 (first Sunday is Jan 2)
        let t2000 = tp(2000, 1, 1, 12, 0, 0, 0);
        let n = t2000.format_u8("%U", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"00");

        let n = t2000.format_u8("%W", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"00");

        // 2023-12-31 was Sunday → %U = 53
        let t_sun = tp(2023, 12, 31, 12, 0, 0, 0);
        let n = t_sun.format_u8("%U", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"53");

        // ── Fractional seconds extremes ───────────────────────────────
        let t_frac = tp(2025, 4, 16, 0, 0, 0, 0);
        let n = t_frac.format_u8("%.0f", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b""); // width 0 = nothing

        let n = t_frac.format_u8("%.9N", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"000000000");

        let n = t_frac.format_u8("%S.%f", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"00.000000000000000000");

        // ── Timezone offsets with seconds & different colon counts ─────
        let ny = UtcOffset::from_hms(-5, 0, 0);
        let la = UtcOffset::from_hms(-8, 0, 0);
        let weird = UtcOffset::from_hms(1, 23, 45);

        let n = t_frac.format_u8_with_offset("%::z", &mut buf, ny).unwrap();
        assert_eq!(&buf[0..n], b"-05:00:00");

        let n = t_frac.format_u8_with_offset("%:z", &mut buf, la).unwrap();
        assert_eq!(&buf[0..n], b"-08:00");

        // %::z with seconds component (tests full +HH:MM:SS support)
        let n = t_frac
            .format_u8_with_offset("%::z", &mut buf, weird)
            .unwrap();
        assert_eq!(&buf[0..n], b"+01:23:45");

        // ── Padding + explicit width + flags combined ─────────────────
        let t_small = tp(2025, 4, 5, 3, 9, 7, 0);

        let n = t_small.format_u8("%03d", &mut buf).unwrap(); // explicit width 3, default zero
        assert_eq!(&buf[0..n], b"005");

        let n = t_small.format_u8("%-5H", &mut buf).unwrap(); // left-justify, width 5 → no pad
        assert_eq!(&buf[0..n], b"3");

        // space-pad to width 3 (correct behavior for flag '_')
        let n = t_small.format_u8("%_3M", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"  9"); // two spaces + '9'

        // ── Negative Unix timestamp ───────────────────────────────────
        let t_neg_unix = tp(1969, 12, 31, 23, 59, 59, 0);
        let n = t_neg_unix.format_u8("%s", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"-1");

        // Large positive (well within i64 for %s)
        let t_large = tp(2038, 1, 19, 3, 14, 7, 0);
        let n = t_large.format_u8("%s", &mut buf).unwrap();
        assert_eq!(&buf[0..n], b"2147483647");
    }
}
