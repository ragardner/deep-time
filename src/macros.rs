//! Macros for easy unit conversion and
//! [`Dt`](../struct.Dt.html) construction.
//!
//! Each macro expands to a call on an equivalent [`Dt`](../struct.Dt.html)
//! method.
//!
//! ## Overview
//!
//! ### Unit → attoseconds
//!
//! Returns total attoseconds as `i128`.
//!
//! - [`fs!`]
//! - [`ps!`]
//! - [`ns!`]
//! - [`us!`]
//! - [`ms!`]
//! - [`sec!`]
//! - [`sec_f!`]
//! - [`mins!`]
//! - [`hours!`]
//! - [`days!`]
//! - [`days_f!`]
//! - [`weeks!`]
//!
//! ### Attoseconds → unit
//!
//! Returns a whole-unit count as `i128`, or a lossy
//! [`Real`](../type.Real.html) for the `_f` forms.
//!
//! - [`as_fs!`]
//! - [`as_ps!`]
//! - [`as_ns!`]
//! - [`as_us!`]
//! - [`as_ms!`]
//! - [`as_sec!`]
//! - [`as_sec_f!`]
//! - [`as_mins!`]
//! - [`as_hours!`]
//! - [`as_days!`]
//! - [`as_days_f!`]
//! - [`as_weeks!`]
//!
//! ### Instant / duration
//!
//! Returns a [`Dt`](../struct.Dt.html).
//!
//! - [`dt!`]
//! - [`from_sec!`]
//! - [`from_sec_f!`]
//! - [`from_ms!`]
//! - [`from_us!`]
//! - [`from_ns!`]
//! - [`from_ps!`]
//! - [`from_fs!`]
//! - [`from_days_f!`]
//! - [`from_ymd!`]
//! - [`from_jd!`]
//! - [`from_jd_f!`]
//! - [`from_mjd!`]
//! - [`from_mjd_f!`]
//!
//! ## Import paths
//!
//! All macros can be imported from this module:
//!
//! ```
//! use deep_time::macros::{from_ns, ms, sec};
//! ```
//!
//! These are also available at the crate root via `use deep_time::{…}`:
//!
//! - [`ns!`]
//! - [`ms!`]
//! - [`days_f!`]
//! - [`dt!`]
//! - [`from_sec_f!`]
//! - [`from_ymd!`]
//!
//! The rest are available only under `deep_time::macros`.
//! `use deep_time::macros::*` brings in every macro listed above.
//!
//! ## How they work
//!
//! - **Attosecond storage unit.** Forward converters such as [`ms!`] and
//!   [`sec!`] return total attoseconds as `i128`. Reverse converters such as
//!   [`as_ms!`] and [`as_sec!`] take total attoseconds and return a count in
//!   the named unit.
//! - **Truncation toward zero.** Integer reverse converters use ordinary
//!   `i128` division (`attos / unit`). Any leftover below one whole unit is
//!   dropped, and the result moves toward zero—not toward −∞. For example,
//!   −0.5 s as whole seconds is `0`, and −1.5 s is `-1`. The floating reverse
//!   converters ([`as_sec_f!`], [`as_days_f!`]) are lossy `f64` casts instead.
//! - **Signed remainders on constructors.** Macros such as [`from_sec!`] and
//!   [`from_ns!`] accept an optional fractional remainder in **attoseconds**.
//!   Both signs of the remainder are valid; the total is
//!   `whole × unit + frac` (with saturating arithmetic on the underlying
//!   method). The same total can often be written with a signed remainder or
//!   as a floor-style split with a non-negative remainder.
//! - **Scale labels vs conversion.** Most `from_*` macros only set the
//!   returned [`Dt`](../struct.Dt.html)'s `scale` / `target` fields; they do
//!   **not** convert the attosecond count between time scales. [`from_ymd!`],
//!   [`from_jd!`], [`from_jd_f!`], [`from_mjd!`], and [`from_mjd_f!`] are the
//!   exceptions: they build a [`Dt`](../struct.Dt.html) on TAI (converting
//!   from the `on=` scale when needed) and set `target` from `on=`, as
//!   documented on each macro.
//! - **`const` contexts.** Expansions call `const` methods on
//!   [`Dt`](../struct.Dt.html), so these macros work in `const` contexts where
//!   the underlying method does.
//!
//! ## Examples
//!
//! ```
//! use deep_time::{Dt, Scale, dt, from_ymd, ms, ns};
//! use deep_time::macros::{as_sec, from_sec, sec};
//!
//! // Unit helpers avoid hand-counting zeros in attosecond literals.
//! assert_eq!(ms!(300), Dt::ms_to_attos(300));
//! assert_eq!(ns!(1), 1_000_000_000);
//!
//! // Whole seconds + sub-second remainder (remainder is attoseconds).
//! let a = from_sec!(1, ms!(300));
//! assert_eq!(a, dt!(1_300_000_000_000_000_000));
//!
//! // Reverse conversion truncates toward zero.
//! assert_eq!(as_sec!(sec!(1) + ms!(900)), 1);
//! assert_eq!(as_sec!(-ms!(500)), 0);
//! assert_eq!(as_sec!(-sec!(1) - ms!(500)), -1);
//!
//! // Calendar construction: builds a Dt, converting civil time to TAI.
//! assert_eq!(from_ymd!(2000, 1, 1; 12, on=Scale::TAI), Dt::ZERO);
//! ```
//!
//! See each macro's own documentation for accepted forms, defaults, and
//! links to the corresponding [`Dt`](../struct.Dt.html) methods.

/// Converts whole femtoseconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::fs_to_attos`](../struct.Dt.html#method.fs_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::fs};
///
/// assert_eq!(fs!(1), Dt::fs_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __fs {
    ($x:expr) => {
        $crate::Dt::fs_to_attos($x)
    };
}

#[doc(inline)]
pub use __fs as fs;

