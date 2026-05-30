#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/*
from astropy.time import Time
import astropy.units as u
import erfa


def compute_values(mjd: float, longitude_deg: float = 0.0):
    """Compute all the quantities and return them in a dict."""
    utc = Time(mjd, format="mjd", scale="utc")
    ut1 = utc.ut1
    tt = utc.tt  # for eo06a

    # --- Mean quantities ---
    era = ut1.earth_rotation_angle(longitude=longitude_deg * u.deg).rad
    gmst = ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).rad
    mean_time_sec = (
        ut1.sidereal_time("mean", longitude=longitude_deg * u.deg).hour * 3600.0
    )

    # --- Apparent quantities ---
    gast = ut1.sidereal_time("apparent", longitude=longitude_deg * u.deg).rad
    apparent_time_sec = (
        ut1.sidereal_time("apparent", longitude=longitude_deg * u.deg).hour * 3600.0
    )

    # EO computations
    eo_mean = era - gmst
    eo_apparent_erfa = erfa.eo06a(tt.jd1, tt.jd2)
    ee = gast - gmst

    return {
        "mjd": mjd,
        "longitude_deg": longitude_deg,
        "ut1_mjd": float(ut1.mjd),
        "era_rad": float(era),
        "eo_apparent_rad": float(eo_apparent_erfa),
        "eo_mean_rad": float(eo_mean),
        "gmst_rad": float(gmst),
        "mean_sidereal_time_sec": float(mean_time_sec),
        "ee_rad": float(ee),
        "gast_rad": float(gast),
        "apparent_sidereal_time_sec": float(apparent_time_sec),
    }


def main():
    cases = [
        (57259.0, 0.0),
        (57753.5, 0.0),
        (60961.0, 0.0),
        (56879.0, 350.0),
    ]

    results = [compute_values(mjd, lon) for mjd, lon in cases]

    # === Print Rust code ===
    print("#[derive(Debug, Clone, Copy)]")
    print("pub struct EoAndSiderealTimes {")
    print("    pub mjd: f64,")
    print("    pub longitude_deg: f64,")
    print("    pub ut1_mjd: f64,")
    print("    pub earth_rotation_angle_rad: f64,      // ERA (Earth Rotation Angle)")
    print(
        "    pub eo_apparent_rad: f64,               // Equation of Origins from erfa::eo06a (apparent, TT)"
    )
    print("    pub eo_mean_rad: f64,                   // Mean EO = ERA - GMST")
    print("    pub gmst_rad: f64,                      // Greenwich Mean Sidereal Time")
    print("    pub mean_sidereal_time_sec: f64,")
    print("    pub equation_of_equinoxes_rad: f64,     // EE = GAST - GMST")
    print(
        "    pub gast_rad: f64,                      // Greenwich Apparent Sidereal Time"
    )
    print("    pub apparent_sidereal_time_sec: f64,")
    print("}")
    print()
    print(f"pub const TEST_CASES: [EoAndSiderealTimes; {len(results)}] = [")

    for res in results:
        print("    EoAndSiderealTimes {")
        print(f"        mjd: {res['mjd']:.1f},")
        print(f"        longitude_deg: {res['longitude_deg']:.1f},")
        print(f"        ut1_mjd: {res['ut1_mjd']:.15f},")
        print(f"        earth_rotation_angle_rad: {res['era_rad']:.15f},")
        print(f"        eo_apparent_rad: {res['eo_apparent_rad']:.15f},")
        print(f"        eo_mean_rad: {res['eo_mean_rad']:.15f},")
        print(f"        gmst_rad: {res['gmst_rad']:.15f},")
        print(f"        mean_sidereal_time_sec: {res['mean_sidereal_time_sec']:.12f},")
        print(f"        equation_of_equinoxes_rad: {res['ee_rad']:.15f},")
        print(f"        gast_rad: {res['gast_rad']:.15f},")
        print(
            f"        apparent_sidereal_time_sec: {res['apparent_sidereal_time_sec']:.12f},"
        )
        print("    },")

    print("];")


if __name__ == "__main__":
    main()
*/

