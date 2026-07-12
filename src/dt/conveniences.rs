use crate::{
    ATTOS_PER_DAY, ATTOS_PER_SEC_I128, ATTOS_PER_WEEK, Dt, JD_2000_2_451_545F, Real, SEC_PER_DAY_F,
    SEC_PER_DAY_I64, Scale,
};

impl Dt {
    /// Returns this [`Dt`] but as time since the
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH).
    /// - The count is on whatever scale sits in this [`Dt`]'s `target` field — for example
    ///   `Scale::UTC` if you built it with `from_ymd(..., Scale::UTC, ...)`. The result's
    ///   `scale` and `target` are both set to that same value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // because from_ymd() with Scale::UTC sets the returned
    /// // Dt's target field to Scale::UTC, we do not need to use
    /// // .target() prior to calling to_unix() in order to get
    /// // a utc unix timestamp
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    /// let unix = dt.to_unix();
    ///
    /// assert_eq!(
    ///     unix.to_sec(),
    ///     946728000,
    ///     "unix sec for 2000-01-01 12:00:00 UTC is wrong, got: {}, expected: 946728000",
    ///     unix.to_sec()
    /// );
    ///
    /// let dt2 = Dt::from_unix(unix);
    ///
    /// assert_eq!(
    ///     dt.to_attos(), dt2.to_attos(),
    ///     "round trip to Dt got wrong attos, old: {}, new: {}",
    ///     dt.to_attos(), dt2.to_attos()
    /// );
    ///
    /// let ymd = dt2.to_ymd();
    /// assert_eq!(ymd.yr(), 2000_i64);
    /// assert_eq!(ymd.mo(), 1);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 12);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_unix`](../struct.Dt.html#method.from_unix)
    #[inline(always)]
    pub const fn to_unix(&self) -> Dt {
        self.to_scale_and_diff(Self::UNIX_EPOCH, true)
    }

    /// Creates a **TAI** [`Dt`] from a [`Dt`] that is attoseconds since
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_unix`](../struct.Dt.html#method.to_unix).
    ///
    /// ## Important:
    ///
    /// - `unix` must be a [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) — typically the
    ///   return value of [`Dt::to_unix`](../struct.Dt.html#method.to_unix).
    ///   The input's `scale` field says which time scale that count is on — if it
    ///   is `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `attos` is no longer a count since
    /// [`UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) — it is attoseconds since
    /// the library epoch (2000-01-01 noon TAI). Its `target` field is taken from `unix`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 0);
    /// let unix = dt.to_unix();
    /// let roundtrip = Dt::from_unix(unix);
    ///
    /// assert_eq!(roundtrip, dt);
    /// ```
    ///
    /// ### From an external POSIX unix seconds count
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // 2012-08-08 15:30:00 → 1344439800.000000 s
    /// let unix = 1344439800_i128;
    ///
    /// // no scale conversion — only labels the count as UTC seconds
    /// let unix_dt = Dt::from_sec(unix, Scale::UTC, Scale::UTC);
    ///
    /// let dt = Dt::from_unix(unix_dt);
    ///
    /// let ymd = dt.to_ymd();
    /// assert_eq!(ymd.yr(), 2012);
    /// assert_eq!(ymd.mo(), 8);
    /// assert_eq!(ymd.day(), 8);
    /// assert_eq!(ymd.hr(), 15);
    /// assert_eq!(ymd.min(), 30);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
    #[inline(always)]
    pub const fn from_unix(unix: Dt) -> Dt {
        Self::from_diff_and_scale(unix, Dt::UNIX_EPOCH, true)
    }

    /// Interprets a POSIX Unix nanosecond count as UTC elapsed time since the Unix
    /// epoch.
    ///
    /// **Differs** with [`from_unix`](../struct.Dt.html#method.from_unix) in that
    /// it assumes the nanoseconds are on the UTC time scale and converts from UTC ->
    /// TAI (adding any leap seconds to the end result).
    #[inline(always)]
    pub const fn from_unix_ns(ns: i128) -> Dt {
        Dt::from_unix(Dt::new(Dt::ns_to_attos(ns), Scale::UTC, Scale::UTC))
    }

    /// Interprets a POSIX Unix millisecond count as UTC elapsed time since the Unix
    /// epoch.
    ///
    /// **Differs** with [`from_unix`](../struct.Dt.html#method.from_unix) in that
    /// it assumes the milliseconds are on the UTC time scale and converts from UTC ->
    /// TAI (adding any leap seconds to the end result).
    #[inline(always)]
    pub const fn from_unix_ms(ms: i128) -> Dt {
        Dt::from_unix(Dt::new(Dt::ms_to_attos(ms), Scale::UTC, Scale::UTC))
    }

    /// Returns this [`Dt`] as a day count since
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH)
    /// (1970-01-01 00:00:00) on its `target` time scale.
    ///
    /// This is the day-granularity counterpart to
    /// [`Dt::to_unix`](../struct.Dt.html#method.to_unix): elapsed time since the
    /// Unix epoch is split into whole days plus a sub-day fractional part.
    ///
    /// ## Important:
    ///
    /// - Uses [`Dt::to_unix`](../struct.Dt.html#method.to_unix) internally: this [`Dt`]
    ///   and [`UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) are both
    ///   converted to the `target` time scale before differencing.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling if you need
    ///   the count on a particular time scale, e.g. `Scale::UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A (days, frac attos) pair where:
    ///
    /// - days (`i128`): whole days elapsed since
    ///   [`UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH)
    ///   on the `target` scale.
    /// - `frac` ([`Dt`]): sub-day fractional part in attoseconds since the start of
    ///   that day. `frac.attos` is always in `[0, ATTOS_PER_DAY)`.
    /// - `frac.scale` and `frac.target` match the
    ///   [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
    ///   result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, consts::ATTOS_PER_HALF_DAY_U128};
    ///
    /// let epoch = Dt::from_ymd(1970, 1, 1, Scale::UTC, 0, 0, 0, 0);
    /// let (days, frac) = epoch.to_unix_days();
    /// assert_eq!(days, 0);
    /// assert_eq!(frac.to_attos(), 0);
    ///
    /// let neg = Dt::from_ymd(1969, 12, 31, Scale::UTC, 12, 0, 0, 0);
    /// let (days, frac) = neg.to_unix_days();
    /// assert_eq!(days, -1);
    /// assert_eq!(frac.to_days_f(), 0.5);
    ///
    /// let roundtrip = Dt::from_unix_days(days, frac);
    /// assert_eq!(roundtrip, neg);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_unix_days`](../struct.Dt.html#method.from_unix_days)
    /// - [`Dt::to_unix_days_f`](../struct.Dt.html#method.to_unix_days_f)
    /// - [`Dt::to_unix`](../struct.Dt.html#method.to_unix)
    #[inline(always)]
    pub const fn to_unix_days(&self) -> (i128, Dt) {
        let unix = self.to_unix();
        let (days, attos) = unix.to_days_floor();
        (days, Dt::new(Self::to_i128(attos), unix.scale, unix.target))
    }

    /// Creates a **TAI** [`Dt`] from a day count since
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_unix_days`](../struct.Dt.html#method.to_unix_days).
    ///
    /// ## Important:
    ///
    /// - `days` and `frac.attos` are interpreted on `frac.scale` — if it is
    ///   `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `target` field is taken from
    /// `frac.target`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale, from_ymd};
    ///
    /// let dt = from_ymd!(2000, 1, 1; 12);
    /// let (days, frac) = dt.to_unix_days();
    /// let roundtrip = Dt::from_unix_days(days, frac);
    ///
    /// assert_eq!(roundtrip, dt);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_unix_days`](../struct.Dt.html#method.to_unix_days)
    /// - [`Dt::from_unix_days_f`](../struct.Dt.html#method.from_unix_days_f)
    /// - [`Dt::from_unix`](../struct.Dt.html#method.from_unix)
    pub const fn from_unix_days(days: i128, frac: Dt) -> Dt {
        let total_attos = days
            .saturating_mul(ATTOS_PER_DAY)
            .saturating_add(frac.attos);

        Self::from_unix(Dt::new(total_attos, frac.scale, frac.target))
    }

    /// Returns the day count since
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH) as a floating-point
    /// `Real`.
    ///
    /// This is the lossy counterpart to
    /// [`Dt::to_unix_days`](../struct.Dt.html#method.to_unix_days).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_unix_days`](../struct.Dt.html#method.to_unix_days)
    /// - [`Dt::from_unix_days_f`](../struct.Dt.html#method.from_unix_days_f)
    #[inline]
    pub const fn to_unix_days_f(&self) -> Real {
        let (days, frac) = self.to_unix_days();
        f!(days) + f!(frac.attos) / f!(ATTOS_PER_DAY)
    }

    /// Creates a **TAI** [`Dt`] from a floating-point day count since
    /// [`Dt::UNIX_EPOCH`](../struct.Dt.html#associatedconstant.UNIX_EPOCH).
    ///
    /// This is the inverse of
    /// [`Dt::to_unix_days_f`](../struct.Dt.html#method.to_unix_days_f).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_unix_days_f`](../struct.Dt.html#method.to_unix_days_f)
    /// - [`Dt::from_unix_days`](../struct.Dt.html#method.from_unix_days)
    #[inline(always)]
    pub const fn from_unix_days_f(days: Real, on: Scale) -> Dt {
        Self::from_unix(Dt::new(Dt::sec_f_to_attos(days * SEC_PER_DAY_F), on, on))
    }

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH).
    /// - The count is on whatever scale sits in this [`Dt`]'s `target` field — for example
    ///   `Scale::UTC` if you built it with `from_ymd(..., Scale::UTC, ...)`. The result's
    ///   `scale` and `target` are both set to that same value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// // 2698012800
    /// let dt = Dt::from_ymd(1985, 7, 1, Scale::TAI, 0, 0, 0, 0);
    /// let ntp = dt.to_ntp();
    ///
    /// assert_eq!(
    ///     ntp.to_attos(), Dt::sec_to_attos(2698012800_i128),
    ///     "ntp sec for 1985 is wrong, got: {}, expected: {}",
    ///     ntp.to_attos(), Dt::sec_to_attos(2698012800_i128)
    /// );
    ///
    /// let dt2 = Dt::from_ntp(ntp);
    ///
    /// assert_eq!(
    ///     dt.to_attos(), dt2.to_attos(),
    ///     "round trip to Dt got wrong sec, old: {}, new: {}",
    ///     dt.to_attos(), dt2.to_attos()
    /// );
    ///
    /// let ymd = dt2.to_ymd();
    /// assert_eq!(ymd.yr(), 1985_i64);
    /// assert_eq!(ymd.mo(), 7);
    /// assert_eq!(ymd.day(), 1);
    /// assert_eq!(ymd.hr(), 0);
    /// assert_eq!(ymd.min(), 0);
    /// assert_eq!(ymd.sec(), 0);
    /// assert_eq!(ymd.attos(), 0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_ntp`](../struct.Dt.html#method.from_ntp)
    #[inline(always)]
    pub const fn to_ntp(&self) -> Dt {
        self.to_scale_and_diff(Self::NTP_EPOCH, true)
    }

    /// Creates a **TAI** [`Dt`] from a [`Dt`] that is attoseconds since
    /// [`Dt::NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_ntp`](../struct.Dt.html#method.to_ntp).
    ///
    /// ## Important:
    ///
    /// - `ntp` must be a [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH) — typically the
    ///   return value of [`Dt::to_ntp`](../struct.Dt.html#method.to_ntp)
    /// - The input's `scale` field says which time scale that count is on — if it
    ///   is `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `attos` is no longer a count since
    /// [`NTP_EPOCH`](../struct.Dt.html#associatedconstant.NTP_EPOCH) — it is attoseconds since
    /// the library epoch (2000-01-01 noon TAI). Its `target` field is taken from `ntp`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let dt = Dt::from_ymd(1985, 7, 1, Scale::TAI, 0, 0, 0, 0);
    /// let ntp = dt.to_ntp();
    /// let roundtrip = Dt::from_ntp(ntp);
    ///
    /// assert_eq!(roundtrip, dt);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_ntp`](../struct.Dt.html#method.to_ntp)
    #[inline(always)]
    pub const fn from_ntp(ntp: Dt) -> Dt {
        Self::from_diff_and_scale(ntp, Self::NTP_EPOCH, true)
    }

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g.
    ///   `.target(Scale::GPS)`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH).
    /// - The count is on whatever scale sits in this [`Dt`]'s `target` field — for example
    ///   `Scale::GPS` after `.target(Scale::GPS)`. The result's `scale` and `target` are both
    ///   set to that same value.
    ///
    /// ## See also
    ///
    /// - [`Dt::from_gps`](../struct.Dt.html#method.from_gps)
    /// - [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd)
    /// - [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    ///
    /// ## Implementation
    ///
    /// `convert_epoch` is `true`. If we did not convert the epoch, we would not get seconds
    /// since the GPS epoch; we would get seconds since something else.
    ///
    /// [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd) / [`Dt::to_ymd`](../struct.Dt.html#method.to_ymd)
    /// do the opposite: if they converted the epoch too, the difference would cancel out. See
    /// [`to_ymd`](../struct.Dt.html#method.to_ymd).
    #[inline(always)]
    pub const fn to_gps(&self) -> Dt {
        self.to_scale_and_diff(Self::GPS_EPOCH, true)
    }

    /// Creates a **TAI** [`Dt`] from a [`Dt`] that is attoseconds since
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_gps`](../struct.Dt.html#method.to_gps).
    ///
    /// ## Important:
    ///
    /// - `elapsed` must be a [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) — typically the
    ///   return value of [`Dt::to_gps`](../struct.Dt.html#method.to_gps)
    ///   The input's `scale` field says which time scale that count is on — if it
    ///   is `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `attos` is no longer a count since
    /// [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) — it is attoseconds since
    /// the library epoch (2000-01-01 noon TAI). Its `target` field is taken from `elapsed`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let gps = x.target(Scale::GPS).to_gps();
    /// let roundtrip = Dt::from_gps(gps);
    ///
    /// assert_eq!(roundtrip, x);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_gps`](../struct.Dt.html#method.to_gps)
    /// - [`Dt::from_gps_wk_and_tow`](../struct.Dt.html#method.from_gps_wk_and_tow)
    #[inline(always)]
    pub const fn from_gps(elapsed: Dt) -> Dt {
        Self::from_diff_and_scale(elapsed, Self::GPS_EPOCH, true)
    }

    /// Returns the GPS week number and Time of Week (TOW) for this instant.
    ///
    /// Elapsed time since [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH)
    /// is computed by [`Dt::to_gps`](../struct.Dt.html#method.to_gps) — on this [`Dt`]'s
    /// `target` time scale — and then split into whole weeks plus a remainder.
    ///
    /// This is the inverse of
    /// [`Dt::from_gps_wk_and_tow`](../struct.Dt.html#method.from_gps_wk_and_tow).
    ///
    /// ## Important:
    ///
    /// - Uses [`Dt::to_gps`](../struct.Dt.html#method.to_gps) internally: this [`Dt`] and
    ///   [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) are both converted
    ///   to the `target` time scale before differencing.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling if you need
    ///   week/TOW on a particular time scale, e.g. `Scale::GPS`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// A `(week, tow)` pair:
    ///
    /// - `week` (`i64`): whole weeks in the elapsed time from
    ///   [`Dt::to_gps`](../struct.Dt.html#method.to_gps). Week 0 starts at the GPS epoch
    ///   (1980-01-06). Before that date the elapsed time is negative and `div_euclid` yields a
    ///   negative week — this is not a broadcast GPS week number, just how the split is defined.
    ///   A plain integer is enough here; it is only a week count, not a duration in attoseconds.
    /// - `tow` ([`Dt`]): seconds-within-the-week as attoseconds in `0 .. 604800`. Its `scale` and
    ///   `target` are set to this [`Dt`]'s `target` so
    ///   [`Dt::from_gps_wk_and_tow`](../struct.Dt.html#method.from_gps_wk_and_tow) knows which
    ///   time scale the pair belongs to. `tow` is a [`Dt`] rather than a bare integer so
    ///   sub-second precision and scale are preserved together; the week number alone cannot
    ///   carry either. `div_euclid` / `rem_euclid` are used (not truncating `/`) so TOW stays
    ///   non-negative even when the elapsed time is negative.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    ///
    /// // for conventional GPS-time week/TOW, set target first:
    /// let g = x.target(Scale::GPS).to_gps_wk_and_tow();
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_gps_wk_and_tow`](../struct.Dt.html#method.from_gps_wk_and_tow)
    /// - [`Dt::to_gps`](../struct.Dt.html#method.to_gps)
    pub const fn to_gps_wk_and_tow(&self) -> (i64, Dt) {
        let total_attos = self.to_gps().to_attos();
        let wk = total_attos.div_euclid(ATTOS_PER_WEEK) as i64;
        let tow_attos = total_attos.rem_euclid(ATTOS_PER_WEEK);
        // was converted to target scale, scale is now target
        (wk, Dt::new(tow_attos, self.target, self.target))
    }

    /// Creates a [`Dt`] from a GPS week number and Time of Week (TOW).
    ///
    /// Recombines `week` and `tow` into elapsed time since
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH), then passes that to
    /// [`Dt::from_gps`](../struct.Dt.html#method.from_gps).
    ///
    /// This is the inverse of
    /// [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow).
    ///
    /// ## Important:
    ///
    /// - Uses [`Dt::from_gps`](../struct.Dt.html#method.from_gps) internally: the elapsed time
    ///   is interpreted on the `tow` [`Dt`]'s `scale` / `target` fields, and
    ///   [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) is converted to that
    ///   same scale before the sum.
    /// - Pass back the `tow` from [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow)
    ///   unchanged if you want a round trip.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `target` field is taken from `tow`.
    ///
    /// `tow` must be a [`Dt`] (not a bare second count) because
    /// [`Dt::from_gps`](../struct.Dt.html#method.from_gps) needs both the within-week attoseconds
    /// and the `scale` / `target` that say which time scale `week` and `tow` were expressed on.
    /// The week number is multiplied back into attoseconds (`week * 604800` seconds); only `tow`
    /// carries the scale and sub-week precision needed for the round trip.
    ///
    /// `tow` should be in `0 .. 604800` seconds, as returned by
    /// [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow). Negative `week`
    /// values only arise from dates before 1980-01-06 (see that function).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let g = x.to_gps_wk_and_tow();
    /// let z = Dt::from_gps_wk_and_tow(g.0, g.1);
    /// assert_eq!(x, z);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_gps_wk_and_tow`](../struct.Dt.html#method.to_gps_wk_and_tow)
    /// - [`Dt::from_gps`](../struct.Dt.html#method.from_gps)
    pub const fn from_gps_wk_and_tow(wk: i64, tow: Dt) -> Dt {
        let total_attos = (wk as i128)
            .saturating_mul(ATTOS_PER_WEEK)
            .saturating_add(tow.to_attos());

        Self::from_gps(Dt::new(total_attos, tow.scale, tow.target))
    }

    /// Returns the day of the GPS week (0 = Sunday, 1 = Monday, …, 6 = Saturday).
    ///
    /// This value is computed directly from the GPS Time of Week and is
    /// independent of the Gregorian calendar or civil time.
    pub const fn to_gps_day_of_wk(&self) -> u8 {
        let (_, tow) = self.to_gps_wk_and_tow();
        let secs = tow.to_attos() / ATTOS_PER_SEC_I128;

        (secs / SEC_PER_DAY_I64 as i128) as u8
    }

    /// Returns this [`Dt`] but as time since the
    /// [`Dt::CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH) on its
    /// `target` time scale.
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the epoch to the time scale of its
    ///   `target` field before doing a raw difference with the epoch.
    /// - **You may need to change the [`Dt`]'s `target` field** before calling the function
    ///   if you need the timestamp to be on a particular time scale, e.g. `UTC`.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon epoch,
    ///   if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH).
    /// - The count is on whatever scale sits in this [`Dt`]'s `target` field — for example
    ///   `Scale::TT` after `.target(Scale::TT)`. The result's `scale` and `target` are both
    ///   set to that same value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let cxc = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0)
    ///     .target(Scale::TT)
    ///     .to_cxcsec()
    ///     .to_sec_f();
    ///
    /// // cxcsec 694224032.184 (matches Astropy)
    /// assert_eq!(cxc, 694224032.184);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_cxcsec`](../struct.Dt.html#method.from_cxcsec)
    #[inline(always)]
    pub const fn to_cxcsec(&self) -> Dt {
        self.to_scale_and_diff(Self::CXC_EPOCH, true)
    }

    /// Creates a **TAI** [`Dt`] from a [`Dt`] that is attoseconds since
    /// [`Dt::CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_cxcsec`](../struct.Dt.html#method.to_cxcsec).
    ///
    /// ## Important:
    ///
    /// - `elapsed` must be a [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH) — typically the
    ///   return value of [`Dt::to_cxcsec`](../struct.Dt.html#method.to_cxcsec)
    ///   The input's `scale` field says which time scale that count is on — if it
    ///   is `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `attos` is no longer a count since
    /// [`CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH) — it is attoseconds since
    /// the library epoch (2000-01-01 noon TAI). Its `target` field is taken from `elapsed`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// let cxc = x.target(Scale::TT).to_cxcsec();
    /// let roundtrip = Dt::from_cxcsec(cxc);
    ///
    /// assert_eq!(roundtrip, x);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_cxcsec`](../struct.Dt.html#method.to_cxcsec)
    /// - [`Dt::from_cxcsec_f`](../struct.Dt.html#method.from_cxcsec_f)
    #[inline(always)]
    pub const fn from_cxcsec(elapsed: Dt) -> Dt {
        Self::from_diff_and_scale(elapsed, Self::CXC_EPOCH, true)
    }

    /// Convenience wrapper around
    /// [`Dt::from_cxcsec`](../struct.Dt.html#method.from_cxcsec)
    /// for a bare floating-point second count.
    ///
    /// ## Parameters
    ///
    /// - `sec` — seconds elapsed since
    ///   [`CXC_EPOCH`](../struct.Dt.html#associatedconstant.CXC_EPOCH).
    /// - `on` — which [`Scale`] the count is measured in (for example `Scale::TT` or
    ///   `Scale::UTC`). This becomes the wrapped [`Dt`]'s `scale`;
    ///   [`Dt::from_cxcsec`](../struct.Dt.html#method.from_cxcsec)
    ///   then uses it when turning the elapsed count into an absolute TAI instant
    ///   (including leap-second handling where applicable). Same role as the `scale`
    ///   field on the [`Dt`] you would hand to
    ///   [`Dt::from_cxcsec`](../struct.Dt.html#method.from_cxcsec)
    ///   directly.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// let cxc = x.target(Scale::TT).to_cxcsec().to_sec_f();
    /// let roundtrip = Dt::from_cxcsec_f(cxc, Scale::TT);
    ///
    /// assert_eq!(roundtrip.to_cxcsec().to_sec_f(), cxc);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_cxcsec`](../struct.Dt.html#method.from_cxcsec)
    /// - [`Dt::to_cxcsec`](../struct.Dt.html#method.to_cxcsec)
    #[inline(always)]
    pub const fn from_cxcsec_f(sec: Real, on: Scale) -> Dt {
        Self::from_cxcsec(Dt::new(Dt::sec_f_to_attos(sec), on, on))
    }

    /// Returns the elapsed time since the GALEX epoch as a [`Dt`] expressed
    /// in this object's current `target` scale.
    ///
    /// This method can match Astropy’s `Time.galexsec` format. To match
    /// Astropy output, set `.target(Scale::UTC)`
    /// before calling.
    ///
    /// The GALEX epoch is
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH)
    /// (same epoch used by GPS time).
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself and the [`Dt::GPS_EPOCH`] to the time
    ///   scale of its `target` field before doing a raw difference with the epoch.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch, if it's not then the output will be incorrect.
    ///
    /// ## Returns
    ///
    /// - A [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH).
    /// - The count is on whatever scale sits in this [`Dt`]'s `target` field — for example
    ///   `Scale::UTC` after `.target(Scale::UTC)`. The result's `scale` and `target` are both
    ///   set to that same value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let galexsec = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0)
    ///     .target(Scale::UTC)
    ///     .to_galexsec()
    ///     .to_sec_f();
    ///
    /// assert_eq!(galexsec, 1261871963.0);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec)
    #[inline(always)]
    pub const fn to_galexsec(&self) -> Dt {
        self.to_scale_and_diff(Self::GPS_EPOCH, true)
    }

    /// Creates a **TAI** [`Dt`] from a [`Dt`] that is attoseconds since
    /// [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH).
    ///
    /// This is the inverse of [`Dt::to_galexsec`](../struct.Dt.html#method.to_galexsec).
    /// GALEX seconds use the same epoch as GPS time.
    ///
    /// ## Important:
    ///
    /// - `elapsed` must be a [`Dt`] whose `attos` is how many attoseconds have elapsed since
    ///   [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) — typically the
    ///   return value of [`Dt::to_galexsec`](../struct.Dt.html#method.to_galexsec)
    ///   The input's `scale` field says which time scale that count is on — if it
    ///   is `Scale::UTC`, the count is treated as UTC and converted to TAI (leap seconds
    ///   included).
    /// - [`Dt::GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) is converted
    ///   to that same scale before the sum.
    ///
    /// ## Returns
    ///
    /// A **TAI** [`Dt`] for the reconstructed instant. Its `attos` is no longer a count since
    /// [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH) — it is attoseconds since
    /// the library epoch (2000-01-01 noon TAI). Its `target` field is taken from `elapsed`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// let galex = x.target(Scale::UTC).to_galexsec();
    /// let roundtrip = Dt::from_galexsec(galex);
    ///
    /// assert_eq!(roundtrip, x);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::to_galexsec`](../struct.Dt.html#method.to_galexsec)
    /// - [`Dt::from_galexsec_f`](../struct.Dt.html#method.from_galexsec_f)
    #[inline(always)]
    pub const fn from_galexsec(elapsed: Dt) -> Dt {
        Self::from_diff_and_scale(elapsed, Self::GPS_EPOCH, true)
    }

    /// Convenience wrapper around
    /// [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec)
    /// for a bare floating-point second count.
    ///
    /// ## Parameters
    ///
    /// - `sec` — seconds elapsed since
    ///   [`GPS_EPOCH`](../struct.Dt.html#associatedconstant.GPS_EPOCH).
    /// - `on` — which [`Scale`] the count is measured in (for example `Scale::UTC` or
    ///   `Scale::TT`). This becomes the wrapped [`Dt`]'s `scale`;
    ///   [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec)
    ///   then uses it when turning the elapsed count into an absolute TAI instant
    ///   (including leap-second handling where applicable). Same role as the `scale`
    ///   field on the [`Dt`] you would hand to
    ///   [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec) directly.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// let galex = x.target(Scale::UTC).to_galexsec().to_sec_f();
    /// let roundtrip = Dt::from_galexsec_f(galex, Scale::UTC);
    ///
    /// assert_eq!(roundtrip, x);
    /// ```
    ///
    /// ## See also
    ///
    /// - [`Dt::from_galexsec`](../struct.Dt.html#method.from_galexsec)
    /// - [`Dt::to_galexsec`](../struct.Dt.html#method.to_galexsec)
    #[inline(always)]
    pub const fn from_galexsec_f(sec: Real, on: Scale) -> Dt {
        Self::from_galexsec(Dt::new(Dt::sec_f_to_attos(sec), on, on))
    }

    /// Returns the **Julian epoch year** (JYEAR) for this instant.
    ///
    /// Julian years are defined as exactly 365.25 days of 86400 SI seconds.
    /// This is the system used for J2000.0 and many astronomical calculations.
    ///
    /// This is **not** the same as
    /// [`Dt::to_decimalyear`](../struct.Dt.html#method.to_decimalyear),
    /// which uses the actual length of the specific Gregorian year.
    ///
    /// This is the inverse of
    /// [`Dt::from_jyear`](../struct.Dt.html#method.from_jyear).
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch, if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
    ///
    /// assert_eq!(x.to_jyear(), 2019.9986310746065);
    /// ```
    #[inline(always)]
    pub const fn to_jyear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(2000.0) + (jd_tt - JD_2000_2_451_545F) / f!(365.25)
    }

    /// Inverse of
    /// [`Dt::to_jyear`](../struct.Dt.html#method.to_jyear).
    pub const fn from_jyear(jyear: Real, scale: Scale) -> Dt {
        if jyear.is_nan() {
            return Self::ZERO;
        }
        if jyear.is_infinite() {
            return if jyear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = JD_2000_2_451_545F + (jyear - f!(2000.0)) * f!(365.25);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **Besselian epoch year** (BYEAR) for this instant.
    ///
    /// Besselian years are an older astronomical convention based on a
    /// tropical year length of approximately 365.242198781 days.
    ///
    /// This is the inverse of
    /// [`Dt::from_byear`](../struct.Dt.html#method.from_byear).
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch, if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::UTC, 0, 0, 0, 0);
    ///
    /// assert!((x.to_byear() - 2020.000335739628).abs() < 1e-12);
    /// ```
    #[inline]
    pub const fn to_byear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(1900.0) + (jd_tt - f!(2415020.31352)) / f!(365.242198781)
    }

    /// Inverse of
    /// [`Dt::to_byear`](../struct.Dt.html#method.to_byear).
    pub const fn from_byear(byear: Real, scale: Scale) -> Dt {
        if byear.is_nan() {
            return Self::ZERO;
        }
        if byear.is_infinite() {
            return if byear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = f!(2415020.31352) + (byear - f!(1900.0)) * f!(365.242198781);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **decimal year** (Gregorian calendar year + fraction of the year).
    ///
    /// This is the direct equivalent of Astropy’s `Time.decimalyear`:
    /// - Uses the *actual* length of the specific Gregorian year (365 or 366 days,
    ///   plus any leap seconds on UTC/UtcSpice/etc.).
    /// - Scale-aware (TAI, TT, UTC, TDB, etc.), converts to this [`Dt`]'s target time
    ///   scale before producing an output.
    /// - Exact integer arithmetic for the year boundaries, then a high-precision
    ///   `to_sec_f` division (lossy only at the final `Real` step, same as Astropy).
    ///
    /// ## Important:
    ///
    /// - The [`Dt`] first converts itself to the time scale of its `target` field
    ///   before producing a result.
    /// - This function assumes this [`Dt`] is currently from the 2000-01-01 noon
    ///   epoch, if it's not then the output will be incorrect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2020, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert_eq!(x.to_decimalyear(), 2020.0);
    ///
    /// // Also works for negative years
    /// let y = Dt::from_ymd(-2000, 1, 1, Scale::TAI, 0, 0, 0, 0);
    /// assert_eq!(y.to_decimalyear(), -2000.0);
    /// ```
    pub fn to_decimalyear(&self) -> Real {
        let ymd = self.to_ymd();
        let year = ymd.yr;

        let start = Self::from_ymd(year, 1, 1, self.target, 0, 0, 0, 0);
        let next_start = Self::from_ymd(year + 1, 1, 1, self.target, 0, 0, 0, 0);

        let elapsed = self.to_diff_raw(start).to_sec_f();
        let year_length = next_start.to_diff_raw(start).to_sec_f();

        // year_length is never zero for representable years
        f!(year) + elapsed / year_length
    }
}
