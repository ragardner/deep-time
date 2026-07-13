//! Convenience macros

/// Converts whole femtoseconds (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::fs_to_attos`](../struct.Dt.html#method.fs_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, fs};
///
/// assert_eq!(fs!(1), Dt::fs_to_attos(1));
/// ```
#[macro_export]
macro_rules! fs {
    ($x:expr) => {
        $crate::Dt::fs_to_attos($x)
    };
}

/// Converts whole picoseconds (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::ps_to_attos`](../struct.Dt.html#method.ps_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, ps};
///
/// assert_eq!(ps!(1), Dt::ps_to_attos(1));
/// ```
#[macro_export]
macro_rules! ps {
    ($x:expr) => {
        $crate::Dt::ps_to_attos($x)
    };
}

/// Converts whole nanoseconds (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::ns_to_attos`](../struct.Dt.html#method.ns_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
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
/// Sugar for [`Dt::us_to_attos`](../struct.Dt.html#method.us_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, us};
///
/// assert_eq!(us!(1), Dt::us_to_attos(1));
/// ```
#[macro_export]
macro_rules! us {
    ($x:expr) => {
        $crate::Dt::us_to_attos($x)
    };
}

/// Converts whole milliseconds (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::ms_to_attos`](../struct.Dt.html#method.ms_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
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
/// Sugar for [`Dt::sec_to_attos`](../struct.Dt.html#method.sec_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, sec};
///
/// assert_eq!(sec!(1), Dt::sec_to_attos(1));
/// ```
#[macro_export]
macro_rules! sec {
    ($x:expr) => {
        $crate::Dt::sec_to_attos($x)
    };
}

/// Converts a floating-point second count ([`Real`](crate::Real)) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::sec_f_to_attos`](../struct.Dt.html#method.sec_f_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only converts the second count to attoseconds via the
/// high-precision IEEE 754 path.
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, sec_f};
///
/// assert_eq!(sec_f!(1.5), Dt::sec_f_to_attos(1.5));
/// ```
#[macro_export]
macro_rules! sec_f {
    ($x:expr) => {
        $crate::Dt::sec_f_to_attos($x)
    };
}

/// Converts whole minutes (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::mins_to_attos`](../struct.Dt.html#method.mins_to_attos). Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, mins};
///
/// assert_eq!(mins!(1), Dt::mins_to_attos(1));
/// ```
#[macro_export]
macro_rules! mins {
    ($x:expr) => {
        $crate::Dt::mins_to_attos($x)
    };
}

/// Converts whole hours (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::hours_to_attos`](../struct.Dt.html#method.hours_to_attos).
/// Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the
/// attoseconds-per-unit constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, hours};
///
/// assert_eq!(hours!(1), Dt::hours_to_attos(1));
/// ```
#[macro_export]
macro_rules! hours {
    ($x:expr) => {
        $crate::Dt::hours_to_attos($x)
    };
}

/// Converts whole days (`i128`) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::days_to_attos`](../struct.Dt.html#method.days_to_attos). Does **not**
/// build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the attoseconds-per-unit
/// constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, days};
///
/// assert_eq!(days!(1), Dt::days_to_attos(1));
/// ```
#[macro_export]
macro_rules! days {
    ($x:expr) => {
        $crate::Dt::days_to_attos($x)
    };
}

/// Converts a floating-point day count ([`Real`](crate::Real)) to total attoseconds (`i128`).
///
/// Sugar for [`Dt::days_f_to_attos`](../struct.Dt.html#method.days_f_to_attos). Does **not**
/// build a
/// [`Dt`](../struct.Dt.html) — only converts the day count to attoseconds.
///
/// ## Example
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
/// Sugar for [`Dt::weeks_to_attos`](../struct.Dt.html#method.weeks_to_attos).
/// Does **not** build a
/// [`Dt`](../struct.Dt.html) — only multiplies the unit count by the
/// attoseconds-per-unit constant (saturating).
///
/// ## Example
///
/// ```rust
/// use deep_time::{Dt, weeks};
///
/// assert_eq!(weeks!(1), Dt::weeks_to_attos(1));
/// ```
#[macro_export]
macro_rules! weeks {
    ($x:expr) => {
        $crate::Dt::weeks_to_attos($x)
    };
}

