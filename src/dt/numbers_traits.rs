//! Ergonomic time-unit constructors (optional import).
//!
//! ```
//! use deep_time::{Scale, TimeTraits};
//!
//! let span = 5.sec() + 250.ms() + 123_456.ns();
//! let stamp = 3.days().before_zero(Scale::UTC);
//!
//! // Wall-clock relative (requires the `std` feature):
//! // let past = 3.days().ago();
//! // let future = 3.days().from_now();
//! ```

use crate::{Dt, SEC_PER_DAY, SEC_PER_DAY_F, Scale};

/// Trait that adds ergonomic attosecond conversions on integer values.
///
/// Covers both directions:
/// - `attos_to_*` — total attoseconds → whole units (truncating division)
/// - `*_to_attos` — whole units → total attoseconds (saturating multiply)
///
/// ## Examples
///
/// ```rust
/// use deep_time::AttosTraits;
///
/// let attos: i128 = -5_600_000_000_000_000_000;
/// let seconds = attos.attos_to_sec();
/// assert_eq!(seconds, -5);
///
/// assert_eq!(5_i128.ns_to_attos(), 5_000_000_000);
/// assert_eq!(1_i128.ms_to_attos().attos_to_ms(), 1);
/// ```
pub trait AttosTraits: Copy + Sized {
    /// attoseconds → seconds (s)
    fn attos_to_sec(self) -> i128;

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

    /// femtoseconds → attoseconds (`× 10³`)
    fn fs_to_attos(self) -> i128;

    /// picoseconds → attoseconds (`× 10⁶`)
    fn ps_to_attos(self) -> i128;

    /// nanoseconds → attoseconds (`× 10⁹`)
    fn ns_to_attos(self) -> i128;

    /// microseconds → attoseconds (`× 10¹²`)
    fn us_to_attos(self) -> i128;

    /// milliseconds → attoseconds (`× 10¹⁵`)
    fn ms_to_attos(self) -> i128;

    /// seconds → attoseconds (`× 10¹⁸`)
    fn sec_to_attos(self) -> i128;

    /// minutes → attoseconds (`× 60 × 10¹⁸`)
    fn mins_to_attos(self) -> i128;

    /// hours → attoseconds (`× 3600 × 10¹⁸`)
    fn hours_to_attos(self) -> i128;
}

impl AttosTraits for i128 {
    #[inline]
    fn attos_to_sec_f(self) -> f64 {
        Dt::attos_to_sec_f(self)
    }

    #[inline]
    fn attos_to_sec(self) -> i128 {
        Dt::attos_to_sec(self)
    }

    #[inline]
    fn attos_to_ms(self) -> i128 {
        Dt::attos_to_ms(self)
    }

    #[inline]
    fn attos_to_us(self) -> i128 {
        Dt::attos_to_us(self)
    }

    #[inline]
    fn attos_to_ns(self) -> i128 {
        Dt::attos_to_ns(self)
    }

    #[inline]
    fn attos_to_ps(self) -> i128 {
        Dt::attos_to_ps(self)
    }

    #[inline]
    fn attos_to_fs(self) -> i128 {
        Dt::attos_to_fs(self)
    }

    #[inline]
    fn fs_to_attos(self) -> i128 {
        Dt::fs_to_attos(self)
    }

    #[inline]
    fn ps_to_attos(self) -> i128 {
        Dt::ps_to_attos(self)
    }

    #[inline]
    fn ns_to_attos(self) -> i128 {
        Dt::ns_to_attos(self)
    }

    #[inline]
    fn us_to_attos(self) -> i128 {
        Dt::us_to_attos(self)
    }

    #[inline]
    fn ms_to_attos(self) -> i128 {
        Dt::ms_to_attos(self)
    }

    #[inline]
    fn sec_to_attos(self) -> i128 {
        Dt::sec_to_attos(self)
    }

    #[inline]
    fn mins_to_attos(self) -> i128 {
        Dt::mins_to_attos(self)
    }

    #[inline]
    fn hours_to_attos(self) -> i128 {
        Dt::hours_to_attos(self)
    }
}

/// Trait that adds ergonomic time-unit methods to integers and floats.
///
/// ## Examples
///
/// ```rust
/// use deep_time::TimeTraits;
///
/// let dt = 5.days();
/// ```
pub trait TimeTraits: Copy + Sized {
    fn ns(self) -> Dt;
    fn us(self) -> Dt;
    fn ms(self) -> Dt;
    fn sec(self) -> Dt;
    fn mins(self) -> Dt;
    fn hours(self) -> Dt;
    fn days(self) -> Dt; // 86400 s (civil day, not leap-second aware)
    fn weeks(self) -> Dt;
    fn years(self) -> Dt; // 365.25 days (standard approximation)
}

// Integer implementations (all common signed/unsigned types)
macro_rules! impl_time_units_int {
    ($($ty:ty),* $(,)?) => {
        $(
            impl TimeTraits for $ty {
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
impl TimeTraits for u128 {
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

impl TimeTraits for f64 {
    #[inline]
    fn ns(self) -> Dt {
        crate::from_sec_f!(self * 1e-9)
    }

    #[inline]
    fn us(self) -> Dt {
        crate::from_sec_f!(self * 1e-6)
    }

    #[inline]
    fn ms(self) -> Dt {
        crate::from_sec_f!(self * 1e-3)
    }

    #[inline]
    fn sec(self) -> Dt {
        crate::from_sec_f!(self)
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

impl TimeTraits for f32 {
    #[inline]
    fn ns(self) -> Dt {
        crate::from_sec_f!(self as f64 * 1e-9)
    }

    #[inline]
    fn us(self) -> Dt {
        crate::from_sec_f!(self as f64 * 1e-6)
    }

    #[inline]
    fn ms(self) -> Dt {
        crate::from_sec_f!(self as f64 * 1e-3)
    }

    #[inline]
    fn sec(self) -> Dt {
        crate::from_sec_f!(self as f64)
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
