#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

/*
"""Generate UT1 / EOP expected values from Astropy for deep-time comparison tests.

Uses Astropy's default IERS_Auto table. The Rust tests load the C04 asset,
which matches Astropy's eopc04 DUT1 on the shared dates.

Run with Astropy available, e.g. ``python this_script.py``.
"""

from __future__ import annotations

from astropy.time import Time
from astropy.utils import iers

CASES: list[tuple[str, float]] = [
    ("mjd_53371_midnight", 53371.0),
    ("mjd_55197_midnight", 55197.0),
    ("mjd_56879_midnight", 56879.0),
    ("mjd_56879_midday", 56879.5),
    ("mjd_60961_midnight", 60961.0),
    ("mjd_60961_midday", 60961.5),
    ("mjd_51990_frac", 51990.001212179712),
    # 2016-12-31 leap (MJD 57753); 2017-01-01 = 57754
    ("leap_2016_mjd_57753_midnight", 57753.0),
    ("leap_2016_mjd_57753_quarter", 57753.25),
    ("leap_2016_mjd_57753_midday", 57753.5),
    ("leap_2016_mjd_57753_three_quarter", 57753.75),
    ("leap_2016_mjd_57754_midnight", 57754.0),
    ("leap_2016_mjd_57754_midday", 57754.5),
    ("leap_2016_mjd_57755_midnight", 57755.0),
    # 2015-06-30 leap (MJD 57203); 2015-07-01 = 57204
    ("leap_2015_mjd_57203_midnight", 57203.0),
    ("leap_2015_mjd_57203_midday", 57203.5),
    ("leap_2015_mjd_57204_midnight", 57204.0),
    # 2012-06-30 leap (MJD 56108); 2012-07-01 = 56109
    ("leap_2012_mjd_56108_midnight", 56108.0),
    ("leap_2012_mjd_56108_midday", 56108.5),
    ("leap_2012_mjd_56109_midnight", 56109.0),
]


def compute(label: str, mjd: float) -> dict:
    t_utc = Time(mjd, format="mjd", scale="utc")
    pm_x, pm_y = iers.earth_orientation_table.get().pm_xy(t_utc)
    return {
        "label": label,
        "mjd_utc": mjd,
        "delta_ut1_utc_s": float(t_utc.delta_ut1_utc),
        "mjd_ut1": float(t_utc.ut1.mjd),
        "pm_x_arcsec": float(pm_x.to_value("arcsec")),
        "pm_y_arcsec": float(pm_y.to_value("arcsec")),
    }


def print_rust(results: list[dict]) -> None:
    n = len(results)
    print("#[derive(Debug, Clone, Copy)]")
    print("struct AstropyUt1Case {")
    print("    label: &'static str,")
    print("    mjd_utc: f64,")
    print("    delta_ut1_utc_s: f64,")
    print("    mjd_ut1: f64,")
    print("    pm_x_arcsec: f64,")
    print("    pm_y_arcsec: f64,")
    print("}")
    print()
    print(f"const TEST_CASES: [AstropyUt1Case; {n}] = [")
    for r in results:
        print("    AstropyUt1Case {")
        print(f'        label: "{r["label"]}",')
        print(f"        mjd_utc: {r['mjd_utc']!r},")
        print(f"        delta_ut1_utc_s: {r['delta_ut1_utc_s']!r},")
        print(f"        mjd_ut1: {r['mjd_ut1']!r},")
        print(f"        pm_x_arcsec: {r['pm_x_arcsec']!r},")
        print(f"        pm_y_arcsec: {r['pm_y_arcsec']!r},")
        print("    },")
    print("];")


if __name__ == "__main__":
    results = [compute(label, mjd) for label, mjd in CASES]
    print_rust(results)
*/

#[cfg(all(feature = "std", feature = "eop"))]
mod ut1_vs_astropy {
    use deep_time::eop::{EopData, EopFormat, Separator};

    /// C04 / eopc04 definitive series (same DUT1 source as Astropy IERS_Auto).
    fn load_c04() -> EopData {
        let path = "tests/assets/EOP_20u24_C04_one_file_1962-now.txt";
        EopData::from_text_file(path, EopFormat::C04, Separator::Whitespace)
            .expect("failed to load C04 EopData")
    }

