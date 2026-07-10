//! Crate-root convenience macros (`dt!`, `from_sec!`, `from_sec_f!`, `from_ns!`, `from_ymd!`, …).

/// Builds a [`Dt`](crate::Dt) from total attoseconds with optional scale labels.
///
/// This is sugar for [`Dt::new`](crate::Dt::new). When scale is omitted, both
/// `scale` and `target` are [`Scale::TAI`](crate::Scale::TAI). With `on scale`
/// only, `target` matches `scale`. A different `target` is set with a trailing
/// `; target`.
///
/// Does **not** perform any time scale conversions.
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `dt!(attos)` | `Dt::new(attos, Scale::TAI, Scale::TAI)` |
/// | `dt!(attos, on scale)` | `Dt::new(attos, scale, scale)` |
/// | `dt!(attos, on scale; target)` | `Dt::new(attos, scale, target)` |
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::dt;
///
/// let a = dt!(1_000_000_000_000_000_000);
/// let b = dt!(0, on Scale::UTC);
/// let c = dt!(0, on Scale::TAI; Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::new(1_000_000_000_000_000_000, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::new(0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, deep_time::Dt::new(0, Scale::TAI, Scale::UTC));
/// ```
#[macro_export]
macro_rules! dt {
    ($attos:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::new($attos, $scale, $target)
    };
    ($attos:expr, on $scale:expr) => {
        $crate::Dt::new($attos, $scale, $scale)
    };
    ($attos:expr) => {
        $crate::Dt::new($attos, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](crate::Dt) from whole seconds and an optional signed
/// sub-second remainder (attoseconds).
///
/// This is sugar for [`Dt::from_sec_and_frac`](crate::Dt::from_sec_and_frac).
/// When the fraction is omitted it is `0`. When scale is omitted, both `scale`
/// and `target` are [`Scale::TAI`](crate::Scale::TAI). With `on scale` only,
/// `target` matches `scale`. A different `target` is set with a trailing
/// `; target`.
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `from_sec!(sec)` | `Dt::from_sec_and_frac(sec, 0, Scale::TAI, Scale::TAI)` |
/// | `from_sec!(sec, frac)` | `Dt::from_sec_and_frac(sec, frac, Scale::TAI, Scale::TAI)` |
/// | `from_sec!(sec, on scale)` | `Dt::from_sec_and_frac(sec, 0, scale, scale)` |
/// | `from_sec!(sec, frac, on scale)` | `Dt::from_sec_and_frac(sec, frac, scale, scale)` |
/// | `from_sec!(sec, on scale; target)` | `Dt::from_sec_and_frac(sec, 0, scale, target)` |
/// | `from_sec!(sec, frac, on scale; target)` | `Dt::from_sec_and_frac(sec, frac, scale, target)` |
///
/// The `on` keyword must be preceded by a comma. Optional `target` follows a
/// semicolon after the scale (same rules as `from_ns!`).
///
/// `sec` and `frac` are the signed whole/remainder split (e.g. `-1.3` s is
/// `sec = -1`, `frac = -300_000_000_000_000_000`). Floor-style pairs with a
/// non-negative remainder also work. No time-scale conversion is performed.
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::from_sec;
///
/// let a = from_sec!(1);
/// let b = from_sec!(1, 300_000_000_000_000_000);
/// let c = from_sec!(-1, -300_000_000_000_000_000, on Scale::TAI);
/// let d = from_sec!(0, on Scale::UTC);
/// let e = from_sec!(1, on Scale::TAI; Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_sec_and_frac(1, 300_000_000_000_000_000, Scale::TAI, Scale::TAI));
/// assert_eq!(c, deep_time::Dt::from_sec_and_frac(-1, -300_000_000_000_000_000, Scale::TAI, Scale::TAI));
/// assert_eq!(d, deep_time::Dt::from_sec_and_frac(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, deep_time::Dt::from_sec_and_frac(1, 0, Scale::TAI, Scale::UTC));
///
/// // -1.3 s: signed (decimal) split vs floor split — same instant
/// let signed = from_sec!(-1, -300_000_000_000_000_000);
/// let floor = from_sec!(-2, 700_000_000_000_000_000);
/// assert_eq!(signed, floor);
/// assert_eq!(signed, deep_time::dt!(-1_300_000_000_000_000_000));
/// ```
#[macro_export]
macro_rules! from_sec {
    ($sec:expr, $frac:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale, $target)
    };
    ($sec:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale, $target)
    };
    ($sec:expr, $frac:expr, on $scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale, $scale)
    };
    ($sec:expr, on $scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale, $scale)
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
/// This is sugar for [`Dt::from_sec_f`](crate::Dt::from_sec_f). When scale is
/// omitted, both `scale` and `target` are [`Scale::TAI`](crate::Scale::TAI).
/// With `on scale` only, `target` matches `scale`. A different `target` is set
/// with a trailing `; target`.
///
/// Does **not** perform any time scale conversions.
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `from_sec_f!(sec)` | `Dt::from_sec_f(sec, Scale::TAI, Scale::TAI)` |
/// | `from_sec_f!(sec, on scale)` | `Dt::from_sec_f(sec, scale, scale)` |
/// | `from_sec_f!(sec, on scale; target)` | `Dt::from_sec_f(sec, scale, target)` |
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::from_sec_f;
///
/// let a = from_sec_f!(5.5);
/// let b = from_sec_f!(0.0, on Scale::UTC);
/// let c = from_sec_f!(1.0, on Scale::TAI; Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::from_sec_f(5.5, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_sec_f(0.0, Scale::UTC, Scale::UTC));
/// assert_eq!(c, deep_time::Dt::from_sec_f(1.0, Scale::TAI, Scale::UTC));
/// ```
#[macro_export]
macro_rules! from_sec_f {
    ($sec:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::from_sec_f($sec, $scale, $target)
    };
    ($sec:expr, on $scale:expr) => {
        $crate::Dt::from_sec_f($sec, $scale, $scale)
    };
    ($sec:expr) => {
        $crate::Dt::from_sec_f($sec, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](crate::Dt) from whole nanoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// This is sugar for [`Dt::from_ns`](crate::Dt::from_ns). When the fraction is
/// omitted it is `0`. When scale is omitted, both `scale` and `target` are
/// [`Scale::TAI`](crate::Scale::TAI). With `on scale` only, `target` matches
/// `scale`. A different `target` is set with a trailing `; target`.
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `from_ns!(ns)` | `Dt::from_ns(ns, 0, Scale::TAI, Scale::TAI)` |
/// | `from_ns!(ns, frac_attos)` | `Dt::from_ns(ns, frac_attos, Scale::TAI, Scale::TAI)` |
/// | `from_ns!(ns, on scale)` | `Dt::from_ns(ns, 0, scale, scale)` |
/// | `from_ns!(ns, frac_attos, on scale)` | `Dt::from_ns(ns, frac_attos, scale, scale)` |
/// | `from_ns!(ns, on scale; target)` | `Dt::from_ns(ns, 0, scale, target)` |
/// | `from_ns!(ns, frac_attos, on scale; target)` | `Dt::from_ns(ns, frac_attos, scale, target)` |
///
/// The `on` keyword must be preceded by a comma. Optional `target` follows a
/// semicolon after the scale (`;` is allowed after an `:expr` fragment).
///
/// `ns` and `frac_attos` are the signed whole/remainder split (e.g. `-1.3` ns is
/// `ns = -1`, `frac_attos` negative for 0.3 ns in attoseconds). Floor-style
/// pairs with a non-negative remainder also work. No time-scale conversion is
/// performed.
///
/// ## Examples
///
/// ```
/// use deep_time::Scale;
/// use deep_time::from_ns;
///
/// // 1 ns = 10⁹ attoseconds; 0.3 ns = 300_000_000 attoseconds
/// let a = from_ns!(1);
/// let b = from_ns!(1, 300_000_000);
/// let c = from_ns!(-1, -300_000_000, on Scale::TAI);
/// let d = from_ns!(0, on Scale::UTC);
/// let e = from_ns!(1, on Scale::TAI; Scale::UTC);
///
/// assert_eq!(a, deep_time::Dt::from_ns(1, 0, Scale::TAI, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_ns(1, 300_000_000, Scale::TAI, Scale::TAI));
/// assert_eq!(c, deep_time::Dt::from_ns(-1, -300_000_000, Scale::TAI, Scale::TAI));
/// assert_eq!(d, deep_time::Dt::from_ns(0, 0, Scale::UTC, Scale::UTC));
/// assert_eq!(e, deep_time::Dt::from_ns(1, 0, Scale::TAI, Scale::UTC));
///
/// // -1.3 ns: signed split vs floor split — same instant
/// let signed = from_ns!(-1, -300_000_000);
/// let floor = from_ns!(-2, 700_000_000);
/// assert_eq!(signed, floor);
/// assert_eq!(signed, deep_time::Dt::from_ns(-1, -300_000_000, Scale::TAI, Scale::TAI));
/// ```
#[macro_export]
macro_rules! from_ns {
    ($ns:expr, $frac:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale, $target)
    };
    ($ns:expr, on $scale:expr; $target:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale, $target)
    };
    ($ns:expr, $frac:expr, on $scale:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale, $scale)
    };
    ($ns:expr, on $scale:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale, $scale)
    };
    ($ns:expr, $frac:expr) => {
        $crate::Dt::from_ns($ns, $frac, $crate::Scale::TAI, $crate::Scale::TAI)
    };
    ($ns:expr) => {
        $crate::Dt::from_ns($ns, 0, $crate::Scale::TAI, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](crate::Dt) from a Gregorian calendar date and optional time.
///
/// This is sugar for [`Dt::from_ymd`](crate::Dt::from_ymd). Arguments are
/// **in order** and may be stopped at any point; omitted fields use the
/// defaults below. Scale is always last and marked with `on`.
///
/// | Omitted field | Default |
/// |---------------|---------|
/// | month | `1` |
/// | day | `1` |
/// | hour, minute, second | `0` |
/// | attoseconds | `0` |
/// | scale | [`Scale::UTC`](crate::Scale::UTC) |
///
/// ## Forms
///
/// ```text
/// from_ymd!(y)
/// from_ymd!(y, on scale)
/// from_ymd!(y, m)
/// from_ymd!(y, m, on scale)
/// from_ymd!(y, m, d)
/// from_ymd!(y, m, d, on scale)
/// from_ymd!(y, m, d, h)
/// from_ymd!(y, m, d, h, on scale)
/// from_ymd!(y, m, d, h, min)
/// from_ymd!(y, m, d, h, min, on scale)
/// from_ymd!(y, m, d, h, min, s)
/// from_ymd!(y, m, d, h, min, s, on scale)
/// from_ymd!(y, m, d, h, min, s, attos)
/// from_ymd!(y, m, d, h, min, s, attos, on scale)
/// ```
///
/// The `on` keyword must be preceded by a comma (same `macro_rules!` limit as
/// [`from_sec!`](macro.from_sec.html)).
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
///     from_ymd!(2000, 1, 1, 12, on Scale::TAI),
///     deep_time::Dt::ZERO,
/// );
/// assert_eq!(
///     from_ymd!(2000, 1, 1, 12, 0, 0, 123_456_789, on Scale::UTC),
///     deep_time::Dt::from_ymd(2000, 1, 1, Scale::UTC, 12, 0, 0, 123_456_789),
/// );
/// ```
#[macro_export]
macro_rules! from_ymd {
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr, $s:expr, $attos:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, $s, $attos)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr, $s:expr, $attos:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, $s, $attos)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr, $s:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, $s, 0)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr, $s:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, $s, 0)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, $min, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, $min:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, $min, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, $h, 0, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr, $h:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, $h, 0, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $scale, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr, $d:expr) => {
        $crate::Dt::from_ymd($y, $m, $d, $crate::Scale::UTC, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, $m, 1, $scale, 0, 0, 0, 0)
    };
    ($y:expr, $m:expr) => {
        $crate::Dt::from_ymd($y, $m, 1, $crate::Scale::UTC, 0, 0, 0, 0)
    };
    ($y:expr, on $scale:expr) => {
        $crate::Dt::from_ymd($y, 1, 1, $scale, 0, 0, 0, 0)
    };
    ($y:expr) => {
        $crate::Dt::from_ymd($y, 1, 1, $crate::Scale::UTC, 0, 0, 0, 0)
    };
}
