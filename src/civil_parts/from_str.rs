use crate::en::{parse_month_name_abbrev, parse_wkday_name_abbrev};
use crate::{
    ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, ATTOS_PER_SEC_I128, DtErr, DtErrKind, Epoch,
    JD_2000_2_451_545_I128, Offset, ParsedReal, Parts, SEC_PER_DAY, STRTIME_SIZE, Scale, Timestamp,
    Weekday, an_err,
};

impl Parts {
    /// Fast, no-alloc parser for common ISO-like and epoch-style date-time strings.
    ///
    /// Returns raw civil / numeric components in a [`Parts`] value (not a resolved
    /// instant). Convert with [`Parts::to_dt`](struct.Parts.html#method.to_dt) when you
    /// need a [`Dt`](../struct.Dt.html).
    ///
    /// - Only **ASCII** input is supported.
    /// - Inputs longer than [`STRTIME_SIZE`](../consts/constant.STRTIME_SIZE.html) are
    ///   rejected with [`DtErrKind::InvalidLen`](../error/enum.DtErrKind.html#variant.InvalidLen).
    /// - Leading non-date junk is skipped until a year-like start (`digit` or `±`digit),
    ///   a recognized alphabetic prefix (`JD` / `MJD` / `SEC`, case-insensitive), or an
    ///   English weekday name / abbrev (day-month-year order).
    /// - Trailing characters after a successful parse are generally ignored (lenient).
    /// - Considerably faster than format-string / smart parsers when the input is one
    ///   of the shapes below.
    ///
    /// ## Default scale on the returned [`Parts`]
    ///
    /// - **Calendar / day-of-year / ISO week** with no trailing scale → [`Scale::UTC`].
    /// - **`SEC` / `JD` / `MJD`** with no trailing scale → [`Scale::TAI`].
    /// - A trailing library scale (e.g. `TAI`, `TDB`, `GPS`) is accepted on **all**
    ///   formats and overrides the default.
    ///
    /// ## Supported formats
    ///
    /// ### ISO-like civil date-times
    ///
    /// #### Format examples
    ///
    /// - **`+2000-01-01T17:00:00 -0500 [America/New_York] TAI`**
    /// - **`2024 Apr 18, 14:30:25 [America/New_York]`** — month abbrev or full English name
    /// - **`Sat, 07 Feb 2015 11:22:33`**, **`Sat,07Feb2015T11:22:33`** — weekday first
    ///   (day-month-year; `Sun`…`Sat` or full English name)
    /// - **`2024-109 14:30:25`** — day of year (`%Y-%j`)
    /// - **`2024-W11`**, **`2024W11`**, **`2024-W11-4`** — ISO week date (`%G-W%V`, optional
    ///   weekday `%u` with Monday=`1` … Sunday=`7`)
    /// - **`2024`**, **`2024-03`**, **`2024 Mar`** — partial dates (see below)
    /// - **`2024-04-18T9:3:5.5`**, **`2024-04-18T143025`** — flexible / compact time
    ///
    /// #### Date forms
    ///
    /// Year digits are taken **literally** (no century window): `99-01-01` is year 99 AD.
    /// Year overflow during accumulation yields
    /// [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange).
    ///
    /// **Weekday first** → day, month, year (required); weekday stored in [`Parts::wkday`].
    /// Hyphen before the year is a separator (`07-Feb-2015`); a signed year needs space
    /// or a doubled sign (`07 Feb -4714`, `07-Feb--4714`, `+2015`).
    ///
    /// **Otherwise year first.** After an optional sign and year digits, exactly one of:
    ///
    /// 1. **ISO week** — `W`/`w` immediately after the year (or after a non-letter
    ///    separator such as `-`): e.g. `2024-W11`, `2024W114`.
    ///    - Sets [`Parts::iso_wk_yr`] / [`Parts::iso_wk`] (and optional [`Parts::wkday`]);
    ///      clears calendar [`Parts::yr`].
    ///    - Week number required right after `W` (1–2 digits); `0` becomes week `1`.
    ///    - Optional weekday: `-4` or basic trailing digit `1..=7`.
    ///    - This is **not** strftime `%W` (Monday week-of-year on a calendar year).
    /// 2. **Day of year** — three slots `[digit|space][digit|space][digit]` not followed
    ///    by another digit (so `2024-0401` is calendar `04-01`, not DOY). Space padding
    ///    is allowed (`2024-  9`). Sets [`Parts::day_of_yr`].
    /// 3. **Calendar month/day** — numeric month (1–2 digits) or English month name
    ///    (abbrev or full; matched from the first three letters), then day (1–2 digits).
    ///
    /// **Partial calendar dates** (year-first only) default missing fields to `1`
    /// (January / day 1):
    ///
    /// - Year only: `2024`, `2024-` → 1 January.
    /// - Year-month: `2024-03`, `2024 Mar` → day 1.
    /// - Explicit zero month/day digits are treated like omitted (`2024-00`, `2024-03-0` → 1).
    /// - Year or year-month may be followed by time: `2024T12:00`, `2024-03T12:00`
    ///   (the `T` is not read as a day field).
    ///
    /// #### Time (optional)
    ///
    /// - Usually introduced by `T`/`t`, space, or another non-digit separator; compact
    ///   glued times after a full day are also accepted (e.g. `…18143025`).
    /// - Hour, minute, and second are **1 or 2** digits when the field ends at `:` /
    ///   space (or, for seconds, `.` before a fraction).
    /// - Compact digit runs without separators are supported (e.g. `T143025`).
    /// - HMS field separators are only `:` or space (or glued digits). A `.` before the
    ///   seconds field is an error ([`DtErrKind::ExpectedSecond`]); fractional seconds
    ///   require a seconds field first (e.g. `14:30:25.5`, not `14:30.5`).
    /// - Fractional seconds: `.` then digits (up to 18 kept as attoseconds; extra digits
    ///   ignored).
    /// - Optional trailing `Z`/`z` is consumed (does not set a numeric offset by itself).
    /// - Minutes must be `≤ 59`; seconds must be `≤ 60` (leap second `60` allowed).
    ///
    /// #### Optional trailing components
    ///
    /// - **Offset** — `+`/`-` then hours (and optional minutes), with or without `:`:
    ///   `+02:00`, `-0530`, also allowed directly after the date. Hours are not
    ///   range-checked; offset minutes must be `≤ 59`.
    /// - **IANA name** — must be in square brackets, e.g. `[America/New_York]`.
    ///   Resolving non-UTC aliases requires the `jiff-tz` or `jiff-tz-bundle` feature
    ///   (both require `alloc`).
    /// - **Scale** — library abbreviation, e.g. `TAI`, `UTC`, `TDB`, `GPS`.
    ///
    /// ### Seconds since 2000-01-01 noon (library epoch)
    ///
    /// #### Format examples
    ///
    /// - **`SEC 1234.567 TDB`**
    /// - **`sec1234.5 TAI`**
    ///
    /// #### Notes
    ///
    /// - `sec` prefix is required (case-insensitive).
    /// - Fractional seconds optional; sets [`Parts::timestamp`] with
    ///   [`Epoch::Noon2000`](enum.Epoch.html#variant.Noon2000).
    ///
    /// ### Julian Date
    ///
    /// #### Format examples
    ///
    /// - **`JD 2451545.0 TAI`**
    /// - **`JD2451545.25 TT`**
    ///
    /// #### Notes
    ///
    /// - `jd` prefix is required (case-insensitive).
    /// - Fractional days optional; result is a [`Parts::timestamp`] on
    ///   [`Epoch::Noon2000`](enum.Epoch.html#variant.Noon2000).
    ///
    /// ### Modified Julian Date
    ///
    /// #### Format examples
    ///
    /// - **`MJD 51544.5 TT`**
    /// - **`mjd 51544.25`**
    ///
    /// #### Notes
    ///
    /// - `mjd` prefix is required (case-insensitive).
    /// - Fractional days optional; same timestamp representation as JD.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_str`](../struct.Dt.html#method.from_str) — same input, returns
    ///   a [`Dt`](../struct.Dt.html) on TAI via [`Parts::to_dt`](struct.Parts.html#method.to_dt).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, DtErr> {
        let bytes = s.as_bytes();
        let len_ = bytes.len();
        if len_ > STRTIME_SIZE {
            return Err(an_err!(DtErrKind::InvalidLen));
        }