    fn assert_close(got: f64, expected: f64, tol: f64, label: &str) {
        let err = (got - expected).abs();
        assert!(
            err <= tol,
            "{label}: got {got}, expected {expected}, |err|={err} (tol {tol})"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct AstropyUt1Case {
        label: &'static str,
        mjd_utc: f64,
        delta_ut1_utc_s: f64,
        mjd_ut1: f64,
        pm_x_arcsec: f64,
        pm_y_arcsec: f64,
    }

    // Expected values from the Python block at the top of this file (Astropy 8.x).
    const TEST_CASES: [AstropyUt1Case; 20] = [
        AstropyUt1Case {
            label: "mjd_53371_midnight",
            mjd_utc: 53371.0,
            delta_ut1_utc_s: -0.5036316,
            mjd_ut1: 53370.99999417093,
            pm_x_arcsec: 0.149128,
            pm_y_arcsec: 0.238178,
        },
        AstropyUt1Case {
            label: "mjd_55197_midnight",
            mjd_utc: 55197.0,
            delta_ut1_utc_s: 0.1141359,
            mjd_ut1: 55197.000001321016,
            pm_x_arcsec: 0.09867,
            pm_y_arcsec: 0.19284,
        },
        AstropyUt1Case {
            label: "mjd_56879_midnight",
            mjd_utc: 56879.0,
            delta_ut1_utc_s: -0.3170438,
            mjd_ut1: 56878.99999633051,
            pm_x_arcsec: 0.204273,
            pm_y_arcsec: 0.369124,
        },
        AstropyUt1Case {
            label: "mjd_56879_midday",
            mjd_utc: 56879.5,
            delta_ut1_utc_s: -0.31733385000000003,
            mjd_ut1: 56879.499996327155,
            pm_x_arcsec: 0.20486900000000002,
            pm_y_arcsec: 0.368719,
        },
        AstropyUt1Case {
            label: "mjd_60961_midnight",
            mjd_utc: 60961.0,
            delta_ut1_utc_s: 0.0933544,
            mjd_ut1: 60961.000001080494,
            pm_x_arcsec: 0.208808,
            pm_y_arcsec: 0.326409,
        },
        AstropyUt1Case {
            label: "mjd_60961_midday",
            mjd_utc: 60961.5,
            delta_ut1_utc_s: 0.09368015,
            mjd_ut1: 60961.50000108426,
            pm_x_arcsec: 0.2079835,
            pm_y_arcsec: 0.32612450000000004,
        },
        AstropyUt1Case {
            label: "mjd_51990_frac",
            mjd_utc: 51990.00121217971,
            delta_ut1_utc_s: 0.03457277535414211,
            mjd_ut1: 51990.00121257986,
            pm_x_arcsec: 0.06453243531730304,
            pm_y_arcsec: 0.4892744485064933,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57753_midnight",
            mjd_utc: 57753.0,
            delta_ut1_utc_s: -0.4077697,
            mjd_ut1: 57752.99999528044,
            pm_x_arcsec: 0.08144,
            pm_y_arcsec: 0.263099,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57753_quarter",
            mjd_utc: 57753.25,
            delta_ut1_utc_s: -0.408005525,
            mjd_ut1: 57753.24999817123,
            pm_x_arcsec: 0.08121724999999999,
            pm_y_arcsec: 0.26310625,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57753_midday",
            mjd_utc: 57753.5,
            delta_ut1_utc_s: -0.40824135,
            mjd_ut1: 57753.50000106202,
            pm_x_arcsec: 0.0809945,
            pm_y_arcsec: 0.2631135,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57753_three_quarter",
            mjd_utc: 57753.75,
            delta_ut1_utc_s: -0.408477175,
            mjd_ut1: 57753.75000395281,
            pm_x_arcsec: 0.08077175,
            pm_y_arcsec: 0.26312075,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57754_midnight",
            mjd_utc: 57754.0,
            delta_ut1_utc_s: 0.591287,
            mjd_ut1: 57754.0000068436,
            pm_x_arcsec: 0.080549,
            pm_y_arcsec: 0.263128,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57754_midday",
            mjd_utc: 57754.5,
            delta_ut1_utc_s: 0.5907521,
            mjd_ut1: 57754.50000683741,
            pm_x_arcsec: 0.0804435,
            pm_y_arcsec: 0.263354,
        },
        AstropyUt1Case {
            label: "leap_2016_mjd_57755_midnight",
            mjd_utc: 57755.0,
            delta_ut1_utc_s: 0.5902172,
            mjd_ut1: 57755.000006831215,
            pm_x_arcsec: 0.080338,
            pm_y_arcsec: 0.26358,
        },
        AstropyUt1Case {
            label: "leap_2015_mjd_57203_midnight",
            mjd_utc: 57203.0,
            delta_ut1_utc_s: -0.6760308,
            mjd_ut1: 57202.999992175566,
            pm_x_arcsec: 0.140851,
            pm_y_arcsec: 0.448918,
        },
        AstropyUt1Case {
            label: "leap_2015_mjd_57203_midday",
            mjd_utc: 57203.5,
            delta_ut1_utc_s: -0.67633325,
            mjd_ut1: 57203.49999795911,
            pm_x_arcsec: 0.141516,
            pm_y_arcsec: 0.4485285,
        },
        AstropyUt1Case {
            label: "leap_2015_mjd_57204_midnight",
            mjd_utc: 57204.0,
            delta_ut1_utc_s: 0.3233643,
            mjd_ut1: 57204.00000374264,
            pm_x_arcsec: 0.142181,
            pm_y_arcsec: 0.448139,
        },
        AstropyUt1Case {
            label: "leap_2012_mjd_56108_midnight",
            mjd_utc: 56108.0,
            delta_ut1_utc_s: -0.5868284,
            mjd_ut1: 56107.999993208,
            pm_x_arcsec: 0.092807,
            pm_y_arcsec: 0.409396,
        },
        AstropyUt1Case {
            label: "leap_2012_mjd_56108_midday",
            mjd_utc: 56108.5,
            delta_ut1_utc_s: -0.58678715,
            mjd_ut1: 56108.49999899552,
            pm_x_arcsec: 0.0934465,
            pm_y_arcsec: 0.409301,
        },
        AstropyUt1Case {
            label: "leap_2012_mjd_56109_midnight",
            mjd_utc: 56109.0,
            delta_ut1_utc_s: 0.4132541,
            mjd_ut1: 56109.00000478303,
            pm_x_arcsec: 0.094086,
            pm_y_arcsec: 0.409206,
        },
    ];

    #[test]
    fn test_dut1_vs_astropy() {
        let provider = load_c04();
        for c in &TEST_CASES {
            let rust = provider
                .eop_offset(c.mjd_utc)
                .expect("MJD should be in C04 table");
            assert_close(
                rust.offset,
                c.delta_ut1_utc_s,
                1e-12,
                &format!("{} DUT1", c.label),
            );
        }
    }

    #[test]
    fn test_polar_motion_vs_astropy() {
        let provider = load_c04();
        for c in &TEST_CASES {
            let rust = provider
                .eop_offset(c.mjd_utc)
                .expect("MJD should be in C04 table");
            assert_close(
                rust.pm_x,
                c.pm_x_arcsec,
                1e-12,
                &format!("{} pm_x", c.label),
            );
            assert_close(
                rust.pm_y,
                c.pm_y_arcsec,
                1e-12,
                &format!("{} pm_y", c.label),
            );
        }
    }

    /// Compare [`Dt::utc_mjd_to_ut1_mjd`] to Astropy's `ut1.mjd`.
    ///
    /// On leap-insertion interiors the UTC day is 86401 s, so this is not
    /// `mjd + DUT1/86400`.
    #[test]
    fn test_ut1_mjd_vs_astropy() {
        use deep_time::Dt;

        let provider = load_c04();
        for c in &TEST_CASES {
            let rust_ut1_mjd =
                Dt::utc_mjd_to_ut1_mjd(c.mjd_utc, &provider).expect("MJD should be in C04 table");
            assert_close(
                rust_ut1_mjd,
                c.mjd_ut1,
                1e-15,
                &format!("{} UT1 MJD", c.label),
            );
        }
    }
}