/// Converts whole picoseconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::ps_to_attos`](../struct.Dt.html#method.ps_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::ps};
///
/// assert_eq!(ps!(1), Dt::ps_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __ps {
    ($x:expr) => {
        $crate::Dt::ps_to_attos($x)
    };
}

#[doc(inline)]
pub use __ps as ps;

/// Converts whole nanoseconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::ns_to_attos`](../struct.Dt.html#method.ns_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, ns};
///
/// assert_eq!(ns!(1), Dt::ns_to_attos(1));
/// ```
#[macro_export]
macro_rules! ns {
    ($x:expr) => {
        $crate::Dt::ns_to_attos($x)
    };
}

/// Converts whole microseconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::us_to_attos`](../struct.Dt.html#method.us_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::us};
///
/// assert_eq!(us!(1), Dt::us_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __us {
    ($x:expr) => {
        $crate::Dt::us_to_attos($x)
    };
}

#[doc(inline)]
pub use __us as us;

/// Converts whole milliseconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::ms_to_attos`](../struct.Dt.html#method.ms_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, ms};
///
/// assert_eq!(ms!(1), Dt::ms_to_attos(1));
/// ```
#[macro_export]
macro_rules! ms {
    ($x:expr) => {
        $crate::Dt::ms_to_attos($x)
    };
}

/// Converts whole seconds (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::sec_to_attos`](../struct.Dt.html#method.sec_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::sec};
///
/// assert_eq!(sec!(1), Dt::sec_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __sec {
    ($x:expr) => {
        $crate::Dt::sec_to_attos($x)
    };
}

#[doc(inline)]
pub use __sec as sec;

/// Converts a floating-point second count ([`Real`](crate::Real)) to
/// total attoseconds (`i128`).
///
/// Equivalent to [`Dt::sec_f_to_attos`](../struct.Dt.html#method.sec_f_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::sec_f};
///
/// assert_eq!(sec_f!(1.5), Dt::sec_f_to_attos(1.5));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __sec_f {
    ($x:expr) => {
        $crate::Dt::sec_f_to_attos($x)
    };
}

#[doc(inline)]
pub use __sec_f as sec_f;

/// Converts whole minutes (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::mins_to_attos`](../struct.Dt.html#method.mins_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::mins};
///
/// assert_eq!(mins!(1), Dt::mins_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __mins {
    ($x:expr) => {
        $crate::Dt::mins_to_attos($x)
    };
}

#[doc(inline)]
pub use __mins as mins;

/// Converts whole hours (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::hours_to_attos`](../struct.Dt.html#method.hours_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::hours};
///
/// assert_eq!(hours!(1), Dt::hours_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __hours {
    ($x:expr) => {
        $crate::Dt::hours_to_attos($x)
    };
}

#[doc(inline)]
pub use __hours as hours;

/// Converts whole days (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::days_to_attos`](../struct.Dt.html#method.days_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::days};
///
/// assert_eq!(days!(1), Dt::days_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __days {
    ($x:expr) => {
        $crate::Dt::days_to_attos($x)
    };
}

#[doc(inline)]
pub use __days as days;

/// Converts a floating-point day count ([`Real`](crate::Real)) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::days_f_to_attos`](../struct.Dt.html#method.days_f_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, days_f};
///
/// assert_eq!(days_f!(0.25), Dt::days_f_to_attos(0.25));
/// ```
#[macro_export]
macro_rules! days_f {
    ($x:expr) => {
        $crate::Dt::days_f_to_attos($x)
    };
}

/// Converts whole weeks (`i128`) to total attoseconds (`i128`).
///
/// Equivalent to [`Dt::weeks_to_attos`](../struct.Dt.html#method.weeks_to_attos).
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, macros::weeks};
///
/// assert_eq!(weeks!(1), Dt::weeks_to_attos(1));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __weeks {
    ($x:expr) => {
        $crate::Dt::weeks_to_attos($x)
    };
}

#[doc(inline)]
pub use __weeks as weeks;

/// Converts total attoseconds (`i128`) → whole femtoseconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_fs`](../struct.Dt.html#method.attos_to_fs).
///
/// Truncates toward zero.
///
/// Half a femtosecond is `500` attoseconds (no smaller named unit macro).
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_fs, fs};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 fs becomes −1 femtoseconds
/// assert_eq!(as_fs!(-fs!(1) - 500), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_fs {
    ($x:expr) => {
        $crate::Dt::attos_to_fs($x)
    };
}

#[doc(inline)]
pub use __as_fs as as_fs;

/// Converts total attoseconds (`i128`) → whole picoseconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_ps`](../struct.Dt.html#method.attos_to_ps).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_ps, fs, ps};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 ps becomes −1 picoseconds
/// assert_eq!(as_ps!(-ps!(1) - fs!(500)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_ps {
    ($x:expr) => {
        $crate::Dt::attos_to_ps($x)
    };
}

#[doc(inline)]
pub use __as_ps as as_ps;

/// Converts total attoseconds (`i128`) → whole nanoseconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_ns`](../struct.Dt.html#method.attos_to_ns).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_ns, ns, ps};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 ns becomes −1 nanoseconds
/// assert_eq!(as_ns!(-ns!(1) - ps!(500)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_ns {
    ($x:expr) => {
        $crate::Dt::attos_to_ns($x)
    };
}

#[doc(inline)]
pub use __as_ns as as_ns;

/// Converts total attoseconds (`i128`) → whole microseconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_us`](../struct.Dt.html#method.attos_to_us).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_us, ns, us};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 µs becomes −1 microseconds
/// assert_eq!(as_us!(-us!(1) - ns!(500)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_us {
    ($x:expr) => {
        $crate::Dt::attos_to_us($x)
    };
}

#[doc(inline)]
pub use __as_us as as_us;

/// Converts total attoseconds (`i128`) → whole milliseconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_ms`](../struct.Dt.html#method.attos_to_ms).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_ms, ms, us};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 ms becomes −1 milliseconds
/// assert_eq!(as_ms!(-ms!(1) - us!(500)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_ms {
    ($x:expr) => {
        $crate::Dt::attos_to_ms($x)
    };
}

