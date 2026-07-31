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
        use deep_time::macros::{days_f, dt, from_ymd};
        use deep_time::{
            AnErr, Dt, DtErr, DtErrKind, Lang, ParseCfg, Scale, TraitsTime, YmdHms, sec_f,
        };

        eprintln!("{:#}", Dt::MAX);
        let s = Dt::MIN.to_string();
        eprintln!("{}", s);
        let s = Dt::ZERO.to_string();
        eprintln!("{}", s);
    }
}
