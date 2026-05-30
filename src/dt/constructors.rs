#[cfg(any(feature = "js", feature = "std"))]
use crate::DtErr;
use crate::{
    ATTOS_PER_FS, ATTOS_PER_MS, ATTOS_PER_NS, ATTOS_PER_PS, ATTOS_PER_SEC_I128, ATTOS_PER_US, Dt,
    Real, SEC_PER_DAYI64, SEC_PER_DAYI128, SEC_PER_WEEK, Scale,
    TAI_SECS_1970_MIDNIGHT_TO_2000_NOON,
};

impl Dt {
    /// The library’s internal reference epoch: exactly **2000-01-01 12:00:00 TAI**.
    ///
    /// [`Dt::new(0)`].
    pub const ZERO: Self = Self::new(0, Scale::TAI, Scale::TAI);

    /// NTP epoch.
    /// - 1900-01-01 midnight UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -3_155_716_800_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const NTP_EPOCH: Self =
        Self::new(-3155716800000000000000000000i128, Scale::TAI, Scale::TAI);

    /// UNIX epoch.
    /// - 1970-01-01 midnight TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -946_728_000_000_000_000_000_000_000 attoseconds
    /// - Does not take into account historical UTC offsets from the "rubber time" era.
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const UNIX_EPOCH: Self = Self::new(
        -(TAI_SECS_1970_MIDNIGHT_TO_2000_NOON as i128) * ATTOS_PER_SEC_I128,
        Scale::TAI,
        Scale::UTC,
    );

    /// TT/TCG/TCB/TDB epoch.
    /// - 1977-01-01 midnight TAI.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -725_803_200_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const TAI_1977_EPOCH: Self =
        Self::new(-725803200000000000000000000i128, Scale::TAI, Scale::TAI);

    /// Chandra X-ray Center (CXC) Time epoch.
    /// - 1998-01-01 midnight TT.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -63_115_232_184_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const CXC_EPOCH: Self = Self::new(-63115232184000000000000000i128, Scale::TAI, Scale::TT);

    /// GPS/Galileo Experiment (GALEX) Time epoch.
    /// - **1980-01-06 00:00:00 UTC**.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -630_763_181_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const GPS_EPOCH: Self = Self::new(-630763181000000000000000000i128, Scale::TAI, Scale::GPS);

    /// Galileo System Time (GST) epoch.
    /// - 1999-08-22 00:00:00 GST.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - -11_447_981_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const GALILEO_EPOCH: Self =
        Self::new(-11447981000000000000000000i128, Scale::TAI, Scale::GST);

    /// BeiDou Time (BDT) epoch.
    /// - 2006-01-01 00:00:00 UTC.
    /// - Stored here on the **TAI** timescale as an offset from [`Self::ZERO`].
    /// - 189_345_633_000_000_000_000_000_000 attoseconds
    /// - The library's epoch for time scales during conversions is 2000-01-01 noon.
    pub const BDT_EPOCH: Self = Self::new(189345633000000000000000000i128, Scale::TAI, Scale::BDT);

    /// Maximum representable duration.
    pub const MAX: Self = Self::new(i128::MAX, Scale::TAI, Scale::TAI);

    /// Minimum (most negative) representable duration.
    pub const MIN: Self = Self::new(i128::MIN, Scale::TAI, Scale::TAI);

    pub const SEC_19: Self = Self::new(19i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);
    pub const SEC_33: Self = Self::new(33i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);
    pub const SEC_37: Self = Self::new(37i128 * ATTOS_PER_SEC_I128, Scale::TAI, Scale::TAI);
    pub const ONE_DAY: Self = Self::new(
        (SEC_PER_DAYI64 as i128) * ATTOS_PER_SEC_I128,
        Scale::TAI,
        Scale::TAI,
    );

    /// Creates a new `Dt` from a total number of attoseconds (signed i128).
    #[inline(always)]
    pub const fn new(attos: i128, scale: Scale, target: Scale) -> Self {
        Self {
            attos,
            scale,
            target,
        }
    }

    /// Creates a new [`Dt`] from a total number of attoseconds (signed i128) without
    /// performing any time scale conversions.
    ///
    /// Easy way to create a duration.
    #[inline(always)]
    pub const fn span(attos: i128) -> Self {
        Dt::new(attos, Scale::TAI, Scale::TAI)
    }

    /// Creates a [`Dt`] from a floating-point number of seconds.
    #[inline(always)]
    pub const fn span_f(sec_f: Real) -> Self {
        Self::from_sec_f_on(sec_f, Scale::TAI)
    }

    #[inline(always)]
    pub(crate) fn from_sec_and_attos(sec: i64, attos: u64, scale: Scale) -> Dt {
        if attos == 0 {
            Dt::from_attos((sec as i128) * ATTOS_PER_SEC_I128, scale)
        } else {
            Dt::from_attos((sec as i128) * ATTOS_PER_SEC_I128 + (attos as i128), scale)
        }
    }

