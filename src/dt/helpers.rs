use crate::{ATTOS_PER_SEC_I128, ATTOS_PER_SECF, Dt, Real};

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

    /// Combines a whole unit count and fractional attoseconds within that unit into total attoseconds.
    ///
    /// Computes `whole * unit_attos + frac_attos`. The fractional part is always **added**, even
    /// when `whole` is negative — the Euclidean / floor split used by
    /// [`to_ms_floor`](../struct.Dt.html#method.to_ms_floor),
    /// [`to_ns_floor`](../struct.Dt.html#method.to_ns_floor), and related methods, and by
    /// constructors such as
    /// [`from_ms_floor`](../struct.Dt.html#method.from_ms_floor) and
    /// [`from_ns_floor`](../struct.Dt.html#method.from_ns_floor).
    ///
    /// This is **not** the same as pairing with truncating extractors like
    /// [`to_ms`](../struct.Dt.html#method.to_ms) (signed remainder). Unlike
    /// [`from_sec_and_frac`](../struct.Dt.html#method.from_sec_and_frac), the fraction is never
    /// subtracted when `whole` is negative.
    #[inline(always)]
    pub const fn unit_and_attos_to_attos(whole: i128, frac_attos: u128, unit_attos: i128) -> i128 {
        whole
            .saturating_mul(unit_attos)
            .saturating_add(frac_attos as i128)
    }

    /// Converts seconds i128 → total attoseconds i128
    #[inline(always)]
    pub const fn sec_to_attos(sec: i128) -> i128 {
        sec.saturating_mul(ATTOS_PER_SEC_I128)
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
