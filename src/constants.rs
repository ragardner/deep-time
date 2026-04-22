use crate::{Delta, Real};

/// Exactly 86,400 seconds in one standard Earth day  
/// (24 hours × 60 minutes × 60 seconds).
pub(crate) const SEC_PER_DAY: Real = 86_400.0;
pub(crate) const SEC_PER_DAYI64: i64 = 86_400;
pub(crate) const SEC_PER_DAYI128: i128 = 86_400;
pub(crate) const SEC_PER_HALF_DAYI64: i64 = 43_200;

/// Solar gravitational parameter GM☉ in m³ s⁻²  
/// (exact nominal value from IAU 2015 Resolution B3)
pub const GM_SUN: Real = 1.3271244e20;

/// Speed of light in m/s (exact SI definition)
pub const C: Real = 299792458.0;

/// Speed of light squared (c²) in m² s⁻².  
pub const C_SQUARED: Real = C * C;

/// GM☉ / c³ in seconds (exact from `GM_SUN` and `C` — used in Shapiro delay)
pub const GM_SUN_OVER_C3: Real = GM_SUN / (C * C_SQUARED);

/// 2GM☉ / c³ — the standard prefactor in the one-way Shapiro delay formula
pub const TWO_GM_SUN_OVER_C3: Real = 2.0 * GM_SUN_OVER_C3;

/// Attoseconds per second.
pub const ATTOSEC_PER_SEC: u64 = 1_000_000_000_000_000_000;
pub(crate) const ATTOSEC_PER_SEC_I128: i128 = 1_000_000_000_000_000_000;

/// Attoseconds per millisecond (10⁻³ s).
pub const ATTOSEC_PER_MILLISEC: u64 = 1_000_000_000_000_000;
/// Attoseconds per microsecond (10⁻⁶ s).
pub const ATTOSEC_PER_MICROSEC: u64 = 1_000_000_000_000;
/// Attoseconds per nanosecond (10⁻⁹ s).
pub const ATTOSEC_PER_NANOSEC: u64 = 1_000_000_000;
/// Attoseconds per picosecond (10⁻¹² s).
pub const ATTOSEC_PER_PICOSEC: u64 = 1_000_000;
/// Attoseconds per femtosecond (10⁻¹⁵ s).
pub const ATTOSEC_PER_FEMTOSEC: u64 = 1_000;
/// Attoseconds per attosecond (by definition).
pub const ATTOSEC_PER_ATTOSEC: u64 = 1;

/// TT = TAI + exactly 32.184 s
pub(crate) const TT_TAI_OFFSET_SEC: i64 = 32;
pub(crate) const TT_TAI_OFFSET_SUBSEC: u64 = 184_000_000_000_000_000; // 0.184 × 10¹⁸

/// Helper that returns the exact TT–TAI offset as a `Delta`.
pub const TT_TAI_OFFSET_DELTA: Delta = Delta::new(TT_TAI_OFFSET_SEC, TT_TAI_OFFSET_SUBSEC);

// J2000.0 = 2000-01-01 12:00:00 TT → 100 Julian years = exactly 3_155_760_000 s
pub(crate) const J2000_SECONDS_PER_CENTURY: Real = 3_155_760_000.0;

/// Julian Date of the J2000.0 epoch in Terrestrial Time (TT).
pub(crate) const J2000_JD_TT: i64 = 2_451_545;
/// Seconds from the Unix epoch (1970-01-01 00:00:00 UTC) to J2000.0 noon
/// (2000-01-01 12:00:00 UTC).
pub(crate) const UNIX_EPOCH_TO_J2000_NOON_UTC: i64 = 946_728_000;

/// Exact mean length of one Martian sol in Earth seconds (NASA GISS / AM2000)
pub const MARS_SOL_LENGTH_SEC: Real = 88775.244;

pub(crate) const PLANCK_LENGTH: Real = 1.616255e-35; // meters (standard value)
pub(crate) const PLANCK_LENGTH_4: Real =
    PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH * PLANCK_LENGTH;

//

/// L_G = 6.969290134 × 10^{-10} (exact IAU) as fixed-point fraction.
pub(crate) const LG_NUM: i128 = 6_969_290_134;
pub(crate) const LG_DEN: i128 = 10_000_000_000_000_000_000; // 10^19

/// L_B = 1.550519768 × 10^{-8} (exact IAU) as fixed-point fraction.
pub(crate) const LB_NUM: i128 = 1_550_519_768;
pub(crate) const LB_DEN: i128 = 100_000_000_000_000_000; // 10^17

/// TCG/TCB reference epoch (JD 2443144.5003725) broken into integer parts for exact math.
pub(crate) const TCG_TCB_REF_JD_INT: i64 = 2_443_144;
pub(crate) const TCG_TCB_REF_TOD_SEC: i64 = 43_232; // 0.5003725 * 86400 = 43232.184
pub(crate) const TCG_TCB_REF_TOD_SUBSEC: u64 = TT_TAI_OFFSET_SUBSEC;

/// TDB₀ = −65.5 µs expressed in attoseconds (exact).
pub(crate) const TDB0_ATTOS: i128 = -65_500_000_000_000;

/// L_M = 6.48378 × 10^{-10} (exact secular rate from Ashby & Patla 2024 NIST for LTC ↔ TT)
/// as fixed-point fraction.
pub(crate) const LM_NUM: i128 = 648_378;
pub(crate) const LM_DEN: i128 = 1_000_000_000_000_000; // 10^15

/// Mars MSD reference epoch (JD 2405522.0028779 TT) broken into integer parts for exact math.
pub(crate) const MARS_MSD_REF_JD_INT: i64 = 2_405_522;
pub(crate) const MARS_MSD_REF_TOD_SEC: i64 = 248;
pub(crate) const MARS_MSD_REF_TOD_SUBSEC: u64 = 650_560_000_000_000_000;

/// Martian mean sol length in attoseconds (88775.244 s × 10¹⁸).
pub(crate) const MARS_SOL_ATTOS: i128 = 88_775_244_000_000_000_000_000;

/// Precomputed numerical values of the Mars reference epoch on the TT scale (seconds since J2000).
pub(crate) const MARS_REF_SEC: i64 = -3_976_386_952;
pub(crate) const MARS_REF_SUBSEC: u64 = 650_560_000_000_000_000;

pub(crate) const WEEKDAYS_FULL: [&[u8]; 7] = [
    b"Sunday",
    b"Monday",
    b"Tuesday",
    b"Wednesday",
    b"Thursday",
    b"Friday",
    b"Saturday",
];
pub(crate) const WEEKDAYS_ABBR: [&[u8]; 7] =
    [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
pub(crate) const MONTHS_FULL: [&[u8]; 12] = [
    b"January",
    b"February",
    b"March",
    b"April",
    b"May",
    b"June",
    b"July",
    b"August",
    b"September",
    b"October",
    b"November",
    b"December",
];
pub(crate) const MONTHS_ABBR: [&[u8]; 12] = [
    b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec",
];

pub(crate) const STRFTIME_SIZE: usize = 512;