#[doc(inline)]
pub use __as_ms as as_ms;

/// Converts total attoseconds (`i128`) → whole seconds (`i128`).
///
/// Equivalent to [`Dt::attos_to_sec`](../struct.Dt.html#method.attos_to_sec).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_sec, ms, sec};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 s becomes −1 seconds
/// assert_eq!(as_sec!(-sec!(1) - ms!(500)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_sec {
    ($x:expr) => {
        $crate::Dt::attos_to_sec($x)
    };
}

#[doc(inline)]
pub use __as_sec as as_sec;

/// Converts total attoseconds (`i128`) → lossy float seconds ([`Real`](crate::Real)).
///
/// Equivalent to [`Dt::attos_to_sec_f`](../struct.Dt.html#method.attos_to_sec_f).
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_sec_f, ms, sec};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 s becomes -1.5 seconds
/// assert_eq!(as_sec_f!(-sec!(1) - ms!(500)), -1.5);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_sec_f {
    ($x:expr) => {
        $crate::Dt::attos_to_sec_f($x)
    };
}

#[doc(inline)]
pub use __as_sec_f as as_sec_f;

/// Converts total attoseconds (`i128`) → whole minutes (`i128`).
///
/// Equivalent to [`Dt::attos_to_mins`](../struct.Dt.html#method.attos_to_mins).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_mins, mins, sec};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 min becomes −1 minutes
/// assert_eq!(as_mins!(-mins!(1) - sec!(30)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_mins {
    ($x:expr) => {
        $crate::Dt::attos_to_mins($x)
    };
}

#[doc(inline)]
pub use __as_mins as as_mins;

/// Converts total attoseconds (`i128`) → whole hours (`i128`).
///
/// Equivalent to [`Dt::attos_to_hours`](../struct.Dt.html#method.attos_to_hours).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_hours, hours, mins};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 h becomes −1 hours
/// assert_eq!(as_hours!(-hours!(1) - mins!(30)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_hours {
    ($x:expr) => {
        $crate::Dt::attos_to_hours($x)
    };
}

#[doc(inline)]
pub use __as_hours as as_hours;

/// Converts total attoseconds (`i128`) → whole days (`i128`).
///
/// Equivalent to [`Dt::attos_to_days`](../struct.Dt.html#method.attos_to_days).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_days, days, hours};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 d becomes −1 days
/// assert_eq!(as_days!(-days!(1) - hours!(12)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_days {
    ($x:expr) => {
        $crate::Dt::attos_to_days($x)
    };
}

#[doc(inline)]
pub use __as_days as as_days;

/// Converts total attoseconds (`i128`) → lossy float days ([`Real`](crate::Real)).
///
/// Equivalent to [`Dt::attos_to_days_f`](../struct.Dt.html#method.attos_to_days_f).
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_days_f, days, hours};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 d becomes -1.5 days
/// assert_eq!(as_days_f!(-days!(1) - hours!(12)), -1.5);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_days_f {
    ($x:expr) => {
        $crate::Dt::attos_to_days_f($x)
    };
}

#[doc(inline)]
pub use __as_days_f as as_days_f;

/// Converts total attoseconds (`i128`) → whole weeks (`i128`).
///
/// Equivalent to [`Dt::attos_to_weeks`](../struct.Dt.html#method.attos_to_weeks).
///
/// Truncates toward zero.
///
/// ## Examples
///
/// Example shows attos inputs being built with macros rather than counting
/// attosecond zeros by hand.
///
/// ```rust
/// use deep_time::macros::{as_weeks, days, hours, weeks};
///
/// // an amount of attoseconds that is equal to
/// // −1.5 wk becomes −1 weeks
/// assert_eq!(as_weeks!(-weeks!(1) - days!(3) - hours!(12)), -1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __as_weeks {
    ($x:expr) => {
        $crate::Dt::attos_to_weeks($x)
    };
}

#[doc(inline)]
pub use __as_weeks as as_weeks;

