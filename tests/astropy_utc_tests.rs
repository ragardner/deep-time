/*
from astropy.time import Time
import astropy.units as u
import erfa

"""
SCRIPT LAST RUN: 2026-05-31
"""

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

print("\nUTC to TAI:")
lib_zero = Time("2000-01-01 12:00:00", scale="tai", format="iso")
test_cases = [
    ("1963-12-31 23:59:59", "UTC"),
    ("1966-01-01 00:00:00", "UTC"),
    ("1970-01-01 00:00:00", "UTC"),
    ("1971-12-31 23:59:59", "UTC"),
    ("1971-12-31 23:59:60", "UTC"),
    ("1972-01-01 00:00:00", "UTC"),
    ("1972-12-31 23:59:59", "UTC"),
    ("1972-12-31 23:59:60", "UTC"),
    ("1973-01-01 00:00:00", "UTC"),
    ("2000-01-01 12:00:00", "UTC"),
    ("2015-06-30 23:59:59", "UTC"),
    ("2015-06-30 23:59:60", "UTC"),
    ("2015-07-01 00:00:00", "UTC"),
    ("2025-04-16 00:00:00", "UTC"),
    ("2100-01-01 00:00:00", "UTC"),
    ("1900-01-01 00:00:00", "UTC"),
]

for iso, scale in test_cases:
    t = Time(iso, scale=scale.lower(), format="iso")
    delta = (t.tai - lib_zero).to(u.s).value
    print(f"{iso} ({scale}) -> {format(delta, '.20f')}")

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

UTC to TAI:
1961-01-01 00:00:00 (UTC) -> -1230724798.57718205451965332031
1963-12-31 23:59:59 (UTC) -> -1136116798.23420596122741699219
1966-01-01 00:00:00 (UTC) -> -1072958395.68683004379272460938
1970-01-01 00:00:00 (UTC) -> -946727991.99991798400878906250
1971-12-31 23:59:59 (UTC) -> -883655991.10775816440582275391
1971-12-31 23:59:60 (UTC) -> -883655990.10775804519653320312
1972-01-01 00:00:00 (UTC) -> -883655990.00000000000000000000
1972-12-31 23:59:59 (UTC) -> -852033590.00000000000000000000
1972-12-31 23:59:60 (UTC) -> -852033589.00000000000000000000
1973-01-01 00:00:00 (UTC) -> -852033588.00000000000000000000
2000-01-01 12:00:00 (UTC) -> 31.99999999999860733624
2015-06-30 23:59:59 (UTC) -> 488980834.00000000000000000000
2015-06-30 23:59:60 (UTC) -> 488980835.00000000000000000000
2015-07-01 00:00:00 (UTC) -> 488980836.00000000000000000000
2025-04-16 00:00:00 (UTC) -> 798033637.00000000000000000000
2100-01-01 00:00:00 (UTC) -> 3155716837.00000000000000000000
1900-01-01 00:00:00 (UTC) -> -3155716800.00000000000000000000
"""
*/

#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale};

#[test]
fn test_sofa_historical_offsets() {
    let tp = Dt::from_ymd(
        1960,
        12,
        31,
        Scale::UtcHist,
        23,
        59,
        59,
        999_999_999_999_999_999,
    );
    let tp2 = tp.to(Scale::UtcHist).to(Scale::TAI);
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
    let tp = Dt::from_ymd(1972, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert!(
        tp.historical_utc_offset().is_none(),
        "1972-01-01 should return None"
    );

    // 1961-01-01
    let tp = Dt::from_ymd(1961, 1, 1, Scale::TAI, 0, 0, 0, 0);
    let offset = tp.historical_utc_offset().unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-12,
        "1961-01-01 inverse offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01
    let tp = Dt::from_ymd(1966, 5, 1, Scale::TAI, 0, 0, 0, 0);
    let offset = tp.historical_utc_offset().unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-12,
        "1966-05-01 inverse offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31
    let tp = Dt::from_ymd(1971, 12, 31, Scale::TAI, 0, 0, 0, 0);
    let offset = tp.historical_utc_offset().unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-12,
        "1971-12-31 inverse offset was {}, expected 9.889650000000",
        offset
    );
}

