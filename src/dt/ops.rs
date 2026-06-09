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
    /// Compares the time values represented by two `Dt`s.
    ///
    /// - This comparison is based on the total attosecond value (`self.attos` vs `other.attos`).
    /// - Does **not** perform scale conversion.
    pub const fn cmp(&self, other: &Self) -> Ordering {
        if self.attos < other.attos {
            Ordering::Less
        } else if self.attos > other.attos {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    /// Returns the smaller of two `Dt`s according to the total physical-time order
    /// defined by [`Self::cmp`].
    ///
    /// This is a `const fn` and can be used in const contexts.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    /// Returns the larger of two `Dt`s according to the total physical-time order
    /// defined by [`Self::cmp`].
    ///
    /// See [`Self::min`] for more details.
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
    /// Hashes the canonical TAI representation so that two `Dt`s that are
    /// physically equal (after conversion) produce the same hash, regardless of
    /// the original [`Scale`].
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.attos.hash(state);
    }
}
