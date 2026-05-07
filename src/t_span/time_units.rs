//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time::{Scale, TimeUnits};
//!
//! let span = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().ago(Scale::UTC);
//! ```

use crate::{
    ATTOS_PER_FS_I128, ATTOS_PER_MS_I128, ATTOS_PER_NS_I128, ATTOS_PER_PS_I128, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US_I128, Scale, SEC_PER_DAY, SEC_PER_DAY_F, SEC_PER_DAYI64,
    Dt, TSpan,
};

pub trait AttosUnits: Copy + Sized {
    /// attoseconds → seconds (s)
    fn to_sec(self) -> i64;

    /// attoseconds → milliseconds (ms)
    fn to_ms(self) -> i128;

    /// attoseconds → microseconds (us)
    fn to_us(self) -> i128;

    /// attoseconds → nanoseconds (ns)
    fn to_ns(self) -> i128;

    /// attoseconds → picoseconds (ps)
    fn to_ps(self) -> i128;

    /// attoseconds → femtoseconds (fs)
    fn to_fs(self) -> i128;

    /// attoseconds → float seconds (s)
    fn to_sec_f(self) -> f64;
}

impl AttosUnits for i128 {
    #[inline(always)]
    fn to_sec_f(self) -> f64 {
        self as f64 / ATTOS_PER_SECF
    }

    #[inline(always)]
    fn to_sec(self) -> i64 {
        (self / ATTOS_PER_SEC_I128) as i64
    }

    #[inline(always)]
    fn to_ms(self) -> i128 {
        self / ATTOS_PER_MS_I128
    }

    #[inline(always)]
    fn to_us(self) -> i128 {
        self / ATTOS_PER_US_I128
    }

    #[inline(always)]
    fn to_ns(self) -> i128 {
        self / ATTOS_PER_NS_I128
    }

    #[inline(always)]
    fn to_ps(self) -> i128 {
        self / ATTOS_PER_PS_I128
    }

    #[inline(always)]
    fn to_fs(self) -> i128 {
        self / ATTOS_PER_FS_I128
    }
}

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// Import it explicitly to create `TSpan`s directly from rust ints and floats:
/// `use deep_time::TimeUnits;`
pub trait TimeUnits: Copy + Sized {
    // ── TSpan constructors ─────────────────────────────────────
    fn ns(self) -> TSpan;
    fn us(self) -> TSpan;
    fn ms(self) -> TSpan;
    fn sec(self) -> TSpan;
    fn min(self) -> TSpan;
    fn hr(self) -> TSpan;
    fn days(self) -> TSpan; // 86400 s (civil day, not leap-second aware)
    fn wk(self) -> TSpan;
    fn yr(self) -> TSpan; // 365.25 days (standard approximation)

    // ── Dt constructors (anchored at "now" in the chosen scale) ──
    fn ago(self, scale: Scale) -> Dt;
    fn from_now(self, scale: Scale) -> Dt;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeUnits for $ty {
                #[inline]
                fn ns(self) -> TSpan { TSpan::from_ns(self as i128) }

                #[inline]
                fn us(self) -> TSpan { TSpan::from_us(self as i128) }

                #[inline]
                fn ms(self) -> TSpan { TSpan::from_ms(self as i128) }

                #[inline]
                fn sec(self) -> TSpan { TSpan::from_sec(self as i64) }

                #[inline]
                fn min(self) -> TSpan { TSpan::from_min(self as i64) }

                #[inline]
                fn hr(self) -> TSpan { TSpan::from_hr(self as i64) }

                #[inline]
                fn days(self) -> TSpan { TSpan::from_sec((self as i64).saturating_mul(SEC_PER_DAYI64)) }

                #[inline]
                fn wk(self) -> TSpan { TSpan::from_sec((self as i64).saturating_mul(604_800)) }

                #[inline]
                fn yr(self) -> TSpan { TSpan::from_sec((self as i64).saturating_mul(31_557_600)) }

                #[inline]
                fn ago(self, scale: Scale) -> Dt {
                    Dt::from(0, 0,scale).sub(self.sec())
                }

                #[inline]
                fn from_now(self, scale: Scale) -> Dt {
                    Dt::from(0, 0, scale).add(self.sec())
                }
            }
        )*
    };
}

impl_time_units_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// f64 support (most useful for fractional units)
impl TimeUnits for f64 {
    #[inline]
    fn ns(self) -> TSpan {
        TSpan::from_ns(self as i128)
    }

    #[inline]
    fn us(self) -> TSpan {
        TSpan::from_us(self as i128)
    }

    #[inline]
    fn ms(self) -> TSpan {
        TSpan::from_ms(self as i128)
    }

    #[inline]
    fn sec(self) -> TSpan {
        TSpan::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> TSpan {
        (self * 60.0).sec()
    }

    #[inline]
    fn hr(self) -> TSpan {
        (self * 3600.0).sec()
    }

    #[inline]
    fn days(self) -> TSpan {
        (self * SEC_PER_DAY_F).sec()
    }

    #[inline]
    fn wk(self) -> TSpan {
        (self * 604_800.0).sec()
    }

    #[inline]
    fn yr(self) -> TSpan {
        (self * 31_557_600.0).sec()
    }

    #[inline]
    fn ago(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).sub(self.sec())
    }

    #[inline]
    fn from_now(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).add(self.sec())
    }
}

impl TimeUnits for f32 {
    #[inline]
    fn ns(self) -> TSpan {
        TSpan::from_ns(self as i128)
    }

    #[inline]
    fn us(self) -> TSpan {
        TSpan::from_us(self as i128)
    }

    #[inline]
    fn ms(self) -> TSpan {
        TSpan::from_ms(self as i128)
    }

    #[inline]
    fn sec(self) -> TSpan {
        TSpan::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> TSpan {
        (self * 60.0f32).sec()
    }

    #[inline]
    fn hr(self) -> TSpan {
        (self * 3600.0f32).sec()
    }

    #[inline]
    fn days(self) -> TSpan {
        (self * SEC_PER_DAY as f32).sec()
    }

    #[inline]
    fn wk(self) -> TSpan {
        (self * 604_800.0f32).sec()
    }

    #[inline]
    fn yr(self) -> TSpan {
        (self * 31_557_600.0f32).sec()
    }

    #[inline]
    fn ago(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).sub(self.sec())
    }

    #[inline]
    fn from_now(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).add(self.sec())
    }
}
