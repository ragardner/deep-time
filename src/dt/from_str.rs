use crate::{
    ATTOS_PER_NS_I128, ATTOS_PER_SEC_I128, Dt, DtErr, DtErrKind, Parts, SEC_PER_DAY, SEC_PER_MONTH,
    SEC_PER_WEEK, SEC_PER_YEAR, Scale, an_err, dt, sec,
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
        Self::from_str(s)
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
    /// When the `parse` feature is enabled, this is equivalent to calling
    /// `Dt::from_str_parse` with `ParseCfg::DEFAULT`. When that feature is
    /// disabled, it uses the inherent
    /// [`Dt::from_str`](../struct.Dt.html#method.from_str) instead.
    ///
    /// This is the same routing as [`str::parse`](core::str::FromStr) /
    /// [`FromStr`] for [`Dt`].
    ///
    /// Both paths accept the text produced by
    /// [`core::fmt::Display`] / `.to_string()`, for example `[86400s TAI>UTC]`.
    /// When the input is that Display form, no scale conversion is performed:
    /// the returned [`Dt`] takes `attos`, `scale`, and `target` from the string
    /// as written. Other inputs follow the rules of the path that is active
    /// (`from_str_parse` or `from_str`).
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
    /// - `Dt::from_str_parse` (requires the `parse` feature)
    /// - [`Dt::from_str`](../struct.Dt.html#method.from_str)
    #[inline(always)]
    pub fn parse(s: &str) -> Result<Self, DtErr> {
        #[cfg(feature = "parse")]
        {
            Self::from_str_parse(s, &ParseCfg::DEFAULT)
        }
        #[cfg(not(feature = "parse"))]
        {
            Self::from_str(s)
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
    pub fn from_strptime(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<Dt, DtErr> {
        Parts::from_strptime(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )?
        .to_dt()
    }

    /// Fast, no-alloc parser for common ISO-like and epoch-style date-time strings.
    ///
    /// Equivalent to [`Parts::from_str`](../civil_parts/struct.Parts.html#method.from_str)
    /// followed by [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt). The
    /// formats and lenience rules below are those of the `Parts` parser; this method
    /// then resolves components to a single instant.
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
    /// - Timezones beyond UTC aliases require the `jiff-tz` or `jiff-tz-bundle` feature
    ///   (both require `alloc`).
    ///
    /// ## Returns
    ///
    /// For the display format (`[86400s TAI>UTC]`, same as [`Dt`] Display), the returned
    /// [`Dt`] undergoes **no** time scale conversion. The [`Dt`]'s `scale` field is
    /// from the 1st time scale abbreviation in the string, the `target` field is
    /// from the 2nd.
    ///
    /// **Everything else** (civil, `SEC`, `JD`, `MJD`, …) is converted to **TAI** from
    /// a given trailing time scale. **If there is no trailing time scale abbreviation** then
    /// for civil datetimes the conversion is UTC -> TAI, and for prefixed formats such as
    /// `SEC`/`JD`/`MJD` no conversion takes place.
    /// The returned value’s `target` is the scale the text was interpreted on
    /// (before conversion to TAI).
    ///
    /// ## Supported formats
    ///
    /// An **optional** library time scale at the end of the input (e.g. `TAI`) is
    /// supported for the civil / `SEC` / `JD` / `MJD` formats below.
    ///
    /// ### Display form (`Dt` text from Display / `.to_string()`)
    ///
    /// #### Format examples
    ///
    /// - **`[86400s TAI>UTC]`**
    /// - **`[-1.5s TT>GPS]`**
    /// - **`[0s TAI>TAI]`**
    ///
    /// #### Notes
    ///
    /// - Parsing this form and formatting with Display round-trip (same attoseconds,
    ///   `scale`, and `target`).
    /// - Anything after the closing `]` is ignored.
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
    /// - **`2024`**, **`2024-03`**, **`2024 Mar`** — partial dates (missing month/day → `1`)
    /// - **`2024-04-18T9:3:5.5`**, **`2024-04-18T143025`** — flexible / compact time
    ///
    /// #### Date forms
    ///
    /// Year digits are taken **literally** (no century window): `99-01-01` is year 99 AD.
    /// Year overflow during accumulation yields
    /// [`DtErrKind::YearOutOfRange`](../error/enum.DtErrKind.html#variant.YearOutOfRange).
    ///
    /// **Weekday first** → day, month, year (required). Hyphen before the year is a
    /// separator (`07-Feb-2015`); a signed year needs space or a doubled sign
    /// (`07 Feb -4714`, `07-Feb--4714`, `+2015`).
    ///
    /// **Otherwise year first.** After an optional sign and year digits, exactly one of:
    ///
    /// 1. **ISO week** — `W`/`w` immediately after the year (or after a non-letter
    ///    separator such as `-`): e.g. `2024-W11`, `2024W114`.
    ///    - Week number required right after `W` (1–2 digits); `0` becomes week `1`.
    ///    - Optional weekday: `-4` or basic trailing digit `1..=7` (Monday=`1` … Sunday=`7`);
    ///      if omitted, Monday is used when resolving the instant.
    ///    - This is **not** strftime `%W` (Monday week-of-year on a calendar year).
    /// 2. **Day of year** — three slots `[digit|space][digit|space][digit]` not followed
    ///    by another digit (so `2024-0401` is calendar `04-01`, not DOY). Space padding
    ///    is allowed (`2024-  9`).
    /// 3. **Calendar month/day** — numeric month (1–2 digits) or English month name
    ///    (abbrev or full; matched from the first three letters), then day (1–2 digits).
    ///
    /// **Partial calendar dates** (year-first only) default missing fields to `1`
    /// (January / day 1):
    ///
    /// - Year only: `2024`, `2024-` → 1 January.
    /// - Year-month: `2024-03`, `2024 Mar` → day 1.
    /// - Explicit zero month/day digits are treated like omitted (`2024-00`, `2024-03-0` → 1).
    /// - Year or year-month may be followed by time: `2024T12:00`, `2024-03T12:00`.
    ///
    /// #### Time (optional)
    ///
    /// - Usually introduced by `T`/`t`, space, or another non-digit separator; compact
    ///   glued times after a full day are also accepted (e.g. `…18143025`).
    /// - Hour, minute, and second are **1 or 2** digits when the field ends at `:` /
    ///   space (or, for seconds, `.` before a fraction).
    /// - Compact digit runs without separators are supported (e.g. `T143025`).
    /// - Fractional seconds: `.` then digits (up to 18 kept as attoseconds; extra digits
    ///   ignored).
    /// - Optional trailing `Z`/`z` is consumed.
    /// - Minutes must be `≤ 59`; seconds must be `≤ 60` (leap second `60` allowed; resolved
    ///   by [`Parts::to_dt`](../civil_parts/struct.Parts.html#method.to_dt)).
    ///
    /// #### Optional trailing components
    ///
    /// - **Offset** — `+`/`-` then hours (and optional minutes), with or without `:`:
    ///   `+02:00`, `-0530`, also allowed directly after the date.
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
    /// - Fractional seconds optional.
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
    /// - Fractional days optional.
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
    /// - Fractional days optional.
    ///
    /// ## See also
    ///
    /// - [`Parts::from_str`](../civil_parts/struct.Parts.html#method.from_str) —
    ///   same string parse without resolving to a [`Dt`].
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn from_str(s: &str) -> Result<Self, DtErr> {
        Parts::from_str(s)?.to_dt()
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
    /// - Oversized integer parts saturate at [`i128::MAX`] instead of failing.
    /// - Inputs longer than [`STRTIME_SIZE`](../consts/constant.STRTIME_SIZE.html) are rejected.
    /// - Returns `None` only for completely unparseable input (empty, sign/dot
    ///   only, no digits after skipping, etc.).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let d = Dt::from_str_sec_f("1700000000.123456789012345678", Some(Scale::TAI)).unwrap();
    /// assert_eq!(d.to_sec64_floor(), 1700000000);
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
    #[inline]
    pub fn from_str_sec_f(s: &str, scale: Option<Scale>) -> Option<Dt> {
        Parts::from_str_sec_f(s, scale).and_then(|p| p.to_dt().ok())
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
    #[inline]
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
    #[inline]
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
    /// - [`DtErrKind::OutOfRange`] — The accumulated duration does not fit in attoseconds
    ///   when converting from nanoseconds (`checked_mul`).
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

        Self::parse_duration_part(date_part, &mut total_nanos, true, sign, &mut has_fraction)?;
        Self::parse_duration_part(time_part, &mut total_nanos, false, sign, &mut has_fraction)?;

        let total_attos = total_nanos
            .checked_mul(ATTOS_PER_NS_I128)
            .ok_or_else(|| an_err!(DtErrKind::OutOfRange))?;
        Ok(dt!(total_attos))
    }

    /// Parses a single component (number + optional fraction + unit) from the slice,
    /// advancing the index `i`. Returns [`Option::None`] when the slice is exhausted.
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
                    let total_sec = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_YEAR)
                        .ok_or_else(|| an_err!(DtErrKind::YearOutOfRange))?;
                    total_sec * 1_000_000_000i128
                }
                (true, b'M' | b'm') => {
                    let total_sec = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_MONTH)
                        .ok_or_else(|| an_err!(DtErrKind::MonthOutOfRange))?;
                    total_sec * 1_000_000_000i128
                }
                (true, b'W' | b'w') => {
                    let total_sec = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_WEEK as i128)
                        .ok_or_else(|| an_err!(DtErrKind::WeekOutOfRange))?;
                    total_sec * 1_000_000_000i128
                }
                (true, b'D' | b'd') => {
                    let total_sec = (comp.signed_int as i128)
                        .checked_mul(SEC_PER_DAY)
                        .ok_or_else(|| an_err!(DtErrKind::DayOutOfRange))?;
                    total_sec * 1_000_000_000i128
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
    /// - [`Dt::to_str_b_media_duration`](../struct.Dt.html#method.to_str_b_media_duration)
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
        let total_sec: i128 = match count {
            2 => components[0] * 60 + components[1], // M:SS
            3 => components[0] * 3600 + components[1] * 60 + components[2], // H:MM:SS
            4 => components[0] * 86400 + components[1] * 3600 + components[2] * 60 + components[3], // D:H:MM:SS
            _ => unreachable!(),
        };

        let total_sec = if negative { -total_sec } else { total_sec };

        Ok(dt!(sec!(total_sec)))
    }

    /// Hours:Minutes elapsed time: `H:MM` or `H:MM:SS[.frac]` (hours unbounded).
    ///
    /// Minutes and seconds must be `0..=59`. Fractional seconds (optional) may
    /// follow only the seconds field, up to 18 digits. Requires at least one
    /// `:`. Optional leading `-` and ASCII whitespace. Returns a pure duration
    /// on `Scale::TAI`.
    ///
    /// Not media duration: two fields are **hours:minutes**, not minutes:seconds.
    #[inline]
    pub fn from_str_h_mm_duration(input: &str) -> Result<Dt, DtErr> {
        Ok(dt!(Self::h_mm_duration_attos(input)?))
    }

    /// Shared by [`from_str_h_mm_duration`] and relative natural parse.
    pub(crate) fn h_mm_duration_attos(input: &str) -> Result<i128, DtErr> {
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut pos = 0usize;

        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos == len {
            return Err(an_err!(DtErrKind::Empty));
        }

        let neg = if bytes[pos] == b'-' {
            pos += 1;
            if pos == len {
                return Err(an_err!(DtErrKind::InvalidInput));
            }
            true
        } else {
            false
        };

        let hours = parse_uint(bytes, &mut pos, len)?;
        // At least one colon is required (H:MM or H:MM:SS).
        if pos >= len || bytes[pos] != b':' {
            return Err(an_err!(DtErrKind::InvalidSyntax));
        }
        pos += 1;

        let minutes = parse_uint(bytes, &mut pos, len)?;
        if minutes > 59 {
            return Err(an_err!(DtErrKind::InvalidInput));
        }

        let mut seconds: i128 = 0;
        let mut frac_attos: i128 = 0;

        if pos < len && bytes[pos] == b':' {
            pos += 1;
            seconds = parse_uint(bytes, &mut pos, len)?;
            if seconds > 59 {
                return Err(an_err!(DtErrKind::InvalidInput));
            }
            if pos < len && bytes[pos] == b'.' {
                pos += 1;
                if pos >= len || !bytes[pos].is_ascii_digit() {
                    return Err(an_err!(DtErrKind::ExpectedValue));
                }
                let mut digits = 0u32;
                let mut frac = 0i128;
                while pos < len && bytes[pos].is_ascii_digit() {
                    if digits < 18 {
                        frac = frac
                            .saturating_mul(10)
                            .saturating_add((bytes[pos] - b'0') as i128);
                        digits += 1;
                    }
                    pos += 1;
                }
                frac_attos = frac.saturating_mul(10i128.pow(18 - digits));
            }
        } else if pos < len && bytes[pos] == b'.' {
            // No fractional minutes.
            return Err(an_err!(DtErrKind::InvalidSyntax));
        }

        while pos < len && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos != len {
            return Err(an_err!(DtErrKind::TrailingCharacters));
        }

        let mut attos = hours
            .saturating_mul(3600)
            .saturating_add(minutes.saturating_mul(60))
            .saturating_add(seconds)
            .saturating_mul(ATTOS_PER_SEC_I128)
            .saturating_add(frac_attos);
        if neg {
            attos = -attos;
        }
        Ok(attos)
    }
}

fn parse_uint(bytes: &[u8], pos: &mut usize, len: usize) -> Result<i128, DtErr> {
    if *pos >= len || !bytes[*pos].is_ascii_digit() {
        return Err(an_err!(DtErrKind::ExpectedValue));
    }
    let mut v: i128 = 0;
    while *pos < len && bytes[*pos].is_ascii_digit() {
        v = v
            .saturating_mul(10)
            .saturating_add((bytes[*pos] - b'0') as i128);
        *pos += 1;
    }
    Ok(v)
}
