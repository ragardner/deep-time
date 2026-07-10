use crate::{
    ATTOS_PER_FS_I128, ATTOS_PER_HOUR, ATTOS_PER_MIN, ATTOS_PER_MS_I128, ATTOS_PER_NS_I128,
    ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_SECF, ATTOS_PER_US_I128, Dt, Real,
};

impl Dt {
    /// Clamps an `i128` to the representable range of `i64`.
    pub const fn to_i64(x: i128) -> i64 {
        let y = x as i64;
        if x == y as i128 {
            y
        } else if x > 0 {
            i64::MAX
        } else {
            i64::MIN
        }
    }

    /// Clamps an `i128` to the representable range of `u64`.
    pub const fn to_u64(x: i128) -> u64 {
        if x > u64::MAX as i128 {
            u64::MAX
        } else if x < u64::MIN as i128 {
            u64::MIN
        } else {
            x as u64
        }
    }

    /// Converts a `u128` to `i128`, saturating at [`i128::MAX`] when the value would wrap.
    ///
    /// A bare `x as i128` is wrapping: values above `i128::MAX` become negative
    /// (e.g. `u128::MAX as i128 == -1`). Prefer this helper whenever a non-negative
    /// attosecond (or other) magnitude stored as `u128` is fed into signed `i128` arithmetic.
    pub const fn to_i128(x: u128) -> i128 {
        if x > i128::MAX as u128 {
            i128::MAX
        } else {
            x as i128
        }
    }

    /// Combines a whole unit count and fractional attoseconds within that unit into total
    /// attoseconds.
    ///
    /// Computes `whole * unit_attos + frac_attos`. The fractional part is always **added**, even
    /// when `whole` is negative — the Euclidean / floor split used by
    /// [`to_ms_floor`](../struct.Dt.html#method.to_ms_floor),
    /// [`to_ns_floor`](../struct.Dt.html#method.to_ns_floor).
    ///
    /// This is **not** the same as pairing with truncating extractors like
    /// [`to_ms`](../struct.Dt.html#method.to_ms) (signed remainder).
    ///
    /// The fraction is never subtracted even when `whole` is negative.
    ///
    /// For the truncating / signed-remainder split, use
    /// [`unit_and_signed_attos_to_attos`](../struct.Dt.html#method.unit_and_signed_attos_to_attos).
    #[inline(always)]
    pub const fn unit_and_attos_to_attos(whole: i128, frac_attos: u128, unit_attos: i128) -> i128 {
        whole
            .saturating_mul(unit_attos)
            .saturating_add(Self::to_i128(frac_attos))
    }

    /// Combines a whole unit count and a signed fractional remainder into total attoseconds.
    ///
    /// The two parts are the left and right sides of a decimal: e.g. `-1.3` units is
    /// `whole = -1` and a negative `frac_attos` for the `0.3` **(expressed in attoseconds, not
    /// in the unit itself)**.
    ///
    /// Used by constructors such as
    /// [`from_ms`](../struct.Dt.html#method.from_ms).
    ///
    /// For the floor form (fraction always non-negative and always added), use
    /// [`unit_and_attos_to_attos`](../struct.Dt.html#method.unit_and_attos_to_attos).
    #[inline(always)]
    pub const fn unit_and_signed_attos_to_attos(
        whole: i128,
        frac_attos: i128,
        unit_attos: i128,
    ) -> i128 {
        whole.saturating_mul(unit_attos).saturating_add(frac_attos)
    }

    /// Converts whole femtoseconds → total attoseconds (`× 10³`).
    #[inline(always)]
    pub const fn fs_to_attos(fs: i128) -> i128 {
        fs.saturating_mul(ATTOS_PER_FS_I128)
    }

    /// Converts whole picoseconds → total attoseconds (`× 10⁶`).
    #[inline(always)]
    pub const fn ps_to_attos(ps: i128) -> i128 {
        ps.saturating_mul(ATTOS_PER_PS_I128)
    }

    /// Converts whole nanoseconds → total attoseconds (`× 10⁹`).
    #[inline(always)]
    pub const fn ns_to_attos(ns: i128) -> i128 {
        ns.saturating_mul(ATTOS_PER_NS_I128)
    }

    /// Converts whole microseconds → total attoseconds (`× 10¹²`).
    #[inline(always)]
    pub const fn us_to_attos(us: i128) -> i128 {
        us.saturating_mul(ATTOS_PER_US_I128)
    }

    /// Converts whole milliseconds → total attoseconds (`× 10¹⁵`).
    #[inline(always)]
    pub const fn ms_to_attos(ms: i128) -> i128 {
        ms.saturating_mul(ATTOS_PER_MS_I128)
    }

    /// Converts whole seconds → total attoseconds (`× 10¹⁸`).
    #[inline(always)]
    pub const fn sec_to_attos(sec: i128) -> i128 {
        sec.saturating_mul(ATTOS_PER_SEC_I128)
    }

    /// Converts whole minutes → total attoseconds (`× 60 × 10¹⁸`).
    #[inline(always)]
    pub const fn mins_to_attos(mins: i128) -> i128 {
        mins.saturating_mul(ATTOS_PER_MIN)
    }

    /// Converts whole hours → total attoseconds (`× 3600 × 10¹⁸`).
    #[inline(always)]
    pub const fn hours_to_attos(hours: i128) -> i128 {
        hours.saturating_mul(ATTOS_PER_HOUR)
    }

    /// Converts total attoseconds → whole seconds as i64
    #[inline(always)]
    pub const fn attos_to_sec_i64(attos: i128) -> i64 {
        Self::to_i64(attos / ATTOS_PER_SEC_I128)
    }

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

    /// **Lossy** conversion of u128 attoseconds to → float seconds (s).
    #[inline(always)]
    pub const fn attos_to_sec_f(attos: u128) -> Real {
        f!(attos) / ATTOS_PER_SECF
    }
}
