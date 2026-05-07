use deep_time::{
    Dt, Scale, TSpan,
    historical_sofa::{historical_sofa_for_tai_to_utc, historical_sofa_for_utc_to_tai},
};

#[test]
fn test_sofa_historical_offsets() {
    // Start with a UTCSofa instant in the rubber era, to tai
    let original = Dt::from_ymd(1971, 12, 31, Scale::UTCSofa);

    // Convert to utc sofa (applies historical rubber offset)
    let utc_sofa = original.to(Scale::UTCSofa);

    // Convert back to UTCSofa (should subtract the same offset)
    let round_tripped = utc_sofa.to_tai(Scale::UTCSofa);

    // Compare subsec (attoseconds) directly — this avoids f64 precision loss.
    // The round-trip should be accurate to well under 1 nanosecond.
    // (We allow up to 1 ns = 1_000_000_000_000 attoseconds of tolerance.)
    let subsec_diff = (round_tripped.attos() as i128 - original.attos() as i128).abs();
    assert!(
        subsec_diff < 1_000_000_000_000,
        "Round-trip 1971-12-31 subsec diff was {} attoseconds (expected < 1 ns)",
        subsec_diff
    );

    // Also verify the integer seconds are identical
    assert_eq!(
        round_tripped.sec(),
        original.sec(),
        "Round-trip changed the integer seconds!"
    );

    let tp = Dt::from_ymdhms(
        1960,
        12,
        31,
        23,
        59,
        59,
        999_999_999_999_999_999,
        Scale::UTCSofa,
    );
    let tp2 = tp.to(Scale::UTCSofa).to_tai(Scale::UTCSofa);
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
    let tp = Dt::from_ymd(1960, 12, 31, Scale::UTC);
    assert!(
        historical_sofa_for_utc_to_tai(&tp).is_none(),
        "1960-12-31 should return None"
    );

    let tp = Dt::from_ymd(1960, 12, 31, Scale::UTCSofa);
    assert!(
        historical_sofa_for_tai_to_utc(&tp).is_none(),
        "1960-12-31 TAI should return None for inverse"
    );

    // 1972-01-01 (first day of modern leap-second system)
    let tp = Dt::from_ymd(1972, 1, 1, Scale::UTCSofa);
    assert!(
        historical_sofa_for_utc_to_tai(&tp).is_none(),
        "1972-01-01 should return None (use normal leap second path)"
    );

    let tp = Dt::from_ymd(1972, 1, 1, Scale::UTCSofa);
    assert!(
        historical_sofa_for_tai_to_utc(&tp).is_none(),
        "1972-01-01 TAI should return None for inverse"
    );

    // These expected values come from the official SOFA/ERFA formula:
    // offset = entry.offset + (MJD − entry.mjd_ref) × entry.drift
    // Verified against erfa.dat() at runtime.

    // 1961-01-01 00:00:00 UTC → uses 1961-01-01 entry
    let tp = Dt::from_ymd(1961, 1, 1, Scale::UTC);
    let offset = historical_sofa_for_utc_to_tai(&tp).unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-12,
        "1961-01-01 offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01 00:00:00 UTC → uses 1966-01-01 entry (drift continues)
    let tp = Dt::from_ymd(1966, 5, 1, Scale::UTC);
    let offset = historical_sofa_for_utc_to_tai(&tp).unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-12,
        "1966-05-01 offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31 00:00:00 UTC → uses 1968-02-01 entry (last rubber-era entry)
    let tp = Dt::from_ymd(1971, 12, 31, Scale::UTC);
    let offset = historical_sofa_for_utc_to_tai(&tp).unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-12,
        "1971-12-31 offset was {}, expected 9.889650000000",
        offset
    );

    // 1961-01-01
    let tp = Dt::from_ymd(1961, 1, 1, Scale::UTCSofa);
    let offset = historical_sofa_for_tai_to_utc(&tp).unwrap();
    assert!(
        (offset - 1.422818000000).abs() < 1e-6,
        "1961-01-01 inverse offset was {}, expected 1.422818000000",
        offset
    );

    // 1966-05-01
    let tp = Dt::from_ymd(1966, 5, 1, Scale::UTCSofa);
    let offset = historical_sofa_for_tai_to_utc(&tp).unwrap();
    assert!(
        (offset - 4.624210000000).abs() < 1e-6,
        "1966-05-01 inverse offset was {}, expected 4.624210000000",
        offset
    );

    // 1971-12-31
    let tp = Dt::from_ymd(1971, 12, 31, Scale::UTCSofa);
    let offset = historical_sofa_for_tai_to_utc(&tp).unwrap();
    assert!(
        (offset - 9.889650000000).abs() < 1e-6,
        "1971-12-31 inverse offset was {}, expected 9.889650000000",
        offset
    );

    // Sofa from/to attos
    let tp1 = Dt::from_ymd(1971, 12, 31, Scale::UTCSofa);
    let out_attos = tp1.to_epoch(Dt::UNIX_EPOCH, Scale::UTCSofa).to_attos();
    let tp2 = Dt::from_epoch(TSpan::from_attos(out_attos), Dt::UNIX_EPOCH, Scale::UTCSofa);
    assert!(
        tp1.to_tai_since_f(tp2).abs() < 1e-6,
        "SOFA round trip using to_epoch and from_epoch too large"
    );
}
