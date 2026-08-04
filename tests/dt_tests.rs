#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::macros::{dt, from_sec_f};
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
        let dt = from_sec_f!(*sec_f);
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
    let three_sec = dt!(3 * APS);
    let neg_three_sec = dt!(-(3 * APS));
    let two_sec = dt!(2 * APS);

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
    assert_eq!(dt!(10 * APS).div_by_f(4.0).to_attos(), (10 * APS) / 4);
}

#[test]
fn test_neg_saturates_at_min() {
    // Normal cases
    assert_eq!(dt!(5).neg().to_attos(), -5);
    assert_eq!(dt!(-5).neg().to_attos(), 5);
    assert_eq!((-dt!(5)).to_attos(), -5);

    // MAX negates cleanly; MIN saturates to MAX (no overflow panic)
    // i128::saturating_neg: −MIN is not representable → clamp to MAX
    assert_eq!(Dt::MAX.neg().to_attos(), -i128::MAX);
    assert_eq!(Dt::MIN.neg(), Dt::MAX);
    assert_eq!(-Dt::MIN, Dt::MAX);
}

#[test]
fn test_sec_f_to_attos_saturates_large_negatives() {
    // Magnitudes above ~1.7e20 s overflow i128 attoseconds. Positive path already
    // returned MAX; negatives used to put MIN into abs_total then negate → panic.
    assert_eq!(Dt::sec_f_to_attos(1.8e20), i128::MAX);
    assert_eq!(Dt::sec_f_to_attos(-1.8e20), i128::MIN);
    assert_eq!(Dt::sec_f_to_attos(2e20), i128::MAX);
    assert_eq!(Dt::sec_f_to_attos(-2e20), i128::MIN);
    assert_eq!(Dt::sec_f_to_attos(1e25), i128::MAX);
    assert_eq!(Dt::sec_f_to_attos(-1e25), i128::MIN);

    // total_exp > 120 early path (still MIN/MAX)
    assert_eq!(Dt::sec_f_to_attos(f64::MAX), i128::MAX);
    assert_eq!(Dt::sec_f_to_attos(-f64::MAX), i128::MIN);

    // Under the limit: still finite and signed correctly
    assert!(Dt::sec_f_to_attos(1e19) > 0);
    assert!(Dt::sec_f_to_attos(-1e19) < 0);
    assert_eq!(Dt::sec_f_to_attos(-1e19), -Dt::sec_f_to_attos(1e19));

    // Call-chain coverage
    use deep_time::Scale;
    assert_eq!(
        Dt::from_sec_f(-2e20, Scale::TAI, Scale::TAI).to_attos(),
        i128::MIN
    );
    assert_eq!(
        Dt::from_days_f(-1e30, Scale::TAI, Scale::TAI).to_attos(),
        i128::MIN
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
        // edges of the Dt range, including near MIN where sec × 10¹⁸ is tight
        i128::MAX,
        i128::MIN,
        i128::MIN + 1,
        i128::MAX - 1,
        // largest whole-second magnitude that still multiplies cleanly by 10¹⁸
        -170_141_183_460_469_231_731 * APS,
        -170_141_183_460_469_231_731 * APS + (APS - 1),
    ];

    for attos in cases {
        let dt = dt!(attos);
        let rebuilt = Dt::from_sec_and_frac(
            dt.to_sec(),
            dt.to_sec_frac() as i128,
            Scale::TAI,
            Scale::TAI,
        );
        assert_eq!(dt, rebuilt, "trunc round-trip failed for {attos} attos");

        // same split to_ymd uses: floor seconds + non-negative fraction
        let rebuilt_floor = Dt::from_sec_and_frac(
            dt.to_sec_floor(),
            dt.to_sec_ufrac() as i128,
            Scale::TAI,
            Scale::TAI,
        );
        assert_eq!(
            dt, rebuilt_floor,
            "floor/ufrac round-trip failed for {attos} attos"
        );
    }
}

