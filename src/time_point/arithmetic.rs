use crate::{ATTOSEC_PER_SEC, ClockDrift, Delta, LocalSpacetime, Real, TimePoint};

impl TimePoint {
    /// Converts this `TimePoint` to a floating-point number of seconds since the reference epoch of its associated clock type.
    ///
    /// The conversion is lossy by design, as `f64` (`Real`) provides approximately 15.95 decimal digits of precision.
    /// For full exactness, use the integer components `sec` and `subsec` directly or higher-precision arithmetic when available.
    #[inline(always)]
    pub const fn as_sec_f(self) -> Real {
        self.sec as Real + (self.subsec as Real) / (ATTOSEC_PER_SEC as Real)
    }

    /// Performs an overflowing addition of the given `Delta` to this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self`. Overflow wraps around according to two's-complement rules for the seconds component.
    pub const fn add(self, delta: Delta) -> Self {
        let mut sec = self.sec + delta.sec;
        let mut subsec = self.subsec + delta.subsec;

        if subsec >= ATTOSEC_PER_SEC {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs an overflowing addition of the given `Delta` (by reference) to this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self`.
    pub const fn add_ref(self, delta: &Delta) -> Self {
        let mut sec = self.sec + delta.sec;
        let mut subsec = self.subsec + delta.subsec;

        if subsec >= ATTOSEC_PER_SEC {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs an overflowing subtraction of the given `Delta` from this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self`.
    pub const fn sub(self, delta: Delta) -> Self {
        let mut sec = self.sec - delta.sec;
        let mut subsec = self.subsec;

        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC - delta.subsec;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs an overflowing subtraction of the given `Delta` (by reference) from this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self`.
    pub const fn sub_ref(self, delta: &Delta) -> Self {
        let mut sec = self.sec - delta.sec;
        let mut subsec = self.subsec;

        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC - delta.subsec;
        }

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs a saturating addition of the given `Delta` to this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self` and saturates at the representable extremes rather than wrapping.
    pub const fn saturating_add(self, delta: Delta) -> Self {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i64;
        if subsec >= ATTOSEC_PER_SEC {
            subsec -= ATTOSEC_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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

    /// Performs a saturating addition of the given `Delta` (by reference) to this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self` and saturates at the representable extremes rather than wrapping.
    pub const fn saturating_add_ref(self, delta: &Delta) -> Self {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i64;
        if subsec >= ATTOSEC_PER_SEC {
            subsec -= ATTOSEC_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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

    /// Performs a saturating subtraction of the given `Delta` from this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self` and saturates at the representable extremes rather than wrapping.
    pub const fn saturating_sub(self, delta: Delta) -> Self {
        let mut subsec = self.subsec;
        let mut borrow = 0i64;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += ATTOSEC_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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

    /// Performs a saturating subtraction of the given `Delta` (by reference) from this `TimePoint`.
    ///
    /// The resulting `TimePoint` retains the original [`ClockType`] of `self` and saturates at the representable extremes rather than wrapping.
    pub const fn saturating_sub_ref(self, delta: &Delta) -> Self {
        let mut subsec = self.subsec;
        let mut borrow = 0i64;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += ATTOSEC_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
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

    /// Mutably adds the given `Delta` to this `TimePoint` using saturating arithmetic.
    ///
    /// The `clock_type` is left unchanged. The operation saturates at the representable extremes.
    pub fn mut_add(&mut self, delta: &Delta) {
        let mut subsec = self.subsec + delta.subsec;
        let mut carry = 0i64;
        if subsec >= ATTOSEC_PER_SEC {
            subsec -= ATTOSEC_PER_SEC;
            carry = 1;
        }

        let sec = self.sec.saturating_add(delta.sec).saturating_add(carry);

        self.sec = sec;
        self.subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec
        };
    }

    /// Mutably subtracts the given `Delta` from this `TimePoint` using saturating arithmetic.
    ///
    /// The `clock_type` is left unchanged. The operation saturates at the representable extremes.
    pub fn mut_sub(&mut self, delta: &Delta) {
        let mut subsec = self.subsec;
        let mut borrow = 0i64;
        if subsec >= delta.subsec {
            subsec -= delta.subsec;
        } else {
            subsec += ATTOSEC_PER_SEC - delta.subsec;
            borrow = 1;
        }

        let sec = self.sec.saturating_sub(delta.sec).saturating_sub(borrow);

        self.sec = sec;
        self.subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec
        };
    }

    /// Advances this `TimePoint` by the given elapsed duration while applying the relativistic proper-time correction
    /// derived from the supplied `LocalSpacetime` model.
    ///
    /// This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// For the spacecraft's own hardware proper-time clock, use the plain `add` method instead.
    #[inline(always)]
    pub fn adjusted_advance(&mut self, elapsed: &Delta, local_spacetime: &LocalSpacetime) {
        let dtau =
            elapsed.add(ClockDrift::from_local_spacetime(local_spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `TimePoint` by the given elapsed duration while applying the relativistic proper-time correction
    /// from a pre-computed `ClockDrift` value.
    ///
    /// This is an optimized variant of `adjusted_advance` for callers that already hold a `ClockDrift` instance.
    /// It is intended for simulation of remote clocks; the spacecraft's own hardware clock should use the plain `add` method.
    #[inline(always)]
    pub fn adjusted_advance_using_drift(&mut self, elapsed: &Delta, drift: &ClockDrift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Computes the signed duration between this `TimePoint` and an earlier instant.
    ///
    /// The duration is always calculated after converting both instants to the TAI timescale internally,
    /// ensuring correctness even when the two `TimePoint`s belong to different clock types.
    pub const fn duration_since(self, earlier: Self) -> Delta {
        let self_tai = self.to_tai();
        let earlier_tai = earlier.to_tai();

        let mut sec = self_tai.sec - earlier_tai.sec;
        let mut subsec = self_tai.subsec;

        if subsec >= earlier_tai.subsec {
            subsec -= earlier_tai.subsec;
        } else {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC - earlier_tai.subsec;
        }

        Delta { sec, subsec }
    }

    /// Computes the signed duration between this `TimePoint` and an earlier instant (by reference).
    ///
    /// The duration is always calculated after converting both instants to the TAI timescale internally,
    /// ensuring correctness even when the two `TimePoint`s belong to different clock types.
    pub const fn duration_since_ref(self, earlier: &Self) -> Delta {
        let self_tai = self.to_tai();
        let earlier_tai = earlier.to_tai();

        let mut sec = self_tai.sec - earlier_tai.sec;
        let mut subsec = self_tai.subsec;

        if subsec >= earlier_tai.subsec {
            subsec -= earlier_tai.subsec;
        } else {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC - earlier_tai.subsec;
        }

        Delta { sec, subsec }
    }

    /// Returns the numerical difference in seconds between this `TimePoint` and another (ignores `ClockType`).
    ///
    /// This method is lossy by design and is provided for testing and debugging purposes only.
    /// For the exact duration, use `duration_since` or `duration_since_ref`.
    pub const fn numerical_seconds_since(&self, other: &Self) -> Real {
        Delta {
            sec: self.sec,
            subsec: self.subsec,
        }
        .as_sec_f()
            - Delta {
                sec: other.sec,
                subsec: other.subsec,
            }
            .as_sec_f()
    }
}
