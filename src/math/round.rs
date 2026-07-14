use super::trunc;
use crate::Real;

pub const fn round(x: Real) -> Real {
    const THRESHOLD: Real = (1u64 << 52) as Real;

    if x >= 0.0 {
        if x >= THRESHOLD { x } else { trunc(x + 0.5) }
    } else if x <= -THRESHOLD {
        x
    } else {
        trunc(x - 0.5)
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::round;

    #[test]
    fn round_matches_std_round() {
        let cases: &[f64] = &[
            0.0,
            -0.0,
            0.1,
            -0.1,
            0.4,
            -0.4,
            0.5,
            -0.5,
            0.6,
            -0.6,
            1.3,
            -1.3,
            1.5,
            -1.5,
            2.5,
            -2.5,
            123.456,
            -123.456,
            4503599627370495.5, // just below 2^52
            4503599627370496.0, // exactly 2^52
            4503599627370496.3,
            9007199254740992.0, // 2^53
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];

        for &x in cases {
            let expected = x.round();
            let got = round(x);
            assert_eq!(got, expected, "round({}) mismatch", x);
        }

        // NaN must be handled separately because NaN != NaN
        assert!(round(f64::NAN).is_nan());
        assert!(f64::NAN.round().is_nan());
    }

    #[test]
    fn round_is_const() {
        // Just to prove it can be used in const context
        const _C1: f64 = round(1.7);
        const _C2: f64 = round(-2.5);
        const _C3: f64 = round(4503599627370496.7);
    }
}
