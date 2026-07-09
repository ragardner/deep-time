//! Fundamental constants for time-scale conversions,
//! relativistic corrections, and astronomical calculations.

use crate::{Dt, Real, Scale};

/// The size limit for parsing and no-alloc formatting with
/// the strtime related functionality.
pub const STRTIME_SIZE: usize = 512;

/// Number of decimal digits in one attosecond-precision second (`10¹⁸`).
pub const ATTOS_DIGITS: usize = 18;

/// Seconds in one Julian year (365.25 days × 86_400).
pub const SEC_PER_YEAR: i128 = 31_557_600;

/// Seconds in one average month (30.4375 days × 86_400).
pub const SEC_PER_MONTH: i128 = 2_629_800;

/// Seconds in one standard Earth day (24 × 60 × 60).
pub const SEC_PER_DAY: i128 = 86_400;

/// 86,400 seconds in one standard Earth day
/// (24 hours × 60 minutes × 60 seconds).
pub const SEC_PER_DAY_F: Real = 86_400.0;

/// 86,400 seconds in one standard Earth day
/// (24 hours × 60 minutes × 60 seconds).
pub const SEC_PER_DAY_I64: i64 = 86_400;

/// Seconds in one minute.
pub const SEC_PER_MIN: i128 = 60;

/// Seconds in one hour.
pub const SEC_PER_HOUR: i128 = 3600;

/// Attoseconds in one minute.
pub const ATTOS_PER_MIN: i128 = SEC_PER_MIN * ATTOS_PER_SEC_I128;

/// Attoseconds in one hour.
pub const ATTOS_PER_HOUR: i128 = SEC_PER_HOUR * ATTOS_PER_SEC_I128;

/// Seconds in one GPS week (7 days).
pub const SEC_PER_WEEK: i64 = 7 * SEC_PER_DAY_I64;

/// Attoseconds in one GPS week.
pub const ATTOS_PER_WEEK: i128 = SEC_PER_WEEK as i128 * ATTOS_PER_SEC_I128;

/// Attoseconds in one standard Earth day.
pub const ATTOS_PER_DAY: i128 = SEC_PER_DAY * ATTOS_PER_SEC_I128;

/// Attoseconds in half a standard Earth day (12 hours).
pub const ATTOS_PER_HALF_DAY: i128 = ATTOS_PER_DAY / 2;

/// Attoseconds in half a standard Earth day, as `u128`.
pub const ATTOS_PER_HALF_DAY_U128: u128 = ATTOS_PER_HALF_DAY as u128;

/// Attoseconds per second.
pub const ATTOS_PER_SEC: u64 = 1_000_000_000_000_000_000;

/// Attoseconds per second as a floating-point value.
pub const ATTOS_PER_SECF: Real = f!(1_000_000_000_000_000_000.0);

/// Attoseconds per second as `i128`.
pub const ATTOS_PER_SEC_I128: i128 = ATTOS_PER_SEC as i128;

/// Attoseconds per second as `u128`.
pub const ATTOS_PER_SEC_U128: u128 = ATTOS_PER_SEC as u128;

/// Attoseconds per nanosecond (10⁻⁹ s).
pub const ATTOS_PER_NS: u64 = 1_000_000_000;

/// Attoseconds per millisecond (10⁻³ s).
pub const ATTOS_PER_MS_I128: i128 = 1_000_000_000_000_000;

/// Attoseconds per microsecond (10⁻⁶ s).
pub const ATTOS_PER_US_I128: i128 = 1_000_000_000_000;

/// Attoseconds per nanosecond (10⁻⁹ s).
pub const ATTOS_PER_NS_I128: i128 = ATTOS_PER_NS as i128;

/// Attoseconds per picosecond (10⁻¹² s).
pub const ATTOS_PER_PS_I128: i128 = 1_000_000;

/// Attoseconds per femtosecond (10⁻¹⁵ s).
pub const ATTOS_PER_FS_I128: i128 = 1_000;

/// Fractional part of the TT–TAI offset (0.184 s) as attoseconds.
pub const TT_TAI_OFFSET_ATTOS: u64 = 184_000_000_000_000_000; // 0.184 × 10¹⁸

/// TT–TAI offset of 32.184 s as a [`Dt`].
pub const TT_TAI_OFFSET: Dt = Dt::new(32_184_000_000_000_000_000i128, Scale::TAI, Scale::TAI);

/// Julian Date of the J2000.0 epoch (JD 2451545.0).
pub const JD_2000_2_451_545: i64 = 2_451_545;

/// Julian Date of the J2000.0 epoch as `i128`.
pub const JD_2000_2_451_545_I128: i128 = JD_2000_2_451_545 as i128;

/// Julian Date of the J2000.0 epoch as a floating-point value.
pub const JD_2000_2_451_545F: Real = f!(2_451_545.0);

