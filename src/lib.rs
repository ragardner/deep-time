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

// possibly upgrade to f128 when it's stable
pub type Real = f64;
macro_rules! f {
    ($x:expr) => {
        $x as Real
    };
}

#[inline(always)]
pub const fn to_sec_f(attos: u128) -> Real {
    f!(attos) / ATTOS_PER_SECF
}

/// Converts attoseconds → seconds (s)
#[inline(always)]
pub fn to_sec(attos: i128) -> i128 {
    attos / ATTOS_PER_SEC_I128
}

/// Converts attoseconds → milliseconds (ms)
#[inline(always)]
pub fn to_ms(attos: i128) -> i128 {
    attos / ATTOS_PER_MS_I128
}

/// Converts attoseconds → microseconds (us)
#[inline(always)]
pub fn to_us(attos: i128) -> i128 {
    attos / ATTOS_PER_US_I128
}

/// Converts attoseconds → nanoseconds (ns)
#[inline(always)]
pub fn to_ns(attos: i128) -> i128 {
    attos / ATTOS_PER_NS_I128
}

/// Converts attoseconds → picoseconds (ps)
#[inline(always)]
pub fn to_ps(attos: i128) -> i128 {
    attos / ATTOS_PER_PS_I128
}

/// Converts attoseconds → femtoseconds (fs)
#[inline(always)]
pub fn to_fs(attos: i128) -> i128 {
    attos / ATTOS_PER_FS_I128
}

// _________________________________________
// FEATURE MOD
// _________________________________________
#[cfg(feature = "parse")]
mod alloc_parse;

#[cfg(feature = "ut1")]
mod ut1;

// _________________________________________
// MOD
// _________________________________________
mod an_err;
mod ascii_str;
mod clock_drift;
mod clock_model;
mod dt;
mod gregorian_time;
mod light_time;
mod parser;
mod position;
mod scale;
mod t_span;
mod time_parts;
mod time_range;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod constants;
pub mod error;
pub mod historical_sofa;
pub mod leap_seconds;
pub mod tzdb;
pub mod utils;

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
pub(crate) use utils::*;

// _________________________________________
// FEATURE PUB USE
// _________________________________________
#[cfg(feature = "parse")]
pub use alloc_parse::types::{DateOrder, DateParseMode, Lang, ParseCfg};

#[cfg(feature = "ut1")]
pub use ut1::{Separator, Ut1Columns, Ut1Data, Ut1Format, Ut1Row};

#[cfg(feature = "wire")]
pub use an_err::{WireErr, WireLocation};

// _________________________________________
// PUB USE
// _________________________________________
pub use an_err::AnErr;
pub use ascii_str::{AsciiStr, AsciiStrError};
pub use clock_drift::{ClockDrift, LocalSpacetime};
pub use clock_model::ClockModel;
pub use dt::Dt;
pub use error::{DtErr, DtErrKind};
pub use gregorian_time::{GregorianTime, YmdHms};
pub use light_time::{LightContext, ObserverState};
pub use position::{Position, Velocity};
pub use scale::Scale;
pub use t_span::TSpan;
pub use t_span::time_units::{AttosUnits, TimeUnits};
pub use time_parts::{Meridiem, Offset, TimeParts, Weekday};
pub use time_range::{Every, TimeRange};
