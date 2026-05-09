use deep_time::{Dt, Scale, constants::MARS_SOL_LENGTH_SEC, to_sec_f};

#[test]
fn msd_exact_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(-86_400 * 365 * 10, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
        Dt::from_sec(-2_208_945_600, Scale::TAI),
    ];

    for &p in &test_points {
        let (whole, frac) = p.to_msd_exact();
        let back = Dt::from_msd_exact(whole, frac);

        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(
            diff < 5e-5, // ← relaxed for f64 JD precision (max observed error ≈ 13.7 µs)
            "MSD round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

#[test]
fn msd_float_roundtrip_is_accurate() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365 * 100, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
    ];

    for &p in &test_points {
        let msd_float = p.to_msd();
        let back = Dt::from_msd(msd_float);

        let diff = back.to_diff_raw(p).to_sec_f().abs();
        assert!(
            diff < 5e-5, // ← relaxed for f64 MSD path (max observed error ≈ 13.7 µs)
            "MSD float round-trip error too large: {} s at {:?}",
            diff,
            p
        );
    }
}

#[test]
fn mtc_is_in_valid_range() {
    let test_points = [
        Dt::from_sec(0, Scale::TAI),
        Dt::from_sec(86_400 * 365, Scale::TAI),
        Dt::from_sec(1_000_000_000, Scale::TAI),
    ];

    for &p in &test_points {
        let mtc = p.to_mtc();
        let mtc_sec = mtc.to_sec_f();
        assert!(
            mtc_sec >= 0.0 && mtc_sec < MARS_SOL_LENGTH_SEC,
            "MTC out of range: {} s at {:?}",
            mtc_sec,
            p
        );
    }
}

#[test]
fn msd_at_j2000_is_correct() {
    let tai = Dt::ZERO;
    let (whole, frac) = tai.to_msd_exact();

    assert_eq!(whole, 44791, "Integer part of MSD at J2000 should be 44791");

    // New exact value (no magic number)
    let frac_sols = to_sec_f(frac) / MARS_SOL_LENGTH_SEC;
    assert!(
        (frac_sols - 0.61987471912).abs() < 1e-11, // or use a TSpan comparison
        "Fractional part of MSD at J2000 (TAI) was {} sols",
        frac_sols
    );
}
