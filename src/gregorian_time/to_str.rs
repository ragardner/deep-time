use crate::{
    AsciiStr, DtErrKind, DtError, GregorianTime, MONTHS_ABBR, MONTHS_FULL, STRFTIME_SIZE,
    TimePoint, WEEKDAYS_ABBR, WEEKDAYS_FULL,
};

impl GregorianTime {
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtError> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        self.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..pos]).into_owned())
    }

    /// No-allocation formatting.
    #[inline]
    pub fn to_ascii_str(&self, fmt: &str) -> Result<AsciiStr<STRFTIME_SIZE>, DtError> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        self.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    pub(crate) fn format_to_buffer(
        &self,
        fmt: &[u8],
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
    ) -> Result<(), DtError> {
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
                return Err(DtErrKind::UnexpectedEndAfterPercent.into());
            }

            // %% → literal percent
            if fmt[i] == b'%' {
                Self::write_bytes(buf, pos, b"%");
                i += 1;
                continue;
            }

            // ── Parse optional flags (- 0 _ ~) ───────────────────────
            // ~ means "trim trailing zeros" (only affects %f / %N fractional seconds)
            let mut flag = b'0'; // temporary default; many directives override it via pad param
            let mut trim_trailing = false;
            while i < fmt.len() {
                match fmt[i] {
                    b'-' | b'0' | b'_' => {
                        flag = fmt[i];
                        i += 1;
                    }
                    b'~' => {
                        trim_trailing = true;
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
                return Err(DtErrKind::UnexpectedEndAfterPercent.into());
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
                    return Err(DtErrKind::ExpectedFOrNAfterDot.into());
                }

                // optional ~ for trim trailing zeros, after width e.g. %.3~f or %.~f
                if fmt[i] == b'~' {
                    trim_trailing = true;
                    i += 1;
                }

                if i >= fmt.len() {
                    return Err(DtErrKind::ExpectedFOrNAfterDot.into());
                }

                let next = fmt[i];
                i += 1;

                if matches!(next, b'f' | b'N') {
                    // Only print the dot for %f when width > 0.
                    // When trim_trailing (~) is used and the fractional part is zero
                    // after trimming, we suppress the dot entirely. This gives clean
                    // RFC 3339 / ISO 8601 output (no ".000..." for integer seconds).
                    let width_val = frac_width.unwrap_or(18);
                    let add_dot = (next == b'f') && (width_val > 0);

                    let dot_pos = if add_dot {
                        let p = *pos;
                        Self::write_bytes(buf, pos, b".");
                        Some(p)
                    } else {
                        None
                    };

                    let wrote_frac = self.write_fractional_seconds(
                        buf,
                        pos,
                        flag,
                        frac_width,
                        colons,
                        trim_trailing,
                    );

                    if add_dot && !wrote_frac {
                        // Nothing significant was written → remove the dot
                        if let Some(p) = dot_pos {
                            *pos = p;
                        }
                    }
                    continue;
                } else {
                    return Err(DtErrKind::ExpectedFOrNAfterDot.into());
                }
            }

            // ── Normal directives ──
            match directive {
                b'A' => self.write_weekday_full(buf, pos),
                b'a' => self.write_weekday_abbrev(buf, pos),
                b'B' => self.write_month_name_full(buf, pos),
                b'b' | b'h' => self.write_month_name_abbrev(buf, pos),
                b'C' => self.write_century(buf, pos, flag, width, colons),
                b'd' | b'e' => self.write_day_of_month(buf, pos, flag, width, colons, true),
                b'f' | b'N' => {
                    let _ =
                        self.write_fractional_seconds(buf, pos, flag, width, colons, trim_trailing);
                }
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
                b'Q' => self.write_iana(buf, pos),
                b'S' => self.write_second(buf, pos, flag, width, colons, true),
                b's' => self.write_unix_timestamp(buf, pos, flag, width, colons),
                b'U' => self.write_week_number_sunday_based(buf, pos, flag, width, colons),
                b'u' => self.write_weekday_number_monday_based(buf, pos, flag, width, colons),
                b'V' => self.write_week_iso(buf, pos, flag, width, colons),
                b'W' => self.write_week_number_monday_based(buf, pos, flag, width, colons),
                b'w' => self.write_weekday_number_sunday_based(buf, pos, flag, width, colons),
                b'Y' => self.write_full_year(buf, pos, flag, width, colons, true),
                b'y' => self.write_two_digit_year(buf, pos, flag, width, colons, true),
                b'z' => self.write_timezone_offset(buf, pos, flag, width, colons),
                b'F' => self.write_iso_date(buf, pos),
                b'D' => self.write_us_date_shortcut(buf, pos),
                b'T' => self.write_time_with_seconds_shortcut(buf, pos),
                b'R' => self.write_time_without_seconds_shortcut(buf, pos),
                b'Z' => self.write_timezone_abbrev(buf, pos),

                // Library directives
                b'*' => self.write_unbounded_year(buf, pos, flag, width, colons),
                b'L' => self.write_clock_type(buf, pos),

                b'c' | b'r' | b'X' | b'x' => self.write_unsupported(buf, pos),
                _ => return Err(DtErrKind::UnknownFormatDirective.into()),
            }
        }

        Ok(())
    }

    fn write_bytes(buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, bytes: &[u8]) {
        let len = bytes.len();
        if *pos + len > STRFTIME_SIZE {
            return;
        }
        buf[*pos..*pos + len].copy_from_slice(bytes);
        *pos += len;
    }

    fn write_u32_padded(
        buf: &mut [u8; STRFTIME_SIZE],
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

        if *pos + num_digits + pad_len > STRFTIME_SIZE {
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
    fn write_i64(mut buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, value: i64) {
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
            if *pos >= STRFTIME_SIZE {
                return;
            }
            buf[*pos] = b'-';
            *pos += 1;
        }

        if *pos + i > STRFTIME_SIZE {
            return;
        }
        for j in (0..i).rev() {
            buf[*pos] = digits[j];
            *pos += 1;
        }
    }

    fn write_i64_padded(
        buf: &mut [u8; STRFTIME_SIZE],
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

        if *pos + (if negative { 1 } else { 0 }) + num_digits + pad_len > STRFTIME_SIZE {
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
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        subsec: u64,
        width: Option<u8>,
        trim: bool,
    ) -> bool {
        let w = width.unwrap_or(18).min(18) as usize;
        if w == 0 {
            return false;
        }

        let mut n = subsec;
        let mut digits = [b'0'; 18];
        for i in (0..18).rev() {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }

        let mut end = w;
        if trim {
            // Trim trailing zeros from the least-significant end of the selected width.
            // If everything is zero after trimming, return false so the caller
            // can suppress the decimal point entirely (perfect for RFC 3339 / ISO 8601).
            while end > 0 && digits[end - 1] == b'0' {
                end -= 1;
            }
            if end == 0 {
                return false; // emit nothing at all (no dot, no "0")
            }
        }
        Self::write_bytes(buf, pos, &digits[0..end]);
        true
    }

    // ──────────────────────────────────────────────────────────────
    // Individual write_ functions – one per parser directive
    // ──────────────────────────────────────────────────────────────

    #[inline]
    pub(crate) fn write_weekday_full(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        let name = WEEKDAYS_FULL[self.wkday as usize];
        Self::write_bytes(buf, pos, name);
    }

    #[inline]
    pub(crate) fn write_weekday_abbrev(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        let name = WEEKDAYS_ABBR[self.wkday as usize];
        Self::write_bytes(buf, pos, name);
    }

    #[inline]
    pub(crate) fn write_month_name_full(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        let name = MONTHS_FULL[self.mo as usize - 1];
        Self::write_bytes(buf, pos, name);
    }

    #[inline]
    pub(crate) fn write_month_name_abbrev(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        let name = MONTHS_ABBR[self.mo as usize - 1];
        Self::write_bytes(buf, pos, name);
    }

    #[inline]
    pub(crate) fn write_century(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        _colons: u8,
    ) {
        // Floor division → -123 becomes -2 (exactly matches parse_century)
        let century = self.yr.div_euclid(100);
        Self::write_i64(buf, pos, century);
    }

    #[inline]
    pub(crate) fn write_day_of_month(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, self.day as u32, flag, width, default_pad);
    }

    #[inline]
    pub(crate) fn write_fractional_seconds(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        _flag: u8,
        width: Option<u8>,
        _colons: u8,
        trim: bool,
    ) -> bool {
        Self::write_fractional(buf, pos, self.attos, width, trim)
    }

    #[inline]
    pub(crate) fn write_iso_week_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_i64_padded(buf, pos, self.iso_yr, flag, width, b'0');
    }

    #[inline]
    pub(crate) fn write_two_digit_iso_week_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let yy = (self.iso_yr % 100).abs() as u32;
        Self::write_u32_padded(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    #[inline]
    pub(crate) fn write_hour24(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, self.hr as u32, flag, width, default_pad);
    }

    #[inline]
    pub(crate) fn write_hour12(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        let hour24 = self.hr;
        let hour12 = if hour24 == 0 {
            12
        } else if hour24 > 12 {
            hour24 - 12
        } else {
            hour24
        };
        Self::write_u32_padded(buf, pos, hour12 as u32, flag, width.or(Some(2)), b'0');
    }

    #[inline]
    pub(crate) fn write_day_of_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(
            buf,
            pos,
            self.day_of_yr as u32,
            flag,
            width.or(Some(3)),
            b'0',
        );
    }

    #[inline]
    pub(crate) fn write_minute(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, self.min as u32, flag, width, default_pad);
    }

    #[inline]
    pub(crate) fn write_month_number(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, self.mo as u32, flag, width, default_pad);
    }

    #[inline]
    pub(crate) fn write_whitespace(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, ch: u8) {
        let bytes = if ch == b'n' { b"\n" } else { b"\t" };
        Self::write_bytes(buf, pos, bytes);
    }

    #[inline]
    pub(crate) fn write_ampm(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, upper: bool) {
        let hour = self.hr;
        let bytes = if hour < 12 {
            if upper { b"AM" } else { b"am" }
        } else if upper {
            b"PM"
        } else {
            b"pm"
        };
        Self::write_bytes(buf, pos, bytes);
    }

    #[inline]
    pub(crate) fn write_second(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32_padded(buf, pos, self.sec as u32, flag, width, default_pad);
    }

    #[inline]
    pub(crate) fn write_unix_timestamp(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        _colons: u8,
    ) {
        let (seconds, _) = self.unix_timestamp();
        Self::write_i64(buf, pos, seconds);
    }

    #[inline]
    pub(crate) fn write_weekday_number_sunday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(
            buf,
            pos,
            self.wkday_sun() as u32,
            flag,
            width.or(Some(1)),
            b'0',
        );
    }

    #[inline]
    pub(crate) fn write_weekday_number_monday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(
            buf,
            pos,
            self.wkday_mon() as u32,
            flag,
            width.or(Some(1)),
            b'0',
        );
    }

    #[inline]
    pub(crate) fn write_week_iso(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(buf, pos, self.iso_wk as u32, flag, width.or(Some(2)), b'0');
    }

    #[inline]
    pub(crate) fn write_week_number_sunday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(
            buf,
            pos,
            self.wk_of_yr_sun as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline]
    pub(crate) fn write_week_number_monday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_u32_padded(
            buf,
            pos,
            self.wk_of_yr_mon as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline]
    pub(crate) fn write_full_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        _pad: bool,
    ) {
        Self::write_i64_padded(buf, pos, self.yr, flag, width, b'0');
    }

    #[inline]
    pub(crate) fn write_two_digit_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
        _pad: bool,
    ) {
        let yy = (self.yr % 100).abs() as u32;
        Self::write_u32_padded(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    #[inline]
    pub(crate) fn write_unbounded_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        _colons: u8,
    ) {
        Self::write_i64_padded(buf, pos, self.yr, flag, width, b'0');
    }

    #[inline]
    pub(crate) fn write_iana(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        if let Some(iana) = self.tz() {
            Self::write_bytes(buf, pos, iana.as_bytes());
        } else if let Some(abbrev) = self.tz_abbrev() {
            Self::write_bytes(buf, pos, abbrev.as_bytes());
        } else {
            Self::write_bytes(buf, pos, "UTC".as_bytes());
        }
    }

    #[inline]
    pub(crate) fn write_clock_type(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_bytes(buf, pos, self.clock_type().abbrev().as_bytes());
    }

    pub(crate) fn write_timezone_offset(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        _flag: u8,
        _width: Option<u8>,
        colons: u8,
    ) {
        let Some(offset_sec) = self.offset_sec() else {
            return;
        };
        let (negative, hours, minutes) = TimePoint::sec_as_hhmm(offset_sec);
        let sign = if negative { b'-' } else { b'+' };

        // seconds component — only used by %::z
        let seconds = ((offset_sec.abs() % 3600) % 60) as u8;

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

    #[inline]
    pub(crate) fn write_timezone_abbrev(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        if let Some(abbrev) = self.tz_abbrev() {
            Self::write_bytes(buf, pos, abbrev.as_bytes());
        } else {
            Self::write_bytes(buf, pos, b"UTC");
        }
    }

    #[inline]
    pub(crate) fn write_iso_date(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_i64_padded(buf, pos, self.yr, b'0', Some(4), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32_padded(buf, pos, self.mo as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32_padded(buf, pos, self.day as u32, b'0', Some(2), b'0');
    }

    #[inline]
    pub(crate) fn write_us_date_shortcut(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_u32_padded(buf, pos, self.mo as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32_padded(buf, pos, self.day as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32_padded(buf, pos, (self.yr % 100).abs() as u32, b'0', Some(2), b'0');
    }

    #[inline]
    pub(crate) fn write_time_with_seconds_shortcut(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
    ) {
        Self::write_u32_padded(buf, pos, self.hr as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, self.min as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, self.sec as u32, b'0', Some(2), b'0');
    }

    #[inline]
    pub(crate) fn write_time_without_seconds_shortcut(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
    ) {
        Self::write_u32_padded(buf, pos, self.hr as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32_padded(buf, pos, self.min as u32, b'0', Some(2), b'0');
    }

    #[inline]
    pub(crate) fn write_unsupported(&self, _buf: &mut [u8; STRFTIME_SIZE], _pos: &mut usize) {
        // no-op (parser already errors)
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use crate::ClockType;

    // Helper to create a TimePoint at the requested civil UTC time.
    // Now matches GregorianTime::to_time_point exactly (UTC civil seconds).
    fn tp(y: i64, m: u8, d: u8, h: u8, min: u8, s: u8, attos: u64) -> TimePoint {
        let jdn = TimePoint::ymd_to_jdn(y, m, d);
        let days_since_j2000 = jdn - 2451545i64; // J2000_JD_TT
        let seconds_from_noon = (h as i64 - 12) * 3600 + (min as i64) * 60 + (s as i64);
        let sec = days_since_j2000 * 86400i64 + seconds_from_noon;
        TimePoint::new(sec, attos, ClockType::UTC)
    }

    #[test]
    fn test_basic_formatting() {
        let t = tp(2025, 4, 16, 14, 30, 45, 123_456_789_000_000_000);

        let mut buf = [0u8; STRFTIME_SIZE];

        // ISO date + time + fractional (now full attosecond precision)
        let n = t
            .to_u8_with_offset("%Y-%m-%d %H:%M:%S.%f", &mut buf, 0) // 0 = UTC
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16 14:30:45.123456789000000000");

        // Shortcuts
        let n = t.to_u8_with_offset("%F", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2025-04-16");

        let n = t.to_u8_with_offset("%T", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"14:30:45");

        let n = t.to_u8_with_offset("%R", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"14:30");
    }

    #[test]
    fn test_fractional_seconds_fix() {
        let t = tp(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000);

        let mut buf = [0u8; STRFTIME_SIZE];

        // %f and %N now default to 18 attosecond digits
        let n = t.to_u8_with_offset("%f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        let n = t.to_u8_with_offset("%N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456789000000000");

        // Custom width
        let n = t.to_u8_with_offset("%.3f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123");

        let n = t.to_u8_with_offset("%.6N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"123456");
    }

    #[test]
    fn test_iso_week_fix() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // 2000-01-01 was Saturday → belongs to 1999 week 52
        let t2000 = tp(2000, 1, 1, 12, 0, 0, 0);
        let n = t2000.to_u8_with_offset("%G-W%V-%u", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"1999-W52-6");

        // 2000-01-03 is Monday of week 1 of 2000
        let t2000_monday = tp(2000, 1, 3, 12, 0, 0, 0);
        let n = t2000_monday
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2000-W01-1");

        // Year with 53 weeks (2015-12-28 is Monday of week 53 of 2015)
        let t_week53 = tp(2015, 12, 28, 12, 0, 0, 0);
        let n = t_week53.to_u8_with_offset("%G-W%V", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");
    }

    #[test]
    fn test_timezone_offset() {
        let t = tp(2025, 4, 16, 14, 30, 45, 0);
        let mut buf = [0u8; STRFTIME_SIZE];

        // %z with different colon counts
        let n = t.to_u8_with_offset("%z", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"+0000");

        let n = t.to_u8_with_offset("%:z", &mut buf, -5 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-05:00");

        let n = t.to_u8_with_offset("%::z", &mut buf, -8 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-08:00:00");

        let n = t
            .to_u8_with_offset("%z", &mut buf, 2 * 3600 + 30 * 60)
            .unwrap();
        assert_eq!(&buf[0..n], b"+0230");

        // %Q
        let n = t.to_u8_with_offset("%Q", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"UTC");

        let n = t.to_u8_with_offset("%z", &mut buf, -5 * 3600).unwrap();
        assert_eq!(&buf[0..n], b"-0500");
    }

    #[test]
    fn test_padding_and_flags() {
        let t = tp(2025, 4, 5, 3, 9, 7, 0);
        let mut buf = [0u8; STRFTIME_SIZE];

        // Default zero padding
        let n = t.to_u8_with_offset("%d %H %M %S", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"05 03 09 07");

        // Space padding
        let n = t.to_u8_with_offset("%_d %_H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b" 5  3");

        // No padding
        let n = t.to_u8_with_offset("%-d %-H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"5 3");

        // Zero padding explicit
        let n = t.to_u8_with_offset("%0d %0H", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"05 03");
    }

    #[test]
    fn test_weekday_and_month_names() {
        let t = tp(2025, 4, 16, 0, 0, 0, 0); // Wednesday
        let mut buf = [0u8; STRFTIME_SIZE];

        let n = t.to_u8_with_offset("%A, %B %d, %Y", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"Wednesday, April 16, 2025");

        let n = t.to_u8_with_offset("%a %b %d", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"Wed Apr 16");
    }

    #[test]
    fn test_unix_timestamp_and_day_of_year() {
        let t = tp(1970, 1, 1, 0, 0, 0, 0); // Unix epoch
        let mut buf = [0u8; STRFTIME_SIZE];

        let n = t.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0");

        let n = t.to_u8_with_offset("%j", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"001");
    }

    #[test]
    fn test_edge_cases_roundtrip_and_extreme_values() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // ── Negative & zero years ─────────────────────────────────────
        let t_neg = tp(-123, 6, 15, 9, 30, 45, 0);
        let n = t_neg.to_u8_with_offset("%Y-%m-%d", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-0123-06-15");

        let n = t_neg.to_u8_with_offset("%C", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-2"); // century

        let t_zero = tp(0, 1, 1, 0, 0, 0, 0);
        let n = t_zero.to_u8_with_offset("%Y", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0000");

        // ── ISO week year-boundary cases (now fixed correctly) ───────
        // 2024-12-30 (Mon) → belongs to 2025 week 1
        let t_2024_dec30 = tp(2024, 12, 30, 12, 0, 0, 0);
        let n = t_2024_dec30
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-1");

        // 2024-12-31 (Tue) → still 2025-W01-2
        let t_2024_dec31 = tp(2024, 12, 31, 12, 0, 0, 0);
        let n = t_2024_dec31
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-2");

        // 2025-01-01 (Wed) → 2025-W01-3
        let t_2025_jan1 = tp(2025, 1, 1, 12, 0, 0, 0);
        let n = t_2025_jan1
            .to_u8_with_offset("%G-W%V-%u", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2025-W01-3");

        // Year with 53 weeks
        let t_2015_dec28 = tp(2015, 12, 28, 12, 0, 0, 0);
        let n = t_2015_dec28
            .to_u8_with_offset("%G-W%V", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"2015-W53");

        // ── Week numbers %U / %W edge cases ───────────────────────────
        // 2000-01-01 was Saturday → %U = 0 (first Sunday is Jan 2)
        let t2000 = tp(2000, 1, 1, 12, 0, 0, 0);
        let n = t2000.to_u8_with_offset("%U", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00");

        let n = t2000.to_u8_with_offset("%W", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00");

        // 2023-12-31 was Sunday → %U = 53
        let t_sun = tp(2023, 12, 31, 12, 0, 0, 0);
        let n = t_sun.to_u8_with_offset("%U", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"53");

        // ── Fractional seconds extremes ───────────────────────────────
        let t_frac = tp(2025, 4, 16, 0, 0, 0, 0);
        let n = t_frac.to_u8_with_offset("%.0f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // width 0 = nothing

        let n = t_frac.to_u8_with_offset("%.9N", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"000000000");

        let n = t_frac.to_u8_with_offset("%S.%f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"00.000000000000000000");

        // ── Timezone offsets with seconds & different colon counts ─────
        let ny = -5 * 3600;
        let la = -8 * 3600;
        let weird = 1 * 3600 + 23 * 60 + 45;

        let n = t_frac.to_u8_with_offset("%::z", &mut buf, ny).unwrap();
        assert_eq!(&buf[0..n], b"-05:00:00");

        let n = t_frac.to_u8_with_offset("%:z", &mut buf, la).unwrap();
        assert_eq!(&buf[0..n], b"-08:00");

        // %::z with seconds component (tests full +HH:MM:SS support)
        let n = t_frac.to_u8_with_offset("%::z", &mut buf, weird).unwrap();
        assert_eq!(&buf[0..n], b"+01:23:45");

        // ── Padding + explicit width + flags combined ─────────────────
        let t_small = tp(2025, 4, 5, 3, 9, 7, 0);

        let n = t_small.to_u8_with_offset("%03d", &mut buf, 0).unwrap(); // explicit width 3, default zero
        assert_eq!(&buf[0..n], b"005");

        let n = t_small.to_u8_with_offset("%-5H", &mut buf, 0).unwrap(); // left-justify, width 5 → no pad
        assert_eq!(&buf[0..n], b"3");

        // space-pad to width 3 (correct behavior for flag '_')
        let n = t_small.to_u8_with_offset("%_3M", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"  9"); // two spaces + '9'

        // ── Negative Unix timestamp ───────────────────────────────────
        let t_neg_unix = tp(1969, 12, 31, 23, 59, 59, 0);
        let n = t_neg_unix.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"-1");

        // Large positive (well within i64 for %s)
        let t_large = tp(2038, 1, 19, 3, 14, 7, 0);
        let n = t_large.to_u8_with_offset("%s", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"2147483647");
    }

    #[test]
    fn test_fractional_trim_flag() {
        let mut buf = [0u8; STRFTIME_SIZE];

        // Value with trailing zeros in fractional part
        let t = tp(2025, 4, 16, 0, 0, 0, 123_456_789_000_000_000);

        // %.~f should trim all trailing zeros
        let n = t.to_u8_with_offset("%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789");

        // %.9~f should trim to 9 significant digits
        let n = t.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789");

        // %.18~f trims trailing zeros (this is the intended behavior of ~)
        let n = t.to_u8_with_offset("%.18~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123456789"); // trimmed to significant digits

        // Value that becomes all zeros after trimming
        let t_zero = tp(2025, 4, 16, 0, 0, 0, 0);
        let n = t_zero.to_u8_with_offset("%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // no dot, no "0"

        let n = t_zero.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b""); // still nothing

        // Without ~ it should NOT trim (keeps trailing zeros)
        let t_trailing = tp(2025, 4, 16, 0, 0, 0, 123_000_000_000_000_000);
        let n = t_trailing.to_u8_with_offset("%.9f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123000000"); // keeps trailing zeros without ~

        let n = t_trailing.to_u8_with_offset("%.9~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b".123"); // trims with ~

        // %.0~f should always be empty
        let n = t.to_u8_with_offset("%.0~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"");

        // ── Negative years + fractional trim ─────────────────────────────
        let t_neg = tp(-123, 6, 15, 9, 30, 45, 123_456_789_000_000_000);
        let n = t_neg
            .to_u8_with_offset("%Y-%m-%dT%H:%M:%S%.~fZ", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-0123-06-15T09:30:45.123456789Z");

        // Negative year with all-zero fractional after trim
        let t_neg_zero = tp(-1, 1, 1, 0, 0, 0, 0);
        let n = t_neg_zero
            .to_u8_with_offset("%Y-%.~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-0001-"); // no fractional part at all

        // Year 0 with fractional
        let t_year0 = tp(0, 1, 1, 0, 0, 0, 500_000_000_000_000_000);
        let n = t_year0.to_u8_with_offset("%Y%.~f", &mut buf, 0).unwrap();
        assert_eq!(&buf[0..n], b"0000.5");

        // ── Long years (6 digits) + fractional ───────────────────────────
        let t_long_year = tp(123456, 7, 4, 12, 0, 0, 987654321987654321);
        let n = t_long_year
            .to_u8_with_offset("%Y-%m-%dT%H:%M:%S%.~fZ", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"123456-07-04T12:00:00.987654321987654321Z");

        let t_long_neg_year = tp(-100000, 12, 31, 23, 59, 59, 111111111111111111);
        let n = t_long_neg_year
            .to_u8_with_offset("%Y-%.~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b"-100000-.111111111111111111");

        // ── 18-digit attos with NO trailing zeros (with and without ~) ───
        let t_full_attos = tp(2025, 4, 16, 0, 0, 0, 123456789012345678);

        // With ~ (should still output all 18 digits since no trailing zeros)
        let n = t_full_attos
            .to_u8_with_offset("%.18~f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b".123456789012345678");

        // Without ~ (same result)
        let n = t_full_attos
            .to_u8_with_offset("%.18f", &mut buf, 0)
            .unwrap();
        assert_eq!(&buf[0..n], b".123456789012345678");
    }
}
