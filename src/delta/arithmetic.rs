use crate::{ATTOSEC_PER_SEC, ATTOSEC_PER_SEC_I128, Delta, Real, floor_f};

impl Delta {
    /// Returns the sum of `self` and `rhs`.
    ///
    /// The result is normalized so the fractional part lies in `[0, ATTOSEC_PER_SEC)`.
    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        let mut sec = self.sec + rhs.sec;
        let mut subsec = self.subsec + rhs.subsec;

        if subsec >= ATTOSEC_PER_SEC {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC;
        }

        Self { sec, subsec }
    }

    /// Returns the difference `self - rhs`.
    ///
    /// The result is normalized so the fractional part lies in `[0, ATTOSEC_PER_SEC)`.
    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        let mut sec = self.sec - rhs.sec;
        let mut subsec = self.subsec;

        if subsec >= rhs.subsec {
            subsec -= rhs.subsec;
        } else {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC - rhs.subsec;
        }

        Self { sec, subsec }
    }

    /// Returns the sum of `self` and `rhs`, saturating at [`Delta::MAX`] or [`Delta::MIN`] on overflow.
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        let mut subsec = self.subsec + rhs.subsec;
        let mut carry = 0i64;
        if subsec >= ATTOSEC_PER_SEC {
            subsec -= ATTOSEC_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(rhs.sec).saturating_add(carry);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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
        let mut borrow = 0i64;
        if subsec >= rhs.subsec {
            subsec -= rhs.subsec;
        } else {
            subsec += ATTOSEC_PER_SEC - rhs.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(rhs.sec).saturating_sub(borrow);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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

    /// Reconstruct `Delta` from total attoseconds (exact, handles negative values correctly).
    #[inline]
    pub const fn from_total_attos(mut attos: i128) -> Self {
        if attos > (i64::MAX as i128) * ATTOSEC_PER_SEC_I128 {
            return Self::MAX;
        }
        if attos < (i64::MIN as i128) * ATTOSEC_PER_SEC_I128 {
            return Self::MIN;
        }

        if attos >= 0 {
            let sec = (attos / ATTOSEC_PER_SEC_I128) as i64;
            let subsec = (attos % ATTOSEC_PER_SEC_I128) as u64;
            Self { sec, subsec }
        } else {
            attos = -attos;
            let sec_pos = (attos / ATTOSEC_PER_SEC_I128) as i64;
            let rem = (attos % ATTOSEC_PER_SEC_I128) as u64;
            if rem == 0 {
                Self {
                    sec: -sec_pos,
                    subsec: 0,
                }
            } else {
                Self {
                    sec: -sec_pos - 1,
                    subsec: ATTOSEC_PER_SEC - rem,
                }
            }
        }
    }

    /// Converts this duration to a floating-point number of seconds.
    /// It computes `sec + subsec / 10¹⁸` using `f64`.
    /// It is lossy by design (f64 only has ~15.95 decimal digits of precision).
    #[inline]
    pub const fn as_sec_f(self) -> Real {
        self.sec as Real + (self.subsec as Real) / (ATTOSEC_PER_SEC as Real)
    }

    /// Creates a `Delta` from a floating-point number of seconds.
    #[inline]
    pub const fn from_sec_f(sec_f: Real) -> Self {
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

        let floor_val = floor_f(sec_f);
        let frac = sec_f - floor_val;
        let attos_frac = (frac * (ATTOSEC_PER_SEC as Real)) as i128;

        let total = (floor_val as i128) * ATTOSEC_PER_SEC_I128 + attos_frac;
        Self::from_total_attos(total)
    }

    /// Multiplies this duration by an integer scalar (exact).
    ///
    /// Uses 128-bit arithmetic internally.
    #[inline]
    pub const fn mul(self, rhs: i64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total = self.total_attos() * (rhs as i128);
        Self::from_total_attos(total)
    }

    /// Divides this duration by an integer scalar (exact floor division).
    ///
    /// Returns `ZERO` if `rhs == 0`.
    /// Uses floor division (toward negative infinity) for consistency
    /// with the existing `floor` method.
    #[inline]
    pub const fn div(self, rhs: i64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total = self.total_attos();
        let result = total.div_euclid(rhs as i128);
        Self::from_total_attos(result)
    }

    /// Returns the **largest** multiple of `unit` that is ≤ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn floor(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.total_attos();
        let b = unit.total_attos();
        let q = a.div_euclid(b);
        let result = q.wrapping_mul(b);
        Self::from_total_attos(result)
    }

    /// Returns the **smallest** multiple of `unit` that is ≥ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn ceil(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.total_attos();
        let b = unit.total_attos();
        // ceil(a/b) ≡ −floor(−a/b)
        let neg_a = a.wrapping_neg();
        let q = neg_a.div_euclid(b);
        let q_ceil = q.wrapping_neg();
        let result = q_ceil.wrapping_mul(b);
        Self::from_total_attos(result)
    }

    /// Returns the nearest multiple of `unit`.
    /// Halfway cases round **away from zero** (matches old `f64::round`).
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    #[inline]
    pub const fn round(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let a = self.total_attos();
        let b = unit.total_attos();

        let q = a.div_euclid(b);
        let r = a.rem_euclid(b);

        // half = |b| / 2  (rounded up for tie-breaking away from zero)
        let abs_b = b.wrapping_abs();
        let two = 2i128;
        let half = (abs_b + 1) / two;

        if r >= half {
            // round away from zero
            let one = 1i128;
            let q_rounded = if a < 0 { q - one } else { q + one };
            let result = q_rounded.wrapping_mul(b);
            Self::from_total_attos(result)
        } else {
            let result = q.wrapping_mul(b);
            Self::from_total_attos(result)
        }
    }

    /// Returns `floor(|self| / |unit|)` as `usize`, saturating at `usize::MAX`.
    ///
    /// Fully exact integer arithmetic using 128-bit intermediaries. Used by `TimeRange::len`.
    #[inline]
    pub const fn abs_div_floor(self, unit: Delta) -> usize {
        if unit.is_zero() {
            return 0;
        }
        let a = self.total_attos().wrapping_abs();
        let b = unit.total_attos().wrapping_abs();
        let q = a.div_euclid(b);

        if q > (usize::MAX as i128) {
            usize::MAX
        } else {
            q as usize
        }
    }

    /// - Integer part of `rhs` is multiplied **exactly** (pure i128 arithmetic).
    /// - Fractional part (|frac| < 1) uses the 10¹⁵ scaling.
    #[inline]
    pub const fn mul_by_f(self, rhs: Real) -> Self {
        if rhs.is_nan() {
            return Self::ZERO;
        }
        if rhs.is_infinite() {
            if self.is_zero() {
                return Self::ZERO;
            }
            let self_pos = self.sec > 0 || (self.sec == 0 && self.subsec != 0);
            return if (rhs > 0.0) == self_pos {
                Self::MAX
            } else {
                Self::MIN
            };
        }
        if self.is_zero() || rhs == 0.0 {
            return Self::ZERO;
        }

        let int_part = floor_f(rhs) as i128; // exact integer part
        let frac_part = rhs - (int_part as Real); // always in [0, 1)

        // Integer part: fully exact i128 multiply
        let int_result = Self::from_total_attos(self.total_attos() * int_part);

        // Fractional part: scaling is now 100% safe (|frac_part| < 1)
        const SCALE: i128 = 1_000_000_000_000_000; // 10¹⁵
        let frac_scaled = (frac_part * (SCALE as Real)) as i128;
        let frac_product = self.total_attos() * frac_scaled;
        let frac_attos = frac_product / SCALE;
        let frac_result = Self::from_total_attos(frac_attos);

        int_result.saturating_add(frac_result)
    }

    /// Divides by a real number (routes through the high-precision `mul_by_f`).
    #[inline]
    pub const fn div_by_f(self, rhs: Real) -> Self {
        if rhs == 0.0 || rhs.is_nan() {
            return if self.sec >= 0 { Self::MAX } else { Self::MIN };
        }
        self.mul_by_f(1.0 / rhs)
    }

    /// Divides this duration by 2 (convenience wrapper).
    #[inline(always)]
    pub fn div_by_2(self) -> Delta {
        self.div_by_f(2.0)
    }
}
