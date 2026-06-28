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
    #[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
    {
        jiff::tz::db()
            .available()
            .map(|s| LiteStr::new(&s.to_string()))
    }
    #[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
    {
        UTC_ALIASES.iter().copied().map(LiteStr::new)
    }
}

// no-alloc version (only UTC aliases)
#[cfg(not(feature = "alloc"))]
fn tz_names_no_alloc() -> impl Iterator<Item = LiteStr<49>> {
    UTC_ALIASES.iter().copied().map(LiteStr::new)
}
