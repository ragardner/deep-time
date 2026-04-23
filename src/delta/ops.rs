use crate::Delta;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Add<Delta> for Delta {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Delta) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<Delta> for Delta {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Delta) {
        *self = self.add(rhs);
    }
}

impl Sub<Delta> for Delta {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Delta) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<Delta> for Delta {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Delta) {
        *self = self.sub(rhs);
    }
}

impl Neg for Delta {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self.neg()
    }
}

impl Mul<i64> for Delta {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: i64) -> Self {
        self.mul(rhs)
    }
}

impl MulAssign<i64> for Delta {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.mul(rhs);
    }
}

impl Div<i64> for Delta {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: i64) -> Self {
        self.div(rhs)
    }
}

impl DivAssign<i64> for Delta {
    #[inline(always)]
    fn div_assign(&mut self, rhs: i64) {
        *self = self.div(rhs);
    }
}
