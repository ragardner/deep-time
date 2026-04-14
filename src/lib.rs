#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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
mod position;
mod time_point;
mod utils;

pub mod constants;
pub mod dt_big;
pub mod leap_seconds;
pub mod time_range;

pub(crate) use constants::*;
pub(crate) use dt_big::DtBig;
pub(crate) use utils::*;

pub use clock_drift::{ClockDrift, LocalSpacetime};
pub use clock_model::ClockModel;
pub use clock_type::ClockType;
pub use delta::Delta;
pub use delta::time_units::TimeUnits;
pub use light_time::{LightContext, ObserverState};
pub use position::{Position, Velocity};
pub use time_point::TimePoint;
pub use time_point::trajectory::RelativisticTrajectory;
pub use time_range::TimeRange;
pub use utils::{alpha_from_weak_field_potential, kretschmann_from_potential_and_scale};
