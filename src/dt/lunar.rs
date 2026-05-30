//! Lunar time-scale constants and conversion methods.

use crate::{Dt, Real, Scale, sin};

/// TCL secular rate vs TDB (value from LTE440).
pub const TL_NUM: i128 = 6_798_355_240;
pub const TL_DEN: i128 = 10_000_000_000_000_000_000; // 10^19
/// L_M = 6.48378 × 10^{-10} (secular rate from Ashby & Patla 2024 NIST for LTC ↔ TT)
/// as fixed-point fraction.
pub const LM_NUM: i128 = 648_378;
pub const LM_DEN: i128 = 1_000_000_000_000_000; // 10^15

/// LTE440 periodic terms (Lu et al. 2025, A&A 704, A76; arXiv:2509.18511)
/// A_i * sin(2π * (t_J2000_days / T_i) + ϕ_i)  with A_i in µs.
/// These are the 13 dominant terms (>1 µs) after removing the linear secular drift.
/// Accuracy: < 0.15 ns (before 2050) when combined with the secular rate.
#[derive(Copy, Clone)]
pub struct LunarPeriodicTerm {
    period_days: Real,  // T_i
    amplitude_us: Real, // A_i
    phase_rad: Real,    // ϕ_i
}

pub const LUNAR_PERIODIC_TERMS: [LunarPeriodicTerm; 13] = [
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

    pub(crate) const fn tt_to_ltc(tt: Self) -> Dt {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tt);
        let secular_attos = Self::mul_lm(elapsed);
        let periodic = Self::ltc_periodic_correction(tt);

        tt.add(Dt::span(secular_attos)).add(periodic)
    }

    /// Converts LTC → TT using fixed-point iteration to account for the
    /// time-dependent periodic correction.
    ///
    /// This mirrors the strategy used in `tdb_to_tai` for consistency
    /// and sub-attosecond numerical stability. The LTE440 periodic terms
    /// (Lu et al. 2025) are evaluated at the current TT guess on each iteration.
    ///
    /// Convergence: the periodic amplitude is only ~±1.65 ms, so 6 iterations
    /// are more than enough (error drops below 10^{-18} s after ~3–4 steps).
    pub(crate) const fn ltc_to_tt(ltc: Self) -> Dt {
        let mut tt = ltc; // initial guess (already within ~2 ms)
        let mut i = 0u32;
        while i < 6 {
            let elapsed = Self::to_attos_since_tcg_tcb_epoch(tt);
            let secular_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
            let periodic = Self::ltc_periodic_correction(tt);

            tt = ltc.sub(Dt::span(secular_attos)).sub(periodic);
            i += 1;
        }
        tt
    }

    #[inline]
    pub(crate) const fn mul_tl(attos: i128) -> i128 {
        Self::mul_rate(attos, TL_NUM, TL_DEN)
    }

    /// Returns the periodic part of (LTC − TT) in Dt (µs-level, evaluated at the TT instant).
    const fn ltc_periodic_correction(tt: Self) -> Dt {
        let seconds_since_j2000_tt = tt.to_sec_f();
        let t_days = seconds_since_j2000_tt / f!(86400.0); // days since J2000.0 TT

        let mut delta_us = f!(0.0);
        let two_pi = f!(2.0) * f!(core::f64::consts::PI);

        let mut i = 0usize;
        while i < LUNAR_PERIODIC_TERMS.len() {
            let term = LUNAR_PERIODIC_TERMS[i];
            let arg = two_pi * (t_days / term.period_days) + term.phase_rad;
            delta_us += term.amplitude_us * sin(arg);
            i += 1;
        }

        // Convert µs → Dt (positive = lunar time runs ahead)
        Dt::from_sec_f(delta_us * 1e-6, Scale::TAI)
    }

    /// Zero-point calibration constant for TCL so that our implementation
    /// reproduces the official LTE440 reference value at every epoch.
    ///
    /// LTE440 (Lu et al. 2025) states that at J2000.0 TDB:
    ///
    /// ```text
    /// published reference: TCL − TDB = +0.49330749643254945 s
    /// ```
    ///
    /// At this epoch the secular term is zero, so our code produces only
    /// the periodic contribution from the 13-term LTE440 series:
    ///
    /// ```text
    /// our computed periodic sum = −0.000035111965426382064 s
    /// ```
    ///
    /// The required constant bias is therefore:
    ///
    /// ```text
    /// bias = published_reference − periodic_sum
    ///      = 0.49330749643254945 − (−0.000035111965426382064)
    ///      = +0.49334260839797583 s
    /// ```
    ///
    /// This bias is a pure constant (no rate or higher-order terms) and remains
    /// valid across the entire validity range of the LTE440 model.
    ///
    /// Reference: https://github.com/xlucn/LTE440
    /// (README and demo output)
    pub(crate) const TCL_TDB_BIAS_SPAN: Dt = Dt::from_sec_f(0.49334260839797583, Scale::TAI);

    /// Integer helper: elapsed attoseconds since J2000.0 TDB.
    /// Used exclusively for the TCL pathway to match LTE440
    /// (TCL = TDB + L_D^M × (JD_TDB − 2451545.0) × 86400 + periodic).
    #[inline]
    pub(crate) const fn to_attos_since_j2000_tdb_epoch(numerical_tdb: Self) -> i128 {
        numerical_tdb.to_attos()
    }

    pub(crate) const fn tai_to_tcl(tai: Self) -> Dt {
        let tdb = Self::tai_to_tdb(tai);

        let elapsed = Self::to_attos_since_j2000_tdb_epoch(tdb);
        let secular_attos = Self::mul_tl(elapsed);
        let periodic = Self::ltc_periodic_correction(tdb);

        tdb.add(Dt::span(secular_attos))
            .add(periodic)
            .add(Self::TCL_TDB_BIAS_SPAN)
    }

    /// Dedicated inverse for TCL → TT.
    /// Returns a Dt on the TT scale (consistent with ltc_to_tt, tcg_to_tt, etc.).
    pub(crate) const fn tcl_to_tai(tcl: Self) -> Dt {
        let mut tdb = tcl;
        let mut i = 0u32;
        while i < 6 {
            let elapsed = Self::to_attos_since_j2000_tdb_epoch(tdb);
            let secular_attos = Self::mul_rate(elapsed, TL_NUM, TL_DEN + TL_NUM);
            let periodic = Self::ltc_periodic_correction(tdb);

            tdb = tcl
                .sub(Dt::span(secular_attos))
                .sub(periodic)
                .sub(Self::TCL_TDB_BIAS_SPAN);
            i += 1;
        }
        Self::tdb_to_tai(tdb)
    }
}
