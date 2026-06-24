//! Relativistic physics core.
//!
//! This is the primary location of the geometric primitives and relativistic
//! calculation machinery:
//!
//! - [`Position`] – Cartesian position (meters)
//! - [`Velocity`] – Cartesian velocity (m/s)
//! - [`Spacetime`] – local lapse factor, velocity, and curvature
//! - [`Drift`] – quadratic clock drift model
//! - [`Observer`] – observer state with light-time methods
//! - [`trajectory`] – numerical integration of proper time
//!
//! Together these implement velocity time dilation, gravitational time
//! dilation, gravitational light-propagation delay, and accumulated proper
//! time for high-precision astronomical timescales.
//!
//! Information on the underlying physical model (the master Lagrangian,
//! different regimes of behavior, and its relationship to general relativity)
//! can be found [here](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

mod trajectory;

pub mod drift;
pub mod observer;
pub mod position;
pub mod spacetime;
pub mod velocity;
