use core::ops::RangeInclusive;

use crate::ParseCfg;
use alloc::boxed::Box;
use once_cell::race::OnceBox;

static DEFAULT_DATE_PARSE_OPTIONS: OnceBox<ParseCfg> = OnceBox::new();
pub(crate) fn default_date_parse_options() -> &'static ParseCfg {
    DEFAULT_DATE_PARSE_OPTIONS.get_or_init(|| Box::new(ParseCfg::default()))
}

#[cfg(feature = "locale")]
use {once_cell::race::OnceBool, sys_locale};

#[cfg(feature = "locale")]
const MONTH_FIRST_LOCALES: &[&str] = &[
    "en-us", // United States (by far the most common)
    "en-ca", // English Canada (very common in practice due to US influence)
    "en-ph", // Philippines (strong US influence)
    "en-bz", // Belize
    "en-jm", // Jamaica
    "en-tt", // Trinidad & Tobago
    "en-bb", // Barbados
    // Spanish-speaking Caribbean / Central America that predominantly use MM/DD
    "es-do", // Dominican Republic
    "es-pa", // Panama
    "es-pr", // Puerto Rico (US territory influence)
];

#[cfg(feature = "locale")]
static LOCALE_PREFERS_DAY_FIRST: OnceBool = OnceBool::new();

#[cfg(feature = "locale")]
pub(crate) fn locale_prefers_day_first() -> bool {
    LOCALE_PREFERS_DAY_FIRST.get_or_init(|| {
        sys_locale::get_locale()
            .map(|locale| {
                let lower = locale.to_ascii_lowercase();

                // If the locale starts with any of the month-first entries
                // → we want MonthFirst, so return `false`
                if MONTH_FIRST_LOCALES.iter().any(|&m| lower.starts_with(m)) {
                    false
                } else {
                    true // DayFirst (default for the rest of the world)
                }
            })
            .unwrap_or(true) // fallback: DayFirst
    })
}

pub(crate) const DIGIT_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub(crate) const MAX_DATE_STRING_LEN: usize = 255;
pub(crate) const MIN_YEAR: i32 = -9999;
pub(crate) const MAX_YEAR: i32 = 9999;

/// Year range considered plausible for legacy/business ordinal dates (YYYYJJJ / YYJJJ).
/// - 7-digit (YYYYJJJ): full 1850–2300 supported via %Y.
/// - 5-digit (YYJJJ): limited to Chrono's %y window (~1969–2068).
///   Pre-1969 legacy ordinals must use 4-digit year or explicit format.
pub(crate) const LEGACY_ORDINAL_YEAR_RANGE: RangeInclusive<i32> = 1850..=2300;
/// Year range considered plausible for YYYYMM pure-numeric input.
/// Used in Auto mode to distinguish modern "202403" (year-month) from the far more common
/// legacy YYMMDD case "240301" (2024-03-01). 1900–2150 covers all realistic use while
/// excluding fake future years like 2403.
pub(crate) const PLAUSIBLE_YYYYMM_YEAR_RANGE: RangeInclusive<i32> = 1900..=2150;

/// Modified Julian Date range for 5-digit pure-numeric input.
/// Covers ~1968–2130.
pub(crate) const MJD_RANGE: RangeInclusive<i64> = 40_000..=85_000;
/// Julian Day (JD) range for 7-digit pure-numeric input.
/// Covers ~5000 BC to ~10,700 AD
pub(crate) const JD_RANGE: RangeInclusive<i64> = 1_400_000..=4_000_000;
