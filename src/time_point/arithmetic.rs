use crate::{ClockDrift, Delta, LocalSpacetime, MICROQUECTOS_PER_SEC, TimePoint};

impl TimePoint {
    /// Overflowing add. The result keeps the original [`ClockType`].
    pub const fn add(self, delta: Delta) -> Self {
        let mut sec = self.sec + delta.sec;
        let mut subsec = self.subsec + delta.subsec;

        if subsec >= MICROQUECTOS_PER_SEC {
            sec += 1;
            subsec -= MICROQUECTOS_PER_SEC;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Overflowing add by reference.
    pub const fn add_ref(self, delta: &Delta) -> Self {
        let mut sec = self.sec + delta.sec;
        let mut subsec = self.subsec + delta.subsec;

        if subsec >= MICROQUECTOS_PER_SEC {
            sec += 1;
            subsec -= MICROQUECTOS_PER_SEC;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Overflowing sub. The result keeps the original [`ClockType`].
    pub const fn sub(self, delta: Delta) -> Self {
        let mut sec = self.sec - delta.sec;
        let mut subsec = self.subsec;

        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            sec -= 1;
            subsec += MICROQUECTOS_PER_SEC - delta.subsec;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Overflowing sub by reference.
    pub const fn sub_ref(self, delta: &Delta) -> Self {
        let mut sec = self.sec - delta.sec;
        let mut subsec = self.subsec;

        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            sec -= 1;
            subsec += MICROQUECTOS_PER_SEC - delta.subsec;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Saturating add. The result keeps the original [`ClockType`].
    pub const fn saturating_add(self, delta: Delta) -> Self {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i128;
        if subsec >= MICROQUECTOS_PER_SEC {
            subsec -= MICROQUECTOS_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Saturating add by reference.
    pub const fn saturating_add_ref(self, delta: &Delta) -> Self {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i128;
        if subsec >= MICROQUECTOS_PER_SEC {
            subsec -= MICROQUECTOS_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Saturating sub. The result keeps the original [`ClockType`].
    pub const fn saturating_sub(self, delta: Delta) -> Self {
        let mut subsec = self.subsec;
        let mut borrow = 0i128;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += MICROQUECTOS_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Saturating sub by reference.
    pub const fn saturating_sub_ref(self, delta: &Delta) -> Self {
        let mut subsec = self.subsec;
        let mut borrow = 0i128;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += MICROQUECTOS_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        let subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Saturating mut add.
    pub fn mut_add(&mut self, delta: &Delta) {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i128;
        if subsec >= MICROQUECTOS_PER_SEC {
            subsec -= MICROQUECTOS_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        self.sec = sec;
        self.subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };
    }

    /// Saturating mut sub.
    pub fn mut_sub(&mut self, delta: &Delta) {
        let mut subsec = self.subsec;
        let mut borrow = 0i128;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += MICROQUECTOS_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        self.sec = sec;
        self.subsec = if sec == i128::MAX {
            MICROQUECTOS_PER_SEC - 1
        } else if sec == i128::MIN {
            0
        } else {
            subsec
        };
    }

    /// Advances this `TimePoint` by the location time step `elapsed`,
    /// applying the relativistic proper-time rate from `local_spacetime`.
    ///
    /// Intended for simulating **remote clocks** (Earth time as seen from the
    /// spacecraft, another probe’s clock, etc.). Your own spacecraft’s
    /// hardware proper-time clock should just use `.add(dt)` directly.
    #[inline(always)]
    pub fn adjusted_advance(&mut self, elapsed: &Delta, local_spacetime: &LocalSpacetime) {
        let dtau =
            elapsed.add(ClockDrift::from_local_spacetime(local_spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `TimePoint` by the location time step `elapsed`,
    /// applying the relativistic proper-time rate from the pre-computed
    /// `drift`.
    ///
    /// This is an optimized version of [`adjusted_advance`] for when
    /// the caller already has a `ClockDrift` (e.g. cached from
    /// `ClockDrift::from_local_spacetime`).
    ///
    /// Intended for simulating **remote clocks** (Earth time as seen from the
    /// spacecraft, another probe’s clock, etc.). Your own spacecraft’s
    /// hardware proper-time clock should just use `.add(dt)` directly.
    #[inline(always)]
    pub fn adjusted_advance_drift(&mut self, elapsed: &Delta, drift: &ClockDrift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Returns the signed duration between two instants  
    /// (always computed in TAI internally so the result is correct  
    /// even if the two `TimePoint`s have different clock types).
    pub const fn duration_since(self, earlier: Self) -> Delta {
        let self_tai = self.to_tai();
        let earlier_tai = earlier.to_tai();

        let mut sec = self_tai.sec - earlier_tai.sec;
        let mut subsec = self_tai.subsec;

        if subsec >= earlier_tai.subsec {
            subsec -= earlier_tai.subsec;
        } else {
            sec -= 1;
            subsec += MICROQUECTOS_PER_SEC - earlier_tai.subsec;
        }

        Delta { sec, subsec }
    }

    /// Returns the signed duration between two instants (by reference).  
    /// (always computed in TAI internally so the result is correct  
    /// even if the two `TimePoint`s have different clock types).
    pub const fn duration_since_ref(self, earlier: &Self) -> Delta {
        let self_tai = self.to_tai();
        let earlier_tai = earlier.to_tai();

        let mut sec = self_tai.sec - earlier_tai.sec;
        let mut subsec = self_tai.subsec;

        if subsec >= earlier_tai.subsec {
            subsec -= earlier_tai.subsec;
        } else {
            sec -= 1;
            subsec += MICROQUECTOS_PER_SEC - earlier_tai.subsec;
        }

        Delta { sec, subsec }
    }

    /// Floors this instant down to the largest multiple of `unit` that is ≤ `self`.
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn floor(self, unit: Delta) -> Self {
        let origin = Self::ZERO;
        let mut ts = origin.add(self.duration_since_ref(&origin).floor(unit));
        ts.clock_type = self.clock_type;
        ts
    }

    /// Ceils this instant up to the smallest multiple of `unit` that is ≥ `self`.
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn ceil(self, unit: Delta) -> Self {
        let origin = Self::ZERO;
        let mut ts = origin.add(self.duration_since_ref(&origin).ceil(unit));
        ts.clock_type = self.clock_type;
        ts
    }

    /// Rounds this instant to the nearest multiple of `unit`.
    /// (Halfway cases round away from zero, same semantics as `Delta::round`.)
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn round(self, unit: Delta) -> Self {
        let origin = Self::ZERO;
        let mut ts = origin.add(self.duration_since_ref(&origin).round(unit));
        ts.clock_type = self.clock_type;
        ts
    }
}
