//! Gravity, spatial velocity, and proper-time rate for one clock.

use crate::{C_SQUARED, Real, sqrt};

use super::{Position, Velocity};

/// Lapse α and spatial-velocity fraction β for one clock, written in a
/// coordinate system you already chose. The tick rate is compared to that
/// system’s time \(t\) (the same \(t\) used when measuring spatial velocity).
/// This struct does not store a second clock, a
/// [`Position`], or a time-scale tag.
///
/// **α** is the lapse: the gravitational redshift factor of general relativity.
/// With no shift, \(\alpha=\sqrt{-g_{00}}\). It is the number of seconds a
/// clock with no spatial velocity (\(\beta = 0\)) ticks during one second of
/// coordinate time \(t\). From gravitational potential,
/// \(\alpha=\sqrt{1+2\Phi/c^2}\). Φ is the potential of the field (negative
/// for bound gravity), not a location. Whether α is less than 1 depends on
/// how \(t\) is scaled: if Φ → 0 at infinity, a bound well has α < 1. On one
/// shared \(t\), a more negative Φ gives a smaller α.
///
/// **β** is spatial velocity in that coordinate system, as a fraction of light
/// speed. Spatial velocity \(v\) is the [`Velocity`] vector:
/// metres of travel through space per one second of the same \(t\).
/// \(\beta = |v|/c\).
///
/// α = 1 and β = 0 means no spatial velocity and a lapse of 1, so the clock
/// ticks in step with \(t\).
///
/// The general-relativity formula is
/// [`proper_time_rate_offset`](Self::proper_time_rate_offset). Fill from Φ and
/// spatial velocity with
/// [`from_potential_and_velocity`](Self::from_potential_and_velocity), or pass
/// α and β from a metric with [`new`](Self::new).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Spacetime {
    /// Lapse: gravitational redshift factor \(\alpha=\sqrt{-g_{00}}\) (no shift).
    ///
    /// This is the number of seconds a clock with no spatial velocity
    /// (\(\beta = 0\)) ticks during one second of coordinate time \(t\). From
    /// gravitational potential Φ, \(\alpha=\sqrt{1+2\Phi/c^2}\) with Φ
    /// negative for bound gravity. Whether that is less than 1 depends on how
    /// \(t\) is scaled; on one shared \(t\), a more negative Φ gives a smaller
    /// α.
    pub alpha: Real,

    /// Spatial velocity in the same coordinate system as α, as a fraction of
    /// light speed: \(\beta = |v|/c\). Spatial velocity \(v\) is metres of
    /// travel through space per one second of that \(t\).
    pub beta: Real,
}

impl Spacetime {
    /// Lapse α and spatial-velocity fraction β in one coordinate system.
    ///
    /// When these come from a metric, β is the Eulerian speed as a fraction of
    /// light speed (from the spatial metric, not a raw coordinate speed). For
    /// solar-system and GNSS work prefer
    /// [`from_potential_and_velocity`](Self::from_potential_and_velocity),
    /// which fills \(\alpha=\sqrt{1+2\Phi/c^2}\) and Euclidean \(\beta=|v|/c\).
    #[inline]
    pub const fn new(alpha: Real, beta: Real) -> Spacetime {
        Self { alpha, beta }
    }

    /// Number of seconds this clock ticks during one second of the coordinate
    /// time \(t\) that α and β were written in.
    ///
    /// `1.0` means the clock ticks in step with that \(t\). Below `1.0` it
    /// ticks slower than that \(t\). \(t\) is not an argument; it is implied
    /// by how α and β were built. Equal to `1 +`
    /// [`proper_time_rate_offset`](Self::proper_time_rate_offset).
    #[inline]
    pub const fn proper_time_rate(&self) -> Real {
        f!(1.0) + self.proper_time_rate_offset()
    }

    /// General-relativity proper-time equation: \(d\tau/dt - 1\).
    ///
    /// Returns how many extra (or fewer) seconds this clock ticks during one
    /// second of coordinate time \(t\). Negative means it ticked slower than
    /// \(t\). Zero means it matched \(t\). `Drift` and the trajectory
    /// integrators use this value.
    ///
    /// \(t\) is not an input, and there is no second clock on this method. It
    /// is a rate, not a clock reading. α is the lapse: the number of seconds a
    /// clock with no spatial velocity ticks during one second of \(t\), equal
    /// to \(\sqrt{-g_{00}}\) with no shift. β is spatial velocity as a
    /// fraction of light speed. Spatial velocity \(v\) is metres of travel
    /// through space per one second of that same \(t\); \(\beta = |v|/c\).
    ///
    /// \[
    /// \frac{d\tau}{dt} = \alpha\sqrt{1-\beta^2}.
    /// \]
    ///
    /// To compare two clocks, give each its own α and β and subtract the rates
    /// ([`Dt::proper_time_differential_vs_rate`](crate::Dt::proper_time_differential_vs_rate)
    /// /
    /// [`Dt::proper_time_differential_from_paths`](crate::Dt::proper_time_differential_from_paths)).
    ///
    /// When α and β come from Φ and spatial velocity
    /// ([`from_potential_and_velocity`](Self::from_potential_and_velocity)),
    /// the \(O(c^{-2})\) expansion is IERS Conventions (2010) eqs. (10.6)–(10.7)
    /// and Ashby (2003). This method evaluates the square-root interval, not
    /// that linearized right-hand side. Φ is negative for bound gravity; IERS
    /// uses a positive \(U_E\) (\(\Phi=-U_E\)). IERS writes \(t\) as TCG in
    /// GCRS; this crate takes \(t\) as whichever coordinate time Φ and \(v\)
    /// were computed with. IERS eqs. (10.8)–(10.9) are the same expansion with
    /// \(t\) as TT and an extra conventional rate \(L_G\); this method does
    /// not add \(L_G\).
    ///
    /// Computed as \((\delta-1)/(\sqrt{\delta}+1)\) with
    /// \(\delta=\max(\alpha^2(1-\beta^2),0)\), which equals \(\sqrt{\delta}-1\)
    /// without evaluating \(\sqrt{1+\varepsilon}-1\) in floating point.
    ///
    /// ## References
    ///
    /// - Petit, G. and Luzum, B. (eds.), *IERS Conventions (2010)*, IERS
    ///   Technical Note 36, §10.2, eqs. (10.6)–(10.7); see also (10.8)–(10.9)
    ///   for the same expansion with \(t\) as TT.
    /// - Ashby, N., “Relativity in the Global Positioning System,”
    ///   *Living Reviews in Relativity* **6**, 1 (2003).
    /// - Soffel, M. et al., “The IAU 2000 resolutions for astrometry, celestial
    ///   mechanics and metrology in the relativistic framework,” *Astron. J.*
    ///   **126**, 2687 (2003).
    #[inline]
    pub const fn proper_time_rate_offset(&self) -> Real {
        let delta = (self.alpha * self.alpha * (f!(1.0) - self.beta * self.beta)).max(f!(0.0));
        (delta - f!(1.0)) / (sqrt(delta) + f!(1.0))
    }

