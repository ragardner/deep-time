use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US, ClockDrift, LocalSpacetime, Real, TimePoint, TimeSpan,
};

impl TimePoint {
    /// Converts this `TimePoint` to a floating-point number of seconds since the reference epoch of its associated clock type.
    ///
    /// The conversion is lossy by design, as `f64` (`Real`) provides approximately 15.95 decimal digits of precision.
    /// For full exactness, use the integer components `sec` and `subsec` directly or higher-precision arithmetic when available.
    #[inline]
    pub const fn to_sec_f(self) -> Real {
        f!(self.sec) + f!(self.subsec) / ATTOS_PER_SECF
    }

    /// Performs a saturating addition of any `TimeSpan` (positive or negative) to this `TimePoint`.
    ///
    /// Adding a negative `TimeSpan` moves the time point backward in time.
    /// The resulting `TimePoint` retains the original [`ClockType`] and saturates at the
    /// representable extremes (`i64::MIN` / `i64::MAX`) rather than wrapping.
    pub const fn add(self, span: TimeSpan) -> Self {
        let mut sec = self.sec.saturating_add(span.sec);
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
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
    pub const fn sub(self, span: TimeSpan) -> Self {
        let mut sec = self.sec.saturating_sub(span.sec);
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        } else if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        }

        let subsec = if sec == i64::MAX {
            ATTOS_PER_SEC - 1
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
    pub const fn mut_add(&mut self, span: TimeSpan) -> &mut Self {
        let mut sec = self.sec.saturating_add(span.sec);
        let mut subsec = self.subsec as i64 + span.subsec as i64;

        if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        } else if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        }

        self.sec = if sec == i64::MAX {
            i64::MAX
        } else if sec == i64::MIN {
            i64::MIN
        } else {
            sec
        };

        self.subsec = if self.sec == i64::MAX {
            ATTOS_PER_SEC - 1
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
    pub const fn mut_sub(&mut self, span: TimeSpan) -> &mut Self {
        let mut sec = self.sec.saturating_sub(span.sec);
        let mut subsec = self.subsec as i64 - span.subsec as i64;

        if subsec < 0 {
            if sec > i64::MIN {
                sec = sec.saturating_sub(1);
            }
            subsec += ATTOS_PER_SEC as i64;
        } else if subsec >= ATTOS_PER_SEC as i64 {
            if sec < i64::MAX {
                sec = sec.saturating_add(1);
            }
            subsec -= ATTOS_PER_SEC as i64;
        }

        self.sec = if sec == i64::MAX {
            i64::MAX
        } else if sec == i64::MIN {
            i64::MIN
        } else {
            sec
        };

        self.subsec = if self.sec == i64::MAX {
            ATTOS_PER_SEC - 1
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
    pub const fn adjusted_advance(&mut self, elapsed: &TimeSpan, local_spacetime: &LocalSpacetime) {
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
    pub const fn adjusted_advance_using_drift(&mut self, elapsed: &TimeSpan, drift: &ClockDrift) {
        let dtau = elapsed.add(drift.time_diff_after(elapsed));
        *self = self.add(dtau);
    }

    /// Computes the TAI signed duration between this `TimePoint` and an earlier instant.
    #[inline]
    pub const fn to_tai_since(self, earlier: Self) -> TimeSpan {
        TimeSpan::diff_raw(self.sec, self.subsec, earlier.sec, earlier.subsec)
    }

    /// Computes the signed duration between this `TimePoint` and an earlier instant (by reference).
    ///
    /// The duration is always calculated after converting both instants to the TAI timescale internally,
    /// ensuring correctness even when the two `TimePoint`s belong to different clock types.
    #[inline]
    pub const fn to_tai_since_ref(self, earlier: &Self) -> TimeSpan {
        TimeSpan::diff_raw(self.sec, self.subsec, earlier.sec, earlier.subsec)
    }

    /// Returns the numerical difference in seconds between this `TimePoint` and another (ignores `ClockType`).
    ///
    /// This method is lossy by design and is provided for testing and debugging purposes only.
    /// For the exact duration, use `duration_since` or `duration_since_ref`.
    #[inline]
    pub const fn to_tai_since_f(&self, other: &Self) -> Real {
        self.to().to_sec_f() - other.to().to_sec_f()
    }

    /// Adds exactly 1 second to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1sec(&mut self) {
        self.sec = self.sec.saturating_add(1);
    }

    /// Adds exactly 1 minute (60 seconds) to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1min(&mut self) {
        self.sec = self.sec.saturating_add(60);
    }

    /// Adds exactly 1 hour (3600 seconds) to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_1hr(&mut self) {
        self.sec = self.sec.saturating_add(3600);
    }

    /// Adds exactly 1 millisecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1ms(&mut self) {
        self._add_subsec(ATTOS_PER_MS);
    }

