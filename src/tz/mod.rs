#[cfg(feature = "jiff-tz")]
mod jiff_tz;

#[cfg(feature = "jiff-tz")]
use jiff_tz::*;

use crate::LiteStr;

pub static UTC_ALIASES: &[&str] = &[
    "Etc/UCT",
    "Etc/UTC",
    "Etc/Universal",
    "Etc/Zulu",
    "UCT",
    "UTC",
    "Universal",
    "Zulu",
];

// Main function: always available, returns LiteStr<49>
pub fn tz_names() -> impl Iterator<Item = LiteStr<49>> {
    #[cfg(feature = "alloc")]
    {
        tz_names_alloc()
    }
    #[cfg(not(feature = "alloc"))]
    {
        tz_names_no_alloc()
    }
}

// alloc version (uses Jiff when available)
#[cfg(feature = "alloc")]
fn tz_names_alloc() -> impl Iterator<Item = LiteStr<49>> {
    #[cfg(feature = "jiff-tz")]
    {
        jiff::tz::db()
            .available()
            .map(|s| LiteStr::new(&s.to_string()))
    }
    #[cfg(not(feature = "jiff-tz"))]
    {
        UTC_ALIASES.iter().copied().map(LiteStr::new)
    }
}

// no-alloc version (only UTC aliases)
#[cfg(not(feature = "alloc"))]
fn tz_names_no_alloc() -> impl Iterator<Item = LiteStr<49>> {
    UTC_ALIASES.iter().copied().map(LiteStr::new)
}

#[derive(Debug, Clone, Copy)]
pub struct OffsetInfo {
    /// The offset from UTC in seconds (positive = east of UTC).
    pub offset: i32,
    /// Time zone abbreviation in effect (e.g. "EDT", "GMT").
    pub abbrev: LiteStr<49>,
    /// Whether the requested local time falls in a gap (spring-forward).
    pub is_gap: bool,
    /// Size of the gap in seconds (only meaningful when `is_gap == true`).
    pub gap_size: i64,
}

/// Returns offset information for an IANA timezone at the given **local** Unix time.
///
/// If the local time falls in a gap (spring-forward), `is_gap` is `true` and
/// `gap_size` contains the number of skipped seconds. Add `gap_size` to the
/// original local time and re-query to obtain a valid instant.
#[inline(always)]
pub fn offset_for_local(name: &str, local_unix: i64) -> Option<OffsetInfo> {
    #[cfg(feature = "jiff-tz")]
    {
        jiff_offset_info_at_local(name, local_unix)
    }
    #[cfg(not(feature = "jiff-tz"))]
    {
        // Only accept UTC aliases when jiff-tz is disabled
        if UTC_ALIASES
            .iter()
            .any(|&alias| alias.eq_ignore_ascii_case(name))
        {
            Some(OffsetInfo {
                offset: 0,
                abbrev: LiteStr::new("UTC"),
                is_gap: false,
                gap_size: 0,
            })
        } else {
            None
        }
    }
}

/// Returns offset information for an IANA timezone at the given **UTC** Unix time.
///
/// `is_gap` is always `false` because gaps are a local-time concept only.
/// Every UTC instant has exactly one well-defined offset.
#[inline(always)]
pub fn offset_for_utc(name: &str, utc_unix: i64) -> Option<OffsetInfo> {
    #[cfg(feature = "jiff-tz")]
    {
        jiff_offset_info_at_utc(name, utc_unix)
    }
    #[cfg(not(feature = "jiff-tz"))]
    {
        if UTC_ALIASES
            .iter()
            .any(|&alias| alias.eq_ignore_ascii_case(name))
        {
            Some(OffsetInfo {
                offset: 0,
                abbrev: LiteStr::new("UTC"),
                is_gap: false,
                gap_size: 0,
            })
        } else {
            None
        }
    }
}
