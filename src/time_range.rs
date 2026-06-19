use crate::{Dt, Scale};

/// Builder type that enables the ergonomic `start.every(step)` syntax.
///
/// This struct is created by [`Dt::every`] and is used to
/// construct a [`TimeRange`] via either `.until(end)` (inclusive) or
/// `.up_to(end)` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Debug)]
pub struct Every {
    pub(crate) start: Dt,
    pub(crate) step: Dt,
}

impl Dt {
    /// Starts building an evenly-spaced time range.
    ///
    /// This method returns an [`Every`] builder that can be chained with
    /// `.until(end)` or `.up_to(end)` to create a [`TimeRange`] iterator.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let start = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
    /// let end = Dt::from_ymd(2000, 1, 2, Scale::UTC, 0, 0, 0, 0);
    /// let step = Dt::from_hr(1, Scale::TAI);
    ///
    /// for timestamp in start.every(step).to_including(end) {
    ///     println!("{:?}", timestamp.to_ymd());
    /// }
    /// ```
    #[inline]
    pub const fn every(self, step: Dt) -> Every {
        Every { start: self, step }
    }

    /// Creates an **exclusive** evenly-spaced range from `self` to `end`.
    ///
    /// Equivalent to `self.every(step).up_to(end)`.
    #[inline]
    pub const fn range(self, end: Dt, step: Dt) -> TimeRange {
        TimeRange::exclusive(self, end, step)
    }

    /// Creates a range stepping by whole seconds.
    #[inline]
    pub const fn every_sec(self) -> Every {
        self.every(Dt::from_sec(1, Scale::TAI))
    }

    /// Creates a range stepping by whole minutes.
    #[inline]
    pub const fn every_min(self) -> Every {
        self.every(Dt::from_min(1, Scale::TAI))
    }

    /// Creates a range stepping by whole hours.
    #[inline]
    pub const fn every_hr(self) -> Every {
        self.every(Dt::from_hr(1, Scale::TAI))
    }

    /// Creates a range stepping by whole days.
    #[inline]
    pub const fn every_day(self) -> Every {
        self.every(Dt::from_hr(24, Scale::TAI))
    }

    /// Returns the next `n` points **after** `self` (exclusive of `self`)
    /// at the given step.
    ///
    /// This is a convenient way to get future points without including the start.
    #[inline]
    pub fn next_n(self, n: usize, step: Dt) -> impl ExactSizeIterator<Item = Dt> {
        (self + step).for_n_steps(n, step)
    }

    /// Returns an iterator yielding exactly `n` evenly spaced points
    /// starting from `self`.
    ///
    /// This is a convenient one-liner for the common "next N steps" pattern.
    #[inline]
    pub fn for_n_steps(self, n: usize, step: Dt) -> impl ExactSizeIterator<Item = Dt> {
        let end = self + step * (n as i64);
        TimeRange::exclusive(self, end, step).take(n)
    }
}

impl Every {
    /// Creates an **inclusive** time range (`start ... end`).
    ///
    /// The resulting iterator will yield `end` as the final element
    /// (provided `end` is reachable from `start` with the given step).
    #[inline]
    pub fn to_including(self, end: Dt) -> TimeRange {
        TimeRange::new(self.start, end, self.step, true)
    }

    /// Creates an **exclusive** time range (`start ... end`).
    ///
    /// The resulting iterator will **not** yield `end`.
    #[inline]
    pub fn to_excluding(self, end: Dt) -> TimeRange {
        TimeRange::new(self.start, end, self.step, false)
    }
}

