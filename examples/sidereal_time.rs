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
//! The published docs only print whole-hour labels (`12.` / `20.` hourangle).
//! HMS strings, difference table, and `ASTROPY_*` expected values below come from a
//! local Astropy measurement (script in the block comment after this module
//! doc), not from a web page.
//!
//! Loads IERS C04 EOP data, converts UTC → UT1 with DUT1, evaluates the
//! equation of the origins on TT, then checks **mean** and **apparent**
//! sidereal time at Greenwich and at 120° E — plus a sample hour angle.
//!
//! ## Agreement with Astropy (measured on this instant)
//!
//! DUT1 comes from the C04 series (Astropy’s definitive eopc04 / IERS B
//! path). EO uses TT via `utc.to(Scale::TT)`. Measured differences vs Astropy
//! on this instant are ~0.03 µs for GMST/GAST and ~0.03 µs (as time) for ERA.
//! Against the textbook “exactly 12h / 20h” labels, both libraries sit
//! about 2 µs off (the Astropy docs round to the hour).
//!
//! Quiet / assert-driven so it can run under full example test suites.
//!
//! ```text
//! cargo run --example sidereal_time --features "sidereal-earth eop std"
//! ```

/*
Astropy reference values for the asserts below (astropy 8.0.1).

The docs only show `<Longitude 12. hourangle>` / `<Longitude 20. hourangle>`.
Everything more precise (1 ms HMS, ASTROPY_* seconds, µs differences) was
measured with this script (Astropy + PyERFA):

```python
from astropy.time import Time
import erfa

t = Time(
    "2001-03-22 00:01:44.732327132980",
    scale="utc",
    location=("120d", "40d"),
)

def to_sec(st) -> float:
    # Longitude is hourangle; convert hours → seconds of hourangle.
    return float(st.hour) * 3600.0

def format_hms(hourangle_hours: float) -> str:
    sec = (hourangle_hours % 24.0) * 3600.0
    h = int(sec // 3600)
    m = int((sec % 3600) // 60)
    s = sec % 60.0
    return f"{h:02d}:{m:02d}:{s:06.3f}"

gast = t.sidereal_time("apparent", "greenwich")
last = t.sidereal_time("apparent")
gmst = t.sidereal_time("mean", "greenwich")
lmst = t.sidereal_time("mean")
era = float(erfa.era00(t.ut1.jd1, t.ut1.jd2))

print("mjd_utc", float(t.mjd))                 # 51990.00121217971
print("dut1", float(t.delta_ut1_utc))          # 0.0345727753541418 s
print("gast_sec", to_sec(gast))                # 43200.000002614826  → ASTROPY_APPARENT_SEC
print("last_sec", to_sec(last))                # 72000.00000258541
print("gmst_sec", to_sec(gmst))                # 43201.019318368511  → ASTROPY_MEAN_SEC
print("lmst_sec", to_sec(lmst))                # 72001.0193183391
print("era_rad", era)                          # 3.1413939758024103
print(
    "hms",
    format_hms(float(gast.hour)),              # 12:00:00.000
    format_hms(float(last.hour)),              # 20:00:00.000
    format_hms(float(gmst.hour)),              # 12:00:01.019
    format_hms(float(lmst.hour)),              # 20:00:01.019
)
print("gast_off_12h_us", (to_sec(gast) - 43200) * 1e6)  # ~2.615 µs
print("last_off_20h_us", (to_sec(last) - 72000) * 1e6)  # ~2.585 µs
```

With C04 EOP, DUT1 matches Astropy; sidereal differences are well under a microsecond.
*/

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

    // ── UT1 via IERS C04 (same definitive series as Astropy IERS_Auto / eopc04)
    let eop = EopData::from_text_file(
        "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
        EopFormat::C04,
        Separator::Whitespace,
    )?;

    let mjd_utc = utc.to_mjd_f();
    let dut1 = Dt::mjd_to_eop_offset_f(mjd_utc, &eop)?; // seconds; assert vs Astropy
    let mjd_ut1 = utc.to_ut1(&eop)?.to_mjd_f_raw();

    // EO series need TT MJD (ERA / GMST use UT1; precession-nutation uses TT).
    let mjd_tt = utc.to(Scale::TT).to_mjd_f_raw();

    // ── Observatories (Astropy location=('120d', '40d')) ──────────────────
    let mut observer = Sidereal::EARTH;
    observer.longitude_rad = 120.0_f64.to_radians(); // east positive

    // ── Gather results ───────────────────────────────────────────────────
    let gmst_sec = Sidereal::to_sec(Sidereal::gmst(mjd_ut1, mjd_tt));
    let lmst_sec = Sidereal::to_sec(observer.lmst(mjd_ut1, mjd_tt));
    let gast_sec = Sidereal::to_sec(Sidereal::gast(mjd_ut1, mjd_tt));
    let last_sec = Sidereal::to_sec(observer.last(mjd_ut1, mjd_tt));
    let era_rad = Sidereal::era(mjd_ut1);

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
    // Checks:
    // 1. Expected values for this program and C04 data file.
    // 2. Textbook Astropy labels (12h / 20h) within a few microseconds.
    // 3. Cross-check vs measured Astropy (same C04/eopc04 DUT1 source).

    // Input path — DUT1 matches Astropy IERS_Auto on this instant.
    assert_close("MJD UTC", mjd_utc, 51_990.001_212_179_712, 1e-12);
    assert_close("DUT1", dut1, 0.034_572_775_354_142, 1e-12);
    assert!(mjd_ut1 > mjd_utc);

    // Display form at 1 ms — same strings as Astropy when formatted that way
    // (measured; see top-of-file Python block, not in the public docs).
    assert_eq!(gast_hms, "12:00:00.000");
    assert_eq!(last_hms, "20:00:00.000");
    assert_eq!(gmst_hms, "12:00:01.019");
    assert_eq!(lmst_hms, "20:00:01.019");

    // Expected values for this program and C04 data file.
    assert_close("GAST", gast_sec, 43_200.000_002_640_387, 1e-9);
    assert_close("LAST", last_sec, 72_000.000_002_640_401, 1e-9);
    assert_close("GMST", gmst_sec, 43_201.019_318_394_094, 1e-9);
    assert_close("LMST", lmst_sec, 72_001.019_318_394_072, 1e-9);
    assert_close("ERA", era_rad, 3.141_393_975_804_270, 1e-15);
    assert_close(
        "MJD TT",
        utc.to(Scale::TT).to_mjd_f_raw(),
        51_990.001_955_050_08,
        1e-12,
    );

    // Textbook hourangles: within 5 µs of exact 12h / 20h.
    // (Astropy is ~2.6 µs off 12h / 20h on this instant; see Python block.)
    const FIVE_US_AS_HOURS: f64 = 5e-6 / 3600.0;
    assert_close("GAST hourangle", gast_h, 12.0, FIVE_US_AS_HOURS);
    assert_close("LAST hourangle", last_h, 20.0, FIVE_US_AS_HOURS);
    // Mean is ~1.019 s after apparent.
    assert_close("GMST hourangle", gmst_h, 12.000_283_143_998_360, 1e-12);
    assert_close("LMST hourangle", lmst_h, 20.000_283_143_998_352, 1e-12);

    // Geometry
    assert_close("LAST − GAST", lon_hours, 8.0, 1e-12);
    assert_close("HA (RA=18h)", ha_signed, 2.0, FIVE_US_AS_HOURS);

    // Cross-check vs measured Astropy (top-of-file Python; differences ~0.03 µs).
    const ASTROPY_APPARENT_SEC: f64 = 43_200.000_002_614_826;
    const ASTROPY_MEAN_SEC: f64 = 43_201.019_318_368_511;
    const ASTROPY_ERA: f64 = 3.141_393_975_802_410_3;
    assert_close("GAST vs Astropy", gast_sec, ASTROPY_APPARENT_SEC, 1e-7);
    assert_close("GMST vs Astropy", gmst_sec, ASTROPY_MEAN_SEC, 1e-7);
    assert_close("ERA vs Astropy", era_rad, ASTROPY_ERA, 5e-12);

    Ok(())
}
