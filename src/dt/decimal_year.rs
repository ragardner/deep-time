use crate::{Dt, JD_2000_2_451_545F, Real, Scale};

impl Dt {
    /// Returns the **Julian epoch year**.
    #[inline]
    pub const fn to_jyear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(2000.0) + (jd_tt - JD_2000_2_451_545F) / f!(365.25)
    }

    /// Inverse of [`Self::to_jyear`].
    #[inline]
    pub const fn from_jyear(jyear: Real, scale: Scale) -> Dt {
        if jyear.is_nan() {
            return Self::ZERO;
        }
        if jyear.is_infinite() {
            return if jyear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = JD_2000_2_451_545F + (jyear - f!(2000.0)) * f!(365.25);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **Besselian epoch year**.
    #[inline]
    pub const fn to_byear(&self) -> Real {
        let jd_tt = self.to_jd_f();
        f!(1900.0) + (jd_tt - f!(2415020.31352)) / f!(365.242198781)
    }

    /// Inverse of [`Self::to_byear`].
    #[inline]
    pub const fn from_byear(byear: Real, scale: Scale) -> Dt {
        if byear.is_nan() {
            return Self::ZERO;
        }
        if byear.is_infinite() {
            return if byear.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let jd = f!(2415020.31352) + (byear - f!(1900.0)) * f!(365.242198781);
        Self::from_jd_f(jd, scale)
    }

    /// Returns the **decimal year** (Gregorian calendar year + fraction of the year).
    ///
    /// This is the direct equivalent of Astropy’s `Time.decimalyear`:
    /// - Uses the *actual* length of the specific Gregorian year (365 or 366 days,
    ///   plus any leap seconds on UTC/UtcSpice/etc.).
    /// - Fully scale-aware (TAI, TT, UTC, TDB, custom clocks, …).
    /// - Exact integer arithmetic for the year boundaries, then a high-precision
    ///   `to_sec_f` division (lossy only at the final `Real` step, same as Astropy).
    #[inline]
    pub fn to_decimalyear(&self) -> Real {
        let ymd = self.to_ymd();
        let year = ymd.yr;

        let start = Self::from_ymd(year, 1, 1, 0, 0, 0, 0, self.target);
        let next_start = Self::from_ymd(year + 1, 1, 1, 0, 0, 0, 0, self.target);

        let elapsed = self.to_diff_raw(start).to_sec_f();
        let year_length = next_start.to_diff_raw(start).to_sec_f();

        // year_length is never zero for representable years
        f!(year) + elapsed / year_length
    }
}
