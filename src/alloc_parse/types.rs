use crate::{Dt, Lang};
use alloc::string::String;
use alloc::vec::Vec;

/// Used by [`ParseCfg`] in
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse). Controls
/// how ambiguous dates (e.g. `01/02/03`) are parsed.
///
/// The default `Smart` variant applies a practical heuristic that prefers
/// year-first for compact formats and uses numeric plausibility checks
/// for other cases. The other variants force a specific ordering.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum Order {
    /// Heuristic for **mixed data**. Uses the following rules, in this order:
    ///
    /// 1. **Pure-numeric compact formats** (≥ 6 digits with no separators,
    ///    e.g. `240314153045`, `20240315`, `YYMMDDHHMMSS`):
    ///    treated as **Year-first** (`%Y%m%d` / `%y%m%d`).
    ///    These are overwhelmingly used in logs, filenames, databases, APIs,
    ///    configs, and JSON for sortability.
    ///
    /// 2. **Delimited formats that start with a plausible 4-digit year**
    ///    (1900–2100): treated as **Year-first**.
    ///
    /// 3. **Numeric plausibility check** (strongest universal signal):
    ///    - First number is 13–31 → **Day-first** (international/European style).
    ///    - First number is 1–12 **and** second number is 13–31 → **Month-first**
    ///      (US style).
    ///
    /// 4. **Strong ISO 8601 / timestamp markers** (`T` connector, `Z`, numeric
    ///    offsets, or IANA timezone names) → **Year-first**.
    ///
    /// 5. **Fallback**:
    ///    - With the `locale` feature enabled: respects the system locale
    ///      preference (Day-first in most of the world).
    ///    - Without the `locale` feature: **Day-first** (global majority).
    ///
    /// The `/` separator is deliberately ignored in the plausibility step
    /// because it is culturally ambiguous.
    ///
    /// Once the preferred ordering is determined, the parser tries that order
    /// first, then the other two (e.g. Day → Month → Year), before any
    /// year-first unambiguous fallback.
    #[default]
    Smart,
    /// Prefer **Year-first** (YYYY/MM/DD or YY/MM/DD), then Day, then Month.
    Year,
    /// Prefer **Day-first** (DD/MM/YYYY), then Month, then Year.
    Day,
    /// Prefer **Month-first** (MM/DD/YYYY), then Day, then Year.
    Month,
}

/// Used by [`ParseCfg`] in
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse). Controls
/// whether layouts are guessed or taken from caller-supplied `strptime`
/// formats.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ParseFmt {
    /// Guess layouts from the input (default).
    #[default]
    Guess,
    /// Explicit list of formats to try **in the exact order given**, then
    /// continue with the rest of the parser (`order`, unmarked numbers, and
    /// the usual guess).
    ///
    /// Partial date formats are allowed: missing month/day default to `1`
    /// (so `"%Y"` on `"2024"` yields `2024-01-01`, and `"%Y-%m"` on
    /// `"2024-03"` yields `2024-03-01`).
    ///
    /// Example:
    /// ```js
    /// fmt: Prefer(["%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%d.%m.%Y", "%Y"])
    /// ```
    Prefer(Vec<String>),
    /// Explicit list of formats to try **in the exact order given**. Only
    /// these formats are tried; `order` and unmarked-number guessing are
    /// not used. If none match, the parse fails. An empty list fails.
    ///
    /// The format must account for the whole input: leftover text after a
    /// match (e.g. `"%Y-%m-%d"` on `"2024-03-15 12:00"`) is an error.
    /// Partial date formats are still allowed: missing month/day default to `1`
    /// (so `"%Y"` on `"2024"` yields `2024-01-01`, and `"%Y-%m"` on
    /// `"2024-03"` yields `2024-03-01`).
    ///
    /// Example:
    /// ```js
    /// fmt: Only(["%d/%m/%Y", "%Y-%m-%d", "%Y"])
    /// ```
    Only(Vec<String>),
}

