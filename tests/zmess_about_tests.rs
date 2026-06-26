#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{AnErr, DtErr, DtErrKind, an_err};
    // use deep_time::Sidereal;  // needs "sidereal" feature
    use deep_time::{Dt, Lang, ParseCfg, Scale, TimeTraits, YmdHms};

    #[test]
    fn print_stuff() {
        // let one_min = 1.mins();
        // println!("1.mins() attos     = {}", one_min.to_attos());
        // println!("1.mins() seconds   = {}", one_min.to_sec_f());
        // println!("1.mins() + 40s     = {}", (1.mins() + 40.sec()).to_sec_f());
    }
}
