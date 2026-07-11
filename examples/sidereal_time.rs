//! Sidereal time the way an Astropy tutorial would show it.
//!
//! Mirrors the classic Astropy demo
//!
//! <https://docs.astropy.org/en/stable/time/index.html#location>
//!
//! ```text
//! from astropy.time import Time
//! t = Time('2001-03-22 00:01:44.732327132980', scale='utc',
//!          location=('120d', '40d'))
//! t.sidereal_time('apparent', 'greenwich')  # ≈ 12 hourangle
//! t.sidereal_time('apparent')               # ≈ 20 hourangle
//! ```
//!
//! Loads IERS finals (same file as the library tests), converts UTC → UT1
//! with DUT1, then checks **mean** and **apparent** sidereal time at
//! Greenwich and at 120° E — plus a sample hour angle.
//!
//! ## Agreement with Astropy (measured on this instant)
//!
//! With `tests/assets/finals.all.iau2000.txt` vs Astropy’s default IERS
//! tables, residuals on this date are roughly:
//!
//! | Quantity | \|ours − Astropy\| |
//! |----------|--------------------|
//! | UTC MJD  | exact              |
//! | DUT1     | ~0.7 µs            |
//! | ERA      | ~0.7 µs (as time)  |
//! | GAST/LAST (apparent) | ~0.5 µs |
//! | GMST/LMST (mean)     | ~2.5 µs |
//!
//! Most of the gap is DUT1 / IERS table differences, not the EO model.
//! Against the textbook “exactly 12h / 20h” labels, both libraries sit
//! about 2 µs off (the Astropy docs round to the hour).
//!
//! Quiet / assert-driven so it can run under full example test suites.
//!
//! ```text
//! cargo run --example sidereal_time --features "sidereal-earth eop std"
//! ```

use deep_time::eop::{EopData, EopFormat, Separator};
use deep_time::{Dt, DtErr, Scale, Sidereal};

/// Format sidereal seconds-since-midnight as `HH:MM:SS.sss` (hour angle style).
fn format_hms(sidereal_sec: f64) -> String {
    let s = sidereal_sec.rem_euclid(86_400.0);
    let h = (s / 3600.0).floor() as u32;
    let m = ((s % 3600.0) / 60.0).floor() as u32;
    let sec = s % 60.0;
    format!("{h:02}:{m:02}:{sec:06.3}")
}

/// Sidereal seconds → hourangle (hours in \[0, 24)).
fn to_hourangle(sidereal_sec: f64) -> f64 {
    sidereal_sec.rem_euclid(86_400.0) / 3600.0
}

/// `|a - b| < eps`, with a useful panic message.
fn assert_close(name: &str, got: f64, expect: f64, eps: f64) {
    let d = (got - expect).abs();
    assert!(
        d < eps,
        "{name}: got {got:.15} expect {expect:.15} |diff|={d:.3e} (eps={eps:.3e})"
    );
}

