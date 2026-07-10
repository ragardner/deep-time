#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, consts::ATTOS_PER_SEC_I128};

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
    let three_sec = deep_time::dt!(3 * APS);
    let neg_three_sec = deep_time::dt!(-(3 * APS));
    let two_sec = deep_time::dt!(2 * APS);

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
    assert_eq!(
        deep_time::dt!(10 * APS).div_by_f(4.0).to_attos(),
        (10 * APS) / 4
    );
}

#[test]
fn test_from_str_sec_f() {
    use deep_time::Scale;

    // Basic positive
    let d = Dt::from_str_sec_f("123", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 123);
    assert_eq!(d.to_sec_ufrac(), 0);

    let d = Dt::from_str_sec_f("123.5", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 123);
    assert_eq!(d.to_sec_ufrac(), 500_000_000_000_000_000);

    // Negative integer
    let d = Dt::from_str_sec_f("-42", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), -42);

    // Negative with fraction
    let d = Dt::from_str_sec_f("-1.25", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), -2); // accessors use floor + positive ufrac
    assert_eq!(d.to_sec_ufrac(), 750_000_000_000_000_000);

    // Leading dot positive and negative (the special < 1 negative case)
    let d = Dt::from_str_sec_f(".5", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 0);
    assert_eq!(d.to_sec_ufrac(), 500_000_000_000_000_000);

    let d = Dt::from_str_sec_f("-.5", Some(Scale::TAI)).unwrap();
    assert!(d.to_attos() < 0);
    // -0.5 should be represented as sec=-1 + 0.5 ufrac in the pair
    assert_eq!(d.to_sec64_floor(), -1);
    assert_eq!(d.to_sec_ufrac(), 500_000_000_000_000_000);

    // Explicit positive sign
    let d = Dt::from_str_sec_f("+0.25", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec_ufrac(), 250_000_000_000_000_000);

    // Trailing dot
    let d = Dt::from_str_sec_f("99.", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 99);
    assert_eq!(d.to_sec_ufrac(), 0);

    // Full 18 fractional digits (exact attos)
    let d = Dt::from_str_sec_f("0.123456789012345678", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec_ufrac(), 123_456_789_012_345_678);

    // More than 18 frac digits → truncated (first 18 used)
    let d = Dt::from_str_sec_f("0.1234567890123456789", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec_ufrac(), 123_456_789_012_345_678);

    // Tiny 1 attosecond
    let d = Dt::from_str_sec_f("0.000000000000000001", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec_ufrac(), 1);

    // Leading + and dot
    let d = Dt::from_str_sec_f("+.000000000000000001", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec_ufrac(), 1);

    // Error cases (completely unparseable)
    assert!(Dt::from_str_sec_f("", Some(Scale::TAI)).is_none());
    assert!(Dt::from_str_sec_f("-", Some(Scale::TAI)).is_none());
    assert!(Dt::from_str_sec_f(".", Some(Scale::TAI)).is_none());
    assert!(Dt::from_str_sec_f("abc", Some(Scale::TAI)).is_none());
    assert!(Dt::from_str_sec_f("+", Some(Scale::TAI)).is_none());
    assert!(Dt::from_str_sec_f("---", Some(Scale::TAI)).is_none());

    // With new tolerant parsing, these now succeed (leading junk skipped / trailing ignored)
    let d = Dt::from_str_sec_f("123x", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 123);

    let d = Dt::from_str_sec_f("prefix:123.45.67", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), 123);
    assert_eq!(d.to_sec_ufrac(), 450_000_000_000_000_000);

    // Leading junk skipped, trailing ignored
    let d = Dt::from_str_sec_f("time = -42.75 (end)", Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), -43);
    assert_eq!(d.to_sec_ufrac(), 250_000_000_000_000_000);

    // Very large (but valid) i64 — no clamping
    let big = "9223372036854775807"; // i64::MAX
    let d = Dt::from_str_sec_f(big, Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), i64::MAX);

    // i64::MIN
    let min = "-9223372036854775808";
    let d = Dt::from_str_sec_f(min, Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), i64::MIN);

    // Extremely large integers saturate (to_sec* views are clamped by the library)
    let huge_pos = "1234567890123456789012345678901234567890.123";
    let d = Dt::from_str_sec_f(huge_pos, Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), i64::MAX);
    assert!(d.to_sec_ufrac() > 0);

    let huge_neg = "-99999999999999999999999999999999999999.9";
    let d = Dt::from_str_sec_f(huge_neg, Some(Scale::TAI)).unwrap();
    assert_eq!(d.to_sec64_floor(), i64::MIN);

    // Length limit (STRTIME_SIZE)
    let too_long = "1".repeat(600);
    assert!(Dt::from_str_sec_f(&too_long, Some(Scale::TAI)).is_none());

    // Optional scale parsing when passing None (trailing abbrev like ISO parser)
    // Use GPS as a concrete non-default scale.
    let d = Dt::from_str_sec_f("123.5 GPS", None).unwrap();
    assert_eq!(d.target, Scale::GPS);

    // Equivalent to explicit Some(GPS)
    let d_auto = Dt::from_str_sec_f("9876.25 GPS", None).unwrap();
    let d_exp = Dt::from_str_sec_f("9876.25", Some(Scale::GPS)).unwrap();
    assert_eq!(d_auto, d_exp);

    // Scale after whitespace / with leading junk
    let d = Dt::from_str_sec_f("val= 10 TAI", None).unwrap();
    assert_eq!(d.target, Scale::TAI);

    // No scale present -> defaults to TAI
    let d = Dt::from_str_sec_f("55.5", None).unwrap();
    assert_eq!(d.target, Scale::TAI);
}

#[test]
fn test_from_sec_and_frac_round_trip() {
    use deep_time::Scale;

    let cases = [
        0i128,
        1_300_000_000_000_000_000,
        -1_300_000_000_000_000_000,
        -500_000_000_000_000_000,
        500_000_000_000_000_000,
        123_456_789_012_345_678,
        -123_456_789_012_345_678,
    ];

    for attos in cases {
        let dt = deep_time::dt!(attos);
        let rebuilt = Dt::from_sec_and_frac(
            dt.to_sec(),
            dt.to_sec_frac() as i128,
            Scale::TAI,
            Scale::TAI,
        );
        assert_eq!(dt, rebuilt, "round-trip failed for {attos} attos");
    }
}