/// `to_ymd` covers the full range of `Dt` (including MIN/MAX) and round-trips
#[test]
fn test_to_ymd_full_i128_attosecond_range() {
    use deep_time::Scale;

    let extremes = [i128::MAX, i128::MIN, i128::MAX - 1, i128::MIN + 1];
    for attos in extremes {
        let dt = Dt::new(attos, Scale::TAI, Scale::TAI);
        let ymd = dt.to_ymd();

        assert_eq!(
            ymd.to_dt().to_attos(),
            attos,
            "embedded Dt lost for {attos}"
        );
        assert!(
            (1..=12).contains(&ymd.mo()),
            "month for {attos}: {}",
            ymd.mo()
        );
        assert!(
            (1..=31).contains(&ymd.day()),
            "day for {attos}: {}",
            ymd.day()
        );
        assert!(ymd.hr() <= 23, "hour for {attos}: {}", ymd.hr());
        assert!(ymd.min() <= 59, "min for {attos}: {}", ymd.min());
        assert!(ymd.sec() <= 60, "sec for {attos}: {}", ymd.sec());
        assert!(
            ymd.attos() < APS as u64,
            "frac for {attos}: {}",
            ymd.attos()
        );

        let back = Dt::from_ymd(
            ymd.yr(),
            ymd.mo(),
            ymd.day(),
            Scale::TAI,
            ymd.hr(),
            ymd.min(),
            ymd.sec(),
            ymd.attos(),
        );
        assert_eq!(
            back.to_attos(),
            attos,
            "from_ymd(to_ymd) lost attos for {attos}; ymd yr={}",
            ymd.yr()
        );
    }

    // larger than i64 seconds must not share the civil date of i64::MAX seconds
    let past_i64 = Dt::from_sec((i64::MAX as i128) + 1, Scale::TAI, Scale::TAI);
    let at_i64 = Dt::from_sec(i64::MAX as i128, Scale::TAI, Scale::TAI);
    let y_past = past_i64.to_ymd();
    let y_at = at_i64.to_ymd();
    assert!(
        y_past.yr() > y_at.yr()
            || y_past.mo() > y_at.mo()
            || y_past.day() > y_at.day()
            || y_past.hr() > y_at.hr()
            || y_past.min() > y_at.min()
            || y_past.sec() > y_at.sec(),
        "to_ymd still saturates at i64 seconds: past={:?} at={:?}",
        (
            y_past.yr(),
            y_past.mo(),
            y_past.day(),
            y_past.hr(),
            y_past.min(),
            y_past.sec()
        ),
        (
            y_at.yr(),
            y_at.mo(),
            y_at.day(),
            y_at.hr(),
            y_at.min(),
            y_at.sec()
        ),
    );

    let back_past = Dt::from_ymd(
        y_past.yr(),
        y_past.mo(),
        y_past.day(),
        Scale::TAI,
        y_past.hr(),
        y_past.min(),
        y_past.sec(),
        y_past.attos(),
    );
    assert_eq!(back_past, past_i64);

    // MIN/MAX must not look like the i64 second min/max dates
    let y_max = Dt::new(i128::MAX, Scale::TAI, Scale::TAI).to_ymd();
    assert_ne!(
        (y_max.yr(), y_max.mo(), y_max.day()),
        (y_at.yr(), y_at.mo(), y_at.day()),
        "i128::MAX to_ymd collapsed to i64::MAX-seconds date"
    );
    let y_min = Dt::new(i128::MIN, Scale::TAI, Scale::TAI).to_ymd();
    let at_i64_min = Dt::from_sec(i64::MIN as i128, Scale::TAI, Scale::TAI).to_ymd();
    assert_ne!(
        (y_min.yr(), y_min.mo(), y_min.day()),
        (at_i64_min.yr(), at_i64_min.mo(), at_i64_min.day()),
        "i128::MIN to_ymd collapsed to i64::MIN-seconds date"
    );
}
