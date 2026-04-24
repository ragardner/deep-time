#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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
#[cfg(feature = "std")]
mod std_parse;

// _________________________________________
// MOD
// _________________________________________
mod ascii_str;
mod clock_drift;
mod clock_model;
mod clock_type;
mod common;
mod light_time;
mod parse;
mod position;
mod time_parts;
mod time_point;
mod time_range;
mod time_span;
mod utils;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod constants;
pub mod error;
pub mod error_std;
pub mod leap_seconds;
pub mod tzdb;

// _________________________________________
// FEATURE CRATE USE
// _________________________________________
#[cfg(feature = "std")]
pub(crate) use std_parse::{
    date::*, date_classification::*, duration::*, lang::*, lang_map::*, languages::en::*,
    parse_date::*, std_constants::*, types::*,
};

// _________________________________________
// CRATE USE
// _________________________________________
pub(crate) use constants::*;
pub(crate) use error::{DtErrKind, DtError};
pub(crate) use error_std::DtStdError;
pub(crate) use tzdb::TZ_ENTRIES;
pub(crate) use utils::*;

// _________________________________________
// FEATURE PUB USE
// _________________________________________
#[cfg(feature = "std")]
pub use std_parse::{
    lang::Lang,
    types::{DateOrder, DateParseMode, ParseCfg},
};

// _________________________________________
// PUB USE
// _________________________________________
pub use ascii_str::{AsciiStr, AsciiStrError};
pub use clock_drift::{ClockDrift, LocalSpacetime};
pub use clock_model::ClockModel;
pub use clock_type::ClockType;
pub use light_time::{LightContext, ObserverState};
pub use parse::{formatter, parser};
pub use position::{Position, Velocity};
pub use time_parts::{Meridiem, TimeParts, TimeZone, Weekday};
pub use time_point::TimePoint;
pub use time_point::gregorian_time::GregorianTime;
pub use time_range::{Every, TimeRange};
pub use time_span::TimeSpan;
pub use time_span::time_units::TimeUnits;
