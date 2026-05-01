use crate::{ATTOSEC_PER_SEC, ClockDrift, LocalSpacetime, Real, TimePoint, TimeSpan};

impl TimePoint {
    /// Converts this `TimePoint` to a floating-point number of seconds since the reference epoch of its associated clock type.
    ///
    /// The conversion is lossy by design, as `f64` (`Real`) provides approximately 15.95 decimal digits of precision.
    /// For full exactness, use the integer components `sec` and `subsec` directly or higher-precision arithmetic when available.
    #[inline(always)]
    pub const fn as_sec_f(self) -> Real {
        self.sec as Real + (self.subsec as Real) / (ATTOSEC_PER_SEC as Real)
    }

    /// Performs an exact addition of any `TimeSpan` (positive or negative) to this `TimePoint`.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`].
    /// This operation can overflow (wrapping around) if the result exceeds `i64` bounds.
    pub const fn add(self, span: TimeSpan) -> Self {
        let mut sec = self.sec + span.sec;
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOSEC_PER_SEC as i64 {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC as i64;
        } else if subsec < 0 {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC as i64;
        }

        Self {
            sec,
            subsec: subsec as u64,
            clock_type: self.clock_type,
        }
    }

    /// Performs an exact addition of any `TimeSpan` (by reference, positive or negative) to this `TimePoint`.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`].
    /// This operation can overflow (wrapping around) if the result exceeds `i64` bounds.
    pub const fn add_ref(self, span: &TimeSpan) -> Self {
        let mut sec = self.sec + span.sec;
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOSEC_PER_SEC as i64 {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC as i64;
        } else if subsec < 0 {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC as i64;
        }

        Self {
            sec,
            subsec: subsec as u64,
            clock_type: self.clock_type,
        }
    }

    /// Performs an exact subtraction of any `TimeSpan` (positive or negative) from this `TimePoint`.
    ///
    /// Subtracting a negative `TimeSpan` moves the time point forward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`].
    /// This operation can overflow (wrapping around) if the result exceeds `i64` bounds.
    pub const fn sub(self, span: TimeSpan) -> Self {
        let mut sec = self.sec - span.sec;
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC as i64;
        } else if subsec >= ATTOSEC_PER_SEC as i64 {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC as i64;
        }

