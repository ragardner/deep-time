use crate::{TimeSpan, TimePoint};
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add<TimeSpan> for TimePoint {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: TimeSpan) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<TimeSpan> for TimePoint {
    #[inline(always)]
    fn add_assign(&mut self, rhs: TimeSpan) {
        self.mut_add(&rhs);
    }
}

impl Sub<TimeSpan> for TimePoint {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: TimeSpan) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<TimeSpan> for TimePoint {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: TimeSpan) {
        self.mut_sub(&rhs);
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = TimeSpan;

    #[inline(always)]
    fn sub(self, rhs: TimePoint) -> TimeSpan {
        self.duration_since(rhs)
    }
}

impl TimePoint {
    /// Compares this `TimePoint` with another by converting both to the TAI timescale
    /// (the library's canonical physical-time reference) and then comparing their
    /// `(sec, subsec)` pairs.
    ///
    /// This is a `const fn` so it can be used in const contexts and is allocation-free.
    /// It provides the total order used by `<`, `>`, `<=`, `>=`, `cmp`, etc.
    ///
    /// Two `TimePoint`s that represent the exact same physical instant (after all
    /// leap-second, relativistic, and scale conversions) compare as `Equal`, even if
    /// they were constructed with different [`ClockType`]s.
    pub const fn cmp(self, other: Self) -> Ordering {
        let self_tai = self.to_tai();
        let other_tai = other.to_tai();

        // We cannot call `.cmp()` on i64/u64 yet because `Ord` is not stable as a const trait.
        // Manual comparison is fully `const fn` on primitives and does exactly the same thing.
        if self_tai.sec < other_tai.sec {
            Ordering::Less
        } else if self_tai.sec > other_tai.sec {
            Ordering::Greater
        } else if self_tai.subsec < other_tai.subsec {
            Ordering::Less
        } else if self_tai.subsec > other_tai.subsec {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for TimePoint {
    /// Two `TimePoint`s are equal if and only if they represent the same physical
    /// instant (i.e. their TAI representations are identical).
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        TimePoint::cmp(*self, *other) == Ordering::Equal
    }
}

impl Eq for TimePoint {}

impl PartialOrd for TimePoint {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(TimePoint::cmp(*self, *other))
    }
}

impl Ord for TimePoint {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        TimePoint::cmp(*self, *other)
    }
}

impl core::hash::Hash for TimePoint {
    /// Hashes the canonical TAI representation so that two `TimePoint`s that are
    /// physically equal (after conversion) produce the same hash, regardless of
    /// the original [`ClockType`].
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let tai = self.to_tai();
        tai.sec.hash(state);
        tai.subsec.hash(state);
    }
}
