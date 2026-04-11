use crate::{Delta, Point};
use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add<Delta> for Point {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Delta) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<Delta> for Point {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Delta) {
        self.mut_add(rhs);
    }
}

impl Sub<Delta> for Point {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Delta) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<Delta> for Point {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Delta) {
        self.mut_sub(rhs);
    }
}

impl Sub<Point> for Point {
    type Output = Delta;

    #[inline(always)]
    fn sub(self, rhs: Point) -> Delta {
        self.duration_since(rhs)
    }
}
