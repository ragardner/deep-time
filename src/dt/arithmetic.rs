use crate::{
    ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128, ATTOS_PER_NS_I128,
    ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_US_I128, Dt, Real, floor_f,
};

impl Dt {
    /// Computes the signed duration between this [`Dt`] and another [`Dt`].
    ///
    /// Does **not** perform any time scale conversion.
    #[inline(always)]
    pub const fn to_diff_raw(&self, other: Dt) -> Dt {
        Dt::new(
            self.attos.saturating_sub(other.attos),
            self.scale,
            self.target,
        )
    }

    /// Computes the signed duration between this [`Dt`] and another [`Dt`] as a float.
    ///
    /// Does **not** perform any time scale conversion.
    #[inline(always)]
    pub const fn to_diff_raw_f(&self, other: Dt) -> Real {
        self.to_sec_f() - other.to_sec_f()
    }

    /// Saturating add, keeps `self`'s `scale` and `target`.
    ///
    /// Does **not** perform any time scale conversion.
    #[inline]
    pub const fn add(&self, dt: Dt) -> Dt {
        if !dt.is_zero() {
            Dt::new(self.attos.saturating_add(dt.attos), self.scale, self.target)
        } else {
            *self
        }
    }

    /// Saturating sub, keeps `self`'s `scale` and `target`.
    ///
    /// Does **not** perform any time scale conversion.
    #[inline]
    pub const fn sub(&self, dt: Dt) -> Dt {
        if !dt.is_zero() {
            Dt::new(self.attos.saturating_sub(dt.attos), self.scale, self.target)
        } else {
            *self
        }
    }

    /// Adds the specified number of attoseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_attos(-5);
    /// assert_eq!(sub_5.to_attos(), -5);
    /// ```
    #[inline(always)]
    pub const fn add_attos(&self, n: i128) -> Dt {
        Dt::new(self.attos.saturating_add(n), self.scale, self.target)
    }

