//! Mars time-scale constants and conversion methods (MSD, MTC, Ls, LMST, LTST, Mars Year).

use crate::{Dt, Real, Scale, cos, dt, floor_f, rem_euclid_f, sin};

/// mean length of one Martian sol in Earth seconds.
/// Current NASA GISS Mars24 value (updated 2025-01-07): 1.0274912517 Earth days.
pub const MARS_SOL_LENGTH_SEC: Real = 88_775.244_146_88;

/// Martian mean sol length in attoseconds
/// (88_775.24414688 s × 10¹⁸, integer matching the current NASA divisor).
pub const MARS_SOL_ATTOS: i128 = 88_775_244_146_880_000_000_000;

/// Precomputed numerical value of the Mars reference epoch on the TT scale (total attoseconds since J2000).
pub const MARS_REF_TT_ATTOS: i128 = -3_976_386_951_349_440_000_000_000_000;
pub const MARS_REF_TT: Dt = Dt::new(MARS_REF_TT_ATTOS, Scale::TT, Scale::TT);

/// Areocentric solar longitude (Ls) constants from the current NASA GISS Mars24
/// algorithm (AM2000 short series, updated 2025-01-07).
///
/// Ls = 0°   → northern vernal equinox (Martian northern spring begins)
/// Ls = 90°  → northern summer solstice
/// Ls = 180° → northern autumnal equinox
/// Ls = 270° → northern winter solstice
pub const MARS_LS_M0: Real = f!(19.3871);
pub const MARS_LS_M_RATE: Real = f!(0.52402073);
pub const MARS_LS_ALPHA_FMS0: Real = f!(270.3871);
pub const MARS_LS_ALPHA_FMS_RATE: Real = f!(0.524038496);

/// Equation-of-Time coefficients for LTST (from NASA GISS Mars24 / AM2000).
pub const MARS_EOT_COEFF_2LS: Real = f!(2.861);
pub const MARS_EOT_COEFF_4LS: Real = f!(-0.071);
pub const MARS_EOT_COEFF_6LS: Real = f!(0.002);

/// Mars Year epoch: JD 2435208.456 TT (northern vernal equinox Ls = 0° on 1955 April 11).
///
/// This is the Clancy et al. (2000) definition used by NASA, ESA, LMD Mars Climate
/// Database, and every modern Mars mission paper as of 2026.
pub const MARS_YEAR_EPOCH_JD: Real = f!(2435208.456);

/// Length of one Mars tropical year in Earth days (NASA GISS Mars24, 2025).
///
/// This is the interval between successive northern vernal equinoxes.
pub const MARS_TROPICAL_YEAR_DAYS: Real = f!(686.9725);

impl Dt {
    /// helper: elapsed attoseconds since the Mars MSD reference epoch (JD 2405522.0028779 TT).
    #[inline(always)]
    pub(crate) const fn to_attos_since_mars_msd_epoch(numerical_tt: Dt) -> i128 {
        numerical_tt.to_attos() - MARS_REF_TT_ATTOS
    }

    /// Returns the Mars Sol Date (MSD) as a tuple of integer sols and the fractional part of a sol.
    ///
    /// - This [`Dt`](../struct.Dt.html) is first converted to the TT scale before a result is produced.
    /// - Follows the canonical NASA GISS / AM2000 formulation.
    ///
    /// The fractional part is pushing the total towards the positive, and the whole part is floored.
    /// For more information on this return value see
    /// [`Dt::to_sec_and_ufrac`](#method.to_sec_and_ufrac).
    pub const fn to_msd(&self) -> (i128, i128) {
        let tt = self.to(Scale::TT);
        let elapsed = Self::to_attos_since_mars_msd_epoch(tt);
        let whole_sols = elapsed.div_euclid(MARS_SOL_ATTOS);
        let frac_attos = elapsed.rem_euclid(MARS_SOL_ATTOS);

        (whole_sols, frac_attos)
    }

    /// Returns Mars Coordinated Time (MTC) as a [`Dt`] representing
    /// seconds into the current sol (range `[0, one Martian sol)`).
    #[inline]
    pub const fn to_mtc(&self) -> Dt {
        let (_, frac_attos) = self.to_msd();
        dt!(frac_attos)
    }

    /// Creates a [`Dt`] (in TT) from an Mars Sol Date.
    pub const fn from_msd(whole_sols: i128, frac_attos: i128) -> Dt {
        let elapsed_attos = whole_sols
            .saturating_mul(MARS_SOL_ATTOS)
            .saturating_add(frac_attos);
        let tt = MARS_REF_TT.add(dt!(elapsed_attos));
        tt.convert(Scale::TAI)
    }

