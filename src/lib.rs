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
mod clock_drift;
mod clock_model;
mod clock_type;
mod common;
mod delta;
mod light_time;
mod parse;
mod position;
mod time_point;
mod time_range;
mod utils;

// _________________________________________
// PUB MOD
// _________________________________________
pub mod constants;
pub mod date_components;
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
    parse_date::{parse_date, parse_date_ms, parse_date_unix_ms},
    parse_duration::{duration_to_iso, parse_duration},
    types::{DateOrder, DateParseMode, ParseCfg},
};

// _________________________________________
// PUB USE
// _________________________________________
pub use clock_drift::{ClockDrift, LocalSpacetime};
pub use clock_model::ClockModel;
pub use clock_type::ClockType;
pub use date_components::{DateComponents, Meridiem, TimeZone, Weekday};
pub use delta::Delta;
pub use delta::time_units::TimeUnits;
pub use light_time::{LightContext, ObserverState};
pub use parse::parser::strptime;
pub use parse::{ccsds_bin, ccsds_str, formatter, parser};
pub use position::{Position, Velocity};
pub use time_point::TimePoint;
pub use time_range::{Every, TimeRange};
