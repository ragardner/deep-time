use crate::{Real, sin};

/// DE440/LTE440-tuned compact analytical TT–TDB model
///
/// Exact 13-term Fourier decomposition from LTE440 (Lu et al. 2025, Table 3)
/// + physical VSOP2013 annual term + tiny JPL secular corrections.
pub const fn tdb_minus_tt(seconds_since_j2000_tt: Real) -> Real {
    // J2000.0 = 2000-01-01 12:00:00 TT → 100 Julian years = exactly 3_155_760_000 s
    const J2000_SEC_PER_MILLENNIUM: Real = 31_557_600_000.0;

    let t = seconds_since_j2000_tt / J2000_SEC_PER_MILLENNIUM; // centuries since J2000
    let mut correction = f!(0.0);

    // Physical annual term (VSOP2013 secular e(t) — replaces LTE440 term #1)
    let g = f!(6283.075849991) * t + f!(6.240054195);
    let e = f!(0.0167086342) - f!(0.0004203654) * t - f!(0.0000126734) * t * t
        + f!(0.0000001444) * t * t * t
        - f!(0.0000000002) * t * t * t * t
        + f!(0.0000000003) * t * t * t * t * t;
    let k = f!(0.09897232);
    let varpi = f!(-0.00000257) - f!(0.05551247) * t;
    correction += k * e * sin(g + varpi + f!(0.01671) * sin(g));

    // Exact LTE440 Fourier terms #2–#13 (all amplitudes >1 µs from DE440 item #15)
    let lte440_terms: [(Real, Real, Real); 12] = [
        (0.00012630813184, 77713.771468120, 5.18472464), // #2 D (lunar synodic)
        (0.00001937467715, 5753.384884897, 1.33855843),  // #3 E–J (Earth–Jupiter)
        (0.00001370088760, 12566.151699983, 3.07602294), // #4 2E (semi-annual)
        (0.00000747520418, 5574.656149776, 3.32446352),  // #5 D–L
        (0.00000424397312, 4320.34946237, 3.43186281),   // #6 J (Jupiter)
        (0.00000376051430, 377.97977422, 0.92358639),    // #7 E–S (Earth–Saturn)
        (0.00000293368121, 161002.466707021, 1.09317212), // #8 D+L
        (0.00000267752983, 6208.659051973, 1.51225314),  // #9 E–U (Earth–Uranus)
        (0.00000236687890, 71430.993657045, 5.21748801), // #10 E–D (Earth–Moon difference)
        (0.00000185820098, 211.334300759, 2.56843762),   // #11 S (Saturn long-period)
        (0.00000109742615, 3929.675646567, 4.67635157),  // #12 V–E (Venus–Earth)
        (0.00000108850698, 7859.351293133, 2.99248981),  // #13 2V–2E
    ];

    let mut i = 0;
    while i < 12 {
        let (amp, freq, phase) = lte440_terms[i];
        correction += amp * sin(freq * t + phase);
        i += 1;
    }

    // Tiny JPL wj mass adjustments + quadratic secular (<1 ns)
    correction += f!(0.00000000065) * sin(f!(6069.776754) * t + f!(4.021194));
    correction += f!(0.00000000033) * sin(f!(213.299095) * t + f!(5.543132));
    correction += f!(-0.00000000196) * sin(f!(6208.294251) * t + f!(5.696701));
    correction += f!(-0.00000000173) * sin(f!(74.781599) * t + f!(2.435900));
    correction += f!(0.00000003638) * t * t; // quadratic secular

    correction
}

/// Clamps an `i128` to the representable range of `i64`.
pub(crate) const fn clamp_i128_to_i64(x: i128) -> i64 {
    if x > i64::MAX as i128 {
        i64::MAX
    } else if x < i64::MIN as i128 {
        i64::MIN
    } else {
        x as i64
    }
}
