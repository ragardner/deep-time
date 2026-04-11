use crate::leap_seconds::leap_seconds_before;
use crate::{Delta, MICROQUECTOS_PER_SEC, Point, TT_TAI_OFFSET_DELTA, TimePov, sin_approx};

impl Point {
    /// Converts this instant to any other [`TimePov`], representing the exact same physical moment.
    #[inline]
    pub const fn to_pov(self, target: TimePov) -> Self {
        if (self.pov as u8) == (target as u8) {
            return self;
        }
        let tai = self.to_tai();
        tai.from_tai(target)
    }

    /// Returns a copy of this `Point` with a new [`TimePov`] while keeping the exact same
    /// numerical seconds and subseconds value. This is zero-cost after conversion.
    #[inline]
    pub(crate) const fn with_pov(self, pov: TimePov) -> Self {
        Self {
            sec: self.sec,
            subsec: self.subsec,
            pov,
        }
    }

    /// Converts this `Point` (in any scale) to TAI — the library’s internal canonical time scale.
    pub const fn to_tai(self) -> Self {
        match self.pov {
            TimePov::TAI => self,
            TimePov::TT | TimePov::ET => self.sub(TT_TAI_OFFSET_DELTA).with_pov(TimePov::TAI),
            TimePov::UTC => Self::utc_to_tai(self),
            TimePov::GPST | TimePov::QZSST | TimePov::GST => {
                self.add(Delta::from_sec(19)).with_pov(TimePov::TAI)
            }
            TimePov::BDT => self.add(Delta::from_sec(33)).with_pov(TimePov::TAI),
            TimePov::TDB => Self::tdb_to_tai(self),
            TimePov::Proper | TimePov::Custom => self,
        }
    }

    /// Converts a TAI `Point` to any other requested [`TimePov`].
    pub const fn from_tai(self, target: TimePov) -> Self {
        match target {
            TimePov::TAI => self,
            TimePov::TT | TimePov::ET => self.add(TT_TAI_OFFSET_DELTA).with_pov(target),
            TimePov::UTC => Self::tai_to_utc(self),
            TimePov::GPST | TimePov::QZSST | TimePov::GST => {
                self.sub(Delta::from_sec(19)).with_pov(target)
            }
            TimePov::BDT => self.sub(Delta::from_sec(33)).with_pov(target),
            TimePov::TDB => Self::tai_to_tdb(self),
            TimePov::Proper | TimePov::Custom => self.with_pov(target),
        }
    }

    // ──────────────────────────────────────────────────────────────
    // Private UTC ↔ TAI conversion (leap seconds)
    // ──────────────────────────────────────────────────────────────

    const fn utc_to_tai(utc: Self) -> Self {
        let approx_tai_for_lookup = utc.add(Delta::from_sec(37));
        let leaps = leap_seconds_before(approx_tai_for_lookup);
        utc.add(Delta::from_sec(leaps)).with_pov(TimePov::TAI)
    }

    const fn tai_to_utc(tai: Self) -> Self {
        let leaps = leap_seconds_before(tai);
        tai.sub(Delta::from_sec(leaps)).with_pov(TimePov::UTC)
    }

    // ──────────────────────────────────────────────────────────────
    // Private TDB conversion helpers
    // ──────────────────────────────────────────────────────────────

    const fn tdb_minus_tt(tt: Self) -> Delta {
        let seconds_since_j2000_tt =
            (tt.sec as f64) + (tt.subsec as f64 / MICROQUECTOS_PER_SEC as f64);
        let t = seconds_since_j2000_tt / 3_155_760_000.0;
        let g = 2.0 * core::f64::consts::PI * (357.528 + 35_999.050 * t) / 360.0;
        let sin_g = sin_approx(g + 0.0167 * sin_approx(g));
        let sin_2g = sin_approx(2.0 * g);
        let correction = 0.001658 * sin_g + 0.000022 * sin_2g;
        Delta::from_sec_f64(correction)
    }

    const fn tai_to_tdb(tai: Self) -> Self {
        let tt = tai.add(TT_TAI_OFFSET_DELTA).with_pov(TimePov::TT);
        let delta = Self::tdb_minus_tt(tt);
        tt.add(delta).with_pov(TimePov::TDB)
    }

    const fn tdb_to_tai(tdb: Self) -> Self {
        let mut tt = tdb.with_pov(TimePov::TT);
        let mut i = 0u32;

        while i < 8 {
            let delta = Self::tdb_minus_tt(tt);
            tt = tdb.with_pov(TimePov::TT).sub(delta);
            i += 1;
        }

        tt.sub(TT_TAI_OFFSET_DELTA).with_pov(TimePov::TAI)
    }

    // ──────────────────────────────────────────────────────────────
    // Julian Date & Modified Julian Date (TT scale)
    // ──────────────────────────────────────────────────────────────

    /// Returns the standard Julian Date in Terrestrial Time (TT) as `f64`.
    ///
    /// J2000.0 TT corresponds to JD 2451545.0 exactly (the convention used by Astropy, SPICE, NASA, etc.).
    #[inline]
    pub const fn to_jd_tt(self) -> f64 {
        let tt = self.to_pov(TimePov::TT);
        let seconds = tt.sec as f64 + (tt.subsec as f64 / MICROQUECTOS_PER_SEC as f64);
        2451545.0 + seconds / 86400.0
    }

    /// Returns the standard Modified Julian Date in Terrestrial Time (TT) as `f64`.
    ///
    /// J2000.0 TT corresponds to MJD 51544.5 exactly.
    #[inline]
    pub const fn to_mjd_tt(self) -> f64 {
        self.to_jd_tt() - 2_400_000.5
    }

