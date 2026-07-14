//! This example shows how precision can be controlled, in this
//! case it's by 1 minute.
//!
//! In this way you can compare two times at a given precision,
//! and also format them to a string without the seconds.
//!
//! ```text
//! cargo run --example precision_control
//! ```

use deep_time::macros::from_sec;
use deep_time::{Dt, DtErr, Lang, Scale, from_ymd};

const ONE_MIN: Dt = from_sec!(60);

fn to_utc_and_floor(dt: Dt) -> Dt {
    dt.to(Scale::UTC).floor(ONE_MIN)
}

fn main() -> Result<(), DtErr> {
    // Noisy inputs (seconds / subseconds from a feed or clock).
    let depart = to_utc_and_floor(from_ymd!(
        2026,
        7,
        9;
        9,
        15,
        37,
        Dt::to_u64(Dt::ms_to_attos(500))
    ));
    let arrive = to_utc_and_floor(from_ymd!(2026, 7, 9; 12, 42, 11, 1));

    // Comparison at minute resolution.
    assert!(depart < arrive);
    assert_eq!(depart, to_utc_and_floor(from_ymd!(2026, 7, 9; 9, 15)));

    // Display: omit seconds with strftime-style formatting (no need for a wrapper type).
    let depart_s = depart.to_str_lite("%Y-%m-%d %H:%M", Lang::En)?;
    let arrive_s = arrive.to_str_lite("%Y-%m-%d %H:%M", Lang::En)?;
    assert_eq!(depart_s.as_str(), "2026-07-09 09:15");
    assert_eq!(arrive_s.as_str(), "2026-07-09 12:42");

    Ok(())
}
