use crate::en::parse_month_name_abbrev;
use crate::{DtErr, DtErrKind, Offset, Parts, STRTIME_SIZE, Scale, an_err};

impl Parts {
    /// Generalized no alloc parser.
    ///
    /// - Only supports ASCII characters.
    /// - Timezones beyond UTC aliases require the `jiff-tz` feature which requires the
    ///   `std` feature.
    /// - If the format of the input is a typical iso datetime e.g. `2000-01-01T17:00:00`
    ///   and there is no trailing time scale then the `scale` field of the [`Parts`]
    ///   is set to [`Scale::UTC`].
    /// - If the format of the input is a seconds count, jd, or mjd, and there is no
    ///   trailing time scale then the scale field of the returned [`Parts`] is set
    ///   to [`Scale::TAI`].
    /// - This function is considerably faster than all other string parsing methods if
    ///   your date-time string is in one of the supported formats.
    ///
    /// ## Supported formats
    ///
    /// An **optional** library time scale right on the end of the input, e.g. `TAI` is
    /// supported for all of the below formats.
    ///
    /// ### ISO
    ///
    /// #### Format examples:
    ///
    /// - **`+2000-01-01T17:00:00 -0500 [America/New_York] TAI`**.
    /// - **`2024 Apr 18, 14:30:25 [America/New_York]`**. Abbreviated or full month
    /// - **`2024-109 14:30:25 [America/New_York]`**. Day of year
    ///
    /// #### Notes:
    ///
    /// - If a time is included then some kind of date-time separator e.g. `T` or space is
    ///   required.
    /// - Supports both calendar (`%Y-%m-%d`) and day-of-year (`%Y-%j`) formats.
    /// - Treats years digits literally as shown, for example `99-01-01` would be
    ///   the year 99 AD not 1999.
    /// - Supported **optional** components:
    ///     - Time components after a date e.g. `T12:00:00`.
    ///     - Offset after time components or directly after the date e.g. `+0200` or
    ///       `2023-01-01+05:00`.
    ///     - Timezone name, **requires square brackets** and **requires `jiff-tz`**
    ///       feature, after time or offset e.g. `T12:00:00 [America/New_York]`.
    ///
    /// ### Seconds since 2000-01-01 Noon
    ///
    /// #### Format examples:
    ///
    /// - **`SEC 1234.567 TDB`**.
    ///
    /// #### Notes:
    ///
    /// - `sec` prefix is required but case-**in**sensitive.
    /// - Fractional seconds are optional.
    ///
    /// ### JD
    ///
    /// #### Format examples:
    ///
    /// - **`JD 2451545.0 TAI`**.
    ///
    /// #### Notes:
    ///
    /// - `jd` prefix is required but case-**in**sensitive.
    /// - Fractional days are optional.
    ///
    /// ### MJD
    ///
    /// #### Format examples:
    ///
    /// - **`MJD 51544.5 TT`**.
    ///
    /// #### Notes:
    ///
    /// - `mjd` prefix is required but case-**in**sensitive.
    /// - Fractional days are optional.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_str_iso`](../struct.Dt.html#method.from_str_iso)
    pub fn from_str_iso(s: &str) -> Result<Self, DtErr> {
        let bytes = s.as_bytes();
        let len_ = bytes.len();
        if len_ > STRTIME_SIZE {
            return Err(an_err!(DtErrKind::InvalidLen));
        }

        let mut start = 0usize;
        while start < len_ {
            let b = bytes[start];
            if b.is_ascii_digit()
                || (matches!(b, b'+' | b'-')
                    && start + 1 < len_
                    && bytes[start + 1].is_ascii_digit())
            {
                break;
            }

            if b.is_ascii_alphabetic() {
                match detect_word(bytes, b, start, len_) {
                    Word::Jd => {
                        if let Some(p) = Self::from_str_jd_f(&s[start..], None) {
                            return Ok(p);
                        }
                    }
                    Word::Mjd => {
                        if let Some(p) = Self::from_str_mjd_f(&s[start..], None) {
                            return Ok(p);
                        }
                    }
                    Word::Sec => {
                        if let Some(p) = Self::from_str_sec_f(&s[start..], None) {
                            return Ok(p);
                        }
                    }
                    _ => {}
                }
                start = next_item(bytes, start, len_);
            } else {
                start += 1;
            }
        }

        if start == len_ {
            return Err(an_err!(DtErrKind::ExpectedYear));
        }

        let s = &s[start..];
        let bytes = s.as_bytes();
        let len_ = bytes.len();
        let mut pos: usize = 0;
        let mut tp = Parts::new_utc();

        // Year (manual accumulation, optional sign)
        let mut year: i64 = 0;
        let negative_year = if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
            pos += 1;
            bytes[pos] == b'-'
        } else {
            false
        };

