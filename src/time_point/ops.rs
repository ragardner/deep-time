use crate::{TimePoint, TimeSpan};
use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add<TimeSpan> for TimePoint {
    type Output = Self;

    #[inline]
    fn add(self, rhs: TimeSpan) -> Self {
        self.add(rhs)
    }
}

impl AddAssign<TimeSpan> for TimePoint {
    #[inline]
    fn add_assign(&mut self, rhs: TimeSpan) {
        *self = self.add(rhs);
    }
}

impl Sub<TimeSpan> for TimePoint {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: TimeSpan) -> Self {
        self.sub(rhs)
    }
}

impl SubAssign<TimeSpan> for TimePoint {
    #[inline]
    fn sub_assign(&mut self, rhs: TimeSpan) {
        *self = self.sub(rhs);
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = TimeSpan;

    #[inline]
    fn sub(self, rhs: TimePoint) -> TimeSpan {
        self.to_tai_since(rhs)
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

    /// Returns the smaller of two `TimePoint`s according to the total physical-time order
    /// defined by [`Self::cmp`].
    ///
    /// Both instants are converted to TAI internally, so the result is the physically
    /// earlier instant even when the two `TimePoint`s belong to different [`ClockType`]s
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

    /// Returns the larger of two `TimePoint`s according to the total physical-time order
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
        match TimePoint::cmp(*self, *other) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

impl PartialEq for TimePoint {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        TimePoint::eq(self, other)
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
        self.sec.hash(state);
        self.subsec.hash(state);
    }
}
