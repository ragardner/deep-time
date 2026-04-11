use crate::{
    MICROQUECTOS_PER_ATTOSEC, MICROQUECTOS_PER_FEMTOSEC, MICROQUECTOS_PER_MICROSEC,
    MICROQUECTOS_PER_MILLISEC, MICROQUECTOS_PER_NANOSEC, MICROQUECTOS_PER_PICOSEC,
    MICROQUECTOS_PER_QUECTOSEC, MICROQUECTOS_PER_RONTOSEC, MICROQUECTOS_PER_SEC,
    MICROQUECTOS_PER_YOCTOSEC, MICROQUECTOS_PER_ZEPTOSEC, Point, TimePov,
};

impl Point {
    /// Zero duration on TAI (most common default).
    pub const ZERO: Self = Self {
        sec: 0,
        subsec: 0,
        pov: TimePov::TAI,
    };

    /// Creates a new `Point` with automatic normalization.
    #[inline]
    pub const fn new(sec: i128, subsec: u128, pov: TimePov) -> Self {
        Self { sec, subsec, pov }.carry_over()
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

    #[inline]
    pub const fn from_sec(s: i128, pov: TimePov) -> Self {
        Self::new(s, 0, pov)
    }

    #[inline]
    pub const fn from_ms(ms: i128, pov: TimePov) -> Self {
        Self::from_subunits(ms, MICROQUECTOS_PER_MILLISEC, pov)
    }

    #[inline]
    pub const fn from_us(us: i128, pov: TimePov) -> Self {
        Self::from_subunits(us, MICROQUECTOS_PER_MICROSEC, pov)
    }

    #[inline]
    pub const fn from_ns(ns: i128, pov: TimePov) -> Self {
        Self::from_subunits(ns, MICROQUECTOS_PER_NANOSEC, pov)
    }

    #[inline]
    pub const fn from_ps(ps: i128, pov: TimePov) -> Self {
        Self::from_subunits(ps, MICROQUECTOS_PER_PICOSEC, pov)
    }

    #[inline]
    pub const fn from_fs(fs: i128, pov: TimePov) -> Self {
        Self::from_subunits(fs, MICROQUECTOS_PER_FEMTOSEC, pov)
    }

    #[inline]
    pub const fn from_as(as_: i128, pov: TimePov) -> Self {
        Self::from_subunits(as_, MICROQUECTOS_PER_ATTOSEC, pov)
    }

    #[inline]
    pub const fn from_zs(zs: i128, pov: TimePov) -> Self {
        Self::from_subunits(zs, MICROQUECTOS_PER_ZEPTOSEC, pov)
    }

    #[inline]
    pub const fn from_ys(ys: i128, pov: TimePov) -> Self {
        Self::from_subunits(ys, MICROQUECTOS_PER_YOCTOSEC, pov)
    }

    #[inline]
    pub const fn from_rs(rs: i128, pov: TimePov) -> Self {
        Self::from_subunits(rs, MICROQUECTOS_PER_RONTOSEC, pov)
    }

    #[inline]
    pub const fn from_qs(qs: i128, pov: TimePov) -> Self {
        Self::from_subunits(qs, MICROQUECTOS_PER_QUECTOSEC, pov)
    }

    #[inline]
    pub const fn from_min(m: i128, pov: TimePov) -> Self {
        Self::from_sec(m * 60, pov)
    }

    #[inline]
    pub const fn from_hr(h: i128, pov: TimePov) -> Self {
        Self::from_sec(h * 3600, pov)
    }

    #[inline]
    pub const fn from_hms(
        hr: i128,
        min: i128,
        sec: i128,
        ms: i128,
        us: i128,
        ns: i128,
        scale: TimePov,
    ) -> Self {
        // Whole seconds (fast, tiny multipliers)
        let total_sec = hr * 3600i128 + min * 60i128 + sec;

        // Sub-second part only
        let sub_ns = ms * 1_000_000i128 + us * 1_000i128 + ns;

        // Fast path: nothing fractional
        if sub_ns == 0 {
            return Self::new(total_sec, 0, scale);
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

        Self::new(final_sec, final_frac, scale)
    }

    #[inline]
    pub const fn new_tai(sec: i128, subsec: u128) -> Self {
        Self::new(sec, subsec, TimePov::TAI)
    }

    #[inline]
    pub const fn new_utc(sec: i128, subsec: u128) -> Self {
        Self::new(sec, subsec, TimePov::UTC)
    }

    #[inline]
    pub const fn from_tai_sec(s: i128) -> Self {
        Self::from_sec(s, TimePov::TAI)
    }

    #[inline]
    pub const fn from_tai_ms(ms: i128) -> Self {
        Self::from_ms(ms, TimePov::TAI)
    }

    #[inline]
    pub const fn from_tai_us(us: i128) -> Self {
        Self::from_us(us, TimePov::TAI)
    }

    #[inline]
    pub const fn from_tai_ns(ns: i128) -> Self {
        Self::from_ns(ns, TimePov::TAI)
    }

    #[inline]
    pub const fn from_utc_sec(s: i128) -> Self {
        Self::from_sec(s, TimePov::UTC)
    }

    #[inline]
    pub const fn from_utc_ms(ms: i128) -> Self {
        Self::from_ms(ms, TimePov::UTC)
    }

    #[inline]
    pub const fn from_utc_ns(ns: i128) -> Self {
        Self::from_ns(ns, TimePov::UTC)
    }

    #[inline]
    const fn from_subunits(count: i128, subsec_per_unit: u128, pov: TimePov) -> Self {
        let abs_count = count.unsigned_abs();
        let units_per_second = MICROQUECTOS_PER_SEC / subsec_per_unit;

        let extra_sec = (abs_count / units_per_second) as i128;
        let remaining = abs_count % units_per_second;
        let frac = remaining * subsec_per_unit;

        if count >= 0 {
            Self::new(extra_sec, frac, pov)
        } else if frac == 0 {
            Self::new(-extra_sec, 0, pov)
        } else {
            Self::new(-extra_sec - 1, MICROQUECTOS_PER_SEC - frac, pov)
        }
    }

    /// Returns the current system time in the requested `TimePov`.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now(target: TimePov) -> Self {
        use crate::UnixTimestamp;

        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before Unix epoch");
        let secs = dur.as_secs() as i128;
        let nanos = dur.subsec_nanos() as i128;

        secs.unix_seconds()
            .add(crate::Delta::from_ns(nanos))
            .to_pov(target)
    }

    /// Browser WASM version using JavaScript's `Date.now()`
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now(target: TimePov) -> Self {
        use crate::UnixTimestamp;

        // `Date.now()` returns milliseconds since Unix epoch as f64.
        // We cast early and use integer math (perfectly safe for current timestamps).
        let millis = js_sys::Date::now() as i128;
        let secs = millis / 1000;
        let nanos = (millis % 1000) * 1_000_000;

        secs.unix_seconds()
            .add(crate::Delta::from_ns(nanos))
            .to_pov(target)
    }
}
