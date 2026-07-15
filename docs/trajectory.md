# Proper time along trajectories

How to integrate a spacecraft (or ground) clock along tabulated samples using
the `physics` feature.

Methods live on [`Dt`](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html)
(`proper_time_from_states`, `proper_time_drift_from_states`,
`proper_time_differential_vs_rate`, and related).

Runnable sketch (sample table → proper time, vs ground, drift):
[examples/proper_time_path.rs](https://github.com/ragardner/deep-time/blob/main/examples/proper_time_path.rs)
(`cargo run --example proper_time_path --features physics`).

Theory of the rate model (master Lagrangian, weak-field limits):
[relativity.md](relativity.md).

## Concepts

| Term | Meaning |
|------|---------|
| **Coordinate time** \(t\) | Shared timeline in ephemerides and mission plans (the times in your samples). |
| **Proper time** \(\tau\) | What a real clock moving with the vehicle measures. Gravity and speed make \(\tau\) differ slightly from \(t\). |
| **Rate** \(r = d\tau/dt\) | How fast the clock ticks relative to coordinate time. |
| **Drift** \(\Delta\tau - \Delta t = \int(r-1)\,dt\) | How much the clock ran fast (positive) or slow (negative) vs coordinate time over an interval. |

Integration uses the trapezoidal rule on sample-to-sample rates. Between samples
the rate is treated as linear. Accuracy follows sample density and the
gravitational potential \(\Phi\) you supply.

## Which function should I call?

| Question | Method on `Dt` |
|----------|----------------|
| How much proper time over **all** samples I provided? | `proper_time_from_path` / `proper_time_from_states` |
| How much proper time on a **named arc** `[t₁, t₂]`? | `proper_time_from_path_between` / `proper_time_from_states_between` |
| How much did the clock gain/lose vs coordinate time on `[t₁, t₂]`? | `proper_time_drift_from_states` |
| Spacecraft vs ground (or any constant reference rate)? | `proper_time_differential_vs_rate` |
| Clock A vs clock B (two sample paths)? | `proper_time_differential_from_paths` |
| Rate is constant (ground station, circular cruise)? | `proper_time_between_constant_rate` |

Prefer `*_between` / drift / differential when you care about a **named**
interval. Full-span methods integrate whatever first/last samples you pass, with
no separate start/end check.

## Typical flight workflow

1. Build samples `(t, velocity, Φ)` with Φ in **m²/s²** (negative for bound
   gravity). Use your gravity model, or
   `Spacetime::grav_potential_from_point_masses` for a simple point-mass sum.
2. Use the same inertial-style frame for position (when building Φ) and velocity
   (e.g. Earth-centered inertial for near-Earth work).
3. Pass `characteristic_length_scale = 0.0` for ordinary solar-system / GNSS
   work (disables the optional curvature term).
4. Call a `*_between` / drift / differential method with your arc `[t₁, t₂]`.
   Samples must **cover** that interval or you get `DtErrKind::Incomplete`.

## Units and common mistakes

- Trajectory **`*_from_states`** APIs take Φ in **m²/s²** and divide by \(c^2\)
  internally. Do **not** pass Φ/c² there.
- `Spacetime::from_potential_velocity_and_scale` takes **Φ/c²** (dimensionless).
- Velocity is m/s; only speed enters the rate (via \(\beta = v/c\)).
- For almost all flight work, `characteristic_length_scale = 0.0`.

## Coverage rules (interval APIs)

For any method with `start` and `end`:

- `start ≤ end` (else `OutOfRange`)
- At least one sample at or before `start`
- Path must reach at least as far as `end`
- Times non-decreasing (else `NonMonotonic`)

Samples outside `[start, end]` are ignored except as bracketing points for rate
interpolation at the endpoints.

## What this is not

Not an orbit propagator or ephemeris reader. Not a gravity-field library either.

Each sample is basically: time, speed, and **Φ** (phi — gravitational potential).
The trajectory APIs only take those. There is no extra argument for \(J_2\),
Earth flattening, multipoles, and so on.

If your gravity model includes that detail, put it **into Φ** before you call
these methods. A simple point-mass Φ is fine when you do not need that detail.

## Related tests

- [trajectory_tests.rs](https://github.com/ragardner/deep-time/blob/main/tests/trajectory_tests.rs)
- [clock_drift_tests.rs](https://github.com/ragardner/deep-time/blob/main/tests/clock_drift_tests.rs)
