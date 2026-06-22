use crate::{
    ATTOS_PER_SEC_I128, DtErr, DtErrKind, Epoch, Parser, Parts, STRTIME_SIZE, Scale, SecF,
    Timestamp, an_err,
};

impl Parts {
    /// Parser equivalent to `strptime` with a provided format string.
    ///
    /// The parser populates a [`Parts`] struct. After successful parsing,
    /// [`Self::finish`] is called automatically to apply defaults and validation.
    ///
    /// ## Parameters
    ///
    /// - `fmt`: The format string containing `%` directives.
    /// - `input`: The string to parse.
    /// - `inp_can_end_before_fmt`: If `true`, the input may end before the format
    ///   string is fully consumed (extra format specifiers are ignored).
    /// - `fmt_can_end_before_inp`: If `true`, the format may end before the input
    ///   is fully consumed (trailing characters in the input are allowed).
    /// - `allow_partial_date`: If `true`, a missing month/day will be defaulted
    ///   to `1` instead of returning an [`Incomplete`] error.
    ///
    /// ## Supported Directives
    ///
    /// The format string supports literal characters and the following `%` directives.
    /// Literal non-whitespace characters must match the input exactly.
    /// Whitespace in the format matches (and consumes) any leading ASCII whitespace in the input.
    ///
    /// Many directives accept **format extensions** right after `%`:
    /// - **Flags**: `-` (no pad), `_` (space pad), `0` (zero pad), `^`/`#` (treated as default)
    /// - **Width**: 1–3 digits (affects numeric field width / padding expectations)
    /// - **Colons** (only for `%z`): `:`, `::`, `:::` to control offset format
    ///
    /// ### Year / Century / Unbounded
    /// - `%Y` — Four-digit year (e.g. `2024`). Supports sign, flags, and width.
    /// - `%y` — Two-digit year (`00`–`99`; `00`–`68` → 2000+, `69`–`99` → 1900s).
    /// - `%C` — Century (`00`–`99`).
    /// - `%G` — Four-digit ISO week-based year.
    /// - `%g` — Two-digit ISO week-based year (same century rule as `%y`).
    /// - `%*` — **Unbounded year** (arbitrary length, supports negative years). *Library extension.*
    ///
    /// ### Month
    /// - `%m` — Month number `01`–`12`.
    /// - `%B` — Full English month name (e.g. `January`).
    /// - `%b`, `%h` — Abbreviated English month name (3 letters, e.g. `Jan`).
    ///
    /// ### Day
    /// - `%d`, `%e` — Day of month `01`–`31` (`%e` allows space padding).
    /// - `%j` — Day of year `001`–`366`.
    ///
    /// ### Time of day
    /// - `%H`, `%k` — Hour `00`–`23` (24-hour clock; `%k` allows space padding).
    /// - `%I`, `%l` — Hour `01`–`12` (12-hour clock).
    /// - `%M` — Minute `00`–`59`.
    /// - `%S` — Second `00`–`60` (leap second allowed).
    /// - `%f`, `%N` — Fractional seconds (up to 18 digits = attoseconds).
    ///   Width controls precision (`%3f` = ms, `%6N` = µs, `%9f` = ns, etc.).
    ///   Both accept an optional leading `.` in the input.
    /// - `%.f`, `%.N`, `%.3f`, `%.6N`, ... — Same fractional parsing, but the
    ///   dot before the fraction is **optional** in the input (consumes literal `.` if present).
    /// - `%P`, `%p` — `AM`/`PM` indicator (case-insensitive).
    ///
    /// Fractional seconds directives also work with timestamp directives.
    ///
    /// ### Weekday / Week number
    /// - `%A` — Full English weekday name (e.g. `Monday`).
    /// - `%a` — Abbreviated English weekday name (3 letters, e.g. `Mon`).
    /// - `%u` — Weekday number Monday=`1` … Sunday=`7`.
    /// - `%w` — Weekday number Sunday=`0` … Saturday=`6`.
    /// - `%U` — Week number (Sunday-first week), `00`–`53`.
    /// - `%W` — Week number (Monday-first week), `00`–`53`.
    /// - `%V` — ISO 8601 week number `01`–`53`.
    ///
    /// ### Timezone, Offset & Scale
    /// - `%z` — Timezone offset. Colon count selects format:
    ///   - `%z`   → `±HH[MM[SS]]` (minutes/seconds optional)
    ///   - `%:z`  → `±HH:MM` (minutes required)
    ///   - `%::z` → `±HH:MM:SS` (seconds optional)
    ///   - `%:::z` → `±HH:MM:SS` (more flexible)
    /// - `%Q` — IANA timezone name (e.g. `America/New_York`) **or** numeric offset
    ///   (if input starts with `+`/`-`). *Library extension.*
    /// - `%L` — Time scale abbreviation (e.g. `TAI`, `UTC`, `GPS`). See [`Scale`].
    ///   *Library extension.*
    ///
    /// ### Shortcuts (compound directives)
    /// - `%F` — Equivalent to `%Y-%m-%d` (ISO date).
    /// - `%D` — Equivalent to `%m/%d/%y` (US date).
    /// - `%T` — Equivalent to `%H:%M:%S`.
    /// - `%R` — Equivalent to `%H:%M`.
    ///
    /// ### Other
    /// - `%%` — Literal `%` character.
    /// - `%s` — Unix timestamp (seconds since 1970-01-01 00:00 UTC, can be negative).
    /// - `%J` — Seconds since 2000-01-01 12:00 TAI (J2000.0 noon epoch).
    /// - `%n`, `%t` — Any whitespace (consumes it from input).
    ///
    /// ### Unsupported / Unknown
    /// - `%c`, `%r`, `%x`, `%X`, `%Z` → [`DtErrKind::UnsupportedItem`]
    /// - Any other unknown directive character → [`DtErrKind::UnknownItem`]
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] containing one of the following [`DtErrKind`] variants:
    ///
    /// ### Format string errors
    ///
    /// - [`DtErrKind::TruncatedDirective`] — The format string ended immediately
    ///   after a `%` or after a `.` in a fractional directive (e.g. `%.`).
    /// - [`DtErrKind::UnknownItem`] — Unknown `%` directive character.
    /// - [`DtErrKind::UnsupportedItem`] — Known but unsupported directive
    ///   (e.g. `%c`, `%r`, `%x`, `%X`, `%Z`).
    /// - [`DtErrKind::BadFractional`] — Malformed fractional directive
    ///   (e.g. `%.x` where `x` is not `f` or `N`).
    ///
    /// ### Input parsing errors
    ///
    /// - [`DtErrKind::UnexpectedInputEnd`] — Input ended before a required value
    ///   could be parsed.
    /// - `Expected*` variants:
    ///   - [`DtErrKind::ExpectedYear`]
    ///   - [`DtErrKind::ExpectedMonth`]
    ///   - [`DtErrKind::ExpectedDay`]
    ///   - [`DtErrKind::ExpectedDayOfYear`]
    ///   - [`DtErrKind::ExpectedHour`]
    ///   - [`DtErrKind::ExpectedMinute`]
    ///   - [`DtErrKind::ExpectedSecond`]
    ///   - [`DtErrKind::ExpectedFractionalSeconds`]
    ///   - [`DtErrKind::ExpectedTimestamp`]
    ///   - [`DtErrKind::ExpectedWeekNumber`]
    ///   - [`DtErrKind::ExpectedWeekdayNumber`]
    /// - [`DtErrKind::MismatchedLiteral`] — A literal character from the format
    ///   string did not match the input.
    /// - [`DtErrKind::OutOfRange`] — A numeric value was parsed but is outside
    ///   the valid range for that component (e.g. month 13, hour 25, day 32).
    /// - [`DtErrKind::InvalidName`] — Unrecognized month name, weekday name,
    ///   or `am`/`pm` value.
    /// - [`DtErrKind::InvalidTimezoneOffset`] — Invalid or malformed timezone
    ///   offset / IANA name.
    /// - [`DtErrKind::MustStartWith`] — Timezone offset did not start with
    ///   `+` or `-`.
    ///
    /// ### Post-processing / validation errors
    ///
    /// - [`DtErrKind::Incomplete`] — Required date components (month/day) were
    ///   missing and `allow_partial_date` was `false`.
    /// - [`DtErrKind::TrailingCharacters`] — The input contained trailing
    ///   characters after parsing and `fmt_can_end_before_inp` was `false`.
    ///
    /// Because [`DtErrKind`] is `#[non_exhaustive]`, additional variants may
    /// appear in the future. You can match on the variants you care about and
    /// use a wildcard arm for the rest.
    ///
    /// The concrete error kind is available via [`DtErr::kind()`] (or by
    /// iterating [`DtErr::trace()`] if the error was chained with context
    /// higher up the call stack).
    pub fn from_str(
        fmt: &str,
        input: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Parts, DtErr> {
        let mut parts = Parts::new_utc();
        let mut parser = Parser::new(
            fmt.as_bytes(),
            input.as_bytes(),
            &mut parts,
            inp_can_end_before_fmt,
        );
        parser.parse()?;
        if parser.inp.is_empty() || fmt_can_end_before_inp {
            // All input consumed → finalize
            parts.finish(allow_partial_date)?;
            Ok(parts)
        } else {
            // Trailing characters remain
            Err(an_err!(DtErrKind::TrailingCharacters))
        }
    }

    /// Finalizes a [`Parts`] after parsing by applying sensible defaults and
    /// performing validation.
    ///
    /// This is called automatically by the various parsing paths (`from_str`,
    /// CCSDS parsers, etc.). It ensures the struct is in a consistent state
    /// before being turned into a full [`Dt`] or passed to other converters.
    ///
    /// ## Behavior
    ///
    /// - If a Unix timestamp is present then no action is taken.
    /// - Date completeness is checked in this priority order:
    ///   1. Calendar date (`year`, `month`, `day`)
    ///   2. Ordinal date (`year`, `day_of_year`)
    ///   3. ISO week date (`iso_week_year`, `iso_week`)
    /// - If `allow_partial_date` is `true`, missing month/day are defaulted to `1`.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if no valid date representation is present.
    #[inline(always)]
    pub fn finish(&mut self, allow_partial_date: bool) -> Result<(), DtErr> {
        if self.timestamp.is_none() {
            let has_calendar_date = if allow_partial_date {
                if self.day.is_none() {
                    self.day = Some(1);
                }
                if self.mo.is_none() {
                    self.mo = Some(1);
                }
                self.yr.is_some()
            } else {
                self.yr.is_some() && self.mo.is_some() && self.day.is_some()
            };
            let has_ordinal_date = self.yr.is_some() && self.day_of_yr.is_some();
            let has_iso_week_date = self.iso_wk_yr.is_some() && self.iso_wk.is_some();

            if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
                return Err(an_err!(DtErrKind::Incomplete));
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn parse_sec_f(s: &str, scale: Option<Scale>) -> Option<SecF> {
        let bytes = s.as_bytes();
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

        // Integer part (may be empty when we landed on '.')
        let mut int_u: u64 = 0;
        let mut saw_digit = false;

        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            saw_digit = true;
            let d = (bytes[pos] - b'0') as u64;
            if int_u > u64::MAX / 10 {
                int_u = u64::MAX;
                pos += 1;
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
                break;
            } else {
                int_u = int_u * 10 + d;
                pos += 1;
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

        Some(SecF {
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
    /// [`Dt::from_str_sec_f`](crate::Dt::from_str_sec_f).
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
    /// - Oversized integer parts saturate to the limits of `i64` (because
    ///   [`Parts`] stores the offset via [`TimestampSec::Noon2000`]).
    /// - Inputs longer than [`STRTIME_SIZE`] are rejected.
    /// - Returns `None` only for completely unparseable input.
    ///
    /// The returned [`Parts`] has its `timestamp_sec` set to a `Noon2000` value
    /// (seconds since the library epoch) plus the fractional `attos`. Calling
    /// [`.to_dt()`](Self::to_dt) on it produces the equivalent instant.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Scale, civil_parts::Parts};
    ///
    /// let p = Parts::from_str_sec_f("1700000000.123456789012345678", Some(Scale::TAI)).unwrap();
    /// let dt = p.to_dt().unwrap();
    /// assert_eq!(dt.to_sec64(), 1700000000);
    ///
    /// // Trailing scale is recognized when scale arg is None
    /// let p = Parts::from_str_sec_f("42.75 GPS", None).unwrap();
    /// assert_eq!(p.scale, Scale::GPS);
    /// ```
    /// Shared parser for decimal "seconds + optional fraction" input.
    ///
    /// Used by both [`Parts::from_str_sec_f`] and [`Dt::from_str_sec_f`].
    /// Returns the raw numeric components + resolved scale; the caller decides
    /// how to materialize the value (full attos for `Dt`, or Noon2000 timestamp
    /// for `Parts`).
    pub fn from_str_sec_f(s: &str, scale: Option<Scale>) -> Option<Parts> {
        let parsed = Self::parse_sec_f(s, scale)?;

        // Combine integer seconds + fractional attoseconds into one i128 value.
        // This replaces the old TimestampSec + separate attos split.
        let int_attos = (parsed.int_u as i128) * ATTOS_PER_SEC_I128;
        let frac_attos = parsed.frac_attos as i128;

        let total_attos = if parsed.negative {
            -(int_attos + frac_attos)
        } else {
            int_attos + frac_attos
        };

        let mut parts = Parts::default();
        parts.timestamp = Some(Timestamp {
            attos: total_attos,
            epoch: Epoch::Noon2000,
        });
        parts.scale = parsed.scale;

        Some(parts)
    }
}