fn main() -> Result<(), DtErr> {
    // ── Instant (Astropy Time(..., scale='utc')) ─────────────────────────
    // 2001-03-22 00:01:44.732327132980 UTC — chosen so GAST ≈ 12h at
    // longitude 0° and LAST ≈ 20h at 120° E (Astropy docs).
    let frac_attos = Dt::sec_f_to_attos(0.732_327_132_980);
    let utc = Dt::from_ymd(2001, 3, 22, Scale::UTC, 0, 1, 44, Dt::to_u64(frac_attos));

    // ── UT1 via IERS finals (same asset / pattern as astropy_sidereal_tests) ─
    let eop = EopData::from_text_file(
        "tests/assets/finals.all.iau2000.txt",
        EopFormat::Finals2000A,
        Separator::Whitespace,
    )?;

    let mjd_utc = utc.to_mjd_f();
    let dut1 = Dt::mjd_to_eop_offset_f(mjd_utc, &eop)?; // seconds
    let mjd_ut1 = mjd_utc + dut1 / 86_400.0;

    // Equation of the Origins is evaluated on TT.
    // Match the test / ERFA convention: TT ≈ UT1 + 32.184 s.
    let mjd_tt = mjd_ut1 + 32.184 / 86_400.0;

    // ── Observatories (Astropy location=('120d', '40d')) ──────────────────
    let mut greenwich = Sidereal::EARTH;
    greenwich.longitude_rad = 0.0;

    let mut observer = Sidereal::EARTH;
    observer.longitude_rad = 120.0_f64.to_radians(); // east positive

    let eo_mean = greenwich.earth_eo_mean(mjd_tt);
    let eo_app = greenwich.earth_eo_apparent(mjd_tt);

    // ── Gather results ───────────────────────────────────────────────────
    let gmst_sec = greenwich.sidereal_time_mean(mjd_ut1, eo_mean);
    let lmst_sec = observer.local_sidereal_time_mean(mjd_ut1, eo_mean);
    let gast_sec = greenwich.sidereal_time_apparent(mjd_ut1, eo_app);
    let last_sec = observer.local_sidereal_time_apparent(mjd_ut1, eo_app);
    let era_rad = greenwich.rotation_angle(mjd_ut1);

    let gmst_h = to_hourangle(gmst_sec);
    let gast_h = to_hourangle(gast_sec);
    let lmst_h = to_hourangle(lmst_sec);
    let last_h = to_hourangle(last_sec);

    let gmst_hms = format_hms(gmst_sec);
    let gast_hms = format_hms(gast_sec);
    let lmst_hms = format_hms(lmst_sec);
    let last_hms = format_hms(last_sec);

    // 120° east = 8 hours of hour angle
    let lon_hours = (last_sec - gast_sec).rem_euclid(86_400.0) / 3600.0;

    // Hour angle of a source: HA = LAST − RA (RA = 18h → HA ≈ +2h).
    let ra_hours = 18.0;
    let ha_hours = (last_h - ra_hours).rem_euclid(24.0);
    let ha_signed = if ha_hours > 12.0 {
        ha_hours - 24.0
    } else {
        ha_hours
    };

    // ── Asserts ──────────────────────────────────────────────────────────
    //
    // Two layers:
    // 1. Regression goldens for *this* binary / finals file (sub-ns stable).
    // 2. Textbook Astropy labels (12h / 20h) within a few microseconds —
    //    matching measured Astropy agreement on this date (~0.5–2.5 µs).

    // Input path
    assert_close("MJD UTC", mjd_utc, 51_990.001_212_179_712, 1e-12);
    assert_close("DUT1", dut1, 0.034_573_559_110_934, 1e-12);
    assert!(mjd_ut1 > mjd_utc);

    // Display form at 1 ms (both we and Astropy round to these HMS strings).
    assert_eq!(gast_hms, "12:00:00.000");
    assert_eq!(last_hms, "20:00:00.000");
    assert_eq!(gmst_hms, "12:00:01.019");
    assert_eq!(lmst_hms, "20:00:01.019");

    // Regression goldens (seconds since sidereal midnight).
    assert_close("GAST", gast_sec, 43_200.000_002_072_287, 1e-9);
    assert_close("LAST", last_sec, 72_000.000_002_072_265, 1e-9);
    assert_close("GMST", gmst_sec, 43_201.019_315_904_887, 1e-9);
    assert_close("LMST", lmst_sec, 72_001.019_315_904_865, 1e-9);
    assert_close("ERA", era_rad, 3.141_393_975_849_745, 1e-15);

    // Textbook hourangles: within 5 µs of exact 12h / 20h.
    // (Astropy itself is ~2.6 µs off these round numbers on this instant.)
    const FIVE_US_AS_HOURS: f64 = 5e-6 / 3600.0;
    assert_close("GAST hourangle", gast_h, 12.0, FIVE_US_AS_HOURS);
    assert_close("LAST hourangle", last_h, 20.0, FIVE_US_AS_HOURS);
    // Mean is ~1.019 s after apparent → ~12.000283 h / 20.000283 h.
    assert_close("GMST hourangle", gmst_h, 12.000_283_143_306_913, 1e-12);
    assert_close("LMST hourangle", lmst_h, 20.000_283_143_306_905, 1e-12);

    // Geometry
    assert_close("LAST − GAST", lon_hours, 8.0, 1e-12);
    assert_close("HA (RA=18h)", ha_signed, 2.0, FIVE_US_AS_HOURS);

    // Bound vs Astropy-class agreement (see module docs). Leaves headroom
    // above the measured ~0.5 µs (apparent) / ~2.5 µs (mean) residuals.
    const ASTROPY_APPARENT_SEC: f64 = 43_200.000_002_614_826; // GAST from Astropy
    const ASTROPY_MEAN_SEC: f64 = 43_201.019_318_368_511; // GMST from Astropy
    assert_close("GAST vs Astropy", gast_sec, ASTROPY_APPARENT_SEC, 1e-6);
    assert_close("GMST vs Astropy", gmst_sec, ASTROPY_MEAN_SEC, 5e-6);

    Ok(())
}
