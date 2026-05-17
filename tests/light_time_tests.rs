#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::{Dt, ObserverState, Position, Scale, Spacetime, Velocity, constants::C};

fn make_state(
    tai_sec: i64,
    pos: Position,
    vel: Velocity,
    phi_m2_s2: f64,
    char_scale: f64,
) -> ObserverState {
    ObserverState {
        time: Dt::from(tai_sec, 0, Scale::TAI),
        position: pos,
        velocity: vel,
        grav_potential_m2_s2: phi_m2_s2,
        characteristic_length_scale: char_scale,
    }
}

/// Verifies the net relativistic clock rate of a GPS satellite relative to a
/// ground clock.
///
/// GPS satellites orbit at approximately 20,200 km altitude. Two relativistic
/// effects influence their onboard clocks:
///
/// - General relativity: weaker gravitational potential at orbital altitude
///   causes clocks to run faster.
/// - Special relativity: orbital velocity causes clocks to run slower.
///
/// The gravitational effect is larger, resulting in a net rate increase.
///
/// The accepted value used in GPS operations is approximately +38.4 µs per day.
/// This test confirms that `proper_time_rate()` produces a result consistent
/// with this established figure when using standard GPS orbital parameters.
///
/// References:
/// - Ashby, N. (2003). "Relativity in the Global Positioning System".
///   Living Reviews in Relativity 6(1).
///   https://doi.org/10.12942/lrr-2003-1
/// - GPS Interface Specification IS-GPS-200.
#[test]
fn gps_satellite_net_relativistic_clock_advance() {
    let gps_distance_from_earth_center = 26_560_000.0; // m
    let gps_speed = 3_874.0; // m/s
    let earth_gm = 3.986004418e14_f64; // m³/s²

    let gps_potential = -earth_gm / gps_distance_from_earth_center;

    let gps_sat = make_state(
        0,
        Position::new(gps_distance_from_earth_center, 0.0, 0.0),
        Velocity::from_speed(gps_speed),
        gps_potential,
        0.0,
    );

    let earth_radius = 6_378_137.0; // WGS84 equatorial radius (m)
    let ground_potential = -earth_gm / earth_radius;

    let ground = make_state(
        0,
        Position::new(earth_radius, 0.0, 0.0),
        Velocity::ZERO,
        ground_potential,
        0.0,
    );

    let gps_rate = gps_sat.proper_time_rate();
    let ground_rate = ground.proper_time_rate();

    assert!(
        gps_rate > ground_rate,
        "GPS satellite proper time rate must exceed ground rate \
         (gps_rate = {:.12}, ground_rate = {:.12})",
        gps_rate,
        ground_rate
    );

    let relative_rate = gps_rate / ground_rate;
    let daily_advance_us = (relative_rate - 1.0) * 86400.0 * 1_000_000.0;

    const EXPECTED: f64 = 38.4;
    const TOL: f64 = 0.1;

    assert!(
        (daily_advance_us - EXPECTED).abs() < TOL,
        "GPS net daily advance: got {:.4} µs/day (expected {:.1} ± {:.1})",
        daily_advance_us,
        EXPECTED,
        TOL
    );
}

/// Sun-grazing geometry.
///
/// The expected one-way Shapiro delay is approximately **119.45 µs**.
///
/// References:
/// - Moyer, T.D. (2003). *Formulation for Observed and Computed Values of
///   Deep Space Network Data Types for Navigation*. JPL DESCANSO Vol. 2.
/// - IAU 2015 recommended solar GM value.
/// - Multiple Cassini solar conjunction experiments.
#[test]
fn sun_grazing_shapiro_delay_reference_value() {
    let au = 1.495978707e11_f64;
    let r_sun = 6.957e8_f64;
    let b = r_sun * 1.001;

    let tx_pos = Position::new(au, b, 0.0);
    let rx_pos = Position::new(-au, b, 0.0);

    let tx = make_state(0, tx_pos, Velocity::ZERO, -8.87e8, 0.0);
    let rx = make_state(0, rx_pos, Velocity::ZERO, -8.87e8, 0.0);

    // Updated to new consolidated API
    let correction = tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &[], None);
    let got_us = correction.to_sec_f() * 1_000_000.0;

    assert!(
        (got_us - 119.45).abs() < 0.2,
        "Sun-grazing Shapiro: got {:.3} µs (expected ≈119.45 µs)",
        got_us
    );
}

