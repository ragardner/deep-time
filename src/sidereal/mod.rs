//! Sidereal rotation and time calculations for celestial bodies.
//!
//! [`Sidereal`] struct with ready-to-use `EARTH`, `MARS`, `MOON` constants.
//! Computes rotation angle, LMST/LAST, GMST/GAST.
//!
//! With the `"sidereal-earth"` feature enabled a rust implementation of the
//! ERFA Earth Equation of the Origins / Equinoxes are both available as well.

#[cfg(feature = "sidereal-earth")]
pub mod earth_eo_ee;

use crate::Real;
use core::f64::consts::TAU;

#[cfg(feature = "sidereal-earth")]
use earth_eo_ee::*;

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
/// ## Fields
///
/// * `rate_rad_per_sec` — Mean sidereal rotation rate in radians per SI second.
/// * `ref_epoch` — Reference epoch (MJD) at which `ref_angle_rad` is defined.
/// * `ref_angle_rad` — Rotation angle of the prime meridian at `ref_epoch`.
/// * `longitude_rad` — Observer longitude on the body (radians, east positive).
///   `0.0` corresponds to the body's prime meridian.
/// * `correction_rad` — General-purpose additive correction in radians.
///
/// ## Examples
///
/// Basic usage with Earth constants:
///
/// ```rust
/// use deep_time::Sidereal;
///
/// let mut earth = Sidereal::EARTH;
/// earth.longitude_rad = 0.0; // Greenwich
///
/// let mjd = 60000.0;
/// let era = earth.rotation_angle(mjd);
///
/// // Local Mean Sidereal Time using the mean Equation of the Origins
/// // (requires the "sidereal-earth" feature)
/// # #[cfg(feature = "sidereal-earth")] {
/// let eo_mean = earth.earth_eo_mean(mjd + 32.184 / 86400.0);
/// let lmst = earth.local_sidereal_time_mean(mjd, eo_mean);
/// # }
/// ```
///
/// Realistic usage with DUT1 correction (UT1 time scale):
///
/// ```rust
/// // This advanced example requires the "eop" feature for EopData
/// // and "sidereal-earth" for the EO calculations.
/// # #[cfg(all(feature = "eop", feature = "sidereal-earth"))] {
/// use deep_time::Dt;
/// use deep_time::Sidereal;
/// use deep_time::eop::{EopData, EopFormat, Separator};
///
/// let eop = EopData::from_text_file(
///     "finals.all.iau2000.txt",
///     EopFormat::Finals2000A,
///     Separator::Whitespace,
/// ).unwrap();
///
/// let mjd_utc = 56879.0;
/// let dut1 = Dt::mjd_to_eop_offset_f(mjd_utc, &eop).unwrap();
/// let mjd_ut1 = mjd_utc + dut1 / 86400.0;
///
/// let earth = Sidereal::EARTH;
///
/// let era = earth.rotation_angle(mjd_ut1);
///
/// let eo_mean = earth.earth_eo_mean(mjd_ut1 + 32.184 / 86400.0);
/// let gmst = earth.sidereal_angle_mean(mjd_ut1, eo_mean);
///
/// // Local Mean Sidereal Time
/// let lmst = earth.local_sidereal_time_mean(mjd_ut1, eo_mean);
/// # }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    /// General scalar correction in radians.
    pub correction_rad: Real,
}

impl Sidereal {
    /// Pre-configured `Sidereal` for Earth using IAU 2000/2006 conventions.
    ///
    /// This uses:
    /// - The conventional mean sidereal rotation rate of Earth.
    /// - J2000.0 as the reference epoch (`ref_epoch = 51544.5`).
    /// - The Earth Rotation Angle (ERA) at J2000.0 as `ref_angle_rad`.
    ///
    /// You can still customize fields after construction (e.g. `longitude_rad`
    /// or `correction_rad`).
    pub const EARTH: Self = Self {
        rate_rad_per_sec: (1.00273781191135448 * core::f64::consts::TAU) / 86400.0,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.7790572732640 * core::f64::consts::TAU,
        longitude_rad: 0.0,
        correction_rad: 0.0,
    };

