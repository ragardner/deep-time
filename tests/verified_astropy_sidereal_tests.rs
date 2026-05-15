/*
from astropy.time import Time
import astropy.units as u

def print_values(mjd: float, longitude_deg: float = 0.0):
    utc = Time(mjd, format="mjd", scale="utc")
    ut1 = utc.ut1

    # --- Mean quantities ---
    era = ut1.earth_rotation_angle(longitude=longitude_deg * u.deg).rad
    gmst = ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).rad
    mean_time_sec = (
        ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).hour * 3600.0
    )

    eo = gmst - era  # Equation of the Origins

    # --- Apparent quantities ---
    gast = ut1.sidereal_time("apparent", longitude=longitude_deg * u.deg).rad
    apparent_time_sec = (
        ut1.sidereal_time("apparent", longitude=longitude_deg * u.deg).hour * 3600.0
    )

    ee = gast - gmst  # Equation of the Equinoxes

    print(f"MJD                         = {mjd:.1f}   lon = {longitude_deg:6.1f}°")
    print(f"MJD UT1                     = {ut1.mjd:.15f}")
    print(f"ERA                         = {era:.15f} rad")
    print(f"EO  (Equation of Origins)   = {eo:.15f} rad")
    print(f"GMST (Mean Sidereal)        = {gmst:.15f} rad")
    print(f"Mean time                   = {mean_time_sec:.12f} seconds")
    print(f"EE  (Equation of Equinoxes) = {ee:.15f} rad")
    print(f"GAST (Apparent Sidereal)    = {gast:.15f} rad")
    print(f"Apparent time               = {apparent_time_sec:.12f} seconds")
    print("-" * 70)

"""
=== MJD 56879.0 (Greenwich) ===
MJD                         = 56879.0   lon =    0.0°
MJD UT1                     = 56878.999996330509020
ERA                         = 5.553778083625185 rad
EO  (Equation of Origins)   = 0.003265960688002 rad
GMST (Mean Sidereal)        = 5.557044044313187 rad
Mean time                   = 76414.840873789362 seconds
EE  (Equation of Equinoxes) = 0.000036305881223 rad
GAST (Apparent Sidereal)    = 5.557080350194410 rad
Apparent time               = 76415.340115493105 seconds
----------------------------------------------------------------------
=== MJD 57753.5 (Greenwich) ===
MJD                         = 57753.5   lon =    0.0°
MJD UT1                     = 57753.500001062020601
ERA                         = 4.889150664566767 rad
EO  (Equation of Origins)   = 0.003801374923784 rad
GMST (Mean Sidereal)        = 4.892952039490551 rad
Mean time                   = 67282.920293456904 seconds
EE  (Equation of Equinoxes) = -0.000028847974172 rad
GAST (Apparent Sidereal)    = 4.892923191516378 rad
Apparent time               = 67282.523605336683 seconds
----------------------------------------------------------------------

=== MJD 60961.0 (Greenwich) ===
MJD                         = 60961.0   lon =    0.0°
MJD UT1                     = 60961.000001080494258
ERA                         = 0.374881350298881 rad
EO  (Equation of Origins)   = 0.005765238883985 rad
GMST (Mean Sidereal)        = 0.380646589182866 rad
Mean time                   = 5234.266331094793 seconds
EE  (Equation of Equinoxes) = 0.000013909981015 rad
GAST (Apparent Sidereal)    = 0.380660499163881 rad
Apparent time               = 5234.457607064058 seconds
----------------------------------------------------------------------

=== MJD 57259.0 (Greenwich) ===
MJD                         = 57259.0   lon =    0.0°
MJD UT1                     = 57259.000003255881893
ERA                         = 5.807464647552663 rad
EO  (Equation of Origins)   = 0.003498615440127 rad
GMST (Mean Sidereal)        = 5.810963262992789 rad
Mean time                   = 79906.480770013513 seconds
EE  (Equation of Equinoxes) = 0.000007338364795 rad
GAST (Apparent Sidereal)    = 5.810970601357584 rad
Apparent time               = 79906.581679773022 seconds
----------------------------------------------------------------------

=== MJD 56879.0 at +85° E ===
MJD                         = 56879.0   lon =   85.0°
MJD UT1                     = 56878.999996330509020
ERA                         = 0.754122640638924 rad
EO  (Equation of Origins)   = 0.003265960688001 rad
GMST (Mean Sidereal)        = 0.757388601326925 rad
Mean time                   = 10414.840873763829 seconds
EE  (Equation of Equinoxes) = 0.000036305881224 rad
GAST (Apparent Sidereal)    = 0.757424907208149 rad
Apparent time               = 10415.340115467583 seconds
----------------------------------------------------------------------
"""
*/

#[cfg(all(feature = "bop-tests", feature = "sidereal"))]
#[cfg(test)]
mod sidereal_tests {
    use deep_time::bop::{BopData, BopFormat, Separator};
    use deep_time::{Dt, Sidereal};

