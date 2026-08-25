# Relativistic Timing Model

Proper-time rates under the `physics` feature: instantaneous \(d\tau/dt\),
clock
[`Drift`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Drift.html)
polynomials, and integration along trajectories.

The rate engine is the 3+1 interval of general relativity. Potential and
spatial velocity fill the lapse and speed of that interval the way IERS /
Ashby do in the weak field.

## Relation to the library

Implemented in `src/physics/` and exposed as:

- `Spacetime` — lapse \(\alpha\) and speed \(\beta\); builds rates from a
  metric snapshot or from potential and velocity.
- `Drift` — quadratic polynomial that accumulates the difference between proper
  time and coordinate time.
- `Position` and `Velocity` — Cartesian vectors (meters and m/s).

Import physics types via `deep_time::physics`.

- [Proper time along trajectories](trajectory.md) — which `Dt` methods to call,
  units, coverage rules
- [Physics module](https://github.com/ragardner/deep-time/tree/main/src/physics)
- [Drift tests](https://github.com/ragardner/deep-time/blob/main/tests/clock_drift_tests.rs)
- [Trajectory tests](https://github.com/ragardner/deep-time/blob/main/tests/trajectory_tests.rs)
- [Spacetime / Drift rate tests](https://github.com/ragardner/deep-time/blob/main/tests/spacetime_rate_tests.rs)

## The interval

A clock measures the metric interval. In 3+1 form, with lapse \(\alpha\) and
physical speed \(\beta\) relative to the time slices,

\[
\frac{d\tau}{dt} = \alpha \sqrt{1-\beta^2} = \sqrt{\max\bigl(0,\,\alpha^2(1-\beta^2)\bigr)}.
\]

That is the single rate used by `Spacetime` and the trajectory integrators,
and by `Drift` when it is built from a `Spacetime`. \(\beta\) is the Eulerian
speed (spatial metric, not a raw coordinate speed) when \(\alpha\) and
\(\beta\) come from a full metric.

Two ways to supply \(\alpha\) and \(\beta\):

### Industry (GNSS, solar system, spacecraft)

[`Spacetime::from_potential_and_velocity`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Spacetime.html)
and the trajectory `*_from_states` methods fill

\[
\alpha = \sqrt{1 + \frac{2\Phi}{c^2}}, \qquad \beta = \frac{|\mathbf{v}|}{c}
\]

with **Φ negative** for bound gravity. The rate is then

\[
\frac{d\tau}{dt} = \sqrt{\Bigl(1 + \frac{2\Phi}{c^2}\Bigr)\Bigl(1 - \frac{v^2}{c^2}\Bigr)}
\approx 1 + \frac{\Phi}{c^2} - \frac{v^2}{2c^2}.
\]

That expansion is IERS Conventions (2010) eqs. (10.6)–(10.7) and Ashby (2003)
through \(O(c^{-2})\). IERS writes a **positive** \(U_E\) (\(\Phi = -U_E\)).
The library integrates the square-root interval, not the linearized form.

IERS (10.7) takes coordinate time \(t\) as GCRS time (TCG). Trajectory sample
times are treated as that kind of coordinate time: one shared scale, not a
TT-vs-TCG conversion inside the integral. IERS (10.8)–(10.9) are the same
expansion with \(t\) taken as TT and an extra conventional rate \(L_G\); this
crate does not add \(L_G\) inside the interval. TT / TCG / TCB / TDB remain
[`Scale`](https://docs.rs/deep-time/latest/deep_time/enum.Scale.html)
conversions.

IERS (10.6) also has tidal terms \(V(\mathbf{X}_A)-V(\mathbf{X}_E)-x_A^i\partial_i V(\mathbf{X}_E)\).
Fold those into Φ if you need them. Put \(J_2\) and other multipoles into Φ as
well. The GPS closed form \(\Delta t_r = -2\,\mathbf{r}\cdot\mathbf{v}/c^2\) is
a Keplerian special case after a factory frequency offset; IERS says not to use
it for LEO.

### Exact lapse and speed (including strong field)

[`Spacetime::new(alpha, beta)`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Spacetime.html)
takes the lapse and Eulerian speed from **your** metric. Same interval. Φ is
not used. A numerical-relativity or Schwarzschild/Kerr snapshot belongs here;
Newtonian Φ does not describe a horizon.

## Implementation

The offset \(d\tau/dt-1\) is evaluated as
\((\delta-1)/(\sqrt{\delta}+1)\) with \(\delta=\max(\alpha^2(1-\beta^2),0)\),
so a near-unity rate does not go through \(\sqrt{1+\varepsilon}-1\). Trajectory
methods integrate that rate with a trapezoidal rule; see
[trajectory.md](trajectory.md).
