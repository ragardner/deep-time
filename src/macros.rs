//! Crate-root convenience macros (`from_sec!`, `from_ns!`, `from_ymd!`, …).

/// Builds a [`Dt`](crate::Dt) from whole seconds and an optional signed
/// sub-second remainder (attoseconds).
///
/// This is sugar for [`Dt::from_sec_and_frac`](crate::Dt::from_sec_and_frac).
/// When the fraction is omitted it is `0`. When the scale is omitted it is
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `from_sec!(sec)` | `Dt::from_sec_and_frac(sec, 0, Scale::TAI)` |
/// | `from_sec!(sec, frac)` | `Dt::from_sec_and_frac(sec, frac, Scale::TAI)` |
/// | `from_sec!(sec, on scale)` | `Dt::from_sec_and_frac(sec, 0, scale)` |
/// | `from_sec!(sec, frac, on scale)` | `Dt::from_sec_and_frac(sec, frac, scale)` |
///
/// The `on` keyword must be preceded by a comma: `macro_rules!` only allows
/// `=>`, `,`, or `;` immediately after an `:expr` fragment, so
/// `from_sec!(sec on scale)` is not expressible.
///
/// `sec` and `frac` are the signed whole/remainder split (e.g. `-1.3` s is
/// `sec = -1`, `frac = -300_000_000_000_000_000`). Floor-style pairs with a
/// non-negative remainder also work, since the underlying math is
/// `sec × 10¹⁸ + frac`.
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
///
/// assert_eq!(a, deep_time::Dt::from_sec_and_frac(1, 0, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_sec_and_frac(1, 300_000_000_000_000_000, Scale::TAI));
/// assert_eq!(c, deep_time::Dt::from_sec_and_frac(-1, -300_000_000_000_000_000, Scale::TAI));
/// assert_eq!(d, deep_time::Dt::from_sec_and_frac(0, 0, Scale::UTC));
///
/// // -1.3 s: signed (decimal) split vs floor split — same instant
/// let signed = from_sec!(-1, -300_000_000_000_000_000);
/// let floor = from_sec!(-2, 700_000_000_000_000_000);
/// assert_eq!(signed, floor);
/// assert_eq!(signed, deep_time::Dt::span(-1_300_000_000_000_000_000));
/// ```
#[macro_export]
macro_rules! from_sec {
    ($sec:expr, $frac:expr, on $scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $scale)
    };
    ($sec:expr, on $scale:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $scale)
    };
    ($sec:expr, $frac:expr) => {
        $crate::Dt::from_sec_and_frac($sec, $frac, $crate::Scale::TAI)
    };
    ($sec:expr) => {
        $crate::Dt::from_sec_and_frac($sec, 0, $crate::Scale::TAI)
    };
}

/// Builds a [`Dt`](crate::Dt) from whole nanoseconds and an optional signed
/// fractional remainder in attoseconds.
///
/// This is sugar for [`Dt::from_ns`](crate::Dt::from_ns). When the fraction is
/// omitted it is `0`. When the scale is omitted it is
/// [`Scale::TAI`](crate::Scale::TAI).
///
/// ## Forms
///
/// | Form | Equivalent to |
/// |------|----------------|
/// | `from_ns!(ns)` | `Dt::from_ns(ns, 0, Scale::TAI)` |
/// | `from_ns!(ns, frac_attos)` | `Dt::from_ns(ns, frac_attos, Scale::TAI)` |
/// | `from_ns!(ns, on scale)` | `Dt::from_ns(ns, 0, scale)` |
/// | `from_ns!(ns, frac_attos, on scale)` | `Dt::from_ns(ns, frac_attos, scale)` |
///
/// The `on` keyword must be preceded by a comma (same limit as [`from_sec!`]).
///
/// `ns` and `frac_attos` are the signed whole/remainder split (e.g. `-1.3` ns is
/// `ns = -1`, `frac_attos` negative for 0.3 ns in attoseconds). Floor-style
/// pairs with a non-negative remainder also work.
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
///
/// assert_eq!(a, deep_time::Dt::from_ns(1, 0, Scale::TAI));
/// assert_eq!(b, deep_time::Dt::from_ns(1, 300_000_000, Scale::TAI));
/// assert_eq!(c, deep_time::Dt::from_ns(-1, -300_000_000, Scale::TAI));
/// assert_eq!(d, deep_time::Dt::from_ns(0, 0, Scale::UTC));
///
/// // -1.3 ns: signed split vs floor split — same instant
/// let signed = from_ns!(-1, -300_000_000);
/// let floor = from_ns!(-2, 700_000_000);
/// assert_eq!(signed, floor);
/// assert_eq!(signed, deep_time::Dt::from_ns(-1, -300_000_000, Scale::TAI));
/// ```
#[macro_export]
macro_rules! from_ns {
    ($ns:expr, $frac:expr, on $scale:expr) => {
        $crate::Dt::from_ns($ns, $frac, $scale)
    };
    ($ns:expr, on $scale:expr) => {
        $crate::Dt::from_ns($ns, 0, $scale)
    };
    ($ns:expr, $frac:expr) => {
        $crate::Dt::from_ns($ns, $frac, $crate::Scale::TAI)
    };
    ($ns:expr) => {
        $crate::Dt::from_ns($ns, 0, $crate::Scale::TAI)
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
/// [`from_sec!`]).
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