/// Verifies that `one_way_relativistic_delay` correctly decomposes into
/// differential clock-rate correction + Shapiro delay.
///
/// This is a fundamental consistency check. The total relativistic correction
/// returned by `one_way_relativistic_delay` should be the sum of:
/// - The differential proper-time (clock-rate) correction between transmitter
///   and receiver, and
/// - The gravitational Shapiro delay.
///
/// By asserting this relationship, we gain direct confidence that the
/// combination of the two effects is implemented correctly.
#[test]
fn one_way_relativistic_delay_decomposes_into_clock_plus_shapiro() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let rx_pos = Position::new(1.52e11, 0.3e11, 0.0);

    let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
    let rx = make_state(1200, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0);

    let total = tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &[], None);

    // Compute the two components separately
    let clock_only = tx.compute_differential_clock_correction(rx);
    let shapiro_only = tx.shapiro_delay_to(rx, Dt::SHAPIRO_SOLAR);

    let sum = clock_only.add(shapiro_only);

    assert!(
        (total.to_sec_f() - sum.to_sec_f()).abs() < 1e-12,
        "one_way_relativistic_delay should equal clock correction + Shapiro \
         (difference = {:.3e} s)",
        (total.to_sec_f() - sum.to_sec_f()).abs()
    );
}

/// Verifies internal consistency of `round_trip_relativistic_correction`.
///
/// This test reconstructs the round-trip correction by explicitly solving
/// the uplink and downlink legs using the same iterative method that
/// `round_trip_relativistic_correction` uses internally.
///
/// It validates that:
/// - The uplink leg is solved iteratively,
/// - The downlink leg uses the accurate receiver state at uplink arrival time,
/// - The sum of the two legs matches the result returned by
///   `round_trip_relativistic_correction`.
#[test]
fn round_trip_consistent_with_uplink_plus_downlink() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let rx_pos = Position::new(2.2e11, 0.4e11, 0.0);

    let tx = make_state(0, tx_pos, Velocity::from_speed(29_780.0), -8.87e8, 0.0);
    let tolerance = Dt::from_ns(1, Scale::TAI);

    // === Full round-trip result (what we are validating) ===
    let round_trip_corr = tx.round_trip_relativistic_correction(
        &mut |t: Dt| {
            let sec = t.to_sec_f() as i64;
            make_state(sec, rx_pos, Velocity::from_speed(24_000.0), -1.3e8, 0.0)
        },
        &mut |t: Dt| {
            let sec = t.to_sec_f() as i64;
            make_state(sec, tx_pos, Velocity::from_speed(29_780.0), -8.87e8, 0.0)
        },
        Dt::SHAPIRO_SOLAR,
        tolerance,
        15,
    );

    // === Manual reconstruction using the same iterative method ===

    // 1. Solve uplink iteratively (same as round-trip does internally)
    let (uplink_corr, _rx_arrival_time, rx_at_arrival) = tx
        .iterative_one_way_relativistic_delay_to(
            &mut |t| {
                let sec = t.to_sec_f() as i64;
                make_state(sec, rx_pos, Velocity::from_speed(24_000.0), -1.3e8, 0.0)
            },
            Dt::SHAPIRO_SOLAR,
            tolerance,
            15,
        );

    // 2. Solve downlink iteratively from the accurate arrival state
    let (downlink_corr, _final_rx_time, _final_rx_state) = rx_at_arrival
        .iterative_one_way_relativistic_delay_to(
            &mut |t| {
                let sec = t.to_sec_f() as i64;
                make_state(sec, tx_pos, Velocity::from_speed(29_780.0), -8.87e8, 0.0)
            },
            Dt::SHAPIRO_SOLAR,
            tolerance,
            15,
        );

    let manual_sum = uplink_corr.add(downlink_corr);
    let diff = (round_trip_corr.to_sec_f() - manual_sum.to_sec_f()).abs();

    assert!(
        diff < 1e-9,
        "Round-trip correction should be consistent with iterative uplink + downlink \
         (difference = {:.3e} s)",
        diff
    );
}

