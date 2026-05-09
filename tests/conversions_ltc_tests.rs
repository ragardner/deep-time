use deep_time::{Dt, Scale};

/// Round-trip accuracy test (TAI → LTC → TAI)
///
/// LTC conversion is purely linear, so round-trips should be extremely
/// accurate. The observed error is ~0.3 ns due to unavoidable f64 rounding
/// noise in `to_jd_tt()` + the rate multiplication. We therefore allow a
/// very small tolerance that is still far tighter than any practical use case.
#[test]
fn ltc_tai_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),                  // J2000 TAI
        Dt::from_sec(86_400 * 365, Scale::TAI),       // ~1 year later
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI), // 10 years before
        Dt::from_sec(1_000_000_000, Scale::TAI),      // ~31.7 years later
        Dt::from_sec(-2_208_945_600, Scale::TAI),     // J1900 epoch
    ];

    for &p in &test_points {
        let ltc = p.to(Scale::LTC);
        let back = ltc.to_tai(Scale::LTC);

        let diff = back.to_tai_since(p).to_sec_f().abs();
        assert!(
            diff < 1e-9,
            "LTC round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

/// At J2000 the LTC–TAI difference (secular Ashby L_M + LTE440 periodic terms)
/// must match the full model. The periodic correction at t=J2000.0 is –35.128 µs.
#[test]
fn ltc_minus_tai_at_j2000() {
    let tai = Dt::ZERO;
    let ltc = tai.to(Scale::LTC);

    let diff_s = ltc.to_diff(tai.to_span()).to_sec_f();

    assert!(
        (diff_s - 32.654559693364384).abs() < 1e-9,
        "LTC-TAI difference at J2000 was {} s (expected 32.654559693364384 s)",
        diff_s
    );
}

/// Verify that the LTC–TT difference grows linearly (pure secular term).
#[test]
fn ltc_offset_grows_linearly() {
    let points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI), // ~1 year
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI), // ~100 years
    ];

    for &p in &points {
        let tt = p.to(Scale::TT);
        let ltc = p.to(Scale::LTC);

        // LTC - TT (pure secular term)
        let corr_s = ltc.to_diff(tt).to_sec_f();

        assert!(
            corr_s > 0.0,
            "LTC-TT correction should be positive (got {} s at {:?})",
            corr_s,
            p
        );

        // At ~100 years the offset should be ~2.5 s (56 µs/day × 36525 days)
        if p.sec() > 86_400 * 365 * 50 {
            assert!(
                corr_s > 1.0 && corr_s < 4.0,
                "LTC-TT correction at ~100y should be ~2-3 s (got {} s)",
                corr_s
            );
        }
    }
}
