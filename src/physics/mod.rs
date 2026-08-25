//! Proper-time rates and quadratic clock polynomials.
//!
//! ## Types
//!
//! - [`Position`] – Cartesian position (meters)
//! - [`Velocity`] – Cartesian velocity (m/s)
//! - [`Spacetime`] – lapse α and spatial-velocity fraction β for one clock
//! - [`Drift`] – quadratic clock polynomial
//!
//! Import: `use deep_time::physics::{Drift, Position, Spacetime, Velocity}`.
//!
//! The instantaneous rate is [`Spacetime::proper_time_rate_offset`]. The
//! offset over a span is [`Drift::time_diff_after`]. Apply a polynomial to an
//! epoch with [`Dt::convert_using_drift`](crate::Dt::convert_using_drift) or
//! [`Dt::adjusted_advance`](crate::Dt::adjusted_advance). Theory:
//! [docs/relativity.md](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).
//!
//! ## Filling [`Spacetime`]
//!
//! Every constructor uses the same interval. [`Spacetime::new`] stores α and
//! β you already have (from a metric, or computed yourself). The others fill
//! those two numbers from potential and velocity. Then
//! [`Drift::from_spacetime`] if you need the quadratic.
//!
//! | You have | Constructor |
//! |----------|-------------|
//! | α and β already in hand | [`Spacetime::new`] |
//! | Lapse α and spatial velocity (m/s) | [`Spacetime::from_lapse_and_velocity`] |
//! | Φ in m²/s² (negative for bound gravity) and velocity | [`Spacetime::from_potential_and_velocity`] |
//! | IERS / geodesy positive \(U\) (m²/s²) and velocity | [`Spacetime::from_positive_potential_and_velocity`] |
//! | Dimensionless Φ/c² and velocity | [`Spacetime::from_potential_over_c2_and_velocity`] |

pub mod drift;
pub mod position;
pub mod spacetime;
pub mod velocity;

pub use drift::Drift;
pub use position::Position;
pub use spacetime::Spacetime;
pub use velocity::Velocity;
