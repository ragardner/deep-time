#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    use core::fmt::Write;

    use deep_time::{BufStr, Dt, Scale};

    #[cfg(all(
        feature = "parse",
        feature = "std",
        feature = "mars",
        feature = "jiff-tz-bundle"
    ))]
    #[test]
    fn print_stuff() {
        use deep_time::{AnErr, DtErr, DtErrKind, Lang, ParseCfg, TraitsTime, YmdHms, an_err};
    }
}