    /// Adds the specified number of femtoseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_fs(-5);
    /// assert_eq!(sub_5.to_fs().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_fs(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_FS_I128))
    }

    /// Adds the specified number of picoseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_ps(-5);
    /// assert_eq!(sub_5.to_ps().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_ps(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_PS_I128))
    }

    /// Adds the specified number of nanoseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_ns(-5);
    /// assert_eq!(sub_5.to_ns().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_ns(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_NS_I128))
    }

    /// Adds the specified number of microseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_us(-5);
    /// assert_eq!(sub_5.to_us().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_us(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_US_I128))
    }

    /// Adds the specified number of milliseconds to this time value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_ms(-5);
    /// assert_eq!(sub_5.to_ms().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_ms(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_MS_I128))
    }

    /// Adds the specified number of seconds to this time value using saturating arithmetic.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_sec(-5);
    /// assert_eq!(sub_5.to_sec(), -5);
    /// ```
    #[inline(always)]
    pub const fn add_sec(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_SEC_I128))
    }

    /// Adds the specified number of minutes to this time value using saturating arithmetic.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_mins(-5);
    /// assert_eq!(sub_5.to_mins_floor().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_mins(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_MIN))
    }

    /// Adds the specified number of hours to this time value using saturating arithmetic.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::ZERO;
    /// let sub_5 = dt.add_hours(-5);
    /// assert_eq!(sub_5.to_hours_floor().0, -5);
    /// ```
    #[inline(always)]
    pub const fn add_hours(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_HOUR))
    }

    /// Returns `true` if this time is zero.
    ///
    /// Does **not** perform any time scale conversion.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.attos == 0
    }

    /// Returns `true` if this time is strictly positive **> 0**.
    ///
    /// Does **not** perform any time scale conversion.
    #[inline(always)]
    pub const fn is_positive(&self) -> bool {
        self.attos > 0
    }

    /// Multiplies this time by an integer scalar.
    ///
    /// Uses 128-bit arithmetic internally.
    pub const fn mul(self, rhs: i64) -> Dt {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total = self.attos.saturating_mul(rhs as i128);
        Dt::new(total, self.scale, self.target)
    }

    /// Divides this `Dt` by an integer scalar.
    ///
    /// Uses truncating division (rounds toward zero), same as normal integer division.
    /// Returns `ZERO` if `rhs == 0`.
    pub const fn div(self, rhs: i64) -> Dt {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let result = self.attos / (rhs as i128);
        Dt::new(result, self.scale, self.target)
    }

    /// Returns the **largest** multiple of `unit` that is ≤ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn floor(&self, unit: Dt) -> Dt {
        if unit.is_zero() {
            return *self;
        }
        let a = self.attos;
        let b = unit.attos;
        let q = safe_div_euc!(a, b, 0i128);
        let result = q.wrapping_mul(b);
        Dt::new(result, self.scale, self.target)
    }

    /// Returns the **smallest** multiple of `unit` that is ≥ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn ceil(&self, unit: Dt) -> Dt {
        if unit.is_zero() {
            return *self;
        }
        let a = self.attos;
        let b = unit.attos;
        // ceil(a/b) ≡ −floor(−a/b)
        let neg_a = a.wrapping_neg();
        let q = safe_div_euc!(neg_a, b, 0i128);
        let q_ceil = q.wrapping_neg();
        let result = q_ceil.wrapping_mul(b);
        Dt::new(result, self.scale, self.target)
    }

    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, TimeTraits};
    ///
    /// // Round to nearest second
    /// let dt = 1.3.sec();
    /// assert_eq!(dt.round(1.sec()), 1.sec());
    ///
    /// let dt = 1.6.sec();
    /// assert_eq!(dt.round(1.sec()), 2.sec());
    ///
    /// // Negative values
    /// let dt = (-1.3).sec();
    /// assert_eq!(dt.round(1.sec()), (-1).sec());
    ///
    /// // Halfway cases round *away from zero*
    /// assert_eq!(0.5.sec().round(1.sec()), 1.sec());
    /// assert_eq!((-0.5).sec().round(1.sec()), (-1).sec());
    ///
    /// assert_eq!(1.5.sec().round(1.sec()), 2.sec());
    /// assert_eq!((-1.5).sec().round(1.sec()), (-2).sec());
    ///
    /// // Round to nearest minute
    /// let dt = (1.mins() + 40.sec()).round(1.mins());
    /// assert_eq!(dt, 2.mins());
    ///
    /// // Round to nearest hour
    /// let dt = 1.6.hours().round(1.hours());
    /// assert_eq!(dt, 2.hours());
    /// ```
    pub const fn round(&self, unit: Dt) -> Dt {
        if unit.is_zero() {
            return *self;
        }

        let a = self.attos;
        let b = unit.attos;

        let abs_a = a.wrapping_abs();
        let abs_b = b.wrapping_abs();

        let q = safe_div_euc!(abs_a, abs_b, 0i128);
        let r = safe_rem_euc!(abs_a, abs_b, 0i128);

        let half = (abs_b + 1) / 2;

        let q_rounded = if r >= half { q + 1 } else { q };

        let rounded_abs = q_rounded.wrapping_mul(abs_b);

        let result = if a < 0 { -rounded_abs } else { rounded_abs };

        Dt::new(result, self.scale, self.target)
    }

    /// Returns `floor(|self| / |unit|)` as `usize`, saturating at `usize::MAX`.
    ///
    /// Fully exact integer arithmetic using 128-bit intermediaries. Used by `TimeRange::len`.
    pub const fn abs_div_floor(&self, unit: Dt) -> usize {
        if unit.is_zero() {
            return 0;
        }
        let a = self.attos.wrapping_abs();
        let b = unit.attos.wrapping_abs();
        let q = safe_div_euc!(a, b, 0i128);

        if q > (usize::MAX as i128) {
            usize::MAX
        } else {
            q as usize
        }
    }

    /// Multiplies this [`Dt`] by a floating-point scalar using saturating attosecond arithmetic.
    ///
    /// ## Algorithm
    ///
    /// - `rhs` is split into an **integer part** ([`floor_f`]) and a **fractional part** in `[0, 1)`.
    /// - The integer part is multiplied exactly via [`i128::checked_mul`], saturating to
    ///   [`Dt::MAX`] / [`Dt::MIN`] on overflow.
    /// - The fractional part is applied via a `10¹⁵`-scaled decomposition that avoids
    ///   intermediate `i128` overflow.
    /// - The two parts are combined with [`i128::saturating_add`] and clamped to the
    ///   representable attosecond range.
    ///
    /// ## Precision
    ///
    /// - Integer scalars (e.g. `2.0`, `-3.0`) use exact integer arithmetic for their whole part.
    /// - General `f64` scalars are limited by IEEE-754 precision (~15 decimal digits) and the
    ///   `10¹⁵` fractional quantization.
    ///
    /// ## Special cases
    ///
    /// | Condition | Result |
    /// |---|---|
    /// | `rhs` is NaN | [`Dt::ZERO`] |
    /// | `rhs` is ±∞ and `self` is zero | [`Dt::ZERO`] |
    /// | `rhs` is ±∞ and `self` is non-zero | [`Dt::MAX`] or [`Dt::MIN`] (sign of product) |
    /// | `rhs == 0.0` or `self` is zero | [`Dt::ZERO`] |
    /// | Product exceeds `i128` range | [`Dt::MAX`] or [`Dt::MIN`] (sign of product) |
    ///
    /// `NaN` maps to zero rather than poisoning the result: [`Dt`] has no NaN state, and zero
    /// is the additive identity (a safe, non-saturating default for invalid scale factors).
    pub const fn mul_by_f(&self, rhs: Real) -> Dt {
        if rhs.is_nan() {
            return Self::ZERO;
        }
        if rhs.is_infinite() {
            if self.is_zero() {
                return Self::ZERO;
            }
            let self_pos = self.attos > 0;
            return if (rhs > 0.0) == self_pos {
                Self::MAX
            } else {
                Self::MIN
            };
        }
        if self.is_zero() || rhs == 0.0 {
            return Self::ZERO;
        }

        let self_attos = self.attos;
        let max_attos = Self::MAX.to_attos();
        let min_attos = Self::MIN.to_attos();

        // Safe extraction of integer part (handles huge |rhs| without UB)
        let int_part = if rhs >= (i128::MAX as Real) {
            i128::MAX
        } else if rhs <= (i128::MIN as Real) {
            i128::MIN
        } else {
            floor_f(rhs) as i128
        };

        // Huge |rhs| integer → product cannot fit; saturate immediately.
        if int_part == i128::MAX || int_part == i128::MIN {
            let self_pos = self.attos > 0;
            return if (rhs > 0.0) == self_pos {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let frac_part = rhs - f!(int_part); // always in [0, 1)

        let int_attos = if int_part == 0 {
            0
        } else {
            Self::saturating_mul_attos(int_part, self_attos, max_attos, min_attos)
        };

        // Fractional part: decomposed exact computation (never overflows i128)
        const SCALE: i128 = 1_000_000_000_000_000; // 10¹⁵
        let frac_scaled = (frac_part * (SCALE as Real)) as i128;

        let frac_attos = if self_attos >= 0 {
            let high = self_attos / SCALE;
            let low = self_attos % SCALE;
            let high_part = high * frac_scaled;
            let low_part = (low * frac_scaled) / SCALE;
            high_part + low_part
        } else {
            let abs_self = self_attos.wrapping_neg();
            let high = abs_self / SCALE;
            let low = abs_self % SCALE;
            let high_part = high * frac_scaled;
            let low_part = (low * frac_scaled) / SCALE;
            let pos = high_part + low_part;
            pos.wrapping_neg()
        };

        let total_attos = int_attos.saturating_add(frac_attos);
        let clamped = if total_attos > max_attos {
            max_attos
        } else if total_attos < min_attos {
            min_attos
        } else {
            total_attos
        };

        Dt::new(clamped, self.scale, self.target)
    }

    /// `a * b` as attoseconds, saturating to `[min_attos, max_attos]` when not representable.
    #[inline(always)]
    pub(crate) const fn saturating_mul_attos(
        a: i128,
        b: i128,
        max_attos: i128,
        min_attos: i128,
    ) -> i128 {
        match a.checked_mul(b) {
            Some(product) => product,
            None => {
                let a_neg = a < 0;
                let b_neg = b < 0;
                if a_neg == b_neg { max_attos } else { min_attos }
            }
        }
    }

    /// Divides by a real number (routes through the high-precision `mul_by_f`).
    #[inline]
    pub const fn div_by_f(&self, rhs: Real) -> Dt {
        if rhs == 0.0 || rhs.is_nan() {
            return if self.attos >= 0 {
                Self::MAX
            } else {
                Self::MIN
            };
        }
        self.mul_by_f(1.0 / rhs)
    }

    /// Divides this Dt by 2 (convenience wrapper).
    #[inline]
    pub const fn div_by_2(&self) -> Dt {
        self.div_by_f(2.0)
    }

    /// Returns the scalar ratio `self / rhs` expressed in seconds (as `Real`).
    ///
    /// This is the floating-point equivalent of `self.to_sec_f() / rhs.to_sec_f()`.
    ///
    /// ## Special cases (chosen for safety and usability in time arithmetic)
    /// - `non-zero / ZERO` returns `±Real::INFINITY` (sign matches `self`)
    /// - `ZERO / non-zero` returns `0.0`
    /// - `ZERO / ZERO` returns `1.0` (the two durations are identical)
    ///
    /// These rules avoid `NaN` entirely while remaining predictable and useful
    /// in simulations, rate calculations, and control code.
    ///
    /// Negative durations are supported (e.g. `(-5 s) / (2 s) == -2.5`).
    ///
    /// This method is `const fn` and can be used in const contexts.
    #[inline]
    pub const fn div_dt(self, rhs: Dt) -> Real {
        let a = self.to_sec_f();
        let b = rhs.to_sec_f();

        if b == 0.0 {
            if a == 0.0 {
                1.0
            } else {
                Real::INFINITY.copysign(a)
            }
        } else {
            a / b
        }
    }
}
