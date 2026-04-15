use crate::{
    ClockDrift, ClockModel, ClockType, Delta, MICROQUECTOS_PER_ATTOSEC, MICROQUECTOS_PER_FEMTOSEC,
    MICROQUECTOS_PER_MICROSEC, MICROQUECTOS_PER_MILLISEC, MICROQUECTOS_PER_NANOSEC,
    MICROQUECTOS_PER_PICOSEC, MICROQUECTOS_PER_QUECTOSEC, MICROQUECTOS_PER_RONTOSEC,
    MICROQUECTOS_PER_SEC, MICROQUECTOS_PER_YOCTOSEC, MICROQUECTOS_PER_ZEPTOSEC, TimePoint,
};

impl TimePoint {
    /// Zero duration on TAI (most common default).
    pub const ZERO: Self = Self {
        sec: 0,
        subsec: 0,
        clock_type: ClockType::TAI,
    };

    /// J2000_TAI is the library’s chosen internal zero point.
    /// It corresponds exactly to the J2000.0 instant (2000-01-01 12:00:00 TT).
    /// Therefore TAI zero in this library = J2000.0 TT (i.e. 2000-01-01 11:59:27.816 in real-world TAI).
    pub const J2000_TAI: Self = Self::ZERO;

    /// J1900 reference epoch: 1900-01-01 12:00:00 TAI (noon)
    /// Exactly 36,525 days before J2000.0 (integer seconds)
    pub const J1900_TAI: Self = Self::from_tai_sec(-3_155_760_000);

    /// UNIX epoch expressed in TAI: 1970-01-01 00:00:00 TAI
    /// (exact fractional handling for the 0.184 s borrow)
    pub const UNIX_EPOCH_TAI: Self = Self {
        sec: -946_728_000,
        subsec: MICROQUECTOS_PER_SEC - 184 * MICROQUECTOS_PER_MILLISEC,
        clock_type: ClockType::TAI,
    };

    /// GPS Time reference epoch: 1980-01-06 00:00:00 GPST
    pub const GPS_EPOCH: Self = Self::new(0, 0, ClockType::GPST);

    /// Galileo Time reference epoch: 1999-08-22 00:00:00 GST
    pub const GALILEO_EPOCH: Self = Self::new(0, 0, ClockType::GST);

    /// BeiDou Time reference epoch: 2006-01-01 00:00:00 BDT
    pub const BEIDOU_EPOCH: Self = Self::new(0, 0, ClockType::BDT);

    /// QZSS Time reference epoch (identical reference to GPST)
    pub const QZSS_EPOCH: Self = Self::new(0, 0, ClockType::QZSST);

    /// Creates a new `TimePoint` with automatic normalization.
    #[inline]
    pub const fn new(sec: i128, subsec: u128, clock_type: ClockType) -> Self {
        Self {
            sec,
            subsec,
            clock_type,
        }
        .carry_over()
    }

    /// Normalizes the representation so `subsec` stays in `[0, 10³⁶)`.
    const fn carry_over(mut self) -> Self {
        if self.subsec >= MICROQUECTOS_PER_SEC {
            let carry = (self.subsec / MICROQUECTOS_PER_SEC) as i128;
            self.sec += carry;
            self.subsec %= MICROQUECTOS_PER_SEC;
        }
        self
    }

    /// Returns an exact copy of this `TimePoint`.
    ///
    /// This is a zero-cost, always-inlined convenience method.
    #[inline(always)]
    pub const fn copy(self) -> Self {
        self
    }

    /// Creates a new custom clock model (e.g. for a new solar system, planet,
    /// or any user-defined time standard) using **this exact instant** as
    /// the reference epoch.
    ///
    /// The resulting `ClockModel` can be used to convert to/from the new
    /// custom time even after the observer has left the system.
    #[inline]
    pub const fn new_custom_clock(self, drift: ClockDrift) -> ClockModel {
        ClockModel::custom(self, drift)
    }

