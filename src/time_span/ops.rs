use crate::TimeSpan;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Add<TimeSpan> for TimeSpan {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: TimeSpan) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<TimeSpan> for TimeSpan {
    #[inline(always)]
    fn add_assign(&mut self, rhs: TimeSpan) {
        *self = self.add(rhs);
    }
}

impl Sub<TimeSpan> for TimeSpan {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: TimeSpan) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<TimeSpan> for TimeSpan {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: TimeSpan) {
        *self = self.sub(rhs);
    }
}

impl Neg for TimeSpan {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self.neg()
    }
}

impl Mul<i64> for TimeSpan {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: i64) -> Self {
        self.mul(rhs)
    }
}

impl MulAssign<i64> for TimeSpan {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: i64) {
        *self = self.mul(rhs);
    }
}

impl Div<i64> for TimeSpan {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: i64) -> Self {
        self.div(rhs)
    }
}

impl DivAssign<i64> for TimeSpan {
    #[inline(always)]
    fn div_assign(&mut self, rhs: i64) {
        *self = self.div(rhs);
    }
}
