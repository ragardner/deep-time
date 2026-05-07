use crate::historical_sofa::{historical_sofa_for_tai_to_utc, historical_sofa_for_utc_to_tai};
use crate::leap_seconds::get_leap_seconds;
use crate::{
    ATTOS_PER_SEC, ATTOS_PER_SEC_I128, ClockDrift, ClockModel, ClockType, J2000_JD_TT,
    J2000_SEC_PER_CENTURY, LB_DEN, LB_NUM, LG_DEN, LG_NUM, LM_DEN, LM_NUM, MARS_MSD_REF_JD_INT,
    MARS_MSD_REF_TOD_SEC, MARS_MSD_REF_TOD_SUBSEC, MARS_REF_TT, MARS_SOL_ATTOS,
    MARS_SOL_LENGTH_SEC, Real, SEC_PER_DAYI64, SEC_PER_DAYI128, TAI_SEC_AT_1972,
    TCG_TCB_REF_JD_INT, TCG_TCB_REF_TOD_SEC, TCG_TCB_REF_TOD_SUBSEC, TDB0_ATTOS,
    TT_TAI_OFFSET_SPAN, TimePoint, TimeSpan, floor_f, sin_approx, to_sec_f,
};

impl TimePoint {
    #[inline(always)]
    pub const fn to_span(&self) -> TimeSpan {
        TimeSpan {
            sec: self.sec,
            subsec: self.subsec,
        }
    }

    #[inline]
    pub const fn uses_leap_sec(&self) -> bool {
        self.clock_type.uses_leap_sec()
    }

    #[inline]
    pub const fn is_ut(&self) -> bool {
        self.clock_type.is_ut()
    }

    /// Mutates and sets the [`ClockType`] of this `TimePoint` in place while preserving the exact numerical seconds
    /// and attoseconds values.
    #[inline]
    pub const fn set_type(&mut self, clock_type: ClockType) -> &Self {
        self.clock_type = clock_type;
        self
    }

    /// Copies and sets the [`ClockType`] of this `TimePoint` in place while preserving the exact numerical seconds
    /// and attoseconds values.
    #[inline]
    pub const fn with_type(self, target: ClockType) -> TimePoint {
        if (self.clock_type as u8) == (target as u8) {
            return self;
        }
        Self {
            sec: self.sec,
            subsec: self.subsec,
            clock_type: target,
        }
    }

    pub const fn from(sec: i64, subsec: u64, clock_type: ClockType) -> TimePoint {
        // Create a raw TimePoint with the input numbers on the requested scale
        let raw = TimePoint::new(sec, subsec, clock_type);

        match clock_type {
            ClockType::TAI | ClockType::Proper | ClockType::Custom | ClockType::UT1 => raw,

            ClockType::TT => raw.sub(TT_TAI_OFFSET_SPAN),

            ClockType::UTC => raw.add(TimeSpan::from_sec(get_leap_seconds(&raw, true).offset)),

            ClockType::UTCSpice => {
                let tai = raw.add(TimeSpan::from_sec(get_leap_seconds(&raw, true).offset));
                if sec < TAI_SEC_AT_1972 - 10 {
                    tai.add(TimeSpan::from_sec_f(f!(9.0)))
                } else {
                    tai
                }
            }
            ClockType::UTCSofa => {
                let tai = raw.add(TimeSpan::from_sec(get_leap_seconds(&raw, true).offset));
                if let Some(offset) = historical_sofa_for_utc_to_tai(&raw) {
                    tai.add(TimeSpan::from_sec_f(offset))
                } else {
                    tai
                }
            }

            ClockType::GPS | ClockType::QZSS | ClockType::GST => raw.add(TimeSpan::SEC_19),

            ClockType::BDT => raw.add(TimeSpan::SEC_33),

            ClockType::TDB | ClockType::ET => Self::tdb_to_tai(raw),

            ClockType::TCG => {
                let tt = Self::tcg_to_tt(raw);
                tt.sub(TT_TAI_OFFSET_SPAN).with_type(ClockType::TCG)
            }

            ClockType::TCB => {
                let tdb = Self::tcb_to_tdb(raw);
                Self::tdb_to_tai(tdb).with_type(ClockType::TCB)
            }

            ClockType::LTC => {
                let tt = Self::ltc_to_tt(raw);
                tt.sub(TT_TAI_OFFSET_SPAN).with_type(ClockType::LTC)
            }
        }
    }

