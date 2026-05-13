use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US, Drift, Dt, Spacetime, Real, Scale, floor_f,
};

impl Dt {
    #[inline]
    pub const fn add(self, span: Dt) -> Self {
        if !span.is_zero() {
            let (sec, attos) = Dt::add_time(self.sec, self.attos, span.sec, span.attos);
            Self { sec, attos }
        } else {
            self
        }
    }

    #[inline]
    pub const fn sub(self, span: Dt) -> Self {
        if !span.is_zero() {
            let (sec, attos) = Dt::sub_time(self.sec, self.attos, span.sec, span.attos);
            Self { sec, attos }
        } else {
            self
        }
    }

    /// Converts this `Dt` to a floating-point number of seconds since the reference epoch of its associated scale.
    ///
    /// The conversion is lossy by design, as [`Real`] provides approximately 15.95 decimal digits of precision.
    /// For full exactness, use the integer components `sec` and `attos` directly or higher-precision arithmetic when available.
    #[inline]
    pub const fn to_sec_f(&self) -> Real {
        f!(self.sec) + f!(self.attos) / ATTOS_PER_SECF
    }

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// derived from the supplied `Spacetime` model.
    ///
    /// This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// For the spacecraft's own hardware proper-time clock, use the plain `add` method instead.
    #[inline]
    pub const fn adjusted_advance(&mut self, elapsed: &Dt, spacetime: &Spacetime) {
        let dtau =
            elapsed.add(Drift::from_spacetime(spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// from a pre-computed `Drift` value.
    ///
    /// This is an optimized variant of `adjusted_advance` for callers that already hold a `Drift` instance.
    /// It is intended for simulation of remote clocks; the spacecraft's own hardware clock should use the plain `add` method.
    #[inline]
    pub const fn adjusted_advance_using_drift(&mut self, elapsed: &Dt, drift: &Drift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Computes the signed duration between this `Dt` and another `Dt`.
    #[inline]
    pub const fn to_diff_raw(&self, other: Self) -> Dt {
        Self::diff_raw_internal(self.sec, self.attos, other.sec, other.attos)
    }

    /// Computes the signed duration between this `Dt` and another `Dt` as a float.
    #[inline]
    pub const fn to_diff_raw_f(&self, other: Self) -> Real {
        self.to_sec_f() - other.to_sec_f()
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
        Self::add_attos_to(&mut self.sec, &mut self.attos, ATTOS_PER_MS);
    }

    /// Adds exactly 1 microsecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1us(&mut self) {
        Self::add_attos_to(&mut self.sec, &mut self.attos, ATTOS_PER_US);
    }

    /// Adds exactly 1 nanosecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1ns(&mut self) {
        Self::add_attos_to(&mut self.sec, &mut self.attos, ATTOS_PER_NS);
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
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, ATTOS_PER_MS);
    }

    /// Adds the specified number of microseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_us(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, ATTOS_PER_US);
    }

