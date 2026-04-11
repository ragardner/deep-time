use crate::{Delta, Point};

macro_rules! impl_time_inc {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                #[inline(always)]
                pub fn add_1sec(&mut self) {
                    self.sec += 1;
                }

                #[inline(always)]
                pub fn add_1min(&mut self) {
                    self.sec += 60;
                }

                #[inline(always)]
                pub fn add_1hr(&mut self) {
                    self.sec += 3600;
                }

                #[inline(always)]
                pub fn add_1ms(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_MILLISEC);
                }

                #[inline(always)]
                pub fn add_1us(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_MICROSEC);
                }

                #[inline(always)]
                pub fn add_1ns(&mut self) {
                    self._add_subsec(crate::MICROQUECTOS_PER_NANOSEC);
                }

                #[inline(always)]
                pub fn add_sec(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n);
                }

                #[inline(always)]
                pub fn add_min(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n.saturating_mul(60));
                }

                #[inline(always)]
                pub fn add_hr(&mut self, n: i128) {
                    self.sec = self.sec.saturating_add(n.saturating_mul(3600));
                }

                #[inline(always)]
                pub fn add_ms(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MILLISEC);
                }

                #[inline(always)]
                pub fn add_us(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MICROSEC);
                }

                #[inline(always)]
                pub fn add_ns(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_NANOSEC);
                }

                #[inline(always)]
                pub fn add_ps(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_PICOSEC);
                }

                #[inline(always)]
                pub fn add_fs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_FEMTOSEC);
                }

                #[inline(always)]
                pub fn add_as(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_ATTOSEC);
                }

                #[inline(always)]
                pub fn add_zs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_ZEPTOSEC);
                }

                #[inline(always)]
                pub fn add_ys(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_YOCTOSEC);
                }

                #[inline(always)]
                pub fn add_rs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_RONTOSEC);
                }

                #[inline(always)]
                pub fn add_qs(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_QUECTOSEC);
                }

                #[inline(always)]
                pub fn add_subsec(&mut self, n: i128) {
                    self.add_subsec_delta(n, crate::MICROQUECTOS_PER_MICROQUECTOSEC);
                }

                #[inline(always)]
                pub fn sub_1hr(&mut self) {
                    self.sec = self.sec.saturating_sub(3600);
                }

                #[inline(always)]
                pub fn sub_1min(&mut self) {
                    self.sec = self.sec.saturating_sub(60);
                }

                #[inline(always)]
                pub fn sub_1sec(&mut self) {
                    self.sec = self.sec.saturating_sub(1);
                }

                #[inline(always)]
                pub fn sub_1ms(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_MILLISEC);
                }

                #[inline(always)]
                pub fn sub_1us(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_MICROSEC);
                }

                #[inline(always)]
                pub fn sub_1ns(&mut self) {
                    self.add_subsec_delta(-1, crate::MICROQUECTOS_PER_NANOSEC);
                }

                #[inline(always)]
                pub fn sub_sec(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n);
                }

                #[inline(always)]
                pub fn sub_min(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n.saturating_mul(60));
                }

                #[inline(always)]
                pub fn sub_hr(&mut self, n: i128) {
                    self.sec = self.sec.saturating_sub(n.saturating_mul(3600));
                }

                #[inline(always)]
                pub fn sub_ms(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MILLISEC);
                }

                #[inline(always)]
                pub fn sub_us(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MICROSEC);
                }

                #[inline(always)]
                pub fn sub_ns(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_NANOSEC);
                }

                #[inline(always)]
                pub fn sub_ps(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_PICOSEC);
                }

                #[inline(always)]
                pub fn sub_fs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_FEMTOSEC);
                }

                #[inline(always)]
                pub fn sub_as(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_ATTOSEC);
                }

                #[inline(always)]
                pub fn sub_zs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_ZEPTOSEC);
                }

                #[inline(always)]
                pub fn sub_ys(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_YOCTOSEC);
                }

                #[inline(always)]
                pub fn sub_rs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_RONTOSEC);
                }

                #[inline(always)]
                pub fn sub_qs(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_QUECTOSEC);
                }

                #[inline(always)]
                pub fn sub_subsec(&mut self, n: i128) {
                    self.add_subsec_delta(n.saturating_neg(), crate::MICROQUECTOS_PER_MICROQUECTOSEC);
                }

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

                    // Final saturation clamp (covers all edge cases including fractional borrow/carry)
                    if self.sec == i128::MAX {
                        self.subsec = mps - 1;
                    } else if self.sec == i128::MIN {
                        self.subsec = 0;
                    }
                }

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

impl_time_inc!(Point, Delta);
