//! Lunar time-scale constants and conversion methods.
//!
//! - **TCL** — IAU Lunar Coordinate Time (selenocenter). Approximate model:
//!   `TCL − TDB = L_D^M · (t − t₀) + P₁₃(t)` with `t` on TDB, `t₀` = 1977-01-01
//!   (IAU common epoch). `L_D^M` and the 13-term series come from the LTE440
//!   papers (Lu et al. 2025); this is **not** the full LTE440 Chebyshev product.
//! - **LTC** — mean-selenoid scale from TCL via Ashby & Patla (2024) `L_m`,
//!   analogous to TT from TCG: `LTC = TCL − L_m · (TCL − t₀)`. This is **not**
//!   a finalized international “Coordinated Lunar Time” standard; it is this
//!   library’s construction. Mean `dLTC/dTT − 1 ≈ L_D^M − L_m ≈ 6.48×10⁻¹⁰`.

use crate::{Dt, Real, Scale, dt, sin};

/// \(L_D^M = 6.798\,355\,238\times 10^{-10}\) — LTE440 secular rate
/// \(\langle\mathrm{d\,TCL}/\mathrm{d\,TDB}\rangle - 1\) (user manual).
pub const TL_NUM: i128 = 6_798_355_238;
/// Denominator for [`TL_NUM`] fixed-point fraction (\(10^{19}\)).
pub const TL_DEN: i128 = 10_000_000_000_000_000_000; // 10^19

/// \(L_m = 3.13881\times 10^{-11}\) — Ashby & Patla (2024) lunar selenoid
/// scaling constant (analogous to \(L_G\) for TCG → TT).
///
/// Used only for LTC ↔ TCL: `LTC = TCL − L_m · (TCL − t₀)`.
pub const LM_NUM: i128 = 313_881;
/// Denominator for [`LM_NUM`] fixed-point fraction (\(10^{16}\)).
pub const LM_DEN: i128 = 10_000_000_000_000_000; // 10^16

/// Ashby & Patla (2024) mean surface-to-surface rate
/// \(L_M = 6.48378\times 10^{-10}\) (\(\mathrm{d}\tau_m/\mathrm{d}\tau_e - 1\)).
///
/// Informational: expected mean `dLTC/dTT − 1 ≈ L_D^M − L_m ≈ L_M`.
/// Not used directly in the conversion (LTC is derived from TCL via [`LM_NUM`]).
pub const ASHBY_LM_NUM: i128 = 648_378;
/// Denominator for [`ASHBY_LM_NUM`] (\(10^{15}\)).
pub const ASHBY_LM_DEN: i128 = 1_000_000_000_000_000; // 10^15

/// One LTE440 periodic term for **TCL − TDB**.
///
/// Form: `A_i * sin(2π * (t_J2000_days / T_i) + ϕ_i)` with `A_i` in µs and
/// time argument on **TDB** (Lu et al. 2025, A&A 704, A76; arXiv:2509.18511).
#[derive(Clone, Debug)]
pub struct LunarPeriodicTerm {
    /// Period \(T_i\) in days.
    pub period_days: Real,
    /// Amplitude \(A_i\) in microseconds.
    pub amplitude_us: Real,
    /// Phase \(\phi_i\) in radians.
    pub phase_rad: Real,
}