/// An iterator over evenly spaced [`Dt`] values.
///
/// `TimeRange` is the time-domain equivalent of `std::iter::StepBy` or
/// NumPy's `linspace` / `arange`. It supports both forward and backward
/// iteration and implements [`ExactSizeIterator`].
///
/// ## Construction
///
/// ```rust
/// use deep_time::{Dt, Scale, TimeRange};
///
/// let start = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
/// let end = Dt::from_ymd(2000, 1, 2, Scale::UTC, 0, 0, 0, 0);
/// let step = Dt::from_hr(1, Scale::TAI);
///
/// for timestamp in start.every(step).to_including(end) {
///     println!("{:?}", timestamp.to_ymd());
/// }
///
/// // Or use the explicit constructors:
/// TimeRange::inclusive(start, end, step);
/// TimeRange::exclusive(start, end, step);
/// ```
///
/// ## Iteration Behavior
///
/// - Zero step is handled gracefully (yields at most one element).
/// - Negative steps are supported for reverse iteration.
/// - The iterator is **lazy** and evaluates in constant time per step.
/// - Implements [`DoubleEndedIterator`] and [`ExactSizeIterator`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    pub(crate) start: Dt,
    pub(crate) current: Dt,
    pub(crate) end: Dt,
    pub(crate) step: Dt,
    pub(crate) inclusive: bool,
    pub(crate) finished: bool,
}

impl TimeRange {
    /// Creates an **inclusive** evenly-spaced time range.
    ///
    /// The iterator will yield `end` if it is exactly reachable.
    #[inline]
    pub const fn inclusive(start: Dt, end: Dt, step: Dt) -> TimeRange {
        Self::new(start, end, step, true)
    }

    /// Creates an **exclusive** evenly-spaced time range.
    ///
    /// The iterator will **not** yield `end`.
    #[inline]
    pub const fn exclusive(start: Dt, end: Dt, step: Dt) -> TimeRange {
        Self::new(start, end, step, false)
    }

    /// Internal constructor.
    #[inline]
    pub const fn new(start: Dt, end: Dt, step: Dt, inclusive: bool) -> TimeRange {
        Self {
            start,
            current: start,
            end,
            step,
            inclusive,
            finished: false,
        }
    }
}

impl Iterator for TimeRange {
    type Item = Dt;

    /// Advances the iterator and returns the next [`Dt`].
    ///
    /// Returns `None` once the range has been exhausted.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if self.step.is_zero() {
            self.finished = true;
            if self.start == self.end && self.inclusive {
                return Some(self.start);
            }
            return None;
        }

        let item = self.current;

        let to_end = self.current.to_diff_raw(self.end);
        let step_positive = self.step.is_positive();

        let beyond_end = if step_positive {
            to_end > Dt::ZERO
        } else {
            to_end < Dt::ZERO
        };

        if beyond_end {
            self.finished = true;
            return None;
        }

        // Exclusive ranges must not yield `end` even when it is exactly reachable
        if !self.inclusive && self.current == self.end {
            self.finished = true;
            return None;
        }

        self.current += self.step;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for TimeRange {
    /// Returns the next element from the back of the range.
    ///
    /// This allows `TimeRange` to be used with `.rev()` and in
    /// double-ended iteration contexts.
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut rev = *self;
        rev.step = rev.step.neg();

        let item = rev.next();

        if item.is_some() {
            self.current = rev.current;
        }

        item
    }
}

impl ExactSizeIterator for TimeRange {
    /// Returns the exact number of elements this iterator will yield.
    ///
    /// This is computed in constant time without iterating.
    fn len(&self) -> usize {
        if self.finished {
            return 0;
        }

        if self.step.is_zero() {
            return if self.current == self.end && self.inclusive {
                1
            } else {
                0
            };
        }

        // Mirror the yield decision from next()
        let to_end = self.current.to_diff_raw(self.end);
        let step_positive = self.step.is_positive();

        let beyond_end = if step_positive {
            to_end > Dt::ZERO
        } else {
            to_end < Dt::ZERO
        };

        if beyond_end {
            return 0;
        }

        if !self.inclusive && self.current == self.end {
            return 0;
        }

        // current is yieldable → compute remaining points
        let diff = self.end.to_diff_raw(self.current);
        let intervals = diff.abs_div_floor(self.step);

        if self.inclusive {
            intervals.saturating_add(1)
        } else {
            // For exclusive:
            // - If we would land exactly on `end` after `intervals` steps → exclude it
            // - Otherwise include the extra point
            if intervals == 0 {
                1
            } else {
                let reached = self.current + (self.step * (intervals as i64));
                if reached == self.end {
                    intervals
                } else {
                    intervals.saturating_add(1)
                }
            }
        }
    }
}
