pub mod tzdb;

pub use tzdb::*;

#[cfg(all(feature = "jiff-tz", not(feature = "tz-tests")))]
mod jiff_tz;

#[cfg(all(feature = "jiff-tz", not(feature = "tz-tests")))]
use jiff_tz::*;

/// Returns offset information for an IANA timezone at the given **local** Unix time.
///
/// If the local time falls in a gap (spring-forward), `is_gap` is `true` and
/// `gap_size` contains the number of skipped seconds. Add `gap_size` to the
/// original local time and re-query to obtain a valid instant.
#[inline(always)]
pub fn offset_for_local(name: &str, local_unix: i64) -> Option<OffsetInfo> {
    #[cfg(all(feature = "jiff-tz", not(feature = "tz-tests")))]
    {
        jiff_offset_info_at_local(name, local_unix)
    }
    #[cfg(any(not(feature = "jiff-tz"), feature = "tz-tests"))]
    {
        offset_info_at_local(name, local_unix)
    }
}

/// Returns offset information for an IANA timezone at the given **UTC** Unix time.
///
/// `is_gap` is always `false` because gaps are a local-time concept only.
/// Every UTC instant has exactly one well-defined offset.
#[inline(always)]
pub fn offset_for_utc(name: &str, utc_unix: i64) -> Option<OffsetInfo> {
    #[cfg(all(feature = "jiff-tz", not(feature = "tz-tests")))]
    {
        jiff_offset_info_at_utc(name, utc_unix)
    }
    #[cfg(any(not(feature = "jiff-tz"), feature = "tz-tests"))]
    {
        offset_info_at_utc(name, utc_unix)
    }
}
