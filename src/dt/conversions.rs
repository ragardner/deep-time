use crate::historical_sofa::historical_sofa_offset_for_non_adjusted;
use crate::{
    Drift, Dt, LB_DEN, LB_NUM, LG_DEN, LG_NUM, Scale, TAI_ATTOS_AT_1972,
    TCG_TCB_REF_ATTOS_SINCE_J2000, TDB0_ATTOS, TT_TAI_OFFSET,
};

impl Dt {
    /// Convenience wrapper for [`Dt::from`](../struct.Dt.html#method.from)
    #[inline]
    pub const fn from_dt(dt: Dt, scale: Scale) -> Dt {
        Self::from_attos(dt.to_attos(), scale)
    }

    /// Low level constructor from total attoseconds since a given `epoch`.
    ///
    /// Simply adds the total attoseconds to the epoch.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// // A leap second from the middle of the table (36 leap seconds accumulated)
    /// let original = Dt::from(2015, 6, 30, 23, 59, 60, 123_456_789_000_000_000, Scale::UTC);
    ///
    /// // Round-trip through canonical attoseconds
    /// let canon = original.to_diff_raw(Dt::UNIX_EPOCH).to_attos();
    /// let roundtrip1 = Dt::from_attos_since(canon, Dt::UNIX_EPOCH);
    ///
    /// assert_eq!(original, roundtrip1, "Canonical round-trip failed");
    /// ```
    #[inline]
    pub const fn from_attos_since(attos: i128, epoch: Dt) -> Self {
        epoch.add(Dt::new(attos, epoch.tag))
    }

    /// Converts this instant to the target scale and returns the signed difference
    /// from the given epoch.
    ///
    /// This is a low-level `const fn` used internally by higher-level conversion
    /// methods such as [`to_ymdhms_on`](Dt::to_ymdhms_on).
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
    /// let dt = Dt::from_ymdhms(2024, 6, 15, 12, 0, 0, 0);
    /// let diff = dt.to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH);
    ///
    /// // diff can be used as a Unix timestamp offset
    /// let unix_ms = diff.to_ms();
    /// assert!(unix_ms > 1_700_000_000_000);
    /// ```
    #[inline]
    pub const fn to_scale_and_then_diff(&self, epoch: Dt, convert_epoch: bool) -> Dt {
        if convert_epoch {
            self.convert_internal(self.tag)
                .to_diff_raw(epoch.convert_internal(self.tag))
        } else {
            self.convert_internal(self.tag).to_diff_raw(epoch)
        }
    }

    /// Creates a **TAI** [`Dt`] by adding a difference to an epoch and interpreting
    /// the result on the given time scale.
    ///
    /// This is the inverse-style counterpart to
    /// [`Dt::to_scale_and_then_diff`](../struct.Dt.html#method.to_scale_and_then_diff)
    /// and is used by [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on)
    /// and related constructors.
    ///
    /// ## Arguments
    ///
    /// * `diff` — The signed difference (as a [`Dt`]) to add to the epoch.
    /// * `epoch` — The reference epoch (commonly
    ///   [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) or
    ///   [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)).
    /// * `current` — The time scale on which `diff` + `epoch` should be interpreted.
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
    /// * [`Dt::from_dt`](../struct.Dt.html#method.from_dt) — the underlying constructor.
    /// * [`Dt::to_scale_and_then_diff`](../struct.Dt.html#method.to_scale_and_then_diff) — the complementary operation.
    /// * [`Dt::from_ymdhms_on`](../struct.Dt.html#method.from_ymdhms_on) — a higher-level user of this function.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let diff = Dt::from_sec(1_718_467_200, Scale::TAI); // ~2024-06-15
    /// let dt = Dt::from_diff_and_scale(diff, Dt::UNIX_EPOCH, Scale::UTC);
    ///
    /// let ymd = dt.to_ymdhms(Scale::TAI);
    /// assert_eq!(ymd.yr(), 2024);
    /// assert_eq!(ymd.mo(), 6);
    /// assert_eq!(ymd.day(), 15);
    /// ```
    #[inline]
    pub const fn from_diff_and_scale(diff: Dt, epoch: Dt, convert_epoch: bool) -> Self {
        if convert_epoch {
            Self::from_attos(
                epoch
                    .convert_internal(diff.tag)
                    .to_attos()
                    .saturating_add(diff.to_attos()),
                diff.tag,
            )
        } else {
            Self::from_attos(epoch.to_attos().saturating_add(diff.to_attos()), diff.tag)
        }
    }

