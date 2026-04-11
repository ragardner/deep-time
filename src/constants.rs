use crate::Delta;

/// Microquectoseconds per second.
pub const MICROQUECTOS_PER_SEC: u128 = 10u128.pow(36);
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

// 10¹⁵ is exactly representable in f64 (within 53-bit mantissa).
// 10²¹ completes the 36-digit scale exactly in u128.
pub(crate) const POW15: u128 = 1_000_000_000_000_000;
pub(crate) const POW21: u128 = MICROQUECTOS_PER_SEC / POW15; // exactly 10²¹

/// L_G = 6.969290134 × 10^{-10} (exact IAU defining constant for TCG ↔ TT)
pub(crate) const LG: f64 = 6.969290134e-10;
/// L_B = 1.550519768 × 10^{-8} (exact IAU defining constant for TCB ↔ TDB)
pub(crate) const LB: f64 = 1.550519768e-8;
/// Reference epoch T₀ = 2443144.5003725 JD (1977 Jan 1.0 TAI at geocenter)
pub(crate) const TCG_TCB_REF_JD: f64 = 2443144.5003725;
/// TDB₀ = −65.5 µs (exact IAU 2006 constant)
pub(crate) const TDB0: Delta = Delta::from_sec_f64(-0.0000655);
