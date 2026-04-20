use crate::ParseCfg;
use std::sync::LazyLock;

#[cfg(feature = "locale")]
use {std::sync::OnceLock, sys_locale};

#[cfg(feature = "locale")]
static LOCALE_PREFERS_DAY_FIRST: OnceLock<bool> = OnceLock::new();

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
pub(crate) fn locale_prefers_day_first() -> bool {
    *LOCALE_PREFERS_DAY_FIRST.get_or_init(|| {
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

pub(crate) static DEFAULT_DATE_PARSE_OPTIONS: LazyLock<ParseCfg> = LazyLock::new(ParseCfg::default);

pub(crate) const DIGIT_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Fixed-length second equivalents for ISO 8601 calendar units (Y, M, W, D).
///
/// These constants deliberately use the **Julian year** convention (exactly
/// 365.25 days per year) rather than the slightly more precise Gregorian
/// average (365.2425 days). This is the traditional astronomical standard
/// used by Julian Day (JD) and Modified Julian Date (MJD) systems, and it
/// matches the `NS_PER_YEAR` / `NS_PER_MONTH` constants already defined
/// elsewhere in the crate.
///
/// They exist so that years/months/weeks/days can be converted to a
/// **fixed number of seconds**.
/// The resulting `Span` then contains only fixed time units (hours,
/// minutes, seconds, nanoseconds) and no longer requires a reference
/// date for `.total()` conversions.
pub(crate) const SECONDS_PER_YEAR: i128 = 31_557_600; // 365.25 days × 86_400
pub(crate) const SECONDS_PER_MONTH: i128 = 2_629_800; // 30.4375 days × 86_400
pub(crate) const SECONDS_PER_WEEK: i128 = 604_800;
pub(crate) const SECONDS_PER_DAY: i128 = 86_400;
pub(crate) const NS_PER_YEAR: i128 = 31_557_600_000_000_000; // 365.25 days
pub(crate) const NS_PER_MONTH: i128 = 2_629_800_000_000_000; // 30.4375 days
pub(crate) const NS_PER_WEEK: i128 = 604_800_000_000_000;
pub(crate) const NS_PER_DAY: i128 = 86_400_000_000_000;
pub(crate) const NS_PER_HALF_DAY: i128 = 43_200_000_000_000;
pub(crate) const NS_PER_HOUR: i128 = 3_600_000_000_000;
pub(crate) const NS_PER_MINUTE: i128 = 60_000_000_000;
pub(crate) const NS_PER_SECOND: i128 = 1_000_000_000;

pub(crate) const MAX_DATE_STRING_LEN: usize = 255;
pub(crate) const MIN_YEAR: i32 = -9999;
pub(crate) const MAX_YEAR: i32 = 9999;

/// Year range considered plausible for legacy/business ordinal dates (YYYYJJJ / YYJJJ).
/// - 7-digit (YYYYJJJ): full 1850–2300 supported via %Y.
/// - 5-digit (YYJJJ): limited to Chrono's %y window (~1969–2068).
///   Pre-1969 legacy ordinals must use 4-digit year or explicit format.
pub(crate) const LEGACY_ORDINAL_YEAR_RANGE: std::ops::RangeInclusive<i32> = 1850..=2300;
/// Year range considered plausible for YYYYMM pure-numeric input.
/// Used in Auto mode to distinguish modern "202403" (year-month) from the far more common
/// legacy YYMMDD case "240301" (2024-03-01). 1900–2150 covers all realistic use while
/// excluding fake future years like 2403.
pub(crate) const PLAUSIBLE_YYYYMM_YEAR_RANGE: std::ops::RangeInclusive<i32> = 1900..=2150;

/// Modified Julian Date range for 5-digit pure-numeric input.
/// Covers ~1968–2130.
pub(crate) const MJD_RANGE: std::ops::RangeInclusive<i64> = 40_000..=85_000;
/// Julian Day (JD) range for 7-digit pure-numeric input.
/// Covers ~5000 BC to ~10,700 AD
pub(crate) const JD_RANGE: std::ops::RangeInclusive<i64> = 1_400_000..=4_000_000;

/// MJD 40587.0 exactly = 1970-01-01 00:00:00 UTC
pub(crate) const MJD_EPOCH_NANOS: i128 = 40_587_i128 * NS_PER_DAY;
/// JD 2440587.5 exactly = 1970-01-01 00:00:00 UTC
pub(crate) const JD_EPOCH_NANOS: i128 = 2_440_587_i128 * NS_PER_DAY + NS_PER_HALF_DAY;