    /// Convenience one-liner for creating a new local clock with zero drift.
    ///
    /// The drift can be updated later if relativistic effects are discovered.
    #[inline]
    pub const fn new_local_clock(self) -> ClockModel {
        self.new_custom_clock(ClockDrift::ZERO)
    }

    #[inline]
    pub const fn from_sec(s: i128, clock_type: ClockType) -> Self {
        Self::new(s, 0, clock_type)
    }

    #[inline]
    pub const fn from_ms(ms: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ms, MICROQUECTOS_PER_MILLISEC, clock_type)
    }

    #[inline]
    pub const fn from_us(us: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(us, MICROQUECTOS_PER_MICROSEC, clock_type)
    }

    #[inline]
    pub const fn from_ns(ns: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ns, MICROQUECTOS_PER_NANOSEC, clock_type)
    }

    #[inline]
    pub const fn from_ps(ps: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ps, MICROQUECTOS_PER_PICOSEC, clock_type)
    }

    #[inline]
    pub const fn from_fs(fs: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(fs, MICROQUECTOS_PER_FEMTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_as(as_: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(as_, MICROQUECTOS_PER_ATTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_zs(zs: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(zs, MICROQUECTOS_PER_ZEPTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_ys(ys: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ys, MICROQUECTOS_PER_YOCTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_rs(rs: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(rs, MICROQUECTOS_PER_RONTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_qs(qs: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(qs, MICROQUECTOS_PER_QUECTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_min(m: i128, clock_type: ClockType) -> Self {
        Self::from_sec(m * 60, clock_type)
    }

    #[inline]
    pub const fn from_hr(h: i128, clock_type: ClockType) -> Self {
        Self::from_sec(h * 3600, clock_type)
    }

    pub const fn from_hms(
        hr: i128,
        min: i128,
        sec: i128,
        ms: i128,
        us: i128,
        ns: i128,
        clock_type: ClockType,
    ) -> Self {
        // Whole seconds
        let total_sec = hr * 3600i128 + min * 60i128 + sec;
        // Sub-second part only
        let sub_ns = ms * 1_000_000i128 + us * 1_000i128 + ns;
        // Fast path: nothing fractional
        if sub_ns == 0 {
            return Self::new(total_sec, 0, clock_type);
        }
        // Inline from_subunits logic for nanoseconds → microquectoseconds
        let abs_ns = sub_ns.unsigned_abs();
        let extra_sec = (abs_ns / 1_000_000_000u128) as i128;
        let rem_ns = abs_ns % 1_000_000_000u128;
        let frac = rem_ns * MICROQUECTOS_PER_NANOSEC;

        let (final_sec, final_frac) = if sub_ns >= 0 {
            (total_sec + extra_sec, frac)
        } else if frac == 0 {
            (total_sec - extra_sec, 0)
        } else {
            (total_sec - extra_sec - 1, MICROQUECTOS_PER_SEC - frac)
        };

        Self::new(final_sec, final_frac, clock_type)
    }

    #[inline]
    pub const fn new_tai(sec: i128, subsec: u128) -> Self {
        Self::new(sec, subsec, ClockType::TAI)
    }

    #[inline]
    pub const fn new_utc(sec: i128, subsec: u128) -> Self {
        Self::new(sec, subsec, ClockType::UTC)
    }

    #[inline]
    pub const fn from_tai_sec(s: i128) -> Self {
        Self::from_sec(s, ClockType::TAI)
    }

    #[inline]
    pub const fn from_tai_ms(ms: i128) -> Self {
        Self::from_ms(ms, ClockType::TAI)
    }

    #[inline]
    pub const fn from_tai_us(us: i128) -> Self {
        Self::from_us(us, ClockType::TAI)
    }

    #[inline]
    pub const fn from_tai_ns(ns: i128) -> Self {
        Self::from_ns(ns, ClockType::TAI)
    }

    #[inline]
    pub const fn from_utc_sec(s: i128) -> Self {
        Self::from_sec(s, ClockType::UTC)
    }

    #[inline]
    pub const fn from_utc_ms(ms: i128) -> Self {
        Self::from_ms(ms, ClockType::UTC)
    }

    #[inline]
    pub const fn from_utc_us(us: i128) -> Self {
        Self::from_us(us, ClockType::UTC)
    }

    #[inline]
    pub const fn from_utc_ns(ns: i128) -> Self {
        Self::from_ns(ns, ClockType::UTC)
    }

    #[inline]
    pub const fn from_unix_sec(s: i128) -> Self {
        Self::new(
            Self::UNIX_EPOCH_TAI.sec + s,
            Self::UNIX_EPOCH_TAI.subsec,
            ClockType::TAI,
        )
    }

    #[inline]
    pub const fn from_unix_ms(ms: i128) -> Self {
        Self::from_unix_sec(0).add(Delta::from_ms(ms))
    }

    #[inline]
    pub const fn from_unix_us(us: i128) -> Self {
        Self::from_unix_sec(0).add(Delta::from_us(us))
    }

    #[inline]
    pub const fn from_unix_ns(ns: i128) -> Self {
        Self::from_unix_sec(0).add(Delta::from_ns(ns))
    }

    /// GPS Time (GPST) – seconds since GPS epoch (1980-01-06 00:00:00 GPST)
    #[inline]
    pub const fn from_gps_sec(s: i128) -> Self {
        Self::new(s, 0, ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_ms(ms: i128) -> Self {
        Self::from_ms(ms, ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_us(us: i128) -> Self {
        Self::from_us(us, ClockType::GPST)
    }

    #[inline]
    pub const fn from_gps_ns(ns: i128) -> Self {
        Self::from_ns(ns, ClockType::GPST)
    }

    #[inline]
    const fn from_subunits(count: i128, subsec_per_unit: u128, clock_type: ClockType) -> Self {
        let abs_count = count.unsigned_abs();
        let units_per_second = MICROQUECTOS_PER_SEC / subsec_per_unit;

        let extra_sec = (abs_count / units_per_second) as i128;
        let remaining = abs_count % units_per_second;
        let frac = remaining * subsec_per_unit;

        if count >= 0 {
            Self::new(extra_sec, frac, clock_type)
        } else if frac == 0 {
            Self::new(-extra_sec, 0, clock_type)
        } else {
            Self::new(-extra_sec - 1, MICROQUECTOS_PER_SEC - frac, clock_type)
        }
    }

    /// Creates a `TimePoint` from a fully self-describing [`ClockModel`].
    ///
    /// This is the recommended way for spacecraft to represent
    /// onboard proper time that already carries its own relativistic model.
    #[inline]
    pub const fn create_from_model(model: ClockModel) -> Self {
        model.reference.with_clock_type(model.base)
    }

    /// Replaces the current clock type with the base clock_type of a fully self-describing model.
    ///
    /// This is the most common operation on a spacecraft: you have a raw `Proper`
    /// reading and you just received a new polynomial update from ground.
    #[inline]
    pub const fn apply_new_model(self, model: ClockModel) -> Self {
        self.with_clock_type(model.base)
    }

    /// Returns the current system time in the requested `ClockType`.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now(target: ClockType) -> Self {
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before Unix epoch");
        let secs = dur.as_secs() as i128;
        let nanos = dur.subsec_nanos() as i128;
        crate::TimePoint::from_unix_sec(secs)
            .add(crate::Delta::from_ns(nanos))
            .to_clock_type(target)
    }

    /// Browser WASM version using JavaScript's `Date.now()`
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now(target: ClockType) -> Self {
        // `Date.now()` returns milliseconds since Unix epoch as float.
        // We cast early and use integer math (perfectly safe for current timestamps).
        let millis = js_sys::Date::now() as i128;
        let secs = millis / 1000;
        let nanos = (millis % 1000) * 1_000_000;
        crate::TimePoint::from_unix_sec(secs)
            .add(crate::Delta::from_ns(nanos))
            .to_clock_type(target)
    }
}
