use crate::{
    ATTOS_PER_SEC_I128, Dt, DtErr, DtErrKind, Parts, SEC_PER_DAY, SEC_PER_MONTH, SEC_PER_WEEK,
    SEC_PER_YEAR, Scale, StrPTimeFmt, an_err,
};
use core::str::FromStr;

#[cfg(feature = "parse")]
use crate::ParseCfg;

#[cfg(feature = "parse")]
impl FromStr for Dt {
    type Err = DtErr;

    #[inline]
    fn from_str(s: &str) -> Result<Self, DtErr> {
        Dt::from_str_parse(s, &ParseCfg::DEFAULT)
    }
}

#[cfg(not(feature = "parse"))]
impl FromStr for Dt {
    type Err = DtErr;

    #[inline]
    fn from_str(s: &str) -> Result<Self, DtErr> {
        Self::from_str_iso(s)
    }
}

struct ParsedComponent {
    unit: u8,
    signed_int: i64,
    frac_digits: usize,
    frac_num: i64,
}

impl Dt {
    /// Parses a date/time string.
    ///
    /// - When the `parse` feature is enabled: uses the smart auto-parser.
    /// - When the `parse` feature is disabled: falls back to CCSDS format.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // uses impl FromStr but Dt::parse provides the same functionality
    /// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
    ///
    /// let ymd = x.to_ymd();
    /// assert_eq!(ymd.yr(), 2000);
    /// assert_eq!(ymd.mo(), 1);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse)
    /// - [`Dt::from_str_iso`](../struct.Dt.html#method.from_str_iso)
    #[inline(always)]
    pub fn parse(s: &str) -> Result<Self, DtErr> {
        #[cfg(feature = "parse")]
        {
            Self::from_str_parse(s, &ParseCfg::DEFAULT)
        }
        #[cfg(not(feature = "parse"))]
        {
            Self::from_str_iso(s)
        }
    }