/// The 13 dominant LTE440 periodic terms in TCL−TDB with amplitude > 1 µs
/// (preliminary FFT table after removal of the linear secular drift).
///
/// This is an analytical approximation to the full LTE440 Chebyshev product,
/// not a substitute for it at sub-nanosecond accuracy.
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
    // -------------------------------------------------------------------------
    // Fixed-point rate helpers
    // -------------------------------------------------------------------------

    /// \(L_D^M · \mathrm{attos}\) (TCL secular vs TDB).
    #[inline(always)]
    pub(crate) const fn mul_tl(attos: i128) -> i128 {
        Self::mul_rate(attos, TL_NUM, TL_DEN)
    }

    /// \(L_m · \mathrm{attos}\) (LTC scaling vs TCL).
    #[inline(always)]
    pub(crate) const fn mul_lm(attos: i128) -> i128 {
        Self::mul_rate(attos, LM_NUM, LM_DEN)
    }

    // -------------------------------------------------------------------------
    // TCL − TDB periodic series (argument = TDB days since J2000)
    // -------------------------------------------------------------------------

    /// Periodic part of TCL − TDB from the 13-term LTE440 table.
    ///
    /// `tdb` must carry a TDB continuous count since J2000 (only the numerical
    /// value is used). Result is a pure span (scale tags unused by add/sub).
    const fn tcl_tdb_periodic(tdb: Dt) -> Dt {
        let t_days = tdb.to_sec_f() / f!(86400.0);

        let mut delta_us = f!(0.0);
        let two_pi = f!(2.0) * f!(core::f64::consts::PI);

        let mut i = 0usize;
        while i < LUNAR_PERIODIC_TERMS.len() {
            let term = &LUNAR_PERIODIC_TERMS[i];
            let arg = two_pi * (t_days / term.period_days) + term.phase_rad;
            delta_us += term.amplitude_us * sin(arg);
            i += 1;
        }

        Dt::from_sec_f(delta_us * 1e-6, Scale::TAI, Scale::TAI)
    }

    // -------------------------------------------------------------------------
    // TCL ↔ TAI (via TDB)
    // -------------------------------------------------------------------------

    /// TAI → TCL: `TCL = TDB + L_D^M·(TDB−t₀) + P₁₃(TDB)` (`t₀` = 1977 epoch).
    pub(crate) const fn tai_to_tcl(tai: Dt) -> Dt {
        let tdb = tai.tai_to_tdb();

        let elapsed = tdb.to_attos_since_tcg_tcb_epoch();
        let secular_attos = Self::mul_tl(elapsed);
        let periodic = Self::tcl_tdb_periodic(tdb);

        tdb.add(dt!(secular_attos)).add(periodic)
    }

    /// TCL → TAI (fixed-point iteration on TDB).
    pub(crate) const fn tcl_to_tai(tcl: Dt) -> Dt {
        let mut tdb = tcl;
        let mut i = 0u32;
        while i < 6 {
            let periodic = Self::tcl_tdb_periodic(tdb);
            let eff = tcl.sub(periodic);

            let elapsed_eff = eff.to_attos_since_tcg_tcb_epoch();
            let sec_inv_attos = Self::mul_rate(elapsed_eff, TL_NUM, TL_DEN + TL_NUM);

            tdb = eff.sub(dt!(sec_inv_attos));
            i += 1;
        }
        Self::tdb_to_tai(tdb)
    }

    // -------------------------------------------------------------------------
    // LTC ↔ TCL (L_m scaling, parallel to TT ↔ TCG)
    // -------------------------------------------------------------------------

    /// TCL → LTC: `LTC = TCL − L_m · (TCL − t₀)` (t₀ = IAU 1977 epoch).
    ///
    /// Same structure as [`Self::tcg_to_tt`].
    pub(crate) const fn tcl_to_ltc(tcl: Dt) -> Dt {
        let elapsed = tcl.to_attos_since_tcg_tcb_epoch();
        let span_attos = Self::mul_lm(elapsed);
        tcl.add_attos(-span_attos)
    }

    /// LTC → TCL: `TCL = LTC + L_m/(1−L_m) · (LTC − t₀)`.
    ///
    /// Same structure as [`Self::tt_to_tcg`].
    pub(crate) const fn ltc_to_tcl(ltc: Dt) -> Dt {
        let elapsed = ltc.to_attos_since_tcg_tcb_epoch();
        let span_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN - LM_NUM);
        ltc.add_attos(span_attos)
    }

    /// TAI → LTC via TCL.
    #[inline(always)]
    pub(crate) const fn tai_to_ltc(tai: Dt) -> Dt {
        Self::tcl_to_ltc(Self::tai_to_tcl(tai))
    }

    /// LTC → TAI via TCL.
    #[inline(always)]
    pub(crate) const fn ltc_to_tai(ltc: Dt) -> Dt {
        Self::tcl_to_tai(Self::ltc_to_tcl(ltc))
    }
}