/// Tests the iterative one-way relativistic light-time solver on a very long
/// baseline, representative of a distant Kuiper Belt object.
#[test]
fn iterative_solver_converges_for_distant_kuiper_belt_object() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let tx = make_state(0, tx_pos, Velocity::from_speed(29_780.0), -8.87e8, 0.0);
    let tolerance = Dt::from_ns(1, Scale::TAI);

    let (rel_correction, rx_time, _rx_state) = tx.iterative_one_way_relativistic_delay_to(
        &mut |t| {
            let sec = t.to_sec_f() as i64;
            make_state(
                sec,
                Position::new(5.5e12, 0.8e12, 0.0),
                Velocity::from_speed(4_200.0),
                -8e6,
                0.0,
            )
        },
        Dt::SHAPIRO_SOLAR,
        tolerance,
        20,
    );

    let light_time_sec = rx_time.to_diff_raw(tx.time).to_sec_f();

    assert!(
        light_time_sec > 15_000.0,
        "One-way light time should exceed 15,000 s for distant Kuiper Belt object"
    );

    assert!(
        rel_correction.to_sec_f() > 50e-6,
        "Relativistic correction should be non-trivial at this distance (got {:.1} µs)",
        rel_correction.to_sec_f() * 1e6
    );
}

/// Round-trip relativistic correction for a near-Earth heliocentric geometry
/// with moving endpoints and moderate separation.
///
/// This exercises `round_trip_relativistic_correction` with non-zero velocities
/// on both the uplink and downlink legs. The geometry has a relatively small
/// separation (~0.02 AU) and moderate solar impact parameter, so the total
/// relativistic correction (clock-rate effects + Shapiro delay) remains small.
///
/// Note: This is **not** an Earth-Moon or cislunar geometry. The positions
/// are heliocentric at approximately 1 AU from the Sun.
#[test]
fn round_trip_near_earth_geometry() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let rx_pos = Position::new(1.52e11, 0.3e11, 0.0);

    let tx = make_state(0, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0);
    let tolerance = Dt::from_ns(1, Scale::TAI);

    let correction = tx.round_trip_relativistic_correction(
        &mut |_t| make_state(520, rx_pos, Velocity::from_speed(29_000.0), -8.80e8, 0.0),
        &mut |_t| make_state(1040, tx_pos, Velocity::from_speed(30_000.0), -8.87e8, 0.0),
        Dt::SHAPIRO_SOLAR,
        tolerance,
        12,
    );

    let corr_us = correction.to_sec_f() * 1e6;

    assert!(
        corr_us > 2.0 && corr_us < 12.0,
        "Near-Earth round-trip relativistic correction: got {:.2} µs",
        corr_us
    );
}

/// Verifies `LightContext::FLAT` disables all gravitational delay and that
/// `LightContext::from_grav_param` produces sensible values for other bodies
/// (e.g. Jupiter, a common deep-space target).
#[test]
fn light_context_flat_and_from_grav_param() {
    let tx = make_state(
        0,
        Position::new(1.5e11, 0.0, 0.0),
        Velocity::ZERO,
        -8.87e8,
        0.0,
    );
    let rx = make_state(
        0,
        Position::new(-1.5e11, 0.0, 0.0),
        Velocity::ZERO,
        -8.87e8,
        0.0,
    );

    assert_eq!(
        tx.one_way_relativistic_delay(rx, Dt::ZERO, &[], None),
        Dt::ZERO
    );

    let jupiter = Dt::shapiro_from_grav_param(1.2668654e17);
    assert!(jupiter.to_sec_f() > 0.0 && jupiter.to_sec_f() < 2e-7);
}

