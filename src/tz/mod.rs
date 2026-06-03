#[cfg(feature = "jiff")]
use jiff::tz;

// Feature-gated iterator
#[cfg(feature = "jiff")]
pub fn available_timezones() -> impl Iterator<Item = &'static str> {
    tz::db().available().map(|name| {
        let s = name.to_string();
        &*Box::leak(s.into_boxed_str())
    })
}

#[cfg(not(feature = "jiff"))]
pub static TZ_NAMES: [&str; 8] = [
    "Etc/UCT",
    "Etc/UTC",
    "Etc/Universal",
    "Etc/Zulu",
    "UCT",
    "UTC",
    "Universal",
    "Zulu",
];

pub fn is_utc_iana(name: &str) -> bool {
    matches!(
        name,
        "Etc/UCT" | "Etc/UTC" | "Etc/Universal" | "Etc/Zulu" | "UCT" | "UTC" | "Universal" | "Zulu"
    )
}

#[cfg(not(feature = "jiff"))]
pub fn available_timezones() -> impl Iterator<Item = &'static str> {
    TZ_NAMES.iter().copied()
}
