use crate::{Delta, TimePoint};
use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add<Delta> for TimePoint {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Delta) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<Delta> for TimePoint {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Delta) {
        self.mut_add(&rhs);
    }
}

impl Sub<Delta> for TimePoint {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Delta) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<Delta> for TimePoint {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Delta) {
        self.mut_sub(&rhs);
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = Delta;

    #[inline(always)]
    fn sub(self, rhs: TimePoint) -> Delta {
        self.duration_since(rhs)
    }
}