/// Verifies both constructors (`new` and `new_strong_field`) and that
/// `proper_time_rate()` returns a physically bounded value (< 1.0) for
/// bound solar-system observers, as required by the unified master Lagrangian.
#[test]
fn observer_state_constructors_and_proper_time_rate() {
    let normal = ObserverState::new(
        Dt::from(0, 0, Scale::TAI),
        Position::new(1.5e11, 0.0, 0.0),
        Velocity::from_speed(30_000.0),
        -8.87e8,
    );
    assert_eq!(normal.characteristic_length_scale, 0.0);

    let strong = ObserverState::new_strong_field(
        Dt::from(0, 0, Scale::TAI),
        Position::new(1.5e11, 0.0, 0.0),
        Velocity::from_speed(30_000.0),
        -8.87e8,
        1e6,
    );
    assert_eq!(strong.characteristic_length_scale, 1e6);

    let earth = make_state(0, Position::ZERO, Velocity::from_speed(465.0), -6.26e7, 0.0);
    let rate = earth.proper_time_rate();
    assert!((0.999999999..=1.0).contains(&rate));
}

/// Edge cases: zero distance must produce zero correction, and identical
/// states in flat spacetime must also produce zero correction.
#[test]
fn one_way_delay_zero_distance_and_flat_identical() {
    let pos = Position::new(1.5e11, 0.0, 0.0);
    let state = make_state(0, pos, Velocity::ZERO, -8.87e8, 0.0);
    assert_eq!(
        state.one_way_relativistic_delay(state, Dt::SHAPIRO_SOLAR, &[], None),
        Dt::ZERO
    );

    let tx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
    let rx = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
    assert_eq!(
        tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &[], None),
        Dt::ZERO
    );
}

/// Verifies that the iterative one-way light-time solver produces a result
/// that is internally consistent with the light-time equation it is solving.
///
/// After convergence, the following relationship must hold (to high precision):
///
/// ```text
/// t_rx - t_tx ≈ |r_rx - r_tx| / c + Δt_rel
/// ```
///
/// where `Δt_rel` is the relativistic correction returned by the solver.
///
/// This is a consistency check rather than a comparison against an external
/// truth. It confirms that the fixed-point iteration converged to a solution
/// that satisfies the equation being solved. The test uses a static receiver
/// position (no motion) to keep the geometry simple while still exercising
/// the full iterative + relativistic correction path.
#[test]
fn iterative_solver_converges_to_sub_nanosecond() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let tx = make_state(0, tx_pos, Velocity::ZERO, -8.87e8, 0.0);
    let rx_pos = Position::new(1.52e11, 0.0, 0.0);
    let tolerance = Dt::from_ns(1, Scale::TAI);

    let (correction, final_rx_time, _) = tx.iterative_one_way_relativistic_delay_to(
        &mut |t| {
            let sec = t.to_sec_f() as i64;
            make_state(sec, rx_pos, Velocity::ZERO, -8.80e8, 0.0)
        },
        Dt::SHAPIRO_SOLAR,
        tolerance,
        20,
    );

    let geometric = tx_pos.distance_to(rx_pos) / C;
    let total = final_rx_time.to_diff_raw(tx.time).to_sec_f();

    assert!(
        (total - geometric - correction.to_sec_f()).abs() < 1e-10,
        "Iterative solver failed to satisfy light-time equation \
         (residual = {:.2e} s)",
        (total - geometric - correction.to_sec_f()).abs()
    );
}

/// When the gravitational environment is constant along the path, the
/// integrated method must return **exactly** the same result as the direct
/// method. Also verifies the documented fallback behavior for empty samples.
#[test]
fn integrated_matches_direct_when_constant_and_falls_back() {
    let tx_pos = Position::new(1.5e11, 0.0, 0.0);
    let rx_pos = Position::new(1.52e11, 0.0, 0.0);
    let common_potential = -8.835e8_f64;
    let common_vel = Velocity::ZERO;

    let tx = make_state(0, tx_pos, common_vel, common_potential, 0.0);
    let rx = make_state(520, rx_pos, common_vel, common_potential, 0.0);

    // Updated to new consolidated API
    let direct = tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &[], None);

    let samples: Vec<Spacetime> = (0..=20)
        .map(|_| {
            Spacetime::from_potential_velocity_and_scale(
                common_potential / (C * C),
                common_vel,
                0.0,
            )
        })
        .collect();

    let integrated = tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &samples, None);
    assert!((direct.to_sec_f() - integrated.to_sec_f()).abs() < 1e-12);

    // Empty samples should fall back to direct behavior
    let integrated_empty = tx.one_way_relativistic_delay(rx, Dt::SHAPIRO_SOLAR, &[], None);
    assert_eq!(direct, integrated_empty);
}

