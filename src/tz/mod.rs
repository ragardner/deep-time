//! Time zone name helpers.
//!
//! This module exposes [`UTC_ALIASES`] and [`tz_names`], used internally by the
//! parser and formatter to recognize IANA time zone identifiers in input strings
//! and to enumerate available zones at runtime.
//!
//! ## Behavior by feature
//!
//! | Configuration | [`tz_names`] yields |
//! |---------------|---------------------|
//! | `jiff-tz` or `jiff-tz-bundle` (with `alloc`) | All IANA identifiers from the bundled Jiff TZ database |
//! | `alloc` without `jiff-tz` | [`UTC_ALIASES`] only |
//! | `no_alloc` | [`UTC_ALIASES`] only (no heap) |
//!
//! Time zone–aware formatting and calendar math ([`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz),
//! [`Dt::add_hours_tz`](../struct.Dt.html#method.add_hours_tz), etc.) require the `jiff-tz` feature.
//! [`tz_names`] is independent of those APIs but uses the same database when `jiff-tz` is enabled.
//!
//! ## Examples
//!
//! Iterate over IANA time zone names when `jiff-tz` is enabled:
//!
//! ```rust
//! # #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
//! # {
//! use deep_time::tz::tz_names;
//!
//! let mut found_london = false;
//! for name in tz_names() {
//!     if name.as_str() == "Europe/London" {
//!         found_london = true;
//!         break;
//!     }
//! }
//! assert!(found_london);
//! # }
//! ```
//!
//! Without `jiff-tz`, only UTC aliases are returned:
//!
//! ```rust
//! # #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
//! # {
//! use deep_time::tz::{tz_names, UTC_ALIASES};
//!
//! let count = tz_names().count();
//! assert_eq!(count, UTC_ALIASES.len());
//! assert!(tz_names().any(|n| n.as_str() == "UTC"));
//! # }
//! ```

use crate::BufStr;

/// Well-known aliases for UTC accepted in parsed date/time strings.
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

/// Returns an iterator over known time zone names as [`BufStr<49>`](../struct.BufStr.html).
///
/// With `jiff-tz` or `jiff-tz-bundle`, yields every IANA identifier from the Jiff
/// time zone database. Otherwise yields only [`UTC_ALIASES`].
pub fn tz_names() -> impl Iterator<Item = BufStr<49>> {
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
fn tz_names_alloc() -> impl Iterator<Item = BufStr<49>> {
    #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
    {
        use alloc::string::ToString;

        jiff::tz::db()
            .available()
            .map(|s| BufStr::new(&s.to_string()))
    }
    #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
    {
        UTC_ALIASES.iter().copied().map(BufStr::new)
    }
}

// no-alloc version (only UTC aliases)
#[cfg(not(feature = "alloc"))]
fn tz_names_no_alloc() -> impl Iterator<Item = BufStr<49>> {
    UTC_ALIASES.iter().copied().map(BufStr::new)
}
