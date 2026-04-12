//! Ergonomic timestamp constructors for common epochs (optional import).
//!
//! ```
//! use deep_time_core::{
//!     GPSTimestamp, J2000Timestamp, TAITimestamp, UTCTimestamp, UnixTimestamp,
//! };
//!
//! let p1 = 1_741_234_567_i128.unix_seconds();
//! let p2 = 123_456_789_i128.j2000_seconds();
//! let p3 = 1_741_234_567_i128.utc_seconds();
//! let p4 = 1_234_567_890_i128.gps_seconds();
//! ```

use crate::{ClockType, Delta, Timestamp};

// ──────────────────────────────────────────────────────────────
// Traits
// ──────────────────────────────────────────────────────────────

pub trait UnixTimestamp: Copy + Sized {
    fn unix_seconds(self) -> Timestamp;
    fn unix_milliseconds(self) -> Timestamp;
    fn unix_microseconds(self) -> Timestamp;
    fn unix_nanoseconds(self) -> Timestamp;
}

pub trait TAITimestamp: Copy + Sized {
    fn tai_seconds(self) -> Timestamp;
    fn tai_milliseconds(self) -> Timestamp;
    fn tai_microseconds(self) -> Timestamp;
    fn tai_nanoseconds(self) -> Timestamp;
}

pub trait J2000Timestamp: Copy + Sized {
    fn j2000_seconds(self) -> Timestamp;
    fn j2000_milliseconds(self) -> Timestamp;
    fn j2000_microseconds(self) -> Timestamp;
    fn j2000_nanoseconds(self) -> Timestamp;
}

pub trait UTCTimestamp: Copy + Sized {
    fn utc_seconds(self) -> Timestamp;
    fn utc_milliseconds(self) -> Timestamp;
    fn utc_microseconds(self) -> Timestamp;
    fn utc_nanoseconds(self) -> Timestamp;
}

pub trait GPSTimestamp: Copy + Sized {
    fn gps_seconds(self) -> Timestamp;
    fn gps_milliseconds(self) -> Timestamp;
    fn gps_microseconds(self) -> Timestamp;
    fn gps_nanoseconds(self) -> Timestamp;
}

// ──────────────────────────────────────────────────────────────
// Macro for traits that use existing Timestamp::from_* helpers
// ──────────────────────────────────────────────────────────────

macro_rules! impl_timestamp_trait {
    ($trait:ident,
     $sec:ident, $ms:ident, $us:ident, $ns:ident,
     $ctor_sec:ident, $ctor_ms:ident, $ctor_us:ident, $ctor_ns:ident) => {
        impl $trait for i8 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i16 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i32 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i64 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for i128 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self)
            }
        }
        impl $trait for u8 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u16 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u32 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u64 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
        impl $trait for u128 {
            #[inline(always)]
            fn $sec(self) -> Timestamp {
                Timestamp::$ctor_sec(self as i128)
            }
            #[inline(always)]
            fn $ms(self) -> Timestamp {
                Timestamp::$ctor_ms(self as i128)
            }
            #[inline(always)]
            fn $us(self) -> Timestamp {
                Timestamp::$ctor_us(self as i128)
            }
            #[inline(always)]
            fn $ns(self) -> Timestamp {
                Timestamp::$ctor_ns(self as i128)
            }
        }
    };
}

impl_timestamp_trait!(
    UnixTimestamp,
    unix_seconds,
    unix_milliseconds,
    unix_microseconds,
    unix_nanoseconds,
    from_unix_seconds,
    from_unix_milliseconds,
    from_unix_microseconds,
    from_unix_nanoseconds
);

impl_timestamp_trait!(
    TAITimestamp,
    tai_seconds,
    tai_milliseconds,
    tai_microseconds,
    tai_nanoseconds,
    from_tai_sec,
    from_tai_ms,
    from_tai_us,
    from_tai_ns
);

impl_timestamp_trait!(
    J2000Timestamp,
    j2000_seconds,
    j2000_milliseconds,
    j2000_microseconds,
    j2000_nanoseconds,
    from_tai_sec,
    from_tai_ms,
    from_tai_us,
    from_tai_ns
);

impl_timestamp_trait!(
    UTCTimestamp,
    utc_seconds,
    utc_milliseconds,
    utc_microseconds,
    utc_nanoseconds,
    from_utc_sec,
    from_utc_ms,
    from_utc_ns,
    from_utc_ns
);

// ──────────────────────────────────────────────────────────────
// GPS (uses its own clock_type constructor)
// ──────────────────────────────────────────────────────────────

macro_rules! impl_gps_timestamp {
    ($ty:ty) => {
        impl GPSTimestamp for $ty {
            #[inline(always)]
            fn gps_seconds(self) -> Timestamp {
                Timestamp::new(self as i128, 0, ClockType::GPST)
            }
            #[inline(always)]
            fn gps_milliseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_ms(self as i128))
            }
            #[inline(always)]
            fn gps_microseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_us(self as i128))
            }
            #[inline(always)]
            fn gps_nanoseconds(self) -> Timestamp {
                Timestamp::new(0, 0, ClockType::GPST).add(Delta::from_ns(self as i128))
            }
        }
    };
}

impl_gps_timestamp!(i8);
impl_gps_timestamp!(i16);
impl_gps_timestamp!(i32);
impl_gps_timestamp!(i64);
impl_gps_timestamp!(i128);
impl_gps_timestamp!(u8);
impl_gps_timestamp!(u16);
impl_gps_timestamp!(u32);
impl_gps_timestamp!(u64);
impl_gps_timestamp!(u128);
