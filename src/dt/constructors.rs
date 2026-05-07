use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_US,
    ClockDrift, ClockModel, Dt, FS_PER_SEC, MS_PER_SEC, NS_PER_SEC, PS_PER_SEC, Scale,
    TT_TAI_OFFSET_SPAN, UNIX_EPOCH_TO_J2000_NOON_UTC, US_PER_SEC,
};

impl Dt {
    /// The library’s reference zero instant: exactly **2000-01-01 12:00:00 TAI**.
    pub const ZERO: Self = Self::new(0, 0);

    /// The TAI instant that corresponds to the conventional **J2000.0 epoch**
    /// (2000-01-01 12:00:00 **TT**, JD 2451545.0 TT).
    pub const J2000_TAI: Self = Self::ZERO.sub(TT_TAI_OFFSET_SPAN);

    /// The J1900.0 epoch expressed in TAI (1900-01-01 12:00:00 TAI).
    pub const J1900_TAI: Self = Self::new(-3_155_760_000, 0);

    /// Library zero points (same physical instant as ZERO, different tags)
    pub const GPS_ZERO: Self = Self::new(19, 0);
    pub const GST_ZERO: Self = Self::new(19, 0);
    pub const QZSS_ZERO: Self = Self::new(19, 0);
    pub const BDT_ZERO: Self = Self::new(33, 0);

    /// TAI time between 1970-01-01 midnight and 2000-01-01 noon
    pub const UNIX_EPOCH: Self = Self::new(-UNIX_EPOCH_TO_J2000_NOON_UTC, 0);

    /// Traditional GNSS epochs
    pub const GPS_EPOCH: Self = Self::new(-630_763_200 + 19, 0);
    pub const GALEX_EPOCH: Self = Self::GPS_EPOCH;
    pub const GALILEO_EPOCH: Self = Self::new(-11_448_000 + 19, 0);
    pub const BDT_EPOCH: Self = Self::new(189_345_600 + 33, 0);

    /// Creates a new `Dt` from whole seconds, a subsecond part in attoseconds,
    /// and a scale, automatically normalizing the representation.
    #[inline]
    pub const fn new(sec: i64, attos: u64) -> Self {
        let mut tp = Self { sec, attos };
        tp.carry_over();
        tp
    }

    /// Creates a new custom clock model using this exact instant as the reference epoch.
    ///
    /// The supplied `ClockDrift` defines the relativistic model for the new clock.
    /// The resulting `ClockModel` can be used to convert to or from the custom timescale
    /// even after the observer has left the original reference frame.
    #[inline]
    pub const fn new_custom_clock(self, drift: ClockDrift) -> ClockModel {
        ClockModel::new(Scale::Custom, self, drift)
    }

    #[inline]
    pub const fn from_attos(attos: i128, scale: Scale) -> Self {
        let sec = (attos / ATTOS_PER_SEC as i128) as i64;
        let subsec = (attos % ATTOS_PER_SEC as i128) as u64;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_sec(sec: i64, scale: Scale) -> Self {
        Self::from(sec, 0, scale)
    }

    #[inline]
    pub const fn from_ms(ms: i128, scale: Scale) -> Self {
        let sec = ms.div_euclid(MS_PER_SEC) as i64;
        let remaining_ms = ms.rem_euclid(MS_PER_SEC);
        let subsec = (remaining_ms as u64) * ATTOS_PER_MS;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_us(us: i128, scale: Scale) -> Self {
        let sec = us.div_euclid(US_PER_SEC) as i64;
        let remaining_us = us.rem_euclid(US_PER_SEC);
        let subsec = (remaining_us as u64) * ATTOS_PER_US;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_ns(ns: i128, scale: Scale) -> Self {
        let sec = ns.div_euclid(NS_PER_SEC) as i64;
        let remaining_ns = ns.rem_euclid(NS_PER_SEC);
        let subsec = (remaining_ns as u64) * ATTOS_PER_NS;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_ps(ps: i128, scale: Scale) -> Self {
        let sec = ps.div_euclid(PS_PER_SEC) as i64;
        let remaining_ps = ps.rem_euclid(PS_PER_SEC);
        let subsec = (remaining_ps as u64) * ATTOS_PER_PS;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_fs(fs: i128, scale: Scale) -> Self {
        let sec = fs.div_euclid(FS_PER_SEC) as i64;
        let remaining_fs = fs.rem_euclid(FS_PER_SEC);
        let subsec = (remaining_fs as u64) * ATTOS_PER_FS;
        Self::from(sec, subsec, scale)
    }

    #[inline]
    pub const fn from_min(m: i64, scale: Scale) -> Self {
        Self::from(m * 60, 0, scale)
    }

    #[inline]
    pub const fn from_hr(h: i64, scale: Scale) -> Self {
        Self::from(h * 3600, 0, scale)
    }

    /// Creates a `Dt` from hours, minutes, seconds, milliseconds, microseconds,
    /// and nanoseconds on the supplied scale.
    pub const fn from_hms(
        hr: i64,
        min: i64,
        sec: i64,
        ms: i128,
        us: i128,
        ns: i128,
        scale: Scale,
    ) -> Self {
        let total_sec = hr * 3600i64 + min * 60i64 + sec;

        let sub_ns = ms * 1_000_000i128 + us * 1_000i128 + ns;

        if sub_ns == 0 {
            return Self::from(total_sec, 0, scale);
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

        Self::from(final_sec, final_frac, scale)
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now() -> Self {
        use crate::TSpan;

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
        crate::Dt::from_epoch(TSpan::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
            .add(crate::TSpan::from_ns(nanos as i128))
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now() -> Self {
        let millis = js_sys::Date::now() as i64;
        let secs = millis / 1000;
        let nanos = (millis % 1000) * 1_000_000;
        crate::Dt::from_epoch(TSpan::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
            .add(crate::TSpan::from_ns(nanos))
    }
}
