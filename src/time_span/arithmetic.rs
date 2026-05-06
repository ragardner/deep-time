use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US, Real, TimePoint, TimeSpan, floor_f,
};

impl TimeSpan {
    /// Returns the sum of `self` and `rhs`, saturating at [`TimeSpan::MAX`] or
    /// [`TimeSpan::MIN`] on overflow.
    ///
    /// Both positive and negative `rhs` are supported.
    pub const fn add(self, rhs: Self) -> Self {
        let mut sec = self.sec.saturating_add(rhs.sec);
        let mut subsec = self.subsec as i64 + rhs.subsec as i64;

        if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self { sec, subsec }
    }

    /// Returns the difference `self - rhs`, saturating at [`TimeSpan::MAX`] or
    /// [`TimeSpan::MIN`] on overflow.
    ///
    /// Both positive and negative `rhs` are supported.
    pub const fn sub(self, rhs: Self) -> Self {
        let mut sec = self.sec.saturating_sub(rhs.sec);
        let mut subsec = self.subsec as i64 - rhs.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        } else if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self { sec, subsec }
    }

    /// Returns `true` if this duration is exactly zero.
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.sec == 0 && self.subsec == 0
    }

    /// Converts this duration to a floating-point number of seconds.
    /// It computes `sec + subsec / 10¹⁸` using `f64`.
    /// It is lossy by design (f64 only has ~15.95 decimal digits of precision).
    #[inline]
    pub const fn to_sec_f(self) -> Real {
        f!(self.sec) + f!(self.subsec) / ATTOS_PER_SECF
    }

    /// Multiplies this duration by an integer scalar (exact).
    ///
    /// Uses 128-bit arithmetic internally.
    pub const fn mul(self, rhs: i64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total: i128 = self.to_attos().saturating_mul(rhs as i128);
        Self::from_attos(total)
    }

    /// Divides this duration by an integer scalar (exact floor division).
    ///
    /// Returns `ZERO` if `rhs == 0`.
    /// Uses floor division (toward negative infinity) for consistency
    /// with the existing `floor` method.
    pub const fn div(self, rhs: i64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total = self.to_attos();
        let result = total.div_euclid(rhs as i128);
        Self::from_attos(result)
    }

    /// Returns the **largest** multiple of `unit` that is ≤ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn floor(self, unit: TimeSpan) -> TimeSpan {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_attos();
        let b = unit.to_attos();
        let q = a.div_euclid(b);
        let result = q.wrapping_mul(b);
        Self::from_attos(result)
    }

    /// Returns the **smallest** multiple of `unit` that is ≥ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn ceil(self, unit: TimeSpan) -> TimeSpan {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_attos();
        let b = unit.to_attos();
        // ceil(a/b) ≡ −floor(−a/b)
        let neg_a = a.wrapping_neg();
        let q = neg_a.div_euclid(b);
        let q_ceil = q.wrapping_neg();
        let result = q_ceil.wrapping_mul(b);
        Self::from_attos(result)
    }

    /// Returns the nearest multiple of `unit`.
    /// Halfway cases round **away from zero** (matches old `f64::round`).
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn round(self, unit: TimeSpan) -> TimeSpan {
        if unit.is_zero() {
            return self;
        }
        let a = self.to_attos();
        let b = unit.to_attos();

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
            Self::from_attos(result)
        } else {
            let result = q.wrapping_mul(b);
            Self::from_attos(result)
        }
    }

    /// Returns `floor(|self| / |unit|)` as `usize`, saturating at `usize::MAX`.
    ///
    /// Fully exact integer arithmetic using 128-bit intermediaries. Used by `TimeRange::len`.
    pub const fn abs_div_floor(self, unit: TimeSpan) -> usize {
        if unit.is_zero() {
            return 0;
        }
        let a = self.to_attos().wrapping_abs();
        let b = unit.to_attos().wrapping_abs();
        let q = a.div_euclid(b);

        if q > (usize::MAX as i128) {
            usize::MAX
        } else {
            q as usize
        }
    }

    /// - Integer part of `rhs` is multiplied **exactly** (pure i128 arithmetic).
    /// - Fractional part (|frac| < 1) uses the 10¹⁵ scaling.
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
        let frac_part = rhs - f!(int_part); // always in [0, 1)

        // Integer part
        let int_result = Self::from_attos(self.to_attos().saturating_mul(int_part));

        // Fractional part: scaling is safe (|frac_part| < 1)
        const SCALE: i128 = 1_000_000_000_000_000; // 10¹⁵
        let frac_scaled = (frac_part * (SCALE as Real)) as i128;
        let frac_product = self.to_attos().saturating_mul(frac_scaled);
        let frac_attos = frac_product / SCALE;
        let frac_result = Self::from_attos(frac_attos);

        int_result.add(frac_result)
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
    #[inline]
    pub const fn div_by_2(self) -> TimeSpan {
        self.div_by_f(2.0)
    }

    /// Adds exactly 1 second to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1sec(&mut self) {
        self.sec = self.sec.saturating_add(1);
    }

    /// Adds exactly 1 minute (60 seconds) to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1min(&mut self) {
        self.sec = self.sec.saturating_add(60);
    }

    /// Adds exactly 1 hour (3600 seconds) to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1hr(&mut self) {
        self.sec = self.sec.saturating_add(3600);
    }

    /// Adds exactly 1 millisecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1ms(&mut self) {
        self._add_subsec(ATTOS_PER_MS);
    }

    /// Adds exactly 1 microsecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1us(&mut self) {
        self._add_subsec(ATTOS_PER_US);
    }

    /// Adds exactly 1 nanosecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1ns(&mut self) {
        self._add_subsec(ATTOS_PER_NS);
    }

    /// Adds the specified number of seconds to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_sec(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n);
    }

    /// Adds the specified number of minutes to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_min(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n.saturating_mul(60));
    }

    /// Adds the specified number of hours to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_hr(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n.saturating_mul(3600));
    }

    /// Adds the specified number of milliseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ms(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_MS);
    }

    /// Adds the specified number of microseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_us(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_US);
    }

    /// Adds the specified number of nanoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ns(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_NS);
    }

    /// Adds the specified number of picoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ps(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_PS);
    }

    /// Adds the specified number of femtoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_fs(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_FS);
    }

    /// Adds the specified number of attoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_attos(&mut self, n: i64) {
        self.add_subsec_span(n, 1);
    }

    /// Subtracts exactly 1 hour (3600 seconds) from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1hr(&mut self) {
        self.sec = self.sec.saturating_sub(3600);
    }

    /// Subtracts exactly 1 minute (60 seconds) from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1min(&mut self) {
        self.sec = self.sec.saturating_sub(60);
    }

    /// Subtracts exactly 1 second from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1sec(&mut self) {
        self.sec = self.sec.saturating_sub(1);
    }

    /// Subtracts exactly 1 millisecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1ms(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_MS);
    }

    /// Subtracts exactly 1 microsecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1us(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_US);
    }

    /// Subtracts exactly 1 nanosecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1ns(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_NS);
    }

    /// Subtracts the specified number of seconds from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_sec(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n);
    }

    /// Subtracts the specified number of minutes from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_min(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n.saturating_mul(60));
    }

    /// Subtracts the specified number of hours from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_hr(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n.saturating_mul(3600));
    }

    /// Subtracts the specified number of milliseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ms(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_MS);
    }

    /// Subtracts the specified number of microseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_us(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_US);
    }

    /// Subtracts the specified number of nanoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ns(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_NS);
    }

    /// Subtracts the specified number of picoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ps(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_PS);
    }

    /// Subtracts the specified number of femtoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_fs(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_FS);
    }

    /// Subtracts the specified number of attoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_attos(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), 1);
    }

    /// Internal method to add or subtract a subsecond span in a given unit.
    ///
    /// This is the core implementation for all subsecond addition and subtraction
    /// operations. It properly handles carry and borrow between the fractional
    /// part (`subsec`) and the whole seconds (`sec`), using saturating arithmetic
    /// throughout.
    #[doc(hidden)]
    const fn add_subsec_span(&mut self, n: i64, unit: u64) {
        if n == 0 {
            return;
        }

        let mps = ATTOS_PER_SEC;

        if n >= 0 {
            // Positive direction
            let amount = (n as u64).saturating_mul(unit);
            let total = self.subsec.saturating_add(amount);

            let carry = total / mps;
            let new_frac = total % mps;

            self.sec = self.sec.saturating_add(carry as i64);
            self.subsec = new_frac;
        } else {
            // Negative direction
            let amount = n.unsigned_abs().saturating_mul(unit);
            let borrow_sec = amount / mps;
            let borrow_frac = amount % mps;

            self.sec = self.sec.saturating_sub(borrow_sec as i64);

            if self.subsec >= borrow_frac {
                self.subsec -= borrow_frac;
            } else {
                self.subsec += mps - borrow_frac;
                self.sec = self.sec.saturating_sub(1);
            }
        }

        // Final saturation clamp to maintain invariants at extreme values
        if self.sec == i64::MAX {
            self.subsec = mps - 1;
        } else if self.sec == i64::MIN {
            self.subsec = 0;
        }
    }

    /// Internal fast-path method for adding a small positive subsecond amount.
    ///
    /// Used by the single-unit `add_1ms`, `add_1us`, and `add_1ns` methods.
    /// This is intentionally simpler and faster than the general `add_subsec_span`
    /// when the span is known to be positive and small.
    #[doc(hidden)]
    #[inline]
    const fn _add_subsec(&mut self, amount: u64) {
        let total = self.subsec + amount;
        let carry_sec = total / ATTOS_PER_SEC;
        self.subsec = total % ATTOS_PER_SEC;
        self.sec = self.sec.saturating_add(carry_sec as i64);
    }

    /// Total attoseconds (exact i128 representation within the representable range).
    #[inline]
    pub const fn to_attos(self) -> i128 {
        (self.sec as i128) * ATTOS_PER_SEC_I128 + (self.subsec as i128)
    }

    /// Returns the total duration in seconds.
    #[inline]
    pub const fn to_sec(&mut self) -> i64 {
        self.carry_over();
        self.sec
    }

    /// Returns the total duration in milliseconds.
    #[inline]
    pub const fn to_ms(self) -> i128 {
        self.to_attos() / (ATTOS_PER_MS as i128)
    }

    /// Returns the total duration in microseconds.
    #[inline]
    pub const fn to_us(self) -> i128 {
        self.to_attos() / (ATTOS_PER_US as i128)
    }

    /// Returns the total duration in nanoseconds.
    #[inline]
    pub const fn to_ns(self) -> i128 {
        self.to_attos() / (ATTOS_PER_NS as i128)
    }

    /// Returns the total duration in picoseconds.
    #[inline]
    pub const fn to_ps(self) -> i128 {
        self.to_attos() / (ATTOS_PER_PS as i128)
    }

    /// Returns the total duration in femtoseconds.
    #[inline]
    pub const fn to_fs(self) -> i128 {
        self.to_attos() / (ATTOS_PER_FS as i128)
    }

    /// Returns `self - rhs` exactly.
    ///
    /// This is the normal case when subtracting two durations.
    #[inline]
    pub const fn to_diff(self, rhs: Self) -> Self {
        Self::diff_raw(self.sec, self.subsec, rhs.sec, rhs.subsec)
    }

    /// Returns `self - rhs` exactly, where `rhs` is a `TimePoint`.
    #[inline]
    pub const fn to_diff_tp(self, rhs: TimePoint) -> Self {
        Self::diff_raw(self.sec, self.subsec, rhs.sec, rhs.subsec)
    }

    #[inline]
    pub(crate) const fn diff_raw(sec_a: i64, sub_a: u64, sec_b: i64, sub_b: u64) -> Self {
        let mut sec = sec_a - sec_b;
        let mut subsec = sub_a;

        if subsec >= sub_b {
            subsec -= sub_b;
        } else {
            sec -= 1;
            subsec += ATTOS_PER_SEC - sub_b;
        }

        Self { sec, subsec }
    }
}
