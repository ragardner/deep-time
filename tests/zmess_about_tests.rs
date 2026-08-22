#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

mod tests {
    #[cfg(all(
        feature = "parse",
        feature = "std",
        feature = "mars",
        feature = "jiff-tz-bundle"
    ))]
    #[test]
    fn print_stuff() {
        use deep_time::macros::{days_f, dt, from_mjd_f, from_ymd, sec_f};
        use deep_time::{AnErr, Dt, DtErr, DtErrKind, Lang, ParseCfg, Scale, TraitsTime, YmdHms};
    }
}
