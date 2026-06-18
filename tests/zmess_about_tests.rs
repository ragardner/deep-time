#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale, YmdHms};

    #[test]
    fn print_stuff() {
        // let x: Dt = "2025:01:10T00:00:00".parse().unwrap();
        // let x: Dt = "2025:01:10:00:00:00".parse().unwrap();
        // let x: Dt = "2025 01 10:00:00:00".parse().unwrap();
        // let x: Dt = "2025 01:10:00:00:00".parse().unwrap();
    }
}
