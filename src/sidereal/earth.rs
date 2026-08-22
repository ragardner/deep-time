//! Earth equinox sidereal time (IAU 2000/2006).
//!
//! [`Sidereal`](../struct.Sidereal.html) is a prime-meridian clock. On Earth that clock is the Earth
//! Rotation Angle (ERA, CIO origin). This module adds the IAU equinox readout
//! of that clock as methods on [`Sidereal`](../struct.Sidereal.html):
//!
//! ```text
//! GAST = ERA − eo06a(TT)                 // true equinox
//! GMST = ERA − eo06a(TT) − ee06a(TT)     // mean equinox
//! LAST = GAST + λ                        // east longitude, radians
//! LMST = GMST + λ
//! ```
//!
//! ERA / GMST / GAST take **UT1** MJD. The Equation of the Origins (`eo`) and
//! Equation of the Equinoxes (`ee`) take **TT** MJD. These quantities are
//! Earth-only; they are not a generic mode of [`Sidereal`](../struct.Sidereal.html).
//!
//! With matching UT1 and TT, results agree with Astropy mean/apparent sidereal
//! time and ERFA `eo06a` / `ee06a` (see tests).

use super::earth_eo_ee::{earth_ee, earth_eo};
use super::{Sidereal, wrap_angle};
use crate::Real;

/// Earth equinox sidereal time.
///
/// Greenwich forms (`era`, `gmst`, `gast`, `eo`, `ee`) always use the IAU ERA
/// ([`Sidereal::EARTH`](../struct.Sidereal.html#associatedconstant.EARTH)), not this instance's `rate` or `correction_rad`.
/// Local forms (`lmst`, `last`) add this instance's [`Sidereal::longitude_rad`](../struct.Sidereal.html#structfield.longitude_rad)
/// (east positive).
impl Sidereal {
    /// Earth Rotation Angle at **UT1** MJD, radians in `[0, 2π)`.
    ///
    /// Same as [`Sidereal::EARTH`](../struct.Sidereal.html#associatedconstant.EARTH).[`Sidereal::rotation_angle`](../struct.Sidereal.html#method.rotation_angle).
    #[inline]
    pub const fn era(ut1_mjd: Real) -> Real {
        Self::EARTH.rotation_angle(ut1_mjd)
    }

    /// Equation of the Origins at **TT** MJD (`eo06a`): `EO = ERA − GAST`.
    #[inline]
    pub const fn eo(tt_mjd: Real) -> Real {
        earth_eo(2_400_000.5, tt_mjd)
    }

    /// Equation of the Equinoxes at **TT** MJD (`ee06a`): `EE = GAST − GMST`.
    #[inline]
    pub const fn ee(tt_mjd: Real) -> Real {
        earth_ee(2_400_000.5, tt_mjd)
    }

    /// Greenwich Mean Sidereal Time (radians): `ERA(UT1) − eo(TT) − ee(TT)`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std", feature = "sidereal-earth"))] {
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    /// use deep_time::{Dt, Scale, Sidereal};
    ///
    /// let eop = EopData::from_text_file(
    ///     "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
    ///     EopFormat::C04,
    ///     Separator::Whitespace,
    /// ).unwrap();
    /// let utc = Dt::from_mjd_f(56879.0, Scale::UTC);
    /// let ut1 = utc.to_ut1(&eop).unwrap().to_mjd_f_raw();
    /// let tt = utc.to(Scale::TT).to_mjd_f_raw();
    ///
    /// let gmst = Sidereal::gmst(ut1, tt);
    /// let _ = gmst;
    /// # }
    /// ```
    #[inline]
    pub const fn gmst(ut1_mjd: Real, tt_mjd: Real) -> Real {
        wrap_angle(Self::era(ut1_mjd) - Self::eo(tt_mjd) - Self::ee(tt_mjd))
    }

    /// Greenwich Apparent Sidereal Time (radians): `ERA(UT1) − eo(TT)`.
    #[inline]
    pub const fn gast(ut1_mjd: Real, tt_mjd: Real) -> Real {
        wrap_angle(Self::era(ut1_mjd) - Self::eo(tt_mjd))
    }

    /// Local Mean Sidereal Time (radians): `GMST + longitude_rad`.
    ///
    /// Always uses IAU Earth ERA
    /// ([`Sidereal::gmst`](../struct.Sidereal.html#method.gmst)). Only
    /// [`Sidereal::longitude_rad`](../struct.Sidereal.html#structfield.longitude_rad) (east positive) is taken from this instance.
    ///
    /// ```text
    /// HA = lmst(ut1, tt) − RA   // mean-equinox RA
    /// ```
    #[inline]
    pub const fn lmst(&self, ut1_mjd: Real, tt_mjd: Real) -> Real {
        wrap_angle(Self::gmst(ut1_mjd, tt_mjd) + self.longitude_rad)
    }

    /// Local Apparent Sidereal Time (radians): `GAST + longitude_rad`.
    ///
    /// Always uses IAU Earth ERA
    /// ([`Sidereal::gast`](../struct.Sidereal.html#method.gast)). Only
    /// [`Sidereal::longitude_rad`](../struct.Sidereal.html#structfield.longitude_rad) (east positive) is taken from this instance.
    ///
    /// ```text
    /// HA = last(ut1, tt) − RA   // true-equinox RA
    /// ```
    #[inline]
    pub const fn last(&self, ut1_mjd: Real, tt_mjd: Real) -> Real {
        wrap_angle(Self::gast(ut1_mjd, tt_mjd) + self.longitude_rad)
    }
}
