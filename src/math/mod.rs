//! Tested, `const fn` versions of rust libm floating-point math functions.
//!
//! Re-exports of `sin`, `cos`, `tan`, `atan`, `atan2`, `floor_f`, `rem_euclid_f`, `sqrt`, `log`.

pub mod atan;
pub mod atan2;
pub mod cos;
pub mod div;
pub mod floor;
pub mod k_cos;
pub mod k_sin;
pub mod k_tan;
pub mod log;
pub mod rem_pio2;
pub mod rem_pio2_large;
pub mod scalbn;
pub mod sin;
pub mod sqrt;
pub mod tan;

pub use atan::atan;
pub use atan2::atan2;
pub use cos::cos;
pub use div::rem_euclid_f;
pub use floor::floor_f;
pub use log::log;
pub use sin::sin;
pub use sqrt::sqrt;
pub use tan::tan;

use k_cos::*;
use k_sin::*;
use k_tan::*;
use rem_pio2::*;
use rem_pio2_large::*;
use scalbn::*;
