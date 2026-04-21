use crate::{
    ATTOSEC_PER_ATTOSEC, ATTOSEC_PER_FEMTOSEC, ATTOSEC_PER_MICROSEC, ATTOSEC_PER_MILLISEC,
    ATTOSEC_PER_NANOSEC, ATTOSEC_PER_PICOSEC, ATTOSEC_PER_SEC, ClockDrift, ClockModel, ClockType,
    TT_TAI_OFFSET_DELTA, TimePoint,
};

impl TimePoint {
    /// The library’s reference zero instant: exactly **2000-01-01 12:00:00 TAI**.
    ///
    /// This is the common zero point for **all built-in clock types** (except `Proper`/`Custom`).
    /// `TimePoint::new(0, 0, ClockType::XXX)` represents this exact same physical instant
    /// on every built-in scale.
    pub const ZERO: Self = Self {
        sec: 0,
        subsec: 0,
        clock_type: ClockType::TAI,
    };

    /// The TAI instant that corresponds to the conventional **J2000.0 epoch**
    /// (2000-01-01 12:00:00 **TT**, JD 2451545.0 TT).
    ///
    /// Because TT = TAI + 32.184 s, this is exactly 32.184 seconds *before* `ZERO`.
    /// This constant is provided for convenience when working with astronomical
    /// ephemerides that are natively referenced to J2000 TT.
    pub const J2000_TAI: Self = Self::ZERO.sub_ref(&TT_TAI_OFFSET_DELTA);

    /// The J1900.0 epoch expressed in TAI (1900-01-01 12:00:00 TAI).
    pub const J1900_TAI: Self = Self::from_tai_sec(-3_155_760_000);

    /// The library’s common reference zero instant expressed in GPS Time (GPST).
    ///
    /// This is **the same physical moment** as [`Self::ZERO`] (2000-01-01 12:00:00 TAI),
    /// but represented on the GPST scale.
    ///
    /// **Note**: This is *not* the traditional GPS reference epoch (1980-01-06 00:00:00 GPST).
    pub const GPS_EPOCH: Self = Self::new(0, 0, ClockType::GPST);

    /// The library’s common reference zero instant expressed in Galileo Time (GST).
    ///
    /// This is **the same physical moment** as [`Self::ZERO`] (2000-01-01 12:00:00 TAI),
    /// but represented on the GST scale.
    ///
    /// **Note**: This is *not* the traditional Galileo reference epoch (1999-08-22 00:00:00 GST).
    pub const GALILEO_EPOCH: Self = Self::new(0, 0, ClockType::GST);

    /// The library’s common reference zero instant expressed in BeiDou Time (BDT).
    ///
    /// This is **the same physical moment** as [`Self::ZERO`] (2000-01-01 12:00:00 TAI),
    /// but represented on the BDT scale.
    ///
    /// **Note**: This is *not* the traditional BeiDou reference epoch (2006-01-01 00:00:00 BDT).
    pub const BEIDOU_EPOCH: Self = Self::new(0, 0, ClockType::BDT);

    /// The library’s common reference zero instant expressed in QZSS Time (QZSST).
    ///
    /// This is **the same physical moment** as [`Self::ZERO`] (2000-01-01 12:00:00 TAI)
    /// and is identical to [`Self::GPS_EPOCH`] (QZSS uses the same timescale as GPS).
    pub const QZSS_EPOCH: Self = Self::new(0, 0, ClockType::QZSST);

    /// The TAI instant corresponding to the POSIX Unix epoch
    /// (1970-01-01 00:00:00 UTC).
    pub const UNIX_EPOCH_TAI: Self = Self {
        sec: -946_727_963,
        subsec: 0,
        clock_type: ClockType::TAI,
    };

    /// UTC representation of the POSIX Unix epoch (1970-01-01 00:00:00 UTC).
    pub const UNIX_EPOCH_UTC: Self = Self::UNIX_EPOCH_TAI.to_clock_type(ClockType::UTC);

    /// Traditional GPS / QZSS reference epoch: **1980-01-06 00:00:00 GPST**
    ///
    /// This is the epoch that GNSS receivers, navigation software, RINEX files,
    /// and the vast majority of the world expect when working with GPST/QZSST.
    pub const TRADITIONAL_GPS_EPOCH: Self = Self::new(-630_763_200, 0, ClockType::GPST);

    /// Traditional Galileo reference epoch: **1999-08-22 00:00:00 GST**
    pub const TRADITIONAL_GALILEO_EPOCH: Self = Self::new(-11_448_000, 0, ClockType::GST);

