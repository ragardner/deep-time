use crate::TimeSpan;
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Add<TimeSpan> for TimeSpan {
    type Output = Self;

    /// Adds two `TimeSpan`s.
    #[inline]
    fn add(self, rhs: TimeSpan) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<TimeSpan> for TimeSpan {
    /// Adds a `TimeSpan` to this one in place.
    #[inline]
    fn add_assign(&mut self, rhs: TimeSpan) {
        *self = self.add(rhs);
    }
}

impl Sub<TimeSpan> for TimeSpan {
    type Output = Self;

    /// Subtracts a `TimeSpan` from this one.
    #[inline]
    fn sub(self, rhs: TimeSpan) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<TimeSpan> for TimeSpan {
    /// Subtracts a `TimeSpan` from this one in place.
    #[inline]
    fn sub_assign(&mut self, rhs: TimeSpan) {
        *self = self.sub(rhs);
    }
}

impl Neg for TimeSpan {
    type Output = Self;

    /// Negates this `TimeSpan` (returns the additive inverse).
    #[inline]
    fn neg(self) -> Self {
        self.neg()
    }
}

impl Mul<i64> for TimeSpan {
    type Output = Self;

    /// Multiplies this `TimeSpan` by an integer scalar.
    #[inline]
    fn mul(self, rhs: i64) -> Self {
        self.mul(rhs)
    }
}

impl MulAssign<i64> for TimeSpan {
    /// Multiplies this `TimeSpan` by an integer scalar in place.
    #[inline]
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.mul(rhs);
    }
}

impl Div<i64> for TimeSpan {
    type Output = Self;

    /// Divides this `TimeSpan` by an integer scalar.
    #[inline]
    fn div(self, rhs: i64) -> Self {
        self.div(rhs)
    }
}

impl DivAssign<i64> for TimeSpan {
    /// Divides this `TimeSpan` by an integer scalar in place.
    #[inline]
    fn div_assign(&mut self, rhs: i64) {
        *self = self.div(rhs);
    }
}

impl TimeSpan {
    /// Compares two `TimeSpan`s by their `(sec, subsec)` representation.
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

    /// Returns the smaller of two `TimeSpan`s.
    ///
    /// This is a `const fn`.
    pub const fn min(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    /// Returns the larger of two `TimeSpan`s.
    ///
    /// This is a `const fn`.
    pub const fn max(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Less => other,
            _ => self,
        }
    }
}
