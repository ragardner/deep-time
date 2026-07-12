#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(
    feature = "parse",
    feature = "std",
    feature = "mars",
    feature = "jiff-tz-bundle"
))]
mod tests {
    use deep_time::{
        AnErr, Dt, DtErr, DtErrKind, Lang, ParseCfg, Scale, TimeTraits, YmdHms, an_err, from_ymd,
    };
    // use deep_time::Sidereal;  // needs "sidereal" feature

    #[test]
    fn print_stuff() {}
}
