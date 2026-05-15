use crate::Real;
use core::f64::consts::TAU;

/// Represents the rotational state of a celestial body and provides
/// methods to compute the orientation of its prime meridian at any
/// given time.
///
/// The rotation angle of the prime meridian is the basis for
/// calculating local sidereal time. Local sidereal time is required
/// to compute the hour angle of a celestial object (HA = LST − RA),
/// to determine when an object will cross the local meridian,
/// to convert between horizon coordinates (altitude/azimuth) and
/// equatorial coordinates, and to calculate accurate pointing
/// directions for telescopes and spacecraft antennas.
///
/// The struct implements the modern CIO-based rotation model and
/// works for any rotating body (Earth, Mars, the Moon, etc.) by
/// supplying the appropriate rotation rate and reference values.
///
/// # Fields
///
/// * `rate_rad_per_sec` — Mean sidereal rotation rate in radians per SI second.
/// * `ref_epoch` — Reference epoch (MJD) at which `ref_angle_rad` is defined.
/// * `ref_angle_rad` — Rotation angle of the prime meridian at `ref_epoch`.
/// * `longitude_rad` — Observer longitude on the body (radians, east positive).
///   `0.0` corresponds to the body's prime meridian.
/// * `pole_delta_x_rad`, `pole_delta_y_rad` — Small corrections to the body's
///   pole position in the X and Y directions (radians).
/// * `correction_rad` — General-purpose additive correction in radians.
///
/// # Examples
///
/// Basic usage with Earth constants:
///
/// ```rust
/// use deep_time::Sidereal;
///
/// let mut earth = Sidereal::EARTH;
/// earth.longitude_rad = 0.0; // Greenwich meridian
///
/// let mjd = 60000.0;
/// let era = earth.rotation_angle(mjd);
///
/// // Local Mean Sidereal Time (requires Equation of the Origins)
/// let eo = 0.00326596;
/// let lmst = earth.local_sidereal_time_mean(mjd, eo);
/// ```
///
/// Realistic usage with DUT1 correction (UT1 time scale):
///
/// ```rust
/// use deep_time::{Dt, Sidereal};
/// use deep_time::bop::{BopData, BopFormat, Separator};
///
/// let bop = BopData::from_text_file(
///     "finals.all.iau2000.txt",
///     BopFormat::Finals2000A,
///     Separator::Whitespace,
/// ).unwrap();
///
/// let mjd_utc = 56879.0;
/// let dut1 = Dt::orientation_offset(mjd_utc, &bop).unwrap();
/// let mjd_ut1 = mjd_utc + dut1 / 86400.0;
///
/// let earth = Sidereal::EARTH;
///
/// let era = earth.rotation_angle(mjd_ut1);
/// let eo = 0.003265960688002;
///
/// let gmst = earth.sidereal_angle_mean(mjd_ut1, eo);
/// let lmst = earth.local_sidereal_time_mean(mjd_ut1, eo);
/// ```
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

    /// Returns the instantaneous rotation angle of the body's prime meridian
    /// (in radians) at the given instant, normalized to `[0, 2π)`.
    ///
    /// For Earth this is the pure Earth Rotation Angle (ERA) in the
    /// Celestial Intermediate Origin (CIO) frame. It does **not** include
    /// observer longitude or the Equation of the Origins.
    ///
    /// Matches Astropy's `Time.earth_rotation_angle(longitude=None)`
    /// (or with `longitude=0`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let era = Sidereal::EARTH.rotation_angle(57753.5);
    /// ```
    pub const fn rotation_angle(&self, mjd: Real) -> Real {
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

    /// Returns the rotation angle of the prime meridian at the observer's
    /// longitude, normalized to `[0, 2π)`.
    ///
    /// This is equivalent to `rotation_angle(mjd) + self.longitude_rad`.
    /// It gives the angle between the Celestial Intermediate Origin (CIO)
    /// and the observer’s local meridian.
    ///
    /// This value is commonly used when computing the local hour angle
    /// of a celestial object:
    ///
    /// ```text
    /// HA = local_rotation_angle(mjd) - RA
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    /// let local_era = earth.local_rotation_angle(mjd);
    /// ```
    #[inline]
    pub const fn local_rotation_angle(&self, mjd: Real) -> Real {
        Self::normalize_angle(self.rotation_angle(mjd) + self.longitude_rad)
    }

    /// Returns the sidereal angle of the body's prime meridian in radians,
    /// normalized to `[0, 2π)`.
    ///
    /// This method supports two common modes of use:
    ///
    /// - When `eo_rad = 0.0`, the result is the pure rotation angle of the
    ///   prime meridian relative to the Celestial Intermediate Origin (CIO).
    ///   This matches the modern Earth Rotation Angle (ERA) definition.
    ///
    /// - When `eo_rad` contains the Equation of the Origins, the result
    ///   corresponds to Greenwich Mean Sidereal Time (GMST) measured from
    ///   the mean equinox. This is the traditional form of mean sidereal
    ///   time still widely used in astronomy.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    ///
    /// // Pure CIO-based rotation angle
    /// let era = earth.sidereal_angle_mean(mjd, 0.0);
    ///
    /// // Traditional mean sidereal time (requires Equation of the Origins)
    /// let eo = 0.00326596;
    /// let gmst = earth.sidereal_angle_mean(mjd, eo);
    /// ```
    #[inline]
    pub const fn sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns the local sidereal angle at the observer's longitude in radians,
    /// normalized to `[0, 2π)`.
    ///
    /// This is the sidereal angle measured relative to the observer’s own
    /// meridian (rather than the prime meridian).
    ///
    /// - When `eo_rad = 0.0`, the result is the local Earth Rotation Angle
    ///   (CIO-based) at the observer’s longitude.
    /// - When `eo_rad` contains the Equation of the Origins, the result
    ///   corresponds to Local Mean Sidereal Time (LMST) referred to the
    ///   equinox.
    ///
    /// This is the value normally used when calculating the local hour
    /// angle of a celestial object for a specific observing site:
    ///
    /// ```text
    /// HA = local_sidereal_angle_mean(mjd, eo) - RA
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    /// let eo = 0.00326596;
    ///
    /// let local_era = earth.local_sidereal_angle_mean(mjd, 0.0);
    /// let lmst = earth.local_sidereal_angle_mean(mjd, eo);
    /// ```
    #[inline]
    pub const fn local_sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + self.longitude_rad + eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns sidereal time at the body's prime meridian as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of [`sidereal_angle_mean`]. The meaning
    /// of the `eo_rad` parameter is the same:
    ///
    /// - If `eo_rad = 0.0`, the result corresponds to the time equivalent
    ///   of the pure Earth Rotation Angle (ERA).
    /// - If `eo_rad` is the Equation of the Origins, the result corresponds
    ///   to Greenwich Mean Sidereal Time (GMST) referred to the mean equinox.
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("mean").to_value("sec")` (Greenwich Mean Sidereal
    /// Time, i.e. when no longitude is specified) when using matching UT1
    /// time and Equation of the Origins.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    /// let eo = 0.00326596;
    ///
    /// let gmst_seconds = earth.sidereal_time_mean(mjd, eo);
    /// ```
    pub const fn sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns local sidereal time at the observer's longitude as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of [`local_sidereal_angle_mean`]. The meaning
    /// of the `eo_rad` parameter is the same:
    ///
    /// - If `eo_rad = 0.0`, the result is the time equivalent of the local Earth
    ///   Rotation Angle (CIO-based) at the observer’s longitude.
    /// - If `eo_rad` is the Equation of the Origins, the result corresponds to
    ///   Local Mean Sidereal Time (LMST) referred to the equinox.
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("mean", longitude=...).to_value("sec")` when using
    /// matching UT1 time and Equation of the Origins.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    /// let eo = 0.00326596;
    ///
    /// let lmst_seconds = earth.local_sidereal_time_mean(mjd, eo);
    /// ```
    pub const fn local_sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.local_sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns the apparent sidereal angle of the body's prime meridian
    /// in radians, normalized to `[0, 2π)`.
    ///
    /// This is equivalent to `rotation_angle(mjd) + eo_rad + ee_rad`.
    ///
    /// - `eo_rad` = Equation of the Origins
    /// - `ee_rad` = Equation of the Equinoxes (nutation correction)
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent").rad` (when no longitude is specified).
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    /// let eo = 0.00326596; // Equation of the Origins
    /// let ee = 0.00000436; // Equation of the Equinoxes (example value)
    ///
    /// let gast = earth.sidereal_angle_apparent(mjd, eo, ee);
    /// ```
    pub fn sidereal_angle_apparent(&self, mjd: Real, eo_rad: Real, ee_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + eo_rad + ee_rad;
        Self::normalize_angle(angle)
    }

    /// Returns the local apparent sidereal angle at the observer's longitude
    /// in radians, normalized to `[0, 2π)`.
    ///
    /// This is equivalent to `rotation_angle(mjd) + self.longitude_rad + eo_rad + ee_rad`.
    ///
    /// - `eo_rad` = Equation of the Origins
    /// - `ee_rad` = Equation of the Equinoxes (nutation correction)
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent", longitude=...).rad`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    /// let eo = 0.00326596;
    /// let ee = 0.00000436;
    ///
    /// let last = earth.local_sidereal_angle_apparent(mjd, eo, ee);
    /// ```
    pub fn local_sidereal_angle_apparent(&self, mjd: Real, eo_rad: Real, ee_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + self.longitude_rad + eo_rad + ee_rad;
        Self::normalize_angle(angle)
    }

    /// Returns apparent sidereal time at the body's prime meridian as seconds
    /// since sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of [`sidereal_angle_apparent`].
    ///
    /// - `eo_rad` = Equation of the Origins
    /// - `ee_rad` = Equation of the Equinoxes
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent").to_value("sec")` (Greenwich).
    pub fn sidereal_time_apparent(&self, mjd: Real, eo_rad: Real, ee_rad: Real) -> Real {
        let angle = self.sidereal_angle_apparent(mjd, eo_rad, ee_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns local apparent sidereal time at the observer's longitude as
    /// seconds since sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of [`local_sidereal_angle_apparent`].
    ///
    /// - `eo_rad` = Equation of the Origins
    /// - `ee_rad` = Equation of the Equinoxes
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent", longitude=...).to_value("sec")`.
    pub fn local_sidereal_time_apparent(&self, mjd: Real, eo_rad: Real, ee_rad: Real) -> Real {
        let angle = self.local_sidereal_angle_apparent(mjd, eo_rad, ee_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }
}