/// Builds a [`Dt`](../struct.Dt.html) from total attoseconds with optional scale labels.
///
/// Sugar for [`Dt::new`](../struct.Dt.html#method.new). Does **not** perform time-scale
/// conversion.
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
/// Sugar for [`Dt::from_sec_and_frac`](../struct.Dt.html#method.from_sec_and_frac).
/// Does **not** perform time-scale conversion.
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
/// use deep_time::{Dt, Scale, from_sec, ms};
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
/// let floor = from_sec!(-2, ms!(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_sec_and_frac(-1, ms!(-300), Scale::TAI, Scale::TAI));
/// ```
#[macro_export]
macro_rules! from_sec {
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

/// Builds a [`Dt`](../struct.Dt.html) from a floating-point seconds count with optional
/// scale labels.
///
/// Sugar for [`Dt::from_sec_f`](../struct.Dt.html#method.from_sec_f). Does **not** perform
/// time-scale conversion.
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
/// Sugar for [`Dt::from_days_f`](../struct.Dt.html#method.from_days_f). Does **not**
/// perform time-scale conversion.
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
/// use deep_time::from_days_f;
///
/// let a = from_days_f!(1.25);
/// let b = from_days_f!(0.0, on=Scale::UTC);
/// let c = from_days_f!(1.0, on=Scale::TAI, target=Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::from_days_f(1.25, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_days_f(0.0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, deep_time::Dt::from_days_f(1.0, Scale::TAI, Scale::UTC));
/// ```
#[macro_export]
macro_rules! from_days_f {
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

/// Builds a [`Dt`](../struct.Dt.html) from whole nanoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_ns`](../struct.Dt.html#method.from_ns). Does **not** perform
/// time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`ps!`](../macro.ps.html) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_ns, ps};
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
#[macro_export]
macro_rules! from_ns {
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

/// Builds a [`Dt`](../struct.Dt.html) from whole milliseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_ms`](../struct.Dt.html#method.from_ms). Does **not** perform
/// time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`us!`](../macro.us.html) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_ms, us};
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
#[macro_export]
macro_rules! from_ms {
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

/// Builds a [`Dt`](../struct.Dt.html) from whole microseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_us`](../struct.Dt.html#method.from_us). Does **not** perform
/// time-scale conversion.
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
/// use deep_time::{Dt, Scale, from_us, ns};
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
#[macro_export]
macro_rules! from_us {
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

/// Builds a [`Dt`](../struct.Dt.html) from whole picoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_ps`](../struct.Dt.html#method.from_ps). Does **not** perform
/// time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`fs!`](../macro.fs.html) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_ps, fs};
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
#[macro_export]
macro_rules! from_ps {
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

/// Builds a [`Dt`](../struct.Dt.html) from whole femtoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_fs`](../struct.Dt.html#method.from_fs). Does **not** perform
/// time-scale conversion.
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
/// use deep_time::{Dt, Scale, from_fs};
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
#[macro_export]
macro_rules! from_fs {
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

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a Julian Date (whole days plus
/// optional attosecond remainder).
///
/// Sugar for [`Dt::from_jd`](../struct.Dt.html#method.from_jd).
///
/// Converts from the `on` scale to `TAI` when `on` is not TAI (e.g.
/// `on=Scale::UTC` applies leap seconds). `on` defaults to
/// [`Scale::TAI`](crate::Scale::TAI), omit it to skip conversion.
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
/// use deep_time::{Dt, Scale, days_f, from_jd};
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
#[macro_export]
macro_rules! from_jd {
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

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a floating-point Julian Date.
///
/// Sugar for [`Dt::from_jd_f`](../struct.Dt.html#method.from_jd_f).
///
/// Converts from the `on` scale to `TAI` when `on` is not TAI (e.g.
/// `on=Scale::UTC` applies leap seconds). `on` defaults to
/// [`Scale::TAI`](crate::Scale::TAI), omit it to skip conversion.
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
/// use deep_time::{Dt, Scale, days_f, from_jd_f};
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
#[macro_export]
macro_rules! from_jd_f {
    ($jd:expr, on=$scale:expr) => {
        $crate::Dt::from_jd_f($jd, $scale)
    };
    ($jd:expr) => {
        $crate::Dt::from_jd_f($jd, $crate::Scale::TAI)
    };
}

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a Modified Julian Date (whole days
/// plus optional attosecond remainder).
///
/// Sugar for [`Dt::from_mjd`](../struct.Dt.html#method.from_mjd).
///
/// Converts from the `on` scale to `TAI` when `on` is not TAI (e.g.
/// `on=Scale::UTC` applies leap seconds). `on` defaults to
/// [`Scale::TAI`](crate::Scale::TAI), omit it to skip conversion.
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
/// use deep_time::{Dt, Scale, days_f, from_mjd};
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
/// ```
#[macro_export]
macro_rules! from_mjd {
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

/// Builds a **TAI** [`Dt`](../struct.Dt.html) from a floating-point Modified Julian Date.
///
/// Sugar for [`Dt::from_mjd_f`](../struct.Dt.html#method.from_mjd_f).
///
/// Converts from the `on` scale to `TAI` when `on` is not TAI (e.g.
/// `on=Scale::UTC` applies leap seconds). `on` defaults to
/// [`Scale::TAI`](crate::Scale::TAI), omit it to skip conversion.
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
/// use deep_time::{Dt, Scale, days_f, from_mjd_f};
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
#[macro_export]
macro_rules! from_mjd_f {
    ($mjd:expr, on=$scale:expr) => {
        $crate::Dt::from_mjd_f($mjd, $scale)
    };
    ($mjd:expr) => {
        $crate::Dt::from_mjd_f($mjd, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](../struct.Dt.html) from a Gregorian calendar date and optional time.
///
/// Sugar for [`Dt::from_ymd`](../struct.Dt.html#method.from_ymd).
///
/// Date fields are positional (`y`, `m`, `d`). Time is optional: put a
/// **semicolon after the day**, then hour (required if `;` is present), then
/// optional minute, second, and attoseconds. An optional `on=` civil scale may
/// follow.
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
