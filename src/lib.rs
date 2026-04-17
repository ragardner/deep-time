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

mod clock_drift;
mod clock_model;
mod clock_type;
mod common;
mod delta;
mod light_time;
mod parse_and_format;
mod position;
mod time_point;
mod time_range;
mod utils;

pub mod constants;
pub mod leap_seconds;

pub(crate) use constants::*;
pub(crate) use utils::*;

pub use clock_drift::{ClockDrift, LocalSpacetime};
pub use clock_model::ClockModel;
pub use clock_type::ClockType;
pub use delta::Delta;
pub use delta::time_units::TimeUnits;
pub use light_time::{LightContext, ObserverState};
#[cfg(feature = "chrono")]
pub use parse_and_format::to_chrono;
#[cfg(feature = "jiff")]
pub use parse_and_format::to_jiff;
pub use parse_and_format::{ccsds_bin, ccsds_str, formatter, parser};
pub use position::{Position, Velocity};
pub use time_point::TimePoint;
pub use time_range::{Every, TimeRange};
pub use utils::{alpha_from_weak_field_potential, kretschmann_from_potential_and_scale};
