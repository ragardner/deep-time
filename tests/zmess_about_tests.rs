#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    use core::fmt::Write;

    use deep_time::{BufStr, Dt, Scale};

    // ── playground (needs a fat feature set) ────────────────────────────
    #[cfg(all(
        feature = "parse",
        feature = "std",
        feature = "mars",
        feature = "jiff-tz-bundle"
    ))]
    #[test]
    fn print_stuff() {
        use deep_time::{AnErr, DtErr, DtErrKind, Lang, ParseCfg, TraitsTime, YmdHms, an_err};
        // use deep_time::Sidereal;  // needs "sidereal" feature

        // let dt = Dt::from_str("Wed, 16 Apr 2025 14:30:45 GMT").unwrap();
        // eprintln!("{}", dt.to_ymd());
        // let dt = Dt::from_str_parse("Wed, 16 Apr 2025 14:30:45 GMT", &ParseCfg::DEFAULT).unwrap();
        // eprintln!("{}", dt.to_ymd());
    }
}
