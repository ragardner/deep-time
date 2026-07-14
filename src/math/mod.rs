//! Tested, `const fn` versions of libm float math functions. e.g. `use deep_time::math::sin;`
//!
//! Re-exports of `sin`, `cos`, `tan`, `atan`, `atan2`, `floor_f`, `rem_euclid_f`, `sqrt`, `log`.

mod atan;
mod atan2;
mod cos;
mod div;
mod floor;
mod k_cos;
mod k_sin;
mod k_tan;
mod log;
mod powi;
mod rem_pio2;
mod rem_pio2_large;
mod round;
mod scalbn;
mod sin;
mod sqrt;
mod tan;
mod trunc;

pub use atan::atan;
pub use atan2::atan2;
pub use cos::cos;
pub use div::rem_euclid_f;
pub use floor::floor_f;
pub use log::log;
pub use powi::powi;
pub use round::round;
pub use sin::sin;
pub use sqrt::{hypot, sqrt};
pub use tan::tan;
pub use trunc::trunc;

use k_cos::*;
use k_sin::*;
use k_tan::*;
use rem_pio2::*;
use rem_pio2_large::*;
use scalbn::*;