    /// Creates a [`Dt`] (in TT) from a floating-point Mars Sol Date.
    /// Non-exact Real.
    pub const fn from_msd_f(msd: Real) -> Dt {
        let whole = floor_f(msd) as i128;
        let frac = msd - f!(whole);
        let frac_span = Dt::from_sec_f(frac * MARS_SOL_LENGTH_SEC, Scale::TAI, Scale::TAI);
        Self::from_msd(whole, frac_span.to_attos())
    }

    /// Returns the Mars Sol Date (MSD) as a floating-point value (matches NASA Mars24 output).
    /// Non-exact Real.
    #[inline]
    pub const fn to_msd_f(&self) -> Real {
        let (whole, frac) = self.to_msd();
        f!(whole) + Dt::attos_to_sec_f(frac) / MARS_SOL_LENGTH_SEC
    }

    /// Returns the Areocentric Solar Longitude `Ls` in degrees (range [0, 360)).
    ///
    /// Ls is the angular position of the Sun as measured eastward from the Martian
    /// vernal equinox in Mars's orbital plane. It is the standard index of Martian
    /// seasonal progression used in all mission planning, science operations, and
    /// atmospheric modeling. Due to orbital eccentricity, northern spring + summer
    /// last ~381 Earth days while autumn + winter last ~306 Earth days.
    ///
    /// - Ls = 0°   → northern vernal equinox (northern spring begins)
    /// - Ls = 90°  → northern summer solstice
    /// - Ls = 180° → northern autumnal equinox
    /// - Ls = 270° → northern winter solstice
    ///
    /// Reproduces the short-series analytic model (B-1 through B-5) used
    /// by the current NASA GISS Mars24 Sunclock algorithm, which is based on
    /// Allison & McEwen (2000) with the seven largest planetary perturbations.
    ///
    /// Source: NASA Goddard Institute for Space Studies (GISS)  
    /// Title:   Mars24 Sunclock — Algorithm and Worked Examples  
    /// URL:     <https://www.giss.nasa.gov/tools/mars24/help/algorithm.html>
    /// Updated: 2025-01-07
    ///
    /// Works for any input [`Scale`] because it internally converts to TT.
    pub const fn to_mars_ls(&self) -> Real {
        // Δt_J2000 = days since J2000.0 TT
        let jd_tt = self.to(Scale::TT).to_jd_f_raw();
        let dt_j2000 = jd_tt - f!(2451545.0);

        // B-1: Mean anomaly M (degrees)
        let m = MARS_LS_M0 + MARS_LS_M_RATE * dt_j2000;

        // B-2: Right ascension of the Fictitious Mean Sun
        let alpha_fms = MARS_LS_ALPHA_FMS0 + MARS_LS_ALPHA_FMS_RATE * dt_j2000;

        // B-3: Planetary perturbation sum (PBS)
        let pbs = Self::mars_perturber_sum(dt_j2000);

        // B-4: Equation of Center (ν − M) in degrees
        let eq_center = (f!(10.691) + f!(3.0e-7) * dt_j2000) * sin(m.to_radians())
            + f!(0.623) * sin((f!(2.0) * m).to_radians())
            + f!(0.050) * sin((f!(3.0) * m).to_radians())
            + f!(0.005) * sin((f!(4.0) * m).to_radians())
            + f!(0.0005) * sin((f!(5.0) * m).to_radians())
            + pbs;

        // B-5: Areocentric solar longitude
        let mut ls = alpha_fms + eq_center;

        // Normalize to [0, 360)
        ls = ls % f!(360.0);
        if ls < f!(0.0) {
            ls += f!(360.0);
        }
        ls
    }

    const fn mars_perturber_sum(dt_j2000: Real) -> Real {
        let base = f!(0.985626) * dt_j2000;

        let mut sum = f!(0.0);

        sum += f!(0.0071) * cos(base / f!(2.2353) + f!(49.409));
        sum += f!(0.0057) * cos(base / f!(2.7543) + f!(168.173));
        sum += f!(0.0039) * cos(base / f!(1.1177) + f!(191.837));
        sum += f!(0.0037) * cos(base / f!(15.7866) + f!(21.736));
        sum += f!(0.0021) * cos(base / f!(2.1354) + f!(15.704));
        sum += f!(0.0020) * cos(base / f!(2.4694) + f!(95.528));
        sum += f!(0.0018) * cos(base / f!(32.8493) + f!(49.095));

        sum
    }

