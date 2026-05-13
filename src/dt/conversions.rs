use crate::historical_sofa::historical_sofa_offset_for_non_adjusted;
use crate::{
    Drift, ClockModel, Dt, LB_DEN, LB_NUM, LG_DEN, LG_NUM, Scale, TAI_SEC_AT_1972,
    TCG_TCB_REF_ATTOS_SINCE_J2000, TDB0_ATTOS, TT_TAI_OFFSET, tdb_minus_tt,
};

impl Dt {
    #[inline]
    pub const fn from_dt(dt: Dt, scale: Scale) -> Dt {
        Self::from(dt.sec, dt.attos, scale)
    }

    #[inline]
    pub const fn from_attos_since(attos: i128, reference: Dt) -> Self {
        reference.add(Dt::from_attos(attos, Scale::TAI))
    }

    #[inline]
    pub const fn to_scale_and_then_diff(&self, scale: Scale, epoch: Dt) -> Dt {
        self.to_internal(scale).to_diff_raw(epoch)
    }

    #[inline]
    pub const fn from_diff_and_scale(diff: Dt, epoch: Dt, current: Scale) -> Self {
        Dt::from_dt(epoch.add(diff), current)
    }

    /// Creates a TAI [`Dt`], converting from another scale.
    pub const fn from(sec: i64, attos: u64, scale: Scale) -> Dt {
        let raw = Dt::new(sec, attos);
        match scale {
            Scale::TAI | Scale::Custom | Scale::UT1 => raw,
            Scale::TT => raw.sub(TT_TAI_OFFSET),
            Scale::UTC => raw.add(Dt {
                sec: raw.leap_seconds(true).offset,
                attos: 0,
            }),
            Scale::UTCSpice => {
                let tai = raw.add(Dt {
                    sec: raw.leap_seconds(true).offset,
                    attos: 0,
                });
                if sec < TAI_SEC_AT_1972 - 10 {
                    tai.add(Dt::from_sec(9, Scale::TAI))
                } else {
                    tai
                }
            }
            Scale::UTCSofa => {
                let tai = raw.add(Dt {
                    sec: raw.leap_seconds(true).offset,
                    attos: 0,
                });
                if let Some(offset) = historical_sofa_offset_for_non_adjusted(&raw) {
                    tai.add(Dt::from_sec_f(offset))
                } else {
                    tai
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => raw.add(Dt::SEC_19),
            Scale::BDT => raw.add(Dt::SEC_33),
            Scale::TDB | Scale::ET => Self::tdb_to_tai(raw),
            Scale::TCG => {
                let tt = Self::tcg_to_tt(raw);
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCB => {
                let tdb = Self::tcb_to_tdb(raw);
                Self::tdb_to_tai(tdb)
            }
            Scale::LTC => {
                let tt = Self::ltc_to_tt(raw);
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCL => Self::tcl_to_tai(raw),
        }
    }

    pub(crate) const fn to_internal(&self, scale: Scale) -> Dt {
        match scale {
            Scale::TAI | Scale::Custom | Scale::UT1 => *self,
            Scale::TT => self.add(TT_TAI_OFFSET),
            Scale::UTC => self.sub(Dt {
                sec: self.leap_seconds(false).offset,
                attos: 0,
            }),
            Scale::UTCSpice => {
                let spice = self.sub(Dt {
                    sec: self.leap_seconds(false).offset,
                    attos: 0,
                });
                if self.sec < TAI_SEC_AT_1972 {
                    spice.sub(Dt::from_sec_f(f!(9.0)))
                } else {
                    spice
                }
            }
            Scale::UTCSofa => {
                let sofa = self.sub(Dt {
                    sec: self.leap_seconds(false).offset,
                    attos: 0,
                });
                if let Some(offset) = historical_sofa_offset_for_non_adjusted(&self) {
                    sofa.sub(Dt::from_sec_f(offset))
                } else {
                    sofa
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => self.sub(Dt::SEC_19),
            Scale::BDT => self.sub(Dt::SEC_33),
            Scale::TDB | Scale::ET => Self::tai_to_tdb(*self),
            Scale::TCG => Self::tai_to_tcg(*self),
            Scale::TCB => Self::tai_to_tcb(*self),
            Scale::LTC => {
                let tt = self.add(TT_TAI_OFFSET);
                Self::tt_to_ltc(tt)
            }
            Scale::TCL => Self::tai_to_tcl(*self),
        }
    }

    #[inline]
    pub const fn to_tai(&self, current: Scale) -> Dt {
        Self::from(self.sec, self.attos, current)
    }

    #[inline]
    pub const fn to(&self, current: Scale, target: Scale) -> Dt {
        if !current.eq(target) {
            Self::from(self.sec, self.attos, current).to_internal(target)
        } else {
            *self
        }
    }

    /// Converts this instant to any other [`Scale`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`Drift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(self, reference: Self, drift: Drift) -> Self {
        let span = self.to_diff_raw(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Performs the inverse conversion of [`Self::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    pub const fn convert_back_using_drift(self, reference: Self, drift: Drift) -> Self {
        if drift.rate().is_zero() && drift.accel().is_zero() {
            return self.sub(*drift.constant());
        }
        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            let span = guess.to_diff_raw(reference);
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

    pub const fn tai_to_tdb(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET);
        let correction = tdb_minus_tt(tt.to_sec_f());
        tt.add(Dt::from_sec_f(correction))
    }

    pub const fn tdb_to_tai(tdb: Self) -> Self {
        // Linear-rate + constant initial guess (dominant part of the forward transformation)
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tdb);
        let linear_span = Self::mul_lb(elapsed); // LB * elapsed
        let mut tt = tdb
            .sub(Dt::from_attos(linear_span, Scale::TAI))
            .sub(Dt::from_attos(TDB0_ATTOS, Scale::TAI));

        // Fixed-point iteration: TT_{n+1} = TDB − P(TT_n)
        let mut i = 0u32;
        while i < 8 {
            let p = tdb_minus_tt(tt.to_sec_f());
            let new_tt = tdb.sub(Dt::from_sec_f(p));

            // Early exit when change is smaller than ~1 atto-second
            let delta = new_tt.to_diff_raw(tt);
            if delta.sec == 0 && delta.attos < 1 {
                tt = new_tt;
                break;
            }

            tt = new_tt;
            i += 1;
        }

        tt.sub(TT_TAI_OFFSET)
    }

    pub(crate) const fn tai_to_tcg(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET);
        Self::tt_to_tcg(tt)
    }

    pub(crate) const fn tai_to_tcb(tai: Self) -> Self {
        let tdb = Self::tai_to_tdb(tai);
        Self::tdb_to_tcb(tdb)
    }

    /// Exact integer helper: elapsed attoseconds since the TCG/TCB reference epoch (1977-01-01.0 TAI),
    /// using only the numerical `sec`/`attos` of the supplied `Dt` (scale is ignored).
    #[inline]
    pub(crate) const fn to_attos_since_tcg_tcb_epoch(numerical: Self) -> i128 {
        numerical.to_attos() - TCG_TCB_REF_ATTOS_SINCE_J2000
    }

    /// Exact fixed-point multiplication: `attos * num / den` (handles negative values safely, no overflow for library time range).
    pub(crate) const fn mul_rate(attos: i128, num: i128, den: i128) -> i128 {
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
    pub(crate) const fn mul_lg(attos: i128) -> i128 {
        Self::mul_rate(attos, LG_NUM, LG_DEN)
    }

    #[inline]
    pub(crate) const fn mul_lb(attos: i128) -> i128 {
        Self::mul_rate(attos, LB_NUM, LB_DEN)
    }

    pub(crate) const fn tt_to_tcg(tt: Self) -> Self {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tt);
        let span_attos = Self::mul_lg(elapsed);
        tt.add(Dt::from_attos(span_attos, Scale::TAI))
    }

    pub(crate) const fn tcg_to_tt(tcg: Self) -> Self {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcg);
        let span_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.sub(Dt::from_attos(span_attos, Scale::TAI))
    }

    pub(crate) const fn tcb_to_tdb(tcb: Self) -> Self {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcb);
        let span_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.sub(Dt::from_attos(span_attos, Scale::TAI))
            .sub(Dt::from_attos(TDB0_ATTOS, Scale::TAI))
    }

    pub(crate) const fn tdb_to_tcb(tdb: Self) -> Self {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tdb);
        let span_attos = Self::mul_lb(elapsed);
        tdb.add(Dt::from_attos(span_attos, Scale::TAI))
            .add(Dt::from_attos(TDB0_ATTOS, Scale::TAI))
    }
}
