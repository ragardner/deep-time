#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "jiff-tz"))]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale, YmdHms};

    #[test]
    fn print_stuff() {}
}
