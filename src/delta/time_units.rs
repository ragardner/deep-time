//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time_core::{TimePov, TimeUnits};
//!
//! let delta = 5.seconds() + 250.milliseconds() + 123_456.nanoseconds();
//! let point = 3.days().ago(TimePov::UTC);
//! ```

use crate::{Delta, Point, TimePov};

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

    // ── Point constructors (anchored at "now" in the chosen POV) ──
    fn ago(self, pov: TimePov) -> Point;
    fn from_now(self, pov: TimePov) -> Point;
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
                fn ago(self, pov: TimePov) -> Point {
                    Point::from_sec(0, pov).sub(self.seconds())
                }

                #[inline(always)]
                fn from_now(self, pov: TimePov) -> Point {
                    Point::from_sec(0, pov).add(self.seconds())
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
    fn ago(self, pov: TimePov) -> Point {
        Point::from_sec(0, pov).sub(self.seconds())
    }

    #[inline]
    fn from_now(self, pov: TimePov) -> Point {
        Point::from_sec(0, pov).add(self.seconds())
    }
}

impl Delta {
    /// Returns a `Point` that is this duration ago from the given time scale.
    #[inline(always)]
    pub fn ago(self, pov: TimePov) -> Point {
        Point::from_sec(0, pov).sub(self)
    }

    /// Returns a `Point` that is this duration from now in the given time scale.
    #[inline(always)]
    pub fn from_now(self, pov: TimePov) -> Point {
        Point::from_sec(0, pov).add(self)
    }
}
