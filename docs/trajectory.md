# Proper time along trajectories

How to integrate a spacecraft (or ground) clock along tabulated samples using
the `physics` feature.

Methods live on [`Dt`](https://docs.rs/deep-time/latest/deep_time/struct.Dt.html)
(`proper_time_from_states`, `proper_time_drift_from_states`,
`proper_time_differential_vs_rate`, and related).

Runnable sketch (sample table → proper time, vs ground, drift):
[examples/proper_time_path.rs](https://github.com/ragardner/deep-time/blob/main/examples/proper_time_path.rs)
(`cargo run --example proper_time_path --features physics`).

Theory of the rate model (interval, IERS / Ashby weak-field fill):
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

## Rate model

The engine is \(d\tau/dt=\alpha\sqrt{1-\beta^2}\). Trajectory `*_from_states`
methods fill \(\alpha\) and \(\beta\) from Φ and \(v\):

\[
r = \frac{d\tau}{dt} = \sqrt{\Bigl(1 + \frac{2\Phi}{c^2}\Bigr)\Bigl(1 - \frac{v^2}{c^2}\Bigr)}
\approx 1 + \frac{\Phi}{c^2} - \frac{v^2}{2c^2}.
\]

That is IERS Conventions (2010) eqs. (10.6)–(10.7) and Ashby (2003) through
\(O(c^{-2})\). The library takes **Φ negative** for bound gravity (physics
convention). IERS writes the same physics with a **positive** \(U_E\)
(\(\Phi = -U_E\)). Passing \(+GM/r\) makes clocks appear to run fast.

If you already have lapse and Eulerian speed from a metric, build
[`Spacetime`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Spacetime.html)
with `new(alpha, beta)` and use the `*_from_path` methods. Same engine.

This integrator is the IERS **general** method (numerical integral of \(d\tau/dt\)
along the path). The GPS closed form \(\Delta t_r = -2\,\mathbf{r}\cdot\mathbf{v}/c^2\)
is a Keplerian special case after a factory frequency offset; IERS says not to
use it for LEO — put \(J_2\) (and anything else) **into Φ**, then integrate here.

The result is a **duration**, not an epoch. All sample times must share one
coordinate time scale (comparisons use attoseconds only; TAI mixed with TT will
silently integrate the wrong span). IERS (10.7) is written with \(t=\) TCG;
these methods treat the sample times as that kind of coordinate time. IERS
(10.8)–(10.9) add \(L_G\) for \(t=\) TT; that factor is a `Scale` conversion,
not part of this integral. Velocity should be inertial-style (e.g. GCRS / ECI).
ECEF speed includes Earth rotation and is the wrong \(\beta\) unless you mean
that.

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
3. Call a `*_between` / drift / differential method with your arc `[t₁, t₂]`.
   Samples must **cover** that interval or you get `DtErrKind::Incomplete`.

## Units and common mistakes

- Trajectory **`*_from_states`** APIs take Φ in **m²/s²** and divide by \(c^2\)
  internally. Do **not** pass Φ/c² there.
- `Spacetime::from_potential_and_velocity` takes **Φ/c²** (dimensionless).
- Velocity is m/s; only speed enters the rate (via \(\beta = v/c\)).

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
