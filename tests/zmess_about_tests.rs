#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "lang")]
mod tests {
    use deep_time::{Dt, Lang, Mode, Order, ParseCfg, Scale, TimeParts};

    #[cfg(feature = "jiff-tz")]
    #[test]
    fn print_stuff() {}
}