        let mut start = 0usize;
        // Leading English weekday ⇒ day-month-year field order (RFC 2822-ish).
        let mut day_first = false;
        let mut leading_wkday: Option<Weekday> = None;
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
                    Word::None => {
                        // Only the first weekday name selects day-first order.
                        if !day_first
                            && start + 3 <= len_
                            && let Some(idx) = parse_wkday_name_abbrev(&bytes[start..])
                        {
                            leading_wkday = Weekday::from_sunday_0_based(idx);
                            day_first = true;
                        }
                    }
                }
                start = next_item(bytes, start, len_);
            } else {
                start += 1;
            }
        }

        if start == len_ {
            return Err(an_err!(if day_first {
                DtErrKind::ExpectedDay
            } else {
                DtErrKind::ExpectedYear
            }));
        }

        let s = &s[start..];
        let bytes = s.as_bytes();
        let len_ = bytes.len();
        let mut pos: usize = 0;
        let mut tp = Parts::new_utc();
        tp.wkday = leading_wkday;

        if day_first {
            parse_day_first_ymd(bytes, &mut pos, len_, &mut tp)?;
        } else {
            // Year (manual accumulation, optional sign)
            let mut year: i64 = 0;
            let negative_year = if pos < len_ && matches!(bytes[pos], b'+' | b'-') {
                let neg = bytes[pos] == b'-';
                pos += 1;
                neg
            } else {
                false
            };

            if bytes[pos].is_ascii_digit() {
                while pos < len_ && bytes[pos].is_ascii_digit() {
                    let digit = (bytes[pos] - b'0') as i64;
                    year = year
                        .checked_mul(10)
                        .and_then(|n| n.checked_add(digit))
                        .ok_or_else(|| an_err!(DtErrKind::YearOutOfRange))?;
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
                if !bytes[pos].is_ascii_alphabetic() {
                    pos += 1;
                }
            } else {
                // Year only → 1 Jan
                tp.mo = Some(1);
                tp.day = Some(1);
                return Ok(tp);
            }

            // ISO week date (`2024-W11`, `2024W11`, optional weekday `…-4` / `…4`)
            // or day-of-year or calendar month/day.
            // `%G-W%V` / `%G-W%V-%u` — not strftime `%W` (that is calendar `wk_mon`).
            if pos < len_ && matches!(bytes[pos], b'W' | b'w') {
                // Require a digit immediately after W before mutating Parts.
                let mut p = pos + 1;
                if p >= len_ || !bytes[p].is_ascii_digit() {
                    return Err(an_err!(DtErrKind::ExpectedWeekNumber));
                }
                let mut week = bytes[p] - b'0';
                p += 1;
                if p < len_ && bytes[p].is_ascii_digit() {
                    week = week * 10 + (bytes[p] - b'0');
                    p += 1;
                }

                // Optional weekday Mon=1…Sun=7: extended `…-4` or basic trailing digit.
                let mut wkday = None;
                if p < len_ {
                    let b = bytes[p];
                    if b == b'-' {
                        let dpos = p + 1;
                        if dpos < len_ && bytes[dpos].is_ascii_digit() {
                            let d = bytes[dpos] - b'0';
                            wkday = Some(
                                Weekday::from_monday_1_based(d)
                                    .ok_or(an_err!(DtErrKind::ExpectedWeekdayNumber))?,
                            );
                            p = dpos + 1;
                        }
                        // Lone `-` left for later stages.
                    } else if b.is_ascii_digit() {
                        // Basic `YYYYWwwD` — trailing digit is the weekday; must be 1..=7
                        // (do not leave 0/8/9 for the time stage).
                        let d = b - b'0';
                        wkday = Some(
                            Weekday::from_monday_1_based(d)
                                .ok_or(an_err!(DtErrKind::ExpectedWeekdayNumber))?,
                        );
                        p += 1;
                    }
                }

                pos = p;
                tp.iso_wk_yr = tp.yr.take();
                tp.iso_wk = Some(if week == 0 { 1 } else { week });
                tp.wkday = wkday;
                // Missing weekday → Monday in to_dt via unwrap_or.
            } else if let Some(doy) = try_parse_doy(bytes, &mut pos, len_) {
                tp.day_of_yr = Some(doy);
            } else {
                'day_month: {
                    while pos < len_
                        && (bytes[pos].is_ascii_whitespace() || bytes[pos].is_ascii_punctuation())
                    {
                        pos += 1;
                    }
                    if pos >= len_ {
                        // Year only (trailing junk/separators) → 1 Jan
                        tp.mo = Some(1);
                        tp.day = Some(1);
                        return Ok(tp);
                    }

                    // Year-only + time (`2024T12:00`): keep `T` for the time stage.
                    if matches!(bytes[pos], b'T' | b't') {
                        tp.mo = Some(1);
                        tp.day = Some(1);
                        break 'day_month;
                    }

                    // Abbreviated/full month or 1 or 2 digit month
                    if bytes[pos].is_ascii_digit() {
                        let mut mo: u8;
                        mo = bytes[pos] - b'0';
                        pos += 1;
                        if pos < len_ && bytes[pos].is_ascii_digit() {
                            mo = mo * 10 + (bytes[pos] - b'0');
                            pos += 1;
                        }
                        // 0 → January (zero / missing month field → default).
                        tp.mo = Some(if mo == 0 { 1 } else { mo });
                    } else if bytes[pos].is_ascii_alphabetic() && pos + 3 <= len_ {
                        // `<=` so bare "Mar" at EOS matches (exactly 3 letters).
                        tp.mo = Some(
                            parse_month_name_abbrev(&bytes[pos..])
                                .ok_or(an_err!(DtErrKind::InvalidMonthName))?,
                        );
                        // pos stays on the name; non-digit skip below walks past it.
                    }

                    // Skip to day; stop before `T`/`t` so `2024-03T12:00` is time, not day.
                    while pos < len_ {
                        let b = bytes[pos];
                        if b.is_ascii_digit() {
                            break;
                        }
                        if matches!(b, b'T' | b't') && !bytes[pos - 1].is_ascii_alphabetic() {
                            // Year-month only (+ time) → day 1; mo already set (or default below).
                            tp.mo.get_or_insert(1);
                            tp.day = Some(1);
                            break 'day_month;
                        }
                        pos += 1;
                    }
                    if pos >= len_ {
                        // Year-month only → day 1
                        tp.mo.get_or_insert(1);
                        tp.day = Some(1);
                        return Ok(tp);
                    }

                    // 1 or 2 digit day
                    let mut day = bytes[pos] - b'0';
                    pos += 1;
                    if pos < len_ && bytes[pos].is_ascii_digit() {
                        day = day * 10 + (bytes[pos] - b'0');
                        pos += 1;
                    }
                    // 0 → day 1 (zero / missing day field → default).
                    tp.day = Some(if day == 0 { 1 } else { day });
                }
            }
        }

        // required date-time separator
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= len_ {
            return Ok(tp);
        }
        // push past a T
        if !bytes[pos].is_ascii_digit() && pos + 1 < len_ && !matches!(bytes[pos], b'+' | b'-') {
            if bytes[pos + 1].is_ascii_digit() {
                pos += 1;
            } else if bytes[pos + 1].is_ascii_whitespace() || bytes[pos + 1].is_ascii_punctuation()
            {
                pos += 1;
                while pos < len_
                    && (bytes[pos].is_ascii_whitespace() || bytes[pos].is_ascii_punctuation())
                {
                    pos += 1;
                }
            }
        }
        if pos >= len_ {
            return Ok(tp);
        }

        // Optional time components
        'time: {
            if !bytes[pos].is_ascii_digit() {
                break 'time;
            }

            // Hour
            // digit 1
            tp.hr = bytes[pos] - b'0';
            pos += 1;
            // digit 2
            if pos >= len_ {
                return Ok(tp);
            }
            if bytes[pos].is_ascii_digit() {
                tp.hr = tp.hr * 10 + (bytes[pos] - b'0');
                pos += 1;
            } else if bytes[pos] == b'.' {
                // Fraction without a seconds field (e.g. `T12.5`).
                return Err(an_err!(DtErrKind::ExpectedSecond));
            } else if !matches!(bytes[pos], b':' | b' ') {
                break 'time;
            }

            if pos >= len_ {
                return Ok(tp);
            }
            // only continue if it's not a + or - and not an alpha
            if matches!(bytes[pos], b'A'..=b'Z' | b'a'..=b'z' | b'+' | b'-') {
                break 'time;
            }

            // perhaps a separator between H and M (`:` / space only; compact has none)
            if !bytes[pos].is_ascii_digit() {
                if bytes[pos] == b'.' {
                    return Err(an_err!(DtErrKind::ExpectedSecond));
                }
                if !matches!(bytes[pos], b':' | b' ') {
                    break 'time;
                }
                pos += 1;
                if pos >= len_ {
                    return Ok(tp);
                }
            }

            // Minutes
            // digit 1
            if bytes[pos].is_ascii_digit() {
                tp.min = bytes[pos] - b'0';
                pos += 1;
            } else if bytes[pos] == b'.' {
                // e.g. `T12:.5`
                return Err(an_err!(DtErrKind::ExpectedSecond));
            } else {
                break 'time;
            }
            // digit 2
            if pos >= len_ {
                return Ok(tp);
            }
            if bytes[pos].is_ascii_digit() {
                tp.min = tp.min * 10 + (bytes[pos] - b'0');
                pos += 1;
            } else if bytes[pos] == b'.' {
                // Fraction without a seconds field (e.g. `T12:3.5`).
                return Err(an_err!(DtErrKind::ExpectedSecond));
            } else if !matches!(bytes[pos], b':' | b' ') {
                break 'time;
            }
            if tp.min > 59 {
                return Err(an_err!(DtErrKind::MinuteOutOfRange));
            }

            if pos >= len_ {
                return Ok(tp);
            }
            // only continue if it's not a + or - and not an alpha
            if matches!(bytes[pos], b'A'..=b'Z' | b'a'..=b'z' | b'+' | b'-') {
                break 'time;
            }

            // perhaps a separator between M and S (`:` / space only; compact has none)
            if !bytes[pos].is_ascii_digit() {
                if bytes[pos] == b'.' {
                    // Fraction without a seconds field (e.g. `T12:30.5`).
                    return Err(an_err!(DtErrKind::ExpectedSecond));
                }
                if !matches!(bytes[pos], b':' | b' ') {
                    break 'time;
                }
                pos += 1;
                if pos >= len_ {
                    return Ok(tp);
                }
            }

            // Seconds
            // digit 1
            if bytes[pos].is_ascii_digit() {
                tp.sec = bytes[pos] - b'0';
                pos += 1;
            } else if bytes[pos] == b'.' {
                // e.g. `T12:30:.5`
                return Err(an_err!(DtErrKind::ExpectedSecond));
            } else {
                break 'time;
            }
            // digit 2
            if pos >= len_ {
                return Ok(tp);
            }
            if bytes[pos].is_ascii_digit() {
                tp.sec = tp.sec * 10 + (bytes[pos] - b'0');
                pos += 1;
            } else if !matches!(bytes[pos], b':' | b' ' | b'.') {
                // `:` / space: 1-digit sec field ends; `.` continues into fractional seconds.
                break 'time;
            }
            if tp.sec > 60 {
                return Err(an_err!(DtErrKind::SecondOutOfRange));
            }

            if pos >= len_ {
                return Ok(tp);
            }
            // only continue if it's not a + or - and not an alpha
            if matches!(bytes[pos], b'A'..=b'Z' | b'a'..=b'z' | b'+' | b'-') {
                break 'time;
            }

            // perhaps a . between S and fractional S
            if bytes[pos] == b'.' {
                pos += 1;
                if pos >= len_ {
                    return Ok(tp);
                }
            }

            // Fractional seconds (with or without leading dot)
            if !bytes[pos].is_ascii_digit() {
                break 'time;
            }
            tp.attos = (bytes[pos] - b'0') as u64;
            pos += 1;
            let mut digits_seen: usize = 1;
            while pos < len_ && bytes[pos].is_ascii_digit() {
                if digits_seen < 18 {
                    tp.attos = tp.attos * 10 + (bytes[pos] - b'0') as u64;
                    digits_seen += 1;
                }
                // Ignore any digits beyond the first 18
                pos += 1;
            }
            if digits_seen > 0 {
                tp.attos *= 10u64.pow(18u32.saturating_sub(digits_seen as u32));
            }
        }
        // Optional trailing Z/z
        if pos < len_ && matches!(bytes[pos], b'Z' | b'z') {
            pos += 1;
        }

        // Skip any whitespace
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= len_ {
            return Ok(tp);
        }

        // Optional offset (`+HH`, `+HH:MM`, `+HHMM`; hours unrestricted, minutes ≤ 59)
        'offset: {
            if !matches!(bytes[pos], b'+' | b'-') {
                break 'offset;
            }
            let sign: i32 = if bytes[pos] == b'+' { 1 } else { -1 };
            pos += 1;

            // Hours
            // digit 1
            if pos >= len_ || !bytes[pos].is_ascii_digit() {
                // Bare `+`/`-` — leave offset unset (same as before).
                break 'offset;
            }
            let mut hours: i32 = (bytes[pos] - b'0') as i32;
            pos += 1;
            // digit 2
            if pos >= len_ {
                tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                return Ok(tp);
            }
            if bytes[pos].is_ascii_digit() {
                hours = hours * 10 + (bytes[pos] - b'0') as i32;
                pos += 1;
            } else if bytes[pos] != b':' {
                // Hours only; remainder is IANA / scale / junk.
                tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                break 'offset;
            }

            if pos >= len_ {
                tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                return Ok(tp);
            }
            // Stop before scale / IANA / whitespace (do not consume them).
            if matches!(bytes[pos], b'A'..=b'Z' | b'a'..=b'z' | b'[')
                || bytes[pos].is_ascii_whitespace()
            {
                tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                break 'offset;
            }

            // perhaps a separator between H and M (`:` only; compact `+0530` has none)
            if !bytes[pos].is_ascii_digit() {
                if bytes[pos] != b':' {
                    tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                    break 'offset;
                }
                pos += 1;
                if pos >= len_ {
                    tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                    return Ok(tp);
                }
            }

            // Minutes
            // digit 1
            if !bytes[pos].is_ascii_digit() {
                // e.g. `+05:` with nothing after — hours only
                tp.offset = Some(Offset::Fixed(sign * hours * 3600));
                break 'offset;
            }
            let mut minutes: u8 = bytes[pos] - b'0';
            pos += 1;
            // digit 2
            if pos >= len_ {
                if minutes > 59 {
                    return Err(an_err!(DtErrKind::InvalidOffsetMinute));
                }
                tp.offset = Some(Offset::Fixed(sign * (hours * 3600 + minutes as i32 * 60)));
                return Ok(tp);
            }
            if bytes[pos].is_ascii_digit() {
                minutes = minutes * 10 + (bytes[pos] - b'0');
                pos += 1;
            }
            if minutes > 59 {
                return Err(an_err!(DtErrKind::InvalidOffsetMinute));
            }

            tp.offset = Some(Offset::Fixed(sign * (hours * 3600 + minutes as i32 * 60)));
        }

        // Skip any whitespace before IANA name or scale
        while pos < len_ && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= len_ {
            return Ok(tp);
        }

        // Optional IANA timezone name in square brackets, e.g. [America/New_York]
        // Must be explicitly wrapped in [] so we don't mistake a scale for a zone.
        if bytes[pos] == b'[' {
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
            if pos >= len_ {
                return Ok(tp);
            }
        }

        // Optional trailing scale (e.g. TAI, UTC)
        if let Some(sc) = Self::parse_scale(&bytes[pos..]) {
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
        if pos < len_
            && let Ok(s) = core::str::from_utf8(&bytes[pos..(pos + 8).min(len_)])
        {
            return Scale::from_abbrev(s);
        }
        None
    }

    /// Shared parser for decimal input.
    ///
    /// Used by both [`Parts::from_str_sec_f`] and [`Dt::from_str_sec_f`](../struct.Dt.html#method.from_str_sec_f).
    /// Returns the raw numeric components + resolved scale; the caller decides
    /// how to materialize the value (full attos for `Dt`, or a Noon2000
    /// [`Timestamp`] for `Parts`).
    pub(crate) fn parse_str_f(bytes: &[u8], scale: Option<Scale>) -> Option<ParsedReal> {
        if bytes.is_empty() || bytes.len() > STRTIME_SIZE {
            return None;
        }

        // Skip leading junk until we see +, -, ., or a digit.
        let mut pos = 0usize;
        while pos < bytes.len() {
            match bytes[pos] {
                b'+' | b'-' | b'.' | b'0'..=b'9' => break,
                _ => pos += 1,
            }
        }

        if pos >= bytes.len() {
            return None;
        }

        // Optional sign (only at the start of the number we decided to parse)
        let negative = match bytes[pos] {
            b'-' => {
                pos += 1;
                true
            }
            b'+' => {
                pos += 1;
                false
            }
            _ => false,
        };

        if pos >= bytes.len() {
            return None;
        }

        // Integer part (may be empty when we landed on '.').
        // Overflow → saturate at u64::MAX and skip the rest of the integer digits.
        let mut int_u: u64 = 0;
        let mut saw_digit = false;

        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            saw_digit = true;
            let d = (bytes[pos] - b'0') as u64;
            pos += 1;
            match int_u.checked_mul(10).and_then(|n| n.checked_add(d)) {
                Some(n) => int_u = n,
                None => {
                    int_u = u64::MAX;
                    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                        pos += 1;
                    }
                    break;
                }
            }
        }

        // Optional fractional part
        let mut frac_attos: u64 = 0;
        let mut frac_digits: usize = 0;

        if pos < bytes.len() && bytes[pos] == b'.' {
            pos += 1;

            while pos < bytes.len() && bytes[pos].is_ascii_digit() && frac_digits < 18 {
                saw_digit = true;
                let d = (bytes[pos] - b'0') as u64;
                frac_attos = frac_attos * 10 + d;
                frac_digits += 1;
                pos += 1;
            }
        }

        if !saw_digit {
            return None;
        }

        let scl = match scale {
            Some(s) => s,
            None => Parts::parse_scale(&bytes[pos..]).unwrap_or_default(),
        };

        // Left-pad the fractional attos value to 18 digits total
        if frac_digits > 0 {
            let shift = 18 - frac_digits;
            frac_attos *= 10u64.pow(shift as u32);
        }

        Some(ParsedReal {
            negative,
            int_u,
            frac_attos,
            scale: scl,
        })
    }

    /// Parses a decimal seconds string (with optional fractional part) as seconds
    /// since [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)
    /// and returns a [`Parts`] that represents the same instant.
    ///
    /// This is the [`Parts`] equivalent of
    /// [`Dt::from_str_sec_f`](../struct.Dt.html#method.from_str_sec_f).
    ///
    /// - If `scale` is `Some(s)`, the value is interpreted on scale `s`.
    /// - If `scale` is `None`, a trailing scale abbreviation (e.g. `GPS`, `TAI`,
    ///   `UTC`) is parsed from the input. If none is found, `TAI` is used.
    ///
    /// Leading non-numeric characters are skipped until a number start is found
    /// (`+`, `-`, `.`, or digit).
    ///
    /// - Fractional seconds are limited to the first 18 digits (attosecond
    ///   precision); extra digits are truncated.
    /// - Oversized integer parts set the integer component to `u64::MAX`.
    /// - Inputs longer than [`STRTIME_SIZE`] are rejected.
    /// - Returns `None` only for completely unparseable input.
    ///
    /// The returned [`Parts`] has its [`timestamp`](Parts::timestamp) field set to a
    /// [`Timestamp`] using [`Epoch::Noon2000`] (attoseconds since the library epoch).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Scale, civil_parts::Parts};
    ///
    /// let p = Parts::from_str_sec_f("1700000000.123456789012345678", Some(Scale::TAI)).unwrap();
    /// let dt = p.to_dt().unwrap();
    /// assert_eq!(dt.to_sec64_floor(), 1700000000);
    ///
    /// // Trailing scale is recognized when scale arg is None
    /// let p = Parts::from_str_sec_f("42.75 GPS", None).unwrap();
    /// assert_eq!(p.scale, Scale::GPS);
    /// ```
    pub fn from_str_sec_f(s: &str, scale: Option<Scale>) -> Option<Parts> {
        let parsed = Self::parse_str_f(s.as_bytes(), scale)?;

        let int_attos = (parsed.int_u as i128) * ATTOS_PER_SEC_I128;
        let frac_attos = parsed.frac_attos as i128;

        let total_attos = if parsed.negative {
            -(int_attos + frac_attos)
        } else {
            int_attos + frac_attos
        };

        let parts = Parts {
            timestamp: Some(Timestamp {
                attos: total_attos,
                epoch: Epoch::Noon2000,
            }),
            scale: parsed.scale,
            ..Default::default()
        };

        Some(parts)
    }

    /// Parses a decimal Julian Date string (with optional fractional part) and returns
    /// a [`Parts`] that represents the same instant.
    ///
    /// - If `scale` is `Some(s)`, the JD value is interpreted on scale `s`.
    /// - If `scale` is `None`, a trailing scale abbreviation (e.g. `TT`, `TDB`, `TAI`)
    ///   is parsed from the input. If none is found, `TAI` is used.
    ///
    /// Leading non-numeric characters are skipped until a number start is found
    /// (`+`, `-`, `.`, or digit).
    ///
    /// - Fractional days are limited to the first 18 digits (attosecond precision
    ///   after conversion); extra digits are truncated.
    /// - Oversized integer parts set the integer component to `u64::MAX`.
    /// - Day→attosecond conversion saturates at `i128` bounds (a full `u64::MAX`
    ///   day count does not fit in attoseconds).
    /// - Inputs longer than [`STRTIME_SIZE`] are rejected.
    /// - Returns `None` only for completely unparseable input.
    ///
    /// The returned [`Parts`] has its [`timestamp`](Parts::timestamp) field set to a
    /// [`Timestamp`] using [`Epoch::Noon2000`] (attoseconds since the library epoch).
    /// JD 2451545.0 corresponds to attos = 0 (the library epoch).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Scale, civil_parts::Parts};
    ///
    /// // 2000-01-01 noon (JD 2451545.0) on TAI
    /// let p = Parts::from_str_jd_f("2451545.0", Some(Scale::TAI)).unwrap();
    /// let dt = p.to_dt().unwrap();
    /// assert_eq!(dt.to_jd(), (2_451_545, 0));
    ///
    /// // Fractional JD + trailing scale
    /// let p = Parts::from_str_jd_f("2451545.5 TT", None).unwrap();
    /// assert_eq!(p.scale, Scale::TT);
    /// ```
    pub fn from_str_jd_f(s: &str, scale: Option<Scale>) -> Option<Parts> {
        let parsed = Self::parse_str_f(s.as_bytes(), scale)?;

        // Build signed JD components. The integer part + frac_attos (scaled to 1e18)
        // together represent the full JD as a real number of days.
        let jd_days: i128 = if parsed.negative {
            -(parsed.int_u as i128)
        } else {
            parsed.int_u as i128
        };
        let jd_frac: i128 = if parsed.negative {
            -(parsed.frac_attos as i128)
        } else {
            parsed.frac_attos as i128
        };

        // Convert the signed JD (days + fractional day) to attoseconds since JD epoch 0.
        // 1 fractional day unit in frac_attos corresponds to SEC_PER_DAY seconds.
        // Saturate: a saturated u64::MAX day count × ATTOS_PER_DAY does not fit in i128.
        let jd_attos = jd_days
            .saturating_mul(ATTOS_PER_DAY)
            .saturating_add(jd_frac.saturating_mul(SEC_PER_DAY));

        // The library's Noon2000 epoch is exactly JD 2451545.0, so subtract its offset.
        let epoch_offset = JD_2000_2_451_545_I128 * ATTOS_PER_DAY;
        let total_attos = jd_attos.saturating_sub(epoch_offset);

        let parts = Parts {
            timestamp: Some(Timestamp {
                attos: total_attos,
                epoch: Epoch::Noon2000,
            }),
            scale: parsed.scale,
            ..Default::default()
        };

        Some(parts)
    }

    /// Parses a decimal Modified Julian Date string (with optional fractional part) and returns
    /// a [`Parts`] that represents the same instant.
    ///
    /// - If `scale` is `Some(s)`, the MJD value is interpreted on scale `s`.
    /// - If `scale` is `None`, a trailing scale abbreviation (e.g. `TT`, `TDB`, `TAI`)
    ///   is parsed from the input. If none is found, `TAI` is used.
    ///
    /// Leading non-numeric characters are skipped until a number start is found
    /// (`+`, `-`, `.`, or digit).
    ///
    /// - Fractional days are limited to the first 18 digits (attosecond precision
    ///   after conversion); extra digits are truncated.
    /// - Oversized integer parts set the integer component to `u64::MAX`.
    /// - Day→attosecond conversion saturates at `i128` bounds (a full `u64::MAX`
    ///   day count does not fit in attoseconds).
    /// - Inputs longer than [`STRTIME_SIZE`] are rejected.
    /// - Returns `None` only for completely unparseable input.
    ///
    /// The returned [`Parts`] has its [`timestamp`](Parts::timestamp) field set to a
    /// [`Timestamp`] using [`Epoch::Noon2000`] (attoseconds since the library epoch).
    /// MJD 51544.5 corresponds to attos = 0 (the library epoch).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Scale, civil_parts::Parts};
    ///
    /// // 2000-01-01 noon (MJD 51544.5) on TAI
    /// let p = Parts::from_str_mjd_f("51544.5", Some(Scale::TAI)).unwrap();
    /// let dt = p.to_dt().unwrap();
    /// assert_eq!(dt.to_jd(), (2_451_545, 0));
    ///
    /// // Fractional MJD + trailing scale
    /// let p = Parts::from_str_mjd_f("51544.75 TT", None).unwrap();
    /// assert_eq!(p.scale, Scale::TT);
    /// ```
    pub fn from_str_mjd_f(s: &str, scale: Option<Scale>) -> Option<Parts> {
        let parsed = Self::parse_str_f(s.as_bytes(), scale)?;

        // Build signed MJD components.
        let mjd_days: i128 = if parsed.negative {
            -(parsed.int_u as i128)
        } else {
            parsed.int_u as i128
        };
        let mjd_frac: i128 = if parsed.negative {
            -(parsed.frac_attos as i128)
        } else {
            parsed.frac_attos as i128
        };

        // Convert MJD to JD by adding the 2400000.5 day offset.
        // MJD = JD - 2400000.5   =>   JD = MJD + 2400000.5
        // Saturate on overflow: a saturated u64::MAX day count × ATTOS_PER_DAY
        // does not fit in i128.
        let mut jd_days = mjd_days.saturating_add(2_400_000);
        let mut sub_day_attos = mjd_frac
            .saturating_mul(SEC_PER_DAY)
            .saturating_add(ATTOS_PER_HALF_DAY);

        // Normalize sub-day attos (handle carry/borrow when adding the .5 offset)
        if sub_day_attos >= ATTOS_PER_DAY {
            jd_days = jd_days.saturating_add(1);
            sub_day_attos -= ATTOS_PER_DAY;
        } else if sub_day_attos < 0 {
            jd_days = jd_days.saturating_sub(1);
            sub_day_attos += ATTOS_PER_DAY;
        }

        let jd_attos = jd_days
            .saturating_mul(ATTOS_PER_DAY)
            .saturating_add(sub_day_attos);

        // The library's Noon2000 epoch is exactly JD 2451545.0, so subtract its offset.
        let epoch_offset = JD_2000_2_451_545_I128 * ATTOS_PER_DAY;
        let total_attos = jd_attos.saturating_sub(epoch_offset);

        let parts = Parts {
            timestamp: Some(Timestamp {
                attos: total_attos,
                epoch: Epoch::Noon2000,
            }),
            scale: parsed.scale,
            ..Default::default()
        };

        Some(parts)
    }
}

