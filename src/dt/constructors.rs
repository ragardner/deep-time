use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_SECF, ATTOS_PER_US, ClockModel, Drift, Dt, Real, SEC_PER_DAYI64, SEC_PER_WEEK, Scale,
    TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, floor_f,
};

impl Dt {
    /// The library’s internal reference epoch: exactly **2000-01-01 12:00:00 TAI**.
    ///
    /// [`Dt::new(0, 0)`].
    pub const ZERO: Self = Self::new(0, 0);

    /// The Unix epoch (**1970-01-01 00:00:00 UTC**) expressed as a signed
    /// TAI-second offset from [`Self::ZERO`].
    ///
    /// This is computed as `-TAI_SECS_1970_MIDNIGHT_TO_2000_NOON`, i.e. the exact
    /// number of TAI seconds from 1970-01-01 00:00:00 TAI to 2000-01-01 12:00:00 TAI
    /// (the value of the private constant `TAI_SECS_1970_MIDNIGHT_TO_2000_NOON`).
    pub const UNIX_EPOCH: Self = Self::new(-TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, 0);

    /// Canonical reference epoch for modern relativistic time scales:
    /// exactly **1977-01-01 00:00:00 TAI**.
    ///
    /// This is the defining instant (per IAU recommendations) where:
    /// - Terrestrial Time (TT) = TAI + 32.184 s exactly,
    /// - TT, TCG, TCB, and TCL are synchronized by definition,
    /// - Lunar Coordinate Time (TCL) and the proposed TL use this as their official zero point.
    ///
    /// It is **-725_803_200 TAI seconds** before [`Self::ZERO`].
    pub const TAI_1977_EPOCH: Self = Self::new(-725_803_200, 0);

    pub const TCL_1977_EPOCH: Self = {
        let tcl1977 = Self::TAI_1977_EPOCH.to(Scale::TAI, Scale::TCL);
        Self::new(tcl1977.sec, tcl1977.attos)
    };

    /// 1998-01-01 midnight TT, but as **TAI**.
    /// - -63_115_233 sec
    /// - 816000000000000000 attos
    pub const CXC_EPOCH: Self = Self::new(-63_115_233, 816000000000000000);

    /// GPS Time epoch: exactly **1980-01-06 00:00:00 UTC**.
    /// Galileo Experiment (GALEX) epoch: exactly **1980-01-06 00:00:00 UTC**.
    ///
    /// This is the zero point of the continuous GPS Time scale (GPST).
    /// At this epoch, GPST was exactly 19 seconds behind TAI.
    pub const GPS_EPOCH: Self = Self::new(-630_763_200 + 19, 0);

    /// Galileo System Time (GST) epoch: exactly **1999-08-22 00:00:00 UTC**.
    ///
    /// This is the official zero point of the Galileo System Time scale.
    pub const GALILEO_EPOCH: Self = Self::new(-11_448_000 + 19, 0);

    /// BeiDou Time (BDT) epoch: exactly **2006-01-01 00:00:00 UTC**.
    ///
    /// This is the official zero point of the BeiDou Navigation Satellite System Time.
    pub const BDT_EPOCH: Self = Self::new(189_345_600 + 33, 0);

    /// Maximum representable duration (`i64::MAX` seconds + 999... attoseconds).
    pub const MAX: Self = Self {
        sec: i64::MAX,
        attos: ATTOS_PER_SEC - 1,
    };

    /// Minimum (most negative) representable duration (`i64::MIN` seconds).
    pub const MIN: Self = Self {
        sec: i64::MIN,
        attos: 0,
    };

    pub const SEC_19: Self = Self::new(19, 0);
    pub const SEC_33: Self = Self::new(33, 0);
    pub const SEC_37: Self = Self::new(37, 0);
    pub const ONE_DAY: Self = Self::new(SEC_PER_DAYI64, 0);

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
    /// The supplied `Drift` defines the relativistic model for the new clock.
    /// The resulting `ClockModel` can be used to convert to or from the custom timescale
    /// even after the observer has left the original reference frame.
    #[inline]
    pub const fn new_custom_clock(self, drift: Drift) -> ClockModel {
        ClockModel::new(Scale::Custom, self, drift)
    }

    #[inline]
    pub const fn from_attos(attos: i128, scale: Scale) -> Self {
        Self::from_dt(Dt::attos_to_dt(attos), scale)
    }

