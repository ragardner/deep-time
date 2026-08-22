#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

//! Integration tests for Earth Orientation Parameters → UT1 conversion.
//!
//! These exercise:
//! - Finals2000A / C04 parsers against known table rows
//! - `to_ut1` applying DUT1 (and interpolated DUT1)
//! - `from_ut1` fixed-point inversion (including fractional MJD)
//!
//! Pure JD/MJD serialization is covered by `julian_date_tests.rs` and is
//! not re-tested here.
//!
//! ## Precision notes
//!
//! At whole-day table epochs, `to_ut1` / `from_ut1` are exact in attoseconds
//! relative to `Dt::from_sec_f(dut1)`, and `to_sec_f` recovers the table DUT1
//! bit-identically. Prefer `assert_eq!` (and attos equality) over loose float
//! tolerances. A float allowance is only needed where f64 arithmetic is
//! inherently lossy (e.g. reconstructing a total Julian Date as `days + frac`).

#[cfg(all(feature = "eop", feature = "std"))]
mod tests {
    use deep_time::consts::{ATTOS_PER_DAY, ATTOS_PER_HALF_DAY, SEC_PER_DAY_F};
    use deep_time::eop::{EopData, EopDataRow, EopFormat, Separator};
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

    fn load_jpl_eop2() -> EopData {
        let path = "tests/assets/latest_eop2.short";
        EopData::from_text_file(path, EopFormat::JplEop2, Separator::Comma)
            .expect("failed to load JPL EOP2 file")
    }

    /// Instant from a UTC MJD: `from_mjd` converts UTC → TAI.
    ///
    /// [`Dt::to_ut1`] on IERS data converts back to UTC for lookup, so this
    /// MJD is the table key. DUT1 is added to the UTC clock (`dt.to(UTC)`),
    /// not to the TAI attoseconds this returns.
    fn at_utc_mjd(mjd_days: i128, frac_attos: i128) -> Dt {
        Dt::from_mjd(mjd_days, frac_attos, Scale::UTC)
    }

