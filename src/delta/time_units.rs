//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time_core::{ClockType, TimeUnits};
//!
//! let delta = 5.seconds() + 250.milliseconds() + 123_456.nanoseconds();
//! let stamp = 3.days().ago(ClockType::UTC);
//! ```

use crate::{ClockType, Delta, Timestamp};

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// Import it explicitly when you want the nice syntax:
/// `use deep_time_core::TimeUnits;`
pub trait TimeUnits: Copy + Sized {
    // ── Delta constructors ─────────────────────────────────────
    fn nanoseconds(self) -> Delta;
    fn microseconds(self) -> Delta;
    fn milliseconds(self) -> Delta;
    fn seconds(self) -> Delta;
    fn minutes(self) -> Delta;
    fn hours(self) -> Delta;
    fn days(self) -> Delta; // 86400 s (civil day, not leap-second aware)
    fn weeks(self) -> Delta;
    fn years(self) -> Delta; // 365.25 days (standard approximation)

    // ── Timestamp constructors (anchored at "now" in the chosen clock type) ──
    fn ago(self, clock_type: ClockType) -> Timestamp;
    fn from_now(self, clock_type: ClockType) -> Timestamp;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeUnits for $ty {
                #[inline(always)]
                fn nanoseconds(self) -> Delta { Delta::from_ns(self as i128) }

                #[inline(always)]
                fn microseconds(self) -> Delta { Delta::from_us(self as i128) }

                #[inline(always)]
                fn milliseconds(self) -> Delta { Delta::from_ms(self as i128) }

                #[inline(always)]
                fn seconds(self) -> Delta { Delta::from_sec(self as i128) }

                #[inline(always)]
                fn minutes(self) -> Delta { Delta::from_min(self as i128) }

                #[inline(always)]
                fn hours(self) -> Delta { Delta::from_hr(self as i128) }

                #[inline(always)]
                fn days(self) -> Delta { Delta::from_sec((self as i128).saturating_mul(86_400)) }

                #[inline(always)]
                fn weeks(self) -> Delta { Delta::from_sec((self as i128).saturating_mul(604_800)) }

                #[inline(always)]
                fn years(self) -> Delta { Delta::from_sec((self as i128).saturating_mul(31_557_600)) }

                #[inline(always)]
                fn ago(self, clock_type: ClockType) -> Timestamp {
                    Timestamp::from_sec(0, clock_type).sub(self.seconds())
                }

                #[inline(always)]
                fn from_now(self, clock_type: ClockType) -> Timestamp {
                    Timestamp::from_sec(0, clock_type).add(self.seconds())
                }
            }
        )*
    };
}

impl_time_units_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// f64 support (most useful for fractional units)
impl TimeUnits for f64 {
    #[inline]
    fn nanoseconds(self) -> Delta {
        Delta::from_ns(self as i128)
    }

    #[inline]
    fn microseconds(self) -> Delta {
        Delta::from_us(self as i128)
    }

    #[inline]
    fn milliseconds(self) -> Delta {
        Delta::from_ms(self as i128)
    }

    #[inline]
    fn seconds(self) -> Delta {
        Delta::from_sec(self as i128)
    }

    #[inline]
    fn minutes(self) -> Delta {
        (self * 60.0).seconds()
    }

    #[inline]
    fn hours(self) -> Delta {
        (self * 3600.0).seconds()
    }

    #[inline]
    fn days(self) -> Delta {
        (self * 86_400.0).seconds()
    }

    #[inline]
    fn weeks(self) -> Delta {
        (self * 604_800.0).seconds()
    }

    #[inline]
    fn years(self) -> Delta {
        (self * 31_557_600.0).seconds()
    }

    #[inline]
    fn ago(self, clock_type: ClockType) -> Timestamp {
        Timestamp::from_sec(0, clock_type).sub(self.seconds())
    }

    #[inline]
    fn from_now(self, clock_type: ClockType) -> Timestamp {
        Timestamp::from_sec(0, clock_type).add(self.seconds())
    }
}

impl Delta {
    /// Returns a `Timestamp` that is this duration ago from the given clock type.
    #[inline(always)]
    pub fn ago(self, clock_type: ClockType) -> Timestamp {
        Timestamp::from_sec(0, clock_type).sub(self)
    }

    /// Returns a `Timestamp` that is this duration from now in the given clock type.
    #[inline(always)]
    pub fn from_now(self, clock_type: ClockType) -> Timestamp {
        Timestamp::from_sec(0, clock_type).add(self)
    }
}
