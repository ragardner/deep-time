//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time_core::{ClockType, TimeUnits};
//!
//! let span = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().ago(ClockType::UTC);
//! ```

use crate::{ClockType, SEC_PER_DAY, SEC_PER_DAYI64, TimePoint, TimeSpan};

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// Import it explicitly to create `TimeSpan`s directly from rust ints and floats:
/// `use deep_time_core::TimeUnits;`
pub trait TimeUnits: Copy + Sized {
    // ── TimeSpan constructors ─────────────────────────────────────
    fn ns(self) -> TimeSpan;
    fn us(self) -> TimeSpan;
    fn ms(self) -> TimeSpan;
    fn sec(self) -> TimeSpan;
    fn min(self) -> TimeSpan;
    fn hr(self) -> TimeSpan;
    fn days(self) -> TimeSpan; // 86400 s (civil day, not leap-second aware)
    fn wk(self) -> TimeSpan;
    fn yr(self) -> TimeSpan; // 365.25 days (standard approximation)

    // ── TimePoint constructors (anchored at "now" in the chosen clock type) ──
    fn ago(self, clock_type: ClockType) -> TimePoint;
    fn from_now(self, clock_type: ClockType) -> TimePoint;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeUnits for $ty {
                #[inline]
                fn ns(self) -> TimeSpan { TimeSpan::from_ns(self as i128) }

                #[inline]
                fn us(self) -> TimeSpan { TimeSpan::from_us(self as i128) }

                #[inline]
                fn ms(self) -> TimeSpan { TimeSpan::from_ms(self as i128) }

                #[inline]
                fn sec(self) -> TimeSpan { TimeSpan::from_sec(self as i64) }

                #[inline]
                fn min(self) -> TimeSpan { TimeSpan::from_min(self as i64) }

                #[inline]
                fn hr(self) -> TimeSpan { TimeSpan::from_hr(self as i64) }

                #[inline]
                fn days(self) -> TimeSpan { TimeSpan::from_sec((self as i64).saturating_mul(SEC_PER_DAYI64)) }

                #[inline]
                fn wk(self) -> TimeSpan { TimeSpan::from_sec((self as i64).saturating_mul(604_800)) }

                #[inline]
                fn yr(self) -> TimeSpan { TimeSpan::from_sec((self as i64).saturating_mul(31_557_600)) }

                #[inline]
                fn ago(self, clock_type: ClockType) -> TimePoint {
                    TimePoint::from_sec(0, clock_type).sub(self.sec())
                }

                #[inline]
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
    fn ns(self) -> TimeSpan {
        TimeSpan::from_ns(self as i128)
    }

    #[inline]
    fn us(self) -> TimeSpan {
        TimeSpan::from_us(self as i128)
    }

    #[inline]
    fn ms(self) -> TimeSpan {
        TimeSpan::from_ms(self as i128)
    }

    #[inline]
    fn sec(self) -> TimeSpan {
        TimeSpan::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> TimeSpan {
        (self * 60.0).sec()
    }

    #[inline]
    fn hr(self) -> TimeSpan {
        (self * 3600.0).sec()
    }

    #[inline]
    fn days(self) -> TimeSpan {
        (self * SEC_PER_DAY).sec()
    }

    #[inline]
    fn wk(self) -> TimeSpan {
        (self * 604_800.0).sec()
    }

    #[inline]
    fn yr(self) -> TimeSpan {
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
    fn ns(self) -> TimeSpan {
        TimeSpan::from_ns(self as i128)
    }

    #[inline]
    fn us(self) -> TimeSpan {
        TimeSpan::from_us(self as i128)
    }

    #[inline]
    fn ms(self) -> TimeSpan {
        TimeSpan::from_ms(self as i128)
    }

    #[inline]
    fn sec(self) -> TimeSpan {
        TimeSpan::from_sec(self as i64)
    }

    #[inline]
    fn min(self) -> TimeSpan {
        (self * 60.0f32).sec()
    }

    #[inline]
    fn hr(self) -> TimeSpan {
        (self * 3600.0f32).sec()
    }

    #[inline]
    fn days(self) -> TimeSpan {
        (self * SEC_PER_DAY as f32).sec()
    }

    #[inline]
    fn wk(self) -> TimeSpan {
        (self * 604_800.0f32).sec()
    }

    #[inline]
    fn yr(self) -> TimeSpan {
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

impl TimeSpan {
    /// Returns a `TimePoint` that is this duration ago from the given clock type.
    #[inline]
    pub fn ago(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).sub_ref(&self)
    }

    /// Returns a `TimePoint` that is this duration from now in the given clock type.
    #[inline]
    pub fn from_now(self, clock_type: ClockType) -> TimePoint {
        TimePoint::from_sec(0, clock_type).add_ref(&self)
    }
}