#[test]
fn test_utc_unix() {
    /*
    these have been left out because astropy .unix
    spreads out the leap second on the day of its
    addition
    2015-06-30 23:59:59 → 1435708798.000023 s
    2015-06-30 23:59:60 → 1435708799.000012 s
    */
    // 1970-01-01 00:00:00 → 0.000000 s
    let tp = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 0);

    // 1971-01-01 00:00:00 → 31536000.000000 s
    let tp = Dt::from_ymd(1971, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 31536000);

    // 1972-01-01 00:00:00 → 63072000.000000 s
    let tp = Dt::from_ymd(1972, 1, 1, Scale::UTC, 0, 0, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 63072000);

    // 2000-01-01 12:00:00 → 946728000.000000 s
    let tp = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 946728000);

    // 2012-08-08 15:30:00 → 1344439800.000000 s
    let tp = Dt::from_ymd(2012, 8, 8, Scale::UTC, 15, 30, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 1344439800);

    // 2015-07-01 00:00:00 → 1435708800.000000 s
    let tp = Dt::from_ymd(2015, 7, 1, Scale::UTC, 0, 0, 0, 0);
    assert_eq!(tp.to_unix().to_sec64(), 1435708800);
}

#[test]
fn test_from_ymd() {
    use deep_time::{Dt, Scale};

    // 1961-01-01 00:00:00 (UTC) -> -1230724798.57718205451965332031
    let t = Dt::from_ymd(1961, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -1230724798.57718205451965332031);

    // 1963-12-31 23:59:59 (UTC) -> -1136116798.23420596122741699219
    let t = Dt::from_ymd(1963, 12, 31, Scale::UtcHist, 23, 59, 59, 0);
    assert_eq!(t.to_sec_f(), -1136116798.23420596122741699219);

    // 1966-01-01 00:00:00 (UTC) -> -1072958395.68683004379272460938
    let t = Dt::from_ymd(1966, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -1072958395.68683004379272460938);

    // 1970-01-01 00:00:00 (UTC) -> -946727991.99991798400878906250
    let t = Dt::from_ymd(1970, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -946727991.99991798400878906250);

    // 1971-12-31 23:59:59 (UTC) -> -883655991.10775816440582275391
    let t = Dt::from_ymd(1971, 12, 31, Scale::UtcHist, 23, 59, 59, 0);
    assert!((t.to_sec_f() - -883655991.10775816440582275391).abs() < 1e-6);

    // 1971-12-31 23:59:60 (UTC)
    let t = Dt::from_ymd(1971, 12, 31, Scale::UtcHist, 23, 59, 60, 0);
    assert_eq!(t.to_sec_f(), -883655990.10775804519653320312);

    // 1972-01-01 00:00:00 (UTC)
    let t = Dt::from_ymd(1972, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -883655990.00000000000000000000);

    // 1972-12-31 23:59:59 (UTC)
    let t = Dt::from_ymd(1972, 12, 31, Scale::UtcHist, 23, 59, 59, 0);
    assert_eq!(t.to_sec_f(), -852033590.00000000000000000000);

    // 1972-12-31 23:59:60 (UTC)
    let t = Dt::from_ymd(1972, 12, 31, Scale::UtcHist, 23, 59, 60, 0);
    assert_eq!(t.to_sec_f(), -852033589.00000000000000000000);

    // 1973-01-01 00:00:00 (UTC)
    let t = Dt::from_ymd(1973, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -852033588.00000000000000000000);

    // 2000-01-01 12:00:00 (UTC) — use exact 32 as requested
    let t = Dt::from_ymd(2000, 1, 1, Scale::UtcHist, 12, 0, 0, 0);
    assert_eq!(t.to_sec_f(), 32.0);

    // 2015-06-30 23:59:59 (UTC)
    let t = Dt::from_ymd(2015, 6, 30, Scale::UtcHist, 23, 59, 59, 0);
    assert_eq!(t.to_sec_f(), 488980834.00000000000000000000);

    // 2015-06-30 23:59:60 (UTC)
    let t = Dt::from_ymd(2015, 6, 30, Scale::UtcHist, 23, 59, 60, 0);
    assert_eq!(t.to_sec_f(), 488980835.00000000000000000000);

    // 2015-07-01 00:00:00 (UTC)
    let t = Dt::from_ymd(2015, 7, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), 488980836.00000000000000000000);

    // 2025-04-16 00:00:00 (UTC)
    let t = Dt::from_ymd(2025, 4, 16, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), 798033637.00000000000000000000);

    // 2100-01-01 00:00:00 (UTC)
    let t = Dt::from_ymd(2100, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), 3155716837.00000000000000000000);

    // 1900-01-01 00:00:00 (UTC)
    let t = Dt::from_ymd(1900, 1, 1, Scale::UtcHist, 0, 0, 0, 0);
    assert_eq!(t.to_sec_f(), -3155716800.00000000000000000000);
}