        Self {
            sec,
            subsec: subsec as u64,
            clock_type: self.clock_type,
        }
    }

    /// Performs an exact subtraction of any `TimeSpan` (by reference, positive or negative) from this `TimePoint`.
    ///
    /// Subtracting a negative `TimeSpan` moves the time point forward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`].
    /// This operation can overflow (wrapping around) if the result exceeds `i64` bounds.
    pub const fn sub_ref(self, span: &TimeSpan) -> Self {
        let mut sec = self.sec - span.sec;
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            sec -= 1;
            subsec += ATTOSEC_PER_SEC as i64;
        } else if subsec >= ATTOSEC_PER_SEC as i64 {
            sec += 1;
            subsec -= ATTOSEC_PER_SEC as i64;
        }

        Self {
            sec,
            subsec: subsec as u64,
            clock_type: self.clock_type,
        }
    }

    /// Performs a saturating addition of any `TimeSpan` (positive or negative) to this `TimePoint`.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`] and saturates at the
    /// representable extremes (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn saturating_add(self, span: TimeSpan) -> Self {
        let mut sec = self.sec.saturating_add(span.sec);
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs a saturating addition of any `TimeSpan` (by reference, positive or negative) to this `TimePoint`.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`] and saturates at the
    /// representable extremes (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn saturating_add_ref(self, span: &TimeSpan) -> Self {
        let mut sec = self.sec.saturating_add(span.sec);
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs a saturating subtraction of any `TimeSpan` (positive or negative) from this `TimePoint`.
    ///
    /// Subtracting a negative `TimeSpan` moves the time point forward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`] and saturates at the
    /// representable extremes (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn saturating_sub(self, span: TimeSpan) -> Self {
        let mut sec = self.sec.saturating_sub(span.sec);
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        } else if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Performs a saturating subtraction of any `TimeSpan` (by reference, positive or negative) from this `TimePoint`.
    ///
    /// Subtracting a negative `TimeSpan` moves the time point forward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`] and saturates at the
    /// representable extremes (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn saturating_sub_ref(self, span: &TimeSpan) -> Self {
        let mut sec = self.sec.saturating_sub(span.sec);
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        } else if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if sec == i64::MIN {
            0
        } else {
            subsec as u64
        };

        Self {
            sec,
            subsec,
            clock_type: self.clock_type,
        }
    }

    /// Mutably adds the given `TimeSpan` (positive or negative) to this `TimePoint` using saturating arithmetic.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The `clock_type` is left unchanged. The operation saturates at the representable extremes
    /// (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn mut_add(&mut self, span: &TimeSpan) -> &mut Self {
        let mut sec = self.sec.saturating_add(span.sec);
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        }

        self.sec = if sec == i64::MAX {
            i64::MAX
        } else if sec == i64::MIN {
            i64::MIN
        } else {
            sec
        };

        self.subsec = if self.sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if self.sec == i64::MIN {
            0
        } else {
            subsec as u64
        };
        self
    }

    /// Mutably subtracts the given `TimeSpan` (positive or negative) from this `TimePoint` using saturating arithmetic.
    ///
    /// Subtracting a negative `TimeSpan` moves the time point forward in time.
    /// The `clock_type` is left unchanged. The operation saturates at the representable extremes
    /// (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn mut_sub(&mut self, span: &TimeSpan) -> &mut Self {
        let mut sec = self.sec.saturating_sub(span.sec);
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOSEC_PER_SEC as i64;
        } else if subsec >= ATTOSEC_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOSEC_PER_SEC as i64;
        }

        self.sec = if sec == i64::MAX {
            i64::MAX
        } else if sec == i64::MIN {
            i64::MIN
        } else {
            sec
        };

        self.subsec = if self.sec == i64::MAX {
            ATTOSEC_PER_SEC - 1
        } else if self.sec == i64::MIN {
            0
        } else {
            subsec as u64
        };
        self
    }

    /// Advances this `TimePoint` by the given elapsed duration while applying the relativistic proper-time correction
    /// derived from the supplied `LocalSpacetime` model.
    ///
    /// This method is intended for simulation of remote clocks (e.g., Earth time as observed from a spacecraft).
    /// For the spacecraft's own hardware proper-time clock, use the plain `add` method instead.
    #[inline]
    pub fn adjusted_advance(&mut self, elapsed: &TimeSpan, local_spacetime: &LocalSpacetime) {
        let dtau =
            elapsed.add(ClockDrift::from_local_spacetime(local_spacetime).time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Advances this `TimePoint` by the given elapsed duration while applying the relativistic proper-time correction
    /// from a pre-computed `ClockDrift` value.
    ///
    /// This is an optimized variant of `adjusted_advance` for callers that already hold a `ClockDrift` instance.
    /// It is intended for simulation of remote clocks; the spacecraft's own hardware clock should use the plain `add` method.
    #[inline]
    pub fn adjusted_advance_using_drift(&mut self, elapsed: &TimeSpan, drift: &ClockDrift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Computes the signed duration between this `TimePoint` and an earlier instant.
    ///
    /// The duration is always calculated after converting both instants to the TAI timescale internally,
    /// ensuring correctness even when the two `TimePoint`s belong to different clock types.
    pub const fn duration_since(self, earlier: Self) -> TimeSpan {
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

        TimeSpan { sec, subsec }
    }

    /// Computes the signed duration between this `TimePoint` and an earlier instant (by reference).
    ///
    /// The duration is always calculated after converting both instants to the TAI timescale internally,
    /// ensuring correctness even when the two `TimePoint`s belong to different clock types.
    pub const fn duration_since_ref(self, earlier: &Self) -> TimeSpan {
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

        TimeSpan { sec, subsec }
    }

    /// Returns the numerical difference in seconds between this `TimePoint` and another (ignores `ClockType`).
    ///
    /// This method is lossy by design and is provided for testing and debugging purposes only.
    /// For the exact duration, use `duration_since` or `duration_since_ref`.
    pub const fn numerical_seconds_since(&self, other: &Self) -> Real {
        TimeSpan {
            sec: self.sec,
            subsec: self.subsec,
        }
        .as_sec_f()
            - TimeSpan {
                sec: other.sec,
                subsec: other.subsec,
            }
            .as_sec_f()
    }
}
