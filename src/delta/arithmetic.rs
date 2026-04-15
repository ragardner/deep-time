use crate::{Delta, DtBig, MICROQUECTOS_PER_SEC, POW15, POW21, Real, floor_f};

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
    #[inline(always)]
    pub const fn is_zero(self) -> bool {
        self.sec == 0 && self.subsec == 0
    }

    /// Converts this duration to a floating-point number of seconds.
    ///
    /// **Lossy by design** — returns the best possible `float` representation
    /// (≈15.95 decimal digits). Use `sec` + `subsec` (or `DtBig`) for full 36-digit precision.
    #[inline(always)]
    pub const fn as_sec_f(self) -> Real {
        // Extract the top 15 decimal digits exactly (POW15 is fully representable in float mantissa)
        let q = (self.subsec / POW21) as Real;
        let frac = q / f!(POW15);
        self.sec as Real + frac
    }

    /// Creates a `Delta` from a floating-point number of seconds.
    ///
    /// The result is normalized so the fractional part lies in `[0, 1)`.
    /// Negative values are handled correctly.
    ///
    /// **Precision note**: This implementation extracts the full ~15.95 decimal digits
    /// of precision that `float` can provide. It avoids the original loss caused by
    /// `MICROQUECTOS_PER_SEC` (10³⁶) not being exactly representable in `float`.
    /// We split the scaling into two exact steps: `10¹⁵` (safe in `float` mantissa)
    /// followed by an integer `10²¹` multiply.
    #[inline]
    pub const fn from_sec_f(sec_f: Real) -> Self {
        if sec_f.is_nan() {
            return Self::ZERO;
        } else if sec_f.is_infinite() {
            return if sec_f.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        } else {
            let floor_f = floor_f(sec_f);
            let frac = sec_f - floor_f;
            let high = (frac * (POW15 as Real)) as u128;
            Self {
                sec: floor_f as i128,
                subsec: high * POW21,
            }
        }
    }

    /// Divides this duration by 2 using the existing high-precision float path.
    #[inline(always)]
    pub fn div_by_2(self) -> Delta {
        Delta::from_sec_f(self.as_sec_f() / 2.0)
    }

    /// Exact division by a real number (used by Mars time conversions)
    #[inline(always)]
    pub const fn div_by_real(self, rhs: Real) -> Delta {
        Delta::from_sec_f(self.as_sec_f() / rhs)
    }

    /// Exact multiplication by a real number
    #[inline(always)]
    pub const fn mul_by_real(self, rhs: Real) -> Delta {
        Delta::from_sec_f(self.as_sec_f() * rhs)
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