    /// Returns Local Mean Solar Time (LMST) at the given planetocentric east longitude
    /// as a [`Dt`] representing time into the current Martian sol (range [0, one sol)).
    ///
    /// LMST is the uniform mean solar time adjusted for longitude.
    ///
    /// Longitude is east-positive (standard planetocentric convention, 0–360° E).
    /// Internally converts to TT and uses the current NASA GISS Mars24 definition of MST.
    pub const fn to_mars_lmst(&self, east_longitude_deg: Real) -> Dt {
        let jd_tt = self.to(Scale::TT).to_jd_f_raw();

        // MST in hours (0–24) — prime-meridian mean solar time (NASA Mars24 formula)
        let mst = (f!(24.0)
            * ((jd_tt - f!(2451549.5)) / f!(1.0274912517) + f!(44796.0) - f!(0.0009626)))
            % f!(24.0);

        // Convert east-positive longitude to west-positive (NASA convention)
        let lambda_west = rem_euclid_f(-east_longitude_deg, f!(360.0));

        // LMST in hours
        let mut lmst_hours = mst - lambda_west / f!(15.0);
        if lmst_hours < f!(0.0) {
            lmst_hours += f!(24.0);
        }

        // Convert hours → seconds into the sol and return as Dt (consistent with to_mtc)
        let seconds_into_sol = lmst_hours * f!(3600.0);
        Dt::from_sec_f(seconds_into_sol, Scale::TAI, Scale::TAI)
    }

    /// Returns Local True Solar Time (LTST) at the given planetocentric east longitude
    /// as a [`Dt`] representing seconds into the current Martian sol (range [0, one sol)).
    ///
    /// LTST is the actual sundial time (true solar time) at the location — what a
    /// local observer would see on a sundial. It equals LMST plus the Equation of Time.
    ///
    /// Longitude is east-positive (standard planetocentric convention, 0–360° E).
    pub const fn to_mars_ltst(&self, east_longitude_deg: Real) -> Dt {
        let lmst = self.to_mars_lmst(east_longitude_deg);

        // We already have Ls; reuse it for EOT
        let ls = self.to_mars_ls();

        // Equation of center (ν − M) — same term used in to_mars_ls
        let dt_j2000 = self.to(Scale::TT).to_jd_f_raw() - f!(2451545.0);
        let m = MARS_LS_M0 + MARS_LS_M_RATE * dt_j2000;
        let pbs = Self::mars_perturber_sum(dt_j2000);
        let eq_center = (f!(10.691) + f!(3.0e-7) * dt_j2000) * sin(m.to_radians())
            + f!(0.623) * sin((f!(2.0) * m).to_radians())
            + f!(0.050) * sin((f!(3.0) * m).to_radians())
            + f!(0.005) * sin((f!(4.0) * m).to_radians())
            + f!(0.0005) * sin((f!(5.0) * m).to_radians())
            + pbs;

        // Equation of Time in degrees (NASA GISS / AM2000)
        let eot_deg = MARS_EOT_COEFF_2LS * sin(f!(2.0) * ls.to_radians())
            + MARS_EOT_COEFF_4LS * sin(f!(4.0) * ls.to_radians())
            + MARS_EOT_COEFF_6LS * sin(f!(6.0) * ls.to_radians())
            - eq_center;

        // Convert EOT to seconds (1° = 3600 s / 15 = 240 s per degree)
        let eot_seconds = eot_deg * f!(240.0);

        // LTST = LMST + EOT (as duration)
        lmst.add(Dt::from_sec_f(eot_seconds, Scale::TAI, Scale::TAI))
    }

    /// Returns the integer Mars Year (MY) for this instant.
    ///
    /// Mars Year numbering follows the standard Clancy et al. (2000) system:
    /// - Mars Year 1 begins at the northern vernal equinox (Ls = 0°) on 1955 April 11.
    /// - Each Mars Year is one tropical year on Mars (686.9725 Earth days).
    /// - Current missions operate in Mars Year 36–37 (as of 2026).
    ///
    /// This is the canonical year count used in all Mars science literature,
    /// mission reports, and atmospheric databases.
    ///
    /// Source: Clancy et al. (2000), *J. Geophys. Res.: Planets* 105(E4), 9553–9572;
    /// confirmed in NASA GISS Mars24 Technical Notes (2025) and LMD Mars Climate Database.
    ///
    /// To get the fractional progress through the year, simply use:
    /// `self.to_mars_ls(current) / 360.0`
    pub const fn to_mars_year(&self) -> i64 {
        let jd_tt = self.to(Scale::TT).to_jd_f_raw();

        let days_since_epoch = jd_tt - MARS_YEAR_EPOCH_JD;
        let years_elapsed = floor_f(days_since_epoch / MARS_TROPICAL_YEAR_DAYS);

        1 + (years_elapsed as i64)
    }
}
