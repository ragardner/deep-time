use crate::{
    ATTOS_PER_DAY, ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128,
    ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_SECF, ATTOS_PER_US_I128,
    Dt, Real,
};

impl Dt {
    /// Returns the whole seconds portion of this [`Dt`] using truncation towards zero
    /// (i.e., the integer part obtained via truncating division, without rounding).
    ///
    /// This is equivalent to `self.attos / ATTOS_PER_SEC_I128`.
    ///
    /// Unlike
    /// [`to_sec_floor`](../struct.Dt.html#method.to_sec_floor)
    /// (which uses Euclidean division, flooring towards
    /// negative infinity for negative values to keep the fractional part non-negative),
    /// this version truncates towards zero.
    ///
    /// Consequently, for values in `(-1, 0)` seconds (e.g. -0.3 s or -0.8 s),
    /// both return `0`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// // -0.3 seconds → truncates to 0
    /// let dt = Dt::span(-300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec(), 0);
    ///
    /// // -0.8 seconds → truncates to 0
    /// let dt = Dt::span(-800_000_000_000_000_000);
    /// assert_eq!(dt.to_sec(), 0);
    ///
    /// // -1.3 seconds → truncates to -1 (while to_sec_floor gives -2)
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec(), -1);
    /// assert_eq!(dt.to_sec_floor(), -2);
    ///
    /// // Positive values behave the same as `to_sec_floor`
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec(), 1);
    /// assert_eq!(dt.to_sec_floor(), 1);
    /// ```
    #[inline(always)]
    pub const fn to_sec(&self) -> i128 {
        self.attos / ATTOS_PER_SEC_I128
    }

    /// Returns the whole seconds portion of this [`Dt`] using truncation towards zero,
    /// then clamped to an [`i64`].
    ///
    /// If the truncated seconds value lies outside the `i64` range, the result
    /// saturates to [`i64::MAX`] or [`i64::MIN`].
    ///
    /// See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the truncation semantics
    /// (towards zero, no rounding).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64(), -1);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64_floor(), 1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64(&self) -> i64 {
        Self::to_i64(self.attos / ATTOS_PER_SEC_I128)
    }

    /// If this time were turned into [`i128`] seconds and [`u64`] (always
    /// pushing to the positive) fractional attoseconds, this returns the
    /// whole seconds part.
    ///
    /// To just get seconds rounded to the nearest second use
    /// [`Dt::to_sec_round`](../struct.Dt.html#method.to_sec_round)
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
    /// let sec = dt.to_sec_floor();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec_floor(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), 300_000_000_000_000_000);
    ///
    /// // if you just want rounded seconds
    /// // use to_sec_round() instead
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// let sec = dt.to_sec_round();
    /// assert_eq!(sec, -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec_floor(&self) -> i128 {
        self.attos.div_euclid(ATTOS_PER_SEC_I128)
    }

    /// If this time were turned into [`i64`] seconds and [`u64`] (always
    /// pushing to the positive) fractional attoseconds, this returns the
    /// whole seconds part.
    ///
    /// To just get seconds rounded to the nearest second use
    /// [`Dt::to_sec_round`](../struct.Dt.html#method.to_sec_round)
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
    /// let sec = dt.to_sec64_floor();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec64_floor(), 1);
    /// assert_eq!(dt.to_sec_ufrac(), 300_000_000_000_000_000);
    ///
    /// // if you just want rounded seconds
    /// // use to_sec_round() instead
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// let sec = dt.to_sec_round();
    /// assert_eq!(sec, -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64_floor(&self) -> i64 {
        Self::to_i64(self.attos.div_euclid(ATTOS_PER_SEC_I128))
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
    /// assert_eq!(Dt::span(1_300_000_000_000_000_000).to_sec_round(), 1);
    ///
    /// // -1.3 seconds → rounds to -1
    /// assert_eq!(Dt::span(-1_300_000_000_000_000_000).to_sec_round(), -1);
    ///
    /// // 1.6 seconds → rounds to 2
    /// assert_eq!(Dt::span(1_600_000_000_000_000_000).to_sec_round(), 2);
    ///
    /// // Halfway cases
    /// assert_eq!(Dt::span(500_000_000_000_000_000).to_sec_round(), 1);
    /// assert_eq!(Dt::span(-500_000_000_000_000_000).to_sec_round(), -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec_round(&self) -> i128 {
        self.round_to_sec().to_sec()
    }

    /// Returns this [`Dt`] rounded to the nearest whole second, then
    /// converted to an [`i64`] number of seconds.
    ///
    /// - Exactly halfway cases round as follows: 0.5 becomes 1 and -0.5 becomes -1,
    ///   same as
    ///   [`to_sec_round`](../struct.Dt.html#method.to_sec_round).
    /// - If the rounded value is outside the representable `i64` range,
    ///   it saturates to [`i64::MAX`] or [`i64::MIN`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Dt;
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64_round(), 1);
    ///
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// assert_eq!(dt.to_sec64_round(), -1);
    /// ```
    #[inline(always)]
    pub const fn to_sec64_round(&self) -> i64 {
        Self::to_i64(self.round_to_sec().to_sec())
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
    /// - The conversion is lossy, as [`f64`] provides approximately 15.95 decimal
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

    /// If this time were turned into seconds, this returns the signed fractional
    /// attoseconds part — the amount left over after removing whole seconds, with the
    /// same sign as the original value when non-zero.
    ///
    /// Pairs with [`from_sec_and_frac`](../struct.Dt.html#method.from_sec_and_frac).
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
    /// let sec = dt.to_sec64_floor();
    /// assert_eq!(sec, -2);
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    ///
    /// assert_eq!(dt.to_sec64_floor(), 1);
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

    /// Returns the total time in attoseconds.
    #[inline(always)]
    pub const fn to_attos(&self) -> i128 {
        self.attos
    }

    /// Converts this [`Dt`] into whole femtoseconds and a fractional part within one femtosecond.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// - For truncation toward zero, use [`to_fs`](../struct.Dt.html#method.to_fs).
    #[inline(always)]
    pub const fn to_fs_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_FS_I128),
            self.attos.rem_euclid(ATTOS_PER_FS_I128) as u128,
        )
    }

    /// Converts this [`Dt`] into whole picoseconds and a fractional part within one picosecond.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// - For truncation toward zero, use [`to_ps`](../struct.Dt.html#method.to_ps).
    #[inline(always)]
    pub const fn to_ps_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_PS_I128),
            self.attos.rem_euclid(ATTOS_PER_PS_I128) as u128,
        )
    }

    /// Converts this [`Dt`] into whole nanoseconds and a fractional part within one nanosecond.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// - For truncation toward zero, use [`to_ns`](../struct.Dt.html#method.to_ns).
    #[inline(always)]
    pub const fn to_ns_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_NS_I128),
            self.attos.rem_euclid(ATTOS_PER_NS_I128) as u128,
        )
    }

    /// Converts this [`Dt`] into whole microseconds and a fractional part within one microsecond.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// - For truncation toward zero, use [`to_us`](../struct.Dt.html#method.to_us).
    #[inline(always)]
    pub const fn to_us_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_US_I128),
            self.attos.rem_euclid(ATTOS_PER_US_I128) as u128,
        )
    }

    /// Converts this [`Dt`] into whole milliseconds and a fractional part within one millisecond.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// - For truncation toward zero, use [`to_ms`](../struct.Dt.html#method.to_ms).
    #[inline(always)]
    pub const fn to_ms_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_MS_I128),
            self.attos.rem_euclid(ATTOS_PER_MS_I128) as u128,
        )
    }

    /// Converts this [`Dt`] into whole minutes and a fractional part within one minute.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    #[inline(always)]
    pub const fn to_mins_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_MIN),
            self.attos.rem_euclid(ATTOS_PER_MIN) as u128,
        )
    }

    /// Converts this [`Dt`] into whole hours and a fractional part within one hour.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    #[inline(always)]
    pub const fn to_hours_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_HOUR),
            self.attos.rem_euclid(ATTOS_PER_HOUR) as u128,
        )
    }

    /// Converts this [`Dt`] into whole days and a fractional part within one day.
    ///
    /// - Returns `(whole, frac_attos)` where `frac_attos` is always non-negative.
    /// - For negative values this does **not** split at the decimal point — see
    ///   [`to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   [`to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    /// use deep_time::consts::ATTOS_PER_HALF_DAY_U128;
    ///
    /// // library epoch is 2000-01-01 12:00:00 TAI
    /// // so result will be negative 2 + half a day
    /// // effectively -1.5 days
    /// let dt = Dt::from_ymd(1999, 12, 31, Scale::TAI, 0, 0, 0, 0);
    /// let (days, attos) = dt.to_days_floor();
    /// assert_eq!(days, -2);
    /// assert_eq!(attos, ATTOS_PER_HALF_DAY_U128);
    /// ```
    #[inline(always)]
    pub const fn to_days_floor(&self) -> (i128, u128) {
        (
            self.attos.div_euclid(ATTOS_PER_DAY),
            self.attos.rem_euclid(ATTOS_PER_DAY) as u128,
        )
    }

    /// Converts this [`Dt`] into whole milliseconds and a fractional part within one millisecond,
    /// truncating toward zero.
    ///
    /// Returns `(whole, frac_attos)`. `frac_attos` will be negative when `whole` is negative and
    /// there is a non-zero fraction. See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the same rule in seconds.
    #[inline(always)]
    pub const fn to_ms(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_MS_I128,
            self.attos % ATTOS_PER_MS_I128,
        )
    }

    /// Converts this [`Dt`] into whole microseconds and a fractional part within one microsecond,
    /// truncating toward zero.
    ///
    /// Returns `(whole, frac_attos)`. `frac_attos` will be negative when `whole` is negative and
    /// there is a non-zero fraction. See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the same rule in seconds.
    #[inline(always)]
    pub const fn to_us(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_US_I128,
            self.attos % ATTOS_PER_US_I128,
        )
    }

    /// Converts this [`Dt`] into whole nanoseconds and a fractional part within one nanosecond,
    /// truncating toward zero.
    ///
    /// Returns `(whole, frac_attos)`. `frac_attos` will be negative when `whole` is negative and
    /// there is a non-zero fraction. See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the same rule in seconds.
    #[inline(always)]
    pub const fn to_ns(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_NS_I128,
            self.attos % ATTOS_PER_NS_I128,
        )
    }

    /// Converts this [`Dt`] into whole picoseconds and a fractional part within one picosecond,
    /// truncating toward zero.
    ///
    /// Returns `(whole, frac_attos)`. `frac_attos` will be negative when `whole` is negative and
    /// there is a non-zero fraction. See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the same rule in seconds.
    #[inline(always)]
    pub const fn to_ps(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_PS_I128,
            self.attos % ATTOS_PER_PS_I128,
        )
    }

    /// Converts this [`Dt`] into whole femtoseconds and a fractional part within one femtosecond,
    /// truncating toward zero.
    ///
    /// Returns `(whole, frac_attos)`. `frac_attos` will be negative when `whole` is negative and
    /// there is a non-zero fraction. See
    /// [`to_sec`](../struct.Dt.html#method.to_sec)
    /// for the same rule in seconds.
    #[inline(always)]
    pub const fn to_fs(&self) -> (i128, i128) {
        (
            self.attos / ATTOS_PER_FS_I128,
            self.attos % ATTOS_PER_FS_I128,
        )
    }
}
