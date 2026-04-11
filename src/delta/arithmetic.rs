use crate::{Delta, MICROQUECTOS_PER_SEC, ceil_f64, floor_f64, round_f64};

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
    /// The returned `f64` has full integer precision up to the range of `i128`
    /// and fractional precision limited by the 53-bit mantissa of `f64`.
    #[inline]
    pub const fn as_sec_f64(self) -> f64 {
        self.sec as f64 + (self.subsec as f64 / MICROQUECTOS_PER_SEC as f64)
    }

    /// Creates a `Delta` from a floating-point number of seconds.
    ///
    /// The result is normalized so the fractional part lies in `[0, 1)`.
    /// Negative values are handled correctly.
    /// Creates a `Delta` from a floating-point number of seconds.
    pub const fn from_sec_f64(sec_f: f64) -> Self {
        let mut sec = floor_f64(sec_f) as i128;
        let mut frac = sec_f - (sec as f64);

        if frac < 0.0 {
            sec -= 1;
            frac += 1.0;
        }

        let subsec = (frac * MICROQUECTOS_PER_SEC as f64) as u128;

        Self { sec, subsec }
    }

    /// Returns the largest multiple of `unit` that is less than or equal to `self`.
    ///
    /// If `unit` is zero, returns `self` unchanged.
    #[inline]
    pub const fn floor(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let self_f = self.as_sec_f64();
        let unit_f = unit.as_sec_f64();
        let q = floor_f64(self_f / unit_f);
        Self::from_sec_f64(q * unit_f)
    }

    /// Returns the smallest multiple of `unit` that is greater than or equal to `self`.
    ///
    /// If `unit` is zero, returns `self` unchanged.
    #[inline]
    pub const fn ceil(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let self_f = self.as_sec_f64();
        let unit_f = unit.as_sec_f64();
        let q = ceil_f64(self_f / unit_f);
        Self::from_sec_f64(q * unit_f)
    }

    /// Returns the nearest multiple of `unit`.
    ///
    /// Halfway cases round away from zero (matching `f64::round` semantics).
    /// If `unit` is zero, returns `self` unchanged.
    #[inline]
    pub const fn round(self, unit: Delta) -> Delta {
        if unit.is_zero() {
            return self;
        }
        let self_f = self.as_sec_f64();
        let unit_f = unit.as_sec_f64();
        let q = round_f64(self_f / unit_f);
        Self::from_sec_f64(q * unit_f)
    }
}
