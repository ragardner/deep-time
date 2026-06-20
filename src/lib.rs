#![cfg_attr(test, allow(clippy::all))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// ──────────────────────────────────────────────────────────────
// Optional panic handler (opt-in via feature)
// ──────────────────────────────────────────────────────────────
#[cfg(all(feature = "panic-handler", not(feature = "std"), not(test)))]
use core::panic::PanicInfo;

#[cfg(all(feature = "panic-handler", not(feature = "std"), not(test)))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Uses spin_loop() for better power characteristics than plain loop{}
    loop {
        core::hint::spin_loop();
    }
}

/// Alias for f64, maybe upgrade one day
pub type Real = f64;

/// Convert a number to the crates [`Real`] type (f64).
///
/// Equivalent to `n as f64`.
#[macro_export]
macro_rules! f {
    ($x:expr) => {
        $x as $crate::Real
    };
}

/// Safe Euclidean division.
/// Returns `default` if `rhs == 0` or if `lhs == i128::MIN && rhs == -1`.
macro_rules! safe_div_euc {
    ($lhs:expr, $rhs:expr, $default:expr) => {{
        match ($lhs).checked_div_euclid($rhs) {
            Some(q) => q,
            None => $default,
        }
    }};
}

/// Safe Euclidean remainder.
/// Returns `$default` if `rhs == 0` or if `lhs == Self::MIN && rhs == -1`.
macro_rules! safe_rem_euc {
    ($lhs:expr, $rhs:expr, $default:expr) => {{
        match ($lhs).checked_rem_euclid($rhs) {
            Some(r) => r,
            None => $default,
        }
    }};
}

/// Turns a list of string literals into an array of `&'static [u8]`.
macro_rules! byte_arrays {
    ( $( $s:literal ),+ $(,)? ) => {
        [ $( $s.as_bytes() ),+ ]
    };
}

// _________________________________________
// FEATURE MOD
// _________________________________________
#[cfg(feature = "parse")]
mod alloc_parse;

#[cfg(feature = "wire")]
mod wire;

// _________________________________________
// MOD
// _________________________________________
mod dt;
mod lite_str;
mod locale;
mod physics;
mod scale;
mod strtime;
mod time_range;
mod ymdhms;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod an_err;
pub mod civil_parts;
pub mod constants;
pub mod error;
pub mod historical_utc;
pub mod leap_seconds;
pub mod math;
pub mod tz;

// _________________________________________
// FEATURE PUB MOD
// _________________________________________
#[cfg(feature = "eop")]
pub mod eop;

#[cfg(feature = "sidereal")]
pub mod sidereal;

// _________________________________________
// FEATURE CRATE USE
// _________________________________________
#[cfg(feature = "parse")]
pub(crate) use alloc_parse::{
    alloc_constants::*, date::*, date_classification::*, duration::*, helpers::*, parse_date::*,
    types::*,
};
#[cfg(feature = "parse")]
pub(crate) use locale::{lang_data::*, lang_map::*};

// _________________________________________
// CRATE USE
// _________________________________________
pub(crate) use civil_parts::*;
pub(crate) use constants::*;
pub(crate) use locale::*;
#[allow(unused_imports)]
pub(crate) use math::{
    atan2::atan2,
    cos::cos,
    div::rem_euclid_f,
    floor::floor_f,
    log::log,
    sin::sin,
    sqrt::{hypot, sqrt},
};
pub(crate) use strtime::*;

// _________________________________________
// FEATURE PUB USE
// _________________________________________
#[cfg(feature = "parse")]
pub use alloc_parse::types::{Mode, Order, ParseCfg};

#[cfg(feature = "mars")]
pub use dt::mars;

#[cfg(feature = "sidereal")]
pub use sidereal::Sidereal;

// _________________________________________
// PUB USE
// _________________________________________
pub use an_err::AnErr;
pub use dt::Dt;
pub use dt::lunar;
pub use dt::numbers_traits::{AttosTraits, TimeTraits};
pub use error::{DtErr, DtErrKind};
pub use lite_str::LiteStr;
pub use locale::Lang;
pub use physics::drift::{Drift, Spacetime};
pub use physics::light_time::Observer;
pub use physics::position::{Position, Velocity};
pub use scale::Scale;
pub use strtime::StrPTimeFmt;
pub use time_range::{Every, TimeRange};
pub use ymdhms::YmdHms;
