use crate::{Delta, DtBig, Real};

/// Exactly 86,400 seconds in one standard Earth day  
/// (24 hours × 60 minutes × 60 seconds).
///
/// This is the fixed length of a **mean solar day** used everywhere in the
/// library for converting between total seconds and fractional Julian Dates
/// (JD) on TT-based scales (TCG, TCB, LTC, etc.).  
///
/// It is **not** leap-second aware — leap seconds are handled only in the
/// UTC ↔ TAI conversion path.
pub(crate) const SEC_PER_DAY: Real = 86_400.0;

/// Solar gravitational parameter GM☉ in m³ s⁻²  
/// (exact nominal value from IAU 2015 Resolution B3)
pub const GM_SUN: Real = 1.3271244e20;

/// Speed of light in m/s (exact SI definition)
pub const C: Real = 299792458.0;

/// Speed of light squared (c²) in m² s⁻².  
/// Computed at compile time from the exact SI value of `C` — guarantees perfect consistency
/// for weak-field relativistic calculations (e.g. Schwarzschild radius, post-Newtonian terms).
pub const C_SQUARED: Real = C * C;

/// GM☉ / c³ in seconds (exact from `GM_SUN` and `C` — used in Shapiro delay)
pub const GM_SUN_OVER_C3: Real = GM_SUN / (C * C_SQUARED);

/// 2GM☉ / c³ — the standard prefactor in the one-way Shapiro delay formula
pub const TWO_GM_SUN_OVER_C3: Real = 2.0 * GM_SUN_OVER_C3;

/// Microquectoseconds per second.
pub const MICROQUECTOS_PER_SEC: u128 = 10u128.pow(36);
pub(crate) const MQS: DtBig = DtBig::from_u128(MICROQUECTOS_PER_SEC);

/// Microquectoseconds per millisecond (10⁻³ s).
pub const MICROQUECTOS_PER_MILLISEC: u128 = 10u128.pow(33);
/// Microquectoseconds per microsecond (10⁻⁶ s).
pub const MICROQUECTOS_PER_MICROSEC: u128 = 10u128.pow(30);
/// Microquectoseconds per nanosecond (10⁻⁹ s).
pub const MICROQUECTOS_PER_NANOSEC: u128 = 10u128.pow(27);
/// Microquectoseconds per picosecond (10⁻¹² s).
pub const MICROQUECTOS_PER_PICOSEC: u128 = 10u128.pow(24);
/// Microquectoseconds per femtosecond (10⁻¹⁵ s).
pub const MICROQUECTOS_PER_FEMTOSEC: u128 = 10u128.pow(21);
/// Microquectoseconds per attosecond (10⁻¹⁸ s).
pub const MICROQUECTOS_PER_ATTOSEC: u128 = 10u128.pow(18);
/// Microquectoseconds per zeptosecond (10⁻²¹ s).
pub const MICROQUECTOS_PER_ZEPTOSEC: u128 = 10u128.pow(15);
/// Microquectoseconds per yoctosecond (10⁻²⁴ s).
pub const MICROQUECTOS_PER_YOCTOSEC: u128 = 10u128.pow(12);
/// Microquectoseconds per rontosecond (10⁻²⁷ s).
pub const MICROQUECTOS_PER_RONTOSEC: u128 = 10u128.pow(9);
/// Microquectoseconds per quectosecond (10⁻³⁰ s).
pub const MICROQUECTOS_PER_QUECTOSEC: u128 = 10u128.pow(6);
/// Microquectoseconds per microquectosecond (by definition).
pub const MICROQUECTOS_PER_MICROQUECTOSEC: u128 = 1;
/// TT = TAI + exactly 32.184 s (exact integer form — required because f64
/// cannot represent 0.184 * 10³⁶ accurately).
pub(crate) const TT_TAI_OFFSET_SEC: i128 = 32;
pub(crate) const TT_TAI_OFFSET_SUBSEC: u128 = 184 * 10u128.pow(33); // 0.184 × 10³⁶ exactly

/// Helper that returns the exact TT–TAI offset as a `Delta`.
pub const TT_TAI_OFFSET_DELTA: Delta = Delta::new(TT_TAI_OFFSET_SEC, TT_TAI_OFFSET_SUBSEC);
// J2000.0 = 2000-01-01 12:00:00 TT → 100 Julian years = exactly 3_155_760_000 s
pub(crate) const J2000_SECONDS_PER_CENTURY: Real = 3_155_760_000.0;

/// Julian Date of the J2000.0 epoch in Terrestrial Time (TT).
///
/// By international convention (IAU), J2000.0 is defined as the instant
/// 2000 January 1.5 TT, which corresponds exactly to Julian Date 2451545.0 TT.
/// This integer value is the fixed reference point from which all absolute
/// Julian Dates returned by `to_jd_tt_exact` are measured; it is subtracted
/// when converting back from an absolute JD into library-internal seconds
/// since the J2000 epoch.
pub(crate) const J2000_JD_TT: i128 = 2_451_545;

/// Exact mean length of one Martian sol in Earth seconds (NASA GISS / AM2000)
pub const MARS_SOL_LENGTH_SEC: Real = 88775.244;
/// Reference constant for the MSD formula (JD_TT basis, NASA GISS Mars24)
pub(crate) const MARS_MSD_JD_REF: Real = 2405522.0028779;
/// Mean number of Earth days in one Martian sol (for the division)
pub(crate) const MARS_SOL_IN_EARTH_DAYS: Real = 1.0274912517;

// 10¹⁵ is exactly representable in f64 (within 53-bit mantissa).
// 10²¹ completes the 36-digit scale exactly in u128.
pub(crate) const POW15: u128 = 1_000_000_000_000_000;
pub(crate) const POW21: u128 = MICROQUECTOS_PER_SEC / POW15; // exactly 10²¹

/// L_M = 6.48378 × 10^{-10} (exact secular rate from Ashby & Patla 2024 NIST
/// for LTC ↔ TT). Corresponds to +56.02 µs per Earth day on the lunar selenoid.
/// Eccentricity periodic term (±0.108 µs/day in rate) is not included in the core
/// definition and is added via `ClockModel` when needed.
pub(crate) const LM: Real = 6.48378e-10;
/// L_G = 6.969290134 × 10^{-10} (exact IAU defining constant for TCG ↔ TT)
pub(crate) const LG: Real = 6.969290134e-10;
/// L_B = 1.550519768 × 10^{-8} (exact IAU defining constant for TCB ↔ TDB)
pub(crate) const LB: Real = 1.550519768e-8;
/// Reference epoch T₀ = 2443144.5003725 JD (1977 Jan 1.0 TAI at geocenter)
pub(crate) const TCG_TCB_REF_JD: Real = 2443144.5003725;
/// TDB₀ = −65.5 µs (exact IAU 2006 constant)
pub(crate) const TDB0: Delta = Delta::from_sec_f(-0.0000655);

pub(crate) const PLANCK_LENGTH: Real = 1.616255e-35; // meters (standard value)
pub(crate) const PLANCK_LENGTH_4: Real =
    PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH;
