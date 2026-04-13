// #![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod clock_drift;
mod clock_model;
mod clock_type;
mod common;
mod delta;
mod position;
mod timestamp;
mod utils;

pub mod constants;
pub mod dt_big;
pub mod leap_seconds;
pub mod time_range;

pub(crate) use constants::*;
pub(crate) use dt_big::DtBig;
pub(crate) use utils::*;

pub use clock_drift::{ClockDrift, ResolvedMetric};
pub use clock_model::ClockModel;
pub use clock_type::ClockType;
pub use delta::Delta;
pub use delta::time_units::TimeUnits;
pub use position::{Position, RelativisticState, Velocity};
pub use time_range::TimeRange;
pub use timestamp::Timestamp;
pub use timestamp::traits::{
    GPSTimestamp, J2000Timestamp, RelativisticTrajectory, TAITimestamp, UTCTimestamp, UnixTimestamp,
};
pub use utils::{alpha_from_weak_field_potential, kretschmann_from_potential_and_scale};
