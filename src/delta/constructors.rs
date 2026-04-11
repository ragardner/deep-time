use crate::{
    Delta, MICROQUECTOS_PER_ATTOSEC, MICROQUECTOS_PER_FEMTOSEC, MICROQUECTOS_PER_MICROSEC,
    MICROQUECTOS_PER_MILLISEC, MICROQUECTOS_PER_NANOSEC, MICROQUECTOS_PER_PICOSEC,
    MICROQUECTOS_PER_QUECTOSEC, MICROQUECTOS_PER_RONTOSEC, MICROQUECTOS_PER_SEC,
    MICROQUECTOS_PER_YOCTOSEC, MICROQUECTOS_PER_ZEPTOSEC,
};

impl Delta {
    /// Zero duration (`0 s`).
    pub const ZERO: Self = Self { sec: 0, subsec: 0 };

    /// Maximum representable duration (`i128::MAX` seconds + 999... subseconds).
    pub const MAX: Self = Self {
        sec: i128::MAX,
        subsec: MICROQUECTOS_PER_SEC - 1,
    };

    /// Minimum (most negative) representable duration (`i128::MIN` seconds).
    pub const MIN: Self = Self {
        sec: i128::MIN,
        subsec: 0,
    };

    /// Creates a new `Delta` from whole seconds and a subsecond part.
    ///
    /// The result is automatically normalized so `subsec` lies in `[0, 10³⁶)`.
    #[inline]
    pub const fn new(sec: i128, subsec: u128) -> Self {
        Self { sec, subsec }.normalize()
    }

    /// Normalizes the representation so `subsec` stays in `[0, 10³⁶)`.
    const fn normalize(mut self) -> Self {
        if self.subsec >= MICROQUECTOS_PER_SEC {
            let carry = (self.subsec / MICROQUECTOS_PER_SEC) as i128;
            self.sec += carry;
            self.subsec %= MICROQUECTOS_PER_SEC;
        }
        self
    }

    #[inline]
    const fn from_subunits(count: i128, microquectos_per_unit: u128) -> Self {
        let abs_count = count.unsigned_abs();
        let units_per_second = MICROQUECTOS_PER_SEC / microquectos_per_unit;

        let extra_secs = (abs_count / units_per_second) as i128;
        let remaining = abs_count % units_per_second;
        let frac = remaining * microquectos_per_unit;

        if count >= 0 {
            Self::new(extra_secs, frac)
        } else if frac == 0 {
            Self::new(-extra_secs, 0)
        } else {
            Self::new(-extra_secs - 1, MICROQUECTOS_PER_SEC - frac)
        }
    }

    /// Creates a `Delta` representing `s` seconds.
    #[inline]
    pub const fn from_sec(s: i128) -> Self {
        Self::new(s, 0)
    }

    /// Creates a `Delta` representing `ms` milliseconds.
    #[inline]
    pub const fn from_ms(ms: i128) -> Self {
        Self::from_subunits(ms, MICROQUECTOS_PER_MILLISEC)
    }

    /// Creates a `Delta` representing `us` microseconds.
    #[inline]
    pub const fn from_us(us: i128) -> Self {
        Self::from_subunits(us, MICROQUECTOS_PER_MICROSEC)
    }

    /// Creates a `Delta` representing `ns` nanoseconds.
    #[inline]
    pub const fn from_ns(ns: i128) -> Self {
        Self::from_subunits(ns, MICROQUECTOS_PER_NANOSEC)
    }

    /// Creates a `Delta` representing `ps` picoseconds.
    #[inline]
    pub const fn from_ps(ps: i128) -> Self {
        Self::from_subunits(ps, MICROQUECTOS_PER_PICOSEC)
    }

    /// Creates a `Delta` representing `fs` femtoseconds.
    #[inline]
    pub const fn from_fs(fs: i128) -> Self {
        Self::from_subunits(fs, MICROQUECTOS_PER_FEMTOSEC)
    }

    /// Creates a `Delta` representing `as` attoseconds.
    #[inline]
    pub const fn from_as(as_: i128) -> Self {
        Self::from_subunits(as_, MICROQUECTOS_PER_ATTOSEC)
    }

    /// Creates a `Delta` representing `zs` zeptoseconds.
    #[inline]
    pub const fn from_zs(zs: i128) -> Self {
        Self::from_subunits(zs, MICROQUECTOS_PER_ZEPTOSEC)
    }

    /// Creates a `Delta` representing `ys` yoctoseconds.
    #[inline]
    pub const fn from_ys(ys: i128) -> Self {
        Self::from_subunits(ys, MICROQUECTOS_PER_YOCTOSEC)
    }

    /// Creates a `Delta` representing `rs` rontoseconds.
    #[inline]
    pub const fn from_rs(rs: i128) -> Self {
        Self::from_subunits(rs, MICROQUECTOS_PER_RONTOSEC)
    }

    /// Creates a `Delta` representing `qs` quectoseconds.
    #[inline]
    pub const fn from_qs(qs: i128) -> Self {
        Self::from_subunits(qs, MICROQUECTOS_PER_QUECTOSEC)
    }

    /// Creates a `Delta` representing `m` minutes.
    #[inline]
    pub const fn from_min(m: i128) -> Self {
        Self::from_sec(m * 60)
    }

    /// Creates a `Delta` representing `h` hours.
    #[inline]
    pub const fn from_hr(h: i128) -> Self {
        Self::from_sec(h * 3600)
    }

    /// Creates a `Delta` from hours, minutes, seconds, milliseconds,
    /// microseconds, and nanoseconds.
    #[inline]
    pub const fn from_hms(hr: i128, min: i128, sec: i128, ms: i128, us: i128, ns: i128) -> Self {
        let total_secs = hr * 3600i128 + min * 60i128 + sec;

        let sub_ns = ms * 1_000_000i128 + us * 1_000i128 + ns;

        if sub_ns == 0 {
            return Self::new(total_secs, 0);
        }

        let abs_ns = sub_ns.unsigned_abs();
        let extra_secs = (abs_ns / 1_000_000_000u128) as i128;
        let rem_ns = abs_ns % 1_000_000_000u128;
        let frac = rem_ns * MICROQUECTOS_PER_NANOSEC;

        let (final_secs, final_frac) = if sub_ns >= 0 {
            (total_secs + extra_secs, frac)
        } else if frac == 0 {
            (total_secs - extra_secs, 0)
        } else {
            (total_secs - extra_secs - 1, MICROQUECTOS_PER_SEC - frac)
        };

        Self::new(final_secs, final_frac)
    }

    /// Returns the negation of this duration.
    #[inline]
    pub const fn neg(self) -> Self {
        if self.subsec == 0 {
            Self {
                sec: -self.sec,
                subsec: 0,
            }
        } else {
            Self {
                sec: -self.sec - 1,
                subsec: MICROQUECTOS_PER_SEC - self.subsec,
            }
        }
    }
}
