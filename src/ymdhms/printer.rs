use crate::{Dt, DtErr, DtErrKind, Lang, LiteStr, STRFTIME_SIZE, Scale, YmdHms, an_err};

impl YmdHms {
    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn to_str(
        &self,
        fmt: &str,
        offset: Option<i32>,
        tz: Option<LiteStr<49>>,
        abbrev: Option<LiteStr<49>>,
        lang: Lang,
    ) -> Result<alloc::string::String, DtErr> {
        let (buf, pos) = self.format_to_buffer(fmt.as_bytes(), offset, tz, abbrev, lang)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..pos]).into_owned())
    }

    #[inline]
    pub(crate) fn to_str_lite(
        &self,
        fmt: &str,
        offset: Option<i32>,
        tz: Option<LiteStr<49>>,
        abbrev: Option<LiteStr<49>>,
        lang: Lang,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let (buf, pos) = self.format_to_buffer(fmt.as_bytes(), offset, tz, abbrev, lang)?;
        Ok(LiteStr::from_bytes(&buf[..pos]))
    }

    pub(crate) fn format_to_buffer(
        &self,
        fmt: &[u8],
        offset: Option<i32>,
        tz: Option<LiteStr<49>>,
        abbrev: Option<LiteStr<49>>,
        lang: Lang,
    ) -> Result<([u8; STRFTIME_SIZE], usize), DtErr> {
        let names = lang.names();
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        let mut i = 0usize;

        while i < fmt.len() {
            let byte = fmt[i];

            if byte != b'%' {
                Self::write_byte(&mut buf, &mut pos, byte);
                i += 1;
                continue;
            }

            i += 1; // skip '%'

            if i >= fmt.len() {
                return Err(an_err!(DtErrKind::UnexpectedEnd, "after %"));
            }

            // %% → literal percent
            if fmt[i] == b'%' {
                Self::write_byte(&mut buf, &mut pos, b'%');
                i += 1;
                continue;
            }

            // ── Parse optional flags (- 0 _ ~) ───────────────────────
            let mut flag = b'0';
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
            if i > width_start
                && let Ok(s) = core::str::from_utf8(&fmt[width_start..i])
                && let Ok(w) = s.parse::<u8>()
            {
                width = Some(w);
            }

            // ── Parse optional colons (: :: :::) ───────────────────
            let mut colons: u8 = 0;
            while i < fmt.len() && fmt[i] == b':' {
                colons += 1;
                i += 1;
            }

            if i >= fmt.len() {
                return Err(an_err!(DtErrKind::UnexpectedEnd, "after %"));
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
                if i > frac_start
                    && let Ok(s) = core::str::from_utf8(&fmt[frac_start..i])
                    && let Ok(w) = s.parse::<u8>()
                {
                    frac_width = Some(w);
                }
                if i >= fmt.len() {
                    return Err(an_err!(DtErrKind::BadFractional, "expected f or N after ."));
                }

                // optional ~ after width
                if fmt[i] == b'~' {
                    trim_trailing = true;
                    i += 1;
                }

                if i >= fmt.len() {
                    return Err(an_err!(DtErrKind::BadFractional, "expected f or N after ."));
                }

                let next = fmt[i];
                i += 1;

                if matches!(next, b'f' | b'N') {
                    let width_val = frac_width.unwrap_or(18);
                    let add_dot = (next == b'f') && (width_val > 0);

                    let dot_pos = if add_dot {
                        let p = pos;
                        Self::write_byte(&mut buf, &mut pos, b'.');
                        Some(p)
                    } else {
                        None
                    };

                    let wrote_frac = self.write_fractional_seconds(
                        &mut buf,
                        &mut pos,
                        frac_width,
                        trim_trailing,
                    );

                    if add_dot && !wrote_frac {
                        if let Some(p) = dot_pos {
                            buf[p] = 0;
                            pos = p;
                        }
                    }
                    continue;
                } else {
                    return Err(an_err!(DtErrKind::BadFractional, "expected f or N after ."));
                }
            }

            // ── Normal directives ──
            match directive {
                b'Y' => self.write_full_year(&mut buf, &mut pos, flag, width),
                b'm' => self.write_month_number(&mut buf, &mut pos, flag, width, true),
                b'd' | b'e' => self.write_day_of_month(&mut buf, &mut pos, flag, width, true),
                b'y' => self.write_two_digit_year(&mut buf, &mut pos, flag, width),
                b'A' => self.write_weekday_full(&mut buf, &mut pos, names.weekdays_full),
                b'a' => self.write_weekday_abbrev(&mut buf, &mut pos, names.weekdays_abbr),
                b'B' => self.write_month_name_full(&mut buf, &mut pos, names.months_full),
                b'b' | b'h' => self.write_month_name_abbrev(&mut buf, &mut pos, names.months_abbr),
                b'H' | b'k' => self.write_hour24(&mut buf, &mut pos, flag, width, true),
                b'M' => self.write_minute(&mut buf, &mut pos, flag, width, true),
                b'S' => self.write_second(&mut buf, &mut pos, flag, width, true),
                b'f' | b'N' => {
                    let _ = self.write_fractional_seconds(&mut buf, &mut pos, width, trim_trailing);
                }
                b'z' => self.write_timezone_offset(offset, &mut buf, &mut pos, colons),
                b'Q' => {
                    if let Some(iana) = tz {
                        Self::write_bytes(&mut buf, &mut pos, iana.as_bytes());
                    } else if let Some(ab) = abbrev {
                        Self::write_bytes(&mut buf, &mut pos, ab.as_bytes());
                    } else if offset.unwrap_or_default() == 0 {
                        Self::write_bytes(&mut buf, &mut pos, b"UTC");
                    } else if i >= fmt.len() {
                        while pos > 0 && matches!(buf[pos - 1], b' ' | b'\t' | b'\n' | b'\r') {
                            pos -= 1;
                        }
                    }
                }
                b'C' => self.write_century(&mut buf, &mut pos),
                b'G' => self.write_iso_week_year(&mut buf, &mut pos, flag, width),
                b'g' => self.write_two_digit_iso_week_year(&mut buf, &mut pos, flag, width),
                b'I' | b'l' => self.write_hour12(&mut buf, &mut pos, flag, width),
                b'j' => self.write_day_of_year(&mut buf, &mut pos, flag, width),
                b'q' => self.write_quarter(&mut buf, &mut pos, flag, width),
                b'n' => self.write_whitespace(&mut buf, &mut pos, b'n'),
                b't' => self.write_whitespace(&mut buf, &mut pos, b't'),
                b'P' => self.write_ampm(&mut buf, &mut pos, false),
                b'p' => self.write_ampm(&mut buf, &mut pos, true),
                b'r' => self.write_12hour_time_with_ampm(&mut buf, &mut pos),
                b's' => self.write_unix_timestamp(&mut buf, &mut pos),
                b'U' => self.write_week_number_sunday_based(&mut buf, &mut pos, flag, width),
                b'u' => self.write_weekday_number_monday_based(&mut buf, &mut pos, flag, width),
                b'V' => self.write_week_iso(&mut buf, &mut pos, flag, width),
                b'W' => self.write_week_number_monday_based(&mut buf, &mut pos, flag, width),
                b'w' => self.write_weekday_number_sunday_based(&mut buf, &mut pos, flag, width),
                b'F' => self.write_iso_date(&mut buf, &mut pos),
                b'D' => self.write_us_date_shortcut(&mut buf, &mut pos),
                b'T' => self.write_time_with_seconds_shortcut(&mut buf, &mut pos),
                b'R' => self.write_time_without_seconds_shortcut(&mut buf, &mut pos),
                b'Z' => self.write_timezone_abbrev(abbrev, &mut buf, &mut pos),
                b'L' => {
                    if self.scale != Scale::UTC {
                        Self::write_bytes(&mut buf, &mut pos, self.scale.abbrev().as_bytes());
                    }
                }
                b'*' => self.write_unbounded_year(&mut buf, &mut pos, flag, width),
                b'c' | b'X' | b'x' => {}
                _ => return Err(an_err!(DtErrKind::UnknownItem, "{}", directive)),
            }
        }

        Ok((buf, pos))
    }

    #[inline(always)]
    fn write_byte(buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, byte: u8) {
        if *pos < STRFTIME_SIZE {
            buf[*pos] = byte;
            *pos += 1;
        }
    }

    #[inline(always)]
    fn write_bytes(buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, bytes: &[u8]) {
        let len = bytes.len();
        if *pos + len > STRFTIME_SIZE {
            return;
        }
        buf[*pos..*pos + len].copy_from_slice(bytes);
        *pos += len;
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

    #[inline(always)]
    fn write_weekday_full(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        wkdays_full: &[&[u8]; 7],
    ) {
        let name = wkdays_full[self.wkday() as usize];
        Self::write_bytes(buf, pos, name);
    }

    #[inline(always)]
    fn write_weekday_abbrev(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        wkdays_abbrev: &[&[u8]; 7],
    ) {
        let name = wkdays_abbrev[self.wkday() as usize];
        Self::write_bytes(buf, pos, name);
    }

    #[inline(always)]
    fn write_month_name_full(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        mos_full: &[&[u8]; 12],
    ) {
        let name = mos_full[self.mo as usize - 1];
        Self::write_bytes(buf, pos, name);
    }

    #[inline(always)]
    fn write_month_name_abbrev(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        mos_abbrev: &[&[u8]; 12],
    ) {
        let name = mos_abbrev[self.mo as usize - 1];
        Self::write_bytes(buf, pos, name);
    }

    #[inline(always)]
    fn write_century(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        // Floor division → -123 becomes -2 (exactly matches parse_century)
        let century = self.yr.div_euclid(100);
        Self::write_i64(buf, pos, century, b'-', Some(0), b'0');
    }

    #[inline(always)]
    fn write_day_of_month(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32(buf, pos, self.day as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_fractional_seconds(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        width: Option<u8>,
        trim: bool,
    ) -> bool {
        Self::write_fractional(buf, pos, self.attos, width, trim)
    }

    #[inline(always)]
    fn write_iso_week_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_i64(buf, pos, self.iso_yr(), flag, width, b'0');
    }

    #[inline(always)]
    fn write_two_digit_iso_week_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        let yy = (self.iso_yr() % 100).saturating_abs() as u32;
        Self::write_u32(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_hour24(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32(buf, pos, self.hr as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_hour12(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        let hour24 = self.hr;
        let hour12 = if hour24 == 0 {
            12
        } else if hour24 > 12 {
            hour24 - 12
        } else {
            hour24
        };
        Self::write_u32(buf, pos, hour12 as u32, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_12hour_time_with_ampm(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        // Hour (12-hour, zero-padded)
        self.write_hour12(buf, pos, b'0', Some(2));
        Self::write_bytes(buf, pos, b":");
        // Minute (zero-padded)
        self.write_minute(buf, pos, b'0', Some(2), true);
        Self::write_bytes(buf, pos, b":");
        // Second (zero-padded)
        self.write_second(buf, pos, b'0', Some(2), true);
        Self::write_bytes(buf, pos, b" ");
        // AM/PM (uppercase, matching classic POSIX %r)
        self.write_ampm(buf, pos, true);
    }

    #[inline(always)]
    fn write_day_of_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(
            buf,
            pos,
            self.day_of_yr() as u32,
            flag,
            width.or(Some(3)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_minute(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32(buf, pos, self.min as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_month_number(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32(buf, pos, self.mo as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_quarter(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        // Month is 1–12, so this gives 1, 2, 3, or 4
        let quarter = ((self.mo - 1) / 3 + 1) as u32;

        // Default width is 1 (no leading zero unless requested)
        Self::write_u32(buf, pos, quarter, flag, width.or(Some(1)), b'0');
    }

    #[inline(always)]
    fn write_whitespace(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, ch: u8) {
        let bytes = if ch == b'n' { b"\n" } else { b"\t" };
        Self::write_bytes(buf, pos, bytes);
    }

    #[inline(always)]
    fn write_ampm(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize, upper: bool) {
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

    #[inline(always)]
    fn write_second(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
        pad: bool,
    ) {
        let default_pad = if pad { b'0' } else { b' ' };
        Self::write_u32(buf, pos, self.sec as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_unix_timestamp(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        let seconds = Dt::ymd_to_unix_sec(
            self.yr(),
            self.mo(),
            self.day(),
            self.hr(),
            self.min(),
            self.sec(),
        );
        Self::write_i64(buf, pos, seconds, b'-', Some(0), b'0');
    }

    #[inline(always)]
    fn write_weekday_number_sunday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(buf, pos, self.wkday() as u32, flag, width.or(Some(1)), b'0');
    }

    #[inline(always)]
    fn write_weekday_number_monday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(
            buf,
            pos,
            if self.wkday() == 0 { 7 } else { self.wkday() } as u32,
            flag,
            width.or(Some(1)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_week_iso(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(
            buf,
            pos,
            self.iso_wk() as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_week_number_sunday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(
            buf,
            pos,
            self.wk_of_yr_sun() as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_week_number_monday_based(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_u32(
            buf,
            pos,
            self.wk_of_yr_mon() as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_full_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_i64(buf, pos, self.yr, flag, width, b'0');
    }

    #[inline(always)]
    fn write_two_digit_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        let yy = (self.yr % 100).saturating_abs() as u32;
        Self::write_u32(buf, pos, yy, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_unbounded_year(
        &self,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        flag: u8,
        width: Option<u8>,
    ) {
        Self::write_i64(buf, pos, self.yr, flag, width, b'0');
    }

    fn write_timezone_offset(
        &self,
        offset: Option<i32>,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
        colons: u8,
    ) {
        let offset_sec = offset.unwrap_or(0);
        let (negative, hours, minutes) = Dt::sec_as_hhmm(offset_sec);
        let sign = if negative { b'-' } else { b'+' };

        // seconds component — only needed for %::z and %:::z+
        let seconds = ((offset_sec.saturating_abs() % 3600) % 60) as u8;

        match colons {
            // %z → +HHMM
            0 => {
                let mut tmp = [0u8; 5];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b'0' + minutes / 10;
                tmp[4] = b'0' + minutes % 10;
                Self::write_bytes(buf, pos, &tmp);
            }

            // %:z → +HH:MM
            1 => {
                let mut tmp = [0u8; 6];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b':';
                tmp[4] = b'0' + minutes / 10;
                tmp[5] = b'0' + minutes % 10;
                Self::write_bytes(buf, pos, &tmp);
            }

            // %::z → +HH:MM:SS (always include seconds)
            2 => {
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

            // %:::z and higher → use colons, but only show minutes/seconds when non-zero
            _ => {
                let mut tmp = [0u8; 9];
                let mut len = 0usize;

                tmp[len] = sign;
                len += 1;
                tmp[len] = b'0' + hours / 10;
                len += 1;
                tmp[len] = b'0' + hours % 10;
                len += 1;

                // Include minutes only if non-zero (or if seconds are present)
                if minutes != 0 || seconds != 0 {
                    tmp[len] = b':';
                    len += 1;
                    tmp[len] = b'0' + minutes / 10;
                    len += 1;
                    tmp[len] = b'0' + minutes % 10;
                    len += 1;

                    // Include seconds only if non-zero
                    if seconds != 0 {
                        tmp[len] = b':';
                        len += 1;
                        tmp[len] = b'0' + seconds / 10;
                        len += 1;
                        tmp[len] = b'0' + seconds % 10;
                        len += 1;
                    }
                }

                Self::write_bytes(buf, pos, &tmp[..len]);
            }
        }
    }

    #[inline(always)]
    fn write_timezone_abbrev(
        &self,
        abbrev: Option<LiteStr<49>>,
        buf: &mut [u8; STRFTIME_SIZE],
        pos: &mut usize,
    ) {
        if let Some(abbrev) = abbrev {
            Self::write_bytes(buf, pos, abbrev.as_bytes());
        } else {
            Self::write_bytes(buf, pos, b"UTC");
        }
    }

    #[inline(always)]
    fn write_iso_date(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_i64(buf, pos, self.yr, b'0', Some(4), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32(buf, pos, self.mo as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"-");
        Self::write_u32(buf, pos, self.day as u32, b'0', Some(2), b'0');
    }

    #[inline(always)]
    fn write_us_date_shortcut(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_u32(buf, pos, self.mo as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32(buf, pos, self.day as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b"/");
        Self::write_u32(
            buf,
            pos,
            (self.yr % 100).saturating_abs() as u32,
            b'0',
            Some(2),
            b'0',
        );
    }

    #[inline(always)]
    fn write_time_with_seconds_shortcut(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_u32(buf, pos, self.hr as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32(buf, pos, self.min as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32(buf, pos, self.sec as u32, b'0', Some(2), b'0');
    }

    #[inline(always)]
    fn write_time_without_seconds_shortcut(&self, buf: &mut [u8; STRFTIME_SIZE], pos: &mut usize) {
        Self::write_u32(buf, pos, self.hr as u32, b'0', Some(2), b'0');
        Self::write_bytes(buf, pos, b":");
        Self::write_u32(buf, pos, self.min as u32, b'0', Some(2), b'0');
    }

    pub(crate) fn write_u32(
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

        let mut digits = [0u8; 10];
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
    }

    pub(crate) fn write_i64(
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
}
