use crate::Real;
use core::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct Sidereal {
    /// Mean sidereal rotation rate in **radians per SI second**.
    pub rate_rad_per_sec: Real,

    /// Reference epoch.
    pub ref_epoch: Real,

    /// Rotation angle of the prime meridian (radians) at `ref_epoch`.
    pub ref_angle_rad: Real,

    /// Longitude of the observer on the body (radians, east positive).
    /// `0.0` = body's prime meridian.
    pub longitude_rad: Real,

    /// Small correction to the pole position in the X direction (radians).
    pub pole_delta_x_rad: Real,

    /// Small correction to the pole position in the Y direction (radians).
    pub pole_delta_y_rad: Real,

    /// General scalar correction in radians.
    pub correction_rad: Real,
}

impl Sidereal {
    // ============================================================
    // EARTH
    // ============================================================
    pub const EARTH: Self = Self {
        rate_rad_per_sec: (1.00273781191135448 * core::f64::consts::TAU) / 86400.0,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.7790572732640 * core::f64::consts::TAU,
        longitude_rad: 0.0,
        pole_delta_x_rad: 0.0,
        pole_delta_y_rad: 0.0,
        correction_rad: 0.0,
    };

    // ============================================================
    // MARS
    // ============================================================
    pub const MARS: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 88642.663,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
        pole_delta_x_rad: 0.0,
        pole_delta_y_rad: 0.0,
        correction_rad: 0.0,
    };

    // ============================================================
    // MOON
    // ============================================================
    pub const MOON: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 2_360_591.424,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
        pole_delta_x_rad: 0.0,
        pole_delta_y_rad: 0.0,
        correction_rad: 0.0,
    };

    // Normalize to [0, 2π)
    #[inline]
    const fn normalize_angle(angle: Real) -> Real {
        ((angle % TAU) + TAU) % TAU
    }

    /// Returns the instantaneous rotation angle of the body's **prime meridian**
    /// (in radians) at the given instant, normalized to `[0, 2π)`.
    ///
    /// For Earth this is the **pure Earth Rotation Angle (ERA)** referred to the
    /// Celestial Intermediate Origin (CIO). It does **not** include observer
    /// longitude or the Equation of the Origins.
    ///
    /// Matches Astropy's `Time.earth_rotation_angle(longitude=None)` (or with
    /// `longitude=0`).
    pub fn rotation_angle(&self, mjd: Real) -> Real {
        // elapsed time in seconds between ref_epoch (MJD) and the given mjd
        let elapsed_days = mjd - self.ref_epoch;
        let elapsed_sec = elapsed_days * 86400.0;

        let angle = self.ref_angle_rad
            + self.rate_rad_per_sec * elapsed_sec
            + self.pole_delta_x_rad
            + self.pole_delta_y_rad
            + self.correction_rad;

        Self::normalize_angle(angle)
    }

    /// Returns the rotation angle of the prime meridian **at the observer's longitude**,
    /// normalized to `[0, 2π)`.
    ///
    /// Equivalent to `rotation_angle(mjd) + self.longitude_rad`.
    #[inline]
    pub fn local_rotation_angle(&self, mjd: Real) -> Real {
        Self::normalize_angle(self.rotation_angle(mjd) + self.longitude_rad)
    }

    /// Returns the sidereal angle of the body's **prime meridian** in radians,
    /// normalized to `[0, 2π)`.
    ///
    /// - If `eo_rad = 0.0`: this is identical to `rotation_angle` (pure ERA / CIO-based).
    /// - If `eo_rad` is the **Equation of the Origins** (angle between CIO and mean equinox):
    ///   this corresponds to **Greenwich Mean Sidereal Time (GMST)** referred to the equinox.
    ///
    /// This is the modern (CIO-based) equivalent of the traditional mean sidereal angle.
    #[inline]
    pub fn sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns the **local** sidereal angle at the observer's longitude in radians,
    /// normalized to `[0, 2π)`.
    ///
    /// - If `eo_rad = 0.0`: this is the local Earth Rotation Angle (CIO-based).
    /// - If `eo_rad` is the **Equation of the Origins**: this corresponds to
    ///   **Local Mean Sidereal Time (LMST)** referred to the equinox.
    #[inline]
    pub fn local_sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + self.longitude_rad + eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns sidereal time at the body's **prime meridian** as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// See [`sidereal_angle_mean`] for the exact meaning of the `eo_rad` parameter.
    pub fn sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns **local** sidereal time at the observer's longitude as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// See [`local_sidereal_angle_mean`] for the exact meaning of the `eo_rad` parameter.
    pub fn local_sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.local_sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }
}