/// Day → month → year after a leading English weekday (RFC 2822 / HTTP-date style).
///
/// Day and month required; year required. Month may be numeric or an English name
/// (first three letters). Hyphen between month and year is a field separator
/// (`07-Feb-2015`); a year sign is `+digit`, or `-digit` only after whitespace or
/// another sign (`07 Feb -4714`, `07-Feb--4714`). Advances `*pos` past the year.
#[inline(always)]
fn parse_day_first_ymd(
    bytes: &[u8],
    pos: &mut usize,
    len_: usize,
    tp: &mut Parts,
) -> Result<(), DtErr> {
    // Day (1–2 digits)
    if *pos >= len_ || !bytes[*pos].is_ascii_digit() {
        return Err(an_err!(DtErrKind::ExpectedDay));
    }
    let day = take_1_2_digits(bytes, pos, len_);
    tp.day = Some(if day == 0 { 1 } else { day });

    // Skip separators to month
    while *pos < len_ && (bytes[*pos].is_ascii_whitespace() || bytes[*pos].is_ascii_punctuation()) {
        *pos += 1;
    }
    if *pos >= len_ {
        return Err(an_err!(DtErrKind::ExpectedMonth));
    }

    // Month: 1–2 digits or English name
    if bytes[*pos].is_ascii_digit() {
        let mo = take_1_2_digits(bytes, pos, len_);
        tp.mo = Some(if mo == 0 { 1 } else { mo });
    } else if bytes[*pos].is_ascii_alphabetic() && *pos + 3 <= len_ {
        tp.mo = Some(
            parse_month_name_abbrev(&bytes[*pos..]).ok_or(an_err!(DtErrKind::InvalidMonthName))?,
        );
        // Consume the month token so `-` in `07-Feb-2015` is a field separator.
        while *pos < len_ && bytes[*pos].is_ascii_alphabetic() {
            *pos += 1;
        }
    } else {
        return Err(an_err!(DtErrKind::ExpectedMonth));
    }

    // Skip to year (digit or signed year). Stop before bare `T`.
    // `-`+digits is a separator unless after ws/sign (`07-Feb-2015` vs `07 Feb -4714`).
    while *pos < len_ {
        let b = bytes[*pos];
        if b.is_ascii_digit() {
            break;
        }
        if matches!(b, b'T' | b't') {
            return Err(an_err!(DtErrKind::ExpectedYear));
        }
        if matches!(b, b'+' | b'-')
            && *pos + 1 < len_
            && bytes[*pos + 1].is_ascii_digit()
            && (b == b'+'
                || (*pos > 0
                    && (bytes[*pos - 1].is_ascii_whitespace()
                        || matches!(bytes[*pos - 1], b'+' | b'-'))))
        {
            break;
        }
        *pos += 1;
    }
    if *pos >= len_ {
        return Err(an_err!(DtErrKind::ExpectedYear));
    }

    // Year (optional sign + digits); checked accum matches year-first path.
    let negative_year = match bytes[*pos] {
        b'-' => {
            *pos += 1;
            true
        }
        b'+' => {
            *pos += 1;
            false
        }
        _ => false,
    };
    if *pos >= len_ || !bytes[*pos].is_ascii_digit() {
        return Err(an_err!(DtErrKind::ExpectedYear));
    }
    let mut year: i64 = 0;
    while *pos < len_ && bytes[*pos].is_ascii_digit() {
        let digit = (bytes[*pos] - b'0') as i64;
        year = year
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit))
            .ok_or_else(|| an_err!(DtErrKind::YearOutOfRange))?;
        *pos += 1;
    }
    tp.yr = Some(if negative_year { -year } else { year });
    Ok(())
}

