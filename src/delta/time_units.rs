//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time_core::{ClockType, TimeUnits};
//!
//! let delta = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().ago(ClockType::UTC);
//! ```

use crate::{ClockType, Delta, SEC_PER_DAY, SEC_PER_DAYI64, TimePoint};

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// Import it explicitly to create `Delta`s directly from rust ints and floats:
/// `use deep_time_core::TimeUnits;`
pub trait TimeUnits: Copy + Sized {
    // ── Delta constructors ─────────────────────────────────────
    fn ns(self) -> Delta;
    fn us(self) -> Delta;
    fn ms(self) -> Delta;
    fn sec(self) -> Delta;
    fn min(self) -> Delta;
    fn hr(self) -> Delta;
    fn days(self) -> Delta; // 86400 s (civil day, not leap-second aware)
    fn wk(self) -> Delta;
    fn yr(self) -> Delta; // 365.25 days (standard approximation)

    // ── TimePoint constructors (anchored at "now" in the chosen clock type) ──
    fn ago(self, clock_type: ClockType) -> TimePoint;
    fn from_now(self, clock_type: ClockType) -> TimePoint;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeUnits for $ty {
                #[inline(always)]
                fn ns(self) -> Delta { Delta::from_ns(self as i64) }

                #[inline(always)]
                fn us(self) -> Delta { Delta::from_us(self as i64) }

                #[inline(always)]
                fn ms(self) -> Delta { Delta::from_ms(self as i64) }

                #[inline(always)]
                fn sec(self) -> Delta { Delta::from_sec(self as i64) }

                #[inline(always)]
                fn min(self) -> Delta { Delta::from_min(self as i64) }

                #[inline(always)]
                fn hr(self) -> Delta { Delta::from_hr(self as i64) }

                #[inline(always)]
                fn days(self) -> Delta { Delta::from_sec((self as i64).saturating_mul(SEC_PER_DAYI64)) }

                #[inline(always)]
                fn wk(self) -> Delta { Delta::from_sec((self as i64).saturating_mul(604_800)) }

                #[inline(always)]
                fn yr(self) -> Delta { Delta::from_sec((self as i64).saturating_mul(31_557_600)) }

                #[inline(always)]
                fn ago(self, clock_type: ClockType) -> TimePoint {
                    TimePoint::from_sec(0, clock_type).sub(self.sec())
                }

                #[inline(always)]
                fn from_now(self, clock_type: ClockType) -> TimePoint {
                    TimePoint::from_sec(0, clock_type).add(self.sec())
                }
            }
        )*
    };
}

impl_time_units_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// f64 support (most useful for fractional units)
impl TimeUnits for f64 {
    #[inline]
    fn ns(self) -> Delta {
        Delta::from_ns(self as i64)
    }

    #[inline]
    fn us(self) -> Delta {
        Delta::from_us(self as i64)
    }

    #[inline]
    fn ms(self) -> Delta {
        Delta::from_ms(self as i64)
    }

    #[inline]
    fn sec(self) -> Delta {
        Delta::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> Delta {
        (self * 60.0).sec()
    }

    #[inline]
    fn hr(self) -> Delta {
        (self * 3600.0).sec()
    }

    #[inline]
    fn days(self) -> Delta {
        (self * SEC_PER_DAY).sec()
    }

    #[inline]
    fn wk(self) -> Delta {
        (self * 604_800.0).sec()
    }

    #[inline]
    fn yr(self) -> Delta {
        (self * 31_557_600.0).sec()
    }

    #[inline]
    fn ago(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).sub(self.sec())
    }

    #[inline]
    fn from_now(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).add(self.sec())
    }
}

impl TimeUnits for f32 {
    #[inline]
    fn ns(self) -> Delta {
        Delta::from_ns(self as i64)
    }

    #[inline]
    fn us(self) -> Delta {
        Delta::from_us(self as i64)
    }

    #[inline]
    fn ms(self) -> Delta {
        Delta::from_ms(self as i64)
    }

    #[inline]
    fn sec(self) -> Delta {
        Delta::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> Delta {
        (self * 60.0f32).sec()
    }

    #[inline]
    fn hr(self) -> Delta {
        (self * 3600.0f32).sec()
    }

    #[inline]
    fn days(self) -> Delta {
        (self * SEC_PER_DAY as f32).sec()
    }

    #[inline]
    fn wk(self) -> Delta {
        (self * 604_800.0f32).sec()
    }

    #[inline]
    fn yr(self) -> Delta {
        (self * 31_557_600.0f32).sec()
    }

    #[inline]
    fn ago(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).sub(self.sec())
    }

    #[inline]
    fn from_now(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).add(self.sec())
    }
}

impl Delta {
    /// Returns a `TimePoint` that is this duration ago from the given clock type.
    #[inline(always)]
    pub fn ago(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).sub_ref(&self)
    }

    /// Returns a `TimePoint` that is this duration from now in the given clock type.
    #[inline(always)]
    pub fn from_now(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).add_ref(&self)
    }
}
