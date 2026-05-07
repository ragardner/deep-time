use crate::{Lang, Dt};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub enum DateOrder {
    /// **Default & recommended** — Smart modern heuristic (best of all worlds):
    /// - Tries **Year-first** formats first (modern/tech bias: JSON, logs, APIs, databases)
    /// - Then Day-first (international/European)
    /// - Then Month-first (US fallback)
    ///
    /// This gives the best real-world success rate while remaining predictable.
    #[default]
    Smart,
    /// Force **Year-first** only (YYYY/MM/DD or YY/MM/DD)
    YearFirst,
    /// Force **Day-first** only (DD/MM/YYYY)
    DayFirst,
    /// Force **Month-first** only (MM/DD/YYYY)
    MonthFirst,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Only relevant for purely numeric dates.
pub enum DateParseMode {
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
    pub mode: DateParseMode,

    /// Controls ambiguous numeric dates.
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: DateOrder,

    /// Sets language to use, not persistent.
    #[cfg_attr(feature = "serde", serde(default))]
    pub lang: Lang,

    /// Whether to lowercase the input:
    /// ONLY set to `false` if the &str is already lowercase.
    #[cfg_attr(feature = "serde", serde(default = "default_true"))]
    pub to_lower: bool,

    /// **Reference ("current") time** used for relative expressions:
    /// - "tomorrow", "next Friday", "in 3 days", "next week"
    /// - dates missing a year/month ("March 15", "15th of next month")
    ///
    /// - If `Some(tp)`, this `Dt` is used as "now" (overrides everything).
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
    fn default() -> Self {
        Self {
            parse: None,
            mode: DateParseMode::default(),
            order: DateOrder::default(),
            lang: Lang::default(),
            to_lower: true,
            ref_time: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectedDateOrder {
    /// Year-Month-Day ordering (ISO 8601 style, `YYYY-MM-DD`, `20240315`, etc.)
    YearFirst,
    /// Month-Day-Year ordering (US / some English locales, `MM/DD/YYYY`)
    MonthFirst,
    /// Day-Month-Year ordering (most of the world, `DD/MM/YYYY`, `DD.MM.YYYY`)
    DayFirst,
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