/// Used by [`ParseCfg`] in
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse). The type of
/// a pure-numeric input.
///
/// `None` on [`ParseCfg::numeric`] (the default) uses the unguided guess
/// described on [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse):
/// civil compact/ordinal first where the digit length matches, then MJD/JD
/// or unix when that is the remaining reading.
///
/// `Some` applies only when the input is digits (optionally with a sign and
/// decimal point). Named and delimited dates still go through the general
/// parser.
///
/// [`Numeric::Unix`] accepts any length: the pin is the unit, so `"3"` with
/// [`UnixUnit::Millis`] is 3 milliseconds. Other variants require a value
/// that is actually that quantity (MJD/JD in range, a valid compact date,
/// and so on).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Numeric {
    /// Unix timestamp with this unit. Every pure-numeric string is this unit,
    /// including short values such as `"2024"` or `"3"`. The unit is not
    /// inferred from digit length.
    Unix(UnixUnit),
    /// Modified Julian Date (UTC). 5-digit values in `MJD_RANGE` (~1968–2130),
    /// including fractional days (e.g. `60400.75`).
    Mjd,
    /// Julian Day (UTC). 7-digit values in `JD_RANGE` (~5000 BC to ~10,700 AD),
    /// including fractional days (e.g. `2440587.5`).
    ///
    /// An integer JD is noon (the astronomical convention); a fractional JD is
    /// used as written.
    Jd,
    /// Ordinal date only: `YYDDD` / `YYYYDDD` inside `LEGACY_ORDINAL_YEAR_RANGE`
    /// (1850–2300). No MJD/JD fallback.
    Ordinal,
    /// Calendar year.
    Year {
        /// When true, 2-digit years use the 1969–2068 window (`24` → 2024,
        /// `69` → 1969). When false, the digits are the year (`24` → year 24).
        two_digit_pivot: bool,
    },
    /// Compact civil date: `YYMMDD`, `YYYYMM`, or `YYYYMMDD`. Invalid dates
    /// (e.g. month 13) fail; they are not reread as unix.
    CompactYmd,
    /// Spreadsheet date-time serial (LibreOffice/Calc, OOXML, etc.).
    /// Not used in the unguided guess (5-digit unmarked numbers stay ordinal
    /// then MJD).
    ///
    /// Integer part is the calendar day; the fraction is time of day
    /// (`45321.5` is noon). See [`DateBase`].
    DateSerial(DateBase),
}

/// Date base (day 0) for [`Numeric::DateSerial`].
///
/// LibreOffice Calc's default is [`DateBase::Epoch1899`]. OOXML's 1900 date
/// base is [`DateBase::Epoch1900`] (includes the fictitious 1900-02-29).
/// Apple / OOXML `date1904` is [`DateBase::Epoch1904`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateBase {
    /// Serial 0 = 1899-12-30. LibreOffice Calc / OpenOffice default.
    /// Gregorian (no fictitious leap day). Negatives are days before that.
    #[default]
    Epoch1899,
    /// Serial 1 = 1900-01-01, last serial 2_958_465 = 9999-12-31.
    ///
    /// 1900 is treated as a leap year (Lotus 1-2-3 / OOXML 1900 date base),
    /// so serial 60 is the fictitious 1900-02-29 and is **rejected**.
    /// Serials 1–59 are 1900-01-01 through 1900-02-28; serials ≥ 61 match
    /// Gregorian (1899-12-30 + serial). Negatives extend 1900-01-01 backwards
    /// (serial 0 = 1899-12-31).
    Epoch1900,
    /// Serial 0 = 1904-01-01, last serial 2_957_003 = 9999-12-31.
    /// Apple / OOXML 1904 date base. No fictitious leap day.
    /// Epoch1900 serial = Epoch1904 serial + 1462.
    Epoch1904,
}

