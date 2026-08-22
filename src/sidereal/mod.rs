//! Prime-meridian / spin-angle clocks, plus Earth equinox sidereal time.
//!
//! [`Sidereal`] is a planet-agnostic prime-meridian clock with presets
//! [`Sidereal::EARTH`], [`Sidereal::MARS`], and [`Sidereal::MOON`]. It
//! evaluates rotation angle and local meridian angle from a linear spin model.
//!
//! On Earth that angle is the Earth Rotation Angle (ERA, CIO origin). Hour
//! angle uses the same frame as the angle (`HA = local meridian − RA`).
//!
//! **Earth equinox sidereal time** (GMST, GAST, LMST, LAST) is not a generic
//! mode of this clock. It is an Earth-only readout of ERA via the IAU 2000/2006
//! Equation of the Origins / Equinoxes (`sidereal-earth`):
//! [`Sidereal::gmst`](struct.Sidereal.html#method.gmst),
//! [`Sidereal::gast`](struct.Sidereal.html#method.gast),
//! [`Sidereal::lmst`](struct.Sidereal.html#method.lmst),
//! [`Sidereal::last`](struct.Sidereal.html#method.last) (see [`earth`]).

/// ERFA Earth equation of the origins / equinoxes (`sidereal-earth` feature).
#[cfg(feature = "sidereal-earth")]
pub mod earth_eo_ee;

/// Earth equinox sidereal time: GMST, GAST, LMST, LAST (`sidereal-earth`).
#[cfg(feature = "sidereal-earth")]
pub mod earth;

use crate::Real;
use core::f64::consts::TAU;

/// Wrap an angle into `[0, 2π)`.
#[inline]
const fn wrap_angle(angle: Real) -> Real {
    ((angle % TAU) + TAU) % TAU
}

/// Prime-meridian / spin-angle clock for a rotating body.
///
/// The model is linear in time:
///
/// ```text
/// angle(t) = ref_angle + rate × (t − ref_epoch) + correction
/// ```
///
/// plus optional observer longitude for local meridian angle. For Earth that
/// is the Earth Rotation Angle (ERA). For other bodies it is only a simple
/// mean spin / meridian angle if you supply a rate — not a full orientation
/// ephemeris (the Moon’s librations, for example, are not included).
///
/// **Earth.** [`Sidereal::EARTH`] uses the IAU 2000 Earth Rotation Angle
/// relative to the Celestial Intermediate Origin (CIO). Equinox sidereal times
/// (`sidereal-earth`) are on this type:
/// [`Sidereal::gmst`](struct.Sidereal.html#method.gmst),
/// [`Sidereal::gast`](struct.Sidereal.html#method.gast),
/// [`Sidereal::lmst`](struct.Sidereal.html#method.lmst),
/// [`Sidereal::last`](struct.Sidereal.html#method.last).
///
/// **Other bodies.** Supply a published spin rate and reference angle (for
/// example IAU WGCCRE `Ẇ` / `W0`), or start from the simplified
/// [`Sidereal::MARS`] / [`Sidereal::MOON`] presets. Use
/// [`rotation_angle`](Self::rotation_angle) /
/// [`local_rotation_angle`](Self::local_rotation_angle).
///
/// Local meridian angle is the usual input to hour angle
/// (`HA = local meridian − RA`), meridian transit, and horizon ↔ equatorial
/// conversions. Meridian angle and `RA` must share the same equatorial frame
/// (CIO/CIRS with local ERA; mean or true equinox with LMST/LAST).
///
/// ## Fields
///
/// * `rate_rad_per_sec` — Sidereal rotation rate in radians per SI second.
/// * `ref_epoch` — Reference epoch as an MJD at which `ref_angle_rad` is defined.
///   For Earth ERA this is a **UT1** MJD.
/// * `ref_angle_rad` — Rotation angle of the prime meridian at `ref_epoch`.
/// * `longitude_rad` — Observer longitude on the body (radians, east positive).
///   `0.0` corresponds to the body's prime meridian.
/// * `correction_rad` — Optional additive angle (radians) folded into
///   [`rotation_angle`](Self::rotation_angle). Do **not** use this for DUT1;
///   put UT1 in the time argument instead.
///
/// ## Examples
///
/// Earth ERA from UTC via IERS C04 (needs `eop` and `std`). Equinox sidereal
/// time needs `sidereal-earth` as well — see
/// [`Sidereal::gmst`](struct.Sidereal.html#method.gmst).
///
/// ```rust
/// # #[cfg(all(feature = "eop", feature = "std"))] {
/// use deep_time::{Dt, Scale, Sidereal};
/// use deep_time::eop::{EopData, EopFormat, Separator};
///
/// let eop = EopData::from_text_file(
///     "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
///     EopFormat::C04,
///     Separator::Whitespace,
/// ).unwrap();
///
/// let utc = Dt::from_mjd_f(56879.0, Scale::UTC);
/// let mjd_ut1 = utc.to_ut1(&eop).unwrap().to_mjd_f_raw();
///
/// let mut earth = Sidereal::EARTH;
/// earth.longitude_rad = 0.0; // Greenwich
///
/// let era = earth.rotation_angle(mjd_ut1);
/// let local_era = earth.local_rotation_angle(mjd_ut1);
/// let _ = (era, local_era);
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Sidereal {
    /// Sidereal rotation rate in **radians per SI second**.
    pub rate_rad_per_sec: Real,
    /// Reference epoch as an MJD (UT1 for Earth ERA).
    pub ref_epoch: Real,
    /// Rotation angle of the prime meridian (radians) at `ref_epoch`.
    pub ref_angle_rad: Real,
    /// Longitude of the observer on the body (radians, east positive).
    /// `0.0` = body's prime meridian.
    pub longitude_rad: Real,
    /// Optional additive angle (radians) applied inside [`Self::rotation_angle`].
    /// Not a substitute for DUT1 — pass UT1 via the time argument instead.
    pub correction_rad: Real,
}

