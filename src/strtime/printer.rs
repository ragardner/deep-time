use crate::{BufStr, Dt, DtErr, DtErrKind, FormatNames, Lang, STRTIME_SIZE, YmdHms, an_err};

struct Printer<'a> {
    ymd: &'a YmdHms,
    buf: [u8; STRTIME_SIZE],
    pos: usize,
    offset: Option<i32>,
    tz: Option<BufStr<49>>,
    abbrev: Option<BufStr<49>>,
    names: &'static FormatNames,
}

impl<'a> Printer<'a> {
    #[inline]
    fn new(
        ymd: &'a YmdHms,
        offset: Option<i32>,
        tz: Option<BufStr<49>>,
        abbrev: Option<BufStr<49>>,
        lang: Lang,
    ) -> Self {
        Self {
            ymd,
            buf: [0u8; STRTIME_SIZE],
            pos: 0,
            offset,
            tz,
            abbrev,
            names: lang.names(),
        }
    }

    fn print(&mut self, fmt: &[u8]) -> Result<(), DtErr> {
        let mut i = 0usize;

        while i < fmt.len() {
            let byte = fmt[i];

            if byte != b'%' {
                self.write_byte(byte);
                i += 1;
                continue;
            }

            i += 1; // skip '%'

            if i >= fmt.len() {
                return Err(an_err!(DtErrKind::TruncatedDirective));
            }

            // %% → literal percent
            if fmt[i] == b'%' {
                self.write_byte(b'%');
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
                return Err(an_err!(DtErrKind::TruncatedDirective));
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
                    return Err(an_err!(DtErrKind::InvalidFractional));
                }

                // optional ~ after width
                if fmt[i] == b'~' {
                    trim_trailing = true;
                    i += 1;
                }

                if i >= fmt.len() {
                    return Err(an_err!(DtErrKind::InvalidFractional));
                }

                let next = fmt[i];
                i += 1;

                if matches!(next, b'f' | b'N') {
                    let width_val = frac_width.unwrap_or(18);
                    let add_dot = (next == b'f') && (width_val > 0);

                    let dot_pos = if add_dot {
                        let p = self.pos;
                        self.write_byte(b'.');
                        Some(p)
                    } else {
                        None
                    };

                    let wrote_frac = self.write_fractional_seconds(frac_width, trim_trailing);

                    if add_dot
                        && !wrote_frac
                        && let Some(p) = dot_pos
                    {
                        self.buf[p] = 0;
                        self.pos = p;
                    }
                    continue;
                } else {
                    return Err(an_err!(DtErrKind::InvalidFractional));
                }
            }

            // ── Normal directives ──
            match directive {
                b'Y' => self.write_full_year(flag, width),
                b'm' => self.write_month_number(flag, width, true),
                b'd' | b'e' => self.write_day_of_month(flag, width, true),
                b'y' => self.write_two_digit_year(flag, width),
                b'A' => self.write_weekday_full(),
                b'a' => self.write_weekday_abbrev(),
                b'B' => self.write_month_name_full(),
                b'b' | b'h' => self.write_month_name_abbrev(),
                b'H' | b'k' => self.write_hour24(flag, width, true),
                b'M' => self.write_minute(flag, width, true),
                b'S' => self.write_second(flag, width, true),
                b'f' | b'N' => {
                    let _ = self.write_fractional_seconds(width, trim_trailing);
                }
                b'z' => self.write_timezone_offset(colons),
                b'Q' => {
                    if let Some(iana) = self.tz.as_ref() {
                        let bytes = iana.as_bytes();
                        let len = bytes.len();
                        if self.pos + len <= STRTIME_SIZE {
                            self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
                            self.pos += len;
                        }
                    } else if let Some(ab) = self.abbrev.as_ref() {
                        let bytes = ab.as_bytes();
                        let len = bytes.len();
                        if self.pos + len <= STRTIME_SIZE {
                            self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
                            self.pos += len;
                        }
                    } else if self.offset.unwrap_or_default() == 0 {
                        self.write_bytes(b"UTC");
                    } else if i >= fmt.len() {
                        while self.pos > 0
                            && matches!(self.buf[self.pos - 1], b' ' | b'\t' | b'\n' | b'\r')
                        {
                            self.pos -= 1;
                        }
                    }
                }
                b'C' => self.write_century(),
                b'G' => self.write_iso_week_year(flag, width),
                b'g' => self.write_two_digit_iso_week_year(flag, width),
                b'I' | b'l' => self.write_hour12(flag, width),
                b'j' => self.write_day_of_year(flag, width),
                b'q' => self.write_quarter(flag, width),
                b'n' => self.write_whitespace(b'n'),
                b't' => self.write_whitespace(b't'),
                b'P' => self.write_ampm(false),
                b'p' => self.write_ampm(true),
                b'r' => self.write_12hour_time_with_ampm(),
                b's' => self.write_unix_timestamp(),
                b'J' => self.write_noon2000_timestamp(),
                b'U' => self.write_week_number_sunday_based(flag, width),
                b'u' => self.write_weekday_number_monday_based(flag, width),
                b'V' => self.write_week_iso(flag, width),
                b'W' => self.write_week_number_monday_based(flag, width),
                b'w' => self.write_weekday_number_sunday_based(flag, width),
                b'F' => self.write_iso_date(),
                b'D' => self.write_us_date_shortcut(),
                b'T' => self.write_time_with_seconds_shortcut(),
                b'R' => self.write_time_without_seconds_shortcut(),
                b'Z' => self.write_timezone_abbrev(),
                b'L' => self.write_bytes(self.ymd.dt.target.abbrev().as_bytes()),
                b'*' => self.write_unbounded_year(flag, width),
                b'c' | b'X' | b'x' => {}
                _ => return Err(an_err!(DtErrKind::UnknownItem, "{}", char::from(directive))),
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn write_byte(&mut self, byte: u8) {
        if self.pos < STRTIME_SIZE {
            self.buf[self.pos] = byte;
            self.pos += 1;
        }
    }

    #[inline(always)]
    fn write_bytes(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        if self.pos + len > STRTIME_SIZE {
            return;
        }
        self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
        self.pos += len;
    }

    fn write_fractional(&mut self, subsec: u64, width: Option<u8>, trim: bool) -> bool {
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
            while end > 0 && digits[end - 1] == b'0' {
                end -= 1;
            }
            if end == 0 {
                return false;
            }
        }
        self.write_bytes(&digits[0..end]);
        true
    }

    // ──────────────────────────────────────────────────────────────
    // Individual write_ functions – one per parser directive
    // ──────────────────────────────────────────────────────────────

    #[inline(always)]
    fn write_weekday_full(&mut self) {
        let name = self.names.weekdays_full[self.ymd.wkday() as usize];
        self.write_bytes(name);
    }

    #[inline(always)]
    fn write_weekday_abbrev(&mut self) {
        let name = self.names.weekdays_abbr[self.ymd.wkday() as usize];
        self.write_bytes(name);
    }

    #[inline(always)]
    fn write_month_name_full(&mut self) {
        let name = self.names.months_full[self.ymd.mo as usize - 1];
        self.write_bytes(name);
    }

    #[inline(always)]
    fn write_month_name_abbrev(&mut self) {
        let name = self.names.months_abbr[self.ymd.mo as usize - 1];
        self.write_bytes(name);
    }

    #[inline(always)]
    fn write_day_of_month(&mut self, flag: u8, width: Option<u8>, pad: bool) {
        let default_pad = if pad { b'0' } else { b' ' };
        self.write_u32(self.ymd.day as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_fractional_seconds(&mut self, width: Option<u8>, trim: bool) -> bool {
        self.write_fractional(self.ymd.attos, width, trim)
    }

    #[inline(always)]
    fn write_iso_week_year(&mut self, flag: u8, width: Option<u8>) {
        self.write_i64(self.ymd.iso_yr(), flag, width, b'0');
    }

    #[inline(always)]
    fn write_two_digit_iso_week_year(&mut self, flag: u8, width: Option<u8>) {
        let yy = (self.ymd.iso_yr() % 100).saturating_abs() as u32;
        self.write_u32(yy, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_hour24(&mut self, flag: u8, width: Option<u8>, pad: bool) {
        let default_pad = if pad { b'0' } else { b' ' };
        self.write_u32(self.ymd.hr as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_hour12(&mut self, flag: u8, width: Option<u8>) {
        let hour24 = self.ymd.hr;
        let hour12 = if hour24 == 0 {
            12
        } else if hour24 > 12 {
            hour24 - 12
        } else {
            hour24
        };
        self.write_u32(hour12 as u32, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_12hour_time_with_ampm(&mut self) {
        self.write_hour12(b'0', Some(2));
        self.write_bytes(b":");
        self.write_minute(b'0', Some(2), true);
        self.write_bytes(b":");
        self.write_second(b'0', Some(2), true);
        self.write_bytes(b" ");
        self.write_ampm(true);
    }

    #[inline(always)]
    fn write_day_of_year(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(self.ymd.day_of_yr() as u32, flag, width.or(Some(3)), b'0');
    }

    #[inline(always)]
    fn write_minute(&mut self, flag: u8, width: Option<u8>, pad: bool) {
        let default_pad = if pad { b'0' } else { b' ' };
        self.write_u32(self.ymd.min as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_month_number(&mut self, flag: u8, width: Option<u8>, pad: bool) {
        let default_pad = if pad { b'0' } else { b' ' };
        self.write_u32(self.ymd.mo as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_quarter(&mut self, flag: u8, width: Option<u8>) {
        let quarter = ((self.ymd.mo - 1) / 3 + 1) as u32;
        self.write_u32(quarter, flag, width.or(Some(1)), b'0');
    }

    #[inline(always)]
    fn write_whitespace(&mut self, ch: u8) {
        let bytes = if ch == b'n' { b"\n" } else { b"\t" };
        self.write_bytes(bytes);
    }

    #[inline(always)]
    fn write_ampm(&mut self, upper: bool) {
        let hour = self.ymd.hr;
        let bytes = if hour < 12 {
            if upper { b"AM" } else { b"am" }
        } else if upper {
            b"PM"
        } else {
            b"pm"
        };
        self.write_bytes(bytes);
    }

    #[inline(always)]
    fn write_second(&mut self, flag: u8, width: Option<u8>, pad: bool) {
        let default_pad = if pad { b'0' } else { b' ' };
        self.write_u32(self.ymd.sec as u32, flag, width, default_pad);
    }

    #[inline(always)]
    fn write_unix_timestamp(&mut self) {
        let dt = self.ymd.dt.to_unix();
        if dt.to_attos() < 0 {
            self.write_byte(b'-');
        }
        self.write_i64(dt.to_sec64().saturating_abs(), b'-', Some(0), b'0');
        let frac = dt.to_sec_frac().saturating_abs() as u64;
        if frac != 0 {
            self.write_byte(b'.');
            let _ = self.write_fractional(frac, None, true);
        }
    }

    #[inline(always)]
    fn write_noon2000_timestamp(&mut self) {
        let dt = self.ymd.dt.to(self.ymd.dt.target);
        if dt.to_attos() < 0 {
            self.write_byte(b'-');
        }
        self.write_i64(dt.to_sec64().saturating_abs(), b'-', Some(0), b'0');
        let frac = dt.to_sec_frac().saturating_abs() as u64;
        if frac != 0 {
            self.write_byte(b'.');
            let _ = self.write_fractional(frac, None, true);
        }
    }

    #[inline(always)]
    fn write_weekday_number_sunday_based(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(self.ymd.wkday() as u32, flag, width.or(Some(1)), b'0');
    }

    #[inline(always)]
    fn write_weekday_number_monday_based(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(
            if self.ymd.wkday() == 0 {
                7
            } else {
                self.ymd.wkday()
            } as u32,
            flag,
            width.or(Some(1)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_week_iso(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(self.ymd.iso_wk() as u32, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_week_number_sunday_based(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(
            self.ymd.wk_of_yr_sun() as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_week_number_monday_based(&mut self, flag: u8, width: Option<u8>) {
        self.write_u32(
            self.ymd.wk_of_yr_mon() as u32,
            flag,
            width.or(Some(2)),
            b'0',
        );
    }

    #[inline(always)]
    fn write_full_year(&mut self, flag: u8, width: Option<u8>) {
        self.write_i64(self.ymd.yr, flag, width, b'0');
    }

    #[inline(always)]
    fn write_unbounded_year(&mut self, flag: u8, width: Option<u8>) {
        self.write_i64(self.ymd.yr, flag, width, b'0');
    }

    #[inline(always)]
    fn write_two_digit_year(&mut self, flag: u8, width: Option<u8>) {
        let yy = self.ymd.yr % 100;
        self.write_i64(yy, flag, width.or(Some(2)), b'0');
    }

    #[inline(always)]
    fn write_century(&mut self) {
        let century = self.ymd.yr / 100;
        self.write_i64(century, b'-', Some(0), b'0');
    }

    #[inline(always)]
    fn write_timezone_offset(&mut self, colons: u8) {
        let offset_sec = self.offset.unwrap_or(0);
        let (negative, hours, minutes) = Dt::sec_as_hhmm(offset_sec);
        let sign = if negative { b'-' } else { b'+' };

        let seconds = ((offset_sec.saturating_abs() % 3600) % 60) as u8;

        match colons {
            0 => {
                let mut tmp = [0u8; 5];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b'0' + minutes / 10;
                tmp[4] = b'0' + minutes % 10;
                self.write_bytes(&tmp);
            }
            1 => {
                let mut tmp = [0u8; 6];
                tmp[0] = sign;
                tmp[1] = b'0' + hours / 10;
                tmp[2] = b'0' + hours % 10;
                tmp[3] = b':';
                tmp[4] = b'0' + minutes / 10;
                tmp[5] = b'0' + minutes % 10;
                self.write_bytes(&tmp);
            }
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
                self.write_bytes(&tmp);
            }
            _ => {
                let mut tmp = [0u8; 9];
                let mut len = 0usize;

                tmp[len] = sign;
                len += 1;
                tmp[len] = b'0' + hours / 10;
                len += 1;
                tmp[len] = b'0' + hours % 10;
                len += 1;

                if minutes != 0 || seconds != 0 {
                    tmp[len] = b':';
                    len += 1;
                    tmp[len] = b'0' + minutes / 10;
                    len += 1;
                    tmp[len] = b'0' + minutes % 10;
                    len += 1;

                    if seconds != 0 {
                        tmp[len] = b':';
                        len += 1;
                        tmp[len] = b'0' + seconds / 10;
                        len += 1;
                        tmp[len] = b'0' + seconds % 10;
                        len += 1;
                    }
                }

                self.write_bytes(&tmp[..len]);
            }
        }
    }

    #[inline(always)]
    fn write_timezone_abbrev(&mut self) {
        if let Some(abbrev) = self.abbrev.as_ref() {
            let bytes = abbrev.as_bytes();
            let len = bytes.len();
            if self.pos + len <= STRTIME_SIZE {
                self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
                self.pos += len;
            }
        } else {
            self.write_bytes(b"UTC");
        }
    }

    #[inline(always)]
    fn write_iso_date(&mut self) {
        self.write_i64(self.ymd.yr, b'0', Some(4), b'0');
        self.write_bytes(b"-");
        self.write_u32(self.ymd.mo as u32, b'0', Some(2), b'0');
        self.write_bytes(b"-");
        self.write_u32(self.ymd.day as u32, b'0', Some(2), b'0');
    }

    #[inline(always)]
    fn write_us_date_shortcut(&mut self) {
        self.write_u32(self.ymd.mo as u32, b'0', Some(2), b'0');
        self.write_bytes(b"/");
        self.write_u32(self.ymd.day as u32, b'0', Some(2), b'0');
        self.write_bytes(b"/");
        self.write_u32(
            (self.ymd.yr % 100).saturating_abs() as u32,
            b'0',
            Some(2),
            b'0',
        );
    }

    #[inline(always)]
    fn write_time_with_seconds_shortcut(&mut self) {
        self.write_u32(self.ymd.hr as u32, b'0', Some(2), b'0');
        self.write_bytes(b":");
        self.write_u32(self.ymd.min as u32, b'0', Some(2), b'0');
        self.write_bytes(b":");
        self.write_u32(self.ymd.sec as u32, b'0', Some(2), b'0');
    }

    #[inline(always)]
    fn write_time_without_seconds_shortcut(&mut self) {
        self.write_u32(self.ymd.hr as u32, b'0', Some(2), b'0');
        self.write_bytes(b":");
        self.write_u32(self.ymd.min as u32, b'0', Some(2), b'0');
    }

    fn write_u32(&mut self, mut value: u32, flag: u8, width: Option<u8>, default_pad: u8) {
        let w = width.unwrap_or(2) as usize;

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

        if self.pos + num_digits + pad_len > STRTIME_SIZE {
            return;
        }

        if pad_left {
            for _ in 0..pad_len {
                self.buf[self.pos] = pad_char;
                self.pos += 1;
            }
        }

        for j in (0..num_digits).rev() {
            self.buf[self.pos] = digits[j];
            self.pos += 1;
        }
    }

    fn write_i64(&mut self, value: i64, flag: u8, width: Option<u8>, default_pad: u8) {
        let w = width.unwrap_or(4) as usize;

        let negative = value < 0;
        let mut v = value.unsigned_abs();

        let mut digits = [0u8; 20];
        let mut i = 0usize;

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

        if self.pos + (if negative { 1 } else { 0 }) + num_digits + pad_len > STRTIME_SIZE {
            return;
        }

        if negative {
            self.buf[self.pos] = b'-';
            self.pos += 1;
        }

        if pad_left {
            for _ in 0..pad_len {
                self.buf[self.pos] = pad_char;
                self.pos += 1;
            }
        }

        for j in (0..num_digits).rev() {
            self.buf[self.pos] = digits[j];
            self.pos += 1;
        }
    }
}

impl YmdHms {
    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn _to_str(
        &self,
        fmt: &str,
        offset: Option<i32>,
        tz: Option<BufStr<49>>,
        abbrev: Option<BufStr<49>>,
        lang: Lang,
    ) -> Result<alloc::string::String, DtErr> {
        let mut printer = Printer::new(self, offset, tz, abbrev, lang);
        printer.print(fmt.as_bytes())?;
        let pos = printer.pos;
        Ok(alloc::string::String::from_utf8_lossy(&printer.buf[..pos]).into_owned())
    }

    #[inline]
    pub(crate) fn _to_str_b(
        &self,
        fmt: &str,
        offset: Option<i32>,
        tz: Option<BufStr<49>>,
        abbrev: Option<BufStr<49>>,
        lang: Lang,
    ) -> Result<BufStr<STRTIME_SIZE>, DtErr> {
        let mut printer = Printer::new(self, offset, tz, abbrev, lang);
        printer.print(fmt.as_bytes())?;
        Ok(BufStr { bytes: printer.buf })
    }
}
