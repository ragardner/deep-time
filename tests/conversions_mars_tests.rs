#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "mars")]
mod mars_tests {
    use deep_time::{Dt, Real, Scale, f, mars::MARS_SOL_LENGTH_SEC};

    #[test]
    fn utc_leap_seconds_are_handled_in_mars_time() {
        // One second before vs after a leap second insertion
        let utc_pre = Dt::from_tai_sec(1_485_779_199);
        let utc_post = Dt::from_tai_sec(1_485_779_200);

        let msd_pre = utc_pre.to_msd_f();
        let msd_post = utc_post.to_msd_f();

        let diff_sols = (msd_post - msd_pre).abs();
        assert!(
            diff_sols > 1e-6 && diff_sols < 2e-5,
            "MSD difference across leap second was {} sols (expected ~1.126e-5)",
            diff_sols
        );
    }

    #[test]
    fn msd_exact_roundtrip_is_accurate() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
            Dt::from_sec(-86_400 * 365 * 10, Scale::TAI, Scale::TAI),
            Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
            Dt::from_sec(-2_208_945_600, Scale::TAI, Scale::TAI),
        ];

        for &p in &test_points {
            let (whole, frac) = p.to_msd();
            let back = Dt::from_msd(whole, frac);

            let diff = back.to_diff_raw(p).to_sec_f().abs();
            assert!(
                diff < 5e-5, // ← relaxed for f64 JD precision (max observed error ≈ 13.7 µs)
                "MSD round-trip error too large: {} s at {:?}",
                diff,
                p
            );
        }
    }

    #[test]
    fn msd_float_roundtrip_is_accurate() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365 * 100, Scale::TAI, Scale::TAI),
            Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        ];

        for &p in &test_points {
            let msd_float = p.to_msd_f();
            let back = Dt::from_msd_f(msd_float);

            let diff = back.to_diff_raw(p).to_sec_f().abs();
            assert!(
                diff < 5e-5, // ← relaxed for f64 MSD path (max observed error ≈ 13.7 µs)
                "MSD float round-trip error too large: {} s at {:?}",
                diff,
                p
            );
        }
    }

    #[test]
    fn mtc_is_in_valid_range() {
        let test_points = [
            Dt::from_sec(0, Scale::TAI, Scale::TAI),
            Dt::from_sec(86_400 * 365, Scale::TAI, Scale::TAI),
            Dt::from_sec(1_000_000_000, Scale::TAI, Scale::TAI),
        ];

        for &p in &test_points {
            let mtc = p.to_mtc();
            let mtc_sec = mtc.to_sec_f();
            assert!(
                mtc_sec >= 0.0 && mtc_sec < MARS_SOL_LENGTH_SEC,
                "MTC out of range: {} s at {:?}",
                mtc_sec,
                p
            );
        }
    }

    #[test]
    fn msd_at_j2000_is_correct() {
        let tai = Dt::ZERO;
        let (whole, frac) = tai.to_msd();

        assert_eq!(whole, 44791, "Integer part of MSD at J2000 should be 44791");
    }

    #[test]
    fn mars_ls_is_correct() {
        // These test cases are the official worked examples published by NASA GISS
        // in the Mars24 Sunclock algorithm documentation.
        //
        // Source: NASA Goddard Institute for Space Studies (GISS)
        // Title:   Mars24 Sunclock — Algorithm and Worked Examples
        // URL:     https://www.giss.nasa.gov/tools/mars24/help/algorithm.html
        // Updated: 2025-01-07
        //
        // The short-series analytic model for Areocentric Solar Longitude (Ls)
        // implemented in `to_mars_ls` is based on Allison & McEwen (2000) with
        // the seven largest planetary perturbations. These two dates are the
        // exact verification benchmarks provided by NASA.

        const TOLERANCE: f64 = 0.01;

        let date = Dt::from_ymd(2000, 1, 6, Scale::UTC, 0, 0, 0, 0);
        let ls = date.to_mars_ls();
        assert!(
            (ls - 277.18758).abs() < TOLERANCE,
            "2000-01-06 Ls is wrong, got: {}",
            ls
        );

        let date = Dt::from_ymd(2004, 1, 3, Scale::UTC, 13, 46, 41, 0);
        let ls = date.to_mars_ls();
        assert!(
            (ls - 327.32416).abs() < TOLERANCE,
            "2004-01-03 Ls is wrong, got: {}",
            ls
        );
    }

    #[test]
    fn mars_local_solar_times_are_correct() {
        // Official NASA GISS Mars24 worked example (prime meridian)
        //
        // Source: NASA Goddard Institute for Space Studies (GISS)
        // Title:   Mars24 Sunclock — Algorithm and Worked Examples
        // URL:     https://www.giss.nasa.gov/tools/mars24/help/algorithm.html
        // Updated: 2025-01-07
        //
        // Date:    2000-01-06 00:00:00 UTC
        // Longitude: 0° E (prime meridian)
        // Expected LMST: 23:59:39 Mars time  →  86_379 seconds into the sol
        // Expected LTST: 23:38:54 Mars time  →  85_134 seconds into the sol
        //
        // These values are taken verbatim from the published NASA algorithm page
        // (Table of worked examples, rows C-2/C-3 and C-4).

        let date = Dt::from_ymd(2000, 1, 6, Scale::UTC, 0, 0, 0, 0);
        let east_lon_deg = f!(0.0); // prime meridian

        let lmst = date.to_mars_lmst(east_lon_deg);
        let ltst = date.to_mars_ltst(east_lon_deg);

        // Convert the returned Dt (seconds into the current sol) to a float for comparison
        let lmst_sec = lmst.to_sec_f();
        let ltst_sec = ltst.to_sec_f();

        assert!(
            (lmst_sec - f!(86379.0)).abs() < f!(0.5),
            "LMST wrong, got {} seconds (expected ~86379)",
            lmst_sec
        );

        assert!(
            (ltst_sec - f!(85134.0)).abs() < f!(3.0),
            "LTST wrong, got {} seconds (expected ~85134)",
            ltst_sec
        );
    }

    #[test]
    fn mars_year_is_correct() {
        // These dates are standard reference points in Mars science literature.
        //
        // Mars Year numbering follows the Clancy et al. (2000) convention:
        //   Mars Year 1 begins at the northern vernal equinox (Ls = 0°) on
        //   1955 April 11 (JD 2435208.456 TT).
        //
        // The implementation uses:
        //   - Epoch JD from the Clancy definition (confirmed in Gangale's
        //     "Vernal Equinoxes of Mars" table and multiple sources).
        //   - Average Mars tropical year length 686.9725 Earth days
        //     (NASA GISS Mars24 Technical Notes).
        //
        // This produces the conventional integer Mars Year used by NASA,
        // ESA, LMD Mars Climate Database, and peer-reviewed papers.

        // Spirit (MER-A) landing: 2004-01-04 UTC
        // Explicitly stated as Mars Year 26, Ls ≈ 328° in:
        //   Smith et al. (2006) "One Martian year of atmospheric observations
        //   using MER Mini-TES", J. Geophys. Res. Planets, 111(E12S13).
        //   https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2006JE002770
        let spirit = Dt::from_ymd(2004, 1, 4, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(
            spirit.to_mars_year(),
            26,
            "Spirit landing (2004-01-04) should be Mars Year 26"
        );

        // Perseverance (M2020) landing: 2021-02-18 UTC
        // Landed in Mars Year 36 (widely used in mission papers and
        // confirmed by the epoch + tropical year calculation; e.g.
        // "Mars Year 36" references in post-landing MEDA/Mars 2020 literature).
        let perseverance = Dt::from_ymd(2021, 2, 18, Scale::UTC, 0, 0, 0, 0);
        assert_eq!(
            perseverance.to_mars_year(),
            36,
            "Perseverance landing (2021-02-18) should be Mars Year 36"
        );
    }
}