/// Builds a [`Dt`](../struct.Dt.html) from total attoseconds with optional scale labels.
///
/// - Equivalent to [`Dt::new`](../struct.Dt.html#method.new).
/// - Does **not** perform time-scale conversion.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// | Form | Meaning |
/// |------|---------|
/// | `dt!(attos)` | both scales TAI |
/// | `dt!(attos, on=s)` | scale and target `s` |
/// | `dt!(attos, target=t)` | scale TAI, target `t` |
/// | `dt!(attos, on=s, target=t)` | either order |
/// | `dt!(attos, target=t, on=s)` | either order |
///
/// ## Examples
///
/// ```rust
/// use deep_time::{Dt, Scale, dt};
///
/// let a = dt!(Dt::sec_to_attos(1));
/// let b = dt!(0, on=Scale::UTC);
/// let c = dt!(0, on=Scale::TAI, target=Scale::UTC);
/// let d = dt!(0, target=Scale::UTC, on=Scale::TAI);
/// let e = dt!(0, target=Scale::UTC);
///
/// assert_eq!(a, Dt::new(Dt::sec_to_attos(1), Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::new(0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, Dt::new(0, Scale::TAI, Scale::UTC));
/// assert_eq!(d, c);
/// assert_eq!(e, Dt::new(0, Scale::TAI, Scale::UTC));
/// ```
#[macro_export]
macro_rules! dt {
    ($attos:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::new($attos, $scale, $target)
    };
    ($attos:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::new($attos, $scale, $target)
    };
    ($attos:expr, on=$scale:expr) => {
        $crate::Dt::new($attos, $scale, $scale)
    };
    ($attos:expr, target=$target:expr) => {
        $crate::Dt::new($attos, $crate::Scale::TAI, $target)
    };
    ($attos:expr) => {
        $crate::Dt::new($attos, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](../struct.Dt.html) from whole seconds and an optional signed
/// sub-second remainder (attoseconds).
///
/// - Equivalent to [`Dt::from_sec_and_frac`](../struct.Dt.html#method.from_sec_and_frac).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`ms!`](../macro.ms.html) (or another `*_to_attos`
/// helper) instead of writing the attosecond literal by hand.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_sec!(sec)
/// from_sec!(sec, frac)
/// from_sec!(sec, on=s)
/// from_sec!(sec, target=t)
/// from_sec!(sec, on=s, target=t)
/// from_sec!(sec, target=t, on=s)
/// from_sec!(sec, frac, on=s)
/// from_sec!(sec, frac, target=t)
/// from_sec!(sec, frac, on=s, target=t)
/// from_sec!(sec, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_sec, ms};
///
/// // 1.3 s → whole seconds + 300 ms remainder
/// let a = from_sec!(1);
/// let b = from_sec!(1, ms!(300));
/// let c = from_sec!(-1, ms!(-300), on=Scale::TAI);
/// let d = from_sec!(0, on=Scale::UTC);
/// let e = from_sec!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_sec!(1, target=Scale::UTC, on=Scale::TAI);
///
/// assert_eq!(a, Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_sec_and_frac(1, ms!(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_sec_and_frac(-1, ms!(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_sec_and_frac(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_sec!(-1, ms!(-300));
/// let floored = from_sec!(-2, ms!(700));
/// assert_eq!(signed, floored);
/// assert_eq!(signed, Dt::from_sec_and_frac(-1, ms!(-300), Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_sec {
    ($sec:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale, $target)
    };
    ($sec:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale, $target)
    };
    ($sec:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale, $target)
    };
    ($sec:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale, $target)
    };
    ($sec:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale, $scale)
    };
    ($sec:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $crate::Scale::TAI, $target)
    };
    ($sec:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale, $scale)
    };
    ($sec:expr, target=$target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $crate::Scale::TAI, $target)
    };
    ($sec:expr, $frac:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($sec:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_sec as from_sec;

/// Builds a [`Dt`](../struct.Dt.html) from a floating-point seconds count with optional
/// scale labels.
///
/// - Equivalent to [`Dt::from_sec_f`](../struct.Dt.html#method.from_sec_f).
/// - Does **not** perform time-scale conversion.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// | Form | Meaning |
/// |------|---------|
/// | `from_sec_f!(sec)` | both scales TAI |
/// | `from_sec_f!(sec, on=s)` | scale and target `s` |
/// | `from_sec_f!(sec, target=t)` | scale TAI, target `t` |
/// | `from_sec_f!(sec, on=s, target=t)` | either order |
/// | `from_sec_f!(sec, target=t, on=s)` | either order |
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::from_sec_f;
///
/// let a = from_sec_f!(5.5);
/// let b = from_sec_f!(0.0, on=Scale::UTC);
/// let c = from_sec_f!(1.0, on=Scale::TAI, target=Scale::UTC);
/// let d = from_sec_f!(1.0, target=Scale::UTC, on=Scale::TAI);
///
/// assert_eq!(a, deep_time::Dt::from_sec_f(5.5, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_sec_f(0.0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, deep_time::Dt::from_sec_f(1.0, Scale::TAI, Scale::UTC));
/// assert_eq!(d, c);
/// ```
#[macro_export]
macro_rules! from_sec_f {
    ($sec:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_sec_f($sec, $scale, $target)
    };
    ($sec:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_f($sec, $scale, $target)
    };
    ($sec:expr, on=$scale:expr) => {
        $crate::Dt::from_sec_f($sec, $scale, $scale)
    };
    ($sec:expr, target=$target:expr) => {
        $crate::Dt::from_sec_f($sec, $crate::Scale::TAI, $target)
    };
    ($sec:expr) => {
        $crate::Dt::from_sec_f($sec, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](../struct.Dt.html) from a floating-point day count with optional
/// scale labels.
///
/// - Equivalent to [`Dt::from_days_f`](../struct.Dt.html#method.from_days_f).
/// - Does **not** perform time-scale conversion.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// | Form | Meaning |
/// |------|---------|
/// | `from_days_f!(days)` | both scales TAI |
/// | `from_days_f!(days, on=s)` | scale and target `s` |
/// | `from_days_f!(days, target=t)` | scale TAI, target `t` |
/// | `from_days_f!(days, on=s, target=t)` | either order |
/// | `from_days_f!(days, target=t, on=s)` | either order |
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::macros::from_days_f;
///
/// let a = from_days_f!(1.25);
/// let b = from_days_f!(0.0, on=Scale::UTC);
/// let c = from_days_f!(1.0, on=Scale::TAI, target=Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::from_days_f(1.25, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_days_f(0.0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, deep_time::Dt::from_days_f(1.0, Scale::TAI, Scale::UTC));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_days_f {
    ($days:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_days_f($days, $scale, $target)
    };
    ($days:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_days_f($days, $scale, $target)
    };
    ($days:expr, on=$scale:expr) => {
        $crate::Dt::from_days_f($days, $scale, $scale)
    };
    ($days:expr, target=$target:expr) => {
        $crate::Dt::from_days_f($days, $crate::Scale::TAI, $target)
    };
    ($days:expr) => {
        $crate::Dt::from_days_f($days, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_days_f as from_days_f;

/// Builds a [`Dt`](../struct.Dt.html) from whole nanoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// - Equivalent to [`Dt::from_ns`](../struct.Dt.html#method.from_ns).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`ps!`](macro.ps.html) (or another `*_to_attos`
/// helper) instead of writing the attosecond literal by hand.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_ns!(ns)
/// from_ns!(ns, frac)
/// from_ns!(ns, on=s)
/// from_ns!(ns, target=t)
/// from_ns!(ns, on=s, target=t)
/// from_ns!(ns, target=t, on=s)
/// from_ns!(ns, frac, on=s)
/// from_ns!(ns, frac, target=t)
/// from_ns!(ns, frac, on=s, target=t)
/// from_ns!(ns, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_ns, macros::ps};
///
/// // 1.3 ns → whole nanoseconds + 300 ps remainder
/// let a = from_ns!(1);
/// let b = from_ns!(1, ps!(300));
/// let c = from_ns!(-1, ps!(-300), on=Scale::TAI);
/// let d = from_ns!(0, on=Scale::UTC);
/// let e = from_ns!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_ns!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_ns(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_ns(1, ps!(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_ns(-1, ps!(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_ns(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_ns(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_ns!(-1, ps!(-300));
/// let floor = from_ns!(-2, ps!(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_ns(-1, ps!(-300), Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_ns {
    ($ns:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale, $target)
    };
    ($ns:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale, $target)
    };
    ($ns:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale, $target)
    };
    ($ns:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale, $target)
    };
    ($ns:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale, $scale)
    };
    ($ns:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_ns($ns, $frac, $crate::Scale::TAI, $target)
    };
    ($ns:expr, on=$scale:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale, $scale)
    };
    ($ns:expr, target=$target:expr) => {
        $crate::Dt::from_ns($ns, 0, $crate::Scale::TAI, $target)
    };
    ($ns:expr, $frac:expr) => {
        $crate::Dt::from_ns($ns, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($ns:expr) => {
        $crate::Dt::from_ns($ns, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_ns as from_ns;

/// Builds a [`Dt`](../struct.Dt.html) from whole milliseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// - Equivalent to [`Dt::from_ms`](../struct.Dt.html#method.from_ms).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`us!`](macro.us.html) (or another `*_to_attos`
/// helper) instead of writing the attosecond literal by hand.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_ms!(ms)
/// from_ms!(ms, frac)
/// from_ms!(ms, on=s)
/// from_ms!(ms, target=t)
/// from_ms!(ms, on=s, target=t)
/// from_ms!(ms, target=t, on=s)
/// from_ms!(ms, frac, on=s)
/// from_ms!(ms, frac, target=t)
/// from_ms!(ms, frac, on=s, target=t)
/// from_ms!(ms, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_ms, macros::us};
///
/// // 1.3 ms → whole milliseconds + 300 µs remainder
/// let a = from_ms!(1);
/// let b = from_ms!(1, us!(300));
/// let c = from_ms!(-1, us!(-300), on=Scale::TAI);
/// let d = from_ms!(0, on=Scale::UTC);
/// let e = from_ms!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_ms!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_ms(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_ms(1, us!(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_ms(-1, us!(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_ms(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_ms(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_ms!(-1, us!(-300));
/// let floor = from_ms!(-2, us!(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_ms(-1, us!(-300), Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_ms {
    ($ms:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ms($ms, $frac, $scale, $target)
    };
    ($ms:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ms($ms, $frac, $scale, $target)
    };
    ($ms:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ms($ms, 0, $scale, $target)
    };
    ($ms:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ms($ms, 0, $scale, $target)
    };
    ($ms:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_ms($ms, $frac, $scale, $scale)
    };
    ($ms:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_ms($ms, $frac, $crate::Scale::TAI, $target)
    };
    ($ms:expr, on=$scale:expr) => {
        $crate::Dt::from_ms($ms, 0, $scale, $scale)
    };
    ($ms:expr, target=$target:expr) => {
        $crate::Dt::from_ms($ms, 0, $crate::Scale::TAI, $target)
    };
    ($ms:expr, $frac:expr) => {
        $crate::Dt::from_ms($ms, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($ms:expr) => {
        $crate::Dt::from_ms($ms, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_ms as from_ms;

/// Builds a [`Dt`](../struct.Dt.html) from whole microseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// - Equivalent to [`Dt::from_us`](../struct.Dt.html#method.from_us).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`ns!`](../macro.ns.html) (or another `*_to_attos`
/// helper) instead of writing the attosecond literal by hand.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_us!(us)
/// from_us!(us, frac)
/// from_us!(us, on=s)
/// from_us!(us, target=t)
/// from_us!(us, on=s, target=t)
/// from_us!(us, target=t, on=s)
/// from_us!(us, frac, on=s)
/// from_us!(us, frac, target=t)
/// from_us!(us, frac, on=s, target=t)
/// from_us!(us, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_us, ns};
///
/// // 1.3 µs → whole microseconds + 300 ns remainder
/// let a = from_us!(1);
/// let b = from_us!(1, ns!(300));
/// let c = from_us!(-1, ns!(-300), on=Scale::TAI);
/// let d = from_us!(0, on=Scale::UTC);
/// let e = from_us!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_us!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_us(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_us(1, ns!(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_us(-1, ns!(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_us(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_us(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_us!(-1, ns!(-300));
/// let floor = from_us!(-2, ns!(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_us(-1, ns!(-300), Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_us {
    ($us:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_us($us, $frac, $scale, $target)
    };
    ($us:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_us($us, $frac, $scale, $target)
    };
    ($us:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_us($us, 0, $scale, $target)
    };
    ($us:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_us($us, 0, $scale, $target)
    };
    ($us:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_us($us, $frac, $scale, $scale)
    };
    ($us:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_us($us, $frac, $crate::Scale::TAI, $target)
    };
    ($us:expr, on=$scale:expr) => {
        $crate::Dt::from_us($us, 0, $scale, $scale)
    };
    ($us:expr, target=$target:expr) => {
        $crate::Dt::from_us($us, 0, $crate::Scale::TAI, $target)
    };
    ($us:expr, $frac:expr) => {
        $crate::Dt::from_us($us, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($us:expr) => {
        $crate::Dt::from_us($us, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_us as from_us;

/// Builds a [`Dt`](../struct.Dt.html) from whole picoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// - Equivalent to [`Dt::from_ps`](../struct.Dt.html#method.from_ps).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`fs!`](macro.fs.html) (or another `*_to_attos`
/// helper) instead of writing the attosecond literal by hand.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_ps!(ps)
/// from_ps!(ps, frac)
/// from_ps!(ps, on=s)
/// from_ps!(ps, target=t)
/// from_ps!(ps, on=s, target=t)
/// from_ps!(ps, target=t, on=s)
/// from_ps!(ps, frac, on=s)
/// from_ps!(ps, frac, target=t)
/// from_ps!(ps, frac, on=s, target=t)
/// from_ps!(ps, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_ps, macros::fs};
///
/// // 1.3 ps → whole picoseconds + 300 fs remainder
/// let a = from_ps!(1);
/// let b = from_ps!(1, fs!(300));
/// let c = from_ps!(-1, fs!(-300), on=Scale::TAI);
/// let d = from_ps!(0, on=Scale::UTC);
/// let e = from_ps!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_ps!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_ps(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_ps(1, fs!(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_ps(-1, fs!(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_ps(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_ps(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_ps!(-1, fs!(-300));
/// let floor = from_ps!(-2, fs!(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_ps(-1, fs!(-300), Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_ps {
    ($ps:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ps($ps, $frac, $scale, $target)
    };
    ($ps:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ps($ps, $frac, $scale, $target)
    };
    ($ps:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_ps($ps, 0, $scale, $target)
    };
    ($ps:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_ps($ps, 0, $scale, $target)
    };
    ($ps:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_ps($ps, $frac, $scale, $scale)
    };
    ($ps:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_ps($ps, $frac, $crate::Scale::TAI, $target)
    };
    ($ps:expr, on=$scale:expr) => {
        $crate::Dt::from_ps($ps, 0, $scale, $scale)
    };
    ($ps:expr, target=$target:expr) => {
        $crate::Dt::from_ps($ps, 0, $crate::Scale::TAI, $target)
    };
    ($ps:expr, $frac:expr) => {
        $crate::Dt::from_ps($ps, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($ps:expr) => {
        $crate::Dt::from_ps($ps, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_ps as from_ps;

/// Builds a [`Dt`](../struct.Dt.html) from whole femtoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// - Equivalent to [`Dt::from_fs`](../struct.Dt.html#method.from_fs).
/// - Does **not** perform time-scale conversion.
///
/// The fractional remainder is already in **attoseconds** (one femtosecond is
/// 1000 attoseconds) — there is no smaller named unit macro.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` and `target` | both [`Scale::TAI`](crate::Scale::TAI) |
/// | only `on=s` | `target=s` |
/// | only `target=t` | `on=TAI` |
///
/// ## Forms
///
/// ```text
/// from_fs!(fs)
/// from_fs!(fs, frac)
/// from_fs!(fs, on=s)
/// from_fs!(fs, target=t)
/// from_fs!(fs, on=s, target=t)
/// from_fs!(fs, target=t, on=s)
/// from_fs!(fs, frac, on=s)
/// from_fs!(fs, frac, target=t)
/// from_fs!(fs, frac, on=s, target=t)
/// from_fs!(fs, frac, target=t, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, macros::from_fs};
///
/// // 1.3 fs → whole femtoseconds + 300 attoseconds remainder
/// let a = from_fs!(1);
/// let b = from_fs!(1, 300);
/// let c = from_fs!(-1, -300, on=Scale::TAI);
/// let d = from_fs!(0, on=Scale::UTC);
/// let e = from_fs!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_fs!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_fs(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_fs(1, 300, Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_fs(-1, -300, Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_fs(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_fs(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_fs!(-1, -300);
/// let floor = from_fs!(-2, 700);
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_fs(-1, -300, Scale::TAI, Scale::TAI));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_fs {
    ($fs:expr, $frac:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_fs($fs, $frac, $scale, $target)
    };
    ($fs:expr, $frac:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_fs($fs, $frac, $scale, $target)
    };
    ($fs:expr, on=$scale:expr, target=$target:expr) => {
        $crate::Dt::from_fs($fs, 0, $scale, $target)
    };
    ($fs:expr, target=$target:expr, on=$scale:expr) => {
        $crate::Dt::from_fs($fs, 0, $scale, $target)
    };
    ($fs:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_fs($fs, $frac, $scale, $scale)
    };
    ($fs:expr, $frac:expr, target=$target:expr) => {
        $crate::Dt::from_fs($fs, $frac, $crate::Scale::TAI, $target)
    };
    ($fs:expr, on=$scale:expr) => {
        $crate::Dt::from_fs($fs, 0, $scale, $scale)
    };
    ($fs:expr, target=$target:expr) => {
        $crate::Dt::from_fs($fs, 0, $crate::Scale::TAI, $target)
    };
    ($fs:expr, $frac:expr) => {
        $crate::Dt::from_fs($fs, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($fs:expr) => {
        $crate::Dt::from_fs($fs, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_fs as from_fs;

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a Julian Date (whole days plus
/// optional attosecond remainder).
///
/// Equivalent to [`Dt::from_jd`](../struct.Dt.html#method.from_jd).
///
/// When an `on` arg is provided and it's not [`Scale::TAI`](crate::Scale::TAI)
/// then a time scale conversion is performed equivalent to `on` ->
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// There is no `target=` on this macro — the returned [`Dt`](../struct.Dt.html)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_jd`](../struct.Dt.html#method.from_jd) does. Chain
/// [`.target(…)`](../struct.Dt.html#method.target) if needed.
///
/// The fractional remainder is in **attoseconds** — use a `*_to_attos` helper
/// (e.g. a day fraction built from [`days_f!`](../macro.days_f.html))
/// instead of hand-counting zeros when convenient.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` | [`Scale::TAI`](crate::Scale::TAI) |
///
/// ## Forms
///
/// ```text
/// from_jd!(jd_days)
/// from_jd!(jd_days, frac)
/// from_jd!(jd_days, on=s)
/// from_jd!(jd_days, frac, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, days_f, macros::from_jd};
///
/// // 2_460_782.25
/// let a = from_jd!(2_460_782);
/// let b = from_jd!(2_460_782, days_f!(0.25));
/// let c = from_jd!(2_460_782, days_f!(0.25), on=Scale::TAI);
/// let d = from_jd!(2_460_782, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_jd(2_460_782, 0, Scale::TAI));
/// assert_eq!(b, Dt::from_jd(2_460_782, days_f!(0.25), Scale::TAI));
/// assert_eq!(c, b);
/// assert_eq!(d, Dt::from_jd(2_460_782, 0, Scale::UTC));
///
/// // -1_000.25 (signed remainder)
/// let neg = from_jd!(-1_000, -days_f!(0.25));
/// assert_eq!(neg, Dt::from_jd(-1_000, -days_f!(0.25), Scale::TAI));
/// assert_eq!(neg.to_jd(), (-1_000, -days_f!(0.25)));
///
/// // or with floor style
/// assert_eq!(neg, from_jd!(-1_001, days_f!(0.75)));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_jd {
    // `on=` arms before bare `$frac:expr` so `on=…` is not taken as an expr.
    ($jd_days:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_jd($jd_days, $frac, $scale)
    };
    ($jd_days:expr, on=$scale:expr) => {
        $crate::Dt::from_jd($jd_days, 0, $scale)
    };
    ($jd_days:expr, $frac:expr) => {
        $crate::Dt::from_jd($jd_days, $frac, $crate::Scale::TAI)
    };
    ($jd_days:expr) => {
        $crate::Dt::from_jd($jd_days, 0, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_jd as from_jd;

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a floating-point Julian Date.
///
/// Equivalent to [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f).
///
/// When an `on` arg is provided and it's not [`Scale::TAI`](crate::Scale::TAI)
/// then a time scale conversion is performed equivalent to `on` ->
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// There is no `target=` on this macro — the returned [`Dt`](../struct.Dt.html)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_jd_f`](../struct.Dt.html#method.from_jd_f) does. Chain
/// [`.target(…)`](../struct.Dt.html#method.target) if needed.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | `on` | [`Scale::TAI`](crate::Scale::TAI) |
///
/// ## Forms
///
/// ```text
/// from_jd_f!(jd)
/// from_jd_f!(jd, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, days_f, macros::from_jd_f};
///
/// // 2_460_782.25
/// let a = from_jd_f!(2_460_782.25);
/// let b = from_jd_f!(2_460_782.25, on=Scale::TAI);
/// let c = from_jd_f!(2_460_782.0, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_jd_f(2_460_782.25, Scale::TAI));
/// assert_eq!(b, a);
/// assert_eq!(c, Dt::from_jd_f(2_460_782.0, Scale::UTC));
/// assert_eq!(a.to_jd(), (2_460_782, days_f!(0.25)));
///
/// // -1_000.25 (signed remainder)
/// let neg = from_jd_f!(-1_000.25);
/// assert_eq!(neg, Dt::from_jd_f(-1_000.25, Scale::TAI));
/// assert_eq!(neg.to_jd(), (-1_000, -days_f!(0.25)));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_jd_f {
    ($jd:expr, on=$scale:expr) => {
        $crate::Dt::from_jd_f($jd, $scale)
    };
    ($jd:expr) => {
        $crate::Dt::from_jd_f($jd, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_jd_f as from_jd_f;

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a Modified Julian Date (whole days
/// plus optional attosecond remainder).
///
/// Equivalent to [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd).
///
/// When an `on` arg is provided and it's not [`Scale::TAI`](crate::Scale::TAI)
/// then a time scale conversion is performed equivalent to `on` ->
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// There is no `target=` on this macro — the returned [`Dt`](../struct.Dt.html)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_mjd`](../struct.Dt.html#method.from_mjd) does. Chain
/// [`.target(…)`](../struct.Dt.html#method.target) if needed.
///
/// MJD and JD relate by `JD = MJD + 2_400_000.5`.
///
/// The fractional remainder is in **attoseconds** — use a `*_to_attos` helper
/// (e.g. a day fraction built from [`days_f!`](../macro.days_f.html))
/// instead of hand-counting zeros when convenient.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | fraction | `0` |
/// | `on` | [`Scale::TAI`](crate::Scale::TAI) |
///
/// ## Forms
///
/// ```text
/// from_mjd!(mjd_days)
/// from_mjd!(mjd_days, frac)
/// from_mjd!(mjd_days, on=s)
/// from_mjd!(mjd_days, frac, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, days_f, macros::from_mjd};
///
/// // J2000.0 → MJD 51_544.5
/// let a = from_mjd!(51_544);
/// let b = from_mjd!(51_544, days_f!(0.5));
/// let c = from_mjd!(51_544, days_f!(0.5), on=Scale::TAI);
/// let d = from_mjd!(51_544, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_mjd(51_544, 0, Scale::TAI));
/// assert_eq!(b, Dt::from_mjd(51_544, days_f!(0.5), Scale::TAI));
/// assert_eq!(c, b);
/// assert_eq!(d, Dt::from_mjd(51_544, 0, Scale::UTC));
/// assert_eq!(b.to_mjd(), (51_544, days_f!(0.5)));
///
/// // -1_000.25 (signed truncating remainder)
/// let neg = from_mjd!(-1_000, -days_f!(0.25));
/// assert_eq!(neg, Dt::from_mjd(-1_000, -days_f!(0.25), Scale::TAI));
/// assert_eq!(neg.to_mjd(), (-1_000, -days_f!(0.25)));
///
/// // or with floor style
/// assert_eq!(neg, from_mjd!(-1_001, days_f!(0.75)));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_mjd {
    // `on=` arms before bare `$frac:expr` so `on=…` is not taken as an expr.
    ($mjd_days:expr, $frac:expr, on=$scale:expr) => {
        $crate::Dt::from_mjd($mjd_days, $frac, $scale)
    };
    ($mjd_days:expr, on=$scale:expr) => {
        $crate::Dt::from_mjd($mjd_days, 0, $scale)
    };
    ($mjd_days:expr, $frac:expr) => {
        $crate::Dt::from_mjd($mjd_days, $frac, $crate::Scale::TAI)
    };
    ($mjd_days:expr) => {
        $crate::Dt::from_mjd($mjd_days, 0, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_mjd as from_mjd;

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a floating-point Modified Julian Date.
///
/// Equivalent to [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f).
///
/// When an `on` arg is provided and it's not [`Scale::TAI`](crate::Scale::TAI)
/// then a time scale conversion is performed equivalent to `on` ->
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// There is no `target=` on this macro — the returned [`Dt`](../struct.Dt.html)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_mjd_f`](../struct.Dt.html#method.from_mjd_f) does. Chain
/// [`.target(…)`](../struct.Dt.html#method.target) if needed.
///
/// MJD and JD relate by `JD = MJD + 2_400_000.5`.
///
/// ## Defaults
///
/// | Omitted | Default |
/// |---------|---------|
/// | `on` | [`Scale::TAI`](crate::Scale::TAI) |
///
/// ## Forms
///
/// ```text
/// from_mjd_f!(mjd)
/// from_mjd_f!(mjd, on=s)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::{Dt, Scale, days_f, macros::from_mjd_f};
///
/// // 60_961.25
/// let a = from_mjd_f!(60_961.25);
/// let b = from_mjd_f!(60_961.25, on=Scale::TAI);
/// let c = from_mjd_f!(60_961.0, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_mjd_f(60_961.25, Scale::TAI));
/// assert_eq!(b, a);
/// assert_eq!(c, Dt::from_mjd_f(60_961.0, Scale::UTC));
/// assert_eq!(a.to_mjd_floor(), (60_961, days_f!(0.25)));
///
/// // -1_000.25 as -1_001 + 0.75 day
/// let neg = from_mjd_f!(-1_000.25);
/// assert_eq!(neg, Dt::from_mjd_f(-1_000.25, Scale::TAI));
/// assert_eq!(neg.to_mjd_floor(), (-1_001, days_f!(0.75)));
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __from_mjd_f {
    ($mjd:expr, on=$scale:expr) => {
        $crate::Dt::from_mjd_f($mjd, $scale)
    };
    ($mjd:expr) => {
        $crate::Dt::from_mjd_f($mjd, $crate::Scale::TAI)
    };
}

#[doc(inline)]
pub use __from_mjd_f as from_mjd_f;

/// Builds a [`Dt`](../struct.Dt.html) from a Gregorian calendar date and optional time.
///
/// Equivalent to [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd).
///
/// Date fields are positional (`y`, `m`, `d`). Time is optional: put a
/// **semicolon after the day**, then hour (required if `;` is present), then
/// optional minute, second, and attoseconds. An optional `on=` civil scale may
/// follow.
///
/// A time scale conversion is performed from the `on` arg ->
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// When no `on` arg is used the time scale is assumed to be
/// [`Scale::UTC`](crate::Scale::UTC).
///
/// | Omitted field | Default |
/// |---------------|---------|
/// | month | `1` |
/// | day | `1` |
/// | time (no `;`) | `0, 0, 0, 0` |
/// | minute / second / attos after `; h` | `0` |
/// | `on` | [`Scale::UTC`](crate::Scale::UTC) |
///
/// The resulting [`Dt`](../struct.Dt.html)'s `target` field is set from that civil
/// scale (the `on=` value, or UTC when omitted), as
/// [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd) does. There is no `target=` on
/// this macro — chain [`.target(…)`](../struct.Dt.html#method.target) if needed.
///
/// ## Forms
///
/// ```text
/// from_ymd!(y)
/// from_ymd!(y, m)
/// from_ymd!(y, m, d)
/// from_ymd!(y, m, d, on=Scale::TAI)
/// from_ymd!(y, m, d; h)
/// from_ymd!(y, m, d; h, min)
/// from_ymd!(y, m, d; h, min, sec)
/// from_ymd!(y, m, d; h, min, sec, attos)
/// from_ymd!(y, m, d; h, min, sec, attos, on=Scale::UTC)
/// ```
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::from_ymd;
///
/// assert_eq!(
///     from_ymd!(1970),
///     deep_time::Dt::UNIX_EPOCH,
/// );
/// assert_eq!(
///     from_ymd!(2026, 6, 16),
///     deep_time::Dt::from_ymd(2026, 6, 16, Scale::UTC, 0, 0, 0, 0),
/// );
/// assert_eq!(
///     from_ymd!(2000, 1, 1; 12, on=Scale::TAI),
///     deep_time::Dt::ZERO,
/// );
/// assert_eq!(
///     from_ymd!(2000, 1, 1; 12, 0, 0, 123_456_789, on=Scale::UTC),
///     deep_time::Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 123_456_789),
/// );
///
/// // different target field after construction
/// let _ = from_ymd!(2020, 1, 1, on=Scale::UTC).target(Scale::TAI);
/// ```
#[macro_export]
macro_rules! from_ymd {
    // Time section: `d; h …` — `on=` arms before bare so `on=…` is not an `:expr`.
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr, $s:expr, $attos:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, $s, $attos)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr, $s:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, $s, 0)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, 0, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr, $s:expr, $attos:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, $s, $attos)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr, $s:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, $s, 0)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr, $min:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr; $h:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, 0, 0, 0)
    };

    // Date only (+ optional on=)
    ($y:expr, $m:expr, $d:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, $m, 1, $scale, 0, 0, 0, 0)
    };
    ($y:expr, on=$scale:expr) => {
        $crate::Dt::from_ymd($y, 1, 1, $scale, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr) => {
        $crate::Dt::from_ymd($y, $m, 1, $crate::Scale::UTC, 0, 0, 0, 0)
    };
    ($y:expr) => {
        $crate::Dt::from_ymd($y, 1, 1, $crate::Scale::UTC, 0, 0, 0, 0)
    };
}

#[doc(inline)]
pub use crate::{days_f, dt, from_sec_f, from_ymd, ms, ns};
