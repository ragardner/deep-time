//! Crate-root convenience macros (`dt!`, `from_sec!`, `from_sec_f!`, `from_ns!`,
//! `from_ms!`, `from_jd!`, `from_mjd!`, `from_ymd!`, …).
//!
//! Optional scale labels use Python-style keyword arguments on **count** macros
//! (`dt!`, `from_sec!`, `from_sec_f!`, `from_ns!`, `from_ms!`, …):
//!
//! - `on=<scale>` — stored as the [`Dt`](crate::Dt)'s `scale` field (and as
//!   `target` when `target=` is omitted).
//! - `target=<scale>` — stored as the `target` field; `on` defaults to
//!   [`Scale::TAI`](crate::Scale::TAI).
//!
//! Either keyword may appear alone or together, in either order.
//!
//! [`from_jd!`](crate::from_jd), [`from_mjd!`](crate::from_mjd), and
//! [`from_ymd!`](crate::from_ymd) only take a single `on=` scale (see each
//! macro); use [`.target(…)`](crate::Dt::target) afterward if the
//! `target` field should differ.

/// Builds a [`Dt`](crate::Dt) from total attoseconds with optional scale labels.
///
/// Sugar for [`Dt::new`](crate::Dt::new). Does **not** perform time-scale
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

/// Builds a [`Dt`](crate::Dt) from whole seconds and an optional signed
/// sub-second remainder (attoseconds).
///
/// Sugar for [`Dt::from_sec_and_frac`](crate::Dt::from_sec_and_frac). Does
/// **not** perform time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`Dt::ms_to_attos`](crate::Dt::ms_to_attos) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_sec};
///
/// // 1.3 s → whole seconds + 300 ms remainder
/// let a = from_sec!(1);
/// let b = from_sec!(1, Dt::ms_to_attos(300));
/// let c = from_sec!(-1, Dt::ms_to_attos(-300), on=Scale::TAI);
/// let d = from_sec!(0, on=Scale::UTC);
/// let e = from_sec!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_sec!(1, target=Scale::UTC, on=Scale::TAI);
///
/// assert_eq!(a, Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_sec_and_frac(1, Dt::ms_to_attos(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_sec_and_frac(-1, Dt::ms_to_attos(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_sec_and_frac(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_sec!(-1, Dt::ms_to_attos(-300));
/// let floor = from_sec!(-2, Dt::ms_to_attos(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_sec_and_frac(-1, Dt::ms_to_attos(-300), Scale::TAI, Scale::TAI));
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

/// Builds a [`Dt`](crate::Dt) from a floating-point seconds count with optional
/// scale labels.
///
/// Sugar for [`Dt::from_sec_f`](crate::Dt::from_sec_f). Does **not** perform
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

/// Builds a [`Dt`](crate::Dt) from whole nanoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_ns`](crate::Dt::from_ns). Does **not** perform
/// time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`Dt::ps_to_attos`](crate::Dt::ps_to_attos) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_ns};
///
/// // 1.3 ns → whole nanoseconds + 300 ps remainder
/// let a = from_ns!(1);
/// let b = from_ns!(1, Dt::ps_to_attos(300));
/// let c = from_ns!(-1, Dt::ps_to_attos(-300), on=Scale::TAI);
/// let d = from_ns!(0, on=Scale::UTC);
/// let e = from_ns!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_ns!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_ns(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_ns(1, Dt::ps_to_attos(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_ns(-1, Dt::ps_to_attos(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_ns(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_ns(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_ns!(-1, Dt::ps_to_attos(-300));
/// let floor = from_ns!(-2, Dt::ps_to_attos(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_ns(-1, Dt::ps_to_attos(-300), Scale::TAI, Scale::TAI));
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

/// Builds a [`Dt`](crate::Dt) from whole milliseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// Sugar for [`Dt::from_ms`](crate::Dt::from_ms). Does **not** perform
/// time-scale conversion.
///
/// The fractional remainder is in **attoseconds** — use
/// [`Dt::us_to_attos`](crate::Dt::us_to_attos) (or another `*_to_attos`
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
/// use deep_time::{Dt, Scale, from_ms};
///
/// // 1.3 ms → whole milliseconds + 300 µs remainder
/// let a = from_ms!(1);
/// let b = from_ms!(1, Dt::us_to_attos(300));
/// let c = from_ms!(-1, Dt::us_to_attos(-300), on=Scale::TAI);
/// let d = from_ms!(0, on=Scale::UTC);
/// let e = from_ms!(1, on=Scale::TAI, target=Scale::UTC);
/// let f = from_ms!(1, target=Scale::UTC);
///
/// assert_eq!(a, Dt::from_ms(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, Dt::from_ms(1, Dt::us_to_attos(300), Scale::TAI, Scale::TAI));
/// assert_eq!(c, Dt::from_ms(-1, Dt::us_to_attos(-300), Scale::TAI, Scale::TAI));
/// assert_eq!(d, Dt::from_ms(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, Dt::from_ms(1, 0, Scale::TAI, Scale::UTC));
/// assert_eq!(f, e);
///
/// let signed = from_ms!(-1, Dt::us_to_attos(-300));
/// let floor = from_ms!(-2, Dt::us_to_attos(700));
/// assert_eq!(signed, floor);
/// assert_eq!(signed, Dt::from_ms(-1, Dt::us_to_attos(-300), Scale::TAI, Scale::TAI));
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

