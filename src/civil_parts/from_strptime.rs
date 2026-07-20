use crate::{DtErr, DtErrKind, Parser, Parts, Weekday, an_err};

impl Parts {
    /// Parser equivalent to `strptime` with a provided format string.
    ///
    /// The parser populates a [`Parts`] struct. After successful parsing,
    /// [`Parts::finish`](#method.finish) is called automatically to apply defaults and validation.
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
    ///   to `1` instead of returning a [`DtErrKind::Incomplete`](../error/enum.DtErrKind.html#variant.Incomplete) error.
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
    /// - `%L` — Time scale abbreviation (e.g. `TAI`, `UTC`, `GPS`). See [`Scale`](../enum.Scale.html).
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
    ///   This directive greedily consumes any fractional seconds.
    /// - `%J` — Seconds since 2000-01-01 12:00 TAI (2000-01-01 noon epoch), can be
    ///   negative.
    ///   This directive greedily consumes any fractional seconds.
    /// - `%n`, `%t` — Any whitespace (consumes it from input).
    ///
    /// ### Unsupported / Unknown
    /// - `%c`, `%r`, `%x`, `%X`, `%Z` → [`DtErrKind::UnsupportedItem`]
    /// - Any other unknown directive character → [`DtErrKind::UnknownItem`]
    ///
    /// ## Errors
    ///
    /// Returns a [`DtErr`] if parsing fails. The concrete error kind is available via
    /// [`DtErr::kind()`].
    ///
    /// ### Format string errors
    ///
    /// - [`DtErrKind::TruncatedDirective`] — A `%` appeared at the end of the format
    ///   string, or after flags/width/colons with no directive character following it.
    /// - [`DtErrKind::UnexpectedEnd`] — A `%` was followed only by extensions with no
    ///   directive character.
    /// - [`DtErrKind::InvalidFractional`] — A `%.` fractional directive was followed by
    ///   an invalid character (not `f` or `N`).
    /// - [`DtErrKind::ExpectedFractional`] — A `%.` fractional directive was started
    ///   but no directive character followed the dot.
    /// - [`DtErrKind::UnsupportedItem`] — The format contains `%c`, `%r`, `%x`, `%X`,
    ///   or `%Z`.
    /// - [`DtErrKind::UnknownItem`] — The format contains an unrecognized `%` directive.
    ///
    /// ### Input parsing errors
    ///
    /// - [`DtErrKind::UnexpectedEnd`] — The input ended before a required value could
    ///   be parsed.
    /// - `Expected*` variants:
    ///   - [`DtErrKind::ExpectedYear`], [`DtErrKind::ExpectedCentury`],
    ///     [`DtErrKind::ExpectedMonth`], [`DtErrKind::ExpectedDay`],
    ///     [`DtErrKind::ExpectedDayOfYear`], [`DtErrKind::ExpectedHour`],
    ///     [`DtErrKind::ExpectedMinute`], [`DtErrKind::ExpectedSecond`],
    ///     [`DtErrKind::ExpectedFractional`], [`DtErrKind::ExpectedTimestamp`],
    ///     [`DtErrKind::ExpectedWeekNumber`], [`DtErrKind::ExpectedMonWeekday`],
    ///     [`DtErrKind::ExpectedSunWeekday`], [`DtErrKind::ExpectedMonWeek`],
    ///     [`DtErrKind::ExpectedSunWeek`]
    /// - Out-of-range errors:
    ///   - [`DtErrKind::MonthOutOfRange`], [`DtErrKind::DayOutOfRange`],
    ///     [`DtErrKind::DayOfYearOutOfRange`], [`DtErrKind::HourOutOfRange`],
    ///     [`DtErrKind::MinuteOutOfRange`], [`DtErrKind::SecondOutOfRange`],
    ///     [`DtErrKind::IsoWeekOutOfRange`], [`DtErrKind::MonWeekdayOutOfRange`],
    ///     [`DtErrKind::SunWeekdayOutOfRange`]
    /// - [`DtErrKind::MismatchedLiteral`] — A literal character in the format string
    ///   did not match the input.
    /// - Name errors: [`DtErrKind::InvalidMonthName`], [`DtErrKind::InvalidWeekdayName`],
    ///   [`DtErrKind::InvalidMeridiem`].
    ///
    /// ### Timezone and Offset errors
    ///
    /// - [`DtErrKind::OffsetMissingSign`] — A timezone offset (`%z` / `%Q`) did not
    ///   start with `+` or `-`.
    /// - [`DtErrKind::InvalidOffsetHour`] — Invalid hour value in a timezone offset.
    /// - [`DtErrKind::InvalidOffsetMinute`] — Invalid minute value in a timezone offset.
    /// - [`DtErrKind::InvalidOffsetSecond`] — Invalid second value in a timezone offset.
    /// - [`DtErrKind::InvalidOffsetColons`] — Incorrect number of colons or missing
    ///   required colon in a timezone offset.
    /// - [`DtErrKind::InvalidOffset`] — General failure while parsing a numeric
    ///   timezone offset.
    /// - [`DtErrKind::InvalidTimeZone`] — Invalid or unparseable IANA timezone name
    ///   (used by the `%Q` directive).
    ///
    /// ### Post-processing / validation errors
    ///
    /// - [`DtErrKind::TrailingCharacters`] — The input contained trailing characters
    ///   after parsing and `fmt_can_end_before_inp` was `false`.
    /// - [`DtErrKind::Incomplete`] — Required date components (month or day) were
    ///   missing and `allow_partial_date` was `false`.
    ///
    /// The error kind is available via [`DtErr::kind()`].
    pub fn from_strptime(
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

    /// Applies defaults; returns [`DtErrKind::Incomplete`] if no complete date form remains.
    ///
    /// Full YMD always completes (even with mixed week fields). Full ISO week
    /// (`iso_wk_yr`+`iso_wk`) completes and defaults weekday to Monday. With
    /// `allow_partial_date`, missing `iso_wk` is set to 1 when `iso_wk_yr` is
    /// present; calendar month/day default to 1 only when there is no ISO week
    /// fragment at all.
    #[inline(always)]
    pub fn finish(&mut self, allow_partial_date: bool) -> Result<(), DtErr> {
        if self.timestamp.is_some() {
            return Ok(());
        }

        // Resolve ISO week in one pass; `pure_calendar` means no ISO fields left hanging.
        let pure_calendar = match (self.iso_wk_yr.is_some(), self.iso_wk.is_some()) {
            (true, true) => {
                self.wkday.get_or_insert(Weekday::Monday);
                return Ok(());
            }
            (true, false) if allow_partial_date => {
                self.iso_wk = Some(1);
                self.wkday.get_or_insert(Weekday::Monday);
                return Ok(());
            }
            (false, false) => true,
            _ => false, // incomplete ISO fragment
        };

        if self.yr.is_none() {
            return Err(an_err!(DtErrKind::Incomplete));
        }

        if self.mo.is_some() && self.day.is_some() {
            return Ok(());
        }
        if self.day_of_yr.is_some() {
            return Ok(());
        }
        if self.wk_sun.is_some() {
            self.wkday.get_or_insert(Weekday::Sunday);
            return Ok(());
        }
        if self.wk_mon.is_some() {
            self.wkday.get_or_insert(Weekday::Monday);
            return Ok(());
        }

        if allow_partial_date && pure_calendar {
            self.day.get_or_insert(1);
            self.mo.get_or_insert(1);
            return Ok(());
        }

        Err(an_err!(DtErrKind::Incomplete))
    }
}