    /// Traditional BeiDou reference epoch: **2006-01-01 00:00:00 BDT**
    pub const TRADITIONAL_BEIDOU_EPOCH: Self = Self::new(189_345_600, 0, ClockType::BDT);

    /// Creates a new `TimePoint` from whole seconds, a subsecond part in attoseconds,
    /// and a clock type, automatically normalizing the representation.
    #[inline]
    pub const fn new(sec: i64, subsec: u64, clock_type: ClockType) -> Self {
        Self {
            sec,
            subsec,
            clock_type,
        }
        .carry_over()
    }

    /// Returns an exact copy of this `TimePoint`.
    ///
    /// This is a zero-cost, always-inlined convenience method.
    #[inline(always)]
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
    pub const fn from_sec(s: i64, clock_type: ClockType) -> Self {
        Self::new(s, 0, clock_type)
    }

    #[inline]
    pub const fn from_ms(ms: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ms, ATTOSEC_PER_MILLISEC, clock_type)
    }

    #[inline]
    pub const fn from_us(us: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(us, ATTOSEC_PER_MICROSEC, clock_type)
    }

    #[inline]
    pub const fn from_ns(ns: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ns, ATTOSEC_PER_NANOSEC, clock_type)
    }

    #[inline]
    pub const fn from_ps(ps: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(ps, ATTOSEC_PER_PICOSEC, clock_type)
    }

    #[inline]
    pub const fn from_fs(fs: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(fs, ATTOSEC_PER_FEMTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_as(as_: i128, clock_type: ClockType) -> Self {
        Self::from_subunits(as_, ATTOSEC_PER_ATTOSEC, clock_type)
    }

    #[inline]
    pub const fn from_min(m: i64, clock_type: ClockType) -> Self {
        Self::from_sec(m * 60, clock_type)
    }

    #[inline]
    pub const fn from_hr(h: i64, clock_type: ClockType) -> Self {
        Self::from_sec(h * 3600, clock_type)
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
        let frac = (rem_ns as u64) * ATTOSEC_PER_NANOSEC;

        let (final_sec, final_frac) = if sub_ns >= 0 {
            (total_sec + extra_sec, frac)
        } else if frac == 0 {
            (total_sec - extra_sec, 0)
        } else {
            (total_sec - extra_sec - 1, ATTOSEC_PER_SEC - frac)
        };

        Self::new(final_sec, final_frac, clock_type)
    }

    #[inline]
    pub const fn new_tai(sec: i64, subsec: u64) -> Self {
        Self::new(sec, subsec, ClockType::TAI)
    }

    #[inline]
    pub const fn from_tai_sec(s: i64) -> Self {
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

    const fn from_subunits(count: i128, attos_per_unit: u64, clock_type: ClockType) -> Self {
        let abs_count = count.unsigned_abs();
        let units_per_second = ATTOSEC_PER_SEC / attos_per_unit;

        let extra_sec = (abs_count / (units_per_second as u128)) as i64;
        let remaining = abs_count % (units_per_second as u128);
        let frac = (remaining as u64) * attos_per_unit;

        if count >= 0 {
            Self::new(extra_sec, frac, clock_type)
        } else if frac == 0 {
            Self::new(-extra_sec, 0, clock_type)
        } else {
            Self::new(-extra_sec - 1, ATTOSEC_PER_SEC - frac, clock_type)
        }
    }

    /// Creates a `TimePoint` from a fully self-describing `ClockModel`.
    ///
    /// This is the recommended constructor when a spacecraft already carries its own
    /// relativistic clock model.
    #[inline]
    pub const fn create_from_model(model: ClockModel) -> Self {
        model.reference.with_clock_type(model.base)
    }

    /// Replaces the current clock type of this `TimePoint` with the base clock type
    /// of the supplied `ClockModel`.
    ///
    /// This is the standard operation performed when a spacecraft receives an updated
    /// polynomial model from ground control.
    #[inline]
    pub const fn apply_new_model(self, model: ClockModel) -> Self {
        self.with_clock_type(model.base)
    }

    /// Returns the current system time converted to the requested `ClockType`.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now(target: ClockType) -> Self {
        let dur = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before Unix epoch");
        let secs = dur.as_secs() as i64;
        let nanos = dur.subsec_nanos() as i64;
        crate::TimePoint::from_unix_sec(secs)
            .add(crate::Delta::from_ns(nanos))
            .to_clock_type(target)
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
            .add(crate::Delta::from_ns(nanos))
            .to_clock_type(target)
    }
}
