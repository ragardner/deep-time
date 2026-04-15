use crate::leap_seconds::leap_seconds_before;
use crate::{
    ClockDrift, ClockModel, ClockType, Delta, J2000_SECONDS_PER_CENTURY, LB, LG, LM, POW15, POW21,
    Real, SEC_PER_DAY, TCG_TCB_REF_JD, TDB0, TT_TAI_OFFSET_DELTA, TimePoint, sin_approx,
};
#[cfg(test)]
#[path = "conversions_tests.rs"]
mod tests;

impl TimePoint {
    /// Converts this instant to any other [`ClockType`], representing the exact same physical moment.
    #[inline]
    pub const fn to_clock_type(self, target: ClockType) -> Self {
        if (self.clock_type as u8) == (target as u8) {
            return self;
        }
        let tai = self.to_tai();
        tai.from_tai(target)
    }

    /// Returns a copy of this `TimePoint` with a new [`ClockType`] while keeping the exact same
    /// numerical seconds and subseconds value. This is zero-cost after conversion.
    #[inline]
    pub(crate) const fn with_clock_type(self, clock_type: ClockType) -> Self {
        Self {
            sec: self.sec,
            subsec: self.subsec,
            clock_type,
        }
    }

    /// Sets the [`ClockType`] of this `TimePoint` **in place**, while keeping the
    /// exact same numerical seconds and subseconds value.
    ///
    /// This is the mutable counterpart to [`with_clock_type`]. It is zero-cost
    /// (just a single field assignment) and is also `const fn`.
    #[inline]
    pub const fn set_clock_type(&mut self, clock_type: ClockType) {
        self.clock_type = clock_type;
    }

    /// Converts this `TimePoint` (in any clock type) to TAI — the library’s internal canonical time clock type.
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

            ClockType::LTC => {
                // Still goes through the helper because it needs TT first,
                // but the helper itself already uses the low-copy style.
                Self::ltc_to_tt(self).to_tai()
            }

