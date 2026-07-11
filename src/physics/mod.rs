//! Relativistic physics core.
//!
//! Geometric primitives and relativistic calculation machinery:
//!
//! - [`Position`] – Cartesian position (meters)
//! - [`Velocity`] – Cartesian velocity (m/s)
//! - [`Spacetime`] – local lapse factor, velocity, and curvature
//! - [`Drift`] – quadratic clock drift model
//! - [`Observer`] – observer state with light-time methods
//! - trajectory methods on [`Dt`] – integrate proper time along samples,
//!   including named intervals and path-to-path differentials
//!
//! These cover velocity and gravitational time dilation, light-time delays,
//! and accumulated proper time along tabulated trajectories. Accuracy of
//! clock integrals depends on the supplied potential and sample density.
//!
//! Background on the underlying model:
//! [docs/relativity.md](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

mod trajectory;

pub mod drift;
pub mod observer;
pub mod position;
pub mod spacetime;
pub mod velocity;
