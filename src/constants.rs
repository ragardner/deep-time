//! Fundamental constants for time-scale conversions,
//! relativistic corrections, and astronomical calculations.

use crate::{Dt, Real};

pub const STRFTIME_SIZE: usize = 512;

pub(crate) const SEC_PER_YEAR: i128 = 31_557_600; // 365.25 days × 86_400
pub(crate) const SEC_PER_MONTH: i128 = 2_629_800; // 30.4375 days × 86_400
pub(crate) const SEC_PER_DAY: i128 = 86_400;

/// 86,400 seconds in one standard Earth day  
/// (24 hours × 60 minutes × 60 seconds).
pub const SEC_PER_DAY_F: Real = 86_400.0;
pub const SEC_PER_DAYI64: i64 = 86_400;
pub(crate) const SEC_PER_DAYI128: i128 = 86_400;

/// Seconds in one GPS week (7 days).
pub(crate) const SEC_PER_WEEK: i64 = 7 * SEC_PER_DAYI64;
/// Attoseconds in one GPS week.
pub(crate) const ATTOS_PER_WEEK: i128 = SEC_PER_WEEK as i128 * ATTOS_PER_SEC_I128;
pub const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOS_PER_SEC_I128;
pub const ATTOS_PER_HALF_DAY: i128 = ATTOS_PER_DAY / 2;
pub const ATTOS_PER_HALF_DAYU: u128 = ATTOS_PER_HALF_DAY as u128;

/// Solar gravitational parameter GM☉ in m³ s⁻²  
/// (nominal value from IAU 2015 Resolution B3)
pub const GM_SUN: Real = 1.3271244e20;

/// Speed of light in m/s (SI definition)
pub const C: Real = 299792458.0;

/// Speed of light squared (c²) in m² s⁻².  
pub const C_SQUARED: Real = C * C;

/// GM☉ / c³ in seconds (from `GM_SUN` and `C` — used in Shapiro delay)
pub const GM_SUN_OVER_C3: Real = GM_SUN / (C * C_SQUARED);

/// 2GM☉ / c³ — the standard prefactor in the one-way Shapiro delay formula
pub const TWO_GM_SUN_OVER_C3: Real = 2.0 * GM_SUN_OVER_C3;

/// Attoseconds per second.
pub const ATTOS_PER_SEC: u64 = 1_000_000_000_000_000_000;
pub const ATTOS_PER_SECF: Real = f!(1_000_000_000_000_000_000.0);
pub const ATTOS_PER_SEC_I128: i128 = ATTOS_PER_SEC as i128;
pub const ATTOS_PER_SEC_U128: u128 = ATTOS_PER_SEC as u128;

/// Attoseconds per millisecond (10⁻³ s).
pub const ATTOS_PER_MS: u64 = 1_000_000_000_000_000;
/// Attoseconds per microsecond (10⁻⁶ s).
pub const ATTOS_PER_US: u64 = 1_000_000_000_000;
/// Attoseconds per nanosecond (10⁻⁹ s).
pub const ATTOS_PER_NS: u64 = 1_000_000_000;
/// Attoseconds per picosecond (10⁻¹² s).
pub const ATTOS_PER_PS: u64 = 1_000_000;
/// Attoseconds per femtosecond (10⁻¹⁵ s).
pub const ATTOS_PER_FS: u64 = 1_000;
/// Attoseconds per millisecond (10⁻³ s).
pub const ATTOS_PER_MS_I128: i128 = ATTOS_PER_MS as i128;
/// Attoseconds per microsecond (10⁻⁶ s).
pub const ATTOS_PER_US_I128: i128 = ATTOS_PER_US as i128;
/// Attoseconds per nanosecond (10⁻⁹ s).
pub const ATTOS_PER_NS_I128: i128 = ATTOS_PER_NS as i128;
/// Attoseconds per picosecond (10⁻¹² s).
pub const ATTOS_PER_PS_I128: i128 = ATTOS_PER_PS as i128;
/// Attoseconds per femtosecond (10⁻¹⁵ s).
pub const ATTOS_PER_FS_I128: i128 = ATTOS_PER_FS as i128;

/// TT = TAI + 32.184 s
pub(crate) const TT_TAI_OFFSET_SEC: i64 = 32;
pub(crate) const TT_TAI_OFFSET_SUBSEC: u64 = 184_000_000_000_000_000; // 0.184 × 10¹⁸

/// Helper that returns the TT–TAI offset as a `Dt`.
pub const TT_TAI_OFFSET: Dt = Dt::new(TT_TAI_OFFSET_SEC, TT_TAI_OFFSET_SUBSEC);

/// Julian Date of the J2000.0 epoch.
pub const JD_2000_2_451_545: i64 = 2_451_545;
pub const JD_2000_2_451_545F: Real = f!(2_451_545.0);
/// MJD 40587.0 = 1970-01-01 00:00:00 UTC
pub const MJD_1970: i64 = 40_587;
/// Number of TAI seconds backwards from noon 2000-01-01 to midnight 1972-01-01
pub const TAI_SEC_AT_1972: i64 = -883_655_990;

/// TAI secs from 1970-01-01 midnight to 2000-01-01 noon
pub(crate) const TAI_SECS_1970_MIDNIGHT_TO_2000_NOON: i64 = 946_728_000;

pub const PLANCK_LENGTH: Real = 1.616255e-35; // meters (standard value)
pub const PLANCK_LENGTH_4: Real = PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH;

/// L_G = 6.969290134 × 10^{-10} (IAU) as fixed-point fraction.
pub(crate) const LG_NUM: i128 = 6_969_290_134;
pub(crate) const LG_DEN: i128 = 10_000_000_000_000_000_000; // 10^19

/// L_B = 1.550519768 × 10^{-8} (IAU) as fixed-point fraction.
pub(crate) const LB_NUM: i128 = 1_550_519_768;
pub(crate) const LB_DEN: i128 = 100_000_000_000_000_000; // 10^17

/// TCG/TCB reference epoch (JD 2443144.5003725) broken into integer parts for exact math.
pub(crate) const TCG_TCB_REF_JD_INT: i64 = 2_443_144;
pub(crate) const TCG_TCB_REF_TOD_SEC: i64 = 43_232; // 0.5003725 * 86400 = 43232.184
pub(crate) const TCG_TCB_REF_TOD_SUBSEC: u64 = TT_TAI_OFFSET_SUBSEC;

/// Attoseconds since J2000.0 TT of the TCG/TCB reference epoch
/// (JD 2443144.5003725 TT). Computed from the existing reference constants.
pub(crate) const TCG_TCB_REF_ATTOS_SINCE_J2000: i128 = {
    let days_since_j2000 = (TCG_TCB_REF_JD_INT - JD_2000_2_451_545) as i128;
    let sec_part = days_since_j2000 * SEC_PER_DAYI128 + (TCG_TCB_REF_TOD_SEC as i128);
    sec_part * ATTOS_PER_SEC_I128 + (TCG_TCB_REF_TOD_SUBSEC as i128)
};

/// TDB₀ = −65.5 µs expressed in attoseconds.
pub(crate) const TDB0_ATTOS: i128 = -65_500_000_000_000;
