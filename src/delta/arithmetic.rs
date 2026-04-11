use crate::{Delta, DtBig, MICROQUECTOS_PER_SEC, POW15, POW21, floor_f64};

impl Delta {
    /// Returns the sum of `self` and `rhs`.
    ///
    /// The result is normalized so the fractional part lies in `[0, MICROQUECTOS_PER_SEC)`.
    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        let mut sec = self.sec + rhs.sec;
        let mut subsec = self.subsec + rhs.subsec;

        if subsec >= MICROQUECTOS_PER_SEC {
            sec += 1;
            subsec -= MICROQUECTOS_PER_SEC;
        }

        Self { sec, subsec }
    }

    /// Returns the difference `self - rhs`.
    ///
    /// The result is normalized so the fractional part lies in `[0, MICROQUECTOS_PER_SEC)`.
    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        let mut sec = self.sec - rhs.sec;
        let mut subsec = self.subsec;

        if subsec >= rhs.subsec {
            subsec -= rhs.subsec;
        } else {
            sec -= 1;
            subsec += MICROQUECTOS_PER_SEC - rhs.subsec;
        }

        Self { sec, subsec }
    }

    /// Returns the sum of `self` and `rhs`, saturating at [`Delta::MAX`] or [`Delta::MIN`] on overflow.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        let mut subsec = self.subsec + rhs.subsec;
        let mut carry = 0i128;
        if subsec >= MICROQUECTOS_PER_SEC {
            subsec -= MICROQUECTOS_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(rhs.sec).saturating_add(carry);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self { sec, subsec }
    }

    /// Returns the difference `self - rhs`, saturating at [`Delta::MAX`] or [`Delta::MIN`] on overflow.
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        let mut subsec = self.subsec;
        let mut borrow = 0i128;
        if subsec >= rhs.subsec {
            subsec -= rhs.subsec;
        } else {
            subsec += MICROQUECTOS_PER_SEC - rhs.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(rhs.sec).saturating_sub(borrow);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self { sec, subsec }
    }

    /// Returns `true` if this duration is exactly zero.
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.sec == 0 && self.subsec == 0
    }

    /// Converts this duration to a floating-point number of seconds.
    ///
    /// **Lossy by design** — returns the best possible `f64` representation
    /// (≈15.95 decimal digits). Use `sec` + `subsec` (or `DtBig`) for full 36-digit precision.
    #[inline]
    pub const fn as_sec_f64(self) -> f64 {
        let whole = self.sec as f64;

        // Extract the top 15 decimal digits exactly (POW15 is fully representable in f64 mantissa)
        let q = self.subsec / POW21; // integer division, exact
        let frac = (q as f64) / (POW15 as f64);

        whole + frac
    }

    /// Creates a `Delta` from a floating-point number of seconds.
    ///
    /// The result is normalized so the fractional part lies in `[0, 1)`.
    /// Negative values are handled correctly.
    ///
    /// **Precision note**: This implementation extracts the full ~15.95 decimal digits
    /// of precision that `f64` can provide. It avoids the original loss caused by
    /// `MICROQUECTOS_PER_SEC` (10³⁶) not being exactly representable in `f64`.
    /// We split the scaling into two exact steps: `10¹⁵` (safe in `f64` mantissa)
    /// followed by an integer `10²¹` multiply.
    pub const fn from_sec_f64(sec_f: f64) -> Self {
        if sec_f.is_nan() {
            return Self::ZERO;
        }
        if sec_f.is_infinite() {
            return if sec_f.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let mut sec = floor_f64(sec_f) as i128;
        let mut frac = sec_f - (sec as f64);

        if frac < 0.0 {
            sec -= 1;
            frac += 1.0;
        }

        let high = (frac * (POW15 as f64)) as u128;
        let subsec = high * POW21;

        Self { sec, subsec }
    }

    /// 10³⁶ as a `DtBig` (constant)
    const fn mqs() -> DtBig {
        DtBig::TEN.pow(36)
    }

    /// Convert `Delta` → total microquectoseconds (exact 320-bit signed integer).
    #[inline(always)]
    pub const fn to_big(self) -> DtBig {
        let sec_big = DtBig::from_i128(self.sec);
        let sub_big = DtBig::from_u128(self.subsec);
        sec_big.wrapping_mul(Self::mqs()).wrapping_add(sub_big)
    }

    /// Convert total microquectoseconds back to normalized/saturated `Delta`.
    #[inline(always)]
    pub const fn from_big(total: DtBig) -> Self {
        let m = Self::mqs();
        let sec_big = total.div_euclid(m);
        let sub_big = total.rem_euclid(m);
        let sec = sec_big.to_i128_saturating();
        let subsec = sub_big.to_u128_saturating();
        if sec == i128::MAX {
            Self {
                sec,
                subsec: MICROQUECTOS_PER_SEC - 1,
            }
        } else if sec == i128::MIN {
            Self { sec, subsec: 0 }
        } else {
            Self { sec, subsec }
        }
    }

    /// Returns the **largest** multiple of `unit` that is ≤ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn floor(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_big();
        let b = unit.to_big();
        let q = a.div_euclid(b);
        let result = q.wrapping_mul(b);
        Self::from_big(result)
    }

    /// Returns the **smallest** multiple of `unit` that is ≥ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn ceil(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_big();
        let b = unit.to_big();
        // ceil(a/b) ≡ −floor(−a/b)
        let neg_a = a.wrapping_neg();
        let q = neg_a.div_euclid(b);
        let q_ceil = q.wrapping_neg();
        let result = q_ceil.wrapping_mul(b);
        Self::from_big(result)
    }

    /// Returns the nearest multiple of `unit`.
    /// Halfway cases round **away from zero** (matches old `f64::round`).
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn round(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_big();
        let b = unit.to_big();

        let q = a.div_euclid(b);
        let r = a.rem_euclid(b);

        // half = |b| / 2  (rounded up for tie-breaking away from zero)
        let abs_b = b.wrapping_abs();
        let two = DtBig::from_i128(2);
        let half = abs_b.wrapping_add(DtBig::ONE).wrapping_div(two);

        if r.ge(half) {
            // round away from zero
            let one = DtBig::ONE;
            let q_rounded = if a.is_negative() {
                q.wrapping_sub(one)
            } else {
                q.wrapping_add(one)
            };
            let result = q_rounded.wrapping_mul(b);
            Self::from_big(result)
        } else {
            let result = q.wrapping_mul(b);
            Self::from_big(result)
        }
    }

    /// Returns `floor(|self| / |unit|)` as `usize`, saturating at `usize::MAX`.
    ///
    /// Fully exact integer arithmetic using `DtBig`. Used by `TimeRange::len`.
    #[inline]
    pub const fn abs_div_floor(self, unit: Delta) -> usize {
        if unit.is_zero() {
            return 0;
        }
        let a = self.to_big().wrapping_abs();
        let b = unit.to_big().wrapping_abs();
        let q = a.div_euclid(b);

        match q.try_to_usize() {
            Ok(n) => n,
            Err(_) => usize::MAX,
        }
    }
}
