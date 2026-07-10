#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(
    feature = "parse",
    feature = "std",
    feature = "mars",
    feature = "jiff-tz-bundle"
))]
mod tests {
    use deep_time::{AnErr, DtErr, DtErrKind, an_err};
    // use deep_time::Sidereal;  // needs "sidereal" feature
    use deep_time::{Dt, Lang, ParseCfg, Scale, TimeTraits, YmdHms};

    #[test]
    fn print_stuff() {
        use deep_time::{Dt, Scale, TimeTraits};
        use deep_time::{from_sec, from_ymd};

        let x = from_sec!(0, on Scale::TAI);
        let y = from_ymd!(1970);
        assert_eq!(y, Dt::UNIX_EPOCH);
    }
}
