//! Time increment and decrement methods for `TimePoint` and `TimeSpan`.
use crate::{
    ATTOSEC_PER_ATTOSEC, ATTOSEC_PER_FEMTOSEC, ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC,
    ATTOSEC_PER_NANOSEC, ATTOSEC_PER_PICOSEC, ATTOSEC_PER_SEC, ATTOSEC_PER_SEC_I128, TimePoint,
    TimeSpan,
};

macro_rules! impl_time_inc {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {

                /// Seconds field getter.
                #[inline]
                pub const fn sec(&self) -> i64 {
                    self.sec
                }

                /// Subseconds field getter (attoseconds).
                #[inline]
                pub const fn subsec(&self) -> u64 {
                    self.subsec
                }

                /// Normalizes the representation so that the attosecond part lies in the range `[0, ATTOSEC_PER_SEC)`.
                #[inline]
                pub const fn carry_over(&mut self) -> &mut Self {
                    if self.subsec >= ATTOSEC_PER_SEC {
                        self.sec += (self.subsec / ATTOSEC_PER_SEC) as i64;
                        self.subsec %= ATTOSEC_PER_SEC;
                    }
                    self
                }

                // =====================================================================
                // Single-unit addition methods (convenience methods for +1)
                // =====================================================================

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
                    self._add_subsec(ATTOSEC_PER_MILLISEC);
                }

                /// Adds exactly 1 microsecond to this time value.
                ///
                /// This affects the subsecond component and may cause a carry into the seconds field.
                #[inline]
                pub const fn add_1us(&mut self) {
                    self._add_subsec(ATTOSEC_PER_MICROSEC);
                }

                /// Adds exactly 1 nanosecond to this time value.
                ///
                /// This affects the subsecond component and may cause a carry into the seconds field.
                #[inline]
                pub const fn add_1ns(&mut self) {
                    self._add_subsec(ATTOSEC_PER_NANOSEC);
                }

                // =====================================================================
                // Multi-unit addition methods (saturating)
                // =====================================================================

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
                    self.add_subsec_span(n, ATTOSEC_PER_MILLISEC);
                }

                /// Adds the specified number of microseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_us(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_MICROSEC);
                }

                /// Adds the specified number of nanoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_ns(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_NANOSEC);
                }

                /// Adds the specified number of picoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_ps(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_PICOSEC);
                }

                /// Adds the specified number of femtoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_fs(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_FEMTOSEC);
                }

                /// Adds the specified number of attoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_as(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_ATTOSEC);
                }

                /// Adds the specified number of attoseconds (the internal subsecond unit)
                /// to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline]
                pub const fn add_subsec(&mut self, n: i64) {
                    self.add_subsec_span(n, ATTOSEC_PER_ATTOSEC);
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
                    self.add_subsec_span(-1, ATTOSEC_PER_MILLISEC);
                }

                /// Subtracts exactly 1 microsecond from this time value.
                ///
                /// This affects the subsecond component and may cause a borrow from the seconds field.
                #[inline]
                pub const fn sub_1us(&mut self) {
                    self.add_subsec_span(-1, ATTOSEC_PER_MICROSEC);
                }

                /// Subtracts exactly 1 nanosecond from this time value.
                ///
                /// This affects the subsecond component and may cause a borrow from the seconds field.
                #[inline]
                pub const fn sub_1ns(&mut self) {
                    self.add_subsec_span(-1, ATTOSEC_PER_NANOSEC);
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
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_MILLISEC);
                }

                /// Subtracts the specified number of microseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_us(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_MICROSEC);
                }

                /// Subtracts the specified number of nanoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_ns(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_NANOSEC);
                }

                /// Subtracts the specified number of picoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_ps(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_PICOSEC);
                }

                /// Subtracts the specified number of femtoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_fs(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_FEMTOSEC);
                }

                /// Subtracts the specified number of attoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_as(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_ATTOSEC);
                }

                /// Subtracts the specified number of attoseconds (the internal subsecond unit)
                /// from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline]
                pub const fn sub_subsec(&mut self, n: i64) {
                    self.add_subsec_span(n.saturating_neg(), ATTOSEC_PER_ATTOSEC);
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

                    let mps = ATTOSEC_PER_SEC;

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
                    let carry_sec = total / ATTOSEC_PER_SEC;
                    self.subsec = total % ATTOSEC_PER_SEC;
                    self.sec = self.sec.saturating_add(carry_sec as i64);
                }

                /// Total attoseconds (exact i128 representation within the representable range).
                #[inline]
                pub const fn total_attos(self) -> i128 {
                    (self.sec as i128) * ATTOSEC_PER_SEC_I128 + (self.subsec as i128)
                }

                /// Returns the total duration in milliseconds.
                #[inline]
                pub const fn as_ms(self) -> i128 {
                    self.total_attos() / (ATTOSEC_PER_MILLISEC as i128)
                }

                /// Returns the total duration in microseconds.
                #[inline]
                pub const fn as_us(self) -> i128 {
                    self.total_attos() / (ATTOSEC_PER_MICROSEC as i128)
                }

                /// Returns the total duration in nanoseconds.
                #[inline]
                pub const fn as_ns(self) -> i128 {
                    self.total_attos() / (ATTOSEC_PER_NANOSEC as i128)
                }

                /// Returns the total duration in picoseconds.
                #[inline]
                pub const fn as_ps(self) -> i128 {
                    self.total_attos() / (ATTOSEC_PER_PICOSEC as i128)
                }

                /// Returns the total duration in femtoseconds.
                #[inline]
                pub const fn as_fs(self) -> i128 {
                    self.total_attos() / (ATTOSEC_PER_FEMTOSEC as i128)
                }
            }
        )*
    };
}

// Apply the implementation to both `TimePoint` and `TimeSpan` types.
impl_time_inc!(TimePoint, TimeSpan);
