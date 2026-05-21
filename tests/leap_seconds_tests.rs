#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, Scale, constants::ATTOS_PER_SEC_I128, leap_seconds::get_leap_sec};

#[test]
fn to_epoch_leaps_and_tai() {
    // A normal date well after the last leap second
    let t = Dt::from_ymdhms(2023, 6, 15, 12, 0, 0, 0);
    let unix_attos = t
        .to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
        .to_attos();
    assert!(unix_attos > 1_600_000_000_000_000_000);

    // Sub-second precision is preserved
    let t2 = Dt::from_ymdhms(2023, 6, 15, 12, 0, 0, 123_456_789_000_000_000);
    let attos2 = t2
        .to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
        .to_attos();
    assert_eq!(attos2 % ATTOS_PER_SEC_I128, 123_456_789_000_000_000);

    // Roundtrip on GPS scale (non-epoch instant)
    let t_gps = Dt::from_ymdhms(2020, 1, 1, 0, 0, 0, 0);
    let back = Dt::from_diff_and_scale(
        t_gps.to_scale_and_then_diff(Scale::GPS, Dt::GPS_EPOCH),
        Dt::GPS_EPOCH,
        Scale::GPS,
    );
    assert_eq!(t_gps, back);

    let x = Dt::from_ymdhms(2016, 12, 31, 23, 59, 59, 0);
    assert_eq!(
        x.sec, 536500835,
        "internal tai sec for 2016-12-31T23:59:59 should be 536500835, got: {}",
        x.sec,
    );
    let leap = Dt::from_ymdhms(2016, 12, 31, 23, 59, 60, 0);
    assert_eq!(
        leap.sec, 536500836,
        "internal tai sec for 2016-12-31T23:59:60 should be 536500836, got: {}",
        leap.sec,
    );
    assert!(
        get_leap_sec(&leap, false).is_leap_sec,
        "tai 536500836 should be a leap second",
    );
    let y = Dt::from_ymdhms(2017, 1, 1, 0, 0, 0, 0);
    assert_eq!(
        y.sec, 536500837,
        "internal tai sec for 2017-01-01T00:00:00 should be 536500837, got: {}",
        y.sec,
    );

    // ------------------------------------------------------------
    // 2016-12-31 23:59:60 UTC  →  civil unix timestamp of 2017-01-01 00:00:00
    // ------------------------------------------------------------
    let leap = Dt::from_ymdhms(2016, 12, 31, 23, 59, 60, 0);
    let leap_attos = leap
        .to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
        .to_attos();

    let unix_sec_part = leap_attos.div_euclid(ATTOS_PER_SEC_I128);
    assert_eq!(unix_sec_part, 1_483_228_799);

    let after = Dt::from_ymdhms(2017, 1, 1, 0, 0, 0, 0);
    let after_attos = after
        .to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
        .to_attos();

    let unix_sec_part = after_attos.div_euclid(ATTOS_PER_SEC_I128);
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
        let tp = Dt::from_ymdhms(yr, mo, day, hr, min, sec_input, 0);

        // Verify the internal .sec value matches what was printed
        assert_eq!(
            tp.to(Scale::TAI, Scale::UTC).sec,
            expected_sec,
            "sec() mismatch for input {yr}-{mo:02}-{day:02} {hr:02}:{min:02}:{sec_input:02}"
        );

        // Round-trip test
        let g = tp.to_ymdhms(Scale::TAI);
        let tp_roundtrip =
            Dt::from_ymdhms(g.yr(), g.mo(), g.day(), g.hr(), g.min(), g.sec(), g.attos());

        assert_eq!(
            tp.sec,
            tp_roundtrip.sec,
            "roundtrip failed for input {yr}-{mo:02}-{day:02} {hr:02}:{min:02}:{sec_input:02} \
             (to_gregorian produced sec={})",
            g.sec()
        );
    }
}

#[test]
fn test_1972_leap_second_canonical_roundtrip() {
    // Create the leap second the "normal" way (using from_ymdhms)
    let original = Dt::from_ymdhms(1972, 6, 30, 23, 59, 60, 0);

    // Round-trip through attoseconds since the Unix epoch
    // (this exercises the exact civil/POSIX UTC path in to_attos_since/from_attos_since)
    let canon = original.to_diff_raw(Dt::UNIX_EPOCH).to_attos();
    let roundtrip = Dt::from_attos_since(canon, Dt::UNIX_EPOCH);

    // These should be identical if everything is consistent
    assert_eq!(
        original, roundtrip,
        "Round-trip failed for 1972 leap second"
    );

    // Also verify civil time is still correct
    let g = roundtrip.to_ymdhms(Scale::TAI);
    assert_eq!(g.yr(), 1972);
    assert_eq!(g.mo(), 6);
    assert_eq!(g.day(), 30);
    assert_eq!(g.hr(), 23);
    assert_eq!(g.min(), 59);
    assert_eq!(g.sec(), 60, "Should still show sec=60 after round-trip");
}

#[test]
fn test_leap_second_gotcha_1972_06_30() {
    let leap = Dt::from_ymdhms(1972, 6, 30, 23, 59, 60, 0);
    let g = leap.to_ymdhms(Scale::TAI);
    assert_eq!(g.sec(), 60);
    assert_eq!(g.day(), 30);
}

#[test]
fn test_leap_second_roundtrip_2015_06_30() {
    // A leap second from the middle of the table (36 leap seconds accumulated)
    let original = Dt::from_ymdhms(2015, 6, 30, 23, 59, 60, 123_456_789_000_000_000);

    // === Round-trip through canonical attoseconds ===
    let canon = original.to_diff_raw(Dt::UNIX_EPOCH).to_attos();
    let roundtrip1 = Dt::from_attos_since(canon, Dt::UNIX_EPOCH);

    assert_eq!(original, roundtrip1, "Canonical round-trip failed");

    // === Multiple Gregorian round-trips ===
    let mut current = original;
    for i in 0..5 {
        let g = current.to_ymdhms(Scale::TAI);
        assert_eq!(g.sec(), 60, "Leap second lost on iteration {}", i);
        assert_eq!(g.day(), 30);
        assert_eq!(g.mo(), 6);
        assert_eq!(g.yr(), 2015);

        current = Dt::from_ymdhms(g.yr(), g.mo(), g.day(), g.hr(), g.min(), g.sec(), g.attos());
    }
    assert_eq!(original, current, "Multiple Gregorian round-trips failed");

    // Final sanity check via to_date_time
    let gt = original.to_ymdhms_rich_on(Scale::TAI, Scale::UTC);
    assert_eq!(gt.sec(), 60);
    assert_eq!(gt.day(), 30);
}

#[cfg(feature = "std")]
#[test]
fn test_leap_seconds_file() {
    use deep_time::leap_seconds;

    let leap_seconds_table = Dt::leap_sec_data_from_file("leap-seconds.list.txt").unwrap();
    let x = Dt::from_ymdhms(2015, 6, 30, 23, 59, 60, 0);
    let leap_info = Dt::leap_sec_using(&x, false, &leap_seconds_table);
    assert!(leap_info.is_leap_sec == true);
}
