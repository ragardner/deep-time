use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_US,
    ClockDrift, ClockModel, ClockType, FS_PER_SEC, MS_PER_SEC, NS_PER_SEC, PS_PER_SEC,
    TT_TAI_OFFSET_SPAN, TimePoint, UNIX_EPOCH_TO_J2000_NOON_UTC, US_PER_SEC,
};

impl TimePoint {
    /// The library’s reference zero instant: exactly **2000-01-01 12:00:00 TAI**.
    pub const ZERO: Self = Self::new(0, 0, ClockType::TAI);

    /// The TAI instant that corresponds to the conventional **J2000.0 epoch**
    /// (2000-01-01 12:00:00 **TT**, JD 2451545.0 TT).
    pub const J2000_TAI: Self = Self::ZERO.sub(TT_TAI_OFFSET_SPAN);

    /// The J1900.0 epoch expressed in TAI (1900-01-01 12:00:00 TAI).
    pub const J1900_TAI: Self = Self::new(-3_155_760_000, 0, ClockType::TAI);

    /// Library zero points (same physical instant as ZERO, different tags)
    pub const GPS_ZERO: Self = Self::new(19, 0, ClockType::GPS);
    pub const GST_ZERO: Self = Self::new(19, 0, ClockType::GST);
    pub const QZSS_ZERO: Self = Self::new(19, 0, ClockType::QZSS);
    pub const BDT_ZERO: Self = Self::new(33, 0, ClockType::BDT);

    /// TAI time between 1970-01-01 midnight and 2000-01-01 noon
    pub const UNIX_EPOCH: Self = Self::new(-UNIX_EPOCH_TO_J2000_NOON_UTC, 0, ClockType::UTC);

    /// Traditional GNSS epochs
    pub const GPS_EPOCH: Self = Self::new(-630_763_200 + 19, 0, ClockType::GPS);
    pub const GALEX_EPOCH: Self = Self::GPS_EPOCH;
    pub const GALILEO_EPOCH: Self = Self::new(-11_448_000 + 19, 0, ClockType::GST);
    pub const BDT_EPOCH: Self = Self::new(189_345_600 + 33, 0, ClockType::BDT);

    /// Creates a new `TimePoint` from whole seconds, a subsecond part in attoseconds,
    /// and a clock type, automatically normalizing the representation.
    #[inline]
    pub const fn new(sec: i64, subsec: u64, clock_type: ClockType) -> Self {
        let mut tp = Self {
            sec,
            subsec,
            clock_type,
        };
        tp.carry_over();
        tp
    }

    /// Returns an exact copy of this `TimePoint`.
    ///
    /// This is a zero-cost, always-inlined convenience method.
    #[inline]
    pub const fn copy(self) -> Self {
        self
    }

    /// Creates a new custom clock model using this exact instant as the reference epoch.
    ///
    /// The supplied `ClockDrift` defines the relativistic model for the new clock.
    /// The resulting `ClockModel` can be used to convert to or from the custom timescale
    /// even after the observer has left the original reference frame.
    #[inline]
    pub const fn new_custom_clock(self, drift: ClockDrift) -> ClockModel {
        ClockModel::custom(self, drift)
    }

    /// Creates a new local clock model with zero drift using this instant as the reference epoch.
    ///
    /// The drift value can be updated later if relativistic effects are incorporated.
    #[inline]
    pub const fn new_local_clock(self) -> ClockModel {
        self.new_custom_clock(ClockDrift::ZERO)
    }

    #[inline]
    pub const fn from_tai_sec(sec: i64) -> Self {
        Self::from(sec, 0, ClockType::TAI)
    }

