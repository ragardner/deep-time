use crate::leap_seconds::leap_seconds_before;
use crate::{
    Delta, LB, LG, POW15, POW21, Point, TCG_TCB_REF_JD, TDB0, TT_TAI_OFFSET_DELTA, TimePoly,
    TimePolyScale, TimePov, sin_approx,
};

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
            TimePov::TCG => Self::tcg_to_tai(self),
            TimePov::TCB => Self::tcb_to_tai(self),
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
            TimePov::TCG => Self::tai_to_tcg(self),
            TimePov::TCB => Self::tai_to_tcb(self),
            TimePov::Proper | TimePov::Custom => self.with_pov(target),
        }
    }

    /// Converts this instant to any other [`TimePov`] while applying an exact quadratic
    /// relativistic / clock-drift correction via a [`TimePoly`].
    ///
    /// **Primary use for spacecraft/probes**:
    /// - `self` is normally `TimePov::Proper` (onboard clock reading).
    /// - The polynomial models **target = reference + poly(dt)**.
    #[inline]
    pub const fn to_pov_with_poly(self, target: TimePov, reference: Self, poly: TimePoly) -> Self {
        let dt = self.duration_since(reference);
        let correction = poly.evaluate(dt);
        self.add(correction).with_pov(target)
    }

    /// Inverse of `to_pov_with_poly`.
    ///
    /// Uses fixed-point iteration (exactly like the existing `tdb_to_tai` implementation).
    /// Because relativistic / clock-drift rates are always tiny (|rate| ≪ 1),
    /// this recovers the original proper time to full 36-digit precision.
    #[inline]
    pub const fn from_pov_with_poly(
        self, // point *in the target scale*
        source: TimePov,
        reference: Self,
        poly: TimePoly,
    ) -> Self {
        // Fast path for the extremely common pure-constant case
        if poly.rate.is_zero() && poly.accel.is_zero() {
            return self.sub(poly.constant).with_pov(source);
        }

        let mut guess = self;
        let mut i = 0u32;
        while i < 16 {
            // ← changed from 8
            let dt = guess.duration_since(reference);
            let correction = poly.evaluate(dt);
            guess = self.sub(correction); // target - poly(guess - ref)
            i += 1;
        }
        guess.with_pov(source)
    }

    /// Converts using a self-describing [`TimePolyScale`].
    ///
    /// Onboard `Proper`/`Custom` → whatever `scale.base` is (TT, TDB, Custom/Mars time, etc.).
    /// This is the recommended one-line conversion and is now fully flexible.
    #[inline]
    pub const fn to_pov_with_scale(self, scale: TimePolyScale) -> Self {
        self.to_pov_with_poly(scale.base, scale.reference, scale.poly)
    }

    /// Inverse of `to_pov_with_scale`.
    #[inline]
    pub const fn from_pov_with_scale(self, scale: TimePolyScale) -> Self {
        self.from_pov_with_poly(scale.base, scale.reference, scale.poly)
    }

    /// Creates a `Point` from a fully self-describing [`TimePolyScale`].
    ///
    /// This is the recommended way for spacecraft/probes to represent
    /// onboard proper time that already carries its own relativistic model.
    #[inline]
    pub const fn from_scale(scale: TimePolyScale) -> Self {
        scale.reference.with_pov(scale.base)
    }

    /// Replaces the current POV with the base POV of a fully self-describing scale.
    ///
    /// This is the most common operation on a spacecraft: you have a raw `Proper`
    /// reading and you just received a new polynomial update from ground.
    #[inline]
    pub const fn with_scale(self, scale: TimePolyScale) -> Self {
        self.with_pov(scale.base)
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
        // J2000.0 = 2000-01-01 12:00:00 TT → 100 Julian years = exactly 3_155_760_000 s
        const J2000_SECONDS_PER_CENTURY: f64 = 3_155_760_000.0;

        // Whole seconds as f64 (limited by f64 integer precision above ~9e15 s)
        let whole = tt.sec as f64;

        let q = tt.subsec / POW21; // integer < 10¹⁵, exact
        let frac = (q as f64) / (POW15 as f64);

        let seconds_since_j2000_tt = whole + frac;

        let t = seconds_since_j2000_tt / J2000_SECONDS_PER_CENTURY;

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
    // Private TCG/TCB helpers (linear rate conversions)
    // ──────────────────────────────────────────────────────────────

    const fn tcg_to_tai(tcg: Self) -> Self {
        let tt = Self::tcg_to_tt(tcg);
        tt.to_tai()
    }

    const fn tai_to_tcg(tai: Self) -> Self {
        let tt = tai.from_tai(TimePov::TT);
        Self::tt_to_tcg(tt)
    }

    const fn tcb_to_tai(tcb: Self) -> Self {
        let tdb = Self::tcb_to_tdb(tcb);
        tdb.to_tai()
    }

    const fn tai_to_tcb(tai: Self) -> Self {
        let tdb = tai.from_tai(TimePov::TDB);
        Self::tdb_to_tcb(tdb)
    }

    /// TCG ↔ TT (exact IAU linear relation)
    const fn tt_to_tcg(tt: Self) -> Self {
        let jd_tt = tt.to_jd_tt();
        let days = jd_tt - TCG_TCB_REF_JD;
        let delta_s = days * 86_400.0 * LG;
        tt.add(Delta::from_sec_f64(delta_s)).with_pov(TimePov::TCG)
    }

    const fn tcg_to_tt(tcg: Self) -> Self {
        let jd_tcg = tcg.to_jd_tt();
        let days = jd_tcg - TCG_TCB_REF_JD;
        let delta_s = days * 86_400.0 * LG;
        tcg.sub(Delta::from_sec_f64(delta_s)).with_pov(TimePov::TT)
    }

    /// TCB ↔ TDB (exact IAU 2006 linear relation)
    const fn tdb_to_tcb(tdb: Self) -> Self {
        let jd_tdb = tdb.to_jd_tt();
        let days = jd_tdb - TCG_TCB_REF_JD;
        let delta_s = days * 86_400.0 * LB;
        tdb.add(Delta::from_sec_f64(delta_s))
            .add(TDB0) // TDB0 is already part of the defining relation
            .with_pov(TimePov::TCB)
    }

    const fn tcb_to_tdb(tcb: Self) -> Self {
        let jd_tcb = tcb.to_jd_tt();
        let days = jd_tcb - TCG_TCB_REF_JD;
        let delta_s = days * 86_400.0 * LB;
        tcb.sub(Delta::from_sec_f64(delta_s))
            .sub(TDB0)
            .with_pov(TimePov::TDB)
    }

    // ──────────────────────────────────────────────────────────────
    // Julian Date & Modified Julian Date (TT scale)
    // ──────────────────────────────────────────────────────────────

    /// Returns the standard Julian Date in Terrestrial Time (TT) as `f64`.
    ///
    /// J2000.0 TT corresponds to JD 2451545.0 exactly (Astropy/SPICE/NASA convention).
    ///
    /// **Lossy by design** — uses the best possible `f64` conversion of the exact
    /// fractional day. For full precision use `to_jd_tt_exact()` (returns `(i128, Delta)`).
    #[inline]
    pub const fn to_jd_tt(self) -> f64 {
        let (jd_days, frac) = self.to_jd_tt_exact();
        let days_f = jd_days as f64;
        let frac_days = frac.as_sec_f64() / 86_400.0; // 86400.0 is exact in f64
        days_f + frac_days
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
    /// **Lossy by design** (for testing only). Use `duration_since` for the exact `Delta`.
    pub const fn numerical_seconds_since(&self, other: &Self) -> f64 {
        Delta {
            sec: self.sec,
            subsec: self.subsec,
        }
        .as_sec_f64()
            - Delta {
                sec: other.sec,
                subsec: other.subsec,
            }
            .as_sec_f64()
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

#[cfg(test)]
mod poly_tests {
    use super::*;
    use crate::{Delta, TimePoly, TimePolyScale, TimePov};

    #[test]
    fn proper_to_tt_with_poly_roundtrip() {
        let reference = Point::from_tai_sec(0);
        let poly = TimePoly::new(
            Delta::from_ms(100), // exactly 0.1 s
            Delta::from_ns(1),   // exactly 1 ns/s = 1e-9 s/s
            Delta::ZERO,
        );
        let scale = TimePolyScale::proper(reference, poly);

        let onboard_proper = Point::from_scale(scale).add(Delta::from_sec(1_000_000));

        let tt = onboard_proper.to_pov_with_scale(scale);
        let back = tt.from_pov_with_scale(scale);

        assert_eq!(back, onboard_proper);
    }

    #[test]
    fn zero_poly_is_identity() {
        let reference = Point::from_tai_sec(0);
        let poly = TimePoly::ZERO;
        let scale = TimePolyScale::proper(reference, poly);

        let p = Point::from_tai_sec(1_234_567);
        let converted = p.to_pov_with_scale(scale);

        assert_eq!(converted, p.with_pov(TimePov::Proper));
    }

    #[test]
    fn constant_offset_only() {
        let reference = Point::from_tai_sec(0);
        let poly = TimePoly::from_constant(Delta::from_sec_f64(32.184));
        let scale = TimePolyScale::proper(reference, poly);

        let onboard = Point::from_scale(scale).add(Delta::from_sec(100));
        let tt = onboard.to_pov_with_scale(scale);

        let expected = onboard
            .add(Delta::from_sec_f64(32.184))
            .with_pov(TimePov::Proper);
        assert_eq!(tt, expected);
    }

    #[test]
    fn from_pov_with_scale_inverse() {
        let reference = Point::from_tai_sec(0);
        let poly = TimePoly::new(
            Delta::from_ms(500), // exactly 0.5 s
            Delta::from_ns(2),   // exactly 2 ns/s = 2e-9 s/s
            Delta::ZERO,
        );
        let scale = TimePolyScale::proper(reference, poly);

        // Start from onboard Proper time (the natural input for this API)
        let proper = Point::from_scale(scale).add(Delta::from_sec(1_000_000));

        let tt = proper.to_pov_with_scale(scale); // Proper → TT
        let back = tt.from_pov_with_scale(scale); // TT → Proper

        assert_eq!(back, proper);
    }

    #[test]
    fn with_scale_and_from_scale() {
        let reference = Point::from_tai_sec(0);
        let poly = TimePoly::ZERO;
        let scale = TimePolyScale::proper(reference, poly);

        let raw = Point::from_tai_sec(123);
        let tagged = raw.with_scale(scale);

        assert_eq!(tagged.pov(), TimePov::Proper);
        assert_eq!(
            Point::from_scale(scale),
            reference.with_pov(TimePov::Proper)
        );
    }
}
