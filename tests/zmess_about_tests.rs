#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{AnErr, DtErr, DtErrKind, an_err};
    // use deep_time::Sidereal;  // needs "sidereal" feature
    use deep_time::{Dt, Lang, ParseCfg, Scale, TimeTraits, YmdHms};

    #[test]
    fn print_stuff() {
        let dt = Dt::from_ymd(1965, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
        let x = dt.to(Scale::UtcHist);
    }
}