/// Caller must ensure `*pos` is on an ASCII digit.
fn take_1_2_digits(bytes: &[u8], pos: &mut usize, len_: usize) -> u8 {
    let mut n = bytes[*pos] - b'0';
    *pos += 1;
    if *pos < len_ && bytes[*pos].is_ascii_digit() {
        n = n * 10 + (bytes[*pos] - b'0');
        *pos += 1;
    }
    n
}

/// Parse a day-of-year field in one pass: three slots
/// `[digit|space][digit|space][digit]`, not followed by another digit
/// (so `0401` stays calendar `04-01`, not DOY).
///
/// Spaces pad and do not force a ×10 (same as before: `"  9"` → 9, `"1 1"` → 11).
/// On success advances `*pos` past the field; on failure leaves `*pos` unchanged.
#[inline(always)]
fn try_parse_doy(bytes: &[u8], pos: &mut usize, len_: usize) -> Option<u16> {
    let start = *pos;
    let mut p = start;
    let mut doy: u16 = 0;

    // slot 1: digit or space
    if p >= len_ {
        return None;
    }
    let b = bytes[p];
    if b.is_ascii_digit() {
        doy = (b - b'0') as u16;
    } else if b != b' ' {
        return None;
    }
    p += 1;

    // slot 2: digit or space
    if p >= len_ {
        return None;
    }
    let b = bytes[p];
    if b.is_ascii_digit() {
        doy = doy * 10 + (b - b'0') as u16;
    } else if b != b' ' {
        return None;
    }
    p += 1;

    // slot 3: must be digit
    if p >= len_ || !bytes[p].is_ascii_digit() {
        return None;
    }
    doy = doy * 10 + (bytes[p] - b'0') as u16;
    p += 1;

    // not a 4+ digit run (calendar mo+day glued, etc.)
    if p < len_ && bytes[p].is_ascii_digit() {
        return None;
    }

    *pos = p;
    Some(doy)
}

enum Word {
    None,
    Sec,
    Jd,
    Mjd,
}

/// Detect `JD` (2 letters), `MJD` / `SEC` (3 letters) at `pos`.
/// `JD` only needs two bytes so forms like `JD1` / `jd2451545` match.
#[inline(always)]
fn detect_word(bytes: &[u8], b: u8, pos: usize, len_: usize) -> Word {
    match b {
        b'J' | b'j' => {
            // "JD…" — two bytes is enough
            if pos + 2 <= len_ && bytes[pos + 1].eq_ignore_ascii_case(&b'D') {
                Word::Jd
            } else {
                Word::None
            }
        }
        b'M' | b'm' => {
            if pos + 3 <= len_
                && bytes[pos + 1].eq_ignore_ascii_case(&b'J')
                && bytes[pos + 2].eq_ignore_ascii_case(&b'D')
            {
                Word::Mjd
            } else {
                Word::None
            }
        }
        b'S' | b's' => {
            if pos + 3 <= len_
                && bytes[pos + 1].eq_ignore_ascii_case(&b'E')
                && bytes[pos + 2].eq_ignore_ascii_case(&b'C')
            {
                Word::Sec
            } else {
                Word::None
            }
        }
        _ => Word::None,
    }
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
