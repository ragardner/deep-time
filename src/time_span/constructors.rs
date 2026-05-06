use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US, ClockType, Real, SEC_PER_DAYI64, SEC_PER_WEEK, TimePoint,
    TimeSpan, floor_f,
};

impl TimeSpan {
    /// Zero duration (`0 s`).
    pub const ZERO: Self = Self { sec: 0, subsec: 0 };

    /// Maximum representable duration (`i64::MAX` seconds + 999... attoseconds).
    pub const MAX: Self = Self {
        sec: i64::MAX,
        subsec: ATTOS_PER_SEC - 1,
    };

    /// Minimum (most negative) representable duration (`i64::MIN` seconds).
    pub const MIN: Self = Self {
        sec: i64::MIN,
        subsec: 0,
    };

    pub const SEC_19: Self = Self::from_sec(19);
    pub const SEC_33: Self = Self::from_sec(33);
    pub const SEC_37: Self = Self::from_sec(37);
    pub const ONE_DAY: Self = Self::from_days(1);

    /// Reconstruct `TimeSpan` from total attoseconds (exact, handles negative values correctly).
    pub const fn from_attos(mut attos: i128) -> Self {
        if attos > (i64::MAX as i128) * ATTOS_PER_SEC_I128 {
            return Self::MAX;
        }
        if attos < (i64::MIN as i128) * ATTOS_PER_SEC_I128 {
            return Self::MIN;
        }

        if attos >= 0 {
            let sec = (attos / ATTOS_PER_SEC_I128) as i64;
            let subsec = (attos % ATTOS_PER_SEC_I128) as u64;
            Self { sec, subsec }
        } else {
            attos = -attos;
            let sec_pos = (attos / ATTOS_PER_SEC_I128) as i64;
            let rem = (attos % ATTOS_PER_SEC_I128) as u64;
            if rem == 0 {
                Self {
                    sec: -sec_pos,
                    subsec: 0,
                }
            } else {
                Self {
                    sec: -sec_pos - 1,
                    subsec: ATTOS_PER_SEC - rem,
                }
            }
        }
    }

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
        Self::from_attos(ms.saturating_mul(ATTOS_PER_MS as i128))
    }

    /// Creates a `TimeSpan` representing `us` microseconds.
    #[inline]
    pub const fn from_us(us: i128) -> Self {
        Self::from_attos(us.saturating_mul(ATTOS_PER_US as i128))
    }

    /// Creates a `TimeSpan` representing `ns` nanoseconds.
    #[inline]
    pub const fn from_ns(ns: i128) -> Self {
        Self::from_attos(ns.saturating_mul(ATTOS_PER_NS as i128))
    }

    /// Creates a `TimeSpan` representing `ps` picoseconds.
    #[inline]
    pub const fn from_ps(ps: i128) -> Self {
        Self::from_attos(ps.saturating_mul(ATTOS_PER_PS as i128))
    }

    /// Creates a `TimeSpan` representing `fs` femtoseconds.
    #[inline]
    pub const fn from_fs(fs: i128) -> Self {
        Self::from_attos(fs.saturating_mul(ATTOS_PER_FS as i128))
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

    #[inline]
    pub const fn from_days(d: i64) -> TimeSpan {
        Self::from_sec(d.saturating_mul(SEC_PER_DAYI64))
    }

    #[inline]
    pub const fn wk(wk: i64) -> TimeSpan {
        TimeSpan::from_sec(wk.saturating_mul(SEC_PER_WEEK))
    }

    #[inline]
    pub const fn yr(yr: i64) -> TimeSpan {
        TimeSpan::from_sec(yr.saturating_mul(31_557_600))
    }

    /// Returns a `TimePoint` that is this duration ago from the given clock type.
    #[inline]
    pub const fn ago(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from(0, 0, clock_type).sub(self)
    }

    /// Returns a `TimePoint` that is this duration from now in the given clock type.
    #[inline]
    pub const fn from_now(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from(0, 0, clock_type).add(self)
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
        let frac = rem_ns * ATTOS_PER_NS;

        let (final_secs, final_frac) = if sub_ns >= 0 {
            (total_secs + extra_secs, frac)
        } else if frac == 0 {
            (total_secs - extra_secs, 0)
        } else {
            (total_secs - extra_secs - 1, ATTOS_PER_SEC - frac)
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
                subsec: ATTOS_PER_SEC - self.subsec,
            }
        }
    }

    /// Creates a `TimeSpan` from a floating-point number of seconds.
    pub const fn from_sec_f(sec_f: Real) -> Self {
        if sec_f.is_nan() {
            return Self::ZERO;
        }
        if sec_f.is_infinite() {
            return if sec_f.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let floor_val = floor_f(sec_f);
        let frac = sec_f - floor_val;
        let attos_frac = (frac * ATTOS_PER_SECF) as i128;

        let total = (floor_val as i128) * ATTOS_PER_SEC_I128 + attos_frac;
        Self::from_attos(total)
    }
}
