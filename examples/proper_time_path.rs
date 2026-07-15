//! Proper time along a sampled trajectory (`physics` feature).
//!
//! ## Purpose
//!
//! Given tabulated **coordinate time**, **inertial velocity**, and **gravitational
//! potential** Φ, compute:
//!
//! 1. Proper time accumulated by a clock on that path over `[t₀, t₁]`
//! 2. Differential proper time versus a constant-rate ground clock
//! 3. Clock drift relative to coordinate time (Δτ − Δt)
//!
//! The library integrates \(d\tau/dt\) from the samples you supply. It does not
//! propagate orbits or evaluate a gravity field. Φ and velocity must come from
//! your ephemeris or gravity model (here, a point-mass stand-in for illustration).
//!
//! ## Units and frames
//!
//! | API | Potential |
//! |-----|-----------|
//! | [`Dt::proper_time_from_states_between`], [`Dt::proper_time_drift_from_states`] | Φ in **m²/s²** (SI); divided by \(c^2\) internally |
//! | [`Spacetime::from_potential_velocity_and_scale`] | **Φ/c²** (dimensionless) |
//!
//! - Velocity: m/s; only **speed** enters the rate model.
//! - Use a consistent inertial-style frame for position (when forming Φ) and
//!   velocity (e.g. Earth-centered inertial).
//! - Set `characteristic_length_scale = 0.0` for LEO, GNSS, and solar-system
//!   work (disables higher-order curvature terms).
//!
//! Interval methods require `start ≤ end`, sample coverage of `[start, end]`,
//! and non-decreasing sample times. See [docs/trajectory.md](../docs/trajectory.md).
//!
//! ## Run
//!
//! ```text
//! cargo run --example proper_time_path --features physics
//! ```
//!
//! Assert-driven (quiet) so the example can run under automated test suites.

use deep_time::consts::C_SQUARED;
use deep_time::{Dt, DtErr, Scale, Spacetime, Velocity};

/// Earth gravitational parameter (m³/s²). Illustrative; use your project value.
const GM_EARTH: f64 = 3.986_004_418e14;

/// WGS 84 equatorial radius (m) — fixed “ground” radius for the illustration.
const R_GROUND_M: f64 = 6_378_137.0;

/// Illustrative LEO radius (m), ~400 km altitude above `R_GROUND_M`.
const R_CRAFT_M: f64 = 6_778_137.0;

/// Coordinate-time span of the arc (s).
const ARC_S: i128 = 3_600;

fn tai_sec(sec: i128) -> Dt {
    Dt::from_sec(sec, Scale::TAI, Scale::TAI)
}

/// Newtonian point-mass potential Φ = −GM/r (m²/s²).
///
/// Replace with the potential from your gravity model (multipoles, multi-body,
/// etc.) before operational use.
fn phi_point_mass(r_m: f64) -> f64 {
    -GM_EARTH / r_m
}

/// Local spacetime state from SI potential and speed (`characteristic_length_scale = 0`).
fn spacetime_from_phi_speed(phi_m2_s2: f64, speed_m_s: f64) -> Spacetime {
    Spacetime::from_potential_velocity_and_scale(
        phi_m2_s2 / C_SQUARED,
        Velocity::from_speed(speed_m_s),
        0.0,
    )
}

fn main() -> Result<(), DtErr> {
    // --- Ground station (fixed): constant proper-time rate -----------------
    // Operational sites: use ellipsoidal height and a geoid-consistent potential,
    // not a spherical −GM/R alone.
    let phi_ground = phi_point_mass(R_GROUND_M);
    let ground = spacetime_from_phi_speed(phi_ground, 0.0);
    let ground_rate = ground.proper_time_rate();

    // --- Vehicle samples (illustrative circular LEO, constant state) -------
    // In practice each row comes from ephemeris: (t, v_vector, Φ(r)).
    // Constant |v| and Φ here keep the API check exact (see asserts below).
    let speed_craft = (GM_EARTH / R_CRAFT_M).sqrt();
    let phi_craft = phi_point_mass(R_CRAFT_M);
    let craft = spacetime_from_phi_speed(phi_craft, speed_craft);
    let craft_rate = craft.proper_time_rate();

    let t0 = tai_sec(0);
    let t1 = tai_sec(ARC_S);
    let coordinate_span = t1.to_diff_raw(t0);

    let velocity_craft = Velocity::from_speed(speed_craft);
    let samples = [
        (t0, velocity_craft, phi_craft),
        (tai_sec(ARC_S / 2), velocity_craft, phi_craft),
        (t1, velocity_craft, phi_craft),
    ];

    // --- Integrate ---------------------------------------------------------
    // Proper time on the vehicle over [t0, t1].
    let proper_time_craft = Dt::proper_time_from_states_between(t0, t1, samples, 0.0)?;

    // Vehicle versus constant ground rate on the same coordinate interval.
    let path = [(t0, craft.clone()), (t1, craft)];
    let versus_ground = Dt::proper_time_differential_vs_rate(t0, t1, path, ground_rate)?;

    // Drift relative to coordinate time: Δτ − Δt along the vehicle path.
    let drift = Dt::proper_time_drift_from_states(t0, t1, samples, 0.0)?;

    // --- Consistency (uniform samples ⇒ closed-form constant-rate results) -
    let proper_time_const = t0.proper_time_between_constant_rate(t1, craft_rate);
    assert_eq!(
        proper_time_craft, proper_time_const,
        "proper_time_from_states_between must match constant-rate integration when v and Φ are fixed"
    );

    let proper_time_ground = t0.proper_time_between_constant_rate(t1, ground_rate);
    assert_eq!(
        versus_ground,
        proper_time_const.sub(proper_time_ground),
        "proper_time_differential_vs_rate must equal Δτ_craft − Δτ_ground"
    );

    assert_eq!(
        drift,
        proper_time_craft.sub(coordinate_span),
        "proper_time_drift_from_states must equal Δτ − Δt"
    );

    // --- Physics sanity for this geometry ----------------------------------
    // Lower altitude and higher speed than a surface clock at rest ⇒ lower
    // proper-time rate, negative differential vs ground, negative drift vs t.
    assert!(
        craft_rate < ground_rate,
        "craft rate {craft_rate} must be below ground rate {ground_rate}"
    );
    assert!(
        versus_ground.to_sec_f() < 0.0,
        "craft must lag ground over the arc (got {} s)",
        versus_ground.to_sec_f()
    );
    assert!(
        drift.to_sec_f() < 0.0,
        "craft must lag coordinate time (got {} s)",
        drift.to_sec_f()
    );

    // Order of magnitude: |craft − ground| over one hour is microseconds, not
    // milliseconds (for this point-mass illustration).
    let abs_diff_us = versus_ground.to_sec_f().abs() * 1e6;
    assert!(
        (0.1..100.0).contains(&abs_diff_us),
        "|Δτ_craft − Δτ_ground| = {abs_diff_us:.3} µs over {ARC_S} s coordinate time \
         (expected microsecond-class for this LEO illustration)"
    );

    Ok(())
}
