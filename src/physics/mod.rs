//! Relativistic physics core.
//!
//! ## Types
//!
//! - [`Position`] – Cartesian position (meters)
//! - [`Velocity`] – Cartesian velocity (m/s)
//! - [`Spacetime`] – local lapse α, speed β, and optional curvature
//! - [`Drift`] – quadratic clock polynomial; also builds instantaneous rates
//! - [`Observer`] – state for light-time and one-shot rate ratios
//!
//! ## Trajectory (proper time along samples)
//!
//! Integration methods live on [`Dt`](../struct.Dt.html): they walk tabulated
//! states or [`Spacetime`](../struct.Spacetime.html) snapshots and accumulate
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
//! **Typical use:** samples `(time, velocity, Φ)` with Φ in m²/s², pass
//! `characteristic_length_scale = 0.0`, and call a `*_between` / drift /
//! differential method. Samples must cover the requested interval.
//!
//! Longer guide (concepts, coverage rules, units):
//! [docs/trajectory.md](https://github.com/ragardner/deep-time/blob/main/docs/trajectory.md).
//!
//! Rate-model theory:
//! [docs/relativity.md](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).

mod trajectory;

pub mod drift;
pub mod observer;
pub mod position;
pub mod spacetime;
pub mod velocity;