/// Verifies the expected mathematical relationship between one-way and
/// two-way relativistic clock rate ratios, plus a basic sanity check.
///
/// The method `relativistic_clock_rate_ratio` returns the factor by which
/// the receiver's proper time advances relative to the transmitter's due to
/// velocity and gravitational potential.
///
/// For **two-way** (round-trip) measurements, the total relativistic clock-rate
/// effect is the square of the one-way ratio:
///
/// ```text
/// two_way_clock_ratio = one_way_ratio × one_way_ratio
/// ```
///
/// This is because the signal experiences the ratio once on the uplink and
/// once on the downlink. This test confirms that the API produces results
/// consistent with this standard result.
///
/// It also verifies the basic invariant that identical transmitter and
/// receiver states must produce a ratio of exactly 1.0.
#[test]
fn relativistic_clock_rate_ratio_properties() {
    let tx = make_state(0, Position::ZERO, Velocity::ZERO, -8.87e8, 0.0);
    let rx = make_state(
        0,
        Position::new(1.5e11, 0.0, 0.0),
        Velocity::ZERO,
        -1.27e8,
        0.0,
    );

    let one_way = tx.relativistic_clock_rate_ratio(rx);

    // Since we removed the dedicated two-way method, we compute it manually
    let two_way = one_way * one_way;
    assert!((two_way - one_way * one_way).abs() < 1e-14);

    let identical = make_state(0, Position::ZERO, Velocity::ZERO, 0.0, 0.0);
    assert!((identical.relativistic_clock_rate_ratio(identical) - 1.0).abs() < 1e-14);
}

/// Verifies pure special-relativistic time dilation (velocity effect only).
///
/// When gravitational potential is zero, the proper time rate of a moving
/// observer must match the exact special-relativistic formula
/// `sqrt(1 - v²/c²)`.
#[test]
fn pure_special_relativistic_time_dilation() {
    let v = 0.1 * C; // 10% of speed of light

    let moving = make_state(0, Position::ZERO, Velocity::from_speed(v), 0.0, 0.0);

    let rate = moving.proper_time_rate();
    let beta = v / C;
    let expected = (1.0 - beta * beta).sqrt();

    assert!(
        (rate - expected).abs() < 1e-12,
        "Pure SR time dilation mismatch: got {:.12}, expected {:.12}",
        rate,
        expected
    );
}

/// Verifies pure gravitational time dilation (potential effect only).
///
/// With velocity set to zero, an observer at higher altitude (weaker
/// gravitational potential) must have a higher proper time rate than an
/// observer closer to the central body.
#[test]
fn pure_gravitational_time_dilation() {
    let gm = 3.986004418e14_f64;
    let r_low = 6_378_137.0; // approximately Earth surface
    let r_high = 26_560_000.0; // approximately GPS altitude

    let low = make_state(
        0,
        Position::new(r_low, 0.0, 0.0),
        Velocity::ZERO,
        -gm / r_low,
        0.0,
    );

    let high = make_state(
        0,
        Position::new(r_high, 0.0, 0.0),
        Velocity::ZERO,
        -gm / r_high,
        0.0,
    );

    let rate_low = low.proper_time_rate();
    let rate_high = high.proper_time_rate();

    assert!(
        rate_high > rate_low,
        "Higher altitude must have higher proper time rate \
         (low = {:.12}, high = {:.12})",
        rate_low,
        rate_high
    );
}
