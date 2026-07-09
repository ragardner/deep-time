//! This example shows how precision can be controlled, in this
//! case it's by 1 minute.
//!
//! In this way you can compare two times at a given precision,
//! and also format them to a string without the seconds.
//!
//! ```text
//! cargo run --example precision_control
//! ```

use deep_time::{Dt, DtErr, Lang, Scale};

const MINUTE: Dt = Dt::from_sec(60, Scale::TAI);

fn to_utc_and_floor(dt: Dt) -> Dt {
    dt.to(Scale::UTC).floor(MINUTE)
}

fn main() -> Result<(), DtErr> {
    // Noisy inputs (seconds / subseconds from a feed or clock).
    let depart = to_utc_and_floor(Dt::from_ymd(
        2026,
        7,
        9,
        Scale::UTC,
        9,
        15,
        37,
        500_000_000_000,
    ));
    let arrive = to_utc_and_floor(Dt::from_ymd(2026, 7, 9, Scale::UTC, 12, 42, 11, 1));

    // Comparison at minute resolution.
    assert!(depart < arrive);
    assert_eq!(
        depart,
        to_utc_and_floor(Dt::from_ymd(2026, 7, 9, Scale::UTC, 9, 15, 0, 0))
    );

    // Display: omit seconds with strftime-style formatting (no need for a wrapper type).
    let depart_s = depart.to_str_lite("%Y-%m-%d %H:%M", Lang::En)?;
    let arrive_s = arrive.to_str_lite("%Y-%m-%d %H:%M", Lang::En)?;
    assert_eq!(depart_s.as_str(), "2026-07-09 09:15");
    assert_eq!(arrive_s.as_str(), "2026-07-09 12:42");

    Ok(())
}
