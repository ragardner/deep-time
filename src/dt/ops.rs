use crate::Dt;
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Add<Dt> for Dt {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Dt) -> Self {
        self.add(rhs)
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
        self.sub(rhs)
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

impl Dt {
    /// Compares this `Dt` with another by converting both to the TAI timescale
    /// (the library's canonical physical-time reference) and then comparing their
    /// `(sec, attos)` pairs.
    ///
    /// This is a `const fn` so it can be used in const contexts and is allocation-free.
    /// It provides the total order used by `<`, `>`, `<=`, `>=`, `cmp`, etc.
    ///
    /// Two `Dt`s that represent the exact same physical instant (after all
    /// leap-second, relativistic, and scale conversions) compare as `Equal`, even if
    /// they were constructed with different [`Scale`]s.
    pub const fn cmp(self, other: Self) -> Ordering {
        if self.sec < other.sec {
            Ordering::Less
        } else if self.sec > other.sec {
            Ordering::Greater
        } else if self.attos < other.attos {
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
    /// Both instants are converted to TAI internally, so the result is the physically
    /// earlier instant even when the two `Dt`s belong to different [`Scale`]s
    /// (leap seconds, relativistic offsets, etc. are all taken into account).
    ///
    /// This is a `const fn` and can be used in const contexts.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        match self.cmp(other) {
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
        match self.cmp(other) {
            Ordering::Less => other,
            _ => self,
        }
    }

    #[inline]
    pub const fn eq(&self, other: &Self) -> bool {
        match Dt::cmp(*self, *other) {
            Ordering::Equal => true,
            _ => false,
        }
    }

    /// Returns `true` if this `Dt` is less than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn lt(self, other: Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }

    /// Returns `true` if this `Dt` is greater than the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn gt(self, other: Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }

    /// Returns `true` if this `Dt` is less than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn le(self, other: Self) -> bool {
        !matches!(self.cmp(other), Ordering::Greater)
    }

    /// Returns `true` if this `Dt` is greater than or equal to the other.
    ///
    /// This is a `const fn` so it can be used in const contexts.
    pub const fn ge(self, other: Self) -> bool {
        !matches!(self.cmp(other), Ordering::Less)
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
        Some(Dt::cmp(*self, *other))
    }
}

impl Ord for Dt {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Dt::cmp(*self, *other)
    }
}

impl core::hash::Hash for Dt {
    /// Hashes the canonical TAI representation so that two `Dt`s that are
    /// physically equal (after conversion) produce the same hash, regardless of
    /// the original [`Scale`].
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.sec.hash(state);
        self.attos.hash(state);
    }
}
