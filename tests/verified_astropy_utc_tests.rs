/*
from astropy.time import Time
import erfa

print("Historical SOFA offsets (TAI scale):")
for date in ["1961-01-01", "1966-05-01", "1971-12-31"]:
    t = Time(f"{date} 00:00:00", scale="tai")
    y, m, d = int(t.ymdhms["year"]), int(t.ymdhms["month"]), int(t.ymdhms["day"])
    offset = erfa.dat(y, m, d, 0.0)
    print(f"  {date} 00:00:00 → {offset:.12f} s")

print("\nSeconds since Unix epoch (1970-01-01 00:00:00 UTC):")
for date_str in [
    "1970-01-01 00:00:00",
    "1971-01-01 00:00:00",
    "1972-01-01 00:00:00",
    "2000-01-01 12:00:00",
    "2012-08-08 15:30:00",
    #
    "2015-06-30 23:59:59",
    "2015-06-30 23:59:60",
    #
    "2015-07-01 00:00:00",
]:
    t = Time(date_str, scale="utc")
    print(f"  {date_str} → {t.unix:.6f} s")

"""
Historical SOFA offsets (TAI scale):
  1961-01-01 00:00:00 → 1.422818000000 s
  1966-05-01 00:00:00 → 4.624210000000 s
  1971-12-31 00:00:00 → 9.889650000000 s

Seconds since Unix epoch (1970-01-01 00:00:00 UTC):
  1970-01-01 00:00:00 → 0.000000 s
  1971-01-01 00:00:00 → 31536000.000000 s
  1972-01-01 00:00:00 → 63072000.000000 s
  2000-01-01 12:00:00 → 946728000.000000 s
  2012-08-08 15:30:00 → 1344439800.000000 s
  2015-06-30 23:59:59 → 1435708798.000023 s
  2015-06-30 23:59:60 → 1435708799.000012 s
  2015-07-01 00:00:00 → 1435708800.000000 s
"""
*/

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale, historical_sofa::historical_sofa_offset_for_non_adjusted};

#[test]
fn test_sofa_historical_offsets() {
    let tp = Dt::from_ymd(
        1960,
        12,
        31,
        23,
        59,
        59,
        999_999_999_999_999_999,
        Scale::UTCSofa,
    );
    let tp2 = tp.to(Scale::UTCSofa).to(Scale::TAI);
    assert_eq!(
        tp.to_sec(),
        tp2.to_sec(),
        "Round trip just before SOFA start changed integer seconds"
    );
    assert_eq!(
        tp.attos, tp2.attos,
        "Round trip just before SOFA start changed attoseconds"
    );

    // 1972-01-01 (first day of modern leap-second system)
    let tp = Dt::from_ymd(1972, 1, 1, 0, 0, 0, 0, Scale::TAI);
    assert!(
        historical_sofa_offset_for_non_adjusted(&tp).is_none(),
        "1972-01-01 should return None"
    );

    // 1961-01-01
    let tp = Dt::from_ymd(1961, 1, 1, 0, 0, 0, 0, Scale::TAI);
    eprintln!("{}", tp);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-12,
        "1961-01-01 inverse offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01
    let tp = Dt::from_ymd(1966, 5, 1, 0, 0, 0, 0, Scale::TAI);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-12,
        "1966-05-01 inverse offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31
    let tp = Dt::from_ymd(1971, 12, 31, 0, 0, 0, 0, Scale::TAI);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-12,
        "1971-12-31 inverse offset was {}, expected 9.889650000000",
        offset
    );
}

#[test]
fn test_utc_unix() {
    /*
    1970-01-01 00:00:00 → 0.000000 s
    1971-01-01 00:00:00 → 31536000.000000 s
    1972-01-01 00:00:00 → 63072000.000000 s
    2000-01-01 12:00:00 → 946728000.000000 s
    2012-08-08 15:30:00 → 1344439800.000000 s
    2015-06-30 23:59:59 → 1435708798.000023 s
    2015-06-30 23:59:60 → 1435708799.000012 s
    2015-07-01 00:00:00 → 1435708800.000000 s
    */
    // 1970-01-01 00:00:00
    let tp = Dt::from_ymd(1970, 1, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 0);

    // 1971-01-01 00:00:00
    let tp = Dt::from_ymd(1971, 1, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 31536000);

    // 1972-01-01 00:00:00
    let tp = Dt::from_ymd(1972, 1, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 63072000);

    // 2000-01-01 12:00:00
    let tp = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 946728000);

    // 2012-08-08 15:30:00
    let tp = Dt::from_ymd(2012, 8, 8, 15, 30, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 1344439800);

    // 2015-07-01 00:00:00
    let tp = Dt::from_ymd(2015, 7, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(tp.to_unix().to_sec64(), 1435708800);
}