    #[inline]
    pub const fn from_sec(sec: i64, scale: Scale) -> Self {
        Self::from(sec, 0, scale)
    }

    #[inline]
    pub const fn from_ms(ms: i128, scale: Scale) -> Self {
        let attos = ms.saturating_mul(ATTOS_PER_MS as i128);
        Self::from_dt(Dt::attos_to_dt(attos), scale)
    }

    #[inline]
    pub const fn from_us(us: i128, scale: Scale) -> Self {
        let attos = us.saturating_mul(ATTOS_PER_US as i128);
        Self::from_dt(Dt::attos_to_dt(attos), scale)
    }

    #[inline]
    pub const fn from_ns(ns: i128, scale: Scale) -> Self {
        let attos = ns.saturating_mul(ATTOS_PER_NS as i128);
        Self::from_dt(Dt::attos_to_dt(attos), scale)
    }

    #[inline]
    pub const fn from_ps(ps: i128, scale: Scale) -> Self {
        let attos = ps.saturating_mul(ATTOS_PER_PS as i128);
        Self::from_dt(Dt::attos_to_dt(attos), scale)
    }

    #[inline]
    pub const fn from_fs(fs: i128, scale: Scale) -> Self {
        let attos = fs.saturating_mul(ATTOS_PER_FS as i128);
        Self::from_dt(Dt::attos_to_dt(attos), scale)
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

    pub(crate) const fn attos_to_dt(attos: i128) -> Self {
        let q = attos.div_euclid(ATTOS_PER_SEC_I128);

        if q > (i64::MAX as i128) {
            return Self::MAX;
        } else if q < (i64::MIN as i128) {
            return Self::MIN;
        } else {
            let r = attos.rem_euclid(ATTOS_PER_SEC_I128);
            Self {
                sec: q as i64,
                attos: r as u64,
            }
        }
    }

    #[inline]
    pub const fn from_days(d: i64, scale: Scale) -> Dt {
        Self::from_sec(d.saturating_mul(SEC_PER_DAYI64), scale)
    }

    #[inline]
    pub const fn wk(wk: i64, scale: Scale) -> Dt {
        Dt::from_sec(wk.saturating_mul(SEC_PER_WEEK), scale)
    }

    #[inline]
    pub const fn yr(yr: i64, scale: Scale) -> Dt {
        Dt::from_sec(yr.saturating_mul(31_557_600), scale)
    }

    /// Returns a `Dt` that is this duration ago from the given scale.
    #[inline]
    pub const fn ago(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).sub(self)
    }

    /// Returns a `Dt` that is this duration from now in the given scale.
    #[inline]
    pub const fn from_now(self, scale: Scale) -> Dt {
        Dt::from(0, 0, scale).add(self)
    }

    /// Returns the negation of this duration.
    #[inline]
    pub const fn neg(self) -> Self {
        if self.attos == 0 {
            Self {
                sec: -self.sec,
                attos: 0,
            }
        } else {
            Self {
                sec: -self.sec - 1,
                attos: ATTOS_PER_SEC - self.attos,
            }
        }
    }

    /// Returns the positive of this duration.
    #[inline]
    pub const fn abs(self) -> Self {
        Self::from_attos(self.to_attos().saturating_abs(), Scale::TAI)
    }

    /// Creates a `Dt` from a floating-point number of seconds.
    #[inline]
    pub const fn from_sec_f(sec_f: Real) -> Self {
        Self::from_sec_f_on(sec_f, Scale::TAI)
    }

    pub const fn from_sec_f_on(sec_f: Real, s: Scale) -> Self {
        if sec_f.is_nan() {
            return Self::ZERO;
        }
        if sec_f.is_infinite() {
            return if sec_f.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let floor_val = floor_f(sec_f);
        let frac = sec_f - floor_val;
        let attos_frac = (frac * ATTOS_PER_SECF) as i128;

        let total = (floor_val as i128) * ATTOS_PER_SEC_I128 + attos_frac;
        Self::from_attos(total, s)
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now() -> Self {
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

        Dt::from_diff_and_scale(Dt::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
            .add(Dt::from_ns(nanos as i128, Scale::TAI))
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now() -> Self {
        let ms: f64 = js_sys::Date::now();
        let secs = (ms / 1000.0).floor() as i64;
        let nanos = ((ms % 1000.0) * 1_000_000.0) as i128;

        Dt::from_diff_and_scale(Dt::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
            .add(Dt::from_ns(nanos as i128, Scale::TAI))
    }
}
