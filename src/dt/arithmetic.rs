use crate::{
    ATTOS_PER_FS_I128, ATTOS_PER_MS_I128, ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US_I128, Drift, Dt, Real, Spacetime, floor_f,
};

impl Dt {
    /// Saturating add, keeps `self`'s `scale` and `target`.
    #[inline]
    pub const fn add(&self, dt: Dt) -> Dt {
        if !dt.is_zero() {
            Dt::new(self.attos.saturating_add(dt.attos), self.scale, self.target)
        } else {
            *self
        }
    }

    /// Saturating sub, keeps `self`'s `scale` and `target`.
    #[inline]
    pub const fn sub(&self, dt: Dt) -> Dt {
        if !dt.is_zero() {
            Dt::new(self.attos.saturating_sub(dt.attos), self.scale, self.target)
        } else {
            *self
        }
    }

    /// If this time were turned into [`i128`] seconds and [`u64`] (always
    /// pushing to the positive) fractional attoseconds, this returns the
    /// whole seconds part.
    ///
    /// To just get seconds rounded to the nearest second use
    /// [`Dt::to_sec_rounded`](../struct.Dt.html#method.to_sec_rounded)
    /// instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // negative 1.3 seconds
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    ///
    /// // becomes positive 700ms
    /// let frac = dt.to_sec_ufrac();
    /// assert_eq!(frac, 700_000_000_000_000_000);
    ///
    /// // becomes negative 2 seconds
    /// let sec = dt.to_sec();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), 300_000_000_000_000_000);
    ///
    /// // if you just want rounded seconds
    /// // use to_sec_rounded() instead
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// let sec = dt.to_sec_rounded();
    /// assert_eq!(sec, -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec(&self) -> i128 {
        self.attos.div_euclid(ATTOS_PER_SEC_I128)
    }

    /// Returns this [`Dt`] rounded to the nearest whole second, then
    /// converted to an [`i128`] number of seconds.
    ///
    /// - Exactly halfway cases (e.g. 0.5 s, -0.5 s) round as follows:
    ///   0.5 becomes 1 and -0.5 becomes -1.
    /// - Matches the behavior of [`Dt::round`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// // 1.3 seconds → rounds to 1
    /// assert_eq!(Dt::span(1_300_000_000_000_000_000).to_sec_rounded(), 1);
    ///
    /// // -1.3 seconds → rounds to -1
    /// assert_eq!(Dt::span(-1_300_000_000_000_000_000).to_sec_rounded(), -1);
    ///
    /// // 1.6 seconds → rounds to 2
    /// assert_eq!(Dt::span(1_600_000_000_000_000_000).to_sec_rounded(), 2);
    ///
    /// // Halfway cases
    /// assert_eq!(Dt::span(500_000_000_000_000_000).to_sec_rounded(), 1);
    /// assert_eq!(Dt::span(-500_000_000_000_000_000).to_sec_rounded(), -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec_rounded(&self) -> i128 {
        self.round_to_sec().to_sec()
    }

    /// Returns this [`Dt`] rounded to the nearest whole second, then
    /// converted to an [`i64`] number of seconds.
    ///
    /// - Exactly halfway cases round as follows: 0.5 becomes 1 and -0.5 becomes -1,
    ///   same as [`to_sec_rounded`](Self::to_sec_rounded).
    /// - If the rounded value is outside the representable `i64` range,
    ///   it saturates to [`i64::MAX`] or [`i64::MIN`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64_rounded(), 1);
    ///
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64_rounded(), -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64_rounded(&self) -> i64 {
        Self::i128_to_i64(self.round_to_sec().to_sec())
    }

    /// If this time were turned into [`i64`] seconds and [`u64`] (always
    /// pushing to the positive) fractional attoseconds, this returns the
    /// whole seconds part.
    ///
    /// To just get seconds rounded to the nearest second use
    /// [`Dt::to_sec_rounded`](../struct.Dt.html#method.to_sec_rounded)
    /// instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // negative 1.3 seconds
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    ///
    /// // becomes positive 700ms
    /// let frac = dt.to_sec_ufrac();
    /// assert_eq!(frac, 700_000_000_000_000_000);
    ///
    /// // becomes negative 2 seconds
    /// let sec = dt.to_sec64();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec64(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), 300_000_000_000_000_000);
    ///
    /// // if you just want rounded seconds
    /// // use to_sec_rounded() instead
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// let sec = dt.to_sec_rounded();
    /// assert_eq!(sec, -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64(&self) -> i64 {
        Self::i128_to_i64(self.attos.div_euclid(ATTOS_PER_SEC_I128))
    }

    /// Converts this [`Dt`] to an f64 number of seconds since the reference
    /// epoch of its associated scale.
    ///
    /// - The conversion is lossy, as [`f64`] provides approximately 15.95 decimal
    ///   digits of precision.
    #[inline(always)]
    pub const fn to_f64(&self) -> f64 {
        self.to_sec_f()
    }

    /// Converts this [`Dt`] to a floating-point number of seconds since the reference
    /// epoch of its associated scale.
    ///
    /// - The conversion is lossy, as [`Real`] provides approximately 15.95 decimal
    ///   digits of precision.
    pub const fn to_sec_f(&self) -> Real {
        let attos = self.attos;

        if attos == 0 {
            return 0.0;
        }
        let sec = attos.div_euclid(ATTOS_PER_SEC_I128);
        let rem = attos.rem_euclid(ATTOS_PER_SEC_I128); // always in [0, aps)

        if sec < 0 && rem > ATTOS_PER_SEC_I128 / 2 {
            // original cancellation-avoidance path
            let small = ATTOS_PER_SEC_I128 - rem;
            let small_f = f!(small as u64) / ATTOS_PER_SECF;
            (sec as f64) + 1.0 - small_f
        } else {
            (sec as f64) + f!(rem as u64) / ATTOS_PER_SECF
        }
    }

    /// If this time were turned into seconds, this returns the fractional attoseconds part.
    #[inline(always)]
    pub const fn to_sec_frac(&self) -> i64 {
        (self.attos % ATTOS_PER_SEC_I128) as i64
    }

    /// If this time were turned into i64 seconds and u64 (always pushing to the positive)
    /// fractional attoseconds, this returns the fractional attoseconds part.
    ///
    /// - Always returns a value in the range `0 ≤ x < ATTOS_PER_SEC`.
    /// - For negative [`Dt`]s this is **not** simply the decimal part of the time in seconds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // negative 1.3 seconds
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    ///
    /// // becomes positive 700ms
    /// let frac = dt.to_sec_ufrac();
    /// assert_eq!(frac, 700_000_000_000_000_000);
    ///
    /// // becomes -2 seconds
    /// let sec = dt.to_sec64();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec64(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), 300_000_000_000_000_000);
    /// ```
    #[inline(always)]
    pub const fn to_sec_ufrac(&self) -> u64 {
        self.attos.rem_euclid(ATTOS_PER_SEC_I128) as u64
    }

    /// Returns a new [`Dt`] rounded to the nearest second.
    #[inline(always)]
    pub const fn round_to_sec(&self) -> Dt {
        self.round(Dt::span(ATTOS_PER_SEC_I128))
    }

    /// Returns the total time in minutes.
    #[inline(always)]
    pub const fn to_mins(&self) -> i128 {
        self.attos / (60 * ATTOS_PER_SEC_I128)
    }

    /// Returns the total time in hours.
    #[inline(always)]
    pub const fn to_hrs(&self) -> i128 {
        self.attos / (3600 * ATTOS_PER_SEC_I128)
    }

    /// Returns the total time in days.
    #[inline(always)]
    pub const fn to_days(&self) -> i128 {
        self.attos / (86400 * ATTOS_PER_SEC_I128)
    }

    /// Computes the signed duration between this [`Dt`] and another [`Dt`].
    #[inline]
    pub const fn to_diff_raw(&self, other: Dt) -> Dt {
        Dt::new(
            self.attos.saturating_sub(other.attos),
            self.scale,
            self.target,
        )
    }

    /// Computes the signed duration between this [`Dt`] and another [`Dt`] as a float.
    #[inline]
    pub const fn to_diff_raw_f(&self, other: Dt) -> Real {
        self.to_sec_f() - other.to_sec_f()
    }

    /// Low level constructor from total attoseconds since a given epoch.
    ///
    /// Simply adds the total attoseconds to the epoch. Does not perform
    /// any time scale conversions.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // A leap second from the middle of the table (36 leap seconds accumulated)
    /// let original = Dt::from_ymd(2015, 6, 30, Scale::UTC, 23, 59, 60, 123_456_789_000_000_000);
    ///
    /// // Round-trip through canonical attoseconds
    /// let canon = original.to_diff_raw(Dt::UNIX_EPOCH).to_attos();
    /// let roundtrip1 = Dt::from_diff_raw(canon, Dt::UNIX_EPOCH);
    ///
    /// assert_eq!(original, roundtrip1, "Canonical round-trip failed");
    /// ```
    #[inline]
    pub const fn from_diff_raw(attos: i128, epoch: Dt) -> Dt {
        epoch.add(Dt::new(attos, epoch.scale, epoch.target))
    }

    /// Adds the specified number of attoseconds to this time value.
    #[inline(always)]
    pub const fn add_attos(&self, n: i128) -> Dt {
        Dt::new(self.attos.saturating_add(n), self.scale, self.target)
    }

    /// Adds the specified number of seconds to this time value using saturating arithmetic.
    #[inline(always)]
    pub const fn add_sec(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_SEC_I128))
    }

    /// Adds the specified number of milliseconds to this time value.
    #[inline(always)]
    pub const fn add_ms(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_MS_I128))
    }

    /// Adds the specified number of microseconds to this time value.
    #[inline(always)]
    pub const fn add_us(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_US_I128))
    }

    /// Adds the specified number of nanoseconds to this time value.
    #[inline(always)]
    pub const fn add_ns(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_NS_I128))
    }

    /// Adds the specified number of picoseconds to this time value.
    #[inline(always)]
    pub const fn add_ps(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_PS_I128))
    }

    /// Adds the specified number of femtoseconds to this time value.
    #[inline(always)]
    pub const fn add_fs(&self, n: i128) -> Dt {
        self.add_attos(n.saturating_mul(ATTOS_PER_FS_I128))
    }

    /// Adds the specified number of minutes to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_min(&self, n: i64) -> Dt {
        Dt::new(
            self.attos
                .saturating_add((n as i128) * 60 * ATTOS_PER_SEC_I128),
            self.scale,
            self.target,
        )
    }

    /// Adds the specified number of hours to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_hr(&self, n: i64) -> Dt {
        Dt::new(
            self.attos
                .saturating_add((n as i128) * 3600 * ATTOS_PER_SEC_I128),
            self.scale,
            self.target,
        )
    }

    /// Returns the total time in attoseconds.
    #[inline(always)]
    pub const fn to_attos(&self) -> i128 {
        self.attos
    }

    /// Returns the total time in milliseconds.
    #[inline(always)]
    pub const fn to_ms(&self) -> i128 {
        self.attos / ATTOS_PER_MS_I128
    }

    /// Returns the total time in microseconds.
    #[inline(always)]
    pub const fn to_us(&self) -> i128 {
        self.attos / ATTOS_PER_US_I128
    }

    /// Returns the total time in nanoseconds.
    #[inline(always)]
    pub const fn to_ns(&self) -> i128 {
        self.attos / ATTOS_PER_NS_I128
    }

    /// Returns the total time in picoseconds.
    #[inline(always)]
    pub const fn to_ps(&self) -> i128 {
        self.attos / ATTOS_PER_PS_I128
    }

    /// Returns the total time in femtoseconds.
    #[inline(always)]
    pub const fn to_fs(&self) -> i128 {
        self.attos / ATTOS_PER_FS_I128
    }

    /// Returns `true` if this time is zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.attos == 0
    }

    /// Returns `true` if this time is strictly positive **> 0**.
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

    /// Returns the nearest multiple of `unit`.
    ///
    /// Halfway cases round **away from zero** (e.g. `2.5 → 3.0`, `-2.5 → -3.0`),
    /// matching the behavior of the old `f64::round()`.
    ///
    /// - If `unit` is zero, returns `self` unchanged (preserves full precision).
    /// - Uses Euclidean division internally for negative values.
    /// - The result is always a multiple of `unit`.
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

    /// Clamps an `i128` to the representable range of `i64`.
    #[inline(always)]
    pub(crate) const fn i128_to_i64(x: i128) -> i64 {
        let y = x as i64;
        if x == y as i128 {
            y
        } else if x > 0 {
            i64::MAX
        } else {
            i64::MIN
        }
    }

    /// Converts seconds i128 → total attoseconds i128
    #[inline(always)]
    pub const fn sec_to_attos(sec: i128) -> i128 {
        sec.saturating_mul(ATTOS_PER_SEC_I128)
    }

    /// Converts total attoseconds → whole seconds as i64
    #[inline(always)]
    pub const fn attos_to_sec_i64(attos: i128) -> i64 {
        Self::i128_to_i64(attos / ATTOS_PER_SEC_I128)
    }

    /// Clamps `value` to the range `[min, max]`.
    ///
    /// This is a `const fn`, so it can be used in const contexts
    /// (e.g. const generics, statics, const evaluation, etc.).
    ///
    /// If `min > max`, the result is equivalent to clamping to `[max, min]`.
    pub(crate) const fn clamp_u8(value: u8, min: u8, max: u8) -> u8 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Clamps `value` to the range `[min, max]`.
    ///
    /// This is a `const fn`, so it can be used in const contexts
    /// (e.g. const generics, statics, const evaluation, etc.).
    ///
    /// If `min > max`, the result is equivalent to clamping to `[max, min]`.
    pub(crate) const fn clamp_u64(value: u64, min: u64, max: u64) -> u64 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// **Lossy** conversion of u128 attoseconds to → float seconds (s).
    #[inline(always)]
    pub const fn attos_to_sec_f(attos: u128) -> Real {
        f!(attos) / ATTOS_PER_SECF
    }

    /// Converts i128 attoseconds → seconds (s)
    #[inline(always)]
    pub const fn attos_to_sec(attos: i128) -> i128 {
        attos / ATTOS_PER_SEC_I128
    }

    /// Converts i128 attoseconds → milliseconds (ms)
    #[inline(always)]
    pub const fn attos_to_ms(attos: i128) -> i128 {
        attos / ATTOS_PER_MS_I128
    }

    /// Converts i128 attoseconds → microseconds (us)
    #[inline(always)]
    pub const fn attos_to_us(attos: i128) -> i128 {
        attos / ATTOS_PER_US_I128
    }

    /// Converts i128 attoseconds → nanoseconds (ns)
    #[inline(always)]
    pub const fn attos_to_ns(attos: i128) -> i128 {
        attos / ATTOS_PER_NS_I128
    }

    /// Converts i128 attoseconds → picoseconds (ps)
    #[inline(always)]
    pub const fn attos_to_ps(attos: i128) -> i128 {
        attos / ATTOS_PER_PS_I128
    }

    /// Converts i128 attoseconds → femtoseconds (fs)
    #[inline(always)]
    pub const fn attos_to_fs(attos: i128) -> i128 {
        attos / ATTOS_PER_FS_I128
    }

    /// Returns the scalar ratio `self / rhs` expressed in seconds (as `Real`).
    ///
    /// This is the floating-point equivalent of `self.to_sec_f() / rhs.to_sec_f()`.
    ///
    /// # Special cases (chosen for safety and usability in time arithmetic)
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

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// derived from the supplied `Spacetime` model.
    ///
    /// - This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// - For a local hardware proper-time clock, use the plain `add` methods instead.
    #[inline]
    pub const fn adjusted_advance(&mut self, elapsed: &Dt, spacetime: &Spacetime) {
        let dtau = elapsed.add(Drift::from_spacetime(spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `Dt` by the given elapsed duration while applying the relativistic proper-time correction
    /// from a pre-computed `Drift` value.
    ///
    /// - This is an optimized variant of [`Dt::adjusted_advance`](../struct.Dt.html#method.adjusted_advance)
    ///   for callers that already hold a [`Drift`] instance.
    /// - This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// - For a local hardware proper-time clock, use the plain `add` methods instead.
    #[inline]
    pub const fn adjusted_advance_using_drift(&mut self, elapsed: &Dt, drift: &Drift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }
}
