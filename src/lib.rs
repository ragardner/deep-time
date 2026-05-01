#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// ──────────────────────────────────────────────────────────────
// Optional panic handler (opt-in via feature)
// ──────────────────────────────────────────────────────────────
#[cfg(all(feature = "panic-handler", not(feature = "alloc")))]
use core::panic::PanicInfo;

#[cfg(all(feature = "panic-handler", not(feature = "alloc")))]
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
mod clock_type;
mod common;
mod gregorian_time;
mod light_time;
mod parser;
mod position;
mod time_parts;
mod time_point;
mod time_range;
mod time_span;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod constants;
pub mod error;
pub mod historical_sofa_offsets;
pub mod leap_seconds;
pub mod tzdb;
pub mod utils;

// _________________________________________
// FEATURE CRATE USE
// _________________________________________
#[cfg(feature = "parse")]
pub(crate) use alloc_parse::{
    alloc_constants::*, date::*, date_classification::*, duration::*, lang::*, lang_map::*,
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
pub use alloc_parse::{
    lang::Lang,
    types::{DateOrder, DateParseMode, ParseCfg},
};

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
pub use clock_type::ClockType;
pub use error::{DtErr, DtErrKind};
pub use gregorian_time::GregorianTime;
pub use light_time::{LightContext, ObserverState};
pub use position::{Position, Velocity};
pub use time_parts::{Meridiem, Offset, TimeParts, Weekday};
pub use time_point::TimePoint;
pub use time_range::{Every, TimeRange};
pub use time_span::TimeSpan;
pub use time_span::time_units::TimeUnits;
