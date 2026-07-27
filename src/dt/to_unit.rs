use crate::{
    ATTOS_PER_DAY, ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128,
    ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_SECF, ATTOS_PER_US_I128,
    ATTOS_PER_WEEK, Dt, Real, dt,
};

impl Dt {
    /// Converts total attoseconds to a floating-point count of
    /// `unit_attos`-sized units.
    ///
    /// Euclidean split keeps the remainder in `[0, unit_attos)`. For large
    /// negative whole parts with a large remainder, rewrites as
    /// `whole + 1 − small_frac` to avoid cancellation in the float add.
    #[inline(always)]
    const fn attos_to_unit_f(attos: i128, unit_attos: i128) -> Real {
        if attos == 0 {
            return 0.0;
        }
        let whole = attos.div_euclid(unit_attos);
        let rem = attos.rem_euclid(unit_attos);

        if whole < 0 && rem > unit_attos / 2 {
            let small = unit_attos - rem;
            let small_f = f!(small as u128) / f!(unit_attos);
            f!(whole) + 1.0 - small_f
        } else {
            f!(whole) + f!(rem as u128) / f!(unit_attos)
        }
    }

    /// Returns this duration as a whole number of seconds, dropping any
    /// fraction (always toward zero: 1.7 → 1, −1.7 → −1).
    ///
    /// Same as `self.attos / ATTOS_PER_SEC_I128`. Does not round.
    ///
    /// On positive values this matches
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor).
    /// On negatives they diverge: here −1.3 → −1, while `to_sec_floor` gives
    /// −2 (always rounds down). Prefer `to_sec_floor` when you also need a
    /// non-negative fractional part via
    /// [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// // Fraction only — less than one second either way → 0
    /// assert_eq!(from_sec!(0, -ms!(300)).to_sec(), 0);
    /// assert_eq!(from_sec!(0, ms!(300)).to_sec(), 0);
    ///
    /// // -1.3 s → -1 here, but -2 with `to_sec_floor`
    /// let dt = from_sec!(-1, -ms!(300));
    /// assert_eq!(dt.to_sec(), -1);
    /// assert_eq!(dt.to_sec_floor(), -2);
    ///
    /// // Positive values match `to_sec_floor`
    /// let dt = from_sec!(1, ms!(300));
    /// assert_eq!(dt.to_sec(), 1);
    /// assert_eq!(dt.to_sec_floor(), 1);
    /// ```
    #[inline(always)]
    pub const fn to_sec(&self) -> i128 {
        self.attos / ATTOS_PER_SEC_I128
    }

    /// Same as [`to_sec`](../struct.Dt.html#method.to_sec), but returns an
    /// [`i64`].
    ///
    /// Values outside the `i64` range clamp to [`i64::MAX`] or [`i64::MIN`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// // Same toward-zero rule as `to_sec`
    /// assert_eq!(from_sec!(1, ms!(300)).to_sec64(), 1);
    /// assert_eq!(from_sec!(-1, -ms!(300)).to_sec64(), -1);
    ///
    /// // Fits in i64 here; huge values would clamp instead of panicking
    /// assert_eq!(from_sec!(i64::MAX as i128).to_sec64(), i64::MAX);
    /// ```
    #[inline(always)]
    pub const fn to_sec64(&self) -> i64 {
        Self::to_i64(self.attos / ATTOS_PER_SEC_I128)
    }

    /// Returns whole seconds, always rounding down (toward −∞).
    ///
    /// So 1.3 → 1 and −1.3 → −2. The leftover attoseconds are then always
    /// ≥ 0; get them with
    /// [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    ///
    /// For truncation toward zero (1.3 → 1, −1.3 → −1), use
    /// [`to_sec`](../struct.Dt.html#method.to_sec).
    /// For nearest-second rounding, use
    /// [`to_sec_round`](../struct.Dt.html#method.to_sec_round).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// // -1.3 s → whole -2, leftover +0.7 s
    /// let dt = from_sec!(-1, -ms!(300));
    /// assert_eq!(dt.to_sec_floor(), -2);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(700) as u64);
    ///
    /// // +1.3 s → whole 1, leftover +0.3 s
    /// let dt = from_sec!(1, ms!(300));
    /// assert_eq!(dt.to_sec_floor(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(300) as u64);
    /// ```
    #[inline(always)]
    pub const fn to_sec_floor(&self) -> i128 {
        self.attos.div_euclid(ATTOS_PER_SEC_I128)
    }

    /// Same as [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor),
    /// but returns an [`i64`].
    ///
    /// Values outside the `i64` range clamp to [`i64::MAX`] or [`i64::MIN`].
    /// Pair with [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac)
    /// for the leftover.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// let dt = from_sec!(-1, -ms!(300));
    /// assert_eq!(dt.to_sec64_floor(), -2);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(700) as u64);
    ///
    /// let dt = from_sec!(1, ms!(300));
    /// assert_eq!(dt.to_sec64_floor(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(300) as u64);
    /// ```
    #[inline(always)]
    pub const fn to_sec64_floor(&self) -> i64 {
        Self::to_i64(self.attos.div_euclid(ATTOS_PER_SEC_I128))
    }

    /// Rounds to the nearest whole second and returns that count as [`i128`].
    ///
    /// Halfway cases go away from zero: 0.5 → 1 and −0.5 → −1
    /// (same rule as [`Dt::round`]).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// assert_eq!(from_sec!(1, ms!(300)).to_sec_round(), 1);
    /// assert_eq!(from_sec!(1, ms!(600)).to_sec_round(), 2);
    /// assert_eq!(from_sec!(-1, -ms!(300)).to_sec_round(), -1);
    ///
    /// // Halfway: away from zero
    /// assert_eq!(from_sec!(0, ms!(500)).to_sec_round(), 1);
    /// assert_eq!(from_sec!(0, -ms!(500)).to_sec_round(), -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec_round(&self) -> i128 {
        self.round_to_sec().to_sec()
    }

    /// Same as [`to_sec_round`](../struct.Dt.html#method.to_sec_round),
    /// but returns an [`i64`].
    ///
    /// Values outside the `i64` range clamp to [`i64::MAX`] or [`i64::MIN`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// assert_eq!(from_sec!(1, ms!(300)).to_sec64_round(), 1);
    /// assert_eq!(from_sec!(1, ms!(600)).to_sec64_round(), 2);
    /// assert_eq!(from_sec!(-1, -ms!(300)).to_sec64_round(), -1);
    /// assert_eq!(from_sec!(0, ms!(500)).to_sec64_round(), 1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64_round(&self) -> i64 {
        Self::to_i64(self.round_to_sec().to_sec())
    }

    /// Converts this duration to seconds as an [`f64`].
    ///
    /// Alias of [`to_sec_f`](../struct.Dt.html#method.to_sec_f).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// assert_eq!(Dt::ZERO.to_f64(), 0.0);
    /// assert!((from_sec!(1, ms!(500)).to_f64() - 1.5).abs() < 1e-12);
    /// ```
    #[inline(always)]
    pub const fn to_f64(&self) -> f64 {
        self.to_sec_f()
    }

    /// Converts this duration to seconds as a floating-point number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// assert_eq!(Dt::ZERO.to_sec_f(), 0.0);
    /// assert!((from_sec!(1, ms!(500)).to_sec_f() - 1.5).abs() < 1e-12);
    /// assert!((from_sec!(-1, -ms!(500)).to_sec_f() + 1.5).abs() < 1e-12);
    /// ```
    pub const fn to_sec_f(&self) -> Real {
        if self.attos == 0 {
            return 0.0;
        }
        let sec = self.attos.div_euclid(ATTOS_PER_SEC_I128);
        let rem = self.attos.rem_euclid(ATTOS_PER_SEC_I128); // always in [0, aps)

        if sec < 0 && rem > ATTOS_PER_SEC_I128 / 2 {
            // original cancellation-avoidance path
            let small = ATTOS_PER_SEC_I128 - rem;
            let small_f = f!(small as u64) / ATTOS_PER_SECF;
            f!(sec) + 1.0 - small_f
        } else {
            f!(sec) + f!(rem as u64) / ATTOS_PER_SECF
        }
    }

    /// Returns the signed leftover attoseconds after removing whole seconds
    /// toward zero.
    ///
    /// Same sign as the original value when non-zero: 1.3 s → `+0.3 s` in
    /// attoseconds, −1.3 s → `−0.3 s` in attoseconds. Pairs with
    /// [`from_sec_and_frac`](../struct.Dt.html#method.from_sec_and_frac)
    /// and [`to_sec`](../struct.Dt.html#method.to_sec).
    ///
    /// For a leftover that is always ≥ 0 (paired with floor), use
    /// [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// let dt = from_sec!(1, ms!(300));
    /// assert_eq!(dt.to_sec(), 1);
    /// assert_eq!(dt.to_sec_frac(), ms!(300) as i64);
    ///
    /// let dt = from_sec!(-1, -ms!(300));
    /// assert_eq!(dt.to_sec(), -1);
    /// assert_eq!(dt.to_sec_frac(), -ms!(300) as i64);
    /// ```
    #[inline(always)]
    pub const fn to_sec_frac(&self) -> i64 {
        (self.attos % ATTOS_PER_SEC_I128) as i64
    }

    /// Returns the leftover attoseconds after
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor) /
    /// [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor).
    ///
    /// Always in `0 .. ATTOS_PER_SEC`. On negatives this is **not** “the
    /// decimal part with a sign”: −1.3 s floors to −2 whole seconds with a
    /// **+0.7 s** leftover.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// // -1.3 s → floor -2 s + 0.7 s leftover
    /// let dt = from_sec!(-1, -ms!(300));
    /// assert_eq!(dt.to_sec64_floor(), -2);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(700) as u64);
    ///
    /// // +1.3 s → floor 1 s + 0.3 s leftover
    /// let dt = from_sec!(1, ms!(300));
    /// assert_eq!(dt.to_sec64_floor(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), ms!(300) as u64);
    /// ```
    #[inline(always)]
    pub const fn to_sec_ufrac(&self) -> u64 {
        self.attos.rem_euclid(ATTOS_PER_SEC_I128) as u64
    }

    /// Returns a new [`Dt`] rounded to the nearest whole second.
    ///
    /// Halfway cases go away from zero (same rule as [`Dt::round`]). For the
    /// rounded value as an integer count, see
    /// [`to_sec_round`](../struct.Dt.html#method.to_sec_round).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms};
    ///
    /// assert_eq!(from_sec!(1, ms!(300)).round_to_sec(), from_sec!(1));
    /// assert_eq!(from_sec!(1, ms!(600)).round_to_sec(), from_sec!(2));
    /// assert_eq!(from_sec!(0, ms!(500)).round_to_sec(), from_sec!(1));
    /// assert_eq!(from_sec!(0, -ms!(500)).round_to_sec(), from_sec!(-1));
    /// ```
    #[inline(always)]
    pub const fn round_to_sec(&self) -> Dt {
        self.round(dt!(ATTOS_PER_SEC_I128))
    }

    /// Returns the total time in attoseconds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, ms, sec};
    ///
    /// assert_eq!(Dt::ZERO.to_attos(), 0);
    /// assert_eq!(from_sec!(1, ms!(300)).to_attos(), sec!(1) + ms!(300));
    /// ```
    #[inline(always)]
    pub const fn to_attos(&self) -> i128 {
        self.attos
    }

    /// Splits into whole femtoseconds and leftover attoseconds, rounding the
    /// whole part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor).
    /// For truncation toward zero, use [`to_fs`](../struct.Dt.html#method.to_fs).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::from_fs;
    ///
    /// // +1.5 fs → 1 fs + 500 attoseconds leftover
    /// assert_eq!(from_fs!(1, 500).to_fs_floor(), (1, 500));
    /// // -1.5 fs → -2 fs + 500 attoseconds leftover
    /// assert_eq!(from_fs!(-1, -500).to_fs_floor(), (-2, 500));
    /// ```
    #[inline(always)]
    pub const fn to_fs_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_FS_I128),
            self.attos.rem_euclid(ATTOS_PER_FS_I128),
        )
    }

    /// Splits into whole picoseconds and leftover attoseconds, rounding the
    /// whole part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor).
    /// For truncation toward zero, use [`to_ps`](../struct.Dt.html#method.to_ps).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ps, fs};
    ///
    /// // +1.3 ps → 1 ps + 0.3 ps leftover
    /// assert_eq!(from_ps!(1, fs!(300)).to_ps_floor(), (1, fs!(300)));
    /// // -1.3 ps → -2 ps + 0.7 ps leftover
    /// assert_eq!(from_ps!(-1, -fs!(300)).to_ps_floor(), (-2, fs!(700)));
    /// ```
    #[inline(always)]
    pub const fn to_ps_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_PS_I128),
            self.attos.rem_euclid(ATTOS_PER_PS_I128),
        )
    }

    /// Splits into whole nanoseconds and leftover attoseconds, rounding the
    /// whole part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor).
    /// For truncation toward zero, use [`to_ns`](../struct.Dt.html#method.to_ns).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ns, ps};
    ///
    /// // +1.3 ns → 1 ns + 0.3 ns leftover
    /// assert_eq!(from_ns!(1, ps!(300)).to_ns_floor(), (1, ps!(300)));
    /// // -1.3 ns → -2 ns + 0.7 ns leftover
    /// assert_eq!(from_ns!(-1, -ps!(300)).to_ns_floor(), (-2, ps!(700)));
    /// ```
    #[inline(always)]
    pub const fn to_ns_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_NS_I128),
            self.attos.rem_euclid(ATTOS_PER_NS_I128),
        )
    }

    /// Splits into whole microseconds and leftover attoseconds, rounding the
    /// whole part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor).
    /// For truncation toward zero, use [`to_us`](../struct.Dt.html#method.to_us).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_us, ns};
    ///
    /// // +1.3 µs → 1 µs + 0.3 µs leftover
    /// assert_eq!(from_us!(1, ns!(300)).to_us_floor(), (1, ns!(300)));
    /// // -1.3 µs → -2 µs + 0.7 µs leftover
    /// assert_eq!(from_us!(-1, -ns!(300)).to_us_floor(), (-2, ns!(700)));
    /// ```
    #[inline(always)]
    pub const fn to_us_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_US_I128),
            self.attos.rem_euclid(ATTOS_PER_US_I128),
        )
    }

    /// Splits into whole milliseconds and leftover attoseconds, rounding the
    /// whole part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor)
    /// (e.g. −1.3 ms → `(-2, 700 µs in attoseconds)`). For truncation toward
    /// zero, use [`to_ms`](../struct.Dt.html#method.to_ms).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ms, us};
    ///
    /// // +1.3 ms → 1 ms + 0.3 ms leftover
    /// assert_eq!(from_ms!(1, us!(300)).to_ms_floor(), (1, us!(300)));
    ///
    /// // -1.3 ms → -2 ms + 0.7 ms leftover
    /// assert_eq!(from_ms!(-1, -us!(300)).to_ms_floor(), (-2, us!(700)));
    /// ```
    #[inline(always)]
    pub const fn to_ms_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_MS_I128),
            self.attos.rem_euclid(ATTOS_PER_MS_I128),
        )
    }

    /// Splits into whole minutes and leftover attoseconds, rounding the whole
    /// part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. For truncation toward zero (signed
    /// leftover), use [`to_mins`](../struct.Dt.html#method.to_mins).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, sec};
    ///
    /// // 90 s → 1 min + 30 s leftover
    /// assert_eq!(from_sec!(90).to_mins_floor(), (1, sec!(30)));
    /// // -90 s → -2 min + 30 s leftover
    /// assert_eq!(from_sec!(-90).to_mins_floor(), (-2, sec!(30)));
    /// ```
    #[inline(always)]
    pub const fn to_mins_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_MIN),
            self.attos.rem_euclid(ATTOS_PER_MIN),
        )
    }

    /// Splits into whole minutes and leftover attoseconds, dropping any
    /// fraction toward zero (1.5 → 1, −1.5 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_mins_floor`](../struct.Dt.html#method.to_mins_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_sec, sec};
    ///
    /// // 90 s → 1 min + 30 s leftover (signed on negatives)
    /// assert_eq!(from_sec!(90).to_mins(), (1, sec!(30)));
    /// assert_eq!(from_sec!(-90).to_mins(), (-1, sec!(-30)));
    /// ```
    #[inline(always)]
    pub const fn to_mins(&self) -> (i128, i128) {
        (self.attos / ATTOS_PER_MIN, self.attos % ATTOS_PER_MIN)
    }

    /// Converts this duration to minutes as a floating-point number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::from_sec;
    ///
    /// assert_eq!(Dt::ZERO.to_mins_f(), 0.0);
    /// assert_eq!(from_sec!(90).to_mins_f(), 1.5);
    /// assert_eq!(from_sec!(-90).to_mins_f(), -1.5);
    /// ```
    #[inline(always)]
    pub const fn to_mins_f(&self) -> Real {
        Self::attos_to_unit_f(self.attos, ATTOS_PER_MIN)
    }

    /// Splits into whole hours and leftover attoseconds, rounding the whole
    /// part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. For truncation toward zero (signed
    /// leftover), use [`to_hours`](../struct.Dt.html#method.to_hours).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{dt, mins};
    ///
    /// // 90 minutes → 1 hour + 30 minutes leftover
    /// assert_eq!(dt!(mins!(90)).to_hours_floor(), (1, mins!(30)));
    /// // -90 minutes → -2 hours + 30 minutes leftover
    /// assert_eq!(dt!(mins!(-90)).to_hours_floor(), (-2, mins!(30)));
    /// ```
    #[inline(always)]
    pub const fn to_hours_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_HOUR),
            self.attos.rem_euclid(ATTOS_PER_HOUR),
        )
    }

    /// Splits into whole hours and leftover attoseconds, dropping any
    /// fraction toward zero (1.5 → 1, −1.5 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_hours_floor`](../struct.Dt.html#method.to_hours_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{dt, mins};
    ///
    /// assert_eq!(dt!(mins!(90)).to_hours(), (1, mins!(30)));
    /// assert_eq!(dt!(mins!(-90)).to_hours(), (-1, -mins!(30)));
    /// ```
    #[inline(always)]
    pub const fn to_hours(&self) -> (i128, i128) {
        (self.attos / ATTOS_PER_HOUR, self.attos % ATTOS_PER_HOUR)
    }

    /// Converts this duration to hours as a floating-point number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{dt, mins};
    ///
    /// assert_eq!(Dt::ZERO.to_hours_f(), 0.0);
    /// assert_eq!(dt!(mins!(90)).to_hours_f(), 1.5);
    /// assert_eq!(dt!(mins!(-90)).to_hours_f(), -1.5);
    /// ```
    #[inline(always)]
    pub const fn to_hours_f(&self) -> Real {
        Self::attos_to_unit_f(self.attos, ATTOS_PER_HOUR)
    }

    /// Splits into whole days and leftover attoseconds, dropping any fraction
    /// toward zero (1.25 → 1, −1.25 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too — e.g. −1.25 days is
    /// `(-1, −6 hours in attoseconds)`.
    ///
    /// For a leftover that is always ≥ 0, use
    /// [`to_days_floor`](../struct.Dt.html#method.to_days_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_days_f, hours};
    ///
    /// // 1.25 d → 1 day + 6 hours leftover
    /// assert_eq!(from_days_f!(1.25).to_days(), (1, hours!(6)));
    ///
    /// // -1.25 d → -1 day with signed -6 hours leftover
    /// assert_eq!(from_days_f!(-1.25).to_days(), (-1, -hours!(6)));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_days_floor`](../struct.Dt.html#method.to_days_floor)
    /// - [`Dt::from_days`](../struct.Dt.html#method.from_days)
    #[inline(always)]
    pub const fn to_days(&self) -> (i128, i128) {
        (self.attos / ATTOS_PER_DAY, self.attos % ATTOS_PER_DAY)
    }

    /// Splits into whole days and leftover attoseconds, rounding the whole
    /// part down so the leftover is always ≥ 0.
    ///
    /// Returns `(whole, frac_attos)`. Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor) (e.g. −1.5 days →
    /// `(-2, half a day in attoseconds)`). For truncation toward zero, use
    /// [`to_days`](../struct.Dt.html#method.to_days).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_days_f, hours};
    ///
    /// // 1.25 d → 1 day + 6 hours leftover
    /// assert_eq!(from_days_f!(1.25).to_days_floor(), (1, hours!(6)));
    ///
    /// // -1.25 d → -2 whole days + 18 hours leftover
    /// assert_eq!(from_days_f!(-1.25).to_days_floor(), (-2, hours!(18)));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_days`](../struct.Dt.html#method.to_days)
    #[inline(always)]
    pub const fn to_days_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_DAY),
            self.attos.rem_euclid(ATTOS_PER_DAY),
        )
    }

    /// Converts this duration to days as a floating-point number.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::from_days_f;
    ///
    /// assert_eq!(Dt::ZERO.to_days_f(), 0.0);
    /// assert_eq!(from_days_f!(1.5).to_days_f(), 1.5);
    /// assert_eq!(from_days_f!(-1.5).to_days_f(), -1.5);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_days_f`](../struct.Dt.html#method.from_days_f)
    #[inline(always)]
    pub const fn to_days_f(&self) -> Real {
        Self::attos_to_unit_f(self.attos, ATTOS_PER_DAY)
    }

    /// Splits into whole weeks and leftover attoseconds, dropping any fraction
    /// toward zero (1.25 → 1, −1.25 → −1).
    ///
    /// A week is 7 civil days (`604_800` seconds). Returns `(whole, frac_attos)`.
    /// When negative with a fraction, `frac_attos` is negative too — e.g.
    /// −1.25 weeks is `(-1, −1 day − 18 hours in attoseconds)`.
    ///
    /// For a leftover that is always ≥ 0, use
    /// [`to_weeks_floor`](../struct.Dt.html#method.to_weeks_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{days, dt, hours, weeks};
    ///
    /// // Quarter week = 1 day + 18 hours
    /// let q = days!(1) + hours!(18);
    /// assert_eq!(dt!(weeks!(1) + q).to_weeks(), (1, q));
    /// assert_eq!(dt!(-weeks!(1) - q).to_weeks(), (-1, -q));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_weeks_floor`](../struct.Dt.html#method.to_weeks_floor)
    /// - [`Dt::from_weeks`](../struct.Dt.html#method.from_weeks)
    #[inline(always)]
    pub const fn to_weeks(&self) -> (i128, i128) {
        (self.attos / ATTOS_PER_WEEK, self.attos % ATTOS_PER_WEEK)
    }

    /// Splits into whole weeks and leftover attoseconds, rounding the whole
    /// part down so the leftover is always ≥ 0.
    ///
    /// A week is 7 civil days (`604_800` seconds). Returns `(whole, frac_attos)`.
    /// Same floor rule as
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor) (e.g. −1.5 weeks →
    /// `(-2, half a week in attoseconds)`). For truncation toward zero, use
    /// [`to_weeks`](../struct.Dt.html#method.to_weeks).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{days, dt, hours, weeks};
    ///
    /// // Quarter week = 1 day + 18 hours; three-quarter = 5 days + 6 hours
    /// let q = days!(1) + hours!(18);
    /// let three_q = days!(5) + hours!(6);
    /// assert_eq!(dt!(weeks!(1) + q).to_weeks_floor(), (1, q));
    ///
    /// // -1.25 weeks → -2 whole weeks + 0.75 week leftover
    /// assert_eq!(dt!(-weeks!(1) - q).to_weeks_floor(), (-2, three_q));
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_weeks`](../struct.Dt.html#method.to_weeks)
    #[inline(always)]
    pub const fn to_weeks_floor(&self) -> (i128, i128) {
        (
            self.attos.div_euclid(ATTOS_PER_WEEK),
            self.attos.rem_euclid(ATTOS_PER_WEEK),
        )
    }

    /// Converts this duration to weeks as a floating-point number.
    ///
    /// A week is 7 civil days (`604_800` seconds).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{days, dt, hours, weeks};
    ///
    /// assert_eq!(Dt::ZERO.to_weeks_f(), 0.0);
    ///
    /// // Half week = 3 days + 12 hours
    /// let half = days!(3) + hours!(12);
    /// assert_eq!(dt!(weeks!(1) + half).to_weeks_f(), 1.5);
    /// assert_eq!(dt!(-weeks!(1) - half).to_weeks_f(), -1.5);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_weeks`](../struct.Dt.html#method.from_weeks)
    /// - [`Dt::to_weeks`](../struct.Dt.html#method.to_weeks)
    #[inline(always)]
    pub const fn to_weeks_f(&self) -> Real {
        Self::attos_to_unit_f(self.attos, ATTOS_PER_WEEK)
    }

    /// Splits into whole milliseconds and leftover attoseconds, dropping any
    /// fraction toward zero (1.7 → 1, −1.7 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_ms_floor`](../struct.Dt.html#method.to_ms_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ms, us};
    ///
    /// assert_eq!(from_ms!(1, us!(300)).to_ms(), (1, us!(300)));
    /// assert_eq!(from_ms!(-1, -us!(300)).to_ms(), (-1, -us!(300)));
    /// ```
    #[inline(always)]
    pub const fn to_ms(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_MS_I128,
            self.attos % ATTOS_PER_MS_I128,
        )
    }

    /// Splits into whole microseconds and leftover attoseconds, dropping any
    /// fraction toward zero (1.7 → 1, −1.7 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_us_floor`](../struct.Dt.html#method.to_us_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_us, ns};
    ///
    /// assert_eq!(from_us!(1, ns!(300)).to_us(), (1, ns!(300)));
    /// assert_eq!(from_us!(-1, -ns!(300)).to_us(), (-1, -ns!(300)));
    /// ```
    #[inline(always)]
    pub const fn to_us(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_US_I128,
            self.attos % ATTOS_PER_US_I128,
        )
    }

    /// Splits into whole nanoseconds and leftover attoseconds, dropping any
    /// fraction toward zero (1.7 → 1, −1.7 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_ns_floor`](../struct.Dt.html#method.to_ns_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ns, ps};
    ///
    /// assert_eq!(from_ns!(1, ps!(300)).to_ns(), (1, ps!(300)));
    /// assert_eq!(from_ns!(-1, -ps!(300)).to_ns(), (-1, -ps!(300)));
    /// ```
    #[inline(always)]
    pub const fn to_ns(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_NS_I128,
            self.attos % ATTOS_PER_NS_I128,
        )
    }

    /// Splits into whole picoseconds and leftover attoseconds, dropping any
    /// fraction toward zero (1.7 → 1, −1.7 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_ps_floor`](../struct.Dt.html#method.to_ps_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::{from_ps, fs};
    ///
    /// assert_eq!(from_ps!(1, fs!(300)).to_ps(), (1, fs!(300)));
    /// assert_eq!(from_ps!(-1, -fs!(300)).to_ps(), (-1, -fs!(300)));
    /// ```
    #[inline(always)]
    pub const fn to_ps(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_PS_I128,
            self.attos % ATTOS_PER_PS_I128,
        )
    }

    /// Splits into whole femtoseconds and leftover attoseconds, dropping any
    /// fraction toward zero (1.7 → 1, −1.7 → −1).
    ///
    /// Returns `(whole, frac_attos)`. When negative with a fraction,
    /// `frac_attos` is negative too. For a leftover that is always ≥ 0, use
    /// [`to_fs_floor`](../struct.Dt.html#method.to_fs_floor).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    /// use deep_time::macros::from_fs;
    ///
    /// assert_eq!(from_fs!(1, 500).to_fs(), (1, 500));
    /// assert_eq!(from_fs!(-1, -500).to_fs(), (-1, -500));
    /// ```
    #[inline(always)]
    pub const fn to_fs(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_FS_I128,
            self.attos % ATTOS_PER_FS_I128,
        )
    }
}