    #[inline]
    pub const fn from_attos(attos: i128, clock_type: ClockType) -> Self {
        let sec = (attos / ATTOS_PER_SEC as i128) as i64;
        let subsec = (attos % ATTOS_PER_SEC as i128) as u64;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_ms(ms: i128, clock_type: ClockType) -> Self {
        let sec = ms.div_euclid(MS_PER_SEC) as i64;
        let remaining_ms = ms.rem_euclid(MS_PER_SEC);
        let subsec = (remaining_ms as u64) * ATTOS_PER_MS;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_us(us: i128, clock_type: ClockType) -> Self {
        let sec = us.div_euclid(US_PER_SEC) as i64;
        let remaining_us = us.rem_euclid(US_PER_SEC);
        let subsec = (remaining_us as u64) * ATTOS_PER_US;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_ns(ns: i128, clock_type: ClockType) -> Self {
        let sec = ns.div_euclid(NS_PER_SEC) as i64;
        let remaining_ns = ns.rem_euclid(NS_PER_SEC);
        let subsec = (remaining_ns as u64) * ATTOS_PER_NS;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_ps(ps: i128, clock_type: ClockType) -> Self {
        let sec = ps.div_euclid(PS_PER_SEC) as i64;
        let remaining_ps = ps.rem_euclid(PS_PER_SEC);
        let subsec = (remaining_ps as u64) * ATTOS_PER_PS;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_fs(fs: i128, clock_type: ClockType) -> Self {
        let sec = fs.div_euclid(FS_PER_SEC) as i64;
        let remaining_fs = fs.rem_euclid(FS_PER_SEC);
        let subsec = (remaining_fs as u64) * ATTOS_PER_FS;
        Self::from(sec, subsec, clock_type)
    }

    #[inline]
    pub const fn from_min(m: i64, clock_type: ClockType) -> Self {
        Self::from(m * 60, 0, clock_type)
    }

    #[inline]
    pub const fn from_hr(h: i64, clock_type: ClockType) -> Self {
        Self::from(h * 3600, 0, clock_type)
    }

    /// Creates a `TimePoint` from hours, minutes, seconds, milliseconds, microseconds,
    /// and nanoseconds on the supplied clock type.
    pub const fn from_hms(
        hr: i64,
        min: i64,
        sec: i64,
        ms: i128,
        us: i128,
        ns: i128,
        clock_type: ClockType,
    ) -> Self {
        let total_sec = hr * 3600i64 + min * 60i64 + sec;

        let sub_ns = ms * 1_000_000i128 + us * 1_000i128 + ns;

        if sub_ns == 0 {
            return Self::new(total_sec, 0, clock_type);
        }

        let abs_ns = sub_ns.unsigned_abs();
        let extra_sec = (abs_ns / 1_000_000_000u128) as i64;
        let rem_ns = abs_ns % 1_000_000_000u128;
        let frac = (rem_ns as u64) * ATTOS_PER_NS;

        let (final_sec, final_frac) = if sub_ns >= 0 {
            (total_sec + extra_sec, frac)
        } else if frac == 0 {
            (total_sec - extra_sec, 0)
        } else {
            (total_sec - extra_sec - 1, ATTOS_PER_SEC - frac)
        };

        Self::from(final_sec, final_frac, clock_type)
    }

    /// Creates a `TimePoint` from a fully self-describing `ClockModel`.
    ///
    /// This is the recommended constructor when a spacecraft already carries its own
    /// relativistic clock model.
    #[inline]
    pub const fn create_from_model(model: ClockModel) -> Self {
        model.reference.to_type(model.base)
    }

    /// Replaces the current clock type of this `TimePoint` with the base clock type
    /// of the supplied `ClockModel`.
    ///
    /// This is the standard operation performed when a spacecraft receives an updated
    /// polynomial model from ground control.
    #[inline]
    pub const fn apply_new_model(self, model: ClockModel) -> Self {
        self.to_type(model.base)
    }

    /// Returns the current system time converted to the requested `ClockType`.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now(target: ClockType) -> Self {
        use crate::TimeSpan;

        let now = std::time::SystemTime::now();
        let (secs, nanos) = match now.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos() as i64),
            Err(_) => {
                // System time is before Unix epoch — support negative time
                let dur = std::time::SystemTime::UNIX_EPOCH
                    .duration_since(now)
                    .unwrap();
                (-(dur.as_secs() as i64), -(dur.subsec_nanos() as i64))
            }
        };
        crate::TimePoint::from_epoch(
            TimeSpan::new(secs, 0),
            TimePoint::UNIX_EPOCH,
            ClockType::UTC,
        )
        .add(crate::TimeSpan::from_ns(nanos as i128))
        .to_type(target)
    }

    /// Returns the current system time converted to the requested `ClockType`
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now(target: ClockType) -> Self {
        let millis = js_sys::Date::now() as i64;
        let secs = millis / 1000;
        let nanos = (millis % 1000) * 1_000_000;
        crate::TimePoint::from_unix_sec(secs)
            .add(crate::TimeSpan::from_ns(nanos))
            .to_type(target)
    }
}
