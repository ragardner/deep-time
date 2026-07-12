#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

//! Integration tests for Earth Orientation Parameters → UT1 conversion.
//!
//! These exercise:
//! - Finals2000A / C04 parsers against known table rows
//! - `to_eop` applying DUT1 (and interpolated DUT1)
//! - `from_eop` fixed-point inversion (including fractional MJD)
//!
//! Pure JD/MJD serialization is covered by `julian_date_tests.rs` and is
//! not re-tested here.
//!
//! ## Precision notes
//!
//! At whole-day table epochs, `to_eop` / `from_eop` are exact in attoseconds
//! relative to `Dt::from_sec_f(dut1)`, and `to_sec_f` recovers the table DUT1
//! bit-identically. Prefer `assert_eq!` (and attos equality) over loose float
//! tolerances. A residual tolerance is only needed where f64 arithmetic is
//! inherently lossy (e.g. reconstructing a total Julian Date as `days + frac`).

#[cfg(all(feature = "eop", feature = "std"))]
mod tests {
    use deep_time::consts::{ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, SEC_PER_DAY_F};
    use deep_time::eop::{EopData, EopFormat, Separator};
    use deep_time::{Dt, Scale};

    fn load_finals2000a() -> EopData {
        let path = "tests/assets/finals.all.iau2000.txt";
        EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace)
            .expect("failed to load finals.all.iau2000.txt")
    }

    fn load_c04() -> EopData {
        let path = "tests/assets/EOP_20u24_C04_one_file_1962-now.txt";
        EopData::from_text_file(path, EopFormat::C04, Separator::Whitespace)
            .expect("failed to load C04 EOP file")
    }

    /// Build a `Dt` whose raw MJD equals `mjd_days + frac_attos/day`.
    ///
    /// Uses `Scale::TAI` so `from_mjd` does not apply leap-second conversion;
    /// that keeps `to_mjd_f_raw()` on the table epoch (EOP rows are indexed
    /// by UTC MJD at whole days).
    fn at_mjd(mjd_days: i128, frac_attos: i128) -> Dt {
        Dt::from_mjd(mjd_days, frac_attos, Scale::TAI)
    }

    /// Assert `|got - expected| <= tol`.
    ///
    /// `<=` (not `<`) is intentional: "within tolerance" includes the boundary,
    /// and `tol == 0.0` correctly means bit-identical equality. Prefer
    /// `assert_eq!` when you already know the values should match exactly.
    fn assert_close(got: f64, expected: f64, tol: f64, label: &str) {
        let err = (got - expected).abs();
        assert!(
            err <= tol,
            "{label}: got {got}, expected {expected}, |err|={err} (tol {tol})"
        );
    }

    /// `to_eop` must shift the instant by exactly `from_sec_f(dut1)` attos,
    /// and that shift must read back as the same f64 seconds.
    fn assert_to_eop_applies_dut1(before: Dt, after: Dt, dut1: f64, label: &str) {
        let expected_attos = Dt::from_sec_f(dut1, Scale::TAI, Scale::TAI).attos;
        assert_eq!(
            after.attos - before.attos,
            expected_attos,
            "{label}: attos shift mismatch"
        );
        assert_eq!(
            after.to_diff_raw(before).to_sec_f(),
            dut1,
            "{label}: to_sec_f did not recover DUT1 bit-identically"
        );
    }

    fn assert_roundtrip_exact(original: Dt, back: Dt, label: &str) {
        assert_eq!(
            original.attos,
            back.attos,
            "{label}: round-trip attos mismatch (diff {} as)",
            original.attos - back.attos
        );
    }

    // ------------------------------------------------------------------
    // Finals2000A: known rows → parse, apply DUT1, invert
    // ------------------------------------------------------------------
    #[test]
    fn test_finals2000a_known_rows_to_eop_from_eop() {
        let provider = load_finals2000a();

        // (MJD, DUT1 seconds) taken from finals.all.iau2000.txt
        let cases = [(56879_i128, -0.3170554), (60961, 0.0933562)];

        for &(mjd, dut1_expected) in &cases {
            let dut1 = provider
                .eop_offset(mjd as f64)
                .expect("MJD should be in Finals2000A table")
                .offset;
            assert_eq!(
                dut1, dut1_expected,
                "parser DUT1 mismatch at MJD {mjd}: got {dut1}, expected {dut1_expected}"
            );

            let utc = at_mjd(mjd, 0);
            let ut1 = utc.to_eop(&provider).expect("to_eop failed");
            assert_to_eop_applies_dut1(utc, ut1, dut1_expected, &format!("to_eop at MJD {mjd}"));

            let back = ut1.from_eop(&provider).expect("from_eop failed");
            assert_roundtrip_exact(utc, back, &format!("from_eop at MJD {mjd}"));
        }
    }

    // ------------------------------------------------------------------
    // C04: known row → parse, apply DUT1, invert
    // ------------------------------------------------------------------
    #[test]
    fn test_c04_known_row_to_eop_from_eop() {
        let provider = load_c04();

        let mjd = 57259_i128;
        let dut1_expected = 0.2813082;

        let dut1 = provider
            .eop_offset(mjd as f64)
            .expect("MJD should be in C04 table")
            .offset;
        assert_eq!(
            dut1, dut1_expected,
            "C04 parser DUT1 mismatch: got {dut1}, expected {dut1_expected}"
        );

        let utc = at_mjd(mjd, 0);
        let ut1 = utc.to_eop(&provider).expect("to_eop failed");
        assert_to_eop_applies_dut1(utc, ut1, dut1_expected, "C04 to_eop");

        let back = ut1.from_eop(&provider).expect("from_eop failed");
        assert_roundtrip_exact(utc, back, "C04 from_eop");
    }

    // ------------------------------------------------------------------
    // UT1 − UTC in Julian-day space equals DUT1 / 86400
    //
    // Total JD is rebuilt as f64 (`days + frac_attos/day`), so this path is
    // lossy (~2e-10 day residual for this epoch). That is the only place in
    // this file that needs a non-zero float tolerance.
    // ------------------------------------------------------------------
    #[test]
    fn test_to_eop_shifts_jd_by_dut1() {
        let provider = load_finals2000a();
        let dut1_expected = -0.3170554;

        let utc = at_mjd(56879, 0);
        let ut1 = utc.to_eop(&provider).expect("to_eop failed");

        // Exact integer check first: JD components encode the same attos shift
        // that `to_eop` applied.
        let (jd_ut1, frac_ut1) = ut1.to_jd();
        let (jd_utc, frac_utc) = utc.to_jd();
        let total_ut1 = jd_ut1
            .saturating_mul(ATTOS_PER_DAY)
            .saturating_add(frac_ut1);
        let total_utc = jd_utc
            .saturating_mul(ATTOS_PER_DAY)
            .saturating_add(frac_utc);
        assert_eq!(
            total_ut1 - total_utc,
            Dt::from_sec_f(dut1_expected, Scale::TAI, Scale::TAI).attos,
            "integer JD attos shift should equal DUT1 attos"
        );

        // f64 total-JD view (lossy): keep a tight, measured bound.
        let total_jd_ut1 = jd_ut1 as f64 + (frac_ut1 as f64) / (ATTOS_PER_DAY as f64);
        let total_jd_utc = jd_utc as f64 + (frac_utc as f64) / (ATTOS_PER_DAY as f64);
        let diff_days = total_jd_ut1 - total_jd_utc;
        let expected_diff = dut1_expected / SEC_PER_DAY_F;
        // Observed residual ≈ 2.1e-10 days; 1e-9 leaves a small margin without
        // re-introducing the old multi-order-of-magnitude slack.
        assert_close(diff_days, expected_diff, 1e-9, "f64 JD(UT1) − JD(UTC)");
    }

    // ------------------------------------------------------------------
    // Linear interpolation between consecutive EOP rows (midday)
    // ------------------------------------------------------------------
    #[test]
    fn test_eop_linear_interpolation_midday() {
        let provider = load_finals2000a();

        // Adjacent Finals rows:
        //   56879.00  DUT1 = -0.3170554
        //   56880.00  DUT1 = -0.3176567
        let e0 = provider.eop_offset(56879.0).expect("MJD 56879 in table");
        let e1 = provider.eop_offset(56880.0).expect("MJD 56880 in table");
        let mid = provider
            .eop_offset(56879.5)
            .expect("MJD 56879.5 should interpolate");

        // Midpoint `0.5 * (a + b)` is exact for these table values.
        let expected_offset = 0.5 * (e0.offset + e1.offset);
        let expected_pm_x = 0.5 * (e0.pm_x + e1.pm_x);
        let expected_pm_y = 0.5 * (e0.pm_y + e1.pm_y);

        assert_eq!(mid.offset, expected_offset, "interpolated DUT1");
        assert_eq!(mid.pm_x, expected_pm_x, "interpolated pm_x");
        assert_eq!(mid.pm_y, expected_pm_y, "interpolated pm_y");

        let utc = at_mjd(56879, ATTOS_PER_HALF_DAY);
        let ut1 = utc.to_eop(&provider).expect("to_eop at midday failed");
        assert_to_eop_applies_dut1(utc, ut1, expected_offset, "to_eop midday");
    }

    // ------------------------------------------------------------------
    // from_eop fixed-point iteration at fractional MJD
    // ------------------------------------------------------------------
    #[test]
    fn test_from_eop_roundtrip_fractional_day() {
        let provider = load_finals2000a();

        let utc = at_mjd(60961, ATTOS_PER_HALF_DAY);
        let ut1 = utc.to_eop(&provider).expect("to_eop failed");
        let back = ut1.from_eop(&provider).expect("from_eop failed");
        assert_roundtrip_exact(utc, back, "from_eop fractional-day");

        // Forward offset is the interpolated value, not the midnight entry.
        let mjd = utc.to_mjd_f_raw();
        let expected = provider.eop_offset(mjd).expect("in range").offset;
        assert_to_eop_applies_dut1(utc, ut1, expected, "to_eop fractional");
        assert_ne!(
            expected, 0.0933562,
            "midday DUT1 should differ from the midnight table entry"
        );
    }
}
