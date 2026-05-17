#[cfg(any(feature = "js", feature = "std"))]
use crate::DtErr;
use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC, ATTOS_PER_SEC_I128,
    ATTOS_PER_US, Dt, Real, SEC_PER_DAYI64, SEC_PER_WEEK, Scale,
    TAI_SECS_1970_MIDNIGHT_TO_2000_NOON,
};

impl Dt {
    /// The library’s internal reference epoch: exactly **2000-01-01 12:00:00 TAI**.
    ///
    /// [`Dt::new(0, 0)`].
    pub const ZERO: Self = Self::new(0, 0);

    /// UNIX epoch.
    /// - 1970-01-01 midnight TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -946_728_000 sec
    /// - 0 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const UNIX_EPOCH: Self = Self::new(-TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, 0);

    /// TT/TCG/TCB/TDB epoch.
    /// - 1977-01-01 midnight TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -725_803_200 sec
    /// - 0 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const TAI_1977_EPOCH: Self = Self::new(-725_803_200, 0);

    /// TT/TCG/TCB/TDB/TCL epoch.
    /// - 1977-01-01 midnight TAI.
    /// - Stored here on the **TCL** timescale as an offset from [`Self::ZERO`].
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const TCL_1977_EPOCH: Self = { Self::TAI_1977_EPOCH.to(Scale::TAI, Scale::TCL) };

    /// Chandra X-ray Center (CXC) Time epoch.
    /// - 1998-01-01 midnight TT.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -63_115_233 sec
    /// - 816000000000000000 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const CXC_EPOCH: Self = Self::new(-63_115_233, 816000000000000000);

    /// GPS/Galileo Experiment (GALEX) Time epoch.
    /// - **1980-01-06 00:00:00 UTC**.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -630_763_200 **+ 19** sec
    /// - 0 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const GPS_EPOCH: Self = Self::new(-630_763_200 + 19, 0);

    /// Galileo System Time (GST) epoch.
    /// - 1999-08-22 00:00:00 GST.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -11_448_000 **+ 19** sec
    /// - 0 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
    pub const GALILEO_EPOCH: Self = Self::new(-11_448_000 + 19, 0);

    /// BeiDou Time (BDT) epoch.
    /// - 2006-01-01 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - 189_345_600 **+ 33** sec
    /// - 0 attos
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    ///   This const is provided as a convenience.
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
        tp.carry_over_mut();
        tp
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
        let q = safe_div_euc!(attos, ATTOS_PER_SEC_I128, 0i128);

        if q > (i64::MAX as i128) {
            Self::MAX
        } else if q < (i64::MIN as i128) {
            Self::MIN
        } else {
            let r = safe_rem_euc!(attos, ATTOS_PER_SEC_I128, 0i128);
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

    /// High-precision conversion from f64 seconds to total attoseconds (i128).
    /// Uses IEEE 754 bit extraction + exact integer multiplication by 5^18.
    pub const fn sec_f_to_total_attos(sec_f: f64) -> i128 {
        if sec_f == 0.0 {
            return 0;
        }

        let bits = sec_f.to_bits();
        let is_negative = (bits >> 63) != 0;
        let biased_exp = ((bits >> 52) & 0x7ff) as i32;
        let mantissa = bits & 0x000f_ffff_ffff_ffff;

        // Extract significand and true binary exponent
        let (sig, exp) = if biased_exp == 0 {
            // Subnormal
            if mantissa == 0 {
                return 0;
            }
            let lz = mantissa.leading_zeros() as i32;
            let shift = lz - 11;
            let sig = (mantissa as u128) << shift;
            (sig, -1022i32 - 52 + shift)
        } else {
            let sig = ((1u64 << 52) | mantissa) as u128;
            (sig, biased_exp - 1023 - 52)
        };

        const FIVE_POW_18: u64 = 3_814_697_265_625; // 5^18 exactly
        let product = sig * (FIVE_POW_18 as u128);
        let total_exp = exp + 18;

        // Saturation guard for extremely large values
        if total_exp > 120 {
            return if is_negative { i128::MIN } else { i128::MAX };
        }
        if total_exp < -200 {
            return 0;
        }

        let abs_total = if total_exp >= 0 {
            if total_exp >= 128 {
                return if is_negative { i128::MIN } else { i128::MAX };
            }
            (product << total_exp) as i128
        } else {
            let shift = (-total_exp) as u32;
            let int_part = (product >> shift) as i128;

            // === ROUNDING CHOICE ===
            // Round to nearest, half away from zero.
            // This is recommended for maximum precision.
            // (Better average error than truncation.)
            let mask = (1u128 << shift) - 1;
            let rem = product & mask;
            if rem > (mask >> 1) {
                int_part + 1
            } else {
                int_part
            }
        };

        if is_negative { -abs_total } else { abs_total }
    }

    /// Creates a `Dt` from a floating-point number of seconds.
    /// - Assumes the value is on the given scale.
    /// - Converts the values **to TAI**, the returned [`Dt`] is on
    ///   the TAI time scale.
    pub const fn from_sec_f_on(sec_f: Real, s: Scale) -> Dt {
        if sec_f.is_nan() {
            return Self::ZERO;
        } else if sec_f.is_infinite() {
            return if sec_f.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let total_attos = Self::sec_f_to_total_attos(sec_f);

        // Split into floor(seconds) and attos in [0, ATTOS_PER_SEC)
        // Using div_euclid / rem_euclid guarantees a non-negative remainder
        // for both positive and negative total_attos.
        let floor_sec = total_attos.div_euclid(ATTOS_PER_SEC_I128);
        let mut attos = total_attos.rem_euclid(ATTOS_PER_SEC_I128);

        // Noise suppression? treat sub-attosecond values as exactly zero.
        // This prevents floating-point noise (from the f64 input) from
        // turning clean integer seconds into non-zero attos.
        if attos.abs() < 1 {
            attos = 0;
        }

        let total = floor_sec * ATTOS_PER_SEC_I128 + attos;
        Self::from_attos(total, s)
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    ///
    /// This method is only available when the `std` feature is enabled and the target
    /// is not WASM with the `js` feature.
    #[cfg(all(feature = "std", not(all(target_arch = "wasm32", feature = "js"))))]
    #[inline]
    pub fn now() -> Result<Self, DtErr> {
        let now = std::time::SystemTime::now();
        let (secs, nanos) = match now.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos() as i64),
            Err(_) => {
                // System time is before Unix epoch — support negative time
                use crate::{DtErrKind, an_err};
                let dur = std::time::SystemTime::UNIX_EPOCH
                    .duration_since(now)
                    .map_err(|e| an_err!(DtErrKind::IOErr, "{}", e))?;
                (-(dur.as_secs() as i64), -(dur.subsec_nanos() as i64))
            }
        };
        Ok(
            Dt::from_diff_and_scale(Dt::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
                .add(Dt::from_ns(nanos as i128, Scale::TAI)),
        )
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now() -> Result<Self, DtErr> {
        let ms: f64 = js_sys::Date::now();
        let secs = (ms / 1000.0).floor() as i64;
        let nanos = ((ms % 1000.0) * 1_000_000.0) as i128;
        Ok(
            Dt::from_diff_and_scale(Dt::new(secs, 0), Dt::UNIX_EPOCH, Scale::UTC)
                .add(Dt::from_ns(nanos as i128, Scale::TAI)),
        )
    }
}