    /// Adds exactly 1 microsecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1us(&mut self) {
        self._add_subsec(ATTOS_PER_US);
    }

    /// Adds exactly 1 nanosecond to this time value.
    ///
    /// This affects the subsecond component and may cause a carry into the seconds field.
    #[inline]
    pub const fn add_1ns(&mut self) {
        self._add_subsec(ATTOS_PER_NS);
    }

    /// Adds the specified number of seconds to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_sec(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n);
    }

    /// Adds the specified number of minutes to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_min(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n.saturating_mul(60));
    }

    /// Adds the specified number of hours to this time value using saturating arithmetic.
    #[inline]
    pub const fn add_hr(&mut self, n: i64) {
        self.sec = self.sec.saturating_add(n.saturating_mul(3600));
    }

    /// Adds the specified number of milliseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ms(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_MS);
    }

    /// Adds the specified number of microseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_us(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_US);
    }

    /// Adds the specified number of nanoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ns(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_NS);
    }

    /// Adds the specified number of picoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_ps(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_PS);
    }

    /// Adds the specified number of femtoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_fs(&mut self, n: i64) {
        self.add_subsec_span(n, ATTOS_PER_FS);
    }

    /// Adds the specified number of attoseconds to this time value.
    ///
    /// Handles carry into the seconds field using saturating logic.
    #[inline]
    pub const fn add_attos(&mut self, n: i64) {
        self.add_subsec_span(n, 1);
    }

    // =====================================================================
    // Single-unit subtraction methods (convenience methods for -1)
    // =====================================================================

    /// Subtracts exactly 1 hour (3600 seconds) from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1hr(&mut self) {
        self.sec = self.sec.saturating_sub(3600);
    }

    /// Subtracts exactly 1 minute (60 seconds) from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1min(&mut self) {
        self.sec = self.sec.saturating_sub(60);
    }

    /// Subtracts exactly 1 second from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_1sec(&mut self) {
        self.sec = self.sec.saturating_sub(1);
    }

    /// Subtracts exactly 1 millisecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1ms(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_MS);
    }

    /// Subtracts exactly 1 microsecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1us(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_US);
    }

    /// Subtracts exactly 1 nanosecond from this time value.
    ///
    /// This affects the subsecond component and may cause a borrow from the seconds field.
    #[inline]
    pub const fn sub_1ns(&mut self) {
        self.add_subsec_span(-1, ATTOS_PER_NS);
    }

    // =====================================================================
    // Multi-unit subtraction methods (saturating)
    // =====================================================================

    /// Subtracts the specified number of seconds from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_sec(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n);
    }

    /// Subtracts the specified number of minutes from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_min(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n.saturating_mul(60));
    }

    /// Subtracts the specified number of hours from this time value using saturating arithmetic.
    #[inline]
    pub const fn sub_hr(&mut self, n: i64) {
        self.sec = self.sec.saturating_sub(n.saturating_mul(3600));
    }

    /// Subtracts the specified number of milliseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ms(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_MS);
    }

    /// Subtracts the specified number of microseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_us(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_US);
    }

    /// Subtracts the specified number of nanoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ns(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_NS);
    }

    /// Subtracts the specified number of picoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_ps(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_PS);
    }

    /// Subtracts the specified number of femtoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_fs(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), ATTOS_PER_FS);
    }

    /// Subtracts the specified number of attoseconds from this time value.
    ///
    /// Handles borrow from the seconds field using saturating logic.
    #[inline]
    pub const fn sub_attos(&mut self, n: i64) {
        self.add_subsec_span(n.saturating_neg(), 1);
    }

    // =====================================================================
    // Internal helper methods
    // =====================================================================

    /// Internal method to add or subtract a subsecond span in a given unit.
    ///
    /// This is the core implementation for all subsecond addition and subtraction
    /// operations. It properly handles carry and borrow between the fractional
    /// part (`subsec`) and the whole seconds (`sec`), using saturating arithmetic
    /// throughout.
    #[doc(hidden)]
    const fn add_subsec_span(&mut self, n: i64, unit: u64) {
        if n == 0 {
            return;
        }

        let mps = ATTOS_PER_SEC;

        if n >= 0 {
            // Positive direction
            let amount = (n as u64).saturating_mul(unit);
            let total = self.subsec.saturating_add(amount);

            let carry = total / mps;
            let new_frac = total % mps;

            self.sec = self.sec.saturating_add(carry as i64);
            self.subsec = new_frac;
        } else {
            // Negative direction
            let amount = n.unsigned_abs().saturating_mul(unit);
            let borrow_sec = amount / mps;
            let borrow_frac = amount % mps;

            self.sec = self.sec.saturating_sub(borrow_sec as i64);

            if self.subsec >= borrow_frac {
                self.subsec -= borrow_frac;
            } else {
                self.subsec += mps - borrow_frac;
                self.sec = self.sec.saturating_sub(1);
            }
        }

        // Final saturation clamp to maintain invariants at extreme values
        if self.sec == i64::MAX {
            self.subsec = mps - 1;
        } else if self.sec == i64::MIN {
            self.subsec = 0;
        }
    }

    /// Internal fast-path method for adding a small positive subsecond amount.
    ///
    /// Used by the single-unit `add_1ms`, `add_1us`, and `add_1ns` methods.
    /// This is intentionally simpler and faster than the general `add_subsec_span`
    /// when the span is known to be positive and small.
    #[doc(hidden)]
    #[inline]
    const fn _add_subsec(&mut self, amount: u64) {
        let total = self.subsec + amount;
        let carry_sec = total / ATTOS_PER_SEC;
        self.subsec = total % ATTOS_PER_SEC;
        self.sec = self.sec.saturating_add(carry_sec as i64);
    }

    /// Total attoseconds (exact i128 representation within the representable range).
    #[inline]
    pub const fn to_attos(self) -> i128 {
        (self.sec as i128) * ATTOS_PER_SEC_I128 + (self.subsec as i128)
    }

    /// Returns the total duration in milliseconds.
    #[inline]
    pub const fn to_ms(self) -> i128 {
        self.to_attos() / (ATTOS_PER_MS as i128)
    }

    /// Returns the total duration in microseconds.
    #[inline]
    pub const fn to_us(self) -> i128 {
        self.to_attos() / (ATTOS_PER_US as i128)
    }

    /// Returns the total duration in nanoseconds.
    #[inline]
    pub const fn to_ns(self) -> i128 {
        self.to_attos() / (ATTOS_PER_NS as i128)
    }

    /// Returns the total duration in picoseconds.
    #[inline]
    pub const fn to_ps(self) -> i128 {
        self.to_attos() / (ATTOS_PER_PS as i128)
    }

    /// Returns the total duration in femtoseconds.
    #[inline]
    pub const fn to_fs(self) -> i128 {
        self.to_attos() / (ATTOS_PER_FS as i128)
    }
}
