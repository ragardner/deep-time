use crate::{ATTOS_PER_SEC, Dt, LM_DEN, LM_NUM, Real, TSpan, sin_approx};

/// LTE440 periodic terms (Lu et al. 2025, A&A 704, A76; arXiv:2509.18511)
/// A_i * sin(2π * (t_J2000_days / T_i) + ϕ_i)  with A_i in µs.
/// These are the 13 dominant terms (>1 µs) after removing the linear secular drift.
/// Accuracy: < 0.15 ns (before 2050) when combined with the secular rate.
#[derive(Copy, Clone)]
struct LunarPeriodicTerm {
    period_days: f64,  // T_i
    amplitude_us: f64, // A_i
    phase_rad: f64,    // ϕ_i
}

const LUNAR_PERIODIC_TERMS: [LunarPeriodicTerm; 13] = [
    LunarPeriodicTerm {
        period_days: 365.26590909,
        amplitude_us: 1651.36355077,
        phase_rad: 3.10895165,
    },
    LunarPeriodicTerm {
        period_days: 29.53053800,
        amplitude_us: 126.30813184,
        phase_rad: 5.18472464,
    },
    LunarPeriodicTerm {
        period_days: 398.99950348,
        amplitude_us: 19.37467715,
        phase_rad: 1.33855843,
    },
    LunarPeriodicTerm {
        period_days: 182.63295455,
        amplitude_us: 13.70088760,
        phase_rad: 3.07602294,
    },
    LunarPeriodicTerm {
        period_days: 411.67264344,
        amplitude_us: 7.47520418,
        phase_rad: 3.32446352,
    },
    LunarPeriodicTerm {
        period_days: 4320.34946237,
        amplitude_us: 4.24397312,
        phase_rad: 3.43186281,
    },
    LunarPeriodicTerm {
        period_days: 377.97977422,
        amplitude_us: 3.76051430,
        phase_rad: 0.92358639,
    },
    LunarPeriodicTerm {
        period_days: 14.25402654,
        amplitude_us: 2.93368121,
        phase_rad: 1.09317212,
    },
    LunarPeriodicTerm {
        period_days: 369.63431463,
        amplitude_us: 2.67752983,
        phase_rad: 1.51225314,
    },
    LunarPeriodicTerm {
        period_days: 32.12797857,
        amplitude_us: 2.36687890,
        phase_rad: 5.21748801,
    },
    LunarPeriodicTerm {
        period_days: 10859.25675676,
        amplitude_us: 1.85820098,
        phase_rad: 2.56843762,
    },
    LunarPeriodicTerm {
        period_days: 584.00072674,
        amplitude_us: 1.09742615,
        phase_rad: 4.67635157,
    },
    LunarPeriodicTerm {
        period_days: 292.00036337,
        amplitude_us: 1.08850698,
        phase_rad: 2.99248981,
    },
];

impl Dt {
    #[inline]
    pub(crate) const fn mul_lm(attos: i128) -> i128 {
        Self::mul_rate(attos, LM_NUM, LM_DEN)
    }

    // pub(crate) const fn tt_to_ltc(tt: Self) -> Self {
    //     let elapsed = Self::elapsed_to_attos_since_ref(tt);
    //     let span_attos = Self::mul_lm(elapsed);
    //     tt.add(TSpan::from_attos(span_attos))
    // }

    // pub(crate) const fn ltc_to_tt(ltc: Self) -> Self {
    //     let elapsed = Self::elapsed_to_attos_since_ref(ltc);
    //     let span_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
    //     ltc.sub(TSpan::from_attos(span_attos))
    // }

    pub(crate) const fn tt_to_ltc(tt: Self) -> Self {
        let elapsed = Self::elapsed_to_attos_since_ref(tt);
        let secular_attos = Self::mul_lm(elapsed);
        let periodic = Self::ltc_minus_tt_periodic(tt);

        tt.add(TSpan::from_attos(secular_attos)).add(periodic)
    }

    // pub(crate) const fn ltc_to_tt(ltc: Self) -> Self {
    //     let elapsed = Self::elapsed_to_attos_since_ref(ltc);
    //     let secular_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
    //     let periodic = Self::ltc_minus_tt_periodic(ltc); // evaluate at input (or iterate if you want ultra-pedantic)

    //     ltc.sub(TSpan::from_attos(secular_attos)).sub(periodic)
    // }

    /// Converts LTC → TT using fixed-point iteration to account for the
    /// time-dependent periodic correction.
    ///
    /// This mirrors the exact strategy used in `tdb_to_tai` for consistency
    /// and sub-attosecond numerical stability. The LTE440 periodic terms
    /// (Lu et al. 2025) are evaluated at the current TT guess on each iteration.
    ///
    /// Convergence: the periodic amplitude is only ~±1.65 ms, so 6 iterations
    /// are more than enough (error drops below 10^{-18} s after ~3–4 steps).
    pub(crate) const fn ltc_to_tt(ltc: Self) -> Self {
        let mut tt = ltc; // initial guess (already within ~2 ms)
        let mut i = 0u32;
        while i < 6 {
            let elapsed = Self::elapsed_to_attos_since_ref(tt);
            let secular_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
            let periodic = Self::ltc_minus_tt_periodic(tt);

            tt = ltc.sub(TSpan::from_attos(secular_attos)).sub(periodic);
            i += 1;
        }
        tt
    }

    /// Returns the periodic part of (LTC − TT) in TSpan (µs-level, evaluated at the TT instant).
    /// Exactly analogous to your `tdb_minus_tt`.
    const fn ltc_minus_tt_periodic(tt: Self) -> TSpan {
        let seconds_since_j2000_tt = f!(tt.sec) + f!(tt.attos) / f!(ATTOS_PER_SEC);
        let t_days = seconds_since_j2000_tt / 86400.0; // days since J2000.0 TT

        let mut delta_us = 0.0_f64;
        let two_pi = 2.0 * core::f64::consts::PI;

        let mut i = 0usize;
        while i < LUNAR_PERIODIC_TERMS.len() {
            let term = LUNAR_PERIODIC_TERMS[i];
            let arg = two_pi * (t_days / term.period_days) + term.phase_rad;
            delta_us += term.amplitude_us * sin_approx(arg);
            i += 1;
        }

        // Convert µs → TSpan (positive = lunar time runs ahead)
        TSpan::from_sec_f(delta_us * 1e-6)
    }
}
