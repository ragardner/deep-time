//! Relativistic physics core.
//!
//! This is the primary location of the geometric primitives and relativistic
//! calculation machinery:
//!
//! - [`position`] – [`Position`] and [`Velocity`] (Cartesian vectors in meters
//!   and m/s).
//! - [`drift`] – [`Drift`] (quadratic clock model) and [`Spacetime`] (local
//!   lapse, velocity, and curvature used for proper-time rates).
//! - [`light_time`] – [`Observer`] together with Shapiro delay, one-way and
//!   round-trip light-time solvers, and differential clock-rate corrections.
//! - [`trajectory`] – Numerical integration of proper time along sampled
//!   trajectories.
//!
//! Together these implement the velocity time dilation, gravitational time
//! dilation, gravitational light-propagation delay, and accumulated proper
//! time needed for high-precision astronomical timescales and deep-space
//! timing.
//!
//! Information on the underlying physical model (the master Lagrangian,
//! different regimes of behavior, and its relationship to general relativity)
//! can be found [here](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

mod trajectory;

pub mod drift;
pub mod light_time;
pub mod position;
