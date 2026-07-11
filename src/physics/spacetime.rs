//! Local spacetime state (α, β, curvature) for proper-time rates.

use crate::{C_SQUARED, Drift, Position, Real, Velocity, sqrt};

/// Snapshot of the local quantities that set a clock’s rate \(d\tau/dt\).
///
/// Think of this as “how gravity and motion look right here, right now” for a
/// clock:
///
/// - **α** — gravitational redshift factor (deeper in a well → smaller α →
///   slower clocks).
/// - **β** — speed as a fraction of light speed (\(v/c\)).
/// - **kretschmann** — a curvature measure; leave at `0.0` for almost all
///   Earth/solar-system work.
///
/// Trajectory APIs either take [`Spacetime`] samples directly, or build them
/// from velocity and potential via
/// [`Spacetime::from_potential_velocity_and_scale`].
///
/// Instantaneous rate: [`Spacetime::proper_time_rate`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Spacetime {
    /// Gravitational lapse (redshift) factor α.
    ///
    /// Clocks run slower where gravity is stronger: α &lt; 1 in a potential well.
    /// In the weak field, α ≈ √(1 + 2Φ/c²) with Φ &lt; 0.
    pub alpha: Real,

    /// Local three-velocity β = v/c in the coordinate rest frame used for the analysis.
    pub beta: Real,

    /// Kretschmann scalar (curvature invariant), in geometric units of the model.
    ///
    /// For solar-system, GNSS, and similar work leave this **0.0** — the
    /// curvature correction is negligible. Non-zero values matter only in
    /// extreme gravity (near compact objects), where you may estimate K from
    /// potential and a length scale (see
    /// [`Spacetime::kretschmann_from_potential_and_scale`]) or supply K from a
    /// metric.
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

    /// Instantaneous proper-time rate \(d\tau/dt\) for this snapshot.
    ///
    /// Dimensionless: `1.0` means the clock tracks coordinate time; values a
    /// little below `1.0` are typical when moving or sitting in a gravitational
    /// well. Same calculation as [`Drift::proper_time_rate`] after
    /// [`Drift::from_spacetime`].
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        Drift::from_spacetime(self).proper_time_rate()
    }

    /// Build from lapse α, a velocity vector, and Kretschmann K.
    ///
    /// Sets β from [`Velocity::beta`]. Pass `kretschmann = 0.0` for ordinary
    /// weak-field work.
    #[inline]
    pub const fn from_gravitic_and_velocity(
        alpha: Real,
        velocity: Velocity,
        kretschmann: Real,
    ) -> Spacetime {
        Self::new(alpha, velocity.beta(), kretschmann)
    }

    /// Weak-field lapse from dimensionless potential: α = √(1 + 2Φ/c²).
    ///
    /// Given how deep you are in a gravity well (as Φ/c²), return the factor by
    /// which clocks run slow. Φ is **negative** for bound gravity, so α &lt; 1.
    ///
    /// ## Validity
    ///
    /// Good when |Φ|/c² ≪ 1 (Earth, solar system, most spacecraft). Not
    /// sufficient alone near neutron stars or black holes (|Φ|/c² ≳ 0.1); then
    /// you need a strong-field metric treatment and usually a non-zero
    /// Kretschmann on [`Spacetime`].
    ///
    /// ## Note on units
    ///
    /// Argument is **Φ/c²** (dimensionless), not Φ in m²/s². Trajectory
    /// `*_from_states` APIs take SI Φ and divide by \(c^2\) for you.
    #[inline]
    pub const fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
        // grav_potential_over_c2 = Φ/c² < 0 → α < 1 (clocks run slower)
        sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
    }

    /// Estimate Kretschmann scalar \(\mathcal{K} \approx 48\,\phi^2 / L^4\).
    ///
    /// Optional helper to guess curvature from potential strength and a length
    /// scale. For normal flight timing you do **not** need this: pass
    /// `characteristic_length_scale = 0.0` and get K = 0.
    ///
    /// ## Parameters
    ///
    /// - `grav_potential_over_c2` — Φ/c² (typically **negative**). The estimate
    ///   uses φ², so the sign of φ does not matter for K.
    /// - `characteristic_length_scale` — meters. Use **`0.0`** to disable
    ///   (recommended default). A positive L is a curvature scale; for a single
    ///   spherical mass the Schwarzschild match is L = r with
    ///   |φ| = GM/(c² r). L cannot be recovered from φ alone in general.
    ///
    /// Background: [relativity model](https://github.com/ragardner/deep-time/blob/main/docs/relativity.md).
    pub const fn kretschmann_from_potential_and_scale(
        grav_potential_over_c2: Real,
        characteristic_length_scale: Real,
    ) -> Real {
        // Weak-field default: no length scale → curvature term disabled.
        // Do **not** reject negative φ: bound-system potentials are negative, and the
        // estimate uses φ² (see below).
        if characteristic_length_scale <= f!(0.0) {
            return f!(0.0);
        }
        // Weak-field limit: K ≈ 48 φ² / L⁴
        // (curvature_scale = 2φ/L² ⇒ 12 · (curvature_scale)² = 48 φ²/L⁴)
        let curvature_scale = f!(2.0) * grav_potential_over_c2
            / (characteristic_length_scale * characteristic_length_scale);
        f!(12.0) * (curvature_scale * curvature_scale)
    }

    /// Build [`Spacetime`] from dimensionless potential Φ/c², velocity, and length scale.
    ///
    /// Turn “how deep in the well” and “how fast I’m moving” into the α, β, K
    /// snapshot used for clock rates.
    ///
    /// ## Parameters
    ///
    /// - `grav_potential_over_c2` — **Φ/c²** (dimensionless), not SI Φ.
    /// - `velocity` — m/s; only speed enters (via β).
    /// - `characteristic_length_scale` — pass **`0.0`** for solar-system / GNSS
    ///   work (K = 0). Positive L only if you want the optional K estimate.
    ///
    /// For SI potential (m²/s²), divide by \(c^2\) first, or use trajectory
    /// `proper_time_*_from_states` which does that conversion.
    ///
    /// Weak-field α is valid for |Φ|/c² ≪ 1. Strong gravity needs more than
    /// this constructor alone.
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

    /// Newtonian point-mass potential Φ = −Σ GMᵢ / rᵢ at a position (m²/s²).
    ///
    /// Sums “how much gravity well” you feel from a list of bodies treated as
    /// point masses. The result is **negative** near masses. Use it to build
    /// samples for trajectory proper-time APIs, or convert to α via
    /// Φ/c² and [`Spacetime::alpha_from_weak_field_potential`].
    ///
    /// ## Limits
    ///
    /// Point masses only — no Earth \(J_2\), no tides, no extended bodies. Fine
    /// for rough multi-body Φ or cislunar order-of-magnitude work; LEO-grade
    /// timing usually needs multipoles from a full gravity model.
    ///
    /// Body positions and the evaluation point must share the same coordinate
    /// frame.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Position, Spacetime};
    ///
    /// let bodies = [
    ///     (Position::from_au(0.0, 0.0, 0.0), 1.3271244e20),     // Sun GM
    ///     (Position::from_au(1.0, 0.0, 0.0), 3.9860044e14),     // Earth GM
    ///     (Position::from_au(1.00257, 0.0, 0.0), 4.9048695e12), // Moon GM
    /// ];
    /// let position = Position::from_au(1.001, 0.001, 0.0);
    /// let phi = Spacetime::grav_potential_from_point_masses(
    ///     &position,
    ///     bodies.iter().cloned(),
    /// );
    /// assert!(phi < 0.0);
    /// ```
    pub fn grav_potential_from_point_masses<I>(position: &Position, bodies: I) -> Real
    where
        I: IntoIterator<Item = (Position, Real)>, // (body_position, GM in m³/s²)
    {
        let mut phi = 0.0;
        for (body_pos, gm) in bodies {
            let r = position.distance_to(&body_pos);
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
