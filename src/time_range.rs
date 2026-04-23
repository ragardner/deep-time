//! High-precision evenly-spaced `TimePoint` iterator (the "linspace" for time).

use crate::{Delta, TimePoint};

/// Builder type that enables the ergonomic `start.every(step)` syntax.
///
/// This struct is created by [`TimePoint::every`] and is used to
/// construct a [`TimeRange`] via either `.until(end)` (inclusive) or
/// `.up_to(end)` (exclusive).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Debug)]
pub struct Every {
    start: TimePoint,
    step: Delta,
}

impl TimePoint {
    /// Starts building an evenly-spaced time range.
    ///
    /// This method returns an [`Every`] builder that can be chained with
    /// `.until(end)` or `.up_to(end)` to create a [`TimeRange`] iterator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let start = TimePoint::from_gregorian(2025, 1, 1, 0, 0, 0, 0, ClockType::TAI);
    /// let step = Delta::from_hours(1);
    ///
    /// // Inclusive range: yields 25 points (including both start and end)
    /// for t in start.every(step).until(end) { ... }
    ///
    /// // Exclusive range: yields 24 points
    /// for t in start.every(step).up_to(end) { ... }
    /// ```
    #[inline]
    pub const fn every(self, step: Delta) -> Every {
        Every { start: self, step }
    }

    /// Creates an **inclusive** evenly-spaced range from `self` to `end`.
    ///
    /// Equivalent to `self.every(step).until(end)`.
    #[inline]
    pub const fn range_to(self, end: TimePoint, step: Delta) -> TimeRange {
        TimeRange::inclusive(self, end, step)
    }

    /// Creates an **exclusive** evenly-spaced range from `self` to `end`.
    ///
    /// Equivalent to `self.every(step).up_to(end)`.
    #[inline]
    pub const fn range_until(self, end: TimePoint, step: Delta) -> TimeRange {
        TimeRange::exclusive(self, end, step)
    }

    /// Creates a range stepping by whole seconds.
    #[inline]
    pub const fn every_second(self) -> Every {
        self.every(Delta::from_sec(1))
    }

    /// Creates a range stepping by whole minutes.
    #[inline]
    pub const fn every_minute(self) -> Every {
        self.every(Delta::from_min(1))
    }

    /// Creates a range stepping by whole hours.
    #[inline]
    pub const fn every_hour(self) -> Every {
        self.every(Delta::from_hr(1))
    }

    /// Creates a range stepping by whole days.
    #[inline]
    pub const fn every_day(self) -> Every {
        self.every(Delta::from_hr(24))
    }

    /// Returns the next `n` points **after** `self` (exclusive of `self`)
    /// at the given step.
    ///
    /// This is a convenient way to get future points without including the start.
    #[inline]
    pub fn next_n(self, n: usize, step: Delta) -> impl Iterator<Item = TimePoint> {
        (self + step).for_n_steps(n, step)
    }

    /// Returns an iterator yielding exactly `n` evenly spaced points
    /// starting from `self`.
    ///
    /// This is a convenient one-liner for the common "next N steps" pattern.
    #[inline]
    pub fn for_n_steps(self, n: usize, step: Delta) -> impl Iterator<Item = TimePoint> {
        // We create an exclusive range long enough for n steps, then limit it
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
    pub fn until(self, end: TimePoint) -> TimeRange {
        TimeRange::new(self.start, end, self.step, true)
    }

    /// Creates an **exclusive** time range (`start ... end`).
    ///
    /// The resulting iterator will **not** yield `end`.
    #[inline]
    pub fn up_to(self, end: TimePoint) -> TimeRange {
        TimeRange::new(self.start, end, self.step, false)
    }

    /// Creates a **descending** inclusive range.
    ///
    /// Example: `start.every(-1.hour()).down_to(earlier_time)`
    #[inline]
    pub fn down_to(self, end: TimePoint) -> TimeRange {
        TimeRange::new(self.start, end, self.step, true)
    }

    /// Size of the canonical wire representation in bytes (33 bytes).
    pub const WIRE_SIZE: usize = TimePoint::WIRE_SIZE + Delta::WIRE_SIZE;

    /// Serializes this `Every` builder into a fixed 33-byte buffer.
    ///
    /// The layout is simply the concatenation of `start` (17 bytes) and `step` (16 bytes).
    #[inline]
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        let start = self.start.to_wire_bytes();
        let step = self.step.to_wire_bytes();
        buf[0..17].copy_from_slice(&start);
        buf[17..33].copy_from_slice(&step);
        buf
    }

    /// Deserializes an `Every` builder from exactly 33 bytes.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed size with strict validation
    /// of the inner `TimePoint` and `Delta`.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let start = TimePoint::from_wire_bytes(&bytes[0..17])?;
        let step = Delta::from_wire_bytes(&bytes[17..33])?;
        Some(Self { start, step })
    }
}