        if bytes[pos].is_ascii_digit() {
            while pos < len_ && bytes[pos].is_ascii_digit() {
                year = year * 10 + (bytes[pos] - b'0') as i64;
                pos += 1;
            }
        } else {
            return Err(an_err!(DtErrKind::ExpectedYear));
        }

        if negative_year {
            year = -year;
        }
        tp.yr = Some(year);

        // required separator after year
        if pos < len_ {
            pos += 1;
        }

        let is_doy = is_doy(bytes, pos, len_);

        if is_doy {
            // 3-digit day of year
            let mut doy: u16 = 0;
            // digit 1
            if bytes[pos].is_ascii_digit() {
                doy = doy * 10 + (bytes[pos] - b'0') as u16;
            }
            pos += 1;
            // digit 2
            if bytes[pos].is_ascii_digit() {
                doy = doy * 10 + (bytes[pos] - b'0') as u16;
            }
            pos += 1;
            // digit 3
            if bytes[pos].is_ascii_digit() {
                doy = doy * 10 + (bytes[pos] - b'0') as u16;
                pos += 1;
            } else {
                return Err(an_err!(DtErrKind::ExpectedDayOfYear));
            }
            tp.day_of_yr = Some(doy);
        } else {
            // Abbreviated/full month or 1 or 2 digit month
            let mut mo: u8 = 0;
            if pos < len_ {
                if bytes[pos].is_ascii_digit() {
                    mo = mo * 10 + (bytes[pos] - b'0');
                    pos += 1;
                    if pos < len_ && bytes[pos].is_ascii_digit() {
                        mo = mo * 10 + (bytes[pos] - b'0');
                        pos += 1;
                    }
                } else if bytes[pos].is_ascii_alphabetic() && pos + 3 < len_ {
                    mo = parse_month_name_abbrev(&bytes[pos..])
                        .ok_or(an_err!(DtErrKind::InvalidMonthName))?;
                }
            }
            if mo == 0 {
                return Err(an_err!(DtErrKind::ExpectedMonth));
            }
            tp.mo = Some(mo);

            // Optional chars after month
            while pos < len_ && !bytes[pos].is_ascii_digit() {
                pos += 1;
            }

            // 1 or 2 digit day
            let mut day: u8 = 0;
            if pos < len_ && bytes[pos].is_ascii_digit() {
                day = day * 10 + (bytes[pos] - b'0');
                pos += 1;
                if pos < len_ && bytes[pos].is_ascii_digit() {
                    day = day * 10 + (bytes[pos] - b'0');
                    pos += 1;
                }
            }
            if day == 0 {
                return Err(an_err!(DtErrKind::ExpectedDay));
            }
            tp.day = Some(day);
        }

        // required date-time separator
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < len_ {
            let c = bytes[pos];
            // push past a T
            if !c.is_ascii_digit() && pos + 1 < len_ && !matches!(c, b'+' | b'-') {
                if bytes[pos + 1].is_ascii_digit() {
                    pos += 1;
                } else if bytes[pos + 1].is_ascii_whitespace()
                    || bytes[pos + 1].is_ascii_punctuation()
                {
                    pos += 1;
                    while pos < len_
                        && (bytes[pos].is_ascii_whitespace() || bytes[pos].is_ascii_punctuation())
                    {
                        pos += 1;
                    }
                }
            }
        }

