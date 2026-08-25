# Relativistic timing

The `physics` feature compares a real clock to a coordinate time. A clock
carried on a spacecraft, a GNSS satellite, or a ground station does not tick
exactly with the timeline used in ephemerides and mission plans. Gravity and
spatial velocity change the interval that the clock measures. This note is
the theory of that comparison as this crate implements it.

Import the types with `use deep_time::physics::{Drift, Position, Spacetime, Velocity}`.

## Types

[`Spacetime`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Spacetime.html)
holds lapse α and spatial-velocity fraction β for one clock, written in a
coordinate system you already chose. It does not store a second clock, a
position, or a time-scale tag. The tick rate is compared to that system’s
time \(t\): the same \(t\) used when measuring spatial velocity.

[`Drift`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Drift.html)
is a quadratic polynomial for the accumulated difference between proper time
and a chosen coordinate time such as TT or TAI. Fill it from a `Spacetime`
when the linear term should be the general-relativity interval, or from
measured coefficients when the polynomial is steering, aging, or a broadcast
clock correction.

[`Position`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Position.html)
and
[`Velocity`](https://docs.rs/deep-time/latest/deep_time/physics/struct.Velocity.html)
are Cartesian vectors in metres and metres per second. They are inputs for
forming Φ and β. The caller chooses the frame; these types do not tag it.

Time-scale conversions, including the conventional rate \(L_G\) between TCG
and TT, live on
[`Scale`](https://docs.rs/deep-time/latest/deep_time/enum.Scale.html).
They are not part of the interval.

## Proper time and coordinate time

**Coordinate time** \(t\) is the shared timeline in which positions, velocities,
and gravitational potential are written. Ephemerides, GNSS, and mission plans
use a coordinate time. IERS Conventions write that \(t\) as TCG in GCRS for
the weak-field clock equation. This crate takes \(t\) as whichever coordinate
time Φ and \(v\) (or α and β) were computed with.

**Proper time** \(\tau\) is what a real clock measures as it moves through
spacetime. Gravity and spatial velocity make \(\tau\) differ slightly from
\(t\). The comparison is a **rate** \(d\tau/dt\), not a pair of clock
readings and not a second clock stored on `Spacetime`.

When the rate is `1`, the clock ticks in step with \(t\). Below `1` it ticks
slower than that \(t\). \(t\) is not an argument to the rate. It is implied
by how α and β were built.

## The interval

A clock measures the metric interval of general relativity. In 3+1 form, with
lapse α and physical speed β relative to the time slices,

\[
\frac{d\tau}{dt} = \alpha \sqrt{1-\beta^2} = \sqrt{\max\bigl(0,\,\alpha^2(1-\beta^2)\bigr)}.
\]

That is the single rate used by `Spacetime`, and by `Drift` when `Drift` is
built from a `Spacetime`. Every constructor on `Spacetime` ends in this
interval. The crate evaluates the square-root form, not a linearized
right-hand side.

**α** is the lapse: the gravitational redshift factor of general relativity.
With no shift, \(\alpha=\sqrt{-g_{00}}\). It is the number of seconds a clock
with no spatial velocity (\(\beta = 0\)) ticks during one second of coordinate
time \(t\). From gravitational potential, \(\alpha=\sqrt{1+2\Phi/c^2}\). Φ is
the potential of the field, not a location and not a `Position`. Φ is
**negative** for bound gravity. Whether α is less than 1 depends on how \(t\)
is scaled: if Φ → 0 at infinity, a bound well has α < 1. On one shared \(t\),
a more negative Φ gives a smaller α.

**β** is spatial velocity in that same coordinate system, as a fraction of
light speed. Spatial velocity \(v\) is the `Velocity` vector: metres of travel
through space per one second of that \(t\). \(\beta = |v|/c\). When α and β
come from a metric, β is the Eulerian speed from the spatial metric, not a raw
coordinate speed.

When α is 1 and β is 0, there is no spatial velocity and the lapse is 1, so
the clock ticks in step with \(t\).

To compare two clocks, give each its own α and β and subtract the rates. There
is no second clock on `Spacetime`.

## Filling Spacetime

`Spacetime` stores α and β. How you obtain those two numbers depends on what
you already have. The primitive constructor stores them directly and is valid
for any α and β, weak field or strong. The other constructors compute α and β
from potential and spatial velocity, then store them the same way.

### Potential and spatial velocity

Earth, GNSS, and solar-system work usually have gravitational potential and a
spatial-velocity vector rather than a metric lapse. Pass Φ in SI units
**m²/s²**. Φ is **negative** for bound gravity. A point-mass well is
\(-GM/r\). Spatial velocity is metres per second in the same frame as Φ; only
the speed \(|v|\) enters the rate, via \(\beta = |v|/c\).

The fill is

\[
\alpha = \sqrt{1 + \frac{2\Phi}{c^2}}, \qquad \beta = \frac{|\mathbf{v}|}{c}.
\]

The rate is then

\[
\frac{d\tau}{dt} = \sqrt{\Bigl(1 + \frac{2\Phi}{c^2}\Bigr)\Bigl(1 - \frac{v^2}{c^2}\Bigr)}
\approx 1 + \frac{\Phi}{c^2} - \frac{v^2}{2c^2}.
\]

That expansion is IERS Conventions (2010) eqs. (10.6)–(10.7) and Ashby (2003)
through \(O(c^{-2})\). The library evaluates the square-root interval, not the
linearized form.

IERS and geodesy write a **positive** potential \(U_E\), with \(\Phi = -U_E\).
If that is what you have, use the positive-potential constructor, which negates
\(U\) and then uses the same fill. Put tidal terms and multipoles into \(U\)
(or into Φ) before you call it. This crate does not add \(J_2\), tides, or
the IERS tidal combination
\(V(\mathbf{X}_A)-V(\mathbf{X}_E)-x_A^i\partial_i V(\mathbf{X}_E)\).

If the potential is already dimensionless Φ/c², there is a constructor that
skips the division by \(c^2\). Prefer SI Φ in m²/s² when that is what your
gravity model produces.

IERS (10.7) takes coordinate time \(t\) as GCRS time (TCG). Φ and \(v\) must
be built in that kind of coordinate time if you want that IERS comparison.
IERS eqs. (10.8)–(10.9) are the same expansion with \(t\) taken as TT and an
extra conventional rate \(L_G\). This crate does not add \(L_G\) inside the
interval. Convert TT, TCG, TCB, and TDB with `Scale`.

The GPS closed form \(\Delta t_r = -2\,\mathbf{r}\cdot\mathbf{v}/c^2\) is a
Keplerian special case after a factory frequency offset. IERS says not to use
it for LEO. This crate does not implement it. Put the physics you need into Φ
and \(v\), then use the interval.

A small point-mass helper on `Spacetime` sums \(\Phi = -\sum GM_i/r_i\) at a
`Position`. That is enough for a rough multi-body well. LEO-grade timing
usually needs a full gravity model.

### Lapse and speed from a metric

If you already have α and β, store them on `Spacetime` directly. That includes
a Schwarzschild, Kerr, or numerical-relativity snapshot. Newtonian Φ is not
used, and Newtonian Φ does not describe a horizon. Do not put Φ in the α slot:
Φ is a potential, not a lapse.

When those numbers come from a metric, β is the Eulerian speed as a fraction
of light speed, taken from the spatial metric, not from a raw coordinate
speed.

If you have a metric lapse and a spatial-velocity vector in metres per second,
there is a constructor that sets \(\beta = |v|/c\) with Euclidean speed. That
is the usual solar-system choice. If the spatial metric makes Eulerian speed
differ from Euclidean \(|v|/c\), as in a compact-object snapshot, compute β
yourself and store α and β directly.

## Clock polynomials

`Drift` is the quadratic

\[
\mathrm{offset} = a_0 + a_1 s + a_2 s^2,
\]

where \(s\) is elapsed coordinate time. The three coefficients are a fixed
offset, a constant fractional rate (seconds per second), and a quadratic term
(seconds per second squared: aging, or a rate that itself changes). GNSS
broadcast clock corrections and spacecraft steering use this polynomial. All
three coefficients are stored as `Dt`.

Building `Drift` from a `Spacetime` copies the general-relativity tick-rate
offset \(d\tau/dt - 1\) into the linear coefficient. The constant and
quadratic terms are zero. A snapshot is not a quadratic. If you need aging or
a changing rate, set those coefficients yourself.

Evaluating the polynomial after a span of coordinate time returns
\(a_0 + a_1 s + a_2 s^2\) as a `Dt`. When the polynomial was built from a
`Spacetime`, that value is \(\Delta\tau - \Delta t\). Otherwise it is whatever
offset, rate, and aging you stored.

When α and β do not change, proper time over a span is coordinate time plus
that offset:

```rust
let span = end.to_diff_raw(start);
let dtau = span.add(Drift::from_spacetime(&spacetime).time_diff_after(&span));
```

`Dt` can also step an epoch by that proper time, or add a `Drift` polynomial
relative to a reference instant. A path with changing α or β is the caller’s
integral of the rate. Keep \(\Delta t\) in attoseconds and send only the
relativistic piece \((r-1)\Delta t\) through floating point, as the span-plus-
offset recipe above does.

A `Drift` with a nonzero constant term adds that constant on every evaluation.
A stepping loop should use a polynomial whose constant is zero, or the
`Spacetime` snapshot path, which fills a zero constant.

## Numerical evaluation of the offset

The offset \(d\tau/dt - 1\) is evaluated as
\((\delta-1)/(\sqrt{\delta}+1)\) with \(\delta=\max(\alpha^2(1-\beta^2),0)\).
That equals \(\sqrt{\delta}-1\) without evaluating \(\sqrt{1+\varepsilon}-1\)
in floating point, which is inaccurate when the rate is close to unity.

If \(\alpha^2(1-\beta^2)\) is negative (for example a superluminal β), δ is
clamped to zero and the rate is zero.

## In this repository

The implementation is `src/physics/`. Tests for the interval and the
polynomial are `tests/spacetime_rate_tests.rs`, `tests/interval_rate_tests.rs`,
and `tests/clock_drift_tests.rs`.

## References

- Petit, G. and Luzum, B. (eds.), *IERS Conventions (2010)*, IERS Technical
  Note 36, §10.2, eqs. (10.6)–(10.7); see also (10.8)–(10.9) for the same
  expansion with \(t\) as TT.
- Ashby, N., “Relativity in the Global Positioning System,” *Living Reviews
  in Relativity* **6**, 1 (2003).
- Soffel, M. et al., “The IAU 2000 resolutions for astrometry, celestial
  mechanics and metrology in the relativistic framework,” *Astron. J.*
  **126**, 2687 (2003).