    /// Pre-configured `Sidereal` for Mars.
    ///
    /// Uses a simplified mean sidereal rotation rate and J2000.0 as the
    /// reference epoch. `ref_angle_rad` is set to zero (no specific
    /// reference angle is defined).
    ///
    /// You can customize fields (especially `longitude_rad`) after construction.
    pub const MARS: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 88642.663,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
        correction_rad: 0.0,
    };

    /// Pre-configured `Sidereal` for the Moon.
    ///
    /// Uses a simplified mean sidereal rotation rate and J2000.0 as the
    /// reference epoch. `ref_angle_rad` is set to zero (no specific
    /// reference angle is defined).
    ///
    /// You can customize fields (especially `longitude_rad`) after construction.
    pub const MOON: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 2_360_591.424,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
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
    /// ## Examples
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

        let angle = self.ref_angle_rad + self.rate_rad_per_sec * elapsed_sec + self.correction_rad;

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
    /// ## Examples
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
    /// This computes Greenwich Mean Sidereal Time (GMST) when an appropriate
    /// Equation of the Origins value is supplied.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The Equation of the Origins value to subtract from the
    ///   Earth Rotation Angle (ERA).  
    ///   - Pass `0.0` to get the pure CIO-based rotation angle (ERA).
    ///   - Pass the **mean** Equation of the Origins (e.g. from
    ///     [`earth_eo_mean`](Self::earth_eo_mean)) to obtain GMST.
    ///
    /// ## Details
    ///
    /// - When `eo_rad = 0.0`, the result is the modern Earth Rotation Angle (ERA)
    ///   relative to the Celestial Intermediate Origin (CIO).
    ///
    /// - When `eo_rad` is the mean Equation of the Origins (i.e. the value that
    ///   satisfies `GMST = ERA − eo_rad`), the result is Greenwich Mean Sidereal
    ///   Time (GMST) referred to the mean equinox. This is the traditional
    ///   equinox-based mean sidereal time.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    ///
    /// // Pure CIO-based rotation angle (Earth Rotation Angle)
    /// let era = earth.sidereal_angle_mean(mjd, 0.0);
    ///
    /// // Traditional mean sidereal time using the mean Equation of the Origins
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_mean = earth.earth_eo_mean(mjd + 32.184 / 86400.0);
    /// let gmst = earth.sidereal_angle_mean(mjd, eo_mean);
    /// # }
    /// ```
    #[inline]
    pub const fn sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) - eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns the local sidereal angle at the observer's longitude in radians,
    /// normalized to `[0, 2π)`.
    ///
    /// This computes **Local Mean Sidereal Time (LMST)** when an appropriate
    /// Equation of the Origins value is supplied.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The Equation of the Origins value to subtract from the
    ///   Earth Rotation Angle (ERA).  
    ///   - Pass `0.0` to get the pure local Earth Rotation Angle (CIO-based).
    ///   - Pass the **mean** Equation of the Origins (e.g. from
    ///     [`earth_eo_mean`](Self::earth_eo_mean)) to obtain Local Mean
    ///     Sidereal Time (LMST).
    ///
    /// ## Details
    ///
    /// - When `eo_rad = 0.0`, the result is the local Earth Rotation Angle
    ///   relative to the Celestial Intermediate Origin (CIO) at the observer’s
    ///   longitude.
    ///
    /// - When `eo_rad` is the mean Equation of the Origins, the result is
    ///   **Local Mean Sidereal Time (LMST)** referred to the mean equinox.
    ///
    /// This value is commonly used when calculating the local hour angle of a
    /// celestial object:
    ///
    /// ```text
    /// HA = local_sidereal_angle_mean(mjd, eo) − RA
    /// ```
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    ///
    /// // Pure local Earth Rotation Angle (CIO-based)
    /// let local_era = earth.local_sidereal_angle_mean(mjd, 0.0);
    ///
    /// // Local Mean Sidereal Time using the mean Equation of the Origins
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_mean = earth.earth_eo_mean(mjd + 32.184 / 86400.0);
    /// let lmst = earth.local_sidereal_angle_mean(mjd, eo_mean);
    /// # }
    /// ```
    #[inline]
    pub const fn local_sidereal_angle_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + self.longitude_rad - eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns sidereal time at the body's prime meridian as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of
    /// [`Sidereal::sidereal_angle_mean`].
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The Equation of the Origins value to use.  
    ///   - Pass `0.0` to get the time equivalent of the pure Earth Rotation Angle (ERA).  
    ///   - Pass the **mean** Equation of the Origins (e.g. from
    ///     [`earth_eo_mean`](Self::earth_eo_mean)) to obtain Greenwich Mean
    ///     Sidereal Time (GMST).
    ///
    /// ## Details
    ///
    /// - When `eo_rad = 0.0`, the result is the time equivalent of the modern
    ///   Earth Rotation Angle (ERA).
    ///
    /// - When `eo_rad` is the mean Equation of the Origins, the result is
    ///   **Greenwich Mean Sidereal Time (GMST)** referred to the mean equinox.
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("mean").to_value("sec")` (when no longitude is
    /// specified) when using matching UT1 time and the mean Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    ///
    /// // Time equivalent of pure Earth Rotation Angle
    /// let era_seconds = earth.sidereal_time_mean(mjd, 0.0);
    ///
    /// // Greenwich Mean Sidereal Time in seconds
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_mean = earth.earth_eo_mean(mjd + 32.184 / 86400.0);
    /// let gmst_seconds = earth.sidereal_time_mean(mjd, eo_mean);
    /// # }
    /// ```
    pub const fn sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns local sidereal time at the observer's longitude as seconds since
    /// sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of
    /// [`Sidereal::local_sidereal_angle_mean`].
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The Equation of the Origins value to use.  
    ///   - Pass `0.0` to get the time equivalent of the local Earth Rotation Angle (CIO-based).  
    ///   - Pass the **mean** Equation of the Origins (e.g. from
    ///     [`earth_eo_mean`](Self::earth_eo_mean)) to obtain **Local Mean Sidereal Time (LMST)**.
    ///
    /// ## Details
    ///
    /// - When `eo_rad = 0.0`, the result is the time equivalent of the local
    ///   Earth Rotation Angle relative to the Celestial Intermediate Origin (CIO)
    ///   at the observer’s longitude.
    ///
    /// - When `eo_rad` is the mean Equation of the Origins, the result is
    ///   **Local Mean Sidereal Time (LMST)** referred to the mean equinox.
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("mean", longitude=...).to_value("sec")` when using
    /// matching UT1 time and the mean Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    ///
    /// // Time equivalent of local Earth Rotation Angle
    /// let local_era_seconds = earth.local_sidereal_time_mean(mjd, 0.0);
    ///
    /// // Local Mean Sidereal Time in seconds
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_mean = earth.earth_eo_mean(mjd + 32.184 / 86400.0);
    /// let lmst_seconds = earth.local_sidereal_time_mean(mjd, eo_mean);
    /// # }
    /// ```
    pub const fn local_sidereal_time_mean(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.local_sidereal_angle_mean(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns the apparent sidereal angle of the body's prime meridian
    /// in radians, normalized to `[0, 2π)`.
    ///
    /// This computes **Greenwich Apparent Sidereal Time (GAST)** when the
    /// apparent Equation of the Origins is supplied.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The **apparent** Equation of the Origins
    ///   (e.g. from [`earth_eo_apparent`](Self::earth_eo_apparent)).
    ///   When supplied, the result is Greenwich Apparent Sidereal Time (GAST)
    ///   referred to the true equinox.
    ///
    /// ## Details
    ///
    /// This function implements the direct relationship:
    ///
    /// ```text
    /// GAST = ERA − EO_apparent
    /// ```
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent").rad` (when no longitude is specified)
    /// when using matching UT1 time and the apparent Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    ///
    /// // Greenwich Apparent Sidereal Time
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_app = earth.earth_eo_apparent(mjd + 32.184 / 86400.0);
    /// let gast = earth.sidereal_angle_apparent(mjd, eo_app);
    /// # }
    /// ```
    pub const fn sidereal_angle_apparent(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) - eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns the local apparent sidereal angle at the observer's longitude
    /// in radians, normalized to `[0, 2π)`.
    ///
    /// This computes **Local Apparent Sidereal Time (LAST)** when the
    /// apparent Equation of the Origins is supplied.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The **apparent** Equation of the Origins
    ///   (e.g. from [`earth_eo_apparent`](Self::earth_eo_apparent)).
    ///   When supplied, the result is Local Apparent Sidereal Time (LAST)
    ///   at the observer’s longitude, referred to the true equinox.
    ///
    /// ## Details
    ///
    /// This function implements the direct relationship:
    ///
    /// ```text
    /// LAST = ERA + longitude − EO_apparent
    /// ```
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent", longitude=...).rad` when using
    /// matching UT1 time and the apparent Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    ///
    /// // Local Apparent Sidereal Time
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_app = earth.earth_eo_apparent(mjd + 32.184 / 86400.0);
    /// let last = earth.local_sidereal_angle_apparent(mjd, eo_app);
    /// # }
    /// ```
    pub const fn local_sidereal_angle_apparent(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.rotation_angle(mjd) + self.longitude_rad - eo_rad;
        Self::normalize_angle(angle)
    }

    /// Returns apparent sidereal time at the body's prime meridian as seconds
    /// since sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of
    /// [`Sidereal::sidereal_angle_apparent`].
    ///
    /// When the **apparent** Equation of the Origins is supplied, this function
    /// returns **Greenwich Apparent Sidereal Time (GAST)**.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The **apparent** Equation of the Origins
    ///   (e.g. from [`earth_eo_apparent`](Self::earth_eo_apparent)).
    ///   When supplied, the result is Greenwich Apparent Sidereal Time (GAST)
    ///   in seconds since sidereal midnight.
    ///
    /// ## Details
    ///
    /// This function computes:
    ///
    /// ```text
    /// GAST (seconds) = (ERA − EO_apparent) in fractional days × 86400
    /// ```
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent").to_value("sec")` (Greenwich) when using
    /// matching UT1 time and the apparent Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd = 60000.0;
    ///
    /// // Greenwich Apparent Sidereal Time in seconds
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_app = earth.earth_eo_apparent(mjd + 32.184 / 86400.0);
    /// let gast_seconds = earth.sidereal_time_apparent(mjd, eo_app);
    /// # }
    /// ```
    pub const fn sidereal_time_apparent(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.sidereal_angle_apparent(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns local apparent sidereal time at the observer's longitude as
    /// seconds since sidereal midnight, wrapped to the range `[0, 86400)`.
    ///
    /// This is the time equivalent of
    /// [`Sidereal::local_sidereal_angle_apparent`].
    ///
    /// When the **apparent** Equation of the Origins is supplied, this function
    /// returns **Local Apparent Sidereal Time (LAST)**.
    ///
    /// ## Parameters
    ///
    /// - `eo_rad`: The **apparent** Equation of the Origins
    ///   (e.g. from [`earth_eo_apparent`](Self::earth_eo_apparent)).
    ///   When supplied, the result is Local Apparent Sidereal Time (LAST)
    ///   at the observer’s longitude, in seconds since sidereal midnight.
    ///
    /// ## Details
    ///
    /// This function computes:
    ///
    /// ```text
    /// LAST (seconds) = (ERA + longitude − EO_apparent) in fractional days × 86400
    /// ```
    ///
    /// As of Astropy 7.x, this is consistent with
    /// `Time.sidereal_time("apparent", longitude=...).to_value("sec")` when using
    /// matching UT1 time and the apparent Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    ///
    /// let mjd = 60000.0;
    ///
    /// // Local Apparent Sidereal Time in seconds
    /// // (requires "sidereal-earth" feature)
    /// # #[cfg(feature = "sidereal-earth")] {
    /// let eo_app = earth.earth_eo_apparent(mjd + 32.184 / 86400.0);
    /// let last_seconds = earth.local_sidereal_time_apparent(mjd, eo_app);
    /// # }
    /// ```
    pub const fn local_sidereal_time_apparent(&self, mjd: Real, eo_rad: Real) -> Real {
        let angle = self.local_sidereal_angle_apparent(mjd, eo_rad);
        let fraction = ((angle / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns the apparent Equation of the Origins (radians) at the given MJD.
    ///
    /// This returns the value computed by ERFA’s `eo06a`. It is the modern
    /// CIO-based quantity used to derive **Greenwich Apparent Sidereal Time (GAST)**
    /// from the Earth Rotation Angle (ERA).
    ///
    /// When you subtract this value from the ERA, you get GAST:
    ///
    /// ```text
    /// GAST = ERA − earth_eo_apparent(...)
    /// ```
    ///
    /// This method is equivalent to calling `erfa.eo06a(tt.jd1, tt.jd2)` in Astropy.
    ///
    /// You should pass the value returned by this function to the apparent
    /// sidereal time functions (`sidereal_angle_apparent`, `local_sidereal_angle_apparent`,
    /// `sidereal_time_apparent`, and `local_sidereal_time_apparent`).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd_tt = 60000.0 + 32.184 / 86400.0;
    ///
    /// let eo_app = earth.earth_eo_apparent(mjd_tt);
    /// let gast = earth.sidereal_angle_apparent(mjd_tt, eo_app);
    /// ```
    #[cfg(feature = "sidereal-earth")]
    #[inline]
    pub const fn earth_eo_apparent(&self, tt_mjd: Real) -> Real {
        // Convert MJD → two-part Julian Date
        let date1 = 2400000.5 + tt_mjd;
        earth_eo(date1, 0.0)
    }

    /// Returns the mean Equation of the Origins (radians) at the given MJD.
    ///
    /// This returns the value that should be subtracted from the Earth Rotation
    /// Angle (ERA) to obtain **Greenwich Mean Sidereal Time (GMST)**:
    ///
    /// ```text
    /// GMST = ERA − earth_eo_mean(...)
    /// ```
    ///
    /// Internally, this is computed as:
    ///
    /// ```text
    /// earth_eo_mean = earth_eo_apparent() + earth_ee()
    /// ```
    ///
    /// This is equivalent to computing `era - gmst` in Astropy:
    ///
    /// ```python
    /// era = ut1.earth_rotation_angle(...).rad
    /// gmst = ut1.sidereal_time("mean", ...).rad
    /// eo_mean = era - gmst
    /// ```
    ///
    /// You should pass the value returned by this function to the mean
    /// sidereal time functions (`sidereal_angle_mean`, `local_sidereal_angle_mean`,
    /// `sidereal_time_mean`, and `local_sidereal_time_mean`).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd_tt = 60000.0 + 32.184 / 86400.0;
    ///
    /// let eo_mean = earth.earth_eo_mean(mjd_tt);
    /// let gmst = earth.sidereal_angle_mean(mjd_tt, eo_mean);
    /// ```
    #[cfg(feature = "sidereal-earth")]
    #[inline]
    pub const fn earth_eo_mean(&self, tt_mjd: Real) -> Real {
        // Convert MJD → two-part Julian Date
        let date1 = 2400000.5 + tt_mjd;
        earth_eo(date1, 0.0) + earth_ee(date1, 0.0)
    }

    /// Returns the Equation of the Equinoxes (radians) at the given MJD.
    ///
    /// This returns the value computed by ERFA’s `ee06a`. The Equation of the
    /// Equinoxes represents the nutation contribution to sidereal time and is
    /// defined as:
    ///
    /// ```text
    /// EE = GAST − GMST
    /// ```
    ///
    /// It is equivalent to computing `gast - gmst` in Astropy:
    ///
    /// ```python
    /// gast = ut1.sidereal_time("apparent", ...).rad
    /// gmst = ut1.sidereal_time("mean", ...).rad
    /// ee = gast - gmst
    /// ```
    ///
    /// This value is used internally when converting between mean and apparent
    /// sidereal time (for example, when the mean functions are given the apparent
    /// EO + EE).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::Sidereal;
    ///
    /// let earth = Sidereal::EARTH;
    /// let mjd_tt = 60000.0 + 32.184 / 86400.0;
    ///
    /// let ee = earth.earth_ee(mjd_tt);
    /// ```
    #[cfg(feature = "sidereal-earth")]
    #[inline]
    pub const fn earth_ee(&self, tt_mjd: Real) -> Real {
        // Convert MJD → two-part Julian Date
        let date1 = 2400000.5 + tt_mjd;
        earth_ee(date1, 0.0)
    }
}
