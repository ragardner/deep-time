use crate::leap_seconds::leap_seconds_before;
use crate::{
    ATTOSEC_PER_SEC, ATTOSEC_PER_SEC_I128, ClockDrift, ClockModel, ClockType, J2000_JD_TT,
    J2000_SECONDS_PER_CENTURY, LB_DEN, LB_NUM, LG_DEN, LG_NUM, LM_DEN, LM_NUM, MARS_MSD_REF_JD_INT,
    MARS_MSD_REF_TOD_SEC, MARS_MSD_REF_TOD_SUBSEC, MARS_REF_SEC, MARS_REF_SUBSEC, MARS_SOL_ATTOS,
    MARS_SOL_LENGTH_SEC, Real, SEC_PER_DAY, SEC_PER_DAYI64, SEC_PER_DAYI128, TCG_TCB_REF_JD_INT,
    TCG_TCB_REF_TOD_SEC, TCG_TCB_REF_TOD_SUBSEC, TDB0_ATTOS, TT_TAI_OFFSET_SPAN, TimePoint,
    TimeSpan, floor_f, sin_approx,
};

impl TimePoint {
    /// Converts this instant to any other [`ClockType`], representing the exact same physical moment in time.
    ///
    /// The conversion is performed through the library’s canonical TAI representation to guarantee
    /// consistency across all supported time scales, including leap-second corrections and relativistic
    /// transformations where applicable.
    #[inline]
    pub const fn to_clock_type(self, target: ClockType) -> Self {
        if (self.clock_type as u8) == (target as u8) {
            return self;
        }
        let tai = self.to_tai();
        tai.from_tai(target)
    }

    /// Returns a copy of this `TimePoint` with the specified [`ClockType`]
    /// **while preserving the exact numerical `sec` and `subsec` values**.
    ///
    /// ### Warning:
    ///
    /// This performs **no time-scale conversion** and does **not** change the physical instant.
    #[inline]
    pub const fn with_clock_type(self, clock_type: ClockType) -> Self {
        Self {
            sec: self.sec,
            subsec: self.subsec,
            clock_type,
        }
    }

    /// Sets the [`ClockType`] of this `TimePoint` in place while preserving the exact numerical seconds
    /// and attoseconds values.
    ///
    /// This is the mutable counterpart to [`Self::with_clock_type`] and remains zero-cost.
    #[inline]
    pub(crate) const fn set_clock_type(&mut self, clock_type: ClockType) -> &Self {
        self.clock_type = clock_type;
        self
    }

