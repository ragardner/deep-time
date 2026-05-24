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
mod an_err;
mod drift;
mod dt;
mod light_time;
mod lite_str;
mod parser;
mod position;
mod scale;
mod time_parts;
mod time_range;
mod ymdhms;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod constants;
pub mod error;
pub mod historical_sofa;
pub mod leap_seconds;
pub mod math;
pub mod tzdb;

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
    alloc_constants::*, date::*, date_classification::*, duration::*, lang_data::*, lang_map::*,
    languages::en::*, parse_date::*, types::*,
};

// _________________________________________
// CRATE USE
// _________________________________________
pub(crate) use constants::*;
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

// _________________________________________
// FEATURE PUB USE
// _________________________________________
#[cfg(feature = "parse")]
pub use alloc_parse::types::{Lang, Mode, Order, ParseCfg};

#[cfg(feature = "wire")]
pub use an_err::{WireErr, WireLocation};

#[cfg(feature = "sidereal")]
pub use sidereal::Sidereal;

#[cfg(feature = "mars")]
pub use dt::mars;

// _________________________________________
// PUB USE
// _________________________________________
pub use an_err::AnErr;
pub use drift::{Drift, Spacetime};
pub use dt::numbers_traits::{AttosTraits, TimeTraits};
pub use dt::{Dt, lunar};
pub use error::{DtErr, DtErrKind};
pub use light_time::ObserverState;
pub use lite_str::{LiteStr, LiteStrErr};
pub use parser::StrPTimeFmt;
pub use position::{Position, Velocity};
pub use scale::Scale;
pub use time_parts::{Meridiem, Offset, TimeParts, Weekday};
pub use time_range::{Every, TimeRange};
pub use ymdhms::{YmdHms, YmdHmsRich};
