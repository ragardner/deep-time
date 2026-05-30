//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time::{Scale, TimeTraits};
//!
//! let span = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().ago(Scale::UTC);
//! ```

use crate::{
    ATTOS_PER_FS_I128, ATTOS_PER_MS_I128, ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US_I128, Dt, SEC_PER_DAY, SEC_PER_DAY_F, SEC_PER_DAYI128, Scale,
};

/// Trait that adds ergonomic conversions from attoseconds values
/// for i64, i128, and f64.
///
/// ## Examples
///
/// ```
/// use deep_time::AttosTraits;
///
/// let attos: i128 = 5;
/// let seconds = attos.attos_to_sec();
/// ```
pub trait AttosTraits: Copy + Sized {
    /// attoseconds → seconds (s)
    fn attos_to_sec(self) -> i64;

    /// attoseconds → milliseconds (ms)
    fn attos_to_ms(self) -> i128;

    /// attoseconds → microseconds (us)
    fn attos_to_us(self) -> i128;

    /// attoseconds → nanoseconds (ns)
    fn attos_to_ns(self) -> i128;

    /// attoseconds → picoseconds (ps)
    fn attos_to_ps(self) -> i128;

    /// attoseconds → femtoseconds (fs)
    fn attos_to_fs(self) -> i128;

    /// attoseconds → float seconds (s)
    fn attos_to_sec_f(self) -> f64;
}

impl AttosTraits for i128 {
    #[inline]
    fn attos_to_sec_f(self) -> f64 {
        self as f64 / ATTOS_PER_SECF
    }

    #[inline]
    fn attos_to_sec(self) -> i64 {
        (self / ATTOS_PER_SEC_I128) as i64
    }

    #[inline]
    fn attos_to_ms(self) -> i128 {
        self / ATTOS_PER_MS_I128
    }

    #[inline]
    fn attos_to_us(self) -> i128 {
        self / ATTOS_PER_US_I128
    }

    #[inline]
    fn attos_to_ns(self) -> i128 {
        self / ATTOS_PER_NS_I128
    }

    #[inline]
    fn attos_to_ps(self) -> i128 {
        self / ATTOS_PER_PS_I128
    }

    #[inline]
    fn attos_to_fs(self) -> i128 {
        self / ATTOS_PER_FS_I128
    }
}

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// ## Examples
///
/// ```
/// use deep_time::TimeTraits;
///
/// let dt = 5.days();
/// ```
pub trait TimeTraits: Copy + Sized {
    // ── Dt constructors ─────────────────────────────────────
    fn ns(self) -> Dt;
    fn us(self) -> Dt;
    fn ms(self) -> Dt;
    fn sec(self) -> Dt;
    fn min(self) -> Dt;
    fn hr(self) -> Dt;
    fn days(self) -> Dt; // 86400 s (civil day, not leap-second aware)
    fn wk(self) -> Dt;
    fn yr(self) -> Dt; // 365.25 days (standard approximation)

    // ── Dt constructors (anchored at "now" in the chosen scale) ──
    fn ago(self, scale: Scale) -> Dt;
    fn from_now(self, scale: Scale) -> Dt;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeTraits for $ty {
                #[inline]
                fn ns(self) -> Dt { Dt::from_ns(self as i128, Scale::TAI) }

                #[inline]
                fn us(self) -> Dt { Dt::from_us(self as i128, Scale::TAI) }

                #[inline]
                fn ms(self) -> Dt { Dt::from_ms(self as i128, Scale::TAI) }

                #[inline]
                fn sec(self) -> Dt { Dt::from_sec(self as i128, Scale::TAI) }

                #[inline]
                fn min(self) -> Dt { Dt::from_min(self as i64, Scale::TAI) }

                #[inline]
                fn hr(self) -> Dt { Dt::from_hr(self as i64, Scale::TAI) }

                #[inline]
                fn days(self) -> Dt { Dt::from_sec((self as i128).saturating_mul(SEC_PER_DAYI128), Scale::TAI) }

                #[inline]
                fn wk(self) -> Dt { Dt::from_sec((self  as i128).saturating_mul(604_800), Scale::TAI) }

                #[inline]
                fn yr(self) -> Dt { Dt::from_sec((self  as i128).saturating_mul(31_557_600), Scale::TAI) }

                #[inline]
                fn ago(self, scale: Scale) -> Dt {
                    Dt::from_attos(0, scale).sub(self.sec())
                }

                #[inline]
                fn from_now(self, scale: Scale) -> Dt {
                    Dt::from_attos(0, scale).add(self.sec())
                }
            }
        )*
    };
}

impl_time_units_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// f64 support (most useful for fractional units)
impl TimeTraits for f64 {
    #[inline]
    fn ns(self) -> Dt {
        Dt::from_ns(self as i128, Scale::TAI)
    }

    #[inline]
    fn us(self) -> Dt {
        Dt::from_us(self as i128, Scale::TAI)
    }

    #[inline]
    fn ms(self) -> Dt {
        Dt::from_ms(self as i128, Scale::TAI)
    }

    #[inline]
    fn sec(self) -> Dt {
        Dt::from_sec(self as i128, Scale::TAI)
    }

    #[inline]
    fn min(self) -> Dt {
        (self * 60.0).sec()
    }

    #[inline]
    fn hr(self) -> Dt {
        (self * 3600.0).sec()
    }

    #[inline]
    fn days(self) -> Dt {
        (self * SEC_PER_DAY_F).sec()
    }

    #[inline]
    fn wk(self) -> Dt {
        (self * 604_800.0).sec()
    }

    #[inline]
    fn yr(self) -> Dt {
        (self * 31_557_600.0).sec()
    }

    #[inline]
    fn ago(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).sub(self.sec())
    }

    #[inline]
    fn from_now(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).add(self.sec())
    }
}

impl TimeTraits for f32 {
    #[inline]
    fn ns(self) -> Dt {
        Dt::from_ns(self as i128, Scale::TAI)
    }

    #[inline]
    fn us(self) -> Dt {
        Dt::from_us(self as i128, Scale::TAI)
    }

    #[inline]
    fn ms(self) -> Dt {
        Dt::from_ms(self as i128, Scale::TAI)
    }

    #[inline]
    fn sec(self) -> Dt {
        Dt::from_sec(self as i128, Scale::TAI)
    }

    #[inline]
    fn min(self) -> Dt {
        (self * 60.0f32).sec()
    }

    #[inline]
    fn hr(self) -> Dt {
        (self * 3600.0f32).sec()
    }

    #[inline]
    fn days(self) -> Dt {
        (self * SEC_PER_DAY as f32).sec()
    }

    #[inline]
    fn wk(self) -> Dt {
        (self * 604_800.0f32).sec()
    }

    #[inline]
    fn yr(self) -> Dt {
        (self * 31_557_600.0f32).sec()
    }

    #[inline]
    fn ago(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).sub(self.sec())
    }

    #[inline]
    fn from_now(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).add(self.sec())
    }
}