#[cfg(all(feature = "eop-tests", feature = "sidereal-earth"))]
mod sidereal_tests {
    use deep_time::eop::{EopData, EopFormat, Separator};
    use deep_time::{Dt, Sidereal};

    fn load_finals2000a() -> EopData {
        let path = "finals.all.iau2000.txt";
        EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load EopData")
    }

    #[test]
    fn test_sidereal_vs_astropy() {
        fn check(eos: EoAndSiderealTimes, provider: &EopData) {
            let mjd = eos.mjd;
            let lon_deg = eos.longitude_deg;
            let astropy_mjd_ut1 = eos.ut1_mjd;
            let astropy_rot = eos.earth_rotation_angle_rad;
            let _eo_app = eos.eo_apparent_rad;
            let _eo_mean = eos.eo_mean_rad;
            let _ee = eos.equation_of_equinoxes_rad;
            let astropy_gmst = eos.gmst_rad;
            let astropy_time = eos.mean_sidereal_time_sec;
            let astropy_apparent_angle = eos.gast_rad;
            let astropy_apparent_time = eos.apparent_sidereal_time_sec;

            let sid = if lon_deg == 0.0 {
                Sidereal::EARTH
            } else {
                Sidereal {
                    longitude_rad: lon_deg * (core::f64::consts::PI / 180.0),
                    ..Sidereal::EARTH
                }
            };

            let dut1 = Dt::mjd_to_eop_offset_f(mjd, &provider).expect("to_ut1 failed");
            let rust_ut1_mjd = mjd + (dut1 / 86400.0);
            let rust_rot = sid.local_rotation_angle(rust_ut1_mjd);
            let rust_eo = sid.earth_eo_apparent(rust_ut1_mjd + 32.184 / 86400.0);
            let rust_ee = sid.earth_ee(rust_ut1_mjd + 32.184 / 86400.0);
            let rust_eo_mean = rust_eo + rust_ee;

            // Use the EO value from Astropy
            let rust_local = sid.local_sidereal_angle_mean(rust_ut1_mjd, rust_eo_mean);
            let rust_time = sid.local_sidereal_time_mean(rust_ut1_mjd, rust_eo_mean);
            let rust_app_angle = sid.local_sidereal_angle_apparent(rust_ut1_mjd, rust_eo);
            let rust_app_time = sid.local_sidereal_time_apparent(rust_ut1_mjd, rust_eo);

            // eprintln!("\nMJD                  = {mjd}             lon     = {lon_deg}° ===");
            // eprintln!(
            //     "UT1 MJD  Rust        = {:.9}     Astropy = {:.15}   diff = {:.2e}",
            //     rust_ut1_mjd,
            //     astropy_mjd_ut1,
            //     (rust_ut1_mjd - astropy_mjd_ut1).abs()
            // );
            // eprintln!(
            //     "RUST EO APP: {}, ASTROPY EO APP: {}, DIFF: {:.2e}",
            //     rust_eo,
            //     _eo_app,
            //     (rust_eo - _eo_app).abs()
            // );
            // eprintln!(
            //     "RUST EO MEAN: {}, ASTROPY EO MEAN: {}, DIFF: {:.2e}",
            //     rust_eo_mean,
            //     _eo_mean,
            //     (rust_eo_mean - _eo_mean).abs()
            // );
            // eprintln!(
            //     "RUST EE: {}, ASTROPY EE: {}, DIFF: {:.2e}",
            //     rust_ee,
            //     _ee,
            //     (rust_ee - _ee).abs()
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
                rot_diff < 5e-9,
                "rotation_angle too far from Astropy: diff = {rot_diff:.2e}"
            );
            assert!(
                angle_diff < 5e-9,
                "sidereal_angle too far from Astropy: diff = {angle_diff:.2e}"
            );
            assert!(
                time_diff < 6.10e-5,
                "sidereal_time too far from Astropy: diff = {time_diff:.2e}"
            );
            assert!(
                app_angle_diff < 5e-9,
                "apparent sidereal_angle too far from Astropy: diff = {app_angle_diff:.2e}"
            );
            assert!(
                app_time_diff < 7e-5,
                "apparent sidereal_time too far from Astropy: diff = {app_time_diff:.2e}"
            );
        }

