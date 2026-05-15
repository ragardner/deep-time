/*
from astropy.time import Time
import astropy.units as u


def print_values(mjd: float, longitude_deg: float = 0.0):
    utc = Time(mjd, format="mjd", scale="utc")
    ut1 = utc.ut1

    era = ut1.earth_rotation_angle(longitude=longitude_deg * u.deg).rad
    gmst = ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).rad
    local_time_sec = (
        ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).hour * 3600.0
    )

    # Equation of the Origins (only meaningful when longitude=0 for the base value)
    eo = gmst - era

    print(f"MJD                      = {mjd:.1f}   lon = {longitude_deg:6.1f}°")
    print(f"MJD UT1                  = {ut1.mjd:.15f}")
    print(f"ERA (rotation_angle)     = {era:.15f} rad")
    print(f"GMST (mean sidereal)     = {gmst:.15f} rad")
    print(f"EO (Equation of Origins) = {eo:.15f} rad")
    print(f"local_time               = {local_time_sec:.12f} seconds")
    print("-" * 70)


print("=== MJD 56879.0 (Greenwich) ===")
print_values(56879.0)

print("=== MJD 57753.5 (Greenwich) ===")
print_values(57753.5)

print("\n=== MJD 60961.0 (Greenwich) ===")
print_values(60961.0)

print("\n=== MJD 57259.0 (Greenwich) ===")
print_values(57259.0)

print("\n=== MJD 56879.0 at +85° E ===")
print_values(56879.0, longitude_deg=85.0)


"""
=== MJD 56879.0 (Greenwich) ===
MJD                      = 56879.0   lon =    0.0°
MJD UT1                  = 56878.999996330509020
ERA (rotation_angle)     = 5.553778083625185 rad
GMST (mean sidereal)     = 5.557044044313187 rad
EO (Equation of Origins) = 0.003265960688002 rad
local_time               = 76414.840873789362 seconds
----------------------------------------------------------------------
=== MJD 57753.5 (Greenwich) ===
MJD                      = 57753.5   lon =    0.0°
MJD UT1                  = 57753.500001062020601
ERA (rotation_angle)     = 4.889150664566767 rad
GMST (mean sidereal)     = 4.892952039490551 rad
EO (Equation of Origins) = 0.003801374923784 rad
local_time               = 67282.920293456904 seconds
----------------------------------------------------------------------

=== MJD 60961.0 (Greenwich) ===
MJD                      = 60961.0   lon =    0.0°
MJD UT1                  = 60961.000001080494258
ERA (rotation_angle)     = 0.374881350298881 rad
GMST (mean sidereal)     = 0.380646589182866 rad
EO (Equation of Origins) = 0.005765238883985 rad
local_time               = 5234.266331094793 seconds
----------------------------------------------------------------------

=== MJD 57259.0 (Greenwich) ===
MJD                      = 57259.0   lon =    0.0°
MJD UT1                  = 57259.000003255881893
ERA (rotation_angle)     = 5.807464647552663 rad
GMST (mean sidereal)     = 5.810963262992789 rad
EO (Equation of Origins) = 0.003498615440127 rad
local_time               = 79906.480770013513 seconds
----------------------------------------------------------------------

=== MJD 56879.0 at +85° E ===
MJD                      = 56879.0   lon =   85.0°
MJD UT1                  = 56878.999996330509020
ERA (rotation_angle)     = 0.754122640638924 rad
GMST (mean sidereal)     = 0.757388601326925 rad
EO (Equation of Origins) = 0.003265960688001 rad
local_time               = 10414.840873763829 seconds
----------------------------------------------------------------------
"""
*/

#[cfg(all(feature = "bop-tests", feature = "sidereal"))]
#[cfg(test)]
mod sidereal_tests {
    use deep_time::Dt;
    use deep_time::bop::{BopData, BopFormat, Separator};
    use deep_time::sidereal::Sidereal;

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
                     eo: f64| {
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

            eprintln!("\nMJD                  = {mjd:.1}             lon     = {lon_deg:6.1}° ===");
            eprintln!(
                "UT1 MJD  Rust        = {:.9}     Astropy = {:.15}   diff = {:.2e}",
                rust_ut1_mjd,
                astropy_mjd_ut1,
                (rust_ut1_mjd - astropy_mjd_ut1).abs()
            );
            eprintln!(
                "rotation_angle Rust  = {:.15}   Astropy = {:.15}       diff = {:.2e}",
                rust_rot,
                astropy_rot,
                (rust_rot - astropy_rot).abs()
            );
            eprintln!(
                "sidereal_angle    Rust  = {:.15}   Astropy = {:.15}       diff = {:.2e}",
                rust_local,
                astropy_gmst,
                (rust_local - astropy_gmst).abs()
            );
            eprintln!(
                "sidereal_time     Rust  = {:.12}  Astropy = {:.12}      diff = {:.2e}",
                rust_time,
                astropy_time,
                (rust_time - astropy_time).abs()
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
        );
        check(
            57753.5,
            57753.500001062020601,
            0.0,
            4.889150664566767,
            4.892952039490551,
            67282.920293456904,
            0.003801374923784,
        );
        check(
            60961.0,
            60961.000001080494258,
            0.0,
            0.374881350298881,
            0.380646589182866,
            5234.266331094793,
            0.005765238883985,
        );
        check(
            57259.0,
            57259.000003255881893,
            0.0,
            5.807464647552663,
            5.810963262992789,
            79906.480770013513,
            0.003498615440127,
        );
        check(
            56879.0,
            56878.999996330509020,
            85.0,
            0.754122640638924, // Note: Astropy already added longitude here
            0.757388601326925,
            10414.840873763829,
            0.003265960688001,
        );
    }
}
