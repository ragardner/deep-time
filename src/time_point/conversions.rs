use crate::leap_seconds::leap_seconds_before;
use crate::{
    ATTOSEC_PER_SEC, ATTOSEC_PER_SEC_I128, ClockDrift, ClockModel, ClockType, Delta, J2000_JD_TT,
    J2000_SECONDS_PER_CENTURY, LB_DEN, LB_NUM, LG_DEN, LG_NUM, LM_DEN, LM_NUM, MARS_MSD_REF_JD_INT,
    MARS_MSD_REF_TOD_SEC, MARS_MSD_REF_TOD_SUBSEC, MARS_REF_SEC, MARS_REF_SUBSEC, MARS_SOL_ATTOS,
    MARS_SOL_LENGTH_SEC, Real, SEC_PER_DAY, SEC_PER_DAYI64, SEC_PER_DAYI128, TCG_TCB_REF_JD_INT,
    TCG_TCB_REF_TOD_SEC, TCG_TCB_REF_TOD_SUBSEC, TDB0_ATTOS, TT_TAI_OFFSET_DELTA, TimePoint,
    floor_f, sin_approx,
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

    /// Returns a copy of this `TimePoint` with the specified [`ClockType`] while preserving the exact
    /// numerical seconds and attoseconds values.
    ///
    /// This operation is zero-cost and is primarily intended for internal use after a conversion has
    /// already been performed.
    #[inline]
    pub(crate) const fn with_clock_type(self, clock_type: ClockType) -> Self {
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
    pub const fn set_clock_type(&mut self, clock_type: ClockType) -> &Self {
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

            ClockType::TT | ClockType::ET => {
                let mut tp = self.sub_ref(&TT_TAI_OFFSET_DELTA);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::UTC => Self::utc_to_tai(self),

            ClockType::GPST | ClockType::QZSST | ClockType::GST => {
                let mut tp = self.add_ref(&Delta::SEC_19);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::BDT => {
                let mut tp = self.add_ref(&Delta::SEC_33);
                tp.set_clock_type(ClockType::TAI);
                tp
            }

            ClockType::TDB => Self::tdb_to_tai(self),
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

            ClockType::TT | ClockType::ET => {
                let mut tp = self.add_ref(&TT_TAI_OFFSET_DELTA);
                tp.set_clock_type(target);
                tp
            }

            ClockType::UTC => Self::tai_to_utc(self),

            ClockType::GPST | ClockType::QZSST | ClockType::GST => {
                let mut tp = self.sub_ref(&Delta::SEC_19);
                tp.set_clock_type(target);
                tp
            }

            ClockType::BDT => {
                let mut tp = self.sub_ref(&Delta::SEC_33);
                tp.set_clock_type(target);
                tp
            }

            ClockType::TDB => Self::tai_to_tdb(self),
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

    /// Converts this instant to any other [`ClockType`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`ClockDrift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(
        self,
        target: ClockType,
        reference: Self,
        drift: ClockDrift,
    ) -> Self {
        let delta = self.duration_since(reference);
        let correction = drift.time_diff_after(&delta);
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
            let delta = guess.duration_since(reference);
            let correction = drift.time_diff_after(&delta);
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
        utc.add(Delta::from_sec(leaps))
            .with_clock_type(ClockType::TAI)
    }

    const fn tai_to_utc(tai: Self) -> Self {
        let leaps = leap_seconds_before(tai);
        tai.sub(Delta::from_sec(leaps))
            .with_clock_type(ClockType::UTC)
    }

    const fn tdb_minus_tt(tt: Self) -> Delta {
        let seconds_since_j2000_tt =
            (tt.sec as Real) + (tt.subsec as Real) / (ATTOSEC_PER_SEC as Real);

        let t = seconds_since_j2000_tt / J2000_SECONDS_PER_CENTURY;

        let g = f!(2.0) * core::f64::consts::PI * (f!(357.528) + f!(35_999.050) * t) / f!(360.0);
        let sin_g = sin_approx(g + f!(0.0167) * sin_approx(g));
        let sin_2g = sin_approx(f!(2.0) * g);
        let correction = f!(0.001658) * sin_g + f!(0.000022) * sin_2g;

        Delta::from_sec_f(correction)
    }

    const fn tai_to_tdb(tai: Self) -> Self {
        let tt = tai
            .add_ref(&TT_TAI_OFFSET_DELTA)
            .with_clock_type(ClockType::TT);
        let delta = Self::tdb_minus_tt(tt);
        tt.add(delta).with_clock_type(ClockType::TDB)
    }

    const fn tdb_to_tai(tdb: Self) -> Self {
        let mut tt = tdb.with_clock_type(ClockType::TT);
        let mut i = 0u32;

        while i < 8 {
            let delta = Self::tdb_minus_tt(tt);
            tt = tdb.with_clock_type(ClockType::TT).sub(delta);
            i += 1;
        }

        tt.sub_ref(&TT_TAI_OFFSET_DELTA)
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
        let delta_attos = Self::mul_lg(elapsed);
        tt.add(Delta::from_total_attos(delta_attos))
            .with_clock_type(ClockType::TCG)
    }

    const fn tcg_to_tt(tcg: Self) -> Self {
        let elapsed_cg = Self::elapsed_attos_since_ref(tcg);
        let delta_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.sub(Delta::from_total_attos(delta_attos))
            .with_clock_type(ClockType::TT)
    }

    const fn tcb_to_tdb(tcb: Self) -> Self {
        let elapsed_cg = Self::elapsed_attos_since_ref(tcb);
        let delta_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.sub(Delta::from_total_attos(delta_attos))
            .sub(Delta::from_total_attos(TDB0_ATTOS))
            .with_clock_type(ClockType::TDB)
    }

    const fn tdb_to_tcb(tdb: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(tdb.with_clock_type(ClockType::TT));
        let delta_attos = Self::mul_lb(elapsed);
        tdb.add(Delta::from_total_attos(delta_attos))
            .add(Delta::from_total_attos(TDB0_ATTOS))
            .with_clock_type(ClockType::TCB)
    }

    const fn tt_to_ltc(tt: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(tt);
        let delta_attos = Self::mul_lm(elapsed);
        tt.add(Delta::from_total_attos(delta_attos))
            .with_clock_type(ClockType::LTC)
    }

    const fn ltc_to_tt(ltc: Self) -> Self {
        let elapsed = Self::elapsed_attos_since_ref(ltc);
        let delta_attos = Self::mul_rate(elapsed, LM_NUM, LM_DEN + LM_NUM);
        ltc.sub(Delta::from_total_attos(delta_attos))
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
    pub const fn to_msd_exact(self) -> (i64, Delta) {
        let tt = self.to_clock_type(ClockType::TT);
        let elapsed = Self::elapsed_attos_since_mars_ref(tt);
        let attos_per_sol = MARS_SOL_ATTOS;

        let whole_sols = elapsed.div_euclid(attos_per_sol) as i64;
        let frac_attos = elapsed.rem_euclid(attos_per_sol);
        let frac_sol = Delta::from_total_attos(frac_attos);

        (whole_sols, frac_sol)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`Delta`] representing seconds into the current sol
    /// (range [0, one Martian sol)).
    #[inline]
    pub const fn to_mtc(self) -> Delta {
        let (_, frac_sol) = self.to_msd_exact();
        frac_sol
    }

    /// Creates a `TimePoint` (in TT) from an exact Mars Sol Date using full library precision.
    pub const fn from_msd_exact(whole_sols: i64, frac_sol: Delta) -> Self {
        let frac_attos = frac_sol.total_attos();
        let elapsed_attos = (whole_sols as i128) * MARS_SOL_ATTOS + frac_attos;

        let ref_tt = Self::mars_ref_tt();
        let tt = ref_tt.add(Delta::from_total_attos(elapsed_attos));
        tt.to_tai()
    }

    /// Returns an exact Julian Date in Terrestrial Time (TT) with full library precision.
    ///
    /// The returned tuple consists of the integer number of Julian days and the fractional part of
    /// the day expressed as a [`Delta`] (always in the range [0, 1 day)).
    #[inline]
    pub const fn to_jd_tt_exact(self) -> (i64, Delta) {
        let tt = self.to_clock_type(ClockType::TT);
        let days_since_j2000 = tt.sec.div_euclid(SEC_PER_DAYI64);
        let remaining_sec = tt.sec.rem_euclid(SEC_PER_DAYI64);
        let frac = Delta::new(remaining_sec, tt.subsec);
        (J2000_JD_TT + days_since_j2000, frac)
    }

    /// Returns an exact Modified Julian Date in Terrestrial Time (TT) with full library precision.
    #[inline]
    pub const fn to_mjd_tt_exact(self) -> (i64, Delta) {
        let (jd, frac) = self.to_jd_tt_exact();
        (jd - 2_400_000, frac)
    }

    /// Creates a `TimePoint` from an exact Julian Date in Terrestrial Time using full library precision.
    #[inline]
    pub const fn from_jd_tt_exact(jd_days: i64, frac: Delta) -> Self {
        let days_since_j2000 = jd_days - J2000_JD_TT;
        let total_sec = days_since_j2000 * SEC_PER_DAYI64 + frac.sec;
        let tt = TimePoint::new(total_sec, frac.subsec, ClockType::TT);
        tt.to_tai()
    }

    /// Creates a `TimePoint` from an exact Modified Julian Date in Terrestrial Time using full library
    /// precision.
    #[inline]
    pub const fn from_mjd_tt_exact(mjd_days: i64, frac: Delta) -> Self {
        Self::from_jd_tt_exact(mjd_days + 2_400_000, frac)
    }
}

impl TimePoint {
    /// TODO: SHOULDN'T USE TT?
    /// Returns the Julian Date in UTC (computed on the TT scale and then expressed in UTC).
    /// Non-exact Real.
    #[inline]
    pub const fn to_jd_utc(self) -> Real {
        self.to_clock_type(ClockType::UTC).to_jd_tt()
    }

    /// TODO: SHOULDN'T USE TT?
    /// Returns the Modified Julian Date in UTC (computed on the TT scale and then expressed in UTC).
    /// Non-exact Real.
    #[inline]
    pub const fn to_mjd_utc(self) -> Real {
        self.to_clock_type(ClockType::UTC).to_mjd_tt()
    }

    /// Creates a `TimePoint` (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    #[inline]
    pub const fn from_msd(msd: Real) -> Self {
        let whole = floor_f(msd) as i64;
        let frac = msd - (whole as Real);
        let frac_delta = Delta::from_sec_f(frac * MARS_SOL_LENGTH_SEC);
        Self::from_msd_exact(whole, frac_delta)
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd(self) -> Real {
        let (whole, frac) = self.to_msd_exact();
        whole as Real + frac.as_sec_f() / MARS_SOL_LENGTH_SEC
    }

    /// Returns the standard Julian Date in Terrestrial Time (TT) as a floating-point value.
    ///
    /// By international convention J2000.0 TT corresponds to JD 2451545.0 exactly. The returned value
    /// uses the highest precision possible with `Real`. For full attosecond accuracy use
    /// [`Self::to_jd_tt_exact`].
    #[inline]
    pub const fn to_jd_tt(self) -> Real {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let days_f = jd_days as Real;
        let frac_days = frac.as_sec_f() / SEC_PER_DAY;
        days_f + frac_days
    }

    /// Returns the standard Modified Julian Date in Terrestrial Time (TT) as a floating-point value.
    ///
    /// J2000.0 TT corresponds to MJD 51544.5 exactly.
    /// Non-exact Real.
    #[inline]
    pub const fn to_mjd_tt(self) -> Real {
        self.to_jd_tt() - f!(2_400_000.5)
    }
}