    /// Creates a **TAI** [`Dt`].
    ///
    /// - Assumes the given total attoseconds value is the numerical representation
    ///   on the given scale (i.e. the value you would get from `sec * 10¹⁸ + attos`
    ///   on that scale).
    /// - See [`Scale`] for more information on available time scales.
    ///
    /// ## Example
    ///
    /// ```
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_sec(-32, Scale::UTC);
    ///
    /// // leap seconds were added to the `-32` UTC sec
    /// // and the returned [`Dt`] is on the TAI scale
    /// assert_eq!(dt.to_attos(), 0);
    /// ```
    pub const fn from_attos(attos: i128, current: Scale) -> Dt {
        match current {
            Scale::UTC => {
                let raw = Dt::new(attos, current);
                raw.add_sec(raw.leap_sec(true).offset as i128)
            }
            Scale::TAI => Dt::new(attos, current),
            Scale::TT => Dt::new(attos.saturating_sub(TT_TAI_OFFSET.to_attos()), current),
            Scale::UTCSpice => {
                let raw = Dt::new(attos, current);
                if attos < TAI_ATTOS_AT_1972 - 10 {
                    raw.add_sec(9)
                } else {
                    raw.add_sec(raw.leap_sec(true).offset as i128)
                }
            }
            Scale::UTCSofa => {
                let raw = Dt::new(attos, current);
                if let Some(sofa_offset) = historical_sofa_offset_for_non_adjusted(&raw) {
                    raw.add(Dt::from_sec_f(sofa_offset, Scale::TAI))
                } else {
                    raw.add_sec(raw.leap_sec(true).offset as i128)
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => {
                Dt::new(attos.saturating_add(Dt::SEC_19.to_attos()), current)
            }
            Scale::BDT => Dt::new(attos.saturating_add(Dt::SEC_33.to_attos()), current),
            Scale::TDB | Scale::ET => Self::tdb_to_tai(Dt::new(attos, current)),
            Scale::TCG => {
                let tt = Self::tcg_to_tt(Dt::new(attos, current));
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCB => {
                let tdb = Self::tcb_to_tdb(Dt::new(attos, current));
                Self::tdb_to_tai(tdb)
            }
            Scale::LTC => {
                let tt = Self::ltc_to_tt(Dt::new(attos, current));
                tt.sub(TT_TAI_OFFSET)
            }
            Scale::TCL => Self::tcl_to_tai(Dt::new(attos, current)),
            _ => Dt::new(attos, current),
        }
    }

    /// From TAI -> `scale`
    pub const fn convert_internal(&self, scale: Scale) -> Dt {
        match scale {
            Scale::TAI => Self::from_attos(self.attos, self.tag),
            Scale::UTC => {
                let offset = self.leap_sec(false).offset;
                self.add_sec(-offset as i128).tag(scale)
            }
            Scale::TT => self.add(TT_TAI_OFFSET).tag(scale),
            Scale::UTCSpice => {
                if self.to_attos() < TAI_ATTOS_AT_1972 {
                    self.add_sec(-9).tag(scale)
                } else {
                    let offset = self.leap_sec(false).offset;
                    self.add_sec(-offset as i128).tag(scale)
                }
            }
            Scale::UTCSofa => {
                if let Some(sofa_offset) = historical_sofa_offset_for_non_adjusted(&self) {
                    self.sub(Dt::span_f(sofa_offset)).tag(scale)
                } else {
                    let offset = self.leap_sec(false).offset;
                    self.add_sec(-offset as i128).tag(scale)
                }
            }
            Scale::GPS | Scale::QZSS | Scale::GST => {
                self.add_attos(-Dt::SEC_19.to_attos()).tag(scale)
            }
            Scale::BDT => self.add_attos(-Dt::SEC_33.to_attos()).tag(scale),
            Scale::TDB | Scale::ET => Self::tai_to_tdb(*self).tag(scale),
            Scale::TCG => Self::tai_to_tcg(*self).tag(scale),
            Scale::TCB => Self::tai_to_tcb(*self).tag(scale),
            Scale::LTC => {
                let tt = self.add(TT_TAI_OFFSET);
                Self::tt_to_ltc(tt).tag(scale)
            }
            Scale::TCL => Self::tai_to_tcl(*self).tag(scale),
            _ => *self,
        }
    }

    #[inline]
    pub const fn tag(&self, s: Scale) -> Dt {
        Dt::new(self.attos, s)
    }

    /// Converts this instant to any other [`Scale`] while applying an exact quadratic relativistic
    /// or clock-drift correction defined by a [`Drift`] model relative to a reference instant.
    #[inline]
    pub const fn convert_using_drift(self, reference: Self, drift: Drift) -> Self {
        let span = self.to_diff_raw(reference);
        let correction = drift.time_diff_after(&span);
        self.add(correction)
    }

    /// Performs the inverse conversion of [`Dt::convert_using_drift`], recovering the original proper
    /// time on the source clock scale.
    ///
    /// A fixed-point iteration (at most 16 steps) is used to solve the implicit equation. For the common
    /// case of a pure constant offset the function returns immediately without iteration.
    pub const fn convert_back_using_drift(self, reference: Self, drift: Drift) -> Self {
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
    pub(crate) const fn tai_to_tcg(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET);
        Self::tt_to_tcg(tt)
    }

    #[inline]
    pub(crate) const fn tai_to_tcb(tai: Self) -> Self {
        let tdb = Self::tai_to_tdb(tai);
        Self::tdb_to_tcb(tdb)
    }

    /// Exact integer helper: elapsed attoseconds since the TCG/TCB reference epoch (1977-01-01.0 TAI),
    /// using only the numerical value of the supplied `Dt` (scale is ignored).
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
        tt.add_attos(span_attos)
    }

    pub(crate) const fn tcg_to_tt(tcg: Self) -> Self {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcg);
        let span_attos = Self::mul_rate(elapsed_cg, LG_NUM, LG_DEN + LG_NUM);
        tcg.add_attos(-span_attos)
    }

    pub(crate) const fn tcb_to_tdb(tcb: Self) -> Self {
        let elapsed_cg = Self::to_attos_since_tcg_tcb_epoch(tcb);
        let span_attos = Self::mul_rate(elapsed_cg, LB_NUM, LB_DEN + LB_NUM);
        tcb.add_attos(-span_attos).add_attos(-TDB0_ATTOS)
    }

    pub(crate) const fn tdb_to_tcb(tdb: Self) -> Self {
        let elapsed = Self::to_attos_since_tcg_tcb_epoch(tdb);
        let span_attos = Self::mul_lb(elapsed);
        tdb.add_attos(span_attos).add_attos(TDB0_ATTOS)
    }
}