    fn load_finals2000a() -> BopData {
        let path = "finals.all.iau2000.txt";
        BopData::from_text_file(path, BopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load finals2000A.all / finals.all.iau2000.txt")
    }

    #[test]
    fn test_sidereal_vs_astropy() {
        let provider = load_finals2000a();

        // Helper to compare one case
        let check = |mjd: f64,
                     astropy_mjd_ut1: f64,
                     lon_deg: f64,
                     astropy_rot: f64,  // ERA from Astropy
                     astropy_gmst: f64, // GMST (mean sidereal) from Astropy
                     astropy_time: f64,
                     eo: f64,
                     ee: f64,
                     astropy_apparent_angle: f64,
                     astropy_apparent_time: f64| {
            let sid = if lon_deg == 0.0 {
                Sidereal::EARTH
            } else {
                Sidereal {
                    longitude_rad: lon_deg * (core::f64::consts::PI / 180.0),
                    ..Sidereal::EARTH
                }
            };

            let dut1 = Dt::orientation_offset(mjd, &provider).expect("to_ut1 failed");
            let rust_ut1_mjd = mjd + (dut1 / 86400.0);
            let rust_rot = sid.local_rotation_angle(rust_ut1_mjd);

            // Use the EO value from Astropy
            let rust_local = sid.local_sidereal_angle_mean(rust_ut1_mjd, eo);
            let rust_time = sid.local_sidereal_time_mean(rust_ut1_mjd, eo);
            let rust_app_angle = sid.local_sidereal_angle_apparent(rust_ut1_mjd, eo, ee);
            let rust_app_time = sid.local_sidereal_time_apparent(rust_ut1_mjd, eo, ee);

            // eprintln!("\nMJD                  = {mjd:.1}             lon     = {lon_deg:6.1}° ===");
            // eprintln!(
            //     "UT1 MJD  Rust        = {:.9}     Astropy = {:.15}   diff = {:.2e}",
            //     rust_ut1_mjd,
            //     astropy_mjd_ut1,
            //     (rust_ut1_mjd - astropy_mjd_ut1).abs()
            // );
            // eprintln!(
            //     "rotation_angle Rust  = {:.15}   Astropy = {:.15}       diff = {:.2e}",
            //     rust_rot,
            //     astropy_rot,
            //     (rust_rot - astropy_rot).abs()
            // );
            // eprintln!(
            //     "sidereal_angle    Rust  = {:.15}   Astropy = {:.15}       diff = {:.2e}",
            //     rust_local,
            //     astropy_gmst,
            //     (rust_local - astropy_gmst).abs()
            // );
            // eprintln!(
            //     "sidereal_time     Rust  = {:.12}  Astropy = {:.12}      diff = {:.2e}",
            //     rust_time,
            //     astropy_time,
            //     (rust_time - astropy_time).abs()
            // );
            // eprintln!(
            //     "apparent sidereal_angle    Rust  = {:.15}   Astropy = {:.15}       diff = {:.2e}",
            //     rust_app_angle,
            //     astropy_apparent_angle,
            //     (rust_app_angle - astropy_apparent_angle).abs()
            // );
            // eprintln!(
            //     "apparent sidereal_time     Rust  = {:.12}  Astropy = {:.12}      diff = {:.2e}",
            //     rust_app_time,
            //     astropy_apparent_time,
            //     (rust_app_time - astropy_apparent_time).abs()
            // );
            let ut1_diff = (rust_ut1_mjd - astropy_mjd_ut1).abs();
            let rot_diff = (rust_rot - astropy_rot).abs();
            let angle_diff = (rust_local - astropy_gmst).abs();
            let time_diff = (rust_time - astropy_time).abs();
            let app_angle_diff = (rust_app_angle - astropy_apparent_angle).abs();
            let app_time_diff = (rust_app_time - astropy_apparent_time).abs();
            assert!(
                ut1_diff < 1e-9,
                "UT1 MJD too far from Astropy: diff = {ut1_diff:.2e}"
            );
            assert!(
                rot_diff < 1e-9,
                "rotation_angle too far from Astropy: diff = {rot_diff:.2e}"
            );
            assert!(
                angle_diff < 1e-9,
                "sidereal_angle too far from Astropy: diff = {angle_diff:.2e}"
            );
            assert!(
                time_diff < 1.20e-5,
                "sidereal_time too far from Astropy: diff = {time_diff:.2e}"
            );
            assert!(
                app_angle_diff < 1e-9,
                "apparent sidereal_angle too far from Astropy: diff = {app_angle_diff:.2e}"
            );
            assert!(
                app_time_diff < 1.20e-5,
                "apparent sidereal_time too far from Astropy: diff = {app_time_diff:.2e}"
            );
        };

        // EO values taken from Astropy
        check(
            56879.0,
            56878.999996330509020,
            0.0,
            5.553778083625185,
            5.557044044313187,
            76414.840873789362,
            0.003265960688002,
            0.000036305881223,
            5.557080350194410,
            76415.340115493105,
        );
        check(
            57753.5,
            57753.500001062020601,
            0.0,
            4.889150664566767,
            4.892952039490551,
            67282.920293456904,
            0.003801374923784,
            -0.000028847974172,
            4.892923191516378,
            67282.523605336683,
        );
        check(
            60961.0,
            60961.000001080494258,
            0.0,
            0.374881350298881,
            0.380646589182866,
            5234.266331094793,
            0.005765238883985,
            0.000013909981015,
            0.380660499163881,
            5234.457607064058,
        );
        check(
            57259.0,
            57259.000003255881893,
            0.0,
            5.807464647552663,
            5.810963262992789,
            79906.480770013513,
            0.003498615440127,
            0.000007338364795,
            5.810970601357584,
            79906.581679773022,
        );
        check(
            56879.0,
            56878.999996330509020,
            85.0,
            0.754122640638924,
            0.757388601326925,
            10414.840873763829,
            0.003265960688001,
            0.000036305881224,
            0.757424907208149,
            10415.340115467583,
        );
    }
}
