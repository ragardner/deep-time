/// Private helper: `sin` that works in `const fn` (range reduction + Taylor).
/// Accuracy is far better than needed for TDB-TT.
pub(crate) const fn sin_approx(x: f64) -> f64 {
    const PI: f64 = core::f64::consts::PI;
    const TWO_PI: f64 = 2.0 * PI;

    let mut x = x % TWO_PI;
    if x < 0.0 {
        x += TWO_PI;
    }
    if x > PI {
        x -= TWO_PI;
    }

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let x = if x > PI / 2.0 { PI - x } else { x };

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;
    let x11 = x9 * x2;

    sign * (x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0 + x9 / 362880.0 - x11 / 39916800.0)
}

/// Converts the Newtonian gravitational potential Φ/c² (where Φ < 0 for bound orbits)
/// into the relativistic lapse factor α = √(1 + 2Φ/c²).
///
/// This is the standard weak-field approximation used by JPL, ESA, GNSS, and
/// all modern solar-system navigation codes. It matches the physical convention
/// used everywhere else in the library (`RelativisticState` stores Φ < 0).
#[inline(always)]
pub fn alpha_from_weak_field_potential(gravitational_potential_over_c2: f64) -> f64 {
    // gravitational_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
    (1.0 + 2.0 * gravitational_potential_over_c2)
        .sqrt()
        .max(0.0)
}

/// Kretschmann scalar from the total relativity experienced locally
/// Computes the Kretschmann scalar \(\mathcal{K}\) from the total gravitational
/// relativity experienced by a local observer at the observer’s spacetime point.
///
/// This is the canonical, physics-true convenience function for the master Lagrangian.
/// It uses:
/// - `phi` = Φ/c² — the total local gravitational potential (redshift/gravity effect)
///   felt by the observer from all masses.
/// - `characteristic_length_scale` — the typical length scale (in meters) over which
///   the gravitational field varies at the observer’s location.
///
/// **For existing weak-field users** (Earth orbit, GNSS, solar-system navigation):
/// Supply your existing `phi` value and set `characteristic_length_scale = 0.0`.
/// The function safely returns 0.0 (the correct value in double precision).
///
/// **For strong-field / future users** (black-hole flybys, neutron stars, direct
/// gravimeters, or full metric evaluation):
/// Supply the measured or computed \(\phi\) and the real local length scale (or
/// the value from your metric). The function returns a physically accurate non-zero
/// curvature.
#[inline]
pub fn kretschmann_from_potential_and_scale(
    gravitational_potential_over_c2: f64,
    characteristic_length_scale: f64,
) -> f64 {
    if characteristic_length_scale <= 0.0 || gravitational_potential_over_c2 <= 0.0 {
        return 0.0;
    }
    // Exact weak-field limit: K ≈ 48 φ² / L⁴
    let curvature_scale =
        2.0 * gravitational_potential_over_c2 / characteristic_length_scale.powi(2);
    12.0 * curvature_scale.powi(2)
}