/// Unit for [`Numeric::Unix`]. There is no inferred unit on a pin: if you
/// know it is unix, you know the unit.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnixUnit {
    /// Seconds since 1970-01-01T00:00:00Z.
    #[default]
    Seconds,
    /// Milliseconds since 1970-01-01T00:00:00Z.
    Millis,
    /// Microseconds since 1970-01-01T00:00:00Z.
    Micros,
    /// Nanoseconds since 1970-01-01T00:00:00Z.
    Nanos,
}

/// Configuration options for
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
///
/// Controls language, ambiguous date order, unmarked numeric type,
/// explicit `strptime` formats, relative-date support, and reference time.
///
/// With the defaults, the parser guesses civil layouts and unmarked numbers.
/// Set [`ParseCfg::fmt`] and/or [`ParseCfg::numeric`] when the caller
/// knows the layout or the numeric type.
///
/// These settings will not persist between parse calls and have to be used
/// as an arg every time you want them.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseCfg {
    /// Caller-supplied `strptime` formats, or guess. See [`ParseFmt`].
    #[cfg_attr(feature = "serde", serde(default))]
    pub fmt: ParseFmt,

    /// Type of unmarked numbers. `None` uses the unguided guess described on
    /// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
    /// `Some` pins that type for pure-numeric input only.
    #[cfg_attr(feature = "serde", serde(default))]
    pub numeric: Option<Numeric>,

    /// Controls ambiguous numeric dates.
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: Order,

    /// Sets language to use for a particular parse call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lang: Lang,

    /// If `true`, `s` is already lowercase and lowercasing is skipped.
    /// If `false` (default), the parser lowercases.
    /// ONLY set to `true` if the &str is already lowercase.
    #[cfg_attr(feature = "serde", serde(default))]
    pub assume_lowercase: bool,

    /// Whether to parse relative dates as well as normal dates.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub relative: bool,

    /// **Reference ("current") time** used for relative expressions:
    /// - "tomorrow", "next Friday", "in 3 days", "next week"
    /// - If `Some`, this `Dt` is used as "now" (overrides everything).
    /// - If `None` + `std` feature enabled: automatically uses real system time.
    /// - If `None` + no `std`: parsing relative dates will fail with a clear error.
    #[cfg_attr(feature = "serde", serde(default))]
    pub ref_time: Option<Dt>,
}

#[cfg(feature = "serde")]
fn default_true() -> bool {
    true
}

impl ParseCfg {
    /// Default configuration (all smart defaults).
    ///
    /// Pass a reference to this when you want the standard parsing behavior:
    ///
    /// ```rust
    /// use deep_time::{Dt, ParseCfg};
    ///
    /// let dt = Dt::from_str_parse("2024-03-15 12:00", &ParseCfg::DEFAULT).unwrap();
    /// ```
    pub const DEFAULT: Self = Self {
        fmt: ParseFmt::Guess,
        numeric: None,
        order: Order::Smart,
        lang: Lang::En,
        assume_lowercase: false,
        relative: true,
        ref_time: None,
    };
}

impl Default for ParseCfg {
    fn default() -> ParseCfg {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderFirst {
    /// Year-Month-Day ordering (ISO 8601 style, `YYYY-MM-DD`, `20240315`, etc.)
    Year,
    /// Month-Day-Year ordering (US / some English locales, `MM/DD/YYYY`)
    Month,
    /// Day-Month-Year ordering (most of the world, `DD/MM/YYYY`, `DD.MM.YYYY`)
    Day,
}

#[derive(Clone)]
pub(crate) struct AmBuilder {
    pub pieces: Vec<&'static str>,
    pub seen_year: bool,
    pub seen_month: bool,
    pub seen_day: bool,
}

#[inline]
pub(crate) fn append_to_all(builders: &mut Vec<AmBuilder>, s: &'static str) {
    for b in builders {
        b.pieces.push(s);
    }
}
