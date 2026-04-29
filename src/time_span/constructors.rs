use crate::{
    ATTOSEC_PER_FEMTOSEC, ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC, ATTOSEC_PER_NANOSEC,
    ATTOSEC_PER_PICOSEC, ATTOSEC_PER_SEC, TimeSpan,
};

impl TimeSpan {
    /// Zero duration (`0 s`).
    pub const ZERO: Self = Self { sec: 0, subsec: 0 };

    /// Maximum representable duration (`i64::MAX` seconds + 999... attoseconds).
    pub const MAX: Self = Self {
        sec: i64::MAX,
        subsec: ATTOSEC_PER_SEC - 1,
    };

    /// Minimum (most negative) representable duration (`i64::MIN` seconds).
    pub const MIN: Self = Self {
        sec: i64::MIN,
        subsec: 0,
    };

    pub const SEC_19: Self = Self::from_sec(19);
    pub const SEC_33: Self = Self::from_sec(33);
    pub const SEC_37: Self = Self::from_sec(37);

    /// Creates a new `TimeSpan` from whole seconds and a subsecond part.
    ///
    /// The result is automatically normalized so `subsec` lies in `[0, 10¹⁸)`.
    #[inline]
    pub const fn new(sec: i64, subsec: u64) -> Self {
        let mut dt = Self { sec, subsec };
        dt.carry_over();
        dt
    }

    /// Creates a `TimeSpan` representing `s` seconds.
    #[inline]
    pub const fn from_sec(s: i64) -> Self {
        Self::new(s, 0)
    }

    /// Creates a `TimeSpan` representing `ms` milliseconds.
    #[inline]
    pub const fn from_ms(ms: i128) -> Self {
        Self::from_total_attos(ms.saturating_mul(ATTOSEC_PER_MILLISEC as i128))
    }

    /// Creates a `TimeSpan` representing `us` microseconds.
    #[inline]
    pub const fn from_us(us: i128) -> Self {
        Self::from_total_attos(us.saturating_mul(ATTOSEC_PER_MICROSEC as i128))
    }

    /// Creates a `TimeSpan` representing `ns` nanoseconds.
    #[inline]
    pub const fn from_ns(ns: i128) -> Self {
        Self::from_total_attos(ns.saturating_mul(ATTOSEC_PER_NANOSEC as i128))
    }

    /// Creates a `TimeSpan` representing `ps` picoseconds.
    #[inline]
    pub const fn from_ps(ps: i128) -> Self {
        Self::from_total_attos(ps.saturating_mul(ATTOSEC_PER_PICOSEC as i128))
    }

    /// Creates a `TimeSpan` representing `fs` femtoseconds.
    #[inline]
    pub const fn from_fs(fs: i128) -> Self {
        Self::from_total_attos(fs.saturating_mul(ATTOSEC_PER_FEMTOSEC as i128))
    }

    /// Creates a `TimeSpan` representing `as` attoseconds.
    #[inline]
    pub const fn from_as(as_: i128) -> Self {
        Self::from_total_attos(as_)
    }

    /// Creates a `TimeSpan` representing `m` minutes.
    #[inline]
    pub const fn from_min(m: i64) -> Self {
        Self::from_sec(m * 60)
    }

    /// Creates a `TimeSpan` representing `h` hours.
    #[inline]
    pub const fn from_hr(h: i64) -> Self {
        Self::from_sec(h * 3600)
    }

    /// Creates a `TimeSpan` from hours, minutes, seconds, milliseconds,
    /// microseconds, and nanoseconds.
    #[inline]
    pub const fn from_hms(hr: i64, min: i64, sec: i64, ms: i64, us: i64, ns: i64) -> Self {
        let total_secs = hr * 3600i64 + min * 60i64 + sec;

        let sub_ns = ms * 1_000_000i64 + us * 1_000i64 + ns;

        if sub_ns == 0 {
            return Self::new(total_secs, 0);
        }

        let abs_ns = sub_ns.unsigned_abs();
        let extra_secs = (abs_ns / 1_000_000_000u64) as i64;
        let rem_ns = abs_ns % 1_000_000_000u64;
        let frac = rem_ns * ATTOSEC_PER_NANOSEC;

        let (final_secs, final_frac) = if sub_ns >= 0 {
            (total_secs + extra_secs, frac)
        } else if frac == 0 {
            (total_secs - extra_secs, 0)
        } else {
            (total_secs - extra_secs - 1, ATTOSEC_PER_SEC - frac)
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
                subsec: ATTOSEC_PER_SEC - self.subsec,
            }
        }
    }
}
