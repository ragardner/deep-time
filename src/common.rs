use crate::{Delta, Point};

/// Macro that implements time increment and decrement methods for time-related types.
///
/// This macro is applied to both `Point` and `Delta` types, providing a consistent
/// API for adding and subtracting various time units.
///
/// All operations use **saturating arithmetic** to prevent overflow and underflow,
/// ensuring the time values stay within the valid `i128` range for seconds and
/// `u128` range for the subsecond (microquectosecond) component.
macro_rules! impl_time_inc {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                // =====================================================================
                // Single-unit addition methods (convenience methods for +1)
                // =====================================================================

                /// Adds exactly 1 second to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_1sec(&mut self) {
                    self.sec = self.sec.saturating_add(1);
                }

                /// Adds exactly 1 minute (60 seconds) to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_1min(&mut self) {
                    self.sec = self.sec.saturating_add(60);
                }

                /// Adds exactly 1 hour (3600 seconds) to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_1hr(&mut self) {
                    self.sec = self.sec.saturating_add(3600);
                }

                /// Adds exactly 1 millisecond to this time value.
                ///
                /// This affects the subsecond component and may cause a carry into the seconds field.
                #[inline(always)]
                pub fn add_1ms(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_MILLISEC);
                }

                /// Adds exactly 1 microsecond to this time value.
                ///
                /// This affects the subsecond component and may cause a carry into the seconds field.
                #[inline(always)]
                pub fn add_1us(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_MICROSEC);
                }

                /// Adds exactly 1 nanosecond to this time value.
                ///
                /// This affects the subsecond component and may cause a carry into the seconds field.
                #[inline(always)]
                pub fn add_1ns(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_NANOSEC);
                }

                // =====================================================================
                // Multi-unit addition methods (saturating)
                // =====================================================================

                /// Adds the specified number of seconds to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_sec(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n);
                }

                /// Adds the specified number of minutes to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_min(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n.saturating_mul(60));
                }

                /// Adds the specified number of hours to this time value using saturating arithmetic.
                #[inline(always)]
                pub fn add_hr(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n.saturating_mul(3600));
                }

                /// Adds the specified number of milliseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_ms(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MILLISEC);
                }

                /// Adds the specified number of microseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_us(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MICROSEC);
                }

                /// Adds the specified number of nanoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_ns(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_NANOSEC);
                }

                /// Adds the specified number of picoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_ps(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_PICOSEC);
                }

                /// Adds the specified number of femtoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_fs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_FEMTOSEC);
                }

                /// Adds the specified number of attoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_as(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_ATTOSEC);
                }

                /// Adds the specified number of zeptoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_zs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_ZEPTOSEC);
                }

                /// Adds the specified number of yoctoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_ys(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_YOCTOSEC);
                }

                /// Adds the specified number of rontoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_rs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_RONTOSEC);
                }

                /// Adds the specified number of quectoseconds to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_qs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_QUECTOSEC);
                }

                /// Adds the specified number of microquectoseconds (the internal subsecond unit)
                /// to this time value.
                ///
                /// Handles carry into the seconds field using saturating logic.
                #[inline(always)]
                pub fn add_subsec(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MICROQUECTOSEC);
                }

                // =====================================================================
                // Single-unit subtraction methods (convenience methods for -1)
                // =====================================================================

                /// Subtracts exactly 1 hour (3600 seconds) from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_1hr(&mut self) {
                    self.sec = self.sec.saturating_sub(3600);
                }

                /// Subtracts exactly 1 minute (60 seconds) from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_1min(&mut self) {
                    self.sec = self.sec.saturating_sub(60);
                }

                /// Subtracts exactly 1 second from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_1sec(&mut self) {
                    self.sec = self.sec.saturating_sub(1);
                }

                /// Subtracts exactly 1 millisecond from this time value.
                ///
                /// This affects the subsecond component and may cause a borrow from the seconds field.
                #[inline(always)]
                pub fn sub_1ms(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_MILLISEC);
                }

                /// Subtracts exactly 1 microsecond from this time value.
                ///
                /// This affects the subsecond component and may cause a borrow from the seconds field.
                #[inline(always)]
                pub fn sub_1us(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_MICROSEC);
                }

                /// Subtracts exactly 1 nanosecond from this time value.
                ///
                /// This affects the subsecond component and may cause a borrow from the seconds field.
                #[inline(always)]
                pub fn sub_1ns(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_NANOSEC);
                }

                // =====================================================================
                // Multi-unit subtraction methods (saturating)
                // =====================================================================

                /// Subtracts the specified number of seconds from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_sec(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n);
                }

                /// Subtracts the specified number of minutes from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_min(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n.saturating_mul(60));
                }

                /// Subtracts the specified number of hours from this time value using saturating arithmetic.
                #[inline(always)]
                pub fn sub_hr(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n.saturating_mul(3600));
                }

                /// Subtracts the specified number of milliseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_ms(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MILLISEC);
                }

                /// Subtracts the specified number of microseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_us(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MICROSEC);
                }

                /// Subtracts the specified number of nanoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_ns(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_NANOSEC);
                }

                /// Subtracts the specified number of picoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_ps(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_PICOSEC);
                }

                /// Subtracts the specified number of femtoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_fs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_FEMTOSEC);
                }

                /// Subtracts the specified number of attoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_as(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_ATTOSEC);
                }

                /// Subtracts the specified number of zeptoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_zs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_ZEPTOSEC);
                }

                /// Subtracts the specified number of yoctoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_ys(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_YOCTOSEC);
                }

                /// Subtracts the specified number of rontoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_rs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_RONTOSEC);
                }

                /// Subtracts the specified number of quectoseconds from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_qs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_QUECTOSEC);
                }

                /// Subtracts the specified number of microquectoseconds (the internal subsecond unit)
                /// from this time value.
                ///
                /// Handles borrow from the seconds field using saturating logic.
                #[inline(always)]
                pub fn sub_subsec(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MICROQUECTOSEC);
                }

                // =====================================================================
                // Internal helper methods
                // =====================================================================

                /// Internal method to add or subtract a subsecond delta in a given unit.
                ///
                /// This is the core implementation for all subsecond addition and subtraction
                /// operations. It properly handles carry and borrow between the fractional
                /// part (`subsec`) and the whole seconds (`sec`), using saturating arithmetic
                /// throughout.
                ///
                /// # Arguments
                ///
                /// * `n` - The number of units to add (positive) or subtract (negative).
                /// * `unit` - The size of one unit expressed in microquectoseconds.
                #[doc(hidden)]
                #[inline]
                fn add_subsec_delta(&mut self, n: i128, unit: u128) {
                    if n == 0 {
                        return;
                    }

                    let mps = crate::MICROQUECTOS_PER_SEC;

                    if n >= 0 {
                        // Positive direction
                        let amount = (n as u128).saturating_mul(unit);
                        let total = self.subsec.saturating_add(amount);

                        let carry = total / mps;
                        let new_frac = total % mps;

                        let carry_i128 = if carry > i128::MAX as u128 {
                            i128::MAX
                        } else {
                            carry as i128
                        };

                        self.sec = self.sec.saturating_add(carry_i128);
                        self.subsec = new_frac;
                    } else {
                        // Negative direction — safe even for i128::MIN
                        let amount = n.unsigned_abs().saturating_mul(unit);
                        let borrow_sec = amount / mps;
                        let borrow_frac = amount % mps;

                        let borrow_i128 = if borrow_sec > i128::MAX as u128 {
                            i128::MAX
                        } else {
                            borrow_sec as i128
                        };

                        self.sec = self.sec.saturating_sub(borrow_i128);

                        if self.subsec >= borrow_frac {
                            self.subsec -= borrow_frac;
                        } else {
                            self.subsec += mps - borrow_frac;
                            self.sec = self.sec.saturating_sub(1);
                        }
                    }

                    // Final saturation clamp to maintain invariants at extreme values
                    if self.sec == i128::MAX {
                        self.subsec = mps - 1;
                    } else if self.sec == i128::MIN {
                        self.subsec = 0;
                    }
                }

                /// Internal fast-path method for adding a small positive subsecond amount.
                ///
                /// Used by the single-unit `add_1ms`, `add_1us`, and `add_1ns` methods.
                /// This is intentionally simpler and faster than the general `add_subsec_delta`
                /// when the delta is known to be positive and small.
                #[doc(hidden)]
                #[inline]
                fn _add_subsec(&mut self, amount: u128) {
                    let total = self.subsec + amount;
                    let carry_sec = total / crate::MICROQUECTOS_PER_SEC;
                    self.subsec = total % crate::MICROQUECTOS_PER_SEC;
                    self.sec += carry_sec as i128;
                }
            }
        )*
    };
}

// Apply the implementation to both `Point` and `Delta` types.
impl_time_inc!(Point, Delta);