    #[inline]
    pub const fn from_tai_sec(sec: i128) -> Self {
        Self::from_attos(sec.saturating_mul(ATTOS_PER_SEC_I128), Scale::TAI)
    }

    #[inline]
    pub const fn from_sec(sec: i128, scale: Scale) -> Self {
        Self::from_attos(sec.saturating_mul(ATTOS_PER_SEC_I128), scale)
    }

    #[inline]
    pub const fn from_ms(ms: i128, scale: Scale) -> Self {
        let attos = ms.saturating_mul(ATTOS_PER_MS as i128);
        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_us(us: i128, scale: Scale) -> Self {
        let attos = us.saturating_mul(ATTOS_PER_US as i128);
        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_ns(ns: i128, scale: Scale) -> Self {
        let attos = ns.saturating_mul(ATTOS_PER_NS as i128);
        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_ps(ps: i128, scale: Scale) -> Self {
        let attos = ps.saturating_mul(ATTOS_PER_PS as i128);
        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_fs(fs: i128, scale: Scale) -> Self {
        let attos = fs.saturating_mul(ATTOS_PER_FS as i128);
        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_min(m: i64, scale: Scale) -> Self {
        Self::from_sec((m as i128) * 60, scale)
    }

    #[inline]
    pub const fn from_hr(h: i64, scale: Scale) -> Self {
        Self::from_sec((h as i128) * 3600, scale)
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
            return Self::from_sec(total_sec as i128, scale);
        }

        let abs_ns = sub_ns.unsigned_abs();
        let extra_sec = (abs_ns / 1_000_000_000u128) as i64;
        let rem_ns = abs_ns % 1_000_000_000u128;
        let frac_attos = (rem_ns as u64) * ATTOS_PER_NS;

        let attos = if sub_ns >= 0 {
            (total_sec as i128 + extra_sec as i128) * ATTOS_PER_SEC_I128 + frac_attos as i128
        } else if frac_attos == 0 {
            (total_sec as i128 - extra_sec as i128) * ATTOS_PER_SEC_I128
        } else {
            (total_sec as i128 - extra_sec as i128 - 1) * ATTOS_PER_SEC_I128
                + (ATTOS_PER_SEC_I128 - frac_attos as i128)
        };

        Self::from_attos(attos, scale)
    }

    #[inline]
    pub const fn from_days(d: i64, scale: Scale) -> Dt {
        Self::from_sec((d as i128).saturating_mul(SEC_PER_DAYI128), scale)
    }

    #[inline]
    pub const fn wk(wk: i64, scale: Scale) -> Dt {
        Dt::from_sec((wk as i128).saturating_mul(SEC_PER_WEEK as i128), scale)
    }

    #[inline]
    pub const fn yr(yr: i64, scale: Scale) -> Dt {
        Dt::from_sec((yr as i128).saturating_mul(31_557_600), scale)
    }

    /// Returns a `Dt` that is this duration ago from the given scale.
    #[inline]
    pub const fn ago(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).sub(self)
    }

    /// Returns a `Dt` that is this duration from now in the given scale.
    #[inline]
    pub const fn from_now(self, scale: Scale) -> Dt {
        Dt::from_attos(0, scale).add(self)
    }

    /// Returns the negation of this duration.
    #[inline]
    pub const fn neg(self) -> Self {
        Dt::new(-self.attos, self.scale, self.target)
    }

    /// Returns the positive of this duration.
    #[inline]
    pub const fn abs(self) -> Self {
        Dt::new(self.attos.saturating_abs(), self.scale, self.target)
    }

    /// Creates a [`Dt`] from a floating-point number of seconds.
    #[inline]
    pub const fn from_sec_f(sec_f: Real, scale: Scale) -> Self {
        Self::from_sec_f_on(sec_f, scale)
    }

    /// High-precision conversion from [`Real`] seconds to total attoseconds (i128).
    /// Uses IEEE 754 bit extraction + exact integer multiplication by 5^18.
    pub const fn sec_f_to_total_attos(sec_f: Real) -> i128 {
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

        Self::from_attos(total_attos, s)
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
        Ok(Dt::from_diff_and_scale(
            Dt::new(Dt::sec_to_attos(secs as i128), Scale::TAI, Scale::UTC),
            Dt::UNIX_EPOCH,
            false,
        )
        .add(Dt::from_ns(nanos as i128, Scale::TAI)))
    }

    /// Returns the current system time as TAI from 2000-01-01 noon.
    /// (browser WASM version using JavaScript’s `Date.now()`).
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    #[inline]
    pub fn now() -> Result<Self, DtErr> {
        let ms: f64 = js_sys::Date::now();
        let secs = (ms / 1000.0).floor() as i128;
        let nanos = ((ms % 1000.0) * 1_000_000.0) as i128;
        Ok(Dt::from_diff_and_scale(
            Dt::new(Dt::sec_to_attos(secs), Scale::TAI, Scale::UTC),
            Dt::UNIX_EPOCH,
            false,
        )
        .add(Dt::from_ns(nanos as i128, Scale::TAI)))
    }
}
