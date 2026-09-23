use crate::{Dt, Real};
use core::cmp::Ordering;
use core::convert::From;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl From<Dt> for f64 {
    #[inline]
    fn from(dt: Dt) -> f64 {
        dt.to_f64()
    }
}

impl Add<Dt> for Dt {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Dt) -> Self {
        Dt::add(&self, rhs)
    }
}

impl AddAssign<Dt> for Dt {
    #[inline]
    fn add_assign(&mut self, rhs: Dt) {
        *self = self.add(rhs);
    }
}

impl Sub<Dt> for Dt {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Dt) -> Self {
        Dt::sub(&self, rhs)
    }
}

impl SubAssign<Dt> for Dt {
    #[inline]
    fn sub_assign(&mut self, rhs: Dt) {
        *self = self.sub(rhs);
    }
}

impl Neg for Dt {
    type Output = Self;

    /// Negates this `Dt` (returns the additive inverse).
    #[inline]
    fn neg(self) -> Self {
        self.neg()
    }
}

impl Mul<i64> for Dt {
    type Output = Self;

    /// Multiplies this `Dt` by an integer scalar.
    #[inline]
    fn mul(self, rhs: i64) -> Self {
        self.mul(rhs)
    }
}

impl MulAssign<i64> for Dt {
    /// Multiplies this `Dt` by an integer scalar in place.
    #[inline]
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.mul(rhs);
    }
}

impl Div<i64> for Dt {
    type Output = Self;

    /// Divides this `Dt` by an integer scalar.
    #[inline]
    fn div(self, rhs: i64) -> Self {
        self.div(rhs)
    }
}

impl DivAssign<i64> for Dt {
    /// Divides this `Dt` by an integer scalar in place.
    #[inline]
    fn div_assign(&mut self, rhs: i64) {
        *self = self.div(rhs);
    }
}

impl Mul<f64> for Dt {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        self.mul_by_f(rhs)
    }
}

impl MulAssign<f64> for Dt {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = self.mul_by_f(rhs);
    }
}

impl Div<f64> for Dt {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self {
        self.div_by_f(rhs)
    }
}

impl DivAssign<f64> for Dt {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        *self = self.div_by_f(rhs);
    }
}

impl Mul<Dt> for i64 {
    type Output = Dt;

    #[inline]
    fn mul(self, rhs: Dt) -> Dt {
        rhs.mul(self)
    }
}

impl Mul<Dt> for f64 {
    type Output = Dt;

    #[inline]
    fn mul(self, rhs: Dt) -> Dt {
        rhs.mul_by_f(self)
    }
}

impl Div<Dt> for Dt {
    type Output = Real;

    #[inline]
    fn div(self, rhs: Dt) -> Real {
        self.div_dt(rhs)
    }
}

impl Dt {
    /// Compares the raw attosecond counts of two `Dt`s.
    ///
    /// - This comparison is based solely on the raw total attosecond
    ///   value (`self.attos` vs `other.attos`).
    /// - Does **not** perform scale conversion and does not compare anything
    ///   other than the `attos` field.
    pub const fn cmp(&self, other: &Self) -> Ordering {
        if self.attos < other.attos {
            Ordering::Less
        } else if self.attos > other.attos {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    /// Orders two instants by the attosecond count each has on TAI.
    ///
    /// Both values are converted with [`Dt::to_tai`](#method.to_tai), and those
    /// counts are compared. `target` is ignored. `Ordering::Equal` means the
    /// TAI counts match. `==` stays true when the raw `attos` fields match, so
    /// a value and that value converted with [`Dt::to`](#method.to) compare
    /// `Equal` here while remaining unequal under `==`.
    ///
    /// Both arguments are instants counted from the library epoch
    /// (2000-01-01 noon) on their own `scale`. A duration uses
    /// [`Dt::cmp`](#method.cmp).
    ///
    /// `Scale::Custom` is relabeled TAI with the same count. `TDB`, `ET`,
    /// `TCB`, `LTC`, `TCL`, and `UtcHist` compare as `to_tai` returns them,
    /// including that model's rounding. Near either end of the `i128` range,
    /// saturating conversion can collapse distinct counts onto one TAI value.
    ///
    /// `sort()` and `sort_unstable()` compare `attos` and do not call this
    /// function.
    ///
    /// To order instants that are on different time scales, pass this method to
    /// `sort_by` or `sort_unstable_by`. See [Sorting](../struct.Dt.html#sorting)
    /// for more information.
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let tai = Dt::from_ymd(2000, 1, 1, Scale::TAI, 12, 0, 0, 0);
    /// let tt = tai.to(Scale::TT);
    ///
    /// assert_ne!(tai, tt);
    /// assert!(tai.cmp_instant(&tt).is_eq());
    /// assert!(tai.cmp_instant(&tai.add_sec(-10)).is_gt());
    /// ```
    #[inline]
    pub const fn cmp_instant(&self, other: &Self) -> Ordering {
        self.to_tai().cmp(&other.to_tai())
    }

    /// Returns the smaller of two `Dt`s according to [`Dt::cmp`](#method.cmp)
    /// (raw `attos`, no scale conversion).
    ///
    /// This is a `const fn` and can be used in const contexts.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    /// Returns the larger of two `Dt`s according to [`Dt::cmp`](#method.cmp)
    /// (raw `attos`, no scale conversion).
    ///
    /// See [`Dt::min`](#method.min) for more details.
    #[inline]
    pub const fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Less => other,
            _ => self,
        }
    }

    /// True if both sides have the same total attosecond value.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    #[inline(always)]
    pub const fn eq(&self, other: &Self) -> bool {
        self.attos == other.attos
    }

    /// Returns `true` if this `Dt` is less than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    #[inline(always)]
    pub const fn lt(&self, other: &Self) -> bool {
        self.attos < other.attos
    }

    /// Returns `true` if this `Dt` is greater than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    #[inline(always)]
    pub const fn gt(&self, other: &Self) -> bool {
        self.attos > other.attos
    }

    /// Returns `true` if this `Dt` is less than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    #[inline(always)]
    pub const fn le(&self, other: &Self) -> bool {
        self.attos <= other.attos
    }

    /// Returns `true` if this `Dt` is greater than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    #[inline(always)]
    pub const fn ge(&self, other: &Self) -> bool {
        self.attos >= other.attos
    }
}

impl PartialEq for Dt {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Dt::eq(self, other)
    }
}

impl Eq for Dt {}

impl PartialOrd for Dt {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Dt::cmp(self, other))
    }
}

impl Ord for Dt {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Dt::cmp(self, other)
    }
}

impl core::hash::Hash for Dt {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.attos.hash(state);
    }
}