/// Builds a **TAI** [`Dt`](crate::Dt) from a Julian Date (whole days plus
/// optional attosecond remainder).
///
/// Sugar for [`Dt::from_jd`](crate::Dt::from_jd).
///
/// Converts from the `on` scale to `TAI` (e.g. `on=Scale::UTC` applies leap
/// seconds). Use `on=Scale::TAI` (the default) to skip a scale conversion.
///
/// There is no `target=` on this macro — the returned [`Dt`](crate::Dt)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_jd`](crate::Dt::from_jd) does. Chain
/// [`.target(…)`](crate::Dt::target) if needed.
///
/// The fractional remainder is in **attoseconds** — use a `*_to_attos` helper
/// (e.g. a day fraction built from [`ATTOS_PER_DAY`](crate::consts::ATTOS_PER_DAY))
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
/// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY, from_jd};
///
/// // 2_460_782.25
/// let a = from_jd!(2_460_782);
/// let b = from_jd!(2_460_782, ATTOS_PER_DAY / 4);
/// let c = from_jd!(2_460_782, ATTOS_PER_DAY / 4, on=Scale::TAI);
/// let d = from_jd!(2_460_782, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_jd(2_460_782, 0, Scale::TAI));
/// assert_eq!(b, Dt::from_jd(2_460_782, ATTOS_PER_DAY / 4, Scale::TAI));
/// assert_eq!(c, b);
/// assert_eq!(d, Dt::from_jd(2_460_782, 0, Scale::UTC));
///
/// // -1_000.25 (signed remainder)
/// let neg = from_jd!(-1_000, -ATTOS_PER_DAY / 4);
/// assert_eq!(neg, Dt::from_jd(-1_000, -ATTOS_PER_DAY / 4, Scale::TAI));
/// assert_eq!(neg.to_jd(), (-1_000, -ATTOS_PER_DAY / 4));
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

/// Builds a **TAI** [`Dt`](crate::Dt) from a Modified Julian Date (whole days
/// plus optional attosecond remainder).
///
/// Sugar for [`Dt::from_mjd`](crate::Dt::from_mjd).
///
/// Converts from the `on` scale to `TAI` (e.g. `on=Scale::UTC` applies leap
/// seconds). Use `on=Scale::TAI` (the default) to skip a scale conversion.
///
/// There is no `target=` on this macro — the returned [`Dt`](crate::Dt)'s
/// `target` is set from `on` (or TAI when omitted), as
/// [`from_mjd`](crate::Dt::from_mjd) does. Chain
/// [`.target(…)`](crate::Dt::target) if needed.
///
/// MJD and JD relate by `JD = MJD + 2_400_000.5`.
///
/// The fractional remainder is in **attoseconds** — use a `*_to_attos` helper
/// (e.g. a day fraction built from [`ATTOS_PER_DAY`](crate::consts::ATTOS_PER_DAY))
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
/// use deep_time::{Dt, Scale, consts::ATTOS_PER_DAY, from_mjd};
///
/// // 60_961.25
/// let a = from_mjd!(60_961);
/// let b = from_mjd!(60_961, ATTOS_PER_DAY / 4);
/// let c = from_mjd!(60_961, ATTOS_PER_DAY / 4, on=Scale::TAI);
/// let d = from_mjd!(60_961, on=Scale::UTC);
///
/// assert_eq!(a, Dt::from_mjd(60_961, 0, Scale::TAI));
/// assert_eq!(b, Dt::from_mjd(60_961, ATTOS_PER_DAY / 4, Scale::TAI));
/// assert_eq!(c, b);
/// assert_eq!(d, Dt::from_mjd(60_961, 0, Scale::UTC));
///
/// // -1_000.25 as -1_001 + 0.75 day
/// let neg = from_mjd!(-1_001, 3 * ATTOS_PER_DAY / 4);
/// assert_eq!(neg, Dt::from_mjd(-1_001, 3 * ATTOS_PER_DAY / 4, Scale::TAI));
/// assert_eq!(neg.to_mjd(), (-1_001, 3 * ATTOS_PER_DAY / 4));
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

/// Builds a [`Dt`](crate::Dt) from a Gregorian calendar date and optional time.
///
/// Sugar for [`Dt::from_ymd`](crate::Dt::from_ymd).
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
/// The resulting [`Dt`](crate::Dt)'s `target` field is set from that civil
/// scale (the `on=` value, or UTC when omitted), as
/// [`from_ymd`](crate::Dt::from_ymd) does. There is no `target=` on
/// this macro — chain [`.target(…)`](crate::Dt::target) if needed.
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
