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

        let dt = 100.years().ago().add_mins(-60).add_ms(-500);
        let (whole_sols, frac_attos) = dt.to_msd();

        eprintln!("{}. {} sols", whole_sols, frac_attos);
    }
}
