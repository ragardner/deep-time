#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale, YmdHms};

    #[test]
    fn print_stuff() {
        // let x = Dt::from_ymd(2020, 1, 1, Scale::TT, 0, 0, 0, 0);
        // let g = x.target(Scale::GPS).to_gps();

        // eprintln!("{}", g);

        // let y = x.to_ymd();
        // let z = y.to_dt();

        // eprintln!("{}, {}", x, z);
    }
}
