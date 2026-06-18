#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, constants::ATTOS_PER_SEC_I128};

const APS: i128 = ATTOS_PER_SEC_I128;

#[test]
fn test_from_sec_f() {
    // Various "tough" inputs that historically exposed lossiness,
    // exact-integer edge cases, tiny fractions, negative values, etc.
    let test_cases: &[(f64, &str)] = &[
        (0.0, "zero"),
        (1.0, "positive integer"),
        (-1.0, "negative integer"),
        (0.5, "simple fraction"),
        (1.0 + 1e-10, "just above integer"),
        (1.0 - 1e-10, "just below integer"),
        (1.0 + 1e-16, "very tiny positive frac"),
        (1.0 - 1e-16, "very tiny negative frac"),
        (1e-17, "extremely small positive"),
        (-1e-17, "extremely small negative"),
        // The exact value that was failing in the saturation test
        (
            (0.81f64 * 0.81 - 0.81 + 1.0).sqrt() - 1.0,
            "saturation test δ=0.81 (the one that differed by 111 attos)",
        ),
        // Values near mantissa boundaries (where old lossy path was worst)
        (1.0 + 0.9999999999999999, "almost 2.0"),
        (-0.0000000000000001, "tiny negative near zero"),
        (123.45678901234567, "random-looking decimal"),
        (1e10 + 0.123, "large integer + fraction"),
    ];

    for (sec_f, label) in test_cases {
        let dt = Dt::span_f(*sec_f);
        let roundtrip = dt.to_sec_f();

        assert_eq!(
            roundtrip, *sec_f,
            "Roundtrip failed for input {} ({})\n  → Dt:   {:?}\n  → back: {}",
            sec_f, label, dt, roundtrip
        );
    }
}

#[test]
fn test_mul_by_f() {
    let three_sec = Dt::span(3 * APS);
    let neg_three_sec = Dt::span(-(3 * APS));
    let two_sec = Dt::span(2 * APS);

    // Integer and fractional products (exact i128 path for the whole part)
    assert_eq!(three_sec.mul_by_f(2.0).to_attos(), 6 * APS);
    assert_eq!(three_sec.mul_by_f(0.5).to_attos(), (3 * APS) / 2);
    assert_eq!(three_sec.mul_by_f(-2.5).to_attos(), -(7 * APS + APS / 2));
    assert_eq!(neg_three_sec.mul_by_f(2.0).to_attos(), -(6 * APS));
    assert_eq!(neg_three_sec.mul_by_f(-2.5).to_attos(), 7 * APS + APS / 2);
    assert_eq!(two_sec.mul_by_f(-1.0).to_attos(), -(2 * APS));

    // Special floats
    assert_eq!(three_sec.mul_by_f(f64::NAN), Dt::ZERO);
    assert_eq!(Dt::ZERO.mul_by_f(f64::INFINITY), Dt::ZERO);
    assert_eq!(three_sec.mul_by_f(0.0), Dt::ZERO);
    assert_eq!(three_sec.mul_by_f(f64::INFINITY), Dt::MAX);
    assert_eq!(three_sec.mul_by_f(f64::NEG_INFINITY), Dt::MIN);

    // Saturation
    assert_eq!(Dt::MAX.mul_by_f(1.0), Dt::MAX);
    assert_eq!(Dt::MAX.mul_by_f(2.0), Dt::MAX);
    assert_eq!(Dt::MIN.mul_by_f(2.0), Dt::MIN);

    // div_by_f delegates here
    assert_eq!(Dt::span(10 * APS).div_by_f(4.0).to_attos(), (10 * APS) / 4);
}
