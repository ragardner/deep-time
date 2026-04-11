use crate::Delta;
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

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
