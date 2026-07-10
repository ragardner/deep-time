use crate::{
    ATTOS_PER_DAY, ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128,
    ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_US_I128, Dt, Real,
    SEC_PER_DAY_I64, SEC_PER_WEEK, Scale, TAI_SECS_1970_MIDNIGHT_TO_2000_NOON,
};

impl Dt {
    /// The library’s internal reference epoch.
    ///
    /// - **2000-01-01 12:00:00 TAI**.
    /// - 0 attoseconds
    /// - The vast majority of conversion functions in the library expect the given
    ///   [`Dt`] to be an attoseconds count since this epoch.
    pub const ZERO: Self = Self::new(0, Scale::TAI, Scale::TAI);

    /// UNIX epoch.
    ///
    /// - 1970-01-01 00:00:00 TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -946_728_000_000_000_000_000_000_000 attoseconds
    /// - Does not take into account historical UTC offsets from the "rubber time" era.
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const UNIX_EPOCH: Self = Self::new(
        -(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON as i128) * ATTOS_PER_SEC_I128,
        Scale::TAI,
        Scale::UTC,
    );

    /// NTP epoch.
    ///
    /// - 1900-01-01 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -3_155_716_800_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const NTP_EPOCH: Self =
        Self::new(-3155716800000000000000000000i128, Scale::TAI, Scale::TAI);

    /// TT/TCG/TCB/TDB epoch.
    ///
    /// - 1977-01-01 00:00:00 TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -725_803_200_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const TAI_1977_EPOCH: Self =
        Self::new(-725803200000000000000000000i128, Scale::TAI, Scale::TAI);

    /// Chandra X-ray Center (CXC) Time epoch.
    ///
    /// - 1998-01-01 00:00:00 TT.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -63_115_232_184_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const CXC_EPOCH: Self = Self::new(-63115232184000000000000000i128, Scale::TAI, Scale::TT);

    /// GPS/Galileo Experiment (GALEX) Time epoch.
    ///
    /// - 1980-01-06 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -630_763_181_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const GPS_EPOCH: Self = Self::new(-630763181000000000000000000i128, Scale::TAI, Scale::GPS);

    /// Galileo System Time (GST) epoch.
    ///
    /// - 1999-08-22 00:00:00 GST.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -11_447_981_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const GALILEO_EPOCH: Self =
        Self::new(-11447981000000000000000000i128, Scale::TAI, Scale::GST);

    /// BeiDou Time (BDT) epoch.
    ///
    /// - 2006-01-01 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - 189_345_633_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const BDT_EPOCH: Self = Self::new(189345633000000000000000000i128, Scale::TAI, Scale::BDT);

