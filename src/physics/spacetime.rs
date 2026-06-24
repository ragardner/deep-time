//! Local spacetime state (α, β, curvature).

use crate::{C_SQUARED, Drift, Position, Real, Velocity, sqrt};

/// The three local spacetime quantities that fully determine how fast an observer’s
/// proper time advances relative to coordinate time.
///
/// This structure holds the gravitational lapse factor, the observer’s local velocity,
/// and the curvature information needed for the library’s unified proper-time model.
/// It is the low-level input that `Drift` uses internally.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Spacetime {
    /// Gravitational lapse (redshift) factor α.  
    /// This is the factor by which clocks run slower in a gravitational potential.
    pub alpha: Real,

    /// Local three-velocity β = v/c measured in the coordinate rest frame.
    pub beta: Real,

    /// Kretschmann scalar (a scalar measure of spacetime curvature).  
    /// In the weak-field regime — where |Φ|/c² ≪ 1 and the gravitational field varies
    /// over macroscopic distances — this value is effectively zero and can safely be
    /// left at its default. It only becomes numerically relevant in strong-field
    /// environments such as:
    ///
    /// - the surface or immediate vicinity of neutron stars (where |Φ|/c² ≈ 0.15–0.25);
    /// - regions near a black-hole event horizon (e.g. the photon rings imaged by the
    ///   Event Horizon Telescope around M87* or Sgr A*);
    /// - the final inspiral and merger phases of binary neutron-star or black-hole
    ///   systems (as observed by LIGO/Virgo in events such as GW170817 or GW150914).
    ///
    /// In these regimes a realistic non-zero value (estimated from the local potential
    /// and a characteristic length scale) activates the library’s intrinsic Planck-scale
    /// saturation term.
    pub kretschmann: Real,
}

impl Spacetime {
    #[inline]
    pub const fn new(alpha: Real, beta: Real, kretschmann: Real) -> Spacetime {
        Self {
            alpha,
            beta,
            kretschmann,
        }
    }

