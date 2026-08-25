//! Relativistic physics core.
//!
//! ## Types
//!
//! - [`Position`] – Cartesian position (meters)
//! - [`Velocity`] – Cartesian velocity (m/s)
//! - [`Spacetime`] – local lapse α and speed β
//! - [`Drift`] – quadratic clock polynomial; also builds instantaneous rates
//!
//! Import: `use deep_time::physics::{Drift, Position, Spacetime, Velocity}`.
//!
//! ## Trajectory (proper time along samples)
//!
//! Integration methods live on [`Dt`](../struct.Dt.html): they walk tabulated
//! states or [`Spacetime`](struct.Spacetime.html) snapshots and accumulate
//! proper time with a trapezoidal rule.
//!
//! | Question | Method |
//! |----------|--------|
//! | Δτ over all samples | `proper_time_from_path` / `proper_time_from_states` |
//! | Δτ on a named arc `[t₁, t₂]` | `proper_time_from_path_between` / `proper_time_from_states_between` |
//! | Drift Δτ − Δt on `[t₁, t₂]` | `proper_time_drift_from_states` |
//! | Path vs constant ground/reference rate | `proper_time_differential_vs_rate` |
//! | Path A vs path B | `proper_time_differential_from_paths` |
//! | Constant rate only | `proper_time_between_constant_rate` |
//!
//! **Typical use:** samples `(time, velocity, Φ)` with Φ in m²/s² (negative for
//! bound gravity), and call a `*_between` / drift / differential method.
//! Samples must cover the requested interval and share one time scale. The rate
//! is \(\sqrt{(1+2\Phi/c^2)(1-v^2/c^2)}\); that square root’s \(O(c^{-2})\)
//! expansion is IERS 2010 eqs. 10.6–10.7 / Ashby 2003.
//!
//! Longer guide (concepts, coverage rules, units):
//! [docs/trajectory.md](https://github.com/ragardner/deep-time/blob/main/docs/trajectory.md).
//!
//! Runnable example:
//! [examples/proper_time_path.rs](https://github.com/ragardner/deep-time/blob/main/examples/proper_time_path.rs).
//!
//! Rate-model theory:
//! [docs/relativity.md](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

mod trajectory;

pub mod drift;
pub mod position;
pub mod spacetime;
pub mod velocity;

pub use drift::Drift;
pub use position::Position;
pub use spacetime::Spacetime;
pub use velocity::Velocity;
