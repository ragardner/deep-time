use crate::{Delta, MICROQUECTOS_PER_SEC, Timestamp};

impl Timestamp {
    /// Overflowing add. The result keeps the original [`ClockType`].
    #[inline]
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

    /// Overflowing sub. The result keeps the original [`ClockType`].
    #[inline]
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

    /// Saturating add. The result keeps the original [`ClockType`].
    #[inline]
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

    /// Saturating sub. The result keeps the original [`ClockType`].
    #[inline]
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

    /// Saturating mut add.
    #[inline]
    pub fn mut_add(&mut self, delta: Delta) {
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
    #[inline]
    pub fn mut_sub(&mut self, delta: Delta) {
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

    /// Returns the signed duration between two instants  
    /// (always computed in TAI internally so the result is correct  
    /// even if the two `Timestamp`s have different clock types).
    #[inline]
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

    /// Floors this instant down to the largest multiple of `unit` that is ≤ `self`.
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn floor(self, unit: Delta) -> Self {
        let origin = Self::ZERO; // J2000 TAI – consistent with your library's zero point
        let delta = self.duration_since(origin);
        origin
            .add(delta.floor(unit))
            .with_clock_type(self.clock_type)
    }

    /// Ceils this instant up to the smallest multiple of `unit` that is ≥ `self`.
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn ceil(self, unit: Delta) -> Self {
        let origin = Self::ZERO;
        let delta = self.duration_since(origin);
        origin
            .add(delta.ceil(unit))
            .with_clock_type(self.clock_type)
    }

    /// Rounds this instant to the nearest multiple of `unit`.
    /// (Halfway cases round away from zero, same semantics as `Delta::round`.)
    ///
    /// The result keeps the original [`ClockType`].
    #[inline]
    pub const fn round(self, unit: Delta) -> Self {
        let origin = Self::ZERO;
        let delta = self.duration_since(origin);
        origin
            .add(delta.round(unit))
            .with_clock_type(self.clock_type)
    }
}
