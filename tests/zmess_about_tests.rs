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
        let dt = Dt::from_str_iso("1858-11-17 00:00:00").unwrap();
        eprintln!("{:?}", dt);

        let mjd = dt.to_mjd_f_raw();

        eprintln!("{}", mjd);

        assert_eq!(mjd, Dt::MJD_EPOCH.to_mjd_f_raw());
    }
}
