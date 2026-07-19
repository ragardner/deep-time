#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(
    feature = "parse",
    feature = "std",
    feature = "mars",
    feature = "jiff-tz-bundle"
))]
mod tests {
    use deep_time::{
        AnErr, Dt, DtErr, DtErrKind, Lang, ParseCfg, Scale, TraitsTime, YmdHms, an_err,
    };
    // use deep_time::Sidereal;  // needs "sidereal" feature

    #[test]
    fn print_stuff() {
        // let dt = Dt::from_str("Wed, 16 Apr 2025 14:30:45 GMT").unwrap();
        // eprintln!("{}", dt.to_ymd());
        // let dt = Dt::from_str_parse("Wed, 16 Apr 2025 14:30:45 GMT", &ParseCfg::DEFAULT).unwrap();
        // eprintln!("{}", dt.to_ymd());
    }
}