/// An iterator over evenly spaced [`TimePoint`] values.
///
/// `TimeRange` is the time-domain equivalent of `std::iter::StepBy` or
/// NumPy's `linspace` / `arange`. It supports both forward and backward
/// iteration and implements [`ExactSizeIterator`].
///
/// # Construction
///
/// Prefer the ergonomic builder syntax:
///
/// ```ignore
/// start.every(step).until(end)   // inclusive
/// start.every(step).up_to(end)   // exclusive
/// ```
///
/// Or use the explicit constructors:
///
/// ```ignore
/// TimeRange::inclusive(start, end, step)
/// TimeRange::exclusive(start, end, step)
/// ```
///
/// # Iteration Behavior
///
/// - Zero step is handled gracefully (yields at most one element).
/// - Negative steps are supported for reverse iteration.
/// - The iterator is **lazy** and evaluates in constant time per step.
/// - Implements [`DoubleEndedIterator`] and [`ExactSizeIterator`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    start: TimePoint,
    current: TimePoint,
    end: TimePoint,
    step: Delta,
    inclusive: bool,
    finished: bool,
}

impl TimeRange {
    /// Current wire format version.
    pub const WIRE_VERSION: u8 = 1;

    /// Size of the canonical wire representation in bytes.
    /// Only the logical definition is stored (runtime state is not serialized).
    pub const WIRE_SIZE: usize = 1 + 2 * TimePoint::WIRE_SIZE + Delta::WIRE_SIZE + 1;

    /// Serializes this `TimeRange` into a fixed buffer.
    ///
    /// Only the logical definition is stored:
    /// - `start` + `end` + `step` + `inclusive` flag
    ///
    /// Runtime iterator state (`current`, `finished`) is **not** serialized.
    #[inline]
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0] = Self::WIRE_VERSION;

        let start = self.start.to_wire_bytes();
        let end = self.end.to_wire_bytes();
        let step = self.step.to_wire_bytes();

        let tp_size = TimePoint::WIRE_SIZE;
        let delta_size = Delta::WIRE_SIZE;

        buf[1..1 + tp_size].copy_from_slice(&start);
        buf[1 + tp_size..1 + 2 * tp_size].copy_from_slice(&end);
        buf[1 + 2 * tp_size..1 + 2 * tp_size + delta_size].copy_from_slice(&step);
        buf[1 + 2 * tp_size + delta_size] = if self.inclusive { 1 } else { 0 };

        buf
    }

    /// Deserializes a `TimeRange` from exactly `WIRE_SIZE` bytes.
    ///
    /// The iterator is reconstructed in its initial state
    /// (`current = start`, `finished = false`).
    ///
    /// Returns `None` if the version is unknown or any component is invalid.
    ///
    /// ## Security
    ///
    /// Safe for untrusted input. Fixed size with layered validation
    /// of all inner types. No runtime iterator state is accepted from the wire.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }

        if bytes[0] != Self::WIRE_VERSION {
            return None;
        }

        let tp_size = TimePoint::WIRE_SIZE;
        let delta_size = Delta::WIRE_SIZE;

        let start = TimePoint::from_wire_bytes(&bytes[1..1 + tp_size])?;
        let end = TimePoint::from_wire_bytes(&bytes[1 + tp_size..1 + 2 * tp_size])?;
        let step = Delta::from_wire_bytes(&bytes[1 + 2 * tp_size..1 + 2 * tp_size + delta_size])?;
        let inclusive = bytes[1 + 2 * tp_size + delta_size] != 0;

        Some(Self::new(start, end, step, inclusive))
    }

    /// Creates an **inclusive** evenly-spaced time range.
    ///
    /// The iterator will yield `end` if it is exactly reachable.
    #[inline]
    pub const fn inclusive(start: TimePoint, end: TimePoint, step: Delta) -> Self {
        Self::new(start, end, step, true)
    }

    /// Creates an **exclusive** evenly-spaced time range.
    ///
    /// The iterator will **not** yield `end`.
    #[inline]
    pub const fn exclusive(start: TimePoint, end: TimePoint, step: Delta) -> Self {
        Self::new(start, end, step, false)
    }

    /// Internal constructor.
    #[inline]
    const fn new(start: TimePoint, end: TimePoint, step: Delta, inclusive: bool) -> Self {
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
    type Item = TimePoint;

    /// Advances the iterator and returns the next [`TimePoint`].
    ///
    /// Returns `None` once the range has been exhausted.
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let item = self.current;

        let to_end = self.current.duration_since_ref(&self.end);
        let passed = if self.step.is_zero() {
            true
        } else if self.step.sec > 0 || (self.step.sec == 0 && self.step.subsec > 0) {
            to_end > Delta::ZERO
        } else {
            to_end < Delta::ZERO
        };

        if passed {
            self.finished = true;
            if self.inclusive && self.current == self.end {
                return Some(item);
            }
            return None;
        }

        self.current = self.current + self.step;
        Some(item)
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

        let mut rev = self.clone();
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
        if self.step.is_zero() {
            return if self.start == self.end && self.inclusive {
                1
            } else {
                0
            };
        }

        let total = self.end.duration_since_ref(&self.start);
        let steps = total.abs_div_floor(self.step);

        if self.inclusive {
            steps.saturating_add(1)
        } else {
            steps
        }
    }
}