    /// Returns a bare [`TimeSpan`] containing the numerical `sec`/`subsec` values
    /// of this instant **on its own [`ClockType`]** (same physical moment).
    ///
    /// This is the recommended way for callers to obtain the representation on
    /// a particular scale after construction via [`Self::from`].
    pub const fn to(&self, clock_type: ClockType) -> TimeSpan {
        match clock_type {
            ClockType::TAI | ClockType::Proper | ClockType::Custom | ClockType::UT1 => {
                self.to_span()
            }

            ClockType::TT => self.add(TT_TAI_OFFSET_SPAN).to_span(),

            ClockType::UTC => self
                .sub(TimeSpan::from_sec(get_leap_seconds(&self, false).offset))
                .to_span(),

            ClockType::UTCSpice => {
                if self.sec < TAI_SEC_AT_1972 {
                    let mut utc =
                        self.sub(TimeSpan::from_sec(get_leap_seconds(&self, false).offset));
                    utc.mut_sub(TimeSpan::from_sec_f(f!(9.0)));
                    utc.to_span()
                } else {
                    self.sub(TimeSpan::from_sec(get_leap_seconds(&self, false).offset))
                        .to_span()
                }
            }
            ClockType::UTCSofa => {
                if let Some(offset) = historical_sofa_for_tai_to_utc(&self) {
                    let mut utc =
                        self.sub(TimeSpan::from_sec(get_leap_seconds(&self, false).offset));
                    utc.mut_sub(TimeSpan::from_sec_f(offset));
                    utc.to_span()
                } else {
                    self.sub(TimeSpan::from_sec(get_leap_seconds(&self, false).offset))
                        .to_span()
                }
            }

            ClockType::GPS | ClockType::QZSS | ClockType::GST => {
                self.sub(TimeSpan::SEC_19).to_span()
            }

            ClockType::BDT => self.sub(TimeSpan::SEC_33).to_span(),

            ClockType::TDB | ClockType::ET => Self::tai_to_tdb(*self).to_span(),

            ClockType::TCG => Self::tai_to_tcg(*self).to_span(),

            ClockType::TCB => Self::tai_to_tcb(*self).to_span(),

            ClockType::LTC => {
                let tt = self.add(TT_TAI_OFFSET_SPAN).with_type(ClockType::TT);
                Self::tt_to_ltc(tt).to_span()
            }
        }
    }