impl Sidereal {
    /// Pre-configured `Sidereal` for Earth using the IAU 2000 ERA.
    ///
    /// This uses:
    /// - The IAU 2000 Earth Rotation Angle rate
    ///   (`1.00273781191135448` turns per UT1 day).
    /// - J2000.0 as the reference epoch (`ref_epoch = 51544.5` UT1 MJD).
    /// - The Earth Rotation Angle (ERA) at J2000.0 as `ref_angle_rad`.
    ///
    /// You can still customize fields after construction (e.g. `longitude_rad`
    /// or `correction_rad`). For GMST/GAST/LMST/LAST see
    /// [`Sidereal::gmst`](struct.Sidereal.html#method.gmst)
    /// (`sidereal-earth`).
    pub const EARTH: Self = Self {
        rate_rad_per_sec: (1.00273781191135448 * core::f64::consts::TAU) / 86400.0,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.7790572732640 * core::f64::consts::TAU,
        longitude_rad: 0.0,
        correction_rad: 0.0,
    };

    /// Simplified `Sidereal` preset for Mars (mean spin rate only).
    ///
    /// Uses an approximate mean sidereal rotation period and J2000.0 as the
    /// reference epoch. `ref_angle_rad` is `0.0` (not a published prime-meridian
    /// offset). Suitable for demos and coarse meridian/spin geometry — not a
    /// full IAU WGCCRE orientation series. For higher fidelity, replace the
    /// rate and reference angle with published values.
    ///
    /// You can customize fields (especially `longitude_rad`) after construction.
    pub const MARS: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 88642.663,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
        correction_rad: 0.0,
    };

    /// Simplified `Sidereal` preset for the Moon (mean spin rate only).
    ///
    /// Uses an approximate mean sidereal rotation period and J2000.0 as the
    /// reference epoch. `ref_angle_rad` is `0.0` (not a published prime-meridian
    /// offset). Useful for coarse work; precise selenographic orientation needs
    /// lunar librations, which this preset does not include.
    ///
    /// You can customize fields (especially `longitude_rad`) after construction.
    pub const MOON: Self = Self {
        rate_rad_per_sec: core::f64::consts::TAU / 2_360_591.424,
        ref_epoch: 51544.5,
        ref_angle_rad: 0.0,
        longitude_rad: 0.0,
        correction_rad: 0.0,
    };

    /// Convert a meridian / sidereal angle in radians to seconds on a 24-hour
    /// sidereal clock, wrapped to `[0, 86400)`.
    ///
    /// This is `(angle / 2π) × 86400` — an hour-angle clock, not SI seconds of a
    /// sidereal day.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::f64::consts::PI;
    /// use deep_time::Sidereal;
    ///
    /// assert!((Sidereal::to_sec(PI) - 43_200.0).abs() < 1e-9);
    /// ```
    #[inline]
    pub const fn to_sec(angle_rad: Real) -> Real {
        let fraction = ((angle_rad / TAU) % 1.0 + 1.0) % 1.0;
        fraction * 86400.0
    }

    /// Returns the instantaneous rotation angle of the body's prime meridian
    /// (in radians) at the given instant, normalized to `[0, 2π)`.
    ///
    /// For Earth this is the IAU 2000 Earth Rotation Angle (ERA) relative to the
    /// Celestial Intermediate Origin (CIO) — the same definition as ERFA `era00`.
    /// `mjd` is **UT1** MJD for Earth ERA.
    /// It does **not** include observer longitude or the Equation of the Origins.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std"))] {
    /// use deep_time::{Dt, Scale, Sidereal};
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    ///
    /// let eop = EopData::from_text_file(
    ///     "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
    ///     EopFormat::C04,
    ///     Separator::Whitespace,
    /// ).unwrap();
    /// let utc = Dt::from_mjd_f(57753.5, Scale::UTC);
    /// let mjd_ut1 = utc.to_ut1(&eop).unwrap().to_mjd_f_raw();
    ///
    /// let era = Sidereal::EARTH.rotation_angle(mjd_ut1);
    /// let _ = era;
    /// # }
    /// ```
    pub const fn rotation_angle(&self, mjd: Real) -> Real {
        // elapsed time in seconds between ref_epoch (MJD) and the given mjd
        let elapsed_days = mjd - self.ref_epoch;
        let elapsed_sec = elapsed_days * 86400.0;

        let angle = self.ref_angle_rad + self.rate_rad_per_sec * elapsed_sec + self.correction_rad;

        wrap_angle(angle)
    }

    /// Returns the rotation angle of the prime meridian at the observer's
    /// longitude, normalized to `[0, 2π)`.
    ///
    /// This is equivalent to `rotation_angle(mjd) + self.longitude_rad`.
    /// For Earth with [`Sidereal::EARTH`], that is the local ERA: the angle
    /// between the Celestial Intermediate Origin (CIO) and the observer’s
    /// local meridian.
    ///
    /// Hour angle of a source:
    ///
    /// ```text
    /// HA = local_rotation_angle(mjd) − RA
    /// ```
    ///
    /// Use a right ascension in the **same** frame as this angle (CIO/CIRS RA
    /// with local ERA; equinox RA with LMST/LAST).
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std"))] {
    /// use deep_time::{Dt, Scale, Sidereal};
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    ///
    /// let eop = EopData::from_text_file(
    ///     "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
    ///     EopFormat::C04,
    ///     Separator::Whitespace,
    /// ).unwrap();
    /// let utc = Dt::from_mjd_f(56879.0, Scale::UTC);
    /// let mjd_ut1 = utc.to_ut1(&eop).unwrap().to_mjd_f_raw();
    ///
    /// let mut earth = Sidereal::EARTH;
    /// earth.longitude_rad = 0.0; // Greenwich
    /// let local_era = earth.local_rotation_angle(mjd_ut1);
    /// let _ = local_era;
    /// # }
    /// ```
    #[inline]
    pub const fn local_rotation_angle(&self, mjd: Real) -> Real {
        wrap_angle(self.rotation_angle(mjd) + self.longitude_rad)
    }
}