    /// Combines a lapse α with a spatial-velocity vector in the same coordinate
    /// system.
    ///
    /// Sets β from [`Velocity::beta`]: \(|v|/c\), where spatial velocity \(v\)
    /// is metres of travel through space per one second of that system’s \(t\).
    #[inline]
    pub const fn from_lapse_and_velocity(alpha: Real, velocity: Velocity) -> Spacetime {
        Self::new(alpha, velocity.beta())
    }

    /// Builds the lapse α from gravitational potential:
    /// \(\alpha=\sqrt{1+2\Phi/c^2}\).
    ///
    /// α is the gravitational redshift factor (\(\sqrt{-g_{00}}\) with no
    /// shift). It is the number of seconds a clock with no spatial velocity
    /// ticks during one second of coordinate time \(t\). The `1` in the
    /// formula is this coordinate system’s scale: Φ = 0 gives α = 1, so a
    /// clock with no spatial velocity ticks in step with \(t\). Φ is
    /// **negative** for bound gravity. If Φ → 0 at infinity, a bound well has
    /// α < 1.
    ///
    /// Use this for Earth, GNSS, and solar-system work (IERS / Ashby). Near a
    /// compact object pass the metric lapse to [`new`](Self::new) instead.
    ///
    /// The argument is **Φ/c²** (dimensionless). Trajectory `*_from_states`
    /// APIs take SI Φ (m²/s²) and divide by \(c^2\) for you.
    #[inline]
    pub const fn alpha_from_weak_field_potential(grav_potential_over_c2: Real) -> Real {
        // Φ/c²; Φ → 0 at infinity and Φ < 0 in a bound well ⇒ α < 1
        sqrt((f!(1.0) + f!(2.0) * grav_potential_over_c2).max(f!(0.0)))
    }

    /// Builds α and β from gravitational potential Φ and spatial velocity,
    /// both in one coordinate system you already chose.
    ///
    /// Φ is the gravitational potential of the field (how deep the gravity
    /// well is), not a [`Position`]. Spatial velocity \(v\)
    /// (the [`Velocity`] vector) is metres of travel through
    /// space per one second of that system’s \(t\). This function does not
    /// take a reference clock or a time-scale tag; the comparison to \(t\) is
    /// the \(t\) of that system.
    ///
    /// Fills the lapse \(\alpha=\sqrt{1+2\Phi/c^2}\) and \(\beta=|v|/c\), then
    /// uses the same interval as [`new`](Self::new).
    ///
    /// - `grav_potential_over_c2` — **Φ/c²** (dimensionless), not SI Φ.
    ///   Φ is **negative** for bound gravity. IERS writes a positive \(U_E\)
    ///   (\(\Phi=-U_E\)).
    /// - `velocity` — spatial velocity in m/s in the same frame; only the
    ///   speed \(|v|\) enters (via β).
    ///
    /// For SI potential (m²/s²), divide by \(c^2\) first, or use trajectory
    /// `proper_time_*_from_states` which does that conversion.
    pub const fn from_potential_and_velocity(
        grav_potential_over_c2: Real,
        velocity: Velocity,
    ) -> Spacetime {
        let alpha: Real = Self::alpha_from_weak_field_potential(grav_potential_over_c2);
        Self::from_lapse_and_velocity(alpha, velocity)
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
    /// use deep_time::physics::{Position, Spacetime};
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
    /// Size of the canonical wire representation in bytes (16 bytes).
    pub const WIRE_SIZE: usize = 16;

    /// Serializes this [`Spacetime`] snapshot into a fixed 16-byte buffer.
    ///
    /// All fields are stored as little-endian IEEE 754 `f64`.
    pub fn to_wire_bytes(&self) -> [u8; Self::WIRE_SIZE] {
        let mut buf = [0u8; Self::WIRE_SIZE];
        buf[0..8].copy_from_slice(&self.alpha.to_le_bytes());
        buf[8..16].copy_from_slice(&self.beta.to_le_bytes());
        buf
    }

    /// Deserializes a [`Spacetime`] from exactly 16 bytes.
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
        Some(Self { alpha, beta })
    }
}
