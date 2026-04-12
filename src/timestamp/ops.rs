use crate::{Delta, Timestamp};
use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add<Delta> for Timestamp {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Delta) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<Delta> for Timestamp {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Delta) {
        self.mut_add(rhs);
    }
}

impl Sub<Delta> for Timestamp {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Delta) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<Delta> for Timestamp {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Delta) {
        self.mut_sub(rhs);
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = Delta;

    #[inline(always)]
    fn sub(self, rhs: Timestamp) -> Delta {
        self.duration_since(rhs)
    }
}