        // Optional time components
        if pos < len_ && bytes[pos].is_ascii_digit() {
            // Hour (2 digits)
            let mut hr: u8 = 0;
            // digit 1
            if bytes[pos].is_ascii_digit() {
                hr = hr * 10 + (bytes[pos] - b'0');
                pos += 1;
            } else {
                return Err(an_err!(DtErrKind::ExpectedHour));
            }
            // digit 2
            if bytes[pos].is_ascii_digit() {
                hr = hr * 10 + (bytes[pos] - b'0');
                pos += 1;
            } else {
                return Err(an_err!(DtErrKind::ExpectedHour));
            }

            tp.hr = hr;

            'time: {
                // only continue if it's not a + or - and not an alpha
                if pos >= len_
                    || bytes[pos].is_ascii_digit()
                    || matches!(bytes[pos], b'+' | b'-')
                    || bytes[pos].is_ascii_alphabetic()
                {
                    break 'time;
                }
                pos += 1;

                // Minute (2 digits)
                if pos + 2 > len_ {
                    break 'time;
                }
                let mut min: u8 = 0;
                // digit 1
                if bytes[pos].is_ascii_digit() {
                    min = min * 10 + (bytes[pos] - b'0');
                    pos += 1;
                } else {
                    return Err(an_err!(DtErrKind::ExpectedMinute));
                }
                // digit 2
                if bytes[pos].is_ascii_digit() {
                    min = min * 10 + (bytes[pos] - b'0');
                    pos += 1;
                } else {
                    return Err(an_err!(DtErrKind::ExpectedMinute));
                }

                tp.min = min;

                // only continue if it's not a + or - and not an alpha
                if pos >= len_
                    || bytes[pos].is_ascii_digit()
                    || matches!(bytes[pos], b'+' | b'-')
                    || bytes[pos].is_ascii_alphabetic()
                {
                    break 'time;
                }
                pos += 1;

                // Second (2 digits, if present)
                if pos + 2 > len_ {
                    break 'time;
                }
                let mut sec: u8 = 0;
                // digit 1
                if bytes[pos].is_ascii_digit() {
                    sec = sec * 10 + (bytes[pos] - b'0');
                    pos += 1;
                } else {
                    return Err(an_err!(DtErrKind::ExpectedSecond));
                }
                // digit 2
                if bytes[pos].is_ascii_digit() {
                    sec = sec * 10 + (bytes[pos] - b'0');
                    pos += 1;
                } else {
                    return Err(an_err!(DtErrKind::ExpectedSecond));
                }

                tp.sec = sec;

                // only continue if it's not a + or - and not an alpha
                if pos >= len_
                    || bytes[pos].is_ascii_digit()
                    || matches!(bytes[pos], b'+' | b'-')
                    || bytes[pos].is_ascii_alphabetic()
                {
                    break 'time;
                }
                pos += 1;

                // Fractional seconds (with or without leading dot)
                if pos < len_ {
                    if bytes[pos] == b'.' {
                        pos += 1;
                    }

                    if pos < len_ && bytes[pos].is_ascii_digit() {
                        let mut attos: u64 = 0;
                        let mut digits_seen: usize = 0;

                        while pos < len_ && bytes[pos].is_ascii_digit() {
                            if digits_seen < 18 {
                                attos = attos * 10 + (bytes[pos] - b'0') as u64;
                                digits_seen += 1;
                            }
                            // Ignore any digits beyond the first 18
                            pos += 1;
                        }

                        if digits_seen > 0 {
                            tp.attos = attos * 10u64.pow(18u32.saturating_sub(digits_seen as u32));
                        }
                    }
                }
            }
            // Optional trailing Z/z
            if pos < len_ && matches!(bytes[pos], b'Z' | b'z') {
                pos += 1;
            }
        }

        // Skip any whitespace
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        // Optional offset
        if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
            let sign: i64 = if bytes[pos] == b'+' { 1 } else { -1 };
            pos += 1;

            // Parse hours (up to 2 digits). "+05:30"/"+0530"
            let mut hours: i64 = 0;
            let mut h_digits = 0usize;
            while pos < len_ && bytes[pos].is_ascii_digit() && h_digits < 2 {
                hours = hours * 10 + (bytes[pos] - b'0') as i64;
                pos += 1;
                h_digits += 1;
            }

            if h_digits > 0 {
                // Optional ':' separator before minutes
                if pos < len_ && bytes[pos] == b':' {
                    pos += 1;
                }

                // Parse minutes (up to 2 digits; optional)
                let mut minutes: i64 = 0;
                let mut m_digits = 0usize;
                while pos < len_ && bytes[pos].is_ascii_digit() && m_digits < 2 {
                    minutes = minutes * 10 + (bytes[pos] - b'0') as i64;
                    pos += 1;
                    m_digits += 1;
                }

                let total_sec_i64 = sign * (hours * 3600 + minutes * 60);
                let total_seconds: i32 =
                    total_sec_i64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
                tp.offset = Some(Offset::Fixed(total_seconds));
            }
        }

        // Skip any whitespace before IANA name or scale
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        // Optional IANA timezone name in square brackets, e.g. [America/New_York]
        // Must be explicitly wrapped in [] so we don't mistake a scale for a zone.
        if pos < len_ && bytes[pos] == b'[' {
            pos += 1; // skip '['

            let name_start = pos;

            while pos < len_ && bytes[pos] != b']' {
                pos += 1;
            }

            if pos >= len_ {
                return Err(an_err!(DtErrKind::InvalidTimeZone));
            }

            // pos is now at ']'
            let iana_bytes = &bytes[name_start..pos];

            let iana =
                core::str::from_utf8(iana_bytes).map_err(|_| an_err!(DtErrKind::InvalidBytes))?;

            tp.set_iana_name(Some(iana));
            pos += 1; // consume ']'
        }

        // Optional trailing scale (e.g. TAI, UTC)
        if pos < len_
            && let Some(sc) = Self::parse_scale(&bytes[pos..])
        {
            tp.scale = sc;
        }

        Ok(tp)
    }

    /// Parse a time scale abbreviation from a bytes slice.
    /// Skips leading non-alphabetic bytes, then takes up to 8 ASCII alphabetic
    /// characters and attempts to interpret them as a scale via [`Scale::from_abbrev`].
    #[inline(always)]
    pub(crate) fn parse_scale(bytes: &[u8]) -> Option<Scale> {
        let len_ = bytes.len();
        let mut pos = 0usize;
        while pos < len_ && !bytes[pos].is_ascii_alphabetic() {
            pos += 1;
        }
        if let Ok(s) = core::str::from_utf8(&bytes[pos..(pos + 8).min(len_)]) {
            return Scale::from_abbrev(s);
        }
        None
    }
}

