//! High-precision evenly-spaced `Timestamp` iterator (the "linspace" for time).

use crate::{Delta, Timestamp};

/// Builder for nice ergonomic syntax: `start.every(5.minutes()).until(end)`
#[derive(Clone, Debug)]
pub struct Every {
    start: Timestamp,
    step: Delta,
}

impl Timestamp {
    /// Start building a range: `start.every(step)`
    pub const fn every(self, step: Delta) -> Every {
        Every { start: self, step }
    }
}

impl Every {
    pub fn until(self, end: Timestamp) -> TimeRange {
        TimeRange::new(self.start, end, self.step, true)
    }

    pub fn up_to(self, end: Timestamp) -> TimeRange {
        TimeRange::new(self.start, end, self.step, false)
    }
}

/// Evenly spaced time range iterator.
///
/// Created with:
/// - `start.every(step).until(end)` (inclusive)
/// - `start.every(step).up_to(end)` (exclusive)
/// - or the traditional `TimeRange::inclusive(...)` / `exclusive(...)`
#[derive(Clone, Debug)]
pub struct TimeRange {
    start: Timestamp,
    current: Timestamp,
    end: Timestamp,
    step: Delta,
    inclusive: bool,
    finished: bool,
}

impl TimeRange {
    pub const fn inclusive(start: Timestamp, end: Timestamp, step: Delta) -> Self {
        Self::new(start, end, step, true)
    }

    pub const fn exclusive(start: Timestamp, end: Timestamp, step: Delta) -> Self {
        Self::new(start, end, step, false)
    }

    const fn new(start: Timestamp, end: Timestamp, step: Delta, inclusive: bool) -> Self {
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
    type Item = Timestamp;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let item = self.current;

        // Check if we have passed the end
        let to_end = self.current.duration_since(self.end);
        let passed = if self.step.is_zero() {
            true
        } else if self.step.sec > 0 || (self.step.sec == 0 && self.step.subsec > 0) {
            to_end > Delta::ZERO
        } else {
            to_end < Delta::ZERO
        };

        if passed {
            self.finished = true;
            // Still yield the exact end point if inclusive and we are on it
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
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Temporarily reverse direction to find the "last" element
        let mut rev = self.clone();
        rev.step = rev.step.neg();

        let item = rev.next();

        if item.is_some() {
            self.current = rev.current; // sync forward iterator state
        }

        item
    }
}

impl ExactSizeIterator for TimeRange {
    fn len(&self) -> usize {
        if self.step.is_zero() {
            return if self.start == self.end && self.inclusive {
                1
            } else {
                0
            };
        }

        let total = self.end.duration_since(self.start);
        let steps = total.abs_div_floor(self.step);

        if self.inclusive {
            steps.saturating_add(1)
        } else {
            steps
        }
    }
}
