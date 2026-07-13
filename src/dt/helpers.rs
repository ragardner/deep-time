use crate::{
    ATTOS_PER_DAY, ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128,
    ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_SECF, ATTOS_PER_US_I128,
    ATTOS_PER_WEEK, Dt, Real, SEC_PER_DAY_F,
};

impl Dt {
    /// Clamps `value` to the range `[min, max]`.
    ///
    /// This is a `const fn`, so it can be used in const contexts
    /// (e.g. const generics, statics, const evaluation, etc.).
    ///
    /// If `min > max`, the result is equivalent to clamping to `[max, min]`.
    pub const fn clamp_u8(value: u8, min: u8, max: u8) -> u8 {
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
    pub const fn clamp_u64(value: u64, min: u64, max: u64) -> u64 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Clamps an `i128` to the representable range of `i64`.
    pub const fn to_i64(n: i128) -> i64 {
        let y = n as i64;
        if n == y as i128 {
            y
        } else if n > 0 {
            i64::MAX
        } else {
            i64::MIN
        }
    }

    /// Clamps an `i128` to the representable range of `u64`.
    pub const fn to_u64(n: i128) -> u64 {
        if n > u64::MAX as i128 {
            u64::MAX
        } else if n < u64::MIN as i128 {
            u64::MIN
        } else {
            n as u64
        }
    }

    /// Converts a `u128` to `i128`, saturating at [`i128::MAX`] when the value would wrap.
    ///
    /// A bare `x as i128` is wrapping: values above `i128::MAX` become negative
    /// (e.g. `u128::MAX as i128 == -1`). Prefer this helper whenever a non-negative
    /// attosecond (or other) magnitude stored as `u128` is fed into signed `i128` arithmetic.
    pub const fn to_i128(n: u128) -> i128 {
        if n > i128::MAX as u128 {
            i128::MAX
        } else {
            n as i128
        }
    }

    /// Combines a whole unit count and an **attoseconds** remainder
    /// into total attoseconds.
    ///
    /// Used by constructors such as
    /// [`from_ms`](../struct.Dt.html#method.from_ms).
    #[inline(always)]
    pub const fn unit_to_total_attos(whole: i128, attos: i128, unit_attos: i128) -> i128 {
        whole.saturating_mul(unit_attos).saturating_add(attos)
    }

    /// Converts whole femtoseconds → total attoseconds.
    #[inline(always)]
    pub const fn fs_to_attos(fs: i128) -> i128 {
        fs.saturating_mul(ATTOS_PER_FS_I128)
    }

    /// Converts whole picoseconds → total attoseconds.
    #[inline(always)]
    pub const fn ps_to_attos(ps: i128) -> i128 {
        ps.saturating_mul(ATTOS_PER_PS_I128)
    }

    /// Converts whole nanoseconds → total attoseconds.
    #[inline(always)]
    pub const fn ns_to_attos(ns: i128) -> i128 {
        ns.saturating_mul(ATTOS_PER_NS_I128)
    }

    /// Converts whole microseconds → total attoseconds.
    #[inline(always)]
    pub const fn us_to_attos(us: i128) -> i128 {
        us.saturating_mul(ATTOS_PER_US_I128)
    }

    /// Converts whole milliseconds → total attoseconds.
    #[inline(always)]
    pub const fn ms_to_attos(ms: i128) -> i128 {
        ms.saturating_mul(ATTOS_PER_MS_I128)
    }

    /// Converts whole seconds → total attoseconds.
    #[inline(always)]
    pub const fn sec_to_attos(sec: i128) -> i128 {
        sec.saturating_mul(ATTOS_PER_SEC_I128)
    }

    /// Converts whole minutes → total attoseconds.
    #[inline(always)]
    pub const fn mins_to_attos(mins: i128) -> i128 {
        mins.saturating_mul(ATTOS_PER_MIN)
    }

    /// Converts whole hours → total attoseconds.
    #[inline(always)]
    pub const fn hours_to_attos(hours: i128) -> i128 {
        hours.saturating_mul(ATTOS_PER_HOUR)
    }

    /// Converts whole days → total attoseconds.
    #[inline(always)]
    pub const fn days_to_attos(days: i128) -> i128 {
        days.saturating_mul(ATTOS_PER_DAY)
    }

    /// Converts a floating-point day count → total attoseconds.
    ///
    /// Uses the same high-precision path as
    /// [`sec_f_to_attos`](../struct.Dt.html#method.sec_f_to_attos)
    /// (`days × 86_400` seconds).
    #[inline(always)]
    pub const fn days_f_to_attos(days: Real) -> i128 {
        Self::sec_f_to_attos(days * SEC_PER_DAY_F)
    }

    /// Converts whole weeks → total attoseconds.
    #[inline(always)]
    pub const fn weeks_to_attos(weeks: i128) -> i128 {
        weeks.saturating_mul(ATTOS_PER_WEEK)
    }

    /// Converts total attoseconds → whole femtoseconds.
    #[inline(always)]
    pub const fn attos_to_fs(attos: i128) -> i128 {
        attos / ATTOS_PER_FS_I128
    }

    /// Converts total attoseconds → whole picoseconds.
    #[inline(always)]
    pub const fn attos_to_ps(attos: i128) -> i128 {
        attos / ATTOS_PER_PS_I128
    }

    /// Converts total attoseconds → whole nanoseconds.
    #[inline(always)]
    pub const fn attos_to_ns(attos: i128) -> i128 {
        attos / ATTOS_PER_NS_I128
    }

    /// Converts total attoseconds → whole microseconds.
    #[inline(always)]
    pub const fn attos_to_us(attos: i128) -> i128 {
        attos / ATTOS_PER_US_I128
    }

    /// Converts total attoseconds → whole milliseconds.
    #[inline(always)]
    pub const fn attos_to_ms(attos: i128) -> i128 {
        attos / ATTOS_PER_MS_I128
    }

    /// Converts total attoseconds → whole seconds.
    #[inline(always)]
    pub const fn attos_to_sec(attos: i128) -> i128 {
        attos / ATTOS_PER_SEC_I128
    }

    /// Converts total attoseconds → whole seconds as i64, clamped to [`i64::MIN`]/[`i64::MAX`].
    #[inline(always)]
    pub const fn attos_to_sec_i64(attos: i128) -> i64 {
        Self::to_i64(Self::attos_to_sec(attos))
    }

    /// **Lossy** conversion of i128 attoseconds to → float seconds (s).
    #[inline(always)]
    pub const fn attos_to_sec_f(attos: i128) -> Real {
        f!(attos) / ATTOS_PER_SECF
    }

    /// Converts total attoseconds → whole minutes.
    #[inline(always)]
    pub const fn attos_to_mins(attos: i128) -> i128 {
        attos / ATTOS_PER_MIN
    }

    /// Converts total attoseconds → whole hours.
    #[inline(always)]
    pub const fn attos_to_hours(attos: i128) -> i128 {
        attos / ATTOS_PER_HOUR
    }

    /// Converts total attoseconds → whole days.
    #[inline(always)]
    pub const fn attos_to_days(attos: i128) -> i128 {
        attos / ATTOS_PER_DAY
    }

    /// **Lossy** conversion of total attoseconds → floating-point days.
    #[inline(always)]
    pub const fn attos_to_days_f(attos: i128) -> Real {
        Self::attos_to_sec_f(attos) / SEC_PER_DAY_F
    }

    /// Converts total attoseconds → whole weeks.
    #[inline(always)]
    pub const fn attos_to_weeks(attos: i128) -> i128 {
        attos / ATTOS_PER_WEEK
    }
}