    /// Returns the instantaneous proper-time rate `dτ/dt` from this snapshot.
    ///
    /// Convenience method that internally uses the same unified calculation as
    /// `Drift::proper_time_rate`.
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        Drift::from_spacetime(self).proper_time_rate()
    }

    /// Convenience for direct gravimeter / sensor paths.
    #[inline]
    pub const fn from_gravitic_and_velocity(
        alpha: Real,
        velocity: Velocity,
        kretschmann: Real,
    ) -> Spacetime {
        Self::new(alpha, velocity.beta(), kretschmann)
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
    /// than ~0.01. It is exported from `deep_time::alpha_from_weak_field_potential`
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
    /// strong-field treatment (including curvature information passed to `Spacetime`)
    /// is required.
    #[inline]
    pub const fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
        // gravitational_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
        sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
    }

    /// Kretschmann scalar from total relativity
    /// Computes the Kretschmann scalar \(\mathcal{K}\) from the total gravitational
    /// relativity experienced by a local observer at the observer’s spacetime point.
    ///
    /// This is the canonical, physics-true convenience function for the master Lagrangian.
    ///
    /// Information on the master Lagrangian can be found
    /// [here](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).
    ///
    /// It uses:
    /// - `phi` = Φ/c² — the total local gravitational potential (redshift/gravity effect)
    ///   felt by the observer from all masses.
    /// - `characteristic_length_scale` — the typical length scale (in meters) over which
    ///   the gravitational field varies at the observer’s location.
    ///
    /// **For existing weak-field users** (Earth orbit, GNSS, solar-system navigation):
    /// Supply your existing `phi` value and set `characteristic_length_scale = 0.0`.
    /// The function safely returns 0.0 (the value in double precision).
    ///
    /// **For strong-field / future users** (black-hole flybys, neutron stars, direct
    /// gravimeters, or full metric evaluation):
    /// Supply the measured or computed \(\phi\) and the real local length scale (or
    /// the value from your metric). The function returns a physically accurate non-zero
    /// curvature.
    pub const fn kretschmann_from_potential_and_scale(
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

    /// Computes both the gravitational lapse factor `α` and an estimate of the
    /// Kretschmann scalar from the dimensionless gravitational potential Φ/c²
    /// and a characteristic length scale.
    ///
    /// The lapse factor α is computed using `alpha_from_weak_field_potential`,
    /// which is the standard weak-field expression α = √(1 + 2Φ/c²). It is valid
    /// when the dimensionless gravitational potential satisfies |Φ|/c² ≪ 1. In
    /// this regime spacetime is nearly flat, gravitational time dilation is a
    /// small perturbation, and higher-order curvature effects can safely be
    /// neglected. The resulting α gives the factor by which clocks tick more
    /// slowly in a gravitational well relative to a distant reference clock.
    ///
    /// This approximation is excellent for solar-system navigation, GNSS
    /// satellites, most spacecraft operations, and any environment where
    /// |Φ|/c² remains much smaller than ~0.01. It is exported from
    /// `deep_time::alpha_from_weak_field_potential` and is the recommended
    /// way to obtain the lapse factor when you have the local Newtonian potential.
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
    /// strong-field treatment (including curvature information passed to `Spacetime`)
    /// is required.
    ///
    /// For the `characteristic_length_scale` parameter:
    /// - In weak-field conditions, pass `0.0`. This returns exactly the same clock
    ///   rate as the classic relativistic formulation and sets the Kretschmann scalar
    ///   to zero (its default value for all ordinary navigation, GNSS, or solar-system
    ///   work).
    /// - In strong-field conditions, supply the typical length scale (in meters) over
    ///   which the gravitational field varies significantly at the observer’s location.
    ///   This allows the library to estimate the Kretschmann scalar and activate the
    ///   intrinsic Planck-scale saturation term when curvature becomes extreme.
    pub const fn from_potential_velocity_and_scale(
        grav_potential_over_c2: Real, // Φ/c² (total local potential)
        velocity: Velocity,
        characteristic_length_scale: Real,
    ) -> Spacetime {
        let alpha: Real = Self::alpha_from_weak_field_potential(grav_potential_over_c2);
        let kretschmann: Real = Self::kretschmann_from_potential_and_scale(
            grav_potential_over_c2,
            characteristic_length_scale,
        );
        Self::from_gravitic_and_velocity(alpha, velocity, kretschmann)
    }

    /// Recovers the Newtonian gravitational potential Φ (m²/s²) from the
    /// gravitational lapse factor α using the weak-field relation.
    ///
    /// \[
    /// \alpha = \sqrt{1 + \frac{2\Phi}{c^2}} \quad\implies\quad
    /// \Phi = \frac{c^2}{2}(\alpha^2 - 1)
    /// \]
    ///
    /// This is the inverse of [`Spacetime::alpha_from_weak_field_potential`].
    #[inline]
    pub const fn grav_potential_from_alpha(alpha: Real) -> Real {
        let alpha_sq = alpha * alpha;
        (alpha_sq - f!(1.0)) / f!(2.0) * C_SQUARED
    }

    /// Computes the total Newtonian gravitational potential Φ at a given position
    /// from an arbitrary collection of point-mass bodies (Sun, Earth, Moon,
    /// planets, asteroids, etc.).
    ///
    /// This is the standard method used by real mission planners (Apollo,
    /// Artemis, Mars orbiters, lunar landers) and in open-source astrodynamics
    /// libraries (SPICE/NAIF, Orekit, GMAT, poliastro). It evaluates
    ///
    /// \[
    /// \Phi = -\sum_i \frac{GM_i}{r_i}
    /// \]
    ///
    /// ## Examples
    ///
    /// Realistic cislunar trajectory
    ///
    /// ```rust
    /// use deep_time::{Position, Spacetime};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20),     // Sun
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14),     // Earth
    ///     (Position::from_au(1.00257, 0.0, 0.0), 4.9048695e12), // Moon
    /// ];
    ///
    /// let position = Position::from_au(1.001, 0.001, 0.0); // e.g. spacecraft, asteroid, etc.
    ///
    /// let phi = Spacetime::grav_potential_from_point_masses(
    ///     position,
    ///     bodies.iter().copied(),
    /// );
    /// ```
    pub fn grav_potential_from_point_masses<I>(position: Position, bodies: I) -> Real
    where
        I: IntoIterator<Item = (Position, Real)>, // (body_position, GM in m³/s²)
    {
        let mut phi = 0.0;
        for (body_pos, gm) in bodies {
            let r = position.distance_to(body_pos);
            if r > 0.0 {
                phi -= gm / r;
            }
        }
        phi
    }
}

#[cfg(feature = "wire")]
impl Spacetime {
    /// Size of the canonical wire representation in bytes (24 bytes).
    pub const WIRE_SIZE: usize = 24;

    /// Serializes this [`Spacetime`] snapshot into a fixed 24-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.alpha.to_le_bytes());
        buf[8..16].copy_from_slice(&self.beta.to_le_bytes());
        buf[16..24].copy_from_slice(&self.kretschmann.to_le_bytes());
        buf
    }

    /// Deserializes a [`Spacetime`] from exactly 24 bytes.
    ///
    /// ## Security
    ///
    /// Accepts any `f64` bit pattern (including `NaN`/`Inf`) to match the
    /// type’s own invariants. Fixed size makes it immune to length-based
    /// attacks. Safe for untrusted input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_SIZE {
            return None;
        }
        let alpha = Real::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let beta = Real::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let kretschmann = Real::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        Some(Self {
            alpha,
            beta,
            kretschmann,
        })
    }
}
