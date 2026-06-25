#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{AnErr, DtErr, DtErrKind, an_err};
    // use deep_time::Sidereal;  // needs "sidereal" feature
    use deep_time::{Dt, Lang, ParseCfg, Scale, YmdHms};

    #[test]
    fn print_stuff() {
        eprintln!("Size of DtErr: {} bytes", core::mem::size_of::<DtErr>());
    }
}