/// Modified Julian Date of the Unix epoch (MJD 40587.0 = 1970-01-01 00:00:00 UTC).
pub const MJD_1970: i64 = 40_587;

/// Number of TAI attoseconds from noon 2000-01-01 back to midnight 1972-01-01.
pub const TAI_ATTOS_AT_1972: i128 = -883_655_990_000_000_000_000_000_000;

/// TAI seconds from 1970-01-01 midnight to 2000-01-01 noon.
pub const TAI_SECS_1970_MIDNIGHT_TO_2000_NOON: i64 = 946_728_000;

/// Numerator of L_G = 6.969290134 × 10^{-10} (IAU) as a fixed-point fraction.
pub(crate) const LG_NUM: i128 = 6_969_290_134;

/// Denominator of L_G (10¹⁹) for the fixed-point fraction with [`LG_NUM`].
pub(crate) const LG_DEN: i128 = 10_000_000_000_000_000_000; // 10^19

/// Numerator of L_B = 1.550519768 × 10^{-8} (IAU) as a fixed-point fraction.
pub(crate) const LB_NUM: i128 = 1_550_519_768;

/// Denominator of L_B (10¹⁷) for the fixed-point fraction with [`LB_NUM`].
pub(crate) const LB_DEN: i128 = 100_000_000_000_000_000; // 10^17

/// Integer day part of the TCG/TCB reference epoch JD 2443144.5003725.
pub(crate) const TCG_TCB_REF_JD_INT: i64 = 2_443_144;

/// Time-of-day seconds of the TCG/TCB reference epoch
/// (0.5003725 × 86400 = 43232.184, integer part).
pub(crate) const TCG_TCB_REF_TOD_SEC: i64 = 43_232; // 0.5003725 * 86400 = 43232.184

/// Sub-second attoseconds of the TCG/TCB reference epoch time-of-day
/// (the 0.184 s fractional part, same as [`TT_TAI_OFFSET_ATTOS`]).
pub(crate) const TCG_TCB_REF_TOD_SUBSEC: u64 = TT_TAI_OFFSET_ATTOS;

/// Attoseconds since J2000.0 TT of the TCG/TCB reference epoch
/// (JD 2443144.5003725 TT). Computed from the existing reference constants.
pub(crate) const TCG_TCB_REF_ATTOS_SINCE_J2000: i128 = {
    let days_since_j2000 = (TCG_TCB_REF_JD_INT - JD_2000_2_451_545) as i128;
    let sec_part = days_since_j2000 * SEC_PER_DAY + (TCG_TCB_REF_TOD_SEC as i128);
    sec_part * ATTOS_PER_SEC_I128 + (TCG_TCB_REF_TOD_SUBSEC as i128)
};

/// TDB₀ = −65.5 µs expressed in attoseconds.
pub(crate) const TDB0_ATTOS: i128 = -65_500_000_000_000;

/// Solar gravitational parameter GM☉ in m³ s⁻²
/// (nominal value from IAU 2015 Resolution B3)
#[cfg(feature = "physics")]
pub const GM_SUN: Real = 1.3271244e20;

/// Speed of light in m/s (SI definition)
#[cfg(feature = "physics")]
pub const C: Real = 299792458.0;

/// Speed of light squared (c²) in m² s⁻².
#[cfg(feature = "physics")]
pub const C_SQUARED: Real = C * C;

/// GM☉ / c³ in seconds (from `GM_SUN` and `C` — used in Shapiro delay)
#[cfg(feature = "physics")]
pub const GM_SUN_OVER_C3: Real = GM_SUN / (C * C_SQUARED);

/// 2GM☉ / c³ — the standard prefactor in the one-way Shapiro delay formula
#[cfg(feature = "physics")]
pub const TWO_GM_SUN_OVER_C3: Real = 2.0 * GM_SUN_OVER_C3;

/// Planck length ℓ_Pl in meters (standard value).
///
/// This is raised to the fourth power to form the dimensionless curvature
/// parameter `x = ℓ_Pl⁴ × 𝒦` inside the master Lagrangian. The term only
/// affects the proper-time rate at extreme (Planckian) curvatures.
/// See the [relativistic timing model](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).
#[cfg(feature = "physics")]
pub const PLANCK_LENGTH: Real = 1.616255e-35;

/// Planck length to the fourth power (ℓ_Pl⁴) in m⁴.
///
/// This is the coefficient actually used at runtime:
///
/// ```text
/// let x = PLANCK_LENGTH_4 * kretschmann;
/// ```
///
/// The fourth power produces a dimensionless `x` because the Kretschmann
/// scalar has units of L⁻⁴. Information on the underlying model can be found
/// [here](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).
#[cfg(feature = "physics")]
pub const PLANCK_LENGTH_4: Real = PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH;