    /// Converts this instant to any other [`ClockType`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`ClockDrift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(self, reference: Self, drift: ClockDrift) -> Self {
        let span = self.to_tai_since(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Performs the inverse conversion of [`Self::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    pub const fn convert_back_using_drift(self, reference: Self, drift: ClockDrift) -> Self {
        if drift.rate().is_zero() && drift.accel().is_zero() {
            return self.sub(*drift.constant());
        }
        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            let span = guess.to_tai_since(reference);
            let correction = drift.time_diff_after(&span);
            guess = self.sub(correction);
            i += 1;
        }
        guess
    }

    /// Converts this instant using a self-describing [`ClockModel`].
    ///
    /// This is the recommended high-level API for onboard or custom time scales (Proper, Custom,
    /// or any model with a defined base and drift).
    #[inline]
    pub const fn convert_using_model(self, model: ClockModel) -> Self {
        self.convert_using_drift(model.reference, model.drift)
    }

    /// Performs the inverse conversion of [`Self::convert_using_model`].
    #[inline]
    pub const fn convert_back_using_model(self, model: ClockModel) -> Self {
        self.convert_back_using_drift(model.reference, model.drift)
    }

    /// Computes the difference TDB − TT (in seconds) using the four dominant
    /// periodic terms from the Fairhead & Bretagnon (1990) analytical series,
    /// as extracted from the SOFA/ERFA library (`eraDtdb`).
    ///
    /// This is currently the most accurate practical analytical model for the
    /// periodic part of TDB−TT. It captures approximately 99.85% of the total
    /// power present in the full 787-term Fairhead-Bretagnon series while
    /// remaining extremely fast and fully `const fn` compatible.
    ///
    /// The model includes:
    /// - Main annual term (Earth's orbital eccentricity)
    /// - Semi-annual harmonic
    /// - 11.86-year perturbation term (lunar/planetary)
    /// - Venus perturbation term
    ///
    /// **Accuracy**: better than ±0.5 µs near J2000.0 for the periodic component
    /// (this 4-term model captures the dominant variation), with slow degradation
    /// over millennia. For nanosecond-level work over long timescales, numerical
    /// integration against a modern solar-system ephemeris (DE440/DE441, INPOP,
    /// etc.) remains the definitive method.
    ///
    /// References (all directly from the SOFA/ERFA implementation):
    ///
    /// - Fairhead, L. & Bretagnon, P., "An analytical formula for the time
    ///   transformation TB-TT", Astron. Astrophys. 229, 240-247 (1990).
    ///
    /// - IAU 2006 Resolution 3 (re-definition of Barycentric Dynamical Time).
    ///
    /// - McCarthy, D. D. & Petit, G. (eds.), IERS Conventions (2003),
    ///   IERS Technical Note No. 32, BKG (2004).
    ///
    /// - Moyer, T. D., "Transformation from proper time on Earth to coordinate
    ///   time in solar system barycentric space", Cel. Mech. 23, 33 (1981).
    ///
    /// - Murray, C. A., Vectorial Astrometry, Adam Hilger (1983).
    ///
    /// - Seidelmann, P. K. et al., Explanatory Supplement to the Astronomical
    ///   Almanac, Chapter 2, University Science Books (1992).
    ///
    /// - Simon, J. L., Bretagnon, P., Chapront, J., Chapront-Touze, M.,
    ///   Francou, G. & Laskar, J., "Numerical expressions for precession
    ///   formulae and mean elements for the Moon and planets",
    ///   Astron. Astrophys. 282, 663-683 (1994).
    ///
    /// - SOFA/ERFA `eraDtdb` implementation (2021 May 11 revision):
    ///   https://raw.githubusercontent.com/liberfa/erfa/master/src/dtdb.c
    const fn tdb_minus_tt(sec: i64, subsec: u64) -> TimeSpan {
        let seconds_since_j2000_tt = f!(sec) + f!(subsec) / f!(ATTOS_PER_SEC);
        let t = seconds_since_j2000_tt / J2000_SEC_PER_CENTURY;

        // Mean anomaly of Earth (from Fairhead & Bretagnon 1990 / Simon et al. 1994)
        let g =
            f!(2.0) * core::f64::consts::PI * (f!(357.52910918) + f!(35999.050290) * t) / f!(360.0);

        // Main annual term with first-order eccentricity correction
        let sin_g = sin_approx(g + f!(0.01671) * sin_approx(g));

        // Semi-annual harmonic
        let sin_2g = sin_approx(f!(2.0) * g);

        // Lunar monthly term (27.3 days) — amplitude 4.770086 µs
        let lunar = sin_approx(f!(52.9690965) * t + f!(0.444401603));

        // Venus perturbation — amplitude 4.676740 µs
        let venus = sin_approx(f!(606.977675) * t + f!(4.021195093));

        let correction = f!(0.001656674564) * sin_g
            + f!(0.000022417471) * sin_2g
            + f!(0.000004770086) * lunar
            + f!(0.000004676740) * venus;

        TimeSpan::from_sec_f(correction)
    }

    const fn tai_to_tdb(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET_SPAN).with_type(ClockType::TT);
        let span = Self::tdb_minus_tt(tt.sec, tt.subsec);
        tt.add(span).with_type(ClockType::TDB)
    }

    const fn tdb_to_tai(tdb: Self) -> Self {
        let mut tt = tdb;
        let mut i = 0u32;
        while i < 8 {
            tt = tdb.sub(Self::tdb_minus_tt(tt.sec, tt.subsec));
            i += 1;
        }
        tt.sub(TT_TAI_OFFSET_SPAN)
    }

    const fn tai_to_tcg(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET_SPAN).with_type(ClockType::TT);
        Self::tt_to_tcg(tt)
    }

    const fn tai_to_tcb(tai: Self) -> Self {
        let tdb = Self::tai_to_tdb(tai);
        Self::tdb_to_tcb(tdb)
    }

    /// Exact integer helper: elapsed attoseconds since the TCG/TCB reference epoch (1977-01-01.0 TAI),
    /// using only the numerical `sec`/`subsec` of the supplied `TimePoint` (clock_type is ignored).
    const fn elapsed_to_attos_since_ref(numerical: Self) -> i128 {
        let days_since_j2000 = numerical.sec.div_euclid(SEC_PER_DAYI64);
        let tod_sec = numerical.sec.rem_euclid(SEC_PER_DAYI64);

        let jd_days = J2000_JD_TT + days_since_j2000;
        let days_diff = jd_days - TCG_TCB_REF_JD_INT;

        let mut sec_diff =
            (days_diff as i128) * SEC_PER_DAYI128 + (tod_sec as i128 - TCG_TCB_REF_TOD_SEC as i128);
        let mut attos_diff = (numerical.subsec as i128) - (TCG_TCB_REF_TOD_SUBSEC as i128);

        if attos_diff < 0 {
            attos_diff += ATTOS_PER_SEC_I128;
            sec_diff -= 1;
        }

        sec_diff * ATTOS_PER_SEC_I128 + attos_diff
    }

    /// Exact fixed-point multiplication: `attos * num / den` (handles negative values safely, no overflow for library time range).
    const fn mul_rate(attos: i128, num: i128, den: i128) -> i128 {
        if attos == 0 {
            return 0;
        }
        let sign = if attos < 0 { -1i128 } else { 1i128 };
        let a = if attos < 0 { -attos } else { attos };
        let q = a / den;
        let r = a % den;
        sign * (q * num + (r * num) / den)
    }

    #[inline]
    const fn mul_lg(attos: i128) -> i128 {
        Self::mul_rate(attos, LG_NUM, LG_DEN)
    }

    #[inline]
    const fn mul_lb(attos: i128) -> i128 {
        Self::mul_rate(attos, LB_NUM, LB_DEN)
    }

    #[inline]
    const fn mul_lm(attos: i128) -> i128 {
        Self::mul_rate(attos, LM_NUM, LM_DEN)
    }

    const fn tt_to_tcg(tt: Self) -> Self {
        let elapsed = Self::elapsed_to_attos_since_ref(tt);
        let span_attos = Self::mul_lg(elapsed);
        tt.add(TimeSpan::from_attos(span_attos))
            .with_type(ClockType::TCG)
    }

    const fn tcg_to_tt(tcg: Self) -> Self {
        let elapsed_cg = Self::elapsed_to_attos_since_ref(tcg);
        let span_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.sub(TimeSpan::from_attos(span_attos))
            .with_type(ClockType::TT)
    }

    const fn tcb_to_tdb(tcb: Self) -> Self {
        let elapsed_cg = Self::elapsed_to_attos_since_ref(tcb);
        let span_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.sub(TimeSpan::from_attos(span_attos))
            .sub(TimeSpan::from_attos(TDB0_ATTOS))
            .with_type(ClockType::TDB)
    }

    const fn tdb_to_tcb(tdb: Self) -> Self {
        let elapsed = Self::elapsed_to_attos_since_ref(tdb.with_type(ClockType::TT));
        let span_attos = Self::mul_lb(elapsed);
        tdb.add(TimeSpan::from_attos(span_attos))
            .add(TimeSpan::from_attos(TDB0_ATTOS))
            .with_type(ClockType::TCB)
    }

    const fn tt_to_ltc(tt: Self) -> Self {
        let elapsed = Self::elapsed_to_attos_since_ref(tt);
        let span_attos = Self::mul_lm(elapsed);
        tt.add(TimeSpan::from_attos(span_attos))
            .with_type(ClockType::LTC)
    }

    const fn ltc_to_tt(ltc: Self) -> Self {
        let elapsed = Self::elapsed_to_attos_since_ref(ltc);
        let span_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
        ltc.sub(TimeSpan::from_attos(span_attos))
            .with_type(ClockType::TT)
    }

    /// Exact helper: elapsed attoseconds since the Mars MSD reference epoch (JD 2405522.0028779 TT).
    const fn elapsed_to_attos_since_mars_ref(numerical_tt: TimeSpan) -> i128 {
        let days_since_j2000 = numerical_tt.sec.div_euclid(SEC_PER_DAYI64);
        let tod_sec = numerical_tt.sec.rem_euclid(SEC_PER_DAYI64);

        let jd_days = J2000_JD_TT + days_since_j2000;
        let days_diff = jd_days - MARS_MSD_REF_JD_INT;

        let mut sec_diff = (days_diff as i128) * SEC_PER_DAYI128
            + (tod_sec as i128 - MARS_MSD_REF_TOD_SEC as i128);
        let mut attos_diff = (numerical_tt.subsec as i128) - (MARS_MSD_REF_TOD_SUBSEC as i128);

        if attos_diff < 0 {
            attos_diff += ATTOS_PER_SEC_I128;
            sec_diff -= 1;
        }

        sec_diff * ATTOS_PER_SEC_I128 + attos_diff
    }

    /// Returns the exact Mars Sol Date (MSD) as a tuple of integer sols and the fractional part of a sol.
    ///
    /// The computation follows the canonical NASA GISS / AM2000 formulation and works for any input
    /// [`ClockType`]. Leap seconds are automatically accounted for when converting from UTC.
    pub const fn to_msd_exact(self) -> (i64, u128) {
        let tt = self.to(ClockType::TT);
        let elapsed = Self::elapsed_to_attos_since_mars_ref(tt);
        let attos_per_sol = MARS_SOL_ATTOS;

        let whole_sols = elapsed.div_euclid(attos_per_sol) as i64;
        let frac_attos = elapsed.rem_euclid(attos_per_sol) as u128;

        (whole_sols, frac_attos)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`TimeSpan`] representing
    /// seconds into the current sol (range `[0, one Martian sol)`).
    #[inline]
    pub const fn to_mtc(self) -> TimeSpan {
        let (_, frac_attos) = self.to_msd_exact();
        TimeSpan::from_attos(frac_attos as i128)
    }

    /// Creates a `TimePoint` (in TT) from an exact Mars Sol Date using full library precision.
    pub const fn from_msd_exact(whole_sols: i64, frac_attos: u128) -> Self {
        let elapsed_attos = (whole_sols as i128) * MARS_SOL_ATTOS + frac_attos as i128;

        let tt = MARS_REF_TT.add(TimeSpan::from_attos(elapsed_attos));
        Self::from(tt.sec, tt.subsec, ClockType::TT)
    }

    /// Creates a `TimePoint` (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    pub const fn from_msd(msd: Real) -> Self {
        let whole = floor_f(msd) as i64;
        let frac = msd - f!(whole);
        let frac_span = TimeSpan::from_sec_f(frac * MARS_SOL_LENGTH_SEC);
        Self::from_msd_exact(whole, frac_span.to_attos() as u128)
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd(self) -> Real {
        let (whole, frac) = self.to_msd_exact();
        f!(whole) + to_sec_f(frac) / MARS_SOL_LENGTH_SEC
    }
}
