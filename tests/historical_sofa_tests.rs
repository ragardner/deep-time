use deep_time::{Dt, Scale, historical_sofa::historical_sofa_offset_for_non_adjusted};

#[test]
fn test_sofa_historical_offsets() {
    let tp = Dt::from_ymdhms_on(
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
        tp.sec(),
        tp2.sec(),
        "Round trip just before SOFA start changed integer seconds"
    );
    assert_eq!(
        tp.attos(),
        tp2.attos(),
        "Round trip just before SOFA start changed attoseconds"
    );

    // SHOULD RETURN NONE
    // 1960-12-31 (one day before first entry)
    let tp = Dt::from_ymd_on(1960, 12, 31, Scale::UTCSofa);
    assert!(
        historical_sofa_offset_for_non_adjusted(&tp).is_none(),
        "1960-12-31 should return None"
    );

    // 1972-01-01 (first day of modern leap-second system)
    let tp = Dt::from_ymd_on(1972, 1, 1, Scale::UTCSofa);
    assert!(
        historical_sofa_offset_for_non_adjusted(&tp).is_none(),
        "1972-01-01 should return None"
    );

    // These expected values come from the official SOFA/ERFA formula:
    // offset = entry.offset + (MJD − entry.mjd_ref) × entry.drift
    // Verified against erfa.dat() at runtime.

    // 1961-01-01 00:00:00 UTC → uses 1961-01-01 entry
    let tp = Dt::from_ymd_on(1961, 1, 1, Scale::UTC);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-12,
        "1961-01-01 offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01 00:00:00 UTC → uses 1966-01-01 entry (drift continues)
    let tp = Dt::from_ymd_on(1966, 5, 1, Scale::UTC);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-12,
        "1966-05-01 offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31 00:00:00 UTC → uses 1968-02-01 entry (last rubber-era entry)
    let tp = Dt::from_ymd_on(1971, 12, 31, Scale::UTC);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-12,
        "1971-12-31 offset was {}, expected 9.889650000000",
        offset
    );

    // 1961-01-01
    let tp = Dt::from_ymd_on(1961, 1, 1, Scale::UTCSofa);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-6,
        "1961-01-01 inverse offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01
    let tp = Dt::from_ymd_on(1966, 5, 1, Scale::UTCSofa);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-6,
        "1966-05-01 inverse offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31
    let tp = Dt::from_ymd_on(1971, 12, 31, Scale::UTCSofa);
    let offset = historical_sofa_offset_for_non_adjusted(&tp).unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-6,
        "1971-12-31 inverse offset was {}, expected 9.889650000000",
        offset
    );
}