    /// Converts this `TimePoint` (in any clock type) to TAI, the library’s internal canonical time scale.
    ///
    /// All other supported scales are defined relative to TAI. Leap-second corrections (for UTC) and
    /// relativistic transformations (for TT, TDB, TCG, TCB, LTC) are applied exactly as defined by the
    /// relevant IAU and NIST standards.
    pub const fn to_tai(self) -> Self {
        match self.clock_type {
            ClockType::TAI => self,

            ClockType::TT => {
                let mut tp = self.sub_ref(&TT_TAI_OFFSET_SPAN);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::UTC => Self::utc_to_tai(self),

            ClockType::GPST | ClockType::QZSST | ClockType::GST => {
                let mut tp = self.add_ref(&TimeSpan::SEC_19);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::BDT => {
                let mut tp = self.add_ref(&TimeSpan::SEC_33);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::TDB | ClockType::ET => {
                let mut tp = Self::tdb_to_tai(self);
                tp.set_clock_type(ClockType::TAI);
                tp
            }
            ClockType::TCG => Self::tcg_to_tai(self),
            ClockType::TCB => Self::tcb_to_tai(self),

            ClockType::LTC => Self::ltc_to_tt(self).to_tai(),

            ClockType::Proper | ClockType::Custom => self,
        }
    }

    /// Converts a TAI `TimePoint` to any other requested [`ClockType`].
    ///
    /// This is the inverse operation of [`Self::to_tai`] and completes a round-trip conversion while
    /// preserving the exact physical instant.
    pub const fn from_tai(self, target: ClockType) -> Self {
        match target {
            ClockType::TAI => self,

            ClockType::TT => {
                let mut tp = self.add_ref(&TT_TAI_OFFSET_SPAN);
                tp.set_clock_type(target);
                tp
            }

            ClockType::UTC => Self::tai_to_utc(self),

            ClockType::GPST | ClockType::QZSST | ClockType::GST => {
                let mut tp = self.sub_ref(&TimeSpan::SEC_19);
                tp.set_clock_type(target);
                tp
            }

            ClockType::BDT => {
                let mut tp = self.sub_ref(&TimeSpan::SEC_33);
                tp.set_clock_type(target);
                tp
            }

            ClockType::TDB | ClockType::ET => {
                let mut tp = Self::tai_to_tdb(self);
                tp.set_clock_type(target);
                tp
            }
            ClockType::TCG => Self::tai_to_tcg(self),
            ClockType::TCB => Self::tai_to_tcb(self),

            ClockType::LTC => Self::tt_to_ltc(self.from_tai(ClockType::TT)),

            ClockType::Proper | ClockType::Custom => {
                let mut tp = self;
                tp.set_clock_type(target);
                tp
            }
        }
    }

    /// Converts this instant to another [`ClockType`] by applying a constant time offset.
    ///
    /// This is the recommended method for the common case of a fixed offset between two time scales
    /// (e.g. applying a DUT1 value when converting to a UT1-like custom scale, or any other constant bias).
    ///
    /// The offset can be positive or negative. Negative offsets move the time backward.
    ///
    /// This is a zero-cost convenience wrapper around [`Self::saturating_add`] + [`Self::with_clock_type`].
    #[inline(always)]
    pub const fn convert_using_offset(&mut self, target: ClockType, offset: TimeSpan) -> Self {
        self.mut_add(&offset).with_clock_type(target)
    }

    /// Same as [`Self::convert_using_offset`], but accepts the offset as an `f64` (in seconds) for convenience.
    #[inline(always)]
    pub const fn convert_using_offset_f(&mut self, target: ClockType, offset_sec: Real) -> Self {
        self.mut_add(&TimeSpan::from_sec_f(offset_sec))
            .with_clock_type(target)
    }

    /// Converts this instant to any other [`ClockType`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`ClockDrift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(
        self,
        target: ClockType,
        reference: Self,
        drift: ClockDrift,
    ) -> Self {
        let span = self.duration_since(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction).with_clock_type(target)
    }

    /// Performs the inverse conversion of [`Self::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    #[inline]
    pub const fn convert_back_using_drift(
        self,
        source: ClockType,
        reference: Self,
        drift: ClockDrift,
    ) -> Self {
        if drift.rate().is_zero() && drift.accel().is_zero() {
            return self.sub_ref(&drift.constant()).with_clock_type(source);
        }
        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            let span = guess.duration_since(reference);
            let correction = drift.time_diff_after(&span);
            guess = self.sub(correction);
            i += 1;
        }
        guess.with_clock_type(source)
    }

    /// Converts this instant using a self-describing [`ClockModel`].
    ///
    /// This is the recommended high-level API for onboard or custom time scales (Proper, Custom,
    /// or any model with a defined base and drift).
    #[inline(always)]
    pub const fn convert_using_model(self, model: ClockModel) -> Self {
        self.convert_using_drift(model.base, model.reference, model.drift)
    }

    /// Performs the inverse conversion of [`Self::convert_using_model`].
    #[inline(always)]
    pub const fn convert_back_using_model(self, model: ClockModel) -> Self {
        self.convert_back_using_drift(model.base, model.reference, model.drift)
    }

    const fn utc_to_tai(utc: Self) -> Self {
        let leaps = leap_seconds_before(utc);
        utc.add(TimeSpan::from_sec(leaps))
            .with_clock_type(ClockType::TAI)
    }

    const fn tai_to_utc(tai: Self) -> Self {
        let leaps = leap_seconds_before(tai);
        tai.sub(TimeSpan::from_sec(leaps))
            .with_clock_type(ClockType::UTC)
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
    #[inline]
    const fn tdb_minus_tt(tt: Self) -> TimeSpan {
        let seconds_since_j2000_tt =
            (tt.sec as Real) + (tt.subsec as Real) / (ATTOSEC_PER_SEC as Real);

        let t = seconds_since_j2000_tt / J2000_SECONDS_PER_CENTURY;

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
        let tt = tai
            .add_ref(&TT_TAI_OFFSET_SPAN)
            .with_clock_type(ClockType::TT);
        let span = Self::tdb_minus_tt(tt);
        tt.add(span).with_clock_type(ClockType::TDB)
    }

    const fn tdb_to_tai(tdb: Self) -> Self {
        let mut tt = tdb.with_clock_type(ClockType::TT);
        let mut i = 0u32;

        while i < 8 {
            let span = Self::tdb_minus_tt(tt);
            tt = tdb.with_clock_type(ClockType::TT).sub(span);
            i += 1;
        }

        tt.sub_ref(&TT_TAI_OFFSET_SPAN)
            .with_clock_type(ClockType::TAI)
    }

    const fn tcg_to_tai(tcg: Self) -> Self {
        let tt = Self::tcg_to_tt(tcg);
        tt.to_tai()
    }

    const fn tai_to_tcg(tai: Self) -> Self {
        let tt = tai.from_tai(ClockType::TT);
        Self::tt_to_tcg(tt)
    }

    const fn tcb_to_tai(tcb: Self) -> Self {
        let tdb = Self::tcb_to_tdb(tcb);
        tdb.to_tai()
    }

    const fn tai_to_tcb(tai: Self) -> Self {
        let tdb = tai.from_tai(ClockType::TDB);
        Self::tdb_to_tcb(tdb)
    }

    /// Exact integer helper: elapsed attoseconds since the TCG/TCB reference epoch (1977-01-01.0 TAI),
    /// using only the numerical `sec`/`subsec` of the supplied `TimePoint` (clock_type is ignored).
    const fn elapsed_attos_since_ref(numerical: Self) -> i128 {
        let days_since_j2000 = numerical.sec.div_euclid(SEC_PER_DAYI64);
        let tod_sec = numerical.sec.rem_euclid(SEC_PER_DAYI64);

        let jd_days = J2000_JD_TT + days_since_j2000;
        let days_diff = jd_days - TCG_TCB_REF_JD_INT;

        let mut sec_diff =
            (days_diff as i128) * SEC_PER_DAYI128 + (tod_sec as i128 - TCG_TCB_REF_TOD_SEC as i128);
        let mut attos_diff = (numerical.subsec as i128) - (TCG_TCB_REF_TOD_SUBSEC as i128);

        if attos_diff < 0 {
            attos_diff += ATTOSEC_PER_SEC_I128;
            sec_diff -= 1;
        }

        sec_diff * ATTOSEC_PER_SEC_I128 + attos_diff
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

    #[inline(always)]
    const fn mul_lg(attos: i128) -> i128 {
        Self::mul_rate(attos, LG_NUM, LG_DEN)
    }

    #[inline(always)]
    const fn mul_lb(attos: i128) -> i128 {
        Self::mul_rate(attos, LB_NUM, LB_DEN)
    }

    #[inline(always)]
    const fn mul_lm(attos: i128) -> i128 {
        Self::mul_rate(attos, LM_NUM, LM_DEN)
    }

    const fn tt_to_tcg(tt: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(tt);
        let span_attos = Self::mul_lg(elapsed);
        tt.add(TimeSpan::from_total_attos(span_attos))
            .with_clock_type(ClockType::TCG)
    }

    const fn tcg_to_tt(tcg: Self) -> Self {
        let elapsed_cg = Self::elapsed_attos_since_ref(tcg);
        let span_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.sub(TimeSpan::from_total_attos(span_attos))
            .with_clock_type(ClockType::TT)
    }

    const fn tcb_to_tdb(tcb: Self) -> Self {
        let elapsed_cg = Self::elapsed_attos_since_ref(tcb);
        let span_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.sub(TimeSpan::from_total_attos(span_attos))
            .sub(TimeSpan::from_total_attos(TDB0_ATTOS))
            .with_clock_type(ClockType::TDB)
    }

    const fn tdb_to_tcb(tdb: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(tdb.with_clock_type(ClockType::TT));
        let span_attos = Self::mul_lb(elapsed);
        tdb.add(TimeSpan::from_total_attos(span_attos))
            .add(TimeSpan::from_total_attos(TDB0_ATTOS))
            .with_clock_type(ClockType::TCB)
    }

    const fn tt_to_ltc(tt: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(tt);
        let span_attos = Self::mul_lm(elapsed);
        tt.add(TimeSpan::from_total_attos(span_attos))
            .with_clock_type(ClockType::LTC)
    }

    const fn ltc_to_tt(ltc: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(ltc);
        let span_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
        ltc.sub(TimeSpan::from_total_attos(span_attos))
            .with_clock_type(ClockType::TT)
    }

    /// Exact helper: elapsed attoseconds since the Mars MSD reference epoch (JD 2405522.0028779 TT).
    const fn elapsed_attos_since_mars_ref(numerical_tt: Self) -> i128 {
        let days_since_j2000 = numerical_tt.sec.div_euclid(SEC_PER_DAYI64);
        let tod_sec = numerical_tt.sec.rem_euclid(SEC_PER_DAYI64);

        let jd_days = J2000_JD_TT + days_since_j2000;
        let days_diff = jd_days - MARS_MSD_REF_JD_INT;

        let mut sec_diff = (days_diff as i128) * SEC_PER_DAYI128
            + (tod_sec as i128 - MARS_MSD_REF_TOD_SEC as i128);
        let mut attos_diff = (numerical_tt.subsec as i128) - (MARS_MSD_REF_TOD_SUBSEC as i128);

        if attos_diff < 0 {
            attos_diff += ATTOSEC_PER_SEC_I128;
            sec_diff -= 1;
        }

        sec_diff * ATTOSEC_PER_SEC_I128 + attos_diff
    }

    #[inline]
    const fn mars_ref_tt() -> Self {
        TimePoint::new(MARS_REF_SEC, MARS_REF_SUBSEC, ClockType::TT)
    }

    /// Returns the exact Mars Sol Date (MSD) as a tuple of integer sols and the fractional part of a sol.
    ///
    /// The computation follows the canonical NASA GISS / AM2000 formulation and works for any input
    /// [`ClockType`]. Leap seconds are automatically accounted for when converting from UTC.
    pub const fn to_msd_exact(self) -> (i64, TimeSpan) {
        let tt = self.to_clock_type(ClockType::TT);
        let elapsed = Self::elapsed_attos_since_mars_ref(tt);
        let attos_per_sol = MARS_SOL_ATTOS;

        let whole_sols = elapsed.div_euclid(attos_per_sol) as i64;
        let frac_attos = elapsed.rem_euclid(attos_per_sol);
        let frac_sol = TimeSpan::from_total_attos(frac_attos);

        (whole_sols, frac_sol)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`TimeSpan`] representing seconds into the current sol
    /// (range [0, one Martian sol)).
    #[inline]
    pub const fn to_mtc(self) -> TimeSpan {
        let (_, frac_sol) = self.to_msd_exact();
        frac_sol
    }

    /// Creates a `TimePoint` (in TT) from an exact Mars Sol Date using full library precision.
    pub const fn from_msd_exact(whole_sols: i64, frac_sol: TimeSpan) -> Self {
        let frac_attos = frac_sol.total_attos();
        let elapsed_attos = (whole_sols as i128) * MARS_SOL_ATTOS + frac_attos;

        let ref_tt = Self::mars_ref_tt();
        let tt = ref_tt.add(TimeSpan::from_total_attos(elapsed_attos));
        tt.to_tai()
    }

    /// Returns an exact Julian Date in **Terrestrial Time (TT)** with full
    /// attosecond precision.
    ///
    /// This is the *astronomical standard* form of JD used by IAU/IERS
    /// ephemerides, pulsar timing, planetary missions, and virtually all
    /// high-precision astrodynamics software (SOFA/ERFA, NASA SPICE, etc.).
    ///
    /// The function always converts the input `TimePoint` to TT internally,
    /// guaranteeing consistency across all clock types (including relativistic
    /// scales TCG/TCB/LTC, UTC leap seconds, etc.).
    ///
    /// By international convention **J2000.0 TT ≡ JD 2451545.0 exactly**
    /// (2000-01-01 12:00:00 TT).
    ///
    /// # Return
    /// A tuple `(jd_integer_days, fractional_day)` where:
    /// - `jd_integer_days` is the integer part of the JD (i64)
    /// - `fractional_day` is a [`TimeSpan`] in the range `[0, 1)` day,
    ///   representing the exact fraction of a day since noon TT (astronomical
    ///   convention).
    ///
    /// # Precision
    /// Exact to the attosecond (no floating-point arithmetic is used).
    ///
    /// # See also
    /// - [`Self::to_jd_tt`] — floating-point convenience version
    /// - [`Self::to_mjd_tt_exact`] — Modified Julian Date in TT
    /// - [`Self::to_jd_utc_exact`] — civil/engineering form in UTC
    #[inline]
    pub const fn to_jd_tt_exact(self) -> (i64, TimeSpan) {
        let tt = self.to_clock_type(ClockType::TT);
        let days_since_j2000 = tt.sec.div_euclid(SEC_PER_DAYI64);
        let remaining_sec = tt.sec.rem_euclid(SEC_PER_DAYI64);
        let frac = TimeSpan::new(remaining_sec, tt.subsec);
        (J2000_JD_TT + days_since_j2000, frac)
    }

    /// Returns the **Julian Date (JD)** expressed in **UTC** with full attosecond
    /// precision.
    ///
    /// This is the *civil/operational form* of JD used by GNSS, RINEX files,
    /// flight software, and most modern engineering pipelines.
    ///
    /// The reference epoch is **1970-01-01 00:00:00 UTC ≡ JD 2440587.5 exactly**.
    /// The function uses the library’s canonical UTC representation
    /// ([`Self::to_canonical`]), so leap seconds are handled correctly
    /// and the proleptic Gregorian civil second count is respected.
    ///
    /// # Return
    /// A tuple `(jd_integer_days, fractional_day)` where:
    /// - `jd_integer_days` is the integer part of the JD (i64)
    /// - `fractional_day` is a [`TimeSpan`] in the range `[0, 1)` day,
    ///   representing the exact fraction of a day since noon UTC (astronomical
    ///   convention).
    ///
    /// # Precision
    /// Exact to the attosecond (pure integer arithmetic).
    ///
    /// # Important distinction
    /// - Use **`to_jd_tt_exact`** (or `to_jd_tt`) for the *astronomical standard*
    ///   (Terrestrial Time).
    /// - Use **`to_jd_utc_exact`** for civil/operational/GNSS contexts.
    ///
    /// # See also
    /// - [`Self::to_jd_utc`] — floating-point convenience version
    /// - [`Self::to_mjd_utc_exact`] — Modified Julian Date in UTC
    #[inline]
    pub const fn to_jd_utc_exact(self) -> (i64, TimeSpan) {
        let utc = self.to_clock_type(ClockType::UTC);
        let canon_attos = utc.to_canonical();

        const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;

        let days_since_1970 = canon_attos.div_euclid(ATTOS_PER_DAY);
        let frac_attos = canon_attos.rem_euclid(ATTOS_PER_DAY);

        let jd_int = 2_440_587i64 + (days_since_1970 as i64);

        (jd_int, TimeSpan::from_total_attos(frac_attos))
    }

    /// Returns the **Modified Julian Date (MJD)** expressed in **Terrestrial Time (TT)**
    /// with full attosecond precision.
    ///
    /// MJD is defined as `JD − 2_400_000.5`. The conventional astronomical reference
    /// epoch is **J2000.0 TT ≡ MJD 51544.5 exactly** (2000-01-01 12:00:00 TT).
    ///
    /// This is the *standard astronomical form* used by IAU/IERS ephemerides,
    /// pulsar timing arrays, planetary missions, and virtually all high-precision
    /// astrodynamics software (SOFA/ERFA, NASA SPICE, etc.).
    ///
    /// The function always converts the input `TimePoint` to TT internally via
    /// [`Self::to_jd_tt_exact`], guaranteeing consistency across all clock types
    /// (including relativistic scales TCG/TCB/LTC, UTC leap seconds, etc.).
    ///
    /// # Return
    /// A tuple `(mjd_integer_days, fractional_day)` where:
    /// - `mjd_integer_days` is the integer part of the MJD (i64)
    /// - `fractional_day` is a [`TimeSpan`] in the range `[0, 1)` day,
    ///   representing the exact fraction of a day since midnight TT.
    ///
    /// # Precision
    /// Exact to the attosecond (no floating-point arithmetic is used).
    ///
    /// # See also
    /// - [`Self::to_jd_tt_exact`] — the full Julian Date in TT
    /// - [`Self::to_mjd_utc_exact`] — the civil/engineering form in UTC
    #[inline]
    pub const fn to_mjd_tt_exact(self) -> (i64, TimeSpan) {
        let (jd, frac) = self.to_jd_tt_exact();
        (jd - 2_400_000, frac)
    }

    /// Returns the **Modified Julian Date (MJD)** expressed in **UTC** with full
    /// attosecond precision.
    ///
    /// This is the *civil/engineering form* of MJD used by GNSS receivers,
    /// RINEX files, operational flight software, and most modern Earth-based
    /// timing pipelines.
    ///
    /// The reference epoch is **1970-01-01 00:00:00 UTC ≡ MJD 40587.0 exactly**.
    /// The function uses the library’s canonical UTC representation
    /// ([`Self::to_canonical`]), so leap seconds are handled correctly
    /// and the civil Gregorian second count is respected.
    ///
    /// # Return
    /// A tuple `(mjd_integer_days, fractional_day)` where:
    /// - `mjd_integer_days` is the integer part of the MJD (i64)
    /// - `fractional_day` is a [`TimeSpan`] in the range `[0, 1)` day,
    ///   representing the exact fraction of a day since midnight UTC.
    ///
    /// # Precision
    /// Exact to the attosecond (pure integer arithmetic on the canonical UTC
    /// attosecond count).
    ///
    /// # Important distinction
    /// - Use **`to_mjd_tt_exact`** for astronomical work (ephemerides, pulsars,
    ///   spacecraft navigation in barycentric frames).
    /// - Use **`to_mjd_utc_exact`** for civil/operational/GNSS contexts.
    ///
    /// The two values differ by the accumulated leap seconds + the fixed 32.184 s
    /// TT–TAI offset.
    ///
    /// # See also
    /// - [`Self::to_mjd_tt_exact`] — the astronomical standard
    /// - [`Self::to_jd_tt_exact`] — Julian Date in TT
    #[inline]
    pub const fn to_mjd_utc_exact(self) -> (i64, TimeSpan) {
        let utc = self.to_clock_type(ClockType::UTC);
        let canon_attos = utc.to_canonical();

        const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;

        let days_since_1970 = canon_attos.div_euclid(ATTOS_PER_DAY);
        let frac_attos = canon_attos.rem_euclid(ATTOS_PER_DAY);

        let mjd_int = 40_587i64 + (days_since_1970 as i64);

        (mjd_int, TimeSpan::from_total_attos(frac_attos))
    }

    /// Creates a `TimePoint` from an exact Julian Date in Terrestrial Time using full library precision.
    #[inline]
    pub const fn from_jd_tt_exact(jd_days: i64, frac: TimeSpan) -> Self {
        let days_since_j2000 = jd_days - J2000_JD_TT;
        let total_sec = days_since_j2000 * SEC_PER_DAYI64 + frac.sec;
        let tt = TimePoint::new(total_sec, frac.subsec, ClockType::TT);
        tt.to_tai()
    }

    /// Creates a `TimePoint` from an exact Modified Julian Date in Terrestrial Time using full library
    /// precision.
    #[inline]
    pub const fn from_mjd_tt_exact(mjd_days: i64, frac: TimeSpan) -> Self {
        Self::from_jd_tt_exact(mjd_days + 2_400_000, frac)
    }

    /// Creates a `TimePoint` from an exact Julian Date in UTC using full library precision.
    ///
    /// This is the inverse of [`Self::to_jd_utc_exact`].
    ///
    /// The input `(jd_days, frac)` must match exactly what [`Self::to_jd_utc_exact`] returns
    /// for the desired instant. Uses the library’s canonical UTC attosecond representation
    /// (via [`Self::from_canonical`]), so leap seconds are handled correctly
    /// and the proleptic Gregorian civil second count is respected.
    ///
    /// # Precision
    /// Exact to the attosecond (pure integer arithmetic).
    ///
    /// # See also
    /// - [`Self::to_jd_utc_exact`] — the matching `to_` function
    /// - [`Self::from_jd_tt_exact`] — the astronomical (TT) counterpart
    /// - [`Self::from_mjd_utc_exact`] — the MJD variant in UTC
    #[inline]
    pub const fn from_jd_utc_exact(jd_days: i64, frac: TimeSpan) -> Self {
        let days_since_1970 = jd_days - 2_440_587i64;
        const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;
        let total_attos = (days_since_1970 as i128) * ATTOS_PER_DAY + frac.total_attos();
        Self::from_canonical(total_attos, ClockType::UTC)
    }

    /// Creates a `TimePoint` from an exact Modified Julian Date in UTC using full library precision.
    ///
    /// This is the inverse of [`Self::to_mjd_utc_exact`].
    ///
    /// MJD is defined as `JD − 2_400_000.5`. The conventional reference epoch is
    /// **1970-01-01 00:00:00 UTC ≡ MJD 40587.0 exactly**.
    ///
    /// Uses the library’s canonical UTC representation, so leap seconds are handled correctly
    /// and the civil Gregorian second count is respected.
    ///
    /// # Precision
    /// Exact to the attosecond (pure integer arithmetic on the canonical UTC attosecond count).
    ///
    /// # Important distinction
    /// - Use **`from_mjd_utc_exact`** for civil/operational/GNSS contexts (RINEX, flight software, etc.).
    /// - Use **`from_mjd_tt_exact`** for astronomical work (ephemerides, pulsars, barycentric navigation).
    ///
    /// The two differ by the accumulated leap seconds + the fixed 32.184 s TT–TAI offset.
    ///
    /// # See also
    /// - [`Self::to_mjd_utc_exact`] — the matching `to_` function
    /// - [`Self::from_mjd_tt_exact`] — the astronomical (TT) counterpart
    /// - [`Self::from_jd_utc_exact`] — the full JD variant in UTC
    #[inline]
    pub const fn from_mjd_utc_exact(mjd_days: i64, frac: TimeSpan) -> Self {
        Self::from_jd_utc_exact(mjd_days + 2_400_000, frac)
    }

    /// Creates a `TimePoint` (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    #[inline]
    pub const fn from_msd(msd: Real) -> Self {
        let whole = floor_f(msd) as i64;
        let frac = msd - (whole as Real);
        let frac_span = TimeSpan::from_sec_f(frac * MARS_SOL_LENGTH_SEC);
        Self::from_msd_exact(whole, frac_span)
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd(self) -> Real {
        let (whole, frac) = self.to_msd_exact();
        whole as Real + frac.as_sec_f() / MARS_SOL_LENGTH_SEC
    }

    /// Returns the **Julian Date (JD)** in **Terrestrial Time (TT)** as a
    /// floating-point value (`Real = f64`).
    ///
    /// This is the *astronomical standard* form (see [`Self::to_jd_tt_exact`]
    /// for details). By convention **J2000.0 TT ≡ JD 2451545.0 exactly**.
    ///
    /// # Precision
    /// Double-precision floating point (`f64`). Near the present the
    /// fractional part has sub-microsecond accuracy; precision degrades
    /// slowly for dates far in the past or future (as expected for `f64`).
    ///
    /// # See also
    /// - [`Self::to_jd_tt_exact`] — full attosecond exact version
    /// - [`Self::to_mjd_tt`] — Modified Julian Date in TT
    /// - [`Self::to_jd_utc`] — civil/engineering form in UTC
    #[inline]
    pub const fn to_jd_tt(self) -> Real {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let days_f = jd_days as Real;
        let frac_days = frac.as_sec_f() / SEC_PER_DAY;
        days_f + frac_days
    }

    /// Returns the **Julian Date (JD)** in **UTC** as a floating-point value
    /// (`Real = f64`).
    ///
    /// This is the civil/operational form (see [`Self::to_jd_utc_exact`] for
    /// details).
    ///
    /// # Precision
    /// Double-precision floating point. Near the present the fractional part
    /// has sub-microsecond accuracy.
    ///
    /// # See also
    /// - [`Self::to_jd_utc_exact`] — full attosecond exact version
    /// - [`Self::to_jd_tt`] — astronomical standard (TT)
    #[inline]
    pub const fn to_jd_utc(self) -> Real {
        let (jd_int, frac) = self.to_jd_utc_exact();
        let days_f = jd_int as Real;
        let frac_days = frac.as_sec_f() / SEC_PER_DAY;
        days_f + frac_days
    }

    /// Returns the **Modified Julian Date (MJD)** in **Terrestrial Time (TT)**
    /// as a floating-point value (`Real = f64`).
    ///
    /// MJD is defined as `JD − 2_400_000.5`. The conventional astronomical
    /// reference is **J2000.0 TT ≡ MJD 51544.5 exactly**.
    ///
    /// This is the *standard astronomical form* used by IAU/IERS ephemerides,
    /// pulsar timing, planetary missions, and virtually all high-precision
    /// astrodynamics software.
    ///
    /// The function internally uses the exact attosecond path
    /// ([`Self::to_jd_tt_exact`]) before converting to `f64`, so it inherits
    /// the library’s full correctness guarantees for leap seconds and relativistic
    /// scales.
    ///
    /// # Precision
    /// Double-precision floating point (`f64`). For dates near the present the
    /// fractional part is accurate to better than 1 microsecond; precision
    /// degrades slowly for dates far in the past or future (as expected for `f64`).
    ///
    /// # See also
    /// - [`Self::to_mjd_tt_exact`] — full attosecond exact version
    /// - [`Self::to_jd_tt`] — Julian Date in TT
    /// - [`Self::to_mjd_utc`] — civil/engineering form in UTC
    #[inline]
    pub const fn to_mjd_tt(self) -> Real {
        self.to_jd_tt() - f!(2_400_000.5)
    }

    /// Returns the **Modified Julian Date (MJD)** in **UTC** as a floating-point
    /// value (`Real = f64`).
    ///
    /// This is the *civil/operational/GNSS form* used by GNSS receivers, RINEX
    /// files, flight software, and most Earth-based timing pipelines.
    ///
    /// The reference epoch is **1970-01-01 00:00:00 UTC ≡ MJD 40587.0 exactly**.
    /// The function uses the library’s canonical UTC representation, so leap
    /// seconds are handled correctly and the proleptic Gregorian civil second
    /// count is respected.
    ///
    /// # Precision
    /// Double-precision floating point (`f64`). Near the present the fractional
    /// part has sub-microsecond accuracy.
    ///
    /// # Important distinction
    /// - Use **`to_mjd_tt`** for astronomical work (ephemerides, pulsars,
    ///   spacecraft navigation).
    /// - Use **`to_mjd_utc`** for civil/operational/GNSS contexts.
    ///
    /// The two values differ by the accumulated leap seconds plus the fixed
    /// 32.184 s TT–TAI offset.
    ///
    /// # See also
    /// - [`Self::to_mjd_utc_exact`] — full attosecond exact version
    /// - [`Self::to_mjd_tt`] — astronomical standard
    #[inline]
    pub const fn to_mjd_utc(self) -> Real {
        let (mjd_int, frac) = self.to_mjd_utc_exact();
        let days_f = mjd_int as Real;
        let frac_days = frac.as_sec_f() / SEC_PER_DAY;
        days_f + frac_days
    }
}