#[inline(always)]
fn is_doy(bytes: &[u8], mut pos: usize, len_: usize) -> bool {
    // index 1
    if pos == len_ || (!bytes[pos].is_ascii_digit() && !matches!(bytes[pos], b' ')) {
        return false;
    } else {
        pos += 1;
    }
    // index 2
    if pos == len_ || (!bytes[pos].is_ascii_digit() && !matches!(bytes[pos], b' ')) {
        return false;
    } else {
        pos += 1;
    }
    // index 3, must be digit
    if pos == len_ || !bytes[pos].is_ascii_digit() {
        return false;
    } else {
        pos += 1;
    }
    // index 4 end or non digit
    pos == len_ || !bytes[pos].is_ascii_digit()
}

enum Word {
    None,
    Sec,
    Jd,
    Mjd,
}

#[inline(always)]
fn detect_word(bytes: &[u8], b: u8, pos: usize, len_: usize) -> Word {
    if pos + 3 <= len_ {
        if matches!(b, b'J' | b'j') {
            let b1 = bytes[pos + 1].to_ascii_uppercase();
            if b1 == b'D' {
                return Word::Jd;
            }
        } else if matches!(b, b'M' | b'm') {
            let b1 = bytes[pos + 1].to_ascii_uppercase();
            let b2 = bytes[pos + 2].to_ascii_uppercase();
            if b1 == b'J' && b2 == b'D' {
                return Word::Mjd;
            }
        } else if matches!(b, b'S' | b's') {
            let b1 = bytes[pos + 1].to_ascii_uppercase();
            let b2 = bytes[pos + 2].to_ascii_uppercase();
            if b1 == b'E' && b2 == b'C' {
                return Word::Sec;
            }
        }
    }
    Word::None
}

#[inline(always)]
fn next_item(bytes: &[u8], mut pos: usize, len_: usize) -> usize {
    while pos < len_ && bytes[pos].is_ascii_alphabetic() {
        pos += 1;
    }
    while pos < len_ && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }

    pos
}
