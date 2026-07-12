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
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
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
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -3_155_716_800_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const NTP_EPOCH: Self =
        Self::new(-3155716800000000000000000000i128, Scale::TAI, Scale::TAI);

    /// TT/TCG/TCB/TDB epoch.
    ///
    /// - 1977-01-01 00:00:00 TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -725_803_200_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const TAI_1977_EPOCH: Self =
        Self::new(-725803200000000000000000000i128, Scale::TAI, Scale::TAI);

    /// Chandra X-ray Center (CXC) Time epoch.
    ///
    /// - 1998-01-01 00:00:00 TT.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -63_115_232_184_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const CXC_EPOCH: Self = Self::new(-63115232184000000000000000i128, Scale::TAI, Scale::TT);

    /// GPS/Galileo Experiment (GALEX) Time epoch.
    ///
    /// - 1980-01-06 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -630_763_181_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const GPS_EPOCH: Self = Self::new(-630763181000000000000000000i128, Scale::TAI, Scale::GPS);

    /// Galileo System Time (GST) epoch.
    ///
    /// - 1999-08-22 00:00:00 GST.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -11_447_981_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const GALILEO_EPOCH: Self =
        Self::new(-11447981000000000000000000i128, Scale::TAI, Scale::GST);

    /// BeiDou Time (BDT) epoch.
    ///
    /// - 2006-01-01 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - 189_345_633_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const BDT_EPOCH: Self = Self::new(189345633000000000000000000i128, Scale::TAI, Scale::BDT);

    /// CCSDS epoch (used in CCSDS time codes such as CUC).
    ///
    /// - 1958-01-01 00:00:00 TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -1_325_419_200_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const CCSDS_EPOCH: Self = Self::new(
        -1_325_419_200_000_000_000_000_000_000i128,
        Scale::TAI,
        Scale::TAI,
    );

    /// JD epoch (JD 0.0).
    ///
    /// - -4713-11-24 12:00:00
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -211_813_488_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const JD_EPOCH: Self = Self::new(
        -211_813_488_000_000_000_000_000_000_000i128,
        Scale::TAI,
        Scale::TAI,
    );

    /// MJD epoch (MJD 0.0)
    ///
    /// - 1858-11-17 00:00:00
    /// - Stored here on the **TAI** timescale as an offset from [`Dt::ZERO`](#associatedconstant.ZERO).
    /// - -4_453_444_800_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 12:00:00.
    pub const MJD_EPOCH: Self = Self::new(
        -4_453_444_800_000_000_000_000_000_000,
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
        Dt {
            attos,
            scale,
            target,
        }
    }

    /// Low level constructor from total attoseconds since a given epoch.
    ///
    /// Simply adds the total attoseconds to the epoch. Does not perform
    /// any time scale conversions.
    ///
    /// The returned [`Dt`] copies the epoch's `scale` and `target` fields.
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

    /// Builds a [`Dt`] holding the given whole seconds and sub-second remainder.
    ///
    /// The remainder is in **attoseconds**, not seconds. Pairs with
    /// [`Dt::to_sec64`](#method.to_sec64) + [`Dt::to_sec_frac`](#method.to_sec_frac).
    ///
    /// Does **not** perform any time scale conversions.
    ///
    /// ## Parameters
    ///
    /// - `sec` — whole seconds (truncating / signed-remainder split).
    /// - `attos` — fractional part of that split, in attoseconds.
    ///   Prefer helpers such as [`Dt::ms_to_attos`](#method.ms_to_attos) /
    ///   [`Dt::ns_to_attos`](#method.ns_to_attos) (or
    ///   [`AttosTraits`](../trait.AttosTraits.html))
    ///   instead of hand-counting zeros:
    ///   - `1.3` s → `sec = 1`, `attos = Dt::ms_to_attos(300)`
    ///   - `-1.3` s → `sec = -1`, `attos = Dt::ms_to_attos(-300)`
    ///   - `-0.5` s → `sec = 0`, `attos = Dt::ms_to_attos(-500)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale, dt};
    ///
    /// // 1.3 s — convert 300 ms of remainder to attoseconds
    /// let a = Dt::from_sec_and_frac(1, Dt::ms_to_attos(300), Scale::TAI, Scale::TAI);
    /// // same via AttosTraits on the integer
    /// let b = Dt::from_sec_and_frac(1, 300_i128.ms_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a, dt!(1_300_000_000_000_000_000));
    ///
    /// // -1.3 s (signed remainder)
    /// assert_eq!(
    ///     Dt::from_sec_and_frac(-1, Dt::ms_to_attos(-300), Scale::TAI, Scale::TAI),
    ///     dt!(-1_300_000_000_000_000_000),
    /// );
    ///
    /// // -0.5 s
    /// assert_eq!(
    ///     Dt::from_sec_and_frac(0, Dt::ms_to_attos(-500), Scale::TAI, Scale::TAI),
    ///     dt!(-500_000_000_000_000_000),
    /// );
    /// ```
    #[inline(always)]
    pub const fn from_sec_and_frac(sec: i128, attos: i128, on: Scale, target: Scale) -> Dt {
        Dt::new(
            sec.saturating_mul(ATTOS_PER_SEC_I128).saturating_add(attos),
            on,
            target,
        )
    }

    /// Builds a [`Dt`] holding the given whole seconds.
    ///
    /// Does **not** perform any time scale conversions. The `sec` count is stored
    /// as-is (converted only from seconds to attoseconds); its meaning depends on
    /// how you use the value afterward (for example as a library-epoch offset, a
    /// Unix offset passed to [`Dt::from_unix`](#method.from_unix), a duration, etc.).
    ///
    /// ## Parameters
    ///
    /// - `sec` — whole seconds count to store.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    #[inline(always)]
    pub const fn from_sec(sec: i128, on: Scale, target: Scale) -> Dt {
        Dt::new(sec.saturating_mul(ATTOS_PER_SEC_I128), on, target)
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
    ///   Use a smaller-unit converter rather than counting zeros by hand:
    ///   - `1.3` ms → `ms = 1`, `frac_attos = Dt::us_to_attos(300)` (0.3 ms = 300 µs)
    ///   - `-1.3` ms → `ms = -1`, `frac_attos = Dt::us_to_attos(-300)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.3 ms
    /// let a = Dt::from_ms(1, Dt::us_to_attos(300), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_ms(1, 300_i128.us_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_attos(), 1_300_000_000_000_000);
    ///
    /// // -1.3 ms
    /// let neg = Dt::from_ms(-1, Dt::us_to_attos(-300), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_attos(), -1_300_000_000_000_000);
    /// ```
    #[inline(always)]
    pub const fn from_ms(ms: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(ms, frac_attos, ATTOS_PER_MS_I128);
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
    ///   Use a smaller-unit converter rather than counting zeros by hand:
    ///   - `1.3` µs → `us = 1`, `frac_attos = Dt::ns_to_attos(300)` (0.3 µs = 300 ns)
    ///   - `-1.3` µs → `us = -1`, `frac_attos = Dt::ns_to_attos(-300)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.3 µs
    /// let a = Dt::from_us(1, Dt::ns_to_attos(300), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_us(1, 300_i128.ns_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_attos(), 1_300_000_000_000);
    ///
    /// // -1.3 µs
    /// let neg = Dt::from_us(-1, Dt::ns_to_attos(-300), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_attos(), -1_300_000_000_000);
    /// ```
    #[inline(always)]
    pub const fn from_us(us: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(us, frac_attos, ATTOS_PER_US_I128);
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
    ///   Use a smaller-unit converter rather than counting zeros by hand:
    ///   - `1.3` ns → `ns = 1`, `frac_attos = Dt::ps_to_attos(300)` (0.3 ns = 300 ps)
    ///   - `-1.3` ns → `ns = -1`, `frac_attos = Dt::ps_to_attos(-300)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.3 ns → whole nanoseconds + 300 ps remainder
    /// let a = Dt::from_ns(1, Dt::ps_to_attos(300), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_ns(1, 300_i128.ps_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_attos(), 1_300_000_000);
    ///
    /// // -1.3 ns
    /// let neg = Dt::from_ns(-1, Dt::ps_to_attos(-300), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_attos(), -1_300_000_000);
    /// ```
    #[inline(always)]
    pub const fn from_ns(ns: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(ns, frac_attos, ATTOS_PER_NS_I128);
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
    ///   Use a smaller-unit converter rather than counting zeros by hand:
    ///   - `1.3` ps → `ps = 1`, `frac_attos = Dt::fs_to_attos(300)` (0.3 ps = 300 fs)
    ///   - `-1.3` ps → `ps = -1`, `frac_attos = Dt::fs_to_attos(-300)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.3 ps
    /// let a = Dt::from_ps(1, Dt::fs_to_attos(300), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_ps(1, 300_i128.fs_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_attos(), 1_300_000);
    ///
    /// // -1.3 ps
    /// let neg = Dt::from_ps(-1, Dt::fs_to_attos(-300), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_attos(), -1_300_000);
    /// ```
    #[inline(always)]
    pub const fn from_ps(ps: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(ps, frac_attos, ATTOS_PER_PS_I128);
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
    ///   One femtosecond is 1000 attoseconds, so a fractional remainder is already
    ///   a small integer: `1.3` fs → `fs = 1`, `frac_attos = 300`.
    ///   For `-1.3` fs: `fs = -1`, `frac_attos = -300`.
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // 1.3 fs — sub-fs remainder is already in attoseconds (×10³)
    /// let a = Dt::from_fs(1, 300, Scale::TAI, Scale::TAI);
    /// assert_eq!(a.to_attos(), 1_300);
    ///
    /// // whole fs only — still fine to use the converter for the whole part
    /// // if you are building total attos by hand:
    /// assert_eq!(Dt::fs_to_attos(1), 1_000);
    ///
    /// // -1.3 fs
    /// let neg = Dt::from_fs(-1, -300, Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_attos(), -1_300);
    /// ```
    #[inline(always)]
    pub const fn from_fs(fs: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(fs, frac_attos, ATTOS_PER_FS_I128);
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
    ///   Use a time-unit converter rather than counting zeros by hand:
    ///   - `1.5` min → `n = 1`, `frac_attos = Dt::sec_to_attos(30)` (0.5 min = 30 s)
    ///   - `-1.5` min → `n = -1`, `frac_attos = Dt::sec_to_attos(-30)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.5 min
    /// let a = Dt::from_mins(1, Dt::sec_to_attos(30), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_mins(1, 30_i128.sec_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_sec(), 90);
    ///
    /// // -1.5 min
    /// let neg = Dt::from_mins(-1, Dt::sec_to_attos(-30), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_sec(), -90);
    /// ```
    #[inline(always)]
    pub const fn from_mins(n: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(n, frac_attos, ATTOS_PER_MIN);
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
    ///   Use a time-unit converter rather than counting zeros by hand:
    ///   - `1.5` h → `n = 1`, `frac_attos = Dt::mins_to_attos(30)` (0.5 h = 30 min)
    ///   - `-1.5` h → `n = -1`, `frac_attos = Dt::mins_to_attos(-30)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.5 h
    /// let a = Dt::from_hours(1, Dt::mins_to_attos(30), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_hours(1, 30_i128.mins_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_sec(), 5400);
    ///
    /// // -1.5 h
    /// let neg = Dt::from_hours(-1, Dt::mins_to_attos(-30), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_sec(), -5400);
    /// ```
    #[inline(always)]
    pub const fn from_hours(n: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(n, frac_attos, ATTOS_PER_HOUR);
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
    ///   Use a time-unit converter rather than counting zeros by hand:
    ///   - `1.25` d → `d = 1`, `frac_attos = Dt::hours_to_attos(6)` (0.25 d = 6 h)
    ///   - `-1.25` d → `d = -1`, `frac_attos = Dt::hours_to_attos(-6)`
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{AttosTraits, Dt, Scale};
    ///
    /// // 1.25 d
    /// let a = Dt::from_days(1, Dt::hours_to_attos(6), Scale::TAI, Scale::TAI);
    /// let b = Dt::from_days(1, 6_i128.hours_to_attos(), Scale::TAI, Scale::TAI);
    /// assert_eq!(a, b);
    /// assert_eq!(a.to_sec(), 108_000); // 1.25 * 86400
    ///
    /// // -1.25 d
    /// let neg = Dt::from_days(-1, Dt::hours_to_attos(-6), Scale::TAI, Scale::TAI);
    /// assert_eq!(neg.to_sec(), -108_000);
    /// ```
    #[inline(always)]
    pub const fn from_days(d: i128, frac_attos: i128, on: Scale, target: Scale) -> Dt {
        let attos = Dt::unit_to_total_attos(d, frac_attos, ATTOS_PER_DAY);
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

    /// Builds a [`Dt`] holding the given floating-point seconds count.
    ///
    /// Does **not** perform any time scale conversions. The `sec` value is
    /// stored as attoseconds only; its meaning depends on how you use the
    /// result afterward.
    ///
    /// ## Parameters
    ///
    /// - `sec` — seconds count to store (`NaN` → zero attoseconds;
    ///   `±∞` → [`i128::MAX`] / [`i128::MIN`]).
    /// - `on` — value stored in the returned [`Dt`]'s `scale` field.
    /// - `target` — value stored in the returned [`Dt`]'s `target` field.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let seconds = 5.5;
    /// let duration = Dt::from_sec_f(seconds, Scale::TAI, Scale::TAI);
    ///
    /// assert_eq!(duration.to_sec_f(), seconds);
    /// ```
    #[inline]
    pub const fn from_sec_f(sec: Real, on: Scale, target: Scale) -> Dt {
        if sec.is_nan() {
            return Self::new(0, on, target);
        } else if sec.is_infinite() {
            return if sec.is_sign_positive() {
                Self::new(i128::MAX, on, target)
            } else {
                Self::new(i128::MIN, on, target)
            };
        }
        Dt::new(Self::sec_f_to_attos(sec), on, target)
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
