use crate::{Dt, Lang};
use alloc::string::String;
use alloc::vec::Vec;

/// Used by [`ParseCfg`] in
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
///
/// Controls how ambiguous numeric dates (e.g. `01/02/03`).
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
    /// Once the preferred ordering is determined, the parser tries the
    /// corresponding ambiguous candidate formats (Year-first → Day-first →
    /// Month-first, or the reverse, depending on the detected order) and falls
    /// back gracefully.
    #[default]
    Smart,
    /// Force **Year-first** only (YYYY/MM/DD or YY/MM/DD)
    Year,
    /// Force **Day-first** only (DD/MM/YYYY)
    Day,
    /// Force **Month-first** only (MM/DD/YYYY)
    Month,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Only relevant for purely numeric dates.
pub enum Mode {
    /// **Default mode** — Smart heuristic:
    /// - 5/7-digit pure-numeric inside `LEGACY_ORDINAL_YEAR_RANGE` → treated as business ordinal (YYYYDDD / YYDDD)
    /// - Outside that range or invalid ordinal → treated as MJD or JD
    #[default]
    Auto,
    /// When combined with a provided Vec of formats in parse no other formats are tried.
    Explicit,
    /// It's some sort of unix timestamp
    UnixTimestamp,
    /// Business/legacy-only mode:
    /// Only accepts ordinal dates (YYYYDDD / YYDDD). No astronomy (JD/MJD) support.
    /// Strict and predictable for ERP/mainframe data.
    Legacy,
    /// Scientific / astronomy-first mode:
    /// Prioritizes MJD (5-digit) and JD (7-digit). Ordinals are only fallback.
    /// Use this when parsing data from astronomy tools or large numeric epochs.
    Scientific,
}

/// Configuration options for
/// [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse).
///
/// Controls language, ambiguous date order, numeric parsing mode,
/// explicit `strptime` formats, relative-date support, and reference time.
///
/// These settings will not persist between parse calls and have to be used
/// as an arg every time you want them.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseCfg {
    /// Explicit list of formats to try **in the exact order given**.
    ///
    /// If this is provided and the vec is non-empty and the mode is Explicit
    /// then only these formats are tried and `mode` and `order` are ignored.
    ///
    /// If the mode is not Explicit then after trying the formats in parse the
    /// rest of the parser will continue as normal, using `mode` and `order`.
    ///
    /// Example:
    /// ```js
    /// parse: ["%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%d.%m.%Y"]
    /// ```
    #[cfg_attr(feature = "serde", serde(default))]
    pub parse: Option<Vec<String>>,

    /// Controls which preset format sets are used (astronomy/scientific formats,
    /// legacy business rules, etc.).
    #[cfg_attr(feature = "serde", serde(default))]
    pub mode: Mode,

    /// Controls ambiguous numeric dates.
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: Order,

    /// Sets language to use for a particular parse call.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lang: Lang,

    /// Whether to lowercase the input:
    /// ONLY set to `false` if the &str is already lowercase.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub to_lower: bool,

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

impl Default for ParseCfg {
    fn default() -> ParseCfg {
        Self {
            parse: None,
            mode: Mode::default(),
            order: Order::default(),
            lang: Lang::default(),
            to_lower: true,
            relative: true,
            ref_time: None,
        }
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