        let provider = load_finals2000a();

        #[derive(Debug, Clone, Copy)]
        pub struct EoAndSiderealTimes {
            pub mjd: f64,
            pub longitude_deg: f64,
            pub ut1_mjd: f64,
            pub earth_rotation_angle_rad: f64, // ERA (Earth Rotation Angle)
            pub eo_apparent_rad: f64, // Equation of Origins from erfa::eo06a (apparent, TT)
            pub eo_mean_rad: f64,     // Mean EO = ERA - GMST
            pub gmst_rad: f64,        // Greenwich Mean Sidereal Time
            pub mean_sidereal_time_sec: f64,
            pub equation_of_equinoxes_rad: f64, // EE = GAST - GMST
            pub gast_rad: f64,                  // Greenwich Apparent Sidereal Time
            pub apparent_sidereal_time_sec: f64,
        }

        pub const TEST_CASES: [EoAndSiderealTimes; 5] = [
            EoAndSiderealTimes {
                mjd: 53371.0,
                longitude_deg: 0.0,
                ut1_mjd: 53370.999994170932041,
                earth_rotation_angle_rad: 1.757186292111117,
                eo_apparent_rad: -0.001085302402798,
                eo_mean_rad: -0.001118258994844,
                gmst_rad: 1.758304551105961,
                mean_sidereal_time_sec: 24178.423170483926,
                equation_of_equinoxes_rad: -0.000032956592046,
                gast_rad: 1.758271594513915,
                apparent_sidereal_time_sec: 24177.969984812393,
            },
            EoAndSiderealTimes {
                mjd: 55197.0,
                longitude_deg: 0.0,
                ut1_mjd: 55197.000001321015588,
                earth_rotation_angle_rad: 1.752484708914275,
                eo_apparent_rad: -0.002309329056754,
                eo_mean_rad: -0.002236174848923,
                gmst_rad: 1.754720883763197,
                mean_sidereal_time_sec: 24129.144207143308,
                equation_of_equinoxes_rad: 0.000073154207832,
                gast_rad: 1.754794037971029,
                apparent_sidereal_time_sec: 24130.150149710280,
            },
            EoAndSiderealTimes {
                mjd: 57753.999999,
                longitude_deg: 0.0,
                ut1_mjd: 57754.000005843590770,
                earth_rotation_angle_rad: 1.756189226484602,
                eo_apparent_rad: -0.003772953343248,
                eo_mean_rad: -0.003801681054842,
                gmst_rad: 1.759990907539444,
                mean_sidereal_time_sec: 24201.612236018307,
                equation_of_equinoxes_rad: -0.000028727711594,
                gast_rad: 1.759962179827850,
                apparent_sidereal_time_sec: 24201.217201627253,
            },
            EoAndSiderealTimes {
                mjd: 57754.0,
                longitude_deg: 0.0,
                ut1_mjd: 57754.000006843598385,
                earth_rotation_angle_rad: 1.756195526947061,
                eo_apparent_rad: -0.003772953344064,
                eo_mean_rad: -0.003801681055454,
                gmst_rad: 1.759997208002515,
                mean_sidereal_time_sec: 24201.698873604622,
                equation_of_equinoxes_rad: -0.000028727711391,
                gast_rad: 1.759968480291124,
                apparent_sidereal_time_sec: 24201.303839216362,
            },
            EoAndSiderealTimes {
                mjd: 56879.0,
                longitude_deg: 350.0,
                ut1_mjd: 56878.999996330509020,
                earth_rotation_angle_rad: 5.379245158425889,
                eo_apparent_rad: -0.003302266569225,
                eo_mean_rad: -0.003265960688001,
                gmst_rad: 5.382511119113890,
                mean_sidereal_time_sec: 74014.840873791225,
                equation_of_equinoxes_rad: 0.000036305881223,
                gast_rad: 5.382547424995114,
                apparent_sidereal_time_sec: 74015.340115494982,
            },
        ];

        for s in TEST_CASES {
            check(s, &provider);
        }
    }
}