    /// Instant from a TAI MJD (no scale conversion). JPL EOP2 table key.
    fn at_tai_mjd(mjd_days: i128, frac_attos: i128) -> Dt {
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

    /// `to_ut1` must shift the instant by exactly `from_sec_f(dut1)` attos,
    /// and that shift must read back as the same f64 seconds.
    fn assert_to_ut1_applies_dut1(before: Dt, after: Dt, dut1: f64, label: &str) {
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
    fn test_finals2000a_known_rows_to_ut1_from_ut1() {
        let provider = load_finals2000a();
        assert!(provider.strip_offset_leaps, "IERS Finals should leap-strip");
        assert_eq!(provider.epoch_scale, Scale::UTC);

        // (MJD, DUT1 seconds) — Bulletin B columns when present (fixed-width
        // Finals2000A parser prefers B over A, matching Astropy IERS_A).
        let cases = [(56879_i128, -0.3170662), (60961, 0.0933544)];

        for &(mjd, dut1_expected) in &cases {
            let dut1 = provider
                .eop_offset(mjd as f64)
                .expect("MJD should be in Finals2000A table")
                .offset;
            assert_eq!(
                dut1, dut1_expected,
                "parser DUT1 mismatch at MJD {mjd}: got {dut1}, expected {dut1_expected}"
            );

            let dt = at_utc_mjd(mjd, 0);
            let utc = dt.to(Scale::UTC);
            let ut1 = dt.to_ut1(&provider).expect("to_ut1 failed");
            assert_to_ut1_applies_dut1(utc, ut1, dut1_expected, &format!("to_ut1 at MJD {mjd}"));

            let back = ut1.from_ut1(&provider).expect("from_ut1 failed");
            assert_roundtrip_exact(utc, back, &format!("from_ut1 at MJD {mjd}"));
        }
    }

    // ------------------------------------------------------------------
    // C04: known row → parse, apply DUT1, invert
    // ------------------------------------------------------------------
    #[test]
    fn test_c04_known_row_to_ut1_from_ut1() {
        let provider = load_c04();
        assert!(provider.strip_offset_leaps, "IERS C04 should leap-strip");
        assert_eq!(provider.epoch_scale, Scale::UTC);

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

        let dt = at_utc_mjd(mjd, 0);
        let utc = dt.to(Scale::UTC);
        let ut1 = dt.to_ut1(&provider).expect("to_ut1 failed");
        assert_to_ut1_applies_dut1(utc, ut1, dut1_expected, "C04 to_ut1");

        let back = ut1.from_ut1(&provider).expect("from_ut1 failed");
        assert_roundtrip_exact(utc, back, "C04 from_ut1");
    }

    // ------------------------------------------------------------------
    // UT1 − UTC in Julian-day space equals DUT1 / 86400
    //
    // Total JD is rebuilt as f64 (`days + frac_attos/day`), so this path is
    // lossy (~2e-10 day difference for this epoch). That is the only place in
    // this file that needs a non-zero float tolerance.
    // ------------------------------------------------------------------
    #[test]
    fn test_to_ut1_shifts_jd_by_dut1() {
        let provider = load_finals2000a();
        let dut1_expected = -0.3170662; // Bulletin B at MJD 56879

        let dt = at_utc_mjd(56879, 0);
        let utc = dt.to(Scale::UTC);
        let ut1 = dt.to_ut1(&provider).expect("to_ut1 failed");

        // Exact integer check first: JD components encode the same attos shift
        // that `to_ut1` applied.
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
        // f64 total-JD is lossy; difference is ~2e-10 days, allow 1e-9.
        assert_close(diff_days, expected_diff, 1e-9, "f64 JD(UT1) − JD(UTC)");
    }

    // ------------------------------------------------------------------
    // Linear interpolation between consecutive EOP rows (midday)
    // ------------------------------------------------------------------
    #[test]
    fn test_eop_linear_interpolation_midday() {
        let provider = load_finals2000a();

        // Adjacent Finals rows (Bulletin B):
        //   56879.00  DUT1 = -0.3170662
        //   56880.00  DUT1 = -0.3176599
        let e0 = provider.eop_offset(56879.0).expect("MJD 56879 in table");
        let e1 = provider.eop_offset(56880.0).expect("MJD 56880 in table");
        let mid = provider
            .eop_offset(56879.5)
            .expect("MJD 56879.5 should interpolate");

        // No leap second between these days → plain linear midpoint.
        // (Offset path still applies leap-strip; round(Δ) == 0 here.)
        let expected_offset = 0.5 * (e0.offset + e1.offset);
        let expected_pm_x = 0.5 * (e0.pm_x + e1.pm_x);
        let expected_pm_y = 0.5 * (e0.pm_y + e1.pm_y);

        assert_eq!(mid.offset, expected_offset, "interpolated DUT1");
        assert_eq!(mid.pm_x, expected_pm_x, "interpolated pm_x");
        assert_eq!(mid.pm_y, expected_pm_y, "interpolated pm_y");

        let dt = at_utc_mjd(56879, ATTOS_PER_HALF_DAY);
        let utc = dt.to(Scale::UTC);
        let ut1 = dt.to_ut1(&provider).expect("to_ut1 at midday failed");
        assert_to_ut1_applies_dut1(utc, ut1, expected_offset, "to_ut1 midday");
    }

    // ------------------------------------------------------------------
    // Leap-second day: strip integer jump before interpolating UT1−UTC
    // ------------------------------------------------------------------
    #[test]
    fn test_eop_leap_second_day_interpolation() {
        // Use C04 (matches Astropy IERS_B / eopc04 definitive values).
        let provider = load_c04();

        // 2016-12-31 leap: MJD 57753 → 57754 jumps by ~+1 s in UT1−UTC.
        let e0 = provider.eop_offset(57753.0).expect("57753");
        let e1 = provider.eop_offset(57754.0).expect("57754");
        let mid = provider.eop_offset(57753.5).expect("57753.5");

        let d = e1.offset - e0.offset;
        assert!(
            (d - 1.0).abs() < 0.01,
            "expected ~1 s leap jump in table DUT1, got {d}"
        );

        // Blend after removing the nearest integer from the day-to-day Δ.
        let d_stripped = d - d.round();
        let expected = e0.offset + 0.5 * d_stripped;
        assert_eq!(mid.offset, expected, "leap-stripped midday DUT1");

        // Straight midpoint of the table endpoints is off by about half a second.
        let plain_mid = 0.5 * (e0.offset + e1.offset);
        assert!(
            (mid.offset - plain_mid).abs() > 0.4,
            "leap strip should move midday far from plain midpoint"
        );
    }

    // ------------------------------------------------------------------
    // from_ut1 fixed-point iteration at fractional MJD
    // ------------------------------------------------------------------
    #[test]
    fn test_from_ut1_roundtrip_fractional_day() {
        let provider = load_finals2000a();

        let dt = at_utc_mjd(60961, ATTOS_PER_HALF_DAY);
        let utc = dt.to(Scale::UTC);
        let ut1 = dt.to_ut1(&provider).expect("to_ut1 failed");
        let back = ut1.from_ut1(&provider).expect("from_ut1 failed");
        assert_roundtrip_exact(utc, back, "from_ut1 fractional-day");

        // Forward offset is the interpolated value, not the midnight entry.
        let mjd = utc.to_mjd_f_raw();
        let expected = provider.eop_offset(mjd).expect("in range").offset;
        assert_to_ut1_applies_dut1(utc, ut1, expected, "to_ut1 fractional");
        assert_ne!(
            expected, 0.0933544,
            "midday DUT1 should differ from the midnight table entry (Bulletin B)"
        );
    }

    // ------------------------------------------------------------------
    // strip_offset_leaps table policy
    // ------------------------------------------------------------------
    #[test]
    fn test_strip_offset_leaps_off_uses_plain_midpoint() {
        let provider = load_c04().with_strip_offset_leaps(false);

        let e0 = provider.eop_offset(57753.0).expect("57753");
        let e1 = provider.eop_offset(57754.0).expect("57754");
        let mid = provider.eop_offset(57753.5).expect("57753.5");

        let plain_mid = 0.5 * (e0.offset + e1.offset);
        assert_eq!(
            mid.offset, plain_mid,
            "with leap strip off, midday is plain linear midpoint"
        );
    }

    /// Generic `to_eop(data, epoch)` matches `to_ut1` when epoch is raw MJD.
    #[test]
    fn test_to_eop_with_mjd_epoch_matches_to_ut1() {
        let provider = load_c04();
        let dt = at_utc_mjd(56879, 0);
        let utc = dt.to(Scale::UTC);
        let via_ut1 = dt.to_ut1(&provider).expect("to_ut1");
        let via_eop = utc.to_eop(&provider, utc.to_mjd_f_raw()).expect("to_eop");
        assert_eq!(via_ut1.attos, via_eop.attos);
    }

    /// Custom table whose epoch is not this `Dt`’s MJD: `from_eop` needs the
    /// table epoch that `to_eop` used. `from_ut1` looks up at TAI MJD and is
    /// the wrong inverse here.
    #[test]
    fn test_from_rows_to_eop_from_eop_roundtrip() {
        let table = EopData::from_rows(
            vec![
                EopDataRow {
                    epoch: 10.0,
                    offset: 1.25,
                    pm_x: 0.0,
                    pm_y: 0.0,
                },
                EopDataRow {
                    epoch: 12.0,
                    offset: 1.75,
                    pm_x: 0.0,
                    pm_y: 0.0,
                },
            ],
            false,
        );
        assert_eq!(table.epoch_scale, Scale::TAI);

        let dt = at_tai_mjd(60_000, 0);
        let table_epoch = 11.0; // midpoint → 1.5 s
        let expected = 0.5 * (1.25 + 1.75);

        let applied = dt.to_eop(&table, table_epoch).expect("to_eop");
        assert_to_ut1_applies_dut1(dt, applied, expected, "to_eop custom table");

        let back = applied.from_eop(&table, table_epoch).expect("from_eop");
        assert_roundtrip_exact(dt, back, "from_eop custom table");

        // Table epochs 10–12; this Dt’s TAI MJD is 60000. from_ut1 looks up
        // at 60000 and clamps to the last row — not the inverse.
        let via_ut1 = applied.from_ut1(&table).expect("from_ut1");
        assert_ne!(
            via_ut1.attos, dt.attos,
            "from_ut1 must not round-trip a table whose epoch is not this Dt’s MJD"
        );
    }

    // ------------------------------------------------------------------
    // merge(add_rows, overwrite_rows)
    // ------------------------------------------------------------------

    #[test]
    fn test_merge_overwrite_only() {
        let c04 = load_c04();
        let finals = load_finals2000a();
        let finals_late = finals.eop_offset(61_128.0).expect("Finals-only day");

        // Finals base, C04 overwrites shared MJDs; no new C04-only rows.
        let merged = finals.with_merge(&c04, false, true);

        let c04_shared = c04.eop_offset(56_879.0).expect("shared day");
        let merged_shared = merged.eop_offset(56_879.0).expect("shared day");
        assert_eq!(merged_shared.offset, c04_shared.offset);
        assert_eq!(merged_shared.pm_x, c04_shared.pm_x);
        assert_eq!(merged_shared.pm_y, c04_shared.pm_y);

        // Finals-only day (after C04 ends) kept with Finals values.
        let late = merged.eop_offset(61_128.0).expect("late day");
        assert_eq!(late.offset, finals_late.offset);
        assert_eq!(late.pm_x, finals_late.pm_x);
        assert_eq!(late.pm_y, finals_late.pm_y);
    }

    #[test]
    fn test_merge_add_only() {
        let c04 = load_c04();
        let finals = load_finals2000a();
        let c04_early = c04.eop_offset(37_665.0).expect("early C04");
        let c04_shared = c04.eop_offset(56_879.0).expect("shared day");
        let finals_late = finals.eop_offset(61_128.0).expect("Finals-only day");

        // C04 base, Finals adds new days only; shared days stay C04.
        let merged = c04.with_merge(&finals, true, false);

        let early = merged.eop_offset(37_665.0).expect("early day");
        assert_eq!(early.offset, c04_early.offset);
        assert_eq!(early.pm_x, c04_early.pm_x);
        assert_eq!(early.pm_y, c04_early.pm_y);

        let shared = merged.eop_offset(56_879.0).expect("shared day");
        assert_eq!(shared.offset, c04_shared.offset);

        let late = merged.eop_offset(61_128.0).expect("late day");
        assert_eq!(late.offset, finals_late.offset);
        assert_eq!(late.pm_x, finals_late.pm_x);
        assert_eq!(late.pm_y, finals_late.pm_y);
    }

    #[test]
    fn test_merge_add_and_overwrite() {
        let c04 = load_c04();
        let finals = load_finals2000a();
        let c04_early = c04.eop_offset(37_665.0).expect("early C04");
        let finals_shared = finals.eop_offset(56_879.0).expect("shared day");
        let finals_late = finals.eop_offset(61_128.0).expect("Finals-only day");

        // C04 base, Finals overwrites overlap and adds new days.
        let merged = c04.with_merge(&finals, true, true);

        let shared = merged.eop_offset(56_879.0).expect("shared day");
        assert_eq!(shared.offset, finals_shared.offset);
        assert_eq!(shared.pm_x, finals_shared.pm_x);
        assert_eq!(shared.pm_y, finals_shared.pm_y);

        let early = merged.eop_offset(37_665.0).expect("early day");
        assert_eq!(early.offset, c04_early.offset);

        let late = merged.eop_offset(61_128.0).expect("late day");
        assert_eq!(late.offset, finals_late.offset);
    }

    #[test]
    fn test_merge_noop() {
        let c04 = load_c04();
        let finals = load_finals2000a();
        let before = c04.eop_offset(56_879.0).expect("shared day");
        let c04_end = c04.eop_offset(61_127.0).expect("last C04 day");

        let merged = c04.with_merge(&finals, false, false);

        let after = merged.eop_offset(56_879.0).expect("shared day");
        assert_eq!(after.offset, before.offset);

        // Without add_rows, Finals-only day 61128 is not a table row;
        // lookup clamps to the last C04 day.
        let past_end = merged.eop_offset(61_128.0).expect("clamped");
        assert_eq!(past_end.offset, c04_end.offset);
    }

    // ------------------------------------------------------------------
    // JPL EOP2: TAI MJD; file TAI−UT1 (ms) stored as UT1−TAI (seconds)
    // ------------------------------------------------------------------
    #[test]
    fn test_jpl_eop2_parse_known_rows() {
        let provider = load_jpl_eop2();
        assert!(
            !provider.strip_offset_leaps,
            "JPL EOP2 should not leap-strip"
        );
        assert_eq!(provider.epoch_scale, Scale::TAI);
        assert_eq!(provider.rows.len(), 453);

        // File 36924.3602 ms TAI−UT1 → stored −36.9243602 s (UT1−TAI).
        let first = provider.eop_offset(60906.0).expect("first JPL row");
        assert_eq!(first.offset, -(36_924.3602 / 1000.0));
        assert_eq!(first.pm_x, 223.1863 / 1000.0);
        assert_eq!(first.pm_y, 406.0706 / 1000.0);

        let last = provider.eop_offset(61358.0).expect("last JPL row");
        assert_eq!(last.offset, -(37_076.4472 / 1000.0));
        assert_eq!(last.pm_x, 166.5971 / 1000.0);
        assert_eq!(last.pm_y, 292.5728 / 1000.0);
    }

    #[test]
    fn test_jpl_eop2_interpolation_and_to_ut1() {
        let provider = load_jpl_eop2();
        let e0 = provider.eop_offset(60906.0).expect("60906");
        let e1 = provider.eop_offset(60907.0).expect("60907");
        let mid = provider.eop_offset(60906.5).expect("60906.5");

        assert_eq!(mid.offset, 0.5 * (e0.offset + e1.offset));
        assert_eq!(mid.pm_x, 0.5 * (e0.pm_x + e1.pm_x));
        assert_eq!(mid.pm_y, 0.5 * (e0.pm_y + e1.pm_y));

        // TAI + (UT1−TAI) = UT1 (~36.9 s earlier, not later).
        let tai = at_tai_mjd(60906, 0);
        let ut1 = tai.to_ut1(&provider).expect("to_ut1 with JPL EOP2");
        assert_to_ut1_applies_dut1(tai, ut1, e0.offset, "JPL EOP2: TAI + (UT1−TAI) = UT1");
        assert!(
            ut1.attos < tai.attos,
            "UT1 must be before TAI for this JPL row"
        );
    }
}