    /// CCSDS epoch (used in CCSDS time codes such as CUC).
    ///
    /// - 1958-01-01 00:00:00 TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -1_325_419_200_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const CCSDS_EPOCH: Self = Self::new(
        -1_325_419_200_000_000_000_000_000_000i128,
        Scale::TAI,
        Scale::TAI,
    );

    /// Maximum representable duration.
    pub const MAX: Self = Self::new(i128::MAX, Scale::TAI, Scale::TAI);

    /// Minimum (most negative) representable duration.
    pub const MIN: Self = Self::new(i128::MIN, Scale::TAI, Scale::TAI);

    /// 19 seconds.
    pub const SEC_19: Self = Self::new(19i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);

    /// 33 seconds.
    pub const SEC_33: Self = Self::new(33i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);

    /// 37 seconds.
    pub const SEC_37: Self = Self::new(37i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);

    /// One days worth of attoseconds.
    pub const ONE_DAY: Self = Self::new(
        (SEC_PER_DAY_I64 as i128) * ATTOS_PER_SEC_I128,
        Scale::TAI,
        Scale::TAI,
    );

    /// Creates a new [`Dt`] from a total number of attoseconds since the librarys
    /// epoch **2000-01-01 12:00:00 TAI**.
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // current scale TAI, target scale UTC
    /// let a = Dt::new(0, Scale::TAI, Scale::UTC);
    ///
    /// // equivalent to direct construction
    /// let b = Dt { attos: 0, scale: Scale::TAI, target: Scale::UTC };
    ///
    /// assert_eq!(a, b);
    /// ```
    #[inline(always)]
    pub const fn new(attos: i128, scale: Scale, target: Scale) -> Dt {
        Self {
            attos,
            scale,
            target,
        }
    }

    /// Creates a new [`Dt`] from a total number of seconds since the librarys
    /// epoch **2000-01-01 12:00:00 TAI**.
    ///
    /// Does **not** perform any time scale conversions.
    #[inline(always)]
    pub const fn new_sec(sec: i128, scale: Scale, target: Scale) -> Dt {
        Self {
            attos: Dt::sec_to_attos(sec),
            scale,
            target,
        }
    }

    /// Creates a new [`Dt`] from a total number of seconds as a float
    /// since the librarys epoch **2000-01-01 12:00:00 TAI**.
    ///
    /// - Does **not** perform any time scale conversions.
    /// - Fractional seconds represented by any decimals.
    #[inline(always)]
    pub const fn new_f(sec: Real, scale: Scale, target: Scale) -> Dt {
        Self {
            attos: Dt::sec_f_to_attos(sec),
            scale,
            target,
        }
    }

    /// Creates a new [`Dt`] from a total number of attoseconds (signed i128) without
    /// performing any time scale conversions.
    ///
    /// - This is an easy way to create a duration.
    /// - The returned [`Dt`] has its `scale` and `target` fields set to
    ///   `Scale::TAI`.
    #[inline(always)]
    pub const fn span(attos: i128) -> Dt {
        Dt::new(attos, Scale::TAI, Scale::TAI)
    }

    /// Creates a [`Dt`] from a floating-point number of seconds without performing
    /// any time scale conversions.
    ///
    /// - This is an easy way to create a duration or a seconds count that doesn't
    ///   include any time scale conversions, just holds the seconds count as is.
    /// - The returned [`Dt`] has its `scale` and `target` fields set to
    ///   `Scale::TAI`.
    #[inline(always)]
    pub const fn span_f(sec: Real) -> Dt {
        Self::from_sec_f(sec, Scale::TAI)
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

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - **Requires** a seconds and attoseconds count such that would be returned from the
    ///   functions [`Dt::to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor) and
    ///   **[`Dt::to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac)**.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    /// - The `sec` should be from the epoch TAI 2000-01-01 12:00:00.
    ///
    /// This function performs a time scale conversion from the given `scale` to **TAI**,
    /// if you don't want any time scale conversion to take place then either use
    /// `Scale::TAI` as an arg or use any of the following constructors:
    ///
    /// - [`Dt::new`](../struct.Dt.html#method.new)
    /// - [`Dt::new_sec`](../struct.Dt.html#method.new_sec)
    /// - [`Dt::new_f`](../struct.Dt.html#method.new_f)
    /// - [`Dt::span`](../struct.Dt.html#method.span)
    /// - [`Dt::span_f`](../struct.Dt.html#method.span_f)
    /// - [`Dt::from_tai_sec`](../struct.Dt.html#method.from_tai_sec)
    #[inline(always)]
    pub fn from_sec_and_ufrac(sec: i64, attos: u64, scale: Scale) -> Dt {
        if attos == 0 {
            Dt::new((sec as i128) * ATTOS_PER_SEC_I128, scale, scale).to_tai()
        } else {
            Dt::new(
                (sec as i128) * ATTOS_PER_SEC_I128 + (attos as i128),
                scale,
                scale,
            )
            .to_tai()
        }
    }

    /// Builds a [`Dt`] from whole seconds plus a sub-second attoseconds remainder.
    ///
    /// - `sec` — whole seconds only (no fraction). Use
    ///   [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64)
    ///   to obtain this from an existing [`Dt`].
    /// - `attos` — the signed sub-second remainder in attoseconds, as returned by
    ///   [`Dt::to_sec_frac`](../struct.Dt.html#method.to_sec_frac).
    ///   For a total of `1.3` s: `sec = 1`, `attos = 300_000_000_000_000_000`.
    ///   For `-1.3` s: `sec = -1`, `attos = -300_000_000_000_000_000`.
    ///   For `-0.5` s: `sec = 0`, `attos = -500_000_000_000_000_000`.
    ///
    /// This whole/remainder split differs from
    /// [`Dt::to_sec64_floor`](../struct.Dt.html#method.to_sec64_floor)
    /// +
    /// [`Dt::to_sec_ufrac`](../struct.Dt.html#method.to_sec_ufrac).
    /// Use
    /// [`from_sec_and_ufrac`](../struct.Dt.html#method.from_sec_and_ufrac)
    /// for that pairing.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::span(1_300_000_000_000_000_000);
    /// assert_eq!(
    ///     Dt::from_sec_and_frac(1, 300_000_000_000_000_000, Scale::TAI),
    ///     dt,
    /// );
    ///
    /// let dt = Dt::span(-1_300_000_000_000_000_000);
    /// assert_eq!(
    ///     Dt::from_sec_and_frac(-1, -300_000_000_000_000_000, Scale::TAI),
    ///     dt,
    /// );
    ///
    /// let dt = Dt::span(-500_000_000_000_000_000);
    /// assert_eq!(
    ///     Dt::from_sec_and_frac(0, -500_000_000_000_000_000, Scale::TAI),
    ///     dt,
    /// );
    /// ```
    ///
    /// The result is stored on TAI and converted from `scale`.
    /// `sec` is measured from the library epoch: 2000-01-01 12:00:00 TAI.
    ///
    /// To avoid scale conversion, pass `Scale::TAI`, or use one of:
    ///
    /// - [`Dt::new`](../struct.Dt.html#method.new)
    /// - [`Dt::new_sec`](../struct.Dt.html#method.new_sec)
    /// - [`Dt::new_f`](../struct.Dt.html#method.new_f)
    /// - [`Dt::span`](../struct.Dt.html#method.span)
    /// - [`Dt::span_f`](../struct.Dt.html#method.span_f)
    /// - [`Dt::from_tai_sec`](../struct.Dt.html#method.from_tai_sec)
    #[inline(always)]
    pub fn from_sec_and_frac(sec: i64, attos: i64, scale: Scale) -> Dt {
        Dt::new(
            (sec as i128) * ATTOS_PER_SEC_I128 + (attos as i128),
            scale,
            scale,
        )
        .to_tai()
    }

    /// Creates a new [`Dt`] from a total number of seconds (signed i128) without
    /// performing any time scale conversions.
    #[inline(always)]
    pub const fn from_tai_sec(sec: i128) -> Dt {
        Dt::new(
            sec.saturating_mul(ATTOS_PER_SEC_I128),
            Scale::TAI,
            Scale::TAI,
        )
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total seconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    ///
    /// This function performs a time scale conversion from the given `scale` to **TAI**,
    /// if you don't want any time scale conversion to take place then either use
    /// `Scale::TAI` as an arg or use any of the following constructors:
    ///
    /// - [`Dt::new`](../struct.Dt.html#method.new)
    /// - [`Dt::new_sec`](../struct.Dt.html#method.new_sec)
    /// - [`Dt::new_f`](../struct.Dt.html#method.new_f)
    /// - [`Dt::span`](../struct.Dt.html#method.span)
    /// - [`Dt::span_f`](../struct.Dt.html#method.span_f)
    /// - [`Dt::from_tai_sec`](../struct.Dt.html#method.from_tai_sec)
    #[inline(always)]
    pub const fn from_sec(sec: i128, scale: Scale) -> Dt {
        Dt::new(sec.saturating_mul(ATTOS_PER_SEC_I128), scale, scale).to_tai()
    }

    /// Builds a [`Dt`] holding the given whole milliseconds and sub-millisecond remainder.
    ///
    /// The remainder is in **attoseconds**, not milliseconds. Pairs with
    /// [`to_ms`](../struct.Dt.html#method.to_ms).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `ms` — whole milliseconds (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.3` ms: `ms = 1`, `frac_attos` = 0.3 ms in attoseconds.
    ///   For `-1.3` ms: `ms = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_ms(ms: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(ms, frac_attos, ATTOS_PER_MS_I128);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole microseconds and sub-microsecond remainder.
    ///
    /// The remainder is in **attoseconds**, not microseconds. Pairs with
    /// [`to_us`](../struct.Dt.html#method.to_us).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `us` — whole microseconds (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.3` µs: `us = 1`, `frac_attos` = 0.3 µs in attoseconds.
    ///   For `-1.3` µs: `us = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_us(us: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(us, frac_attos, ATTOS_PER_US_I128);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole nanoseconds and sub-nanosecond remainder.
    ///
    /// The remainder is in **attoseconds**, not nanoseconds. Pairs with
    /// [`to_ns`](../struct.Dt.html#method.to_ns).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `ns` — whole nanoseconds (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.3` ns: `ns = 1`, `frac_attos` = 0.3 ns in attoseconds.
    ///   For `-1.3` ns: `ns = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_ns(ns: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(ns, frac_attos, ATTOS_PER_NS_I128);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole picoseconds and sub-picosecond remainder.
    ///
    /// The remainder is in **attoseconds**, not picoseconds. Pairs with
    /// [`to_ps`](../struct.Dt.html#method.to_ps).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `ps` — whole picoseconds (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.3` ps: `ps = 1`, `frac_attos` = 0.3 ps in attoseconds.
    ///   For `-1.3` ps: `ps = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_ps(ps: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(ps, frac_attos, ATTOS_PER_PS_I128);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole femtoseconds and sub-femtosecond remainder.
    ///
    /// The remainder is in **attoseconds**, not femtoseconds. Pairs with
    /// [`to_fs`](../struct.Dt.html#method.to_fs).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `fs` — whole femtoseconds (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.3` fs: `fs = 1`, `frac_attos` = 0.3 fs in attoseconds.
    ///   For `-1.3` fs: `fs = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_fs(fs: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(fs, frac_attos, ATTOS_PER_FS_I128);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole minutes and sub-minute remainder.
    ///
    /// The remainder is in **attoseconds**, not minutes.
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `n` — whole minutes (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.5` min: `n = 1`, `frac_attos` = 0.5 min in attoseconds.
    ///   For `-1.5` min: `n = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_mins(n: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(n, frac_attos, ATTOS_PER_MIN);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole hours and sub-hour remainder.
    ///
    /// The remainder is in **attoseconds**, not hours.
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `n` — whole hours (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.5` h: `n = 1`, `frac_attos` = 0.5 h in attoseconds.
    ///   For `-1.5` h: `n = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_hours(n: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(n, frac_attos, ATTOS_PER_HOUR);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given whole days and sub-day remainder.
    ///
    /// The remainder is in **attoseconds**, not days. Uses `86400` seconds per day.
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `d` — whole days (truncating / signed-remainder split).
    /// - `frac_attos` — fractional part of that split, in attoseconds.
    ///   For `1.25` d: `d = 1`, `frac_attos` = 0.25 d in attoseconds.
    ///   For `-1.25` d: `d = -1`, `frac_attos` negative.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_days(d: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_and_signed_attos_to_attos(d, frac_attos, ATTOS_PER_DAY);
        Dt::new(attos, on, target)
    }

    /// Builds a [`Dt`] holding the given number of weeks (`604800` seconds each).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `n` — whole weeks.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_weeks(n: i128, on: Scale, target: Scale) -> Dt {
        Dt::new(
            n.saturating_mul(SEC_PER_WEEK as i128)
                .saturating_mul(ATTOS_PER_SEC_I128),
            on,
            target,
        )
    }

    /// Builds a [`Dt`] holding the given number of Julian years (`31_557_600` seconds each).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `n` — whole years.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_years(n: i128, on: Scale, target: Scale) -> Dt {
        Dt::new(
            n.saturating_mul(31_557_600)
                .saturating_mul(ATTOS_PER_SEC_I128),
            on,
            target,
        )
    }

    /// Returns an instant that is this duration **before** zero attoseconds on `scale`.
    ///
    /// Zero attoseconds is the library epoch **2000-01-01 12:00:00** (see
    /// [`Dt::ZERO`](../struct.Dt.html#associatedconstant.ZERO)).
    ///
    /// This method does **not** read the system clock.
    ///
    /// For wall-clock “N units ago”, use [`Dt::ago`](../struct.Dt.html#method.ago)
    /// (requires `std`, or WASM with `js`).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, TimeTraits};
    ///
    /// let t = 5.sec().before_zero(Scale::TAI);
    /// assert_eq!(t, Dt::ZERO.sub(5.sec()));
    /// assert_eq!(t.to_sec(), -5);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::after_zero`](../struct.Dt.html#method.after_zero)
    /// - [`Dt::ago`](../struct.Dt.html#method.ago)
    #[inline(always)]
    pub const fn before_zero(self, scale: Scale) -> Dt {
        Dt::new(0, scale, scale).to_tai().sub(self)
    }

    /// Returns the negation of this [`Dt`].
    #[inline(always)]
    pub const fn neg(self) -> Dt {
        Dt::new(-self.attos, self.scale, self.target)
    }

    /// Returns the positive of this [`Dt`].
    #[inline(always)]
    pub const fn abs(self) -> Dt {
        Dt::new(self.attos.saturating_abs(), self.scale, self.target)
    }

    /// Creates a [`Dt`] from a floating-point number of seconds.
    ///
    /// - Assumes the value is on the given scale.
    /// - Converts the value to TAI from the given `scale`.
    /// - The returned [`Dt`] is on the TAI time scale - its `scale`
    ///   field is `TAI` and its `target` field is the provided time
    ///   scale argument.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let seconds = 5.5;
    ///
    /// // use TAI for no conversions
    /// let duration = Dt::from_sec_f(seconds, Scale::TAI);
    ///
    /// assert_eq!(duration.to_sec_f(), seconds);
    /// ```
    #[inline]
    pub const fn from_sec_f(sec: Real, scale: Scale) -> Dt {
        if sec.is_nan() {
            return Self::ZERO;
        } else if sec.is_infinite() {
            return if sec.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }
        Dt::new(Self::sec_f_to_attos(sec), scale, scale).to_tai()
    }

    /// High-precision conversion from [`Real`] seconds to total attoseconds (i128).
    ///
    /// - Uses IEEE 754 bit extraction + exact integer multiplication by 5^18.
    /// - Returns the rounded integer (round-to-nearest, ties away from zero).
    pub const fn sec_f_to_attos(sec: Real) -> i128 {
        if sec == 0.0 {
            return 0;
        }

        let bits = sec.to_bits();
        let is_negative = (bits >> 63) != 0;
        let biased_exp = ((bits >> 52) & 0x7ff) as i32;
        let mantissa = bits & 0x000f_ffff_ffff_ffff;

        let (sig, exp) = if biased_exp == 0 {
            if mantissa == 0 {
                return 0;
            }
            (mantissa as u128, -1022i32 - 52)
        } else {
            let sig = ((1u64 << 52) | mantissa) as u128;
            (sig, biased_exp - 1023 - 52)
        };

        const FIVE_POW_18: u128 = 3_814_697_265_625; // 5^18 exactly
        let product = sig * FIVE_POW_18;
        let total_exp = exp + 18;

        // Safe saturation / underflow guards (prevents invalid shifts >= 128)
        if total_exp > 120 {
            return if is_negative { i128::MIN } else { i128::MAX };
        }
        if total_exp < -97 {
            return 0;
        }

        let abs_total = if total_exp >= 0 {
            let shift = total_exp as u32;
            if product > (u128::MAX >> shift) {
                if is_negative { i128::MIN } else { i128::MAX }
            } else {
                let shifted = product << shift;
                if shifted > i128::MAX as u128 {
                    if is_negative { i128::MIN } else { i128::MAX }
                } else {
                    shifted as i128
                }
            }
        } else {
            let shift = (-total_exp) as u32;
            let int_part = (product >> shift) as i128;

            // Round to nearest, half away from zero (on the absolute value)
            let mask = (1u128 << shift) - 1;
            let rem = product & mask;
            if rem > (mask >> 1) {
                int_part + 1
            } else {
                int_part
            }
        };

        if is_negative { -abs_total } else { abs_total }
    }

    /// Returns the current system time as TAI from 2000-01-01 12:00:00.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    pub fn now() -> Dt {
        let now = std::time::SystemTime::now();

        let (secs, nanos): (i64, i64) = match now.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos() as i64),
            Err(e) => {
                let dur = e.duration();
                (-(dur.as_secs() as i64), -(dur.subsec_nanos() as i64))
            }
        };

        Dt::from_diff_and_scale(
            Dt::new(Dt::sec_to_attos(secs as i128), Scale::TAI, Scale::UTC),
            Dt::UNIX_EPOCH,
            false,
        )
        .add(Dt::from_ns(nanos as i128, 0, Scale::TAI, Scale::TAI))
    }

    /// Returns the current system time as TAI from 2000-01-01 12:00:00.
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    pub fn now() -> Dt {
        let ms: f64 = js_sys::Date::now();
        let secs = (ms / 1000.0).floor() as i128;
        let nanos = ((ms % 1000.0) * 1_000_000.0) as i128;
        Dt::from_diff_and_scale(
            Dt::new(Dt::sec_to_attos(secs), Scale::TAI, Scale::UTC),
            Dt::UNIX_EPOCH,
            false,
        )
        .add(Dt::from_ns(nanos as i128, 0, Scale::TAI, Scale::TAI))
    }

    /// Returns an instant that is this duration **before** the current system time.
    ///
    /// Subtracts `self` from [`Dt::now`](../struct.Dt.html#method.now). Available under
    /// the same conditions as that method: the `std` feature (non-WASM-js), or WASM with
    /// the `js` feature.
    ///
    /// For a `const` offset from the library epoch (no system clock), use
    /// [`Dt::before_zero`](../struct.Dt.html#method.before_zero).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")]
    /// # {
    /// use deep_time::{Dt, TimeTraits};
    ///
    /// // ~3 days in the past relative to the system clock
    /// let past = 3.days().ago();
    /// assert!(past < Dt::now());
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_now`](../struct.Dt.html#method.from_now)
    /// - [`Dt::before_zero`](../struct.Dt.html#method.before_zero)
    /// - [`Dt::now`](../struct.Dt.html#method.now)
    #[cfg(any(
        all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))),
        all(target_arch = "wasm32", feature = "js"),
    ))]
    #[inline]
    pub fn ago(self) -> Dt {
        Dt::now().sub(self)
    }

    /// Returns an instant that is this duration **after** the current system time.
    ///
    /// Adds `self` to [`Dt::now`](../struct.Dt.html#method.now). Available under the same
    /// conditions as that method: the `std` feature (non-WASM-js), or WASM with the `js`
    /// feature.
    ///
    /// For a `const` offset from the library epoch (no system clock), use
    /// [`Dt::after_zero`](../struct.Dt.html#method.after_zero).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "std")]
    /// # {
    /// use deep_time::{Dt, TimeTraits};
    ///
    /// // ~3 days in the future relative to the system clock
    /// let future = 3.days().from_now();
    /// assert!(future > Dt::now());
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::ago`](../struct.Dt.html#method.ago)
    /// - [`Dt::after_zero`](../struct.Dt.html#method.after_zero)
    /// - [`Dt::now`](../struct.Dt.html#method.now)
    #[cfg(any(
        all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))),
        all(target_arch = "wasm32", feature = "js"),
    ))]
    #[inline]
    pub fn from_now(self) -> Dt {
        Dt::now().add(self)
    }
}
