#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale, constants::ATTOS_PER_SEC_I128, leap_seconds::leap_sec};

#[cfg(feature = "parse")]
#[test]
fn leap_seconds_various() {
    use deep_time::{Dt, Scale};

    // not a leap second date, don't roll over to next day
    let orig = Dt::from_ymd(2000, 1, 1, 23, 59, 60, 0, Scale::UTC);
    let new = Dt::from_ymd(2000, 1, 2, 0, 0, 0, 0, Scale::UTC);
    assert_ne!(orig, new);
    let orig = Dt::from_str_parse("2000-01-01T23:59:60", &None).unwrap();
    let new = Dt::from_str_parse("2000-01-02T00:00:00", &None).unwrap();
    assert_ne!(orig, new);

    let before = Dt::from_str_parse("2015-06-30T23:59:59", &None).unwrap();
    assert_eq!(before.to_sec(), 488980834, "59 failed");
    assert_eq!(before.to_sec_ufrac(), 0);

    let leap = Dt::from_str_parse("2015-06-30T23:59:60", &None).unwrap();
    assert_eq!(leap.to_sec(), 488980835, "60 failed");
    assert_eq!(leap.to_sec_ufrac(), 0);

    let after = Dt::from_str_parse("2015-07-01T00:00:00", &None).unwrap();
    assert_eq!(after.to_sec(), 488980836, "00 failed");
    assert_eq!(after.to_sec_ufrac(), 0);

    let before = Dt::from_ymd(2015, 6, 30, 23, 59, 59, 0, Scale::UTC);
    assert_eq!(before.to_sec(), 488980834);
    assert_eq!(before.to_sec_ufrac(), 0);

    let leap = Dt::from_ymd(2015, 6, 30, 23, 59, 60, 0, Scale::UTC);
    assert_eq!(leap.to_sec(), 488980835);
    assert_eq!(leap.to_sec_ufrac(), 0);

    let after = Dt::from_ymd(2015, 7, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(after.to_sec(), 488980836);
    assert_eq!(after.to_sec_ufrac(), 0);

    // NOT utc, BUT it's a leap seconds date, don't roll over to next day
    let leap = Dt::from_str_parse("2015-06-30T23:59:60 TT", &None).unwrap();
    let after = Dt::from_str_parse("2015-07-01T00:00:00 TT", &None).unwrap();
    assert_ne!(leap, after);
    let orig = Dt::from_ymd(2015, 6, 30, 23, 59, 60, 0, Scale::TT);
    let new = Dt::from_ymd(2015, 7, 1, 0, 0, 0, 0, Scale::TT);
    assert_ne!(orig, new);

    // ---- pre 2000 -------------------------------------------------

    // not a leap second date, don't roll over to next day
    let orig = Dt::from_ymd(1972, 2, 1, 23, 59, 60, 0, Scale::UTC);
    let new = Dt::from_ymd(1972, 2, 2, 0, 0, 0, 0, Scale::UTC);
    assert_ne!(orig, new);
    let orig = Dt::from_str_parse("1972-02-01T23:59:60", &None).unwrap();
    let new = Dt::from_str_parse("1972-02-02T00:00:00", &None).unwrap();
    assert_ne!(orig, new);

    let before = Dt::from_str_parse("1972-12-31T23:59:59", &None).unwrap();
    assert_eq!(before.to_sec(), -852033590, "59 failed");
    assert_eq!(before.to_sec_ufrac(), 0);

    let leap = Dt::from_str_parse("1972-12-31T23:59:60", &None).unwrap();
    assert_eq!(leap.to_sec(), -852033589, "60 failed");
    assert_eq!(leap.to_sec_ufrac(), 0);

    let after = Dt::from_str_parse("1973-01-01T00:00:00", &None).unwrap();
    assert_eq!(after.to_sec(), -852033588, "00 failed");
    assert_eq!(after.to_sec_ufrac(), 0);

    let before = Dt::from_ymd(1972, 12, 31, 23, 59, 59, 0, Scale::UTC);
    assert_eq!(before.to_sec(), -852033590);
    assert_eq!(before.to_sec_ufrac(), 0);

    let leap = Dt::from_ymd(1972, 12, 31, 23, 59, 60, 0, Scale::UTC);
    assert_eq!(leap.to_sec(), -852033589);
    assert_eq!(leap.to_sec_ufrac(), 0);

    let after = Dt::from_ymd(1973, 1, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(after.to_sec(), -852033588);
    assert_eq!(after.to_sec_ufrac(), 0);

    // NOT utc, BUT it's a leap seconds date, don't roll over to next day
    let leap = Dt::from_str_parse("1972-12-31T23:59:60 TT", &None).unwrap();
    let after = Dt::from_str_parse("1973-01-01T00:00:00 TT", &None).unwrap();
    assert_ne!(leap, after);
    let orig = Dt::from_ymd(1973, 6, 30, 23, 59, 60, 0, Scale::TT);
    let new = Dt::from_ymd(1973, 7, 1, 0, 0, 0, 0, Scale::TT);
    assert_ne!(orig, new);

    // boundary 1972

    let before = Dt::from_str_parse("1971-12-31T23:59:59 UTCSOFA", &None).unwrap();
    assert!(
        (before.to_sec_f() - -883655991.10775816440582275391).abs() < 1e-6,
        "59 failed {}",
        (before.to_sec_f() - -883655991.10775816440582275391).abs()
    );

    let leap = Dt::from_str_parse("1971-12-31T23:59:60 UTCSOFA", &None).unwrap();
    assert_eq!(
        leap.to_sec_f(),
        -883655990.10775804519653320312,
        "60 failed"
    );

    let after = Dt::from_str_parse("1972-01-01T00:00:00 UTCSOFA", &None).unwrap();
    assert_eq!(
        after.to_sec_f(),
        -883655990.00000000000000000000,
        "00 failed"
    );

    let before = Dt::from_ymd(1971, 12, 31, 23, 59, 59, 0, Scale::UTCSofa);
    assert!(
        (before.to_sec_f() - -883655991.10775816440582275391).abs() < 1e-6,
        "ymd 59 failed {}",
        (before.to_sec_f() - -883655991.10775816440582275391).abs()
    );

    let leap = Dt::from_ymd(1971, 12, 31, 23, 59, 60, 0, Scale::UTCSofa);
    assert_eq!(
        leap.to_sec_f(),
        -883655990.10775804519653320312,
        "ymd 60 failed"
    );

    let after = Dt::from_ymd(1972, 1, 1, 0, 0, 0, 0, Scale::UTCSofa);
    assert_eq!(
        after.to_sec_f(),
        -883655990.00000000000000000000,
        "ymd 00 failed"
    );
}

#[test]
fn to_epoch_leaps_and_tai() {
    // Sub-second precision is preserved
    let t2 = Dt::from_ymd(2023, 6, 15, 12, 0, 0, 123_456_789_000_000_000, Scale::UTC);
    let attos2 = t2.to_unix().to_attos();
    assert_eq!(attos2 % ATTOS_PER_SEC_I128, 123_456_789_000_000_000);

    let x = Dt::from_ymd(2016, 12, 31, 23, 59, 59, 0, Scale::UTC);
    assert_eq!(
        x.to_sec(),
        536500835,
        "internal tai sec for 2016-12-31T23:59:59 should be 536500835, got: {}",
        x.to_sec(),
    );
    let leap = Dt::from_ymd(2016, 12, 31, 23, 59, 60, 0, Scale::UTC);
    assert_eq!(
        leap.to_sec(),
        536500836,
        "internal tai sec for 2016-12-31T23:59:60 should be 536500836, got: {}",
        leap.to_sec(),
    );
    assert!(
        leap_sec(leap.to_sec64(), false).unwrap().is_leap_sec,
        "tai 536500836 should be a leap second",
    );
    let y = Dt::from_ymd(2017, 1, 1, 0, 0, 0, 0, Scale::UTC);
    assert_eq!(
        y.to_sec(),
        536500837,
        "internal tai sec for 2017-01-01T00:00:00 should be 536500837, got: {}",
        y.to_sec(),
    );

    // ------------------------------------------------------------
    // 2016-12-31 23:59:60 UTC  →  civil unix timestamp of 2017-01-01 00:00:00
    // ------------------------------------------------------------
    let leap = Dt::from_ymd(2016, 12, 31, 23, 59, 60, 0, Scale::UTC);
    let leap_attos = leap.to_unix().to_attos();

    let unix_sec_part = Dt::attos_to_sec(leap_attos);
    assert_eq!(unix_sec_part, 1_483_228_799);

    let after = Dt::from_ymd(2017, 1, 1, 0, 0, 0, 0, Scale::UTC);
    let after_attos = after.to_unix().to_attos();

    let unix_sec_part = Dt::attos_to_sec(after_attos);
    assert_eq!(unix_sec_part, 1_483_228_800); // 2017-01-01 00:00:00 UTC

    // The fractional part should be zero for this instant
    assert_eq!(after_attos % ATTOS_PER_SEC_I128, 0);
}

#[test]
fn test_leap_second_roundtrip_and_sec() {
    let test_cases = vec![
        // (year, month, day, hour, minute, second_input, expected_sec)
        (2016, 12, 31, 23, 59, 59, 536500799),
        (2016, 12, 31, 23, 59, 60, 536500799),
        (2017, 1, 1, 0, 0, 0, 536500800),
    ];

    for (yr, mo, day, hr, min, sec_input, expected_sec) in test_cases {
        let tp = Dt::from_ymd(yr, mo, day, hr, min, sec_input, 0, Scale::UTC);

        // Verify the internal .sec value matches what was printed
        assert_eq!(
            tp.to(Scale::UTC).to_sec(),
            expected_sec,
            "sec() mismatch for input {yr}-{mo:02}-{day:02} {hr:02}:{min:02}:{sec_input:02}"
        );

        // Round-trip test
        let g = tp.to_ymd();
        let tp_roundtrip = Dt::from_ymd(
            g.yr(),
            g.mo(),
            g.day(),
            g.hr(),
            g.min(),
            g.sec(),
            g.attos(),
            Scale::UTC,
        );

        assert_eq!(
            tp.to_sec(),
            tp_roundtrip.to_sec(),
            "roundtrip failed for input {yr}-{mo:02}-{day:02} {hr:02}:{min:02}:{sec_input:02} \
             (to_gregorian produced sec={})",
            g.sec()
        );
    }
}

#[cfg(feature = "std")]
#[test]
fn test_leap_seconds_file() {
    use deep_time::leap_seconds::LEAP_SECS;

    let leap_seconds_table = Dt::leap_sec_data_from_file("leap-seconds.list.txt").unwrap();
    assert_eq!(leap_seconds_table[1], LEAP_SECS[1]);
    let x = Dt::from_ymd(2015, 6, 30, 23, 59, 60, 0, Scale::UTC);
    let leap_info = Dt::leap_sec_using(&x, false, &leap_seconds_table).unwrap();
    assert!(leap_info.is_leap_sec == true);
}
