//! Round-trip tests for all wire serialization formats.
//!
//! Run with: `cargo test --test wire_roundtrip`

use deep_time_core::{
    ClockDrift, ClockModel, ClockType, DateComponents, Delta, GregorianPoint, Meridiem, TimePoint,
    TimeRange, TimeZone, Weekday,
};

/// Helper function to test round-trip serialization/deserialization.
fn assert_roundtrip<T>(
    original: &T,
    to_bytes: impl Fn(&T) -> Vec<u8>,
    from_bytes: impl Fn(&[u8]) -> Option<T>,
) where
    T: PartialEq + std::fmt::Debug,
{
    let bytes = to_bytes(original);
    let recovered = from_bytes(&bytes).expect("deserialization failed");
    assert_eq!(original, &recovered, "Round-trip failed");
}

#[test]
fn test_delta_roundtrip() {
    let delta = Delta::from_sec(123456789) + Delta::from_ns(987654321);
    assert_roundtrip(
        &delta,
        |d| d.to_wire_bytes().to_vec(),
        Delta::from_wire_bytes,
    );
}

#[test]
fn test_timepoint_roundtrip() {
    let tp = TimePoint::new(9876543210, 123456789012345678, ClockType::TAI);
    assert_roundtrip(
        &tp,
        |t| t.to_wire_bytes().to_vec(),
        TimePoint::from_wire_bytes,
    );
}

#[test]
fn test_clockdrift_roundtrip() {
    let drift = ClockDrift::new(Delta::from_sec(5), Delta::from_ns(1), Delta::from_as(2));
    assert_roundtrip(
        &drift,
        |d| d.to_wire_bytes().to_vec(),
        ClockDrift::from_wire_bytes,
    );
}

#[test]
fn test_clockmodel_roundtrip() {
    let model = ClockModel::proper(
        TimePoint::new(0, 0, ClockType::TAI),
        ClockDrift::from_offset_and_rate(Delta::from_sec(42), Delta::from_ns(1)),
    );
    assert_roundtrip(
        &model,
        |m| m.to_wire_bytes().to_vec(),
        ClockModel::from_wire_bytes,
    );
}

#[test]
fn test_timerange_roundtrip() {
    let start = TimePoint::new(1000000000, 0, ClockType::TAI);
    let end = start + Delta::from_hr(24);
    let step = Delta::from_hr(1);
    let range = start.range_to(end, step);

    assert_roundtrip(
        &range,
        |r| r.to_wire_bytes().to_vec(),
        TimeRange::from_wire_bytes,
    );
}

#[test]
fn test_gregorianpoint_roundtrip() {
    let gp = GregorianPoint::new(
        1_700_000_000_000_000_000_000_000_000, // unix_attosec
        2024,                                  // yr
        12,                                    // mo
        25,                                    // day
        12,                                    // hr
        0,                                     // min
        0,                                     // sec
        123456789012345678,                    // attos
        2024,                                  // iso_yr
        52,                                    // iso_wk
        Weekday::Wednesday,                    // iso_wkday
        360,                                   // day_of_yr
        3,                                     // wkday
        (2460670, Delta::from_sec(43200)),     // jd_tt_exact
        51,                                    // wk_of_yr_sun
        52,                                    // wk_of_yr_mon
        Some(0),                               // offset_sec
        None,                                  // tz
        None,                                  // tz_abbrev
        ClockType::UTC,                        // clock_type
    );

    assert_roundtrip(
        &gp,
        |g| g.to_wire_bytes().to_vec(),
        GregorianPoint::from_wire_bytes,
    );
}

#[test]
fn test_datecomponents_roundtrip() {
    let mut dc = DateComponents::default();
    dc.year = Some(2025);
    dc.month = Some(6);
    dc.day = Some(15);
    dc.hour = Some(14);
    dc.minute = Some(30);
    dc.second = Some(0);
    dc.attos = Some(0);
    dc.clock_type = ClockType::TAI;
    dc.tz = Some(TimeZone::Utc);

    assert_roundtrip(
        &dc,
        |d| d.to_wire_bytes().to_vec(),
        DateComponents::from_wire_bytes,
    );
}

#[test]
fn test_small_enums_roundtrip() {
    // Meridiem
    assert_roundtrip(
        &Meridiem::AM,
        |m| vec![m.to_wire_byte()],
        |b| Meridiem::from_wire_byte(b[0]),
    );
    assert_roundtrip(
        &Meridiem::PM,
        |m| vec![m.to_wire_byte()],
        |b| Meridiem::from_wire_byte(b[0]),
    );

    // Weekday
    for wd in [Weekday::Sunday, Weekday::Wednesday, Weekday::Saturday] {
        assert_roundtrip(
            &wd,
            |w| vec![w.to_wire_byte()],
            |b| Weekday::from_wire_byte(b[0]),
        );
    }

    // TimeZone
    let zones = [TimeZone::Utc, TimeZone::None, TimeZone::Fixed(3600)];
    for tz in zones {
        assert_roundtrip(
            &tz,
            |t| t.to_wire_bytes().to_vec(),
            TimeZone::from_wire_bytes,
        );
    }
}
