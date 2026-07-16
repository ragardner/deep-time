//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time::{Scale, TraitsTime};
//!
//! let span = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().before_zero(Scale::UTC);
//!
//! // Wall-clock relative (requires the `std` feature):
//! // let past = 3.days().ago();
//! // let future = 3.days().from_now();
//! ```

use crate::{Dt, SEC_PER_DAY, SEC_PER_DAY_F, Scale};

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// ## Examples
///
/// ```rust
/// use deep_time::TraitsTime;
///
/// let dt = 500.ns();
/// let dt = 500.us();
/// let dt = 500.ms();
/// let dt = 30.sec();
/// let dt = 30.mins();
/// let dt = 2.hours();
/// let dt = 5.days();
/// let dt = 4.weeks();
/// let dt = 10.years();
/// ```
pub trait TraitsTime: Copy + Sized {
    /// Duration of `self` nanoseconds as a [`Dt`].
    fn ns(self) -> Dt;
    /// Duration of `self` microseconds as a [`Dt`].
    fn us(self) -> Dt;
    /// Duration of `self` milliseconds as a [`Dt`].
    fn ms(self) -> Dt;
    /// Duration of `self` seconds as a [`Dt`].
    fn sec(self) -> Dt;
    /// Duration of `self` minutes as a [`Dt`].
    fn mins(self) -> Dt;
    /// Duration of `self` hours as a [`Dt`].
    fn hours(self) -> Dt;
    /// Duration of `self` civil days (86 400 s each; not leap-second aware).
    fn days(self) -> Dt;
    /// Duration of `self` weeks (7 civil days each).
    fn weeks(self) -> Dt;
    /// Duration of `self` Julian years (365.25 civil days each).
    fn years(self) -> Dt;
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TraitsTime for $ty {
                #[inline]
                fn ns(self) -> Dt { Dt::from_ns(self as i128, 0, Scale::TAI, Scale::TAI) }

                #[inline]
                fn us(self) -> Dt { Dt::from_us(self as i128, 0, Scale::TAI, Scale::TAI) }

                #[inline]
                fn ms(self) -> Dt { Dt::from_ms(self as i128, 0, Scale::TAI, Scale::TAI) }

                #[inline]
                fn sec(self) -> Dt { Dt::from_sec(self as i128, Scale::TAI, Scale::TAI) }

                #[inline]
                fn mins(self) -> Dt { Dt::from_mins(self as i128, 0, Scale::TAI, Scale::TAI) }

                #[inline]
                fn hours(self) -> Dt { Dt::from_hours(self as i128, 0, Scale::TAI, Scale::TAI) }

                #[inline]
                fn days(self) -> Dt { Dt::from_sec((self as i128).saturating_mul(SEC_PER_DAY), Scale::TAI, Scale::TAI) }

                #[inline]
                fn weeks(self) -> Dt { Dt::from_sec((self  as i128).saturating_mul(604_800), Scale::TAI, Scale::TAI) }

                #[inline]
                fn years(self) -> Dt { Dt::from_sec((self  as i128).saturating_mul(31_557_600), Scale::TAI, Scale::TAI) }
            }
        )*
    };
}

impl_time_units_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64);

// `u128` alone among the integer impls can exceed `i128::MAX`; saturate instead of wrapping.
impl TraitsTime for u128 {
    #[inline]
    fn ns(self) -> Dt {
        Dt::from_ns(Dt::to_i128(self), 0, Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn us(self) -> Dt {
        Dt::from_us(Dt::to_i128(self), 0, Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn ms(self) -> Dt {
        Dt::from_ms(Dt::to_i128(self), 0, Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn sec(self) -> Dt {
        Dt::from_sec(Dt::to_i128(self), Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn mins(self) -> Dt {
        Dt::from_mins(Dt::to_i128(self), 0, Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn hours(self) -> Dt {
        Dt::from_hours(Dt::to_i128(self), 0, Scale::TAI, Scale::TAI)
    }

    #[inline]
    fn days(self) -> Dt {
        Dt::from_sec(
            Dt::to_i128(self).saturating_mul(SEC_PER_DAY),
            Scale::TAI,
            Scale::TAI,
        )
    }

    #[inline]
    fn weeks(self) -> Dt {
        Dt::from_sec(
            Dt::to_i128(self).saturating_mul(604_800),
            Scale::TAI,
            Scale::TAI,
        )
    }

    #[inline]
    fn years(self) -> Dt {
        Dt::from_sec(
            Dt::to_i128(self).saturating_mul(31_557_600),
            Scale::TAI,
            Scale::TAI,
        )
    }
}

impl TraitsTime for f64 {
    #[inline]
    fn ns(self) -> Dt {
        crate::macros::from_sec_f!(self * 1e-9)
    }

    #[inline]
    fn us(self) -> Dt {
        crate::macros::from_sec_f!(self * 1e-6)
    }

    #[inline]
    fn ms(self) -> Dt {
        crate::macros::from_sec_f!(self * 1e-3)
    }

    #[inline]
    fn sec(self) -> Dt {
        crate::macros::from_sec_f!(self)
    }

    #[inline]
    fn mins(self) -> Dt {
        (self * 60.0).sec()
    }

    #[inline]
    fn hours(self) -> Dt {
        (self * 3600.0).sec()
    }

    #[inline]
    fn days(self) -> Dt {
        (self * SEC_PER_DAY_F).sec()
    }

    #[inline]
    fn weeks(self) -> Dt {
        (self * 604_800.0).sec()
    }

    #[inline]
    fn years(self) -> Dt {
        (self * 31_557_600.0).sec()
    }
}

impl TraitsTime for f32 {
    #[inline]
    fn ns(self) -> Dt {
        crate::macros::from_sec_f!(self as f64 * 1e-9)
    }

    #[inline]
    fn us(self) -> Dt {
        crate::macros::from_sec_f!(self as f64 * 1e-6)
    }

    #[inline]
    fn ms(self) -> Dt {
        crate::macros::from_sec_f!(self as f64 * 1e-3)
    }

    #[inline]
    fn sec(self) -> Dt {
        crate::macros::from_sec_f!(self as f64)
    }

    #[inline]
    fn mins(self) -> Dt {
        (self * 60.0f32).sec()
    }

    #[inline]
    fn hours(self) -> Dt {
        (self * 3600.0f32).sec()
    }

    #[inline]
    fn days(self) -> Dt {
        (self * SEC_PER_DAY as f32).sec()
    }

    #[inline]
    fn weeks(self) -> Dt {
        (self * 604_800.0f32).sec()
    }

    #[inline]
    fn years(self) -> Dt {
        (self * 31_557_600.0f32).sec()
    }
}
