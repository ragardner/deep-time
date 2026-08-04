#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/// Without jiff-tz*, [`tz_names`] is only the UTC alias list
#[cfg(not(any(feature = "jiff-tz-bundle", feature = "jiff-tz")))]
#[test]
fn tz_names_without_jiff_is_utc_aliases_only() {
    use deep_time::tz::{UTC_ALIASES, tz_names};
    use std::collections::HashSet;

    let names: HashSet<String> = tz_names().map(|b| b.as_str().to_string()).collect();
    let aliases: HashSet<String> = UTC_ALIASES.iter().map(|s| (*s).to_string()).collect();

    assert_eq!(names, aliases);
    assert!(names.contains("UTC"));
    assert!(names.contains("Zulu"));
    assert!(!names.contains("America/New_York"));
    assert!(!names.contains("Europe/London"));
}

/// With jiff-tz*, the database includes real IANA names beyond UTC aliases
#[cfg(any(feature = "jiff-tz-bundle", feature = "jiff-tz"))]
#[test]
fn tz_names_with_jiff_includes_real_iana() {
    use deep_time::tz::tz_names;

    let names: Vec<_> = tz_names().map(|b| b.as_str().to_string()).collect();
    assert!(
        names.iter().any(|n| n == "America/New_York"),
        "expected America/New_York in jiff db"
    );
    assert!(
        names.iter().any(|n| n == "UTC" || n == "Etc/UTC"),
        "expected a UTC name in jiff db"
    );
    assert!(
        names.len() > 20,
        "jiff db should list many zones, got {}",
        names.len()
    );
}