    /// Returns an **exact** Julian Date in TT with full library precision.
    ///
    /// The returned tuple is `(jd_integer_days, fractional_day)` where the fractional part
    /// is a [`Delta`] representing the part of a day (always < 1 day).
    #[inline]
    pub const fn to_jd_tt_exact(self) -> (i128, Delta) {
        let tt = self.to_pov(TimePov::TT);
        let days = tt.sec / 86_400;
        let remaining_sec = tt.sec % 86_400;
        let frac = Delta::new(remaining_sec, tt.subsec);
        (2451545 + days, frac)
    }

    /// Returns an **exact** Modified Julian Date in TT with full library precision.
    #[inline]
    pub const fn to_mjd_tt_exact(self) -> (i128, Delta) {
        let (jd, frac) = self.to_jd_tt_exact();
        (jd - 2_400_000, frac)
    }

    /// Creates a `Point` from an exact Julian Date in TT (full precision, no loss).
    #[inline]
    pub const fn from_jd_tt_exact(jd_days: i128, frac: Delta) -> Self {
        let total_sec = jd_days * 86_400 + frac.sec;
        let tt = Point::new(total_sec, frac.subsec, TimePov::TT);
        tt.to_tai()
    }

    /// Creates a `Point` from an exact Modified Julian Date in TT.
    #[inline]
    pub const fn from_mjd_tt_exact(mjd_days: i128, frac: Delta) -> Self {
        Self::from_jd_tt_exact(mjd_days + 2_400_000, frac)
    }

    /// Convenience method: Julian Date in UTC (TT-based, f64 only).
    #[inline]
    pub const fn to_jd_utc(self) -> f64 {
        self.to_pov(TimePov::UTC).to_jd_tt()
    }

    /// Convenience method: Modified Julian Date in UTC (TT-based, f64 only).
    #[inline]
    pub const fn to_mjd_utc(self) -> f64 {
        self.to_pov(TimePov::UTC).to_mjd_tt()
    }

    /// Returns the numerical difference in seconds between two `Point`s (ignores `TimePov`).
    ///
    /// Used primarily for testing scale-offset accuracy.
    pub const fn numerical_seconds_since(&self, other: &Self) -> f64 {
        let self_s = self.sec as f64 + (self.subsec as f64 / MICROQUECTOS_PER_SEC as f64);
        let other_s = other.sec as f64 + (other.subsec as f64 / MICROQUECTOS_PER_SEC as f64);
        self_s - other_s
    }
}

#[cfg(test)]
mod tdb_tests {
    use super::*;
    use crate::TimePov;

    /// Round-trip accuracy test (TAI → TDB → TAI)
    #[test]
    fn tdb_tai_roundtrip_is_accurate() {
        let test_points = [
            Point::from_tai_sec(0),                  // J2000 TAI
            Point::from_tai_sec(86_400 * 365),       // ~1 year later
            Point::from_tai_sec(-86_400 * 365 * 10), // 10 years before
            Point::from_tai_sec(1_000_000_000),      // ~31.7 years later
            Point::from_tai_sec(-2_208_945_600),     // J1900 epoch
        ];

        #[cfg(feature = "std")]
        {
            use std::eprintln;
            let tai = Point::ZERO;
            let tdb = tai.to_pov(TimePov::TDB);
            eprintln!("\nTAI sec={}, subsec={}", tai.sec, tai.subsec);
            eprintln!("TDB sec={}, subsec={}", tdb.sec, tdb.subsec);
            eprintln!("diff_s = {}", tdb.duration_since(tai).as_sec_f64());
        }
        for &p in &test_points {
            let tdb = p.to_pov(TimePov::TDB);
            let back = tdb.to_pov(TimePov::TAI);

            let diff = back.duration_since(p).as_sec_f64().abs();
            assert!(
                diff < 1e-6,
                "TDB round-trip error too large: {} s at {:?}",
                diff,
                p
            );
        }
    }

    /// At J2000 the TDB–TAI difference should be ~32.183925 s
    /// (TT = TAI + 32.184 s and TDB − TT ≈ −74.6 µs with this formula)
    #[test]
    fn tdb_minus_tt_at_j2000() {
        let tai = Point::ZERO;
        let tdb = tai.to_pov(TimePov::TDB);

        let diff_s = tdb.numerical_seconds_since(&tai); // see helper below

        assert!(
            (diff_s - 32.183925).abs() < 0.00001,
            "TDB-TAI difference at J2000 was {} s (expected ~32.183925 s)",
            diff_s
        );
    }

    #[test]
    fn tdb_minus_tt_at_j2000_2() {
        let tai = Point::ZERO;
        let tdb = tai.to_pov(TimePov::TDB);
        let diff_s = tdb.numerical_seconds_since(&tai);
        assert!((diff_s - 32.183925).abs() < 1e-6, "got {}", diff_s);
    }

    /// Check that the *periodic correction* (TDB − TT) stays within sensible bounds
    #[test]
    fn tdb_correction_stays_within_bounds() {
        let points = [
            Point::from_tai_sec(0),
            Point::from_tai_sec(86_400 * 365 * 100),
            Point::from_tai_sec(-86_400 * 365 * 50),
        ];

        for &p in &points {
            let tt = p.to_pov(TimePov::TT);
            let tdb = p.to_pov(TimePov::TDB);

            // TDB - TT (periodic term only)
            let corr_s = tdb.numerical_seconds_since(&tt);

            assert!(
                corr_s.abs() < 0.002,
                "TDB-TT correction should be < 2 ms (got {} s)",
                corr_s
            );
        }
    }
}
