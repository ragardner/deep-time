use crate::historical_sofa::historical_sofa_offset_for_non_adjusted;
use crate::{
    Drift, Dt, LB_DEN, LB_NUM, LG_DEN, LG_NUM, Scale, TAI_ATTOS_AT_1972,
    TCG_TCB_REF_ATTOS_SINCE_J2000, TDB0_ATTOS, TT_TAI_OFFSET,
};

impl Dt {
    /// Converts this instant to the target scale and returns the signed difference
    /// from the given epoch.
    ///
    /// This is a low-level `const fn` used internally by higher-level conversion
    /// methods such as [`to_ymd`](Dt::to_ymd).
    ///
    /// ## Arguments
    ///
    /// * `to` — The time scale to convert `self` into before computing the difference.
    /// * `epoch` — The reference epoch (e.g. [`Dt::UNIX_EPOCH`]) from which the
    ///   difference is calculated.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] representing the signed difference (seconds + attoseconds) between
    /// this instant (after conversion to `to`) and the provided `epoch`.
    ///
    /// The returned value is a signed offset relative to `epoch` in the `to` scale.
    /// While it is most commonly used as a pure duration, it can also be interpreted
    /// as a timestamp when `epoch` is something like
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) (e.g. for
    /// generating Unix timestamps via `.to_ms()` or `.to_sec()`).
    ///
    /// ## See also
    ///
    /// * [`Dt::to`](../struct.Dt.html#method.to).
    /// * [`Dt::to_diff_raw`](../struct.Dt.html#method.to_diff_raw).
    /// * [`Dt::from_diff_and_scale`](../struct.Dt.html#method.from_diff_and_scale).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_ymd(2024, 6, 15, 12, 0, 0, 0, Scale::UTC);
    /// let diff = dt.to_scale_and_diff(Dt::UNIX_EPOCH, true);
    ///
    /// // diff can be used as a Unix timestamp offset
    /// let unix_ms = diff.to_ms();
    /// assert!(unix_ms > 1_700_000_000_000);
    /// ```
    pub const fn to_scale_and_diff(&self, epoch: Dt, convert_epoch: bool) -> Dt {
        if convert_epoch {
            self.to(self.target).to_diff_raw(epoch.to(self.target))
        } else {
            self.to(self.target).to_diff_raw(epoch)
        }
    }

    /// Creates a **TAI** [`Dt`] by adding a difference to an epoch and interpreting
    /// the result on the given time scale.
    ///
    /// This is the inverse counterpart to
    /// [`Dt::to_scale_and_diff`](../struct.Dt.html#method.to_scale_and_diff)
    /// and is used by [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
    /// and related constructors.
    ///
    /// ## Arguments
    ///
    /// - `diff` — The signed difference (as a [`Dt`]) to add to the epoch.
    /// - `epoch` — The reference epoch (commonly
    ///   [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) or
    ///   [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)).
    /// - `current` — The time scale on which `diff` + `epoch` should be interpreted.
    ///
    /// ## Returns
    ///
    /// A [`Dt`] on the **TAI** scale representing the absolute instant
    /// `epoch + diff` when interpreted on `current`.
    ///
    /// ## Notes
    ///
    /// - The input `diff` is treated as being on the `current` scale.
    /// - The final result is always converted to TAI (the internal canonical representation).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_scale_and_diff`](../struct.Dt.html#method.to_scale_and_diff)
    /// - [`Dt::from_attos`](../struct.Dt.html#method.from_attos)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let diff = Dt::from_tai_sec(1_718_467_200); // ~2024-06-15
    /// let dt = Dt::from_diff_and_scale(diff, Dt::UNIX_EPOCH, true);
    ///
    /// let ymd = dt.to_ymd();
    /// assert_eq!(ymd.yr(), 2024);
    /// assert_eq!(ymd.mo(), 6);
    /// assert_eq!(ymd.day(), 15);
    /// ```
    pub const fn from_diff_and_scale(diff: Dt, epoch: Dt, convert_epoch: bool) -> Dt {
        if convert_epoch {
            Self::from_attos(
                epoch
                    .to(diff.scale)
                    .to_attos()
                    .saturating_add(diff.to_attos()),
                diff.scale,
            )
        } else {
            Self::from_attos(epoch.to_attos().saturating_add(diff.to_attos()), diff.scale)
        }
    }

    /// Converts the internal attos to be on the TAI time [`Scale`].
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// let tai = Dt::from_ymd(2000, 1, 1, 12, 0, 0, 0, Scale::UTC);
    /// let tt = tai.to(Scale::TT);
    ///
    /// assert_eq!(tt.scale, Scale::TT);
    ///
    /// let roundtrip = tt.to_tai();
    ///
    /// assert_eq!(tai.scale, Scale::TAI);
    /// assert_eq!(roundtrip, tai);
    /// ```
    ///
    /// See [`Dt::to`](../struct.Dt.html#method.to) for more info.
    pub const fn to_tai(&self) -> Dt {
        match self.scale {
            Scale::UTC => {
                let raw = Dt::new(self.attos, Scale::TAI, self.target);
                raw.add_sec(raw.leap_sec(true).offset as i128)
            }
            Scale::TAI => *self,
            Scale::TT => Dt::new(
                self.attos.saturating_sub(TT_TAI_OFFSET.to_attos()),
                Scale::TAI,
                self.target,
            ),
            Scale::UTCSpice => {
                let raw = Dt::new(self.attos, Scale::TAI, self.target);
                if self.attos < TAI_ATTOS_AT_1972 - 10 {
                    raw.add_sec(9)
                } else {
                    raw.add_sec(raw.leap_sec(true).offset as i128)
                }
            }
            Scale::UTCSofa => {
                let raw = Dt::new(self.attos, Scale::TAI, self.target);
                if let Some(sofa_offset) = historical_sofa_offset_for_non_adjusted(&raw) {
                    raw.add(Dt::from_sec_f(sofa_offset, Scale::TAI))
                } else {
                    raw.add_sec(raw.leap_sec(true).offset as i128)
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => Dt::new(
                self.attos.saturating_add(Dt::SEC_19.to_attos()),
                Scale::TAI,
                self.target,
            ),
            Scale::BDT => Dt::new(
                self.attos.saturating_add(Dt::SEC_33.to_attos()),
                Scale::TAI,
                self.target,
            ),
            Scale::TDB | Scale::ET => {
                Self::tdb_to_tai(Dt::new(self.attos, Scale::TAI, self.target))
            }
            Scale::TCG => {
                let tt = Self::tcg_to_tt(Dt::new(self.attos, Scale::TAI, self.target));
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCB => {
                let tdb = Self::tcb_to_tdb(Dt::new(self.attos, Scale::TAI, self.target));
                Self::tdb_to_tai(tdb)
            }
            Scale::LTC => {
                let tt = Self::ltc_to_tt(Dt::new(self.attos, Scale::TAI, self.target));
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCL => Self::tcl_to_tai(Dt::new(self.attos, Scale::TAI, self.target)),
            _ => Dt::new(self.attos, Scale::TAI, self.target),
        }
    }

    /// Converts directly to `new` [`Scale`], without first converting to TAI.
    ///
    /// **Warning:**
    ///
    /// - This function should really only be used if the [`Dt`] is on the TAI
    ///   time scale, OR if you really know what you're doing.
    /// - For the normal time scale conversion function see
    ///   [`Dt::to`](../struct.Dt.html#method.to) which first converts to TAI
    ///   before converting to the target scale.
    pub const fn convert(&self, new: Scale) -> Dt {
        match new {
            Scale::TAI => self.to_tai(),
            Scale::UTC => {
                let offset = self.leap_sec(false).offset;
                self.add_sec(-offset as i128).with(new)
            }
            Scale::TT => self.add(TT_TAI_OFFSET).with(new),
            Scale::UTCSpice => {
                if self.to_attos() < TAI_ATTOS_AT_1972 {
                    self.add_sec(-9).with(new)
                } else {
                    let offset = self.leap_sec(false).offset;
                    self.add_sec(-offset as i128).with(new)
                }
            }
            Scale::UTCSofa => {
                if let Some(sofa_offset) = historical_sofa_offset_for_non_adjusted(&self) {
                    self.sub(Dt::span_f(sofa_offset)).with(new)
                } else {
                    let offset = self.leap_sec(false).offset;
                    self.add_sec(-offset as i128).with(new)
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => {
                self.add_attos(-Dt::SEC_19.to_attos()).with(new)
            }
            Scale::BDT => self.add_attos(-Dt::SEC_33.to_attos()).with(new),
            Scale::TDB | Scale::ET => Self::tai_to_tdb(*self).with(new),
            Scale::TCG => Self::tai_to_tcg(*self).with(new),
            Scale::TCB => Self::tai_to_tcb(*self).with(new),
            Scale::LTC => {
                let tt = self.add(TT_TAI_OFFSET);
                Self::tt_to_ltc(tt).with(new)
            }
            Scale::TCL => Self::tai_to_tcl(*self).with(new),
            _ => *self,
        }
    }

    /// Converts this instant to another time scale, going via TAI.
    ///
    /// Essentially when converting TT to TDB the internal process goes like TT
    /// -> TAI -> TDB. It uses the [`Dt`]s `scale` field to determine what scale
    /// to convert from to TAI, and then the `new` arg dictates the new time scale.
    ///
    /// - It is not necessary to do this if you just want to use such functions
    ///   as [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd) as these internally
    ///   convert to the scale of the object's `target` field before output.
    /// - If a TAI [`Dt`] was created using
    ///   [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd) and the datetime
    ///   had 60 seconds, converting to UTC would lose that info. To round trip a
    ///   60 second UTC datetime you need only set the
    ///   [`Dt::target`](../struct.Dt.html#method.target) [`Scale`] to `UTC` and
    ///   then call the desired output function, such as
    ///   [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd).
    /// - The internal `attos` field changes to be on the new time scale.
    /// - The [`Dt`]s `target` field is ignored and left unchanged.
    /// - The [`Dt`]s `scale` field is changed to the new [`Scale`].
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] representing the same physical instant but on the `new` scale.
    /// - The returned objects `scale` field has been changed to `new`.
    ///
    /// If `current == new`, this method returns `*self` without any computation.
    ///
    /// ## See also
    ///
    /// * [`Dt::to_tai`](../struct.Dt.html#method.to_tai)
    /// * [`Dt::from_attos`](../struct.Dt.html#method.from_attos)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let tai = Dt::from_ymd(2024, 6, 15, 12, 0, 0, 0, Scale::UTC);
    /// let tt = tai.to(Scale::TT);
    /// let tdb = tt.to(Scale::TDB);
    /// let roundtrip = tdb.to(Scale::TAI);
    ///
    /// let ymd = roundtrip.to_ymd();
    ///
    /// assert_eq!(ymd.yr(), 2024);
    /// assert_eq!(ymd.mo(), 6);
    /// assert_eq!(ymd.day(), 15);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    #[inline]
    pub const fn to(&self, new: Scale) -> Dt {
        if matches!(self.scale, Scale::TAI) {
            self.convert(new)
        } else if !self.scale.eq(new) {
            self.to_tai().convert(new)
        } else {
            *self
        }
    }

    /// Converts this instant to any other [`Scale`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`Drift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(self, reference: Dt, drift: Drift) -> Dt {
        let span = self.to_diff_raw(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Performs the inverse conversion of [`Dt::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    pub const fn convert_back_using_drift(self, reference: Dt, drift: Drift) -> Dt {
        if drift.rate.is_zero() && drift.accel.is_zero() {
            return self.sub(drift.constant);
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

    #[inline]
    pub(crate) const fn tai_to_tcg(tai: Dt) -> Dt {
        let tt = tai.add(TT_TAI_OFFSET);
        Self::tt_to_tcg(tt)
    }

    #[inline]
    pub(crate) const fn tai_to_tcb(tai: Dt) -> Dt {
        let tdb = Self::tai_to_tdb(tai);
        Self::tdb_to_tcb(tdb)
    }

    /// Exact integer helper: elapsed attoseconds since the TCG/TCB reference epoch (1977-01-01.0 TAI),
    /// using only the numerical value of the supplied `Dt` (scale is ignored).
    #[inline]
    pub(crate) const fn to_attos_since_tcg_tcb_epoch(numerical: Dt) -> i128 {
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

    pub(crate) const fn tt_to_tcg(tt: Dt) -> Dt {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tt);
        let span_attos = Self::mul_lg(elapsed);
        tt.add_attos(span_attos)
    }

    pub(crate) const fn tcg_to_tt(tcg: Dt) -> Dt {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcg);
        let span_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.add_attos(-span_attos)
    }

    pub(crate) const fn tcb_to_tdb(tcb: Dt) -> Dt {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcb);
        let span_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.add_attos(-span_attos).add_attos(-TDB0_ATTOS)
    }

    pub(crate) const fn tdb_to_tcb(tdb: Dt) -> Dt {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tdb);
        let span_attos = Self::mul_lb(elapsed);
        tdb.add_attos(span_attos).add_attos(TDB0_ATTOS)
    }
}
