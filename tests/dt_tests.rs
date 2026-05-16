use deep_time::Dt;

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

    eprintln!("=== from_sec_f Precision Diagnostics ===");
    eprintln!("(new high-precision implementation)\n");

    for (sec_f, label) in test_cases {
        let dt = Dt::from_sec_f(*sec_f);

        let roundtrip = dt.to_sec_f();

        eprintln!("Input: {:>18.20e}  ({})", sec_f, label);
        eprintln!("  → Dt:   {:?}", dt);
        eprintln!(
            "  → back: {:>18.20e}  (diff = {:e})",
            roundtrip,
            (roundtrip - sec_f).abs()
        );
        eprintln!();
    }

    eprintln!("=== End of diagnostics ===");
}