    /// Adds the specified number of nanoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ns(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, ATTOS_PER_NS);
    }

    /// Adds the specified number of picoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ps(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, ATTOS_PER_PS);
    }

    /// Adds the specified number of femtoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_fs(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, ATTOS_PER_FS);
    }

    /// Adds the specified number of attoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_attos(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n, 1);
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
        Self::add_attos_span(&mut self.sec, &mut self.attos, -1, ATTOS_PER_MS);
    }

    /// Subtracts exactly 1 microsecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1us(&mut self) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, -1, ATTOS_PER_US);
    }

    /// Subtracts exactly 1 nanosecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1ns(&mut self) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, -1, ATTOS_PER_NS);
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
        Self::add_attos_span(
            &mut self.sec,
            &mut self.attos,
            n.saturating_neg(),
            ATTOS_PER_MS,
        );
    }

    /// Subtracts the specified number of microseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_us(&mut self, n: i64) {
        Self::add_attos_span(
            &mut self.sec,
            &mut self.attos,
            n.saturating_neg(),
            ATTOS_PER_US,
        );
    }

    /// Subtracts the specified number of nanoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ns(&mut self, n: i64) {
        Self::add_attos_span(
            &mut self.sec,
            &mut self.attos,
            n.saturating_neg(),
            ATTOS_PER_NS,
        );
    }

    /// Subtracts the specified number of picoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ps(&mut self, n: i64) {
        Self::add_attos_span(
            &mut self.sec,
            &mut self.attos,
            n.saturating_neg(),
            ATTOS_PER_PS,
        );
    }

    /// Subtracts the specified number of femtoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_fs(&mut self, n: i64) {
        Self::add_attos_span(
            &mut self.sec,
            &mut self.attos,
            n.saturating_neg(),
            ATTOS_PER_FS,
        );
    }

    /// Subtracts the specified number of attoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_attos(&mut self, n: i64) {
        Self::add_attos_span(&mut self.sec, &mut self.attos, n.saturating_neg(), 1);
    }

    /// Total attoseconds (exact i128 representation within the representable range).
    #[inline]
    pub const fn to_attos(&self) -> i128 {
        (self.sec as i128) * ATTOS_PER_SEC_I128 + (self.attos as i128)
    }

    /// Returns the total time in milliseconds.
    #[inline]
    pub const fn to_ms(&self) -> i128 {
        self.to_attos() / (ATTOS_PER_MS as i128)
    }

    /// Returns the total time in microseconds.
    #[inline]
    pub const fn to_us(&self) -> i128 {
        self.to_attos() / (ATTOS_PER_US as i128)
    }

    /// Returns the total time in nanoseconds.
    #[inline]
    pub const fn to_ns(&self) -> i128 {
        self.to_attos() / (ATTOS_PER_NS as i128)
    }

    /// Returns the total time in picoseconds.
    #[inline]
    pub const fn to_ps(&self) -> i128 {
        self.to_attos() / (ATTOS_PER_PS as i128)
    }

    /// Returns the total time in femtoseconds.
    #[inline]
    pub const fn to_fs(&self) -> i128 {
        self.to_attos() / (ATTOS_PER_FS as i128)
    }

    /// Core saturating add for (sec, attos) pairs.
    pub(crate) const fn add_time(sec_a: i64, sub_a: u64, sec_b: i64, sub_b: u64) -> (i64, u64) {
        let mut sec = sec_a.saturating_add(sec_b);
        let mut attos = sub_a as i64 + sub_b as i64;

        if attos >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            attos -= ATTOS_PER_SEC as i64;
        } else if attos < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            attos += ATTOS_PER_SEC as i64;
        }

        let attos = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            attos as u64
        };

        (sec, attos)
    }

    /// Core saturating sub for (sec, attos) pairs.
    pub(crate) const fn sub_time(sec_a: i64, sub_a: u64, sec_b: i64, sub_b: u64) -> (i64, u64) {
        let mut sec = sec_a.saturating_sub(sec_b);
        let mut attos = sub_a as i64 - sub_b as i64;

        if attos < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            attos += ATTOS_PER_SEC as i64;
        } else if attos >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            attos -= ATTOS_PER_SEC as i64;
        }

        let attos = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            attos as u64
        };

        (sec, attos)
    }

    /// Returns `true` if this time is exactly zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.sec == 0 && self.attos == 0
    }

    /// Returns `true` if this time is strictly positive **(> 0)**.
    #[inline]
    pub const fn is_positive(&self) -> bool {
        if self.sec > 0 {
            true
        } else if self.sec == 0 {
            self.attos != 0
        } else {
            let k = (-self.sec) as u64;
            let quot = self.attos / ATTOS_PER_SEC;
            let rem = self.attos % ATTOS_PER_SEC;

            quot > k || (quot == k && rem > 0)
        }
    }

    /// Multiplies this time by an integer scalar (exact).
    ///
    /// Uses 128-bit arithmetic internally.
    pub const fn mul(self, rhs: i64) -> Self {
        if rhs == 0 || self.is_zero() {
            return Self::ZERO;
        }
        let total: i128 = self.to_attos().saturating_mul(rhs as i128);
        Self::from_attos(total, Scale::TAI)
    }

    /// Divides this time by an integer scalar (exact floor division).
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
        Self::from_attos(result, Scale::TAI)
    }

    /// Returns the **largest** multiple of `unit` that is ≤ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn floor(&self, unit: Self) -> Self {
        if unit.is_zero() {
            return *self;
        }
        let a = self.to_attos();
        let b = unit.to_attos();
        let q = a.div_euclid(b);
        let result = q.wrapping_mul(b);
        Self::from_attos(result, Scale::TAI)
    }

    /// Returns the **smallest** multiple of `unit` that is ≥ `self`.
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn ceil(&self, unit: Self) -> Self {
        if unit.is_zero() {
            return *self;
        }
        let a = self.to_attos();
        let b = unit.to_attos();
        // ceil(a/b) ≡ −floor(−a/b)
        let neg_a = a.wrapping_neg();
        let q = neg_a.div_euclid(b);
        let q_ceil = q.wrapping_neg();
        let result = q_ceil.wrapping_mul(b);
        Self::from_attos(result, Scale::TAI)
    }

    /// Returns the nearest multiple of `unit`.
    /// Halfway cases round **away from zero** (matches old `f64::round`).
    /// If `unit` is zero, returns `self` unchanged (exact, full precision).
    pub const fn round(&self, unit: Self) -> Self {
        if unit.is_zero() {
            return *self;
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
            Self::from_attos(result, Scale::TAI)
        } else {
            let result = q.wrapping_mul(b);
            Self::from_attos(result, Scale::TAI)
        }
    }

    /// Returns `floor(|self| / |unit|)` as `usize`, saturating at `usize::MAX`.
    ///
    /// Fully exact integer arithmetic using 128-bit intermediaries. Used by `TimeRange::len`.
    pub const fn abs_div_floor(&self, unit: Self) -> usize {
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
    pub const fn mul_by_f(&self, rhs: Real) -> Self {
        if rhs.is_nan() {
            return Self::ZERO;
        }
        if rhs.is_infinite() {
            if self.is_zero() {
                return Self::ZERO;
            }
            let self_pos = self.sec > 0 || (self.sec == 0 && self.attos != 0);
            return if (rhs > 0.0) == self_pos {
                Self::MAX
            } else {
                Self::MIN
            };
        }
        if self.is_zero() || rhs == 0.0 {
            return Self::ZERO;
        }

        let self_attos = self.to_attos();
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

        // Huge |rhs| → definitely saturates the type
        if int_part == i128::MAX || int_part == i128::MIN {
            let self_pos = self.sec > 0 || (self.sec == 0 && self.attos != 0);
            return if (rhs > 0.0) == self_pos {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let frac_part = rhs - f!(int_part); // always in [0, 1)

        // --- Integer part with explicit type-range saturation ---
        let int_attos = if int_part == 0 {
            0
        } else if int_part > 0 {
            if self_attos > 0 {
                if int_part > max_attos / self_attos {
                    max_attos
                } else {
                    self_attos * int_part
                }
            } else {
                let abs_self = self_attos.wrapping_neg();
                let abs_min = min_attos.wrapping_neg();
                if int_part > abs_min / abs_self {
                    min_attos
                } else {
                    self_attos * int_part
                }
            }
        } else {
            // int_part < 0
            if self_attos > 0 {
                let abs_int = int_part.wrapping_neg();
                let abs_min = min_attos.wrapping_neg();
                if abs_int > abs_min / self_attos {
                    min_attos
                } else {
                    self_attos * int_part
                }
            } else {
                let abs_self = self_attos.wrapping_neg();
                let abs_int = int_part.wrapping_neg();
                if abs_int > max_attos / abs_self {
                    max_attos
                } else {
                    self_attos * int_part
                }
            }
        };

        // --- Fractional part: decomposed exact computation (never overflows i128) ---
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

        // Combine + final clamp (manual version because clamp is not const yet)
        let total_attos = int_attos.saturating_add(frac_attos);
        let clamped = if total_attos > max_attos {
            max_attos
        } else if total_attos < min_attos {
            min_attos
        } else {
            total_attos
        };

        Self::from_attos(clamped, Scale::TAI)
    }

    /// Divides by a real number (routes through the high-precision `mul_by_f`).
    #[inline]
    pub const fn div_by_f(&self, rhs: Real) -> Self {
        if rhs == 0.0 || rhs.is_nan() {
            return if self.sec >= 0 { Self::MAX } else { Self::MIN };
        }
        self.mul_by_f(1.0 / rhs)
    }

    /// Divides this Dt by 2 (convenience wrapper).
    #[inline]
    pub const fn div_by_2(&self) -> Self {
        self.div_by_f(2.0)
    }

    /// Internal helper used by add_1ms / add_1us / add_1ns.
    #[doc(hidden)]
    pub(crate) const fn add_attos_to(sec: &mut i64, attos: &mut u64, amount: u64) {
        let total = *attos + amount;
        let carry_sec = total / ATTOS_PER_SEC;
        *attos = total % ATTOS_PER_SEC;
        *sec = sec.saturating_add(carry_sec as i64);
    }

    /// Internal method to add or subtract a subsecond span in a given unit.
    ///
    /// This is the core implementation for all subsecond addition and subtraction
    /// operations. It properly handles carry and borrow between the fractional
    /// part (`attos`) and the whole seconds (`sec`), using saturating arithmetic
    /// throughout.
    #[doc(hidden)]
    pub(crate) const fn add_attos_span(sec: &mut i64, attos: &mut u64, n: i64, unit: u64) {
        if n == 0 {
            return;
        }

        let mps = ATTOS_PER_SEC;

        if n >= 0 {
            let amount = (n as u64).saturating_mul(unit);
            let total = attos.saturating_add(amount);

            let carry = total / mps;
            let new_frac = total % mps;

            *sec = sec.saturating_add(carry as i64);
            *attos = new_frac;
        } else {
            let amount = n.unsigned_abs().saturating_mul(unit);
            let borrow_sec = amount / mps;
            let borrow_frac = amount % mps;

            *sec = sec.saturating_sub(borrow_sec as i64);

            if *attos >= borrow_frac {
                *attos -= borrow_frac;
            } else {
                *attos += mps - borrow_frac;
                *sec = sec.saturating_sub(1);
            }
        }

        // Final saturation clamp
        if *sec == i64::MAX {
            *attos = mps - 1;
        } else if *sec == i64::MIN {
            *attos = 0;
        }
    }

    /// Returns the total time in seconds.
    #[inline]
    pub const fn to_sec(&mut self) -> i64 {
        self.carry_over();
        self.sec
    }

    pub(crate) const fn diff_raw_internal(sec_a: i64, sub_a: u64, sec_b: i64, sub_b: u64) -> Self {
        if sub_a >= sub_b {
            Self {
                sec: sec_a.saturating_sub(sec_b),
                attos: sub_a - sub_b,
            }
        } else {
            Self {
                sec: sec_a.saturating_sub(sec_b).saturating_sub(1),
                attos: sub_a.saturating_add(ATTOS_PER_SEC.saturating_sub(sub_b)),
            }
        }
    }
}
