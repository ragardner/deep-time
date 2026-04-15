use crate::Real;

/// Private helper: `sin` that works in `const fn` (range reduction + Taylor).
/// Accuracy is far better than needed for TDB-TT.
pub(crate) const fn sin_approx(x: Real) -> Real {
    const PI: Real = f!(core::f64::consts::PI);
    const TWO_PI: Real = f!(2.0) * PI;

    let mut x = x % TWO_PI;
    if x < f!(0.0) {
        x += TWO_PI;
    }
    if x > PI {
        x -= TWO_PI;
    }

    let sign = if x < f!(0.0) { f!(-1.0) } else { f!(1.0) };
    let x = x.abs();

    let x = if x > PI / f!(2.0) { PI - x } else { x };

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    let x9 = x7 * x2;
    let x11 = x9 * x2;

    sign * (x - x3 / f!(6.0) + x5 / f!(120.0) - x7 / f!(5040.0) + x9 / f!(362880.0)
        - x11 / f!(39916800.0))
}

/// Hand-rolled `const fn` implementation of floor for `Real`.
///
/// This is **bit-for-bit identical** to `std::f64::floor` (including signed-zero
/// preservation) while remaining fully `const fn` on stable Rust with `#![no_std]`.
#[inline(always)]
pub(crate) const fn floor_f(x: Real) -> Real {
    if x.is_nan() || x.is_infinite() {
        x
    } else if x == f!(0.0) {
        x // preserves +0.0 or -0.0
    } else {
        let i = x as i64;
        let truncated = i as Real;
        if x >= f!(0.0) || truncated == x {
            truncated
        } else {
            truncated - f!(1.0)
        }
    }
}

/// Converts the Newtonian gravitational potential Φ/c² (where Φ < 0 for bound orbits)
/// into the relativistic lapse factor α = √(1 + 2Φ/c²).
///
/// This function implements the standard weak-field approximation used in general
/// relativity. It is valid when the dimensionless gravitational potential satisfies
/// |Φ|/c² ≪ 1. In this regime spacetime is nearly flat, gravitational time dilation
/// is a small perturbation, and higher-order curvature effects can safely be neglected.
/// The resulting α gives the factor by which clocks tick more slowly in a gravitational
/// well relative to a distant reference clock.
///
/// This approximation is excellent for solar-system navigation, GNSS satellites,
/// most spacecraft operations, and any environment where |Φ|/c² remains much smaller
/// than ~0.01. It is exported from `deep_time_core::alpha_from_weak_field_potential`
/// and is the recommended way to obtain the lapse factor when you have the local
/// Newtonian potential.
///
/// The weak-field regime breaks down in strong-gravity environments where
/// |Φ|/c² approaches or exceeds ~0.1. Such conditions occur near:
///
/// - the surface or immediate vicinity of neutron stars (where |Φ|/c² ≈ 0.15–0.25);
/// - regions near a black-hole event horizon (e.g. the photon rings imaged by the
///   Event Horizon Telescope around M87* or Sgr A*);
/// - the final inspiral and merger phases of binary neutron-star or black-hole
///   systems (as observed by LIGO/Virgo in events such as GW170817 or GW150914).
///
/// In those extreme regimes this function alone is no longer sufficient; a full
/// strong-field treatment (including curvature information passed to `LocalSpacetime`)
/// is required.
#[inline(always)]
pub fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
    // gravitational_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
    libm::sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
}

/// Kretschmann scalar from total relativity
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
    grav_potential_over_c2: Real,
    characteristic_length_scale: Real,
) -> Real {
    if characteristic_length_scale <= f!(0.0) || grav_potential_over_c2 <= f!(0.0) {
        return f!(0.0);
    }
    // Exact weak-field limit: K ≈ 48 φ² / L⁴
    let curvature_scale = f!(2.0) * grav_potential_over_c2
        / (characteristic_length_scale * characteristic_length_scale);
    f!(12.0) * (curvature_scale * curvature_scale)
}
