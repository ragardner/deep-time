use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC_I128, ATTOS_PER_US, Dt,
    Real, SEC_PER_DAYI64, SEC_PER_DAYI128, SEC_PER_WEEK, Scale,
    TAI_SECS_1970_MIDNIGHT_TO_2000_NOON,
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
        (SEC_PER_DAYI64 as i128) * ATTOS_PER_SEC_I128,
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

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - **Requires** a seconds and attoseconds count such that would be returned from the
    ///   functions [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64) and
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
            Dt::from_attos((sec as i128) * ATTOS_PER_SEC_I128, scale)
        } else {
            Dt::from_attos((sec as i128) * ATTOS_PER_SEC_I128 + (attos as i128), scale)
        }
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - **Requires** a seconds and attoseconds count such that would be returned from the
    ///   functions [`Dt::to_sec64`](../struct.Dt.html#method.to_sec64) and
    ///   **[`Dt::to_sec_frac`](../struct.Dt.html#method.to_sec_frac)**.
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
    pub fn from_sec_and_attos(sec: i64, attos: u64, scale: Scale) -> Dt {
        let attos = Dt::sec_and_attos_to_attos(sec, attos);
        Dt::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total attoseconds value.
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
    pub const fn from_attos(attos: i128, scale: Scale) -> Dt {
        Dt::new(attos, scale, scale).to_tai()
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total attoseconds value.
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
    pub const fn from_attos_with_target(attos: i128, scale: Scale, target: Scale) -> Dt {
        Dt::new(attos, scale, target).to_tai()
    }

    /// Creates a new [`Dt`] from a total number of seconds (signed i128) without
    /// performing any time scale conversions.
    #[inline(always)]
    pub const fn from_tai_sec(sec: i128) -> Dt {
        Self::from_attos(sec.saturating_mul(ATTOS_PER_SEC_I128), Scale::TAI)
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
        Self::from_attos(sec.saturating_mul(ATTOS_PER_SEC_I128), scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total milliseconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    #[inline(always)]
    pub const fn from_ms(ms: i128, scale: Scale) -> Dt {
        let attos = ms.saturating_mul(ATTOS_PER_MS as i128);
        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total microseconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    #[inline(always)]
    pub const fn from_us(us: i128, scale: Scale) -> Dt {
        let attos = us.saturating_mul(ATTOS_PER_US as i128);
        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total nanoseconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    #[inline(always)]
    pub const fn from_ns(ns: i128, scale: Scale) -> Dt {
        let attos = ns.saturating_mul(ATTOS_PER_NS as i128);
        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total picoseconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    #[inline(always)]
    pub const fn from_ps(ps: i128, scale: Scale) -> Dt {
        let attos = ps.saturating_mul(ATTOS_PER_PS as i128);
        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Requires a total femtoseconds value.
    /// - The value should be from the epoch TAI 2000-01-01 12:00:00.
    /// - The returned object's `scale` field is set to TAI and its `target` field is set to
    ///   the given `scale` arg.
    #[inline(always)]
    pub const fn from_fs(fs: i128, scale: Scale) -> Dt {
        let attos = fs.saturating_mul(ATTOS_PER_FS as i128);
        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// Convenience wrapper around
    /// [`Dt::from_sec`](../struct.Dt.html#method.from_sec).
    #[inline(always)]
    pub const fn from_min(m: i64, scale: Scale) -> Dt {
        Self::from_sec((m as i128) * 60, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// Convenience wrapper around
    /// [`Dt::from_sec`](../struct.Dt.html#method.from_sec).
    #[inline(always)]
    pub const fn from_hr(h: i64, scale: Scale) -> Dt {
        Self::from_sec((h as i128) * 3600, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Params are hours, minutes, seconds, milliseconds, microseconds, and nanoseconds.
    /// - All values are essentially optional (you can use 0 for ones you want to leave out).
    /// - Negative values are handled.
    /// - Uses saturating arithmetic.
    pub const fn from_hms(
        hr: i64,
        min: i64,
        sec: i64,
        ms: i128,
        us: i128,
        ns: i128,
        scale: Scale,
    ) -> Dt {
        // Combine hours/minutes/seconds with saturating arithmetic
        let total_sec: i128 = (hr as i128)
            .saturating_mul(3600)
            .saturating_add((min as i128).saturating_mul(60))
            .saturating_add(sec as i128);

        // Combine sub-second parts (nanoseconds) with saturating arithmetic
        let sub_ns: i128 = ms
            .saturating_mul(1_000_000)
            .saturating_add(us.saturating_mul(1_000))
            .saturating_add(ns);

        if sub_ns == 0 {
            return Self::from_sec(total_sec, scale);
        }

        // Handle carry/borrow from sub-second component
        let abs_ns: u128 = sub_ns.unsigned_abs();
        let extra_sec: i128 = (abs_ns / 1_000_000_000) as i128;
        let rem_ns: u64 = (abs_ns % 1_000_000_000) as u64;
        let frac_attos: u128 = (rem_ns as u128) * (ATTOS_PER_NS as u128);

        let attos = if sub_ns >= 0 {
            total_sec
                .saturating_add(extra_sec)
                .saturating_mul(ATTOS_PER_SEC_I128)
                .saturating_add(frac_attos as i128)
        } else if frac_attos == 0 {
            total_sec
                .saturating_sub(extra_sec)
                .saturating_mul(ATTOS_PER_SEC_I128)
        } else {
            total_sec
                .saturating_sub(extra_sec)
                .saturating_sub(1)
                .saturating_mul(ATTOS_PER_SEC_I128)
                .saturating_add(ATTOS_PER_SEC_I128 - frac_attos as i128)
        };

        Self::from_attos(attos, scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Convenience wrapper around
    ///   [`Dt::from_sec`](../struct.Dt.html#method.from_sec).
    /// - Uses `86400` seconds per day in the calculation.
    #[inline(always)]
    pub const fn from_days(d: i64, scale: Scale) -> Dt {
        Self::from_sec((d as i128).saturating_mul(SEC_PER_DAYI128), scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Convenience wrapper around
    ///   [`Dt::from_sec`](../struct.Dt.html#method.from_sec).
    /// - Uses `604800` seconds per week in the calculation.
    #[inline(always)]
    pub const fn from_wk(wk: i64, scale: Scale) -> Dt {
        Dt::from_sec((wk as i128).saturating_mul(SEC_PER_WEEK as i128), scale)
    }

    /// Returns a [`Dt`] on the TAI time scale, after having been **converted** to TAI from
    /// the given `scale`.
    ///
    /// - Convenience wrapper around
    ///   [`Dt::from_sec`](../struct.Dt.html#method.from_sec).
    /// - Uses `31_557_600` in the calculation.
    #[inline(always)]
    pub const fn from_yr(yr: i64, scale: Scale) -> Dt {
        Dt::from_sec((yr as i128).saturating_mul(31_557_600), scale)
    }

    /// Returns a [`Dt`] that is this duration ago from the given scale.
    #[inline(always)]
    pub const fn ago(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).sub(self)
    }

    /// Returns a [`Dt`] that is this duration from now in the given scale.
    #[inline(always)]
    pub const fn from_now(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).add(self)
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
    /// - Converts the values to TAI from the given `scale`.
    /// - The returned [`Dt`] is on the TAI time scale.
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
        Self::from_attos(Self::sec_f_to_attos(sec), scale)
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
        .add(Dt::from_ns(nanos as i128, Scale::TAI))
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
        .add(Dt::from_ns(nanos as i128, Scale::TAI))
    }
}