            ClockType::Proper | ClockType::Custom => self,
        }
    }

    /// Converts a TAI `TimePoint` to any other requested [`ClockType`].
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

            ClockType::LTC => {
                // We first convert TAI → TT (low-copy), then TT → LTC (low-copy)
                let tp = self.from_tai(ClockType::TT);
                // The LTC conversion is now also written in the low-copy style
                // inside tt_to_ltc / ltc_to_tt, so no extra temporary here.
                Self::tt_to_ltc(tp)
            }

            ClockType::Proper | ClockType::Custom => {
                let mut tp = self;
                tp.set_clock_type(target);
                tp
            }
        }
    }

    /// Converts this instant to any other [`ClockType`] while applying an exact quadratic
    /// relativistic / clock-drift correction via a [`ClockDrift`].
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

    /// Inverse of `convert_using_drift`.
    ///
    /// Uses fixed-point iteration (exactly like the existing `tdb_to_tai` implementation).
    /// Because relativistic / clock-drift rates are always tiny (|rate| ≪ 1),
    /// this recovers the original proper time to full 36-digit precision.
    #[inline]
    pub const fn convert_back_using_drift(
        self,
        source: ClockType,
        reference: Self,
        drift: ClockDrift,
    ) -> Self {
        // Fast path for the extremely common pure-constant case
        if drift.rate().is_zero() && drift.accel().is_zero() {
            return self.sub_ref(&drift.constant()).with_clock_type(source);
        }

        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            let delta = guess.duration_since(reference);
            let correction = drift.time_diff_after(&delta);
            guess = self.sub(correction); // target - drift(guess - ref)
            i += 1;
        }
        guess.with_clock_type(source)
    }

    /// Converts using a self-describing [`ClockModel`].
    ///
    /// Onboard `Proper`/`Custom` → whatever `ClockModel.base` is (TT, TDB, Custom/Mars time, etc.).
    /// This is the recommended one-line conversion.
    #[inline]
    pub const fn convert_using_model(self, model: ClockModel) -> Self {
        self.convert_using_drift(model.base, model.reference, model.drift)
    }

    /// Inverse of `convert_using_model`.
    #[inline]
    pub const fn convert_back_using_model(self, model: ClockModel) -> Self {
        self.convert_back_using_drift(model.base, model.reference, model.drift)
    }

    // ──────────────────────────────────────────────────────────────
    // Private UTC ↔ TAI conversion (leap seconds)
    // ──────────────────────────────────────────────────────────────

    const fn utc_to_tai(utc: Self) -> Self {
        let approx_tai_for_lookup = utc.add_ref(&Delta::SEC_37);
        let leaps = leap_seconds_before(approx_tai_for_lookup);
        utc.add(Delta::from_sec(leaps))
            .with_clock_type(ClockType::TAI)
    }

    const fn tai_to_utc(tai: Self) -> Self {
        let leaps = leap_seconds_before(tai);
        tai.sub(Delta::from_sec(leaps))
            .with_clock_type(ClockType::UTC)
    }

    // ──────────────────────────────────────────────────────────────
    // Private TDB conversion helpers
    // ──────────────────────────────────────────────────────────────

    const fn tdb_minus_tt(tt: Self) -> Delta {
        // Whole seconds as f64 (limited by f64 integer precision above ~9e15 s)
        let whole = tt.sec as Real;

        let q = tt.subsec / POW21; // integer < 10¹⁵, exact
        let frac = (q as Real) / (POW15 as Real);

        let seconds_since_j2000_tt = whole + frac;
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

    // ──────────────────────────────────────────────────────────────
    // Private TCG/TCB helpers (linear rate conversions)
    // ──────────────────────────────────────────────────────────────

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

    /// TCG ↔ TT (exact IAU linear relation)
    const fn tt_to_tcg(tt: Self) -> Self {
        let jd_tt = tt.to_jd_tt();
        let days = jd_tt - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LG;
        tt.add(Delta::from_sec_f(delta_s))
            .with_clock_type(ClockType::TCG)
    }

    const fn tcg_to_tt(tcg: Self) -> Self {
        let jd_tcg = tcg.to_jd_tt();
        let days = jd_tcg - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LG;
        tcg.sub(Delta::from_sec_f(delta_s))
            .with_clock_type(ClockType::TT)
    }

    /// TCB ↔ TDB (exact IAU 2006 linear relation)
    const fn tdb_to_tcb(tdb: Self) -> Self {
        let jd_tdb = tdb.to_jd_tt();
        let days = jd_tdb - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LB;
        tdb.add(Delta::from_sec_f(delta_s))
            .add_ref(&TDB0) // TDB0 is already part of the defining relation
            .with_clock_type(ClockType::TCB)
    }

    const fn tcb_to_tdb(tcb: Self) -> Self {
        let jd_tcb = tcb.to_jd_tt();
        let days = jd_tcb - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LB;
        tcb.sub(Delta::from_sec_f(delta_s))
            .sub_ref(&TDB0)
            .with_clock_type(ClockType::TDB)
    }

    // ──────────────────────────────────────────────────────────────
    // Julian Date & Modified Julian Date (TT scale)
    // ──────────────────────────────────────────────────────────────

    /// Returns the standard Julian Date in Terrestrial Time (TT) as `float`.
    ///
    /// J2000.0 TT corresponds to JD 2451545.0 exactly (Astropy/SPICE/NASA convention).
    ///
    /// **Lossy by design** — uses the best possible `float` conversion of the exact
    /// fractional day. For full precision use `to_jd_tt_exact()` (returns `(i128, Delta)`).
    #[inline]
    pub const fn to_jd_tt(self) -> Real {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let days_f = jd_days as Real;
        let frac_days = frac.as_sec_f() / SEC_PER_DAY;
        days_f + frac_days
    }

    /// Returns the standard Modified Julian Date in Terrestrial Time (TT) as `float`.
    ///
    /// J2000.0 TT corresponds to MJD 51544.5 exactly.
    #[inline]
    pub const fn to_mjd_tt(self) -> Real {
        self.to_jd_tt() - f!(2_400_000.5)
    }

    /// Returns an **exact** Julian Date in TT with full library precision.
    ///
    /// The returned tuple is `(jd_integer_days, fractional_day)` where the fractional part
    /// is a [`Delta`] representing the part of a day (always < 1 day).
    #[inline]
    pub const fn to_jd_tt_exact(self) -> (i128, Delta) {
        let tt = self.to_clock_type(ClockType::TT);
        let days = tt.sec / 86_400;
        let remaining_sec = tt.sec % 86_400;
        let frac = Delta::new(remaining_sec, tt.subsec);
        (2451545 + days, frac)
    }

    /// Returns an **exact** Modified Julian Date in TT with full library precision.
    #[inline]
    pub const fn to_mjd_tt_exact(self) -> (i128, Delta) {
        let (jd, frac) = self.to_jd_tt_exact();
        (jd - 2_400_000, frac)
    }

    /// Creates a `TimePoint` from an exact Julian Date in TT (full precision, no loss).
    #[inline]
    pub const fn from_jd_tt_exact(jd_days: i128, frac: Delta) -> Self {
        let total_sec = jd_days * 86_400 + frac.sec;
        let tt = TimePoint::new(total_sec, frac.subsec, ClockType::TT);
        tt.to_tai()
    }

    /// Creates a `TimePoint` from an exact Modified Julian Date in TT.
    #[inline]
    pub const fn from_mjd_tt_exact(mjd_days: i128, frac: Delta) -> Self {
        Self::from_jd_tt_exact(mjd_days + 2_400_000, frac)
    }

    /// Convenience method: Julian Date in UTC (TT-based).
    #[inline]
    pub const fn to_jd_utc(self) -> Real {
        self.to_clock_type(ClockType::UTC).to_jd_tt()
    }

    /// Convenience method: Modified Julian Date in UTC (TT-based).
    #[inline]
    pub const fn to_mjd_utc(self) -> Real {
        self.to_clock_type(ClockType::UTC).to_mjd_tt()
    }

    // ──────────────────────────────────────────────────────────────
    // Off-planet helpers
    // ──────────────────────────────────────────────────────────────

    /// LTC ↔ TT (exact linear NIST/Ashby & Patla 2024 relation)
    /// Uses the `with_clock_type` trick on the LTC side to avoid any recursion
    /// when `to_jd_tt()` internally calls `to_clock_type(ClockType::TT)`.
    const fn tt_to_ltc(tt: Self) -> Self {
        let days = tt.to_jd_tt() - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LM;
        let mut tp = tt.add(Delta::from_sec_f(delta_s));
        tp.set_clock_type(ClockType::LTC);
        tp
    }

    const fn ltc_to_tt(ltc: Self) -> Self {
        let days = ltc.with_clock_type(ClockType::TT).to_jd_tt() - TCG_TCB_REF_JD;
        let delta_s = days * SEC_PER_DAY * LM;
        let mut tp = ltc.sub(Delta::from_sec_f(delta_s));
        tp.set_clock_type(ClockType::TT);
        tp
    }
}
