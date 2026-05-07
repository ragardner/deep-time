use crate::TSpan;
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Add<TSpan> for TSpan {
    type Output = Self;

    /// Adds two `TSpan`s.
    #[inline]
    fn add(self, rhs: TSpan) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<TSpan> for TSpan {
    /// Adds a `TSpan` to this one in place.
    #[inline]
    fn add_assign(&mut self, rhs: TSpan) {
        *self = self.add(rhs);
    }
}

impl Sub<TSpan> for TSpan {
    type Output = Self;

    /// Subtracts a `TSpan` from this one.
    #[inline]
    fn sub(self, rhs: TSpan) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<TSpan> for TSpan {
    /// Subtracts a `TSpan` from this one in place.
    #[inline]
    fn sub_assign(&mut self, rhs: TSpan) {
        *self = self.sub(rhs);
    }
}

impl Neg for TSpan {
    type Output = Self;

    /// Negates this `TSpan` (returns the additive inverse).
    #[inline]
    fn neg(self) -> Self {
        self.neg()
    }
}

impl Mul<i64> for TSpan {
    type Output = Self;

    /// Multiplies this `TSpan` by an integer scalar.
    #[inline]
    fn mul(self, rhs: i64) -> Self {
        self.mul(rhs)
    }
}

impl MulAssign<i64> for TSpan {
    /// Multiplies this `TSpan` by an integer scalar in place.
    #[inline]
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.mul(rhs);
    }
}

impl Div<i64> for TSpan {
    type Output = Self;

    /// Divides this `TSpan` by an integer scalar.
    #[inline]
    fn div(self, rhs: i64) -> Self {
        self.div(rhs)
    }
}

impl DivAssign<i64> for TSpan {
    /// Divides this `TSpan` by an integer scalar in place.
    #[inline]
    fn div_assign(&mut self, rhs: i64) {
        *self = self.div(rhs);
    }
}

impl TSpan {
    /// Compares two `TSpan`s by their `(sec, subsec)` representation.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn cmp(self, other: Self) -> Ordering {
        if self.sec < other.sec {
            Ordering::Less
        } else if self.sec > other.sec {
            Ordering::Greater
        } else if self.subsec < other.subsec {
            Ordering::Less
        } else if self.subsec > other.subsec {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    /// Returns the smaller of two `TSpan`s.
    ///
    /// This is a `const fn`.
    pub const fn min(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    /// Returns the larger of two `TSpan`s.
    ///
    /// This is a `const fn`.
    pub const fn max(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Less => other,
            _ => self,
        }
    }

    /// Returns `true` if this `TSpan` is less than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn lt(self, other: Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }

    /// Returns `true` if this `TSpan` is greater than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn gt(self, other: Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }

    /// Returns `true` if this `TSpan` is less than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn le(self, other: Self) -> bool {
        !matches!(self.cmp(other), Ordering::Greater)
    }

    /// Returns `true` if this `TSpan` is greater than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn ge(self, other: Self) -> bool {
        !matches!(self.cmp(other), Ordering::Less)
    }
}