    /// Parser equivalent to `strptime` with a provided format string.
    ///
    /// The returned [`Dt`] will be on the `TAI` time scale, converted from whatever
    /// optional time scale (`%L`) was provided in the input. If no time scale was
    /// provided then it's converted from `UTC` -> `TAI`.
    ///
    /// The result is that the [`Dt`]'s `scale` field will be `TAI` and its `target`
    /// field will be whatever time scale it was converted from (`UTC` if no time
    /// scale was in the input).
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
    /// Returns a [`DtErr`] if either the strptime-style parser or the subsequent
    /// conversion from [`Parts`] to [`Dt`] fails.
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
    /// ### Conversion to [`Dt`] errors
    ///
    /// These errors can occur *after* successful parsing, inside [`Parts::to_dt`]:
    ///
    /// - [`DtErrKind::InvalidDate`] or [`DtErrKind::InvalidInput`] — Unable to
    ///   construct a valid date from the parsed components.
    /// - Out-of-range or conflicting field errors (e.g. [`DtErrKind::DayOfYearOutOfRange`],
    ///   [`DtErrKind::IsoWeekOutOfRange`], [`DtErrKind::WeekOutOfRange`], etc.).
    /// - [`DtErrKind::InvalidItem`] — ISO week 53 requested for a year that does not
    ///   contain 53 ISO weeks.
    /// - Feature-dependent errors (when `jiff-tz` is involved):
    ///   - [`DtErrKind::InvalidTimeZone`], [`DtErrKind::InvalidNumber`],
    ///     [`DtErrKind::InvalidBytes`].
    ///
    /// The error kind is available via [`DtErr::kind()`].
    #[inline(always)]
    pub fn from_str(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        Parts::from_str(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_dt()
    }

    /// Parses and validates a `strptime`-style format string into a reusable [`StrPTimeFmt`].
    ///
    /// The format is checked once for syntax errors and unsupported directives,
    /// then stored in a compact fixed-size buffer. The resulting `StrPTimeFmt` is
    /// `Copy`, cheap to clone, and can be used repeatedly with [`StrPTimeFmt::to_dt`]
    /// and [`StrPTimeFmt::to_str`] without re-validating.
    ///
    /// Only ASCII formats up to 256 bytes are accepted.
    ///
    /// ## Parameters
    ///
    /// - `strptime_fmt`: The format string using `%` directives (e.g. `"%Y-%m-%d %H:%M:%S"`,
    ///   `"%F %T"`, `"%Y-%m-%dT%H:%M:%S%.3fZ"`).
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format is:
    /// - Longer than 256 bytes
    /// - Not valid ASCII
    /// - Contains unknown, unsupported, or malformed directives
    #[inline(always)]
    pub fn parse_fmt(strptime_fmt: &str) -> Result<StrPTimeFmt, DtErr> {
        StrPTimeFmt::new(strptime_fmt)
    }

    /// Generalized no alloc parser.
    ///
    /// - Only supports ASCII characters.
    /// - This function is considerably faster than all other string parsing methods if
    ///   your date-time string is in one of the supported formats.
    /// - Timezones beyond UTC aliases require the `jiff-tz` feature, which requires `std`.
    ///
    /// ## Returns
    ///
    /// - If there is NOT a trailing time scale in the input and the format of the input
    ///   is a typical datetime iso e.g. `2000-01-01T17:00:00` then the time scale is
    ///   assumed to be `UTC` and the [`Dt`] goes through a `UTC` -> `TAI` conversion
    ///   (adding leap seconds).
    /// - If there is NOT a trailing time scale in the input and the format of the input
    ///   is a seconds count, jd, or mjd then the time scale is assumed to be `TAI` and
    ///   no conversion happens.
    /// - If there IS a trailing time scale in the input then the input goes through
    ///   a time scale conversion (regardless of input format) of the provided time
    ///   scale -> `TAI`. If the trailing time scale is `TAI` then no conversion occurs.
    ///
    /// A [`Dt`] of the `TAI` time scale is returned.
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
    /// - **`2024 Apr 18, 14:30:25 [America/New_York]`**. Abbreviated month
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
    /// ### Seconds since J2000 Noon
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
    /// - [`Parts::from_str_iso`](../struct.Parts.html#method.from_str_iso)
    #[inline(always)]
    pub fn from_str_iso(s: &str) -> Result<Self, DtErr> {
        Parts::from_str_iso(s)?.to_dt()
    }

    /// Parses a decimal seconds string (with optional fractional part) as seconds
    /// since
    /// [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)
    /// on the chosen time scale.
    ///
    /// The returned [`Dt`] is on the `TAI` time [`Scale`], having been converted
    /// to `TAI` from whatever the **trailing** scale is, or if no scale is provided
    /// then no conversion takes place.
    ///
    /// Leading non-numeric characters are skipped until a number start is found
    /// (`+`, `-`, `.`, or digit).
    ///
    /// - Fractional seconds are limited to the first 18 digits (attosecond
    ///   precision); extra digits are truncated.
    /// - Oversized integer parts saturate instead of failing.
    /// - Inputs longer than [`STRTIME_SIZE`] are rejected.
    /// - Returns `None` only for completely unparseable input (empty, sign/dot
    ///   only, no digits after skipping, etc.).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let d = Dt::from_str_sec_f("1700000000.123456789012345678", Some(Scale::TAI)).unwrap();
    /// assert_eq!(d.to_sec64(), 1700000000);
    ///
    /// // Leading junk is skipped
    /// let d = Dt::from_str_sec_f("ts= -0.00123 suffix", Some(Scale::TAI)).unwrap();
    /// assert!(d.to_attos() < 0);
    ///
    /// // Pure negative fraction
    /// let d = Dt::from_str_sec_f("-.5", Some(Scale::TT)).unwrap();
    /// assert!(d.to_attos() < 0);
    ///
    /// // Scale parsed from trailing abbreviation when passing None
    /// let d = Dt::from_str_sec_f("42.75 GPS", None).unwrap();
    /// assert_eq!(d.target, Scale::GPS);
    ///
    /// // 1 attosecond
    /// let d = Dt::from_str_sec_f("0.000000000000000001", Some(Scale::TAI)).unwrap();
    /// assert_eq!(d.to_attos() % 1_000_000_000_000_000_000, 1);
    /// ```
    pub fn from_str_sec_f(s: &str, scale: Option<Scale>) -> Option<Dt> {
        let parsed = Parts::parse_str_f(s.as_bytes(), scale)?;

        let int_attos = (parsed.int_u as i128) * ATTOS_PER_SEC_I128;
        let signed_attos = if parsed.negative {
            -int_attos - (parsed.frac_attos as i128)
        } else {
            int_attos + (parsed.frac_attos as i128)
        };

        Some(Dt::from_attos(signed_attos, parsed.scale))
    }

    /// Parses a decimal Julian Date string (with optional fractional part).
    ///
    /// The returned [`Dt`] is on the `TAI` time [`Scale`], having been converted
    /// to `TAI` from whatever the **trailing** scale is, or if no scale is provided
    /// then no conversion takes place.
    ///
    /// Leading junk is skipped the same way as [`Dt::from_str_sec_f`].
    /// Fractional day precision up to 18 digits.
    ///
    /// Returns `None` for unparseable input.
    ///
    /// JD 2451545.0 is the library epoch (2000-01-01 noon).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let d = Dt::from_str_jd_f("2451545.0", Some(Scale::TAI)).unwrap();
    /// assert_eq!(d.to_jd(), (2_451_545, 0));
    ///
    /// let d = Dt::from_str_jd_f("2451545.25 TT", None).unwrap();
    /// assert_eq!(d.target, Scale::TT);
    ///
    /// let d = Dt::from_str_jd_f("2451544.5", Some(Scale::TAI)).unwrap();
    /// assert!(d.to_attos() < 0);
    /// ```
    pub fn from_str_jd_f(s: &str, scale: Option<Scale>) -> Option<Dt> {
        Parts::from_str_jd_f(s, scale).and_then(|p| p.to_dt().ok())
    }

    /// Parses a decimal Modified Julian Date string (with optional fractional part).
    ///
    /// The returned [`Dt`] is on the `TAI` time [`Scale`], having been converted
    /// to `TAI` from whatever the **trailing** scale is, or if no scale is provided
    /// then no conversion takes place.
    ///
    /// Leading junk is skipped the same way as [`Dt::from_str_sec_f`].
    /// Fractional day precision up to 18 digits.
    ///
    /// Returns `None` for unparseable input.
    ///
    /// MJD 51544.5 is the library epoch (2000-01-01 noon).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let d = Dt::from_str_mjd_f("51544.5", Some(Scale::TAI)).unwrap();
    /// assert_eq!(d.to_jd(), (2_451_545, 0));
    ///
    /// let d = Dt::from_str_mjd_f("51544.25 TT", None).unwrap();
    /// assert_eq!(d.target, Scale::TT);
    ///
    /// let d = Dt::from_str_mjd_f("51543.5", Some(Scale::TAI)).unwrap();
    /// assert!(d.to_attos() < 0);
    /// ```
    pub fn from_str_mjd_f(s: &str, scale: Option<Scale>) -> Option<Dt> {
        Parts::from_str_mjd_f(s, scale).and_then(|p| p.to_dt().ok())
    }

    /// Parses an ISO 8601 duration string into a [`Dt`] representing a pure time interval.
    ///
    /// Supports the full `PnYnMnDTnHnMnS` format (case-insensitive), including:
    /// - Optional leading `+` or `-` sign
    /// - `P` / `p` prefix (required)
    /// - Optional `T` / `t` separator between date and time parts
    /// - Weeks (`W` / `w`)
    /// - Fractional seconds with up to 9 digits of precision (nanosecond resolution;
    ///   the parsed value is scaled to attosecond resolution in the resulting [`Dt`]).
    ///
    /// The returned [`Dt`] is a **duration** (signed interval) on the TAI scale.
    /// It can be added to/subtracted from other `Dt` values, multiplied/divided,
    /// rounded, etc.
    ///
    /// ## Not Reference-Time Aware
    ///
    /// This parser is **not reference-time aware**. Calendar units (`Y`, `M`) are
    /// converted to a fixed number of seconds using standard average lengths
    /// rather than being resolved against a specific date. This makes parsing
    /// fast and allocation-free, but `P1M` always represents exactly the same
    /// duration regardless of context.
    ///
    /// ## Parameters
    ///
    /// - `s`: The ISO 8601 duration string (e.g. `"P1Y2M3DT4H5M6.123456789012345678S"`,
    ///   `"-PT30M"`, `"P7W"`, `"+P1DT12H"`).
    ///
    /// ## Errors
    ///
    /// Returns a [`DtErr`] if parsing fails. The error kind is available via
    /// [`DtErr::kind()`].
    ///
    /// ### Input / structure errors
    ///
    /// - [`DtErrKind::Empty`] — The input string is empty.
    /// - [`DtErrKind::MustStartWith`] — Missing `P` / `p` prefix (after optional leading sign).
    /// - [`DtErrKind::InvalidSyntax`] — Invalid syntax, e.g. `T` with no following time part,
    ///   or more than one `T`/`t` separator.
    /// - [`DtErrKind::TrailingCharacters`] — Additional components appear after a fractional
    ///   seconds value (only the final `S` component may carry a fraction).
    ///
    /// ### Component parsing errors
    ///
    /// - [`DtErrKind::ExpectedValue`] — Expected a numeric value for a component but found none.
    /// - [`DtErrKind::ExpectedFractional`] — A `.` or `,` was present for a fractional part
    ///   but no digits followed.
    /// - [`DtErrKind::ExpectedUnit`] — A number was parsed but no unit designator
    ///   (`Y`/`M`/`W`/`D`/`H`/`S` etc.) followed it.
    /// - [`DtErrKind::InvalidNumber`] — A numeric component could not be parsed as an `i64`
    ///   (typically too large).
    /// - [`DtErrKind::InvalidBytes`] — Internal UTF-8 conversion failure while reading a number
    ///   (should not occur for valid ASCII input).
    /// - [`DtErrKind::InvalidFractional`] — The fractional part digits could not be parsed as an integer.
    /// - [`DtErrKind::FracOutOfRange`] — More than 9 digits were supplied for fractional seconds.
    /// - [`DtErrKind::InvalidItem`] — A fractional part was supplied on a unit other than seconds.
    ///
    /// ### Unit and range errors
    ///
    /// - [`DtErrKind::UnknownItem`] — An unknown unit designator character was used.
    /// - [`DtErrKind::YearOutOfRange`], [`DtErrKind::MonthOutOfRange`],
    ///   [`DtErrKind::WeekOutOfRange`], [`DtErrKind::DayOutOfRange`] — The component value
    ///   (after sign) overflows when multiplied by the corresponding fixed-length constant
    ///   (checked arithmetic).
    pub fn from_iso_duration(s: &str) -> Result<Dt, DtErr> {
        let len = s.len();
        if len == 0 {
            return Err(an_err!(DtErrKind::Empty));
        }

        let b = s.as_bytes();
        let mut i = 0usize;

        // Optional leading sign (+ or -)
        let mut sign: i64 = 1;
        if i < len && matches!(b[i], b'+' | b'-') {
            if b[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        // Must start with P/p
        if i >= len || !matches!(b[i], b'P' | b'p') {
            return Err(an_err!(DtErrKind::MustStartWith));
        }
        i += 1;

        // Find the (single) T/t separator
        let t_pos = b[i..]
            .iter()
            .position(|&c| matches!(c, b'T' | b't'))
            .map(|p| i + p);

        let (date_part, time_part) = match t_pos {
            Some(pos) => {
                if pos == len - 1 {
                    return Err(an_err!(DtErrKind::InvalidSyntax));
                }
                if b[pos + 1..].iter().any(|&c| matches!(c, b'T' | b't')) {
                    return Err(an_err!(DtErrKind::InvalidSyntax));
                }
                (&b[i..pos], &b[pos + 1..])
            }
            None => (&b[i..], &[] as &[u8]),
        };

        let mut has_fraction = false;
        let mut total_nanos: i128 = 0;

        // Both date and time parts now use the same fixed-length logic
        Self::parse_duration_part(date_part, &mut total_nanos, true, sign, &mut has_fraction)?;
        Self::parse_duration_part(time_part, &mut total_nanos, false, sign, &mut has_fraction)?;

        // Convert accumulated nanoseconds to attoseconds and build Dt
        let total_attos = total_nanos * 1_000_000_000i128;
        Ok(Dt::span(total_attos))
    }

    /// Parses a single component (number + optional fraction + unit) from the slice,
    /// advancing the index `i`. Returns `None` when the slice is exhausted.
    fn parse_next_component(
        chars: &[u8],
        i: &mut usize,
        sign: i64,
        has_fraction: &mut bool,
    ) -> Result<Option<ParsedComponent>, DtErr> {
        if *i >= chars.len() {
            return Ok(None);
        }

        if *has_fraction {
            return Err(an_err!(DtErrKind::TrailingCharacters));
        }

        // Parse integer part
        let start = *i;
        while *i < chars.len() && chars[*i].is_ascii_digit() {
            *i += 1;
        }
        if start == *i {
            return Err(an_err!(DtErrKind::ExpectedValue));
        }

        let int_str = core::str::from_utf8(&chars[start..*i])
            .map_err(|e| an_err!(DtErrKind::InvalidBytes, "{}", e))?;
        let int: i64 = int_str.parse().map_err(|e: core::num::ParseIntError| {
            an_err!(DtErrKind::InvalidNumber, "{}: {}", int_str, e)
        })?;

        // Parse optional fraction
        let mut frac_num: i64 = 0;
        let mut frac_digits: usize = 0;
        if *i < chars.len() && matches!(chars[*i], b'.' | b',') {
            *i += 1;
            let frac_start = *i;
            while *i < chars.len() && chars[*i].is_ascii_digit() {
                *i += 1;
            }
            frac_digits = *i - frac_start;
            if frac_digits == 0 {
                return Err(an_err!(DtErrKind::ExpectedFractional));
            }
            if frac_digits > 9 {
                return Err(an_err!(DtErrKind::FracOutOfRange));
            }

            let frac_str = core::str::from_utf8(&chars[frac_start..*i])
                .map_err(|e| an_err!(DtErrKind::InvalidBytes, "{}", e))?;
            frac_num = frac_str.parse().map_err(|e: core::num::ParseIntError| {
                an_err!(DtErrKind::InvalidFractional, "{}: {}", frac_str, e)
            })?;
        }

        // Unit must follow
        if *i >= chars.len() {
            return Err(an_err!(DtErrKind::ExpectedUnit));
        }
        let unit = chars[*i];
        *i += 1;

        // Only seconds support a fractional part
        if frac_digits > 0 {
            if !matches!(unit, b'S' | b's') {
                return Err(an_err!(DtErrKind::InvalidItem));
            }
            *has_fraction = true;
        }

        let signed_int = (int as i128 * sign as i128) as i64;

        Ok(Some(ParsedComponent {
            unit,
            signed_int,
            frac_digits,
            frac_num,
        }))
    }

    /// Helper that parses **one section** of an ISO duration (date or time part)
    /// and accumulates nanoseconds into `total_nanos`.
    ///
    /// Years, months, weeks, and days are converted using the fixed-length
    /// constants (the only sensible semantics for a pure `Dt`).
    fn parse_duration_part(
        chars: &[u8],
        total_nanos: &mut i128,
        is_date: bool,
        sign: i64,
        has_fraction: &mut bool,
    ) -> Result<(), DtErr> {
        let mut i = 0;
        while let Some(comp) = Self::parse_next_component(chars, &mut i, sign, has_fraction)? {
            let contrib_nanos = match (is_date, comp.unit) {
                (true, b'Y' | b'y') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_YEAR)
                        .ok_or_else(|| an_err!(DtErrKind::YearOutOfRange))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'M' | b'm') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_MONTH)
                        .ok_or_else(|| an_err!(DtErrKind::MonthOutOfRange))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'W' | b'w') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_WEEK as i128)
                        .ok_or_else(|| an_err!(DtErrKind::WeekOutOfRange))?;
                    total_secs * 1_000_000_000i128
                }
                (true, b'D' | b'd') => {
                    let total_secs = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_DAY)
                        .ok_or_else(|| an_err!(DtErrKind::DayOutOfRange))?;
                    total_secs * 1_000_000_000i128
                }
                (false, b'H' | b'h') => (comp.signed_int as i128) * 3_600_000_000_000i128,
                (false, b'M' | b'm') => (comp.signed_int as i128) * 60_000_000_000i128,
                (false, b'S' | b's') => {
                    let mut sec_nanos = (comp.signed_int as i128) * 1_000_000_000i128;
                    if comp.frac_digits > 0 {
                        let frac_ns = (comp.frac_num as i128 * sign as i128 * 1_000_000_000i128)
                            / 10i128.pow(comp.frac_digits as u32);
                        sec_nanos += frac_ns;
                    }
                    sec_nanos
                }
                _ => {
                    return Err(an_err!(DtErrKind::UnknownItem, "{}", comp.unit as char));
                }
            };

            *total_nanos = total_nanos.saturating_add(contrib_nanos);
        }
        Ok(())
    }

    /// Parses a media-style duration string.
    ///
    /// Accepts formats like:
    /// - `"0:45"`, `"9:41"`
    /// - `"1:23:45"`
    /// - `"1:07:54:30"`
    /// - `"-1:23:45"`
    ///
    /// ## Errors
    ///
    /// Returns a [`DtErr`] if the input cannot be parsed as a valid media-style
    /// duration. The error kind is available via [`DtErr::kind`].
    ///
    /// This function uses saturating arithmetic, so it never returns range or
    /// overflow errors.
    ///
    /// ### Input / structure errors
    ///
    /// - [`DtErrKind::Empty`] — The string is empty or contains only ASCII whitespace.
    /// - [`DtErrKind::InvalidInput`] — A single minus sign with nothing after it.
    /// - [`DtErrKind::InvalidSyntax`] — The input does not contain exactly 2, 3, or 4
    ///   colon-separated numeric components.
    /// - [`DtErrKind::TrailingCharacters`] — Non-whitespace characters remain after
    ///   the final numeric component.
    ///
    /// ### Parsing errors
    ///
    /// - [`DtErrKind::ExpectedValue`] — A component was expected to begin with a digit
    ///   (either at the start of the string or immediately after a `:`) but did not.
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_media_duration`](../struct.Dt.html#method.to_str_media_duration)
    /// - [`Dt::to_str_lite_media_duration`](../struct.Dt.html#method.to_str_lite_media_duration)
    pub fn from_str_media_duration(input: &str) -> Result<Dt, DtErr> {
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut pos: usize = 0;

        // Skip leading whitespace
        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos == len {
            return Err(an_err!(DtErrKind::Empty));
        }

        // Optional single leading minus
        let negative = if bytes[pos] == b'-' {
            pos += 1;
            if pos == len {
                return Err(an_err!(DtErrKind::InvalidInput));
            }
            true
        } else {
            false
        };

        // Parse up to 4 numeric components separated by ':'
        let mut components: [i128; 4] = [0; 4];
        let mut count: usize = 0;

        loop {
            if count >= 4 {
                break;
            }

            // Parse one number
            if pos >= len || !bytes[pos].is_ascii_digit() {
                return Err(an_err!(DtErrKind::ExpectedValue));
            }

            let mut value: i128 = 0;
            while pos < len && bytes[pos].is_ascii_digit() {
                value = value
                    .saturating_mul(10)
                    .saturating_add((bytes[pos] - b'0') as i128);
                pos += 1;
            }

            components[count] = value;
            count += 1;

            // Check for more components
            if pos >= len || bytes[pos] != b':' {
                break;
            }

            pos += 1; // consume ':'

            // Reject trailing ':' with no number after it
            if pos >= len || !bytes[pos].is_ascii_digit() {
                return Err(an_err!(DtErrKind::ExpectedValue));
            }
        }

        if !(2..=4).contains(&count) {
            return Err(an_err!(DtErrKind::InvalidSyntax));
        }

        // Skip trailing whitespace
        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos != len {
            return Err(an_err!(DtErrKind::TrailingCharacters));
        }

        // Convert to total seconds
        let total_secs: i128 = match count {
            2 => components[0] * 60 + components[1], // M:SS
            3 => components[0] * 3600 + components[1] * 60 + components[2], // H:MM:SS
            4 => components[0] * 86400 + components[1] * 3600 + components[2] * 60 + components[3], // D:H:MM:SS
            _ => unreachable!(),
        };

        let total_secs = if negative { -total_secs } else { total_secs };
        let attos = total_secs.saturating_mul(ATTOS_PER_SEC_I128);

        Ok(Dt::span(attos))
    }
}
