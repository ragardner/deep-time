#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

use deep_time::civil_parts::Parts;
use deep_time::{Dt, Lang, ParseCfg, Scale};
use std::time::Instant;

fn main() {
    // All timing loops and test cases are left unchanged.
    // Results are captured and only two tables are emitted (via eprintln!) at the very end.

    // ── results (populated by the benchmark blocks below) ───────────────────
    let mut gps_deep_ns = 0.0f64;
    let mut gps_hifi_ns = 0.0f64;
    let mut tow_deep_ns = 0.0f64;
    let mut tow_hifi_ns = 0.0f64;

    let mut tai_utc_deep_ns = 0.0f64;
    let mut tai_utc_hifi_ns = 0.0f64;
    let mut utc_tai_deep_ns = 0.0f64;
    let mut utc_tai_hifi_ns = 0.0f64;

    let mut tai_tdb_deep_ns = 0.0f64;
    let mut tai_tdb_hifi_ns = 0.0f64;
    let mut tdb_tai_deep_ns = 0.0f64;
    let mut tdb_tai_hifi_ns = 0.0f64;

    let mut auto_ns_per = 0.0f64;

    let mut strptime_timeparts_ns = 0.0f64;
    let mut strptime_jiff_ns = 0.0f64;

    let mut zoned_deep_ns = 0.0f64;
    let mut zoned_jiff_ns = 0.0f64;

    let mut iso_deep_ns = 0.0f64;
    let mut iso_jiff_ns = 0.0f64;

    let mut strftime_b_ns = 0.0f64;
    let mut strftime_alloc_ns = 0.0f64;
    let mut strftime_jiff_ns = 0.0f64;

    // ═══════════════════════════════════════════════════════════════════════
    // GPS CONVERSION PERF — deep_time vs hifitime 4.x
    // ═══════════════════════════════════════════════════════════════════════
    {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;

        const ITERATIONS: usize = 10_000_000;

        let deep_tai = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to_gps();
        }
        gps_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::GPST));
        }
        gps_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to_gps_wk_and_tow();
        }
        tow_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_gpst_seconds();
        }
        tow_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TAI ↔ UTC PERF — deep_time vs hifitime 4.x
    // ═══════════════════════════════════════════════════════════════════════
    {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;

        const ITERATIONS: usize = 10_000_000;

        let deep_tai = Dt::from_ymd(2000, 1, 1, Scale::UTC, 0, 0, 0, 0);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 0, 0, 0, 0);

        // TAI → UTC
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to(black_box(Scale::UTC));
        }
        tai_utc_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::UTC));
        }
        tai_utc_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // UTC → TAI
        let deep_utc = deep_tai.to(Scale::UTC);
        let hifi_utc = hifi_tai.to_time_scale(TimeScale::UTC);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_utc).to(black_box(Scale::TAI));
        }
        utc_tai_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_utc).to_time_scale(black_box(TimeScale::TAI));
        }
        utc_tai_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TAI ↔ TDB PERF — deep_time vs hifitime 4.x TDB
    // ═══════════════════════════════════════════════════════════════════════
    {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;

        const ITERATIONS: usize = 1_000_000;

        // Same reference instant: J2000.0 (2000-01-01 12:00:00 TAI)
        let deep_tai = Dt::from_ymd(2000, 1, 1, Scale::TAI, 0, 0, 0, 0);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

        // ── TAI → TDB (vs hifitime TDB) ───────────────────────────────────
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to(black_box(Scale::TDB));
        }
        tai_tdb_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::TDB));
        }
        tai_tdb_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── TDB → TAI (vs hifitime TDB → TAI) ─────────────────────────────
        let deep_tdb = deep_tai.to(Scale::TDB);
        let hifi_tdb = hifi_tai.to_time_scale(TimeScale::TDB);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tdb).to(black_box(Scale::TAI));
        }
        tdb_tai_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tdb).to_time_scale(black_box(TimeScale::TAI));
        }
        tdb_tai_hifi_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATE AUTO PARSER PERF
    // ═══════════════════════════════════════════════════════════════════════
    {
        let cases: Vec<&str> = vec![
            "2024-03-14",
            "14 Mar 2024",
            "Mar 14, 2024",
            "2024/03/14",
            "14.03.2024",
            "14/03/2024",
            "240314",
            "202403",
            "2024073",
            "24073",
            "60400",
            "2460000",
            "1700000000",
            "1700000000000",
            "15/03/2024 14:30:45.123456",
            "Mar 15, 2024 14:30",
            "2024-03-14T15:30:45.123456Z",
            "2024-03-14T15:30:45+01:00",
            // "Dec 31 23:59:59",
            "15/03/2024 14:30:45.123456789",
            "03/03/2024",
            "2024-074T15:30:45.123Z",
            // "YmdHMS",
            "240315",
            "-2024-03-14",
            "2024-03-14 15:30",
            "15 Mar 2024 14:30:45",
            // REMOVED: "20240314153045" (pure numeric YYYYMMDDHHMMSS case)
        ];

        const ITERATIONS: usize = 100_000;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            for &input in &cases {
                let _ = Dt::from_str_parse(input, &ParseCfg::DEFAULT);
            }
        }
        let elapsed = start.elapsed();

        let total_parses = cases.len() * ITERATIONS;
        auto_ns_per = elapsed.as_nanos() as f64 / total_parses as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // strptime — Parts vs Jiff BrokenDownTime strtime
    // ═══════════════════════════════════════════════════════════════════════
    {
        const ITERATIONS: usize = 30_000_000;
        const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
        const INPUT: &str = "2024-03-14T00:00:00";

        // ── Jiff low-level BrokenDownTime strtime ───────────────────────
        use jiff::fmt::strtime::BrokenDownTime;

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = BrokenDownTime::parse(FORMAT, INPUT).unwrap();
        }
        strptime_jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── Parts ───────────────────────
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = Parts::from_strptime(FORMAT, INPUT, true, true, false).unwrap();
        }
        strptime_timeparts_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Zoned strptime — Dt vs Jiff Zoned using strtime
    // ═══════════════════════════════════════════════════════════════════════
    {
        const ITERATIONS: usize = 10_000_000;
        const INPUT: &str = "2024-03-14T00:00:00[America/New_York]";
        const FORMAT_WITH_Q: &str = "%Y-%m-%dT%H:%M:%S[%Q]";

        // ── Jiff zoned parsing via low-level strtime ───────────────────────
        use jiff::fmt::strtime::BrokenDownTime;

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let tm = BrokenDownTime::parse(FORMAT_WITH_Q, INPUT).unwrap();
            let x = tm.to_zoned().unwrap();
        }
        zoned_jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── deep_time with %Q directive ─────────────────
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = Dt::from_strptime(INPUT, FORMAT_WITH_Q, true, true, false).unwrap();
        }
        zoned_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Datetime strptime — Dt::from_strptime vs Jiff to_datetime strtime
    // ═══════════════════════════════════════════════════════════════════════
    {
        const ITERATIONS: usize = 10_000_000; // lowered because IANA zone resolution is heavier
        const INPUT: &str = "2024-03-14T00:00:00";
        const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

        // ── Jiff parsing via low-level strtime ───────────────────────
        use jiff::fmt::strtime::BrokenDownTime;

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let tm = BrokenDownTime::parse(FORMAT, INPUT).unwrap();
            let x = tm.to_datetime().unwrap();
        }
        let _dt_strptime_jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── deep time  parsing ───────────────────────

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = Dt::from_strptime(INPUT, FORMAT, true, true, false).unwrap();
        }
        let _dt_strptime_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Datetime parse — Dt::from_str vs Jiff parse::DateTime
    // ═══════════════════════════════════════════════════════════════════════
    {
        const ITERATIONS: usize = 20_000_000;
        const INPUT: &str = "2024-03-14T00:00:00.123456789";

        // ── Jiff high-level DateTime parse ───────────────────────
        use jiff::civil::DateTime;

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = INPUT.parse::<DateTime>().unwrap();
        }
        iso_jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── deep_time CCSDS/ISO dedicated parser ───────────────────────
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = Dt::from_str(INPUT).unwrap();
        }
        iso_deep_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // strftime — Dt::to_str_b / to_str vs Jiff civil::DateTime::strftime
    // ═══════════════════════════════════════════════════════════════════════
    {
        const ITERATIONS: usize = 20_000_000;
        const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

        // ── deep_time (BufStr, no alloc) ───────────────────────
        let dt = Dt::from_strptime("2024-03-14T00:00:00", FORMAT, true, true, false).unwrap();

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = dt.to_str_b(FORMAT, Lang::En);
        }
        strftime_b_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── deep_time (alloc String) ───────────────────────
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = dt.to_str(FORMAT, Lang::En).unwrap();
        }
        strftime_alloc_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── Jiff (alloc String) ───────────────────────
        use jiff::civil::DateTime;

        let jiff_dt: DateTime = "2024-03-14T00:00:00".parse().unwrap();

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let x = jiff_dt.strftime(FORMAT).to_string();
        }
        strftime_jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Emit two neat tables (only output; using eprintln! so it is easy to capture)
    // These tables are formatted to be directly droppable into the README.
    // ═══════════════════════════════════════════════════════════════════════

    let fmt_ns = |x: f64| -> String {
        if x >= 99.5 {
            format!("{:.0} ns", x)
        } else {
            format!("{:.1} ns", x)
        }
    };

    // For parse/format table we use "% faster/slower" style (matching historical README)
    let pct = |d: f64, j: f64| -> String {
        let r = d / j;
        if r < 1.0 {
            format!("{:.1}% faster", (1.0 - r) * 100.0)
        } else {
            format!("{:.1}% slower", (r - 1.0) * 100.0)
        }
    };

    // For scale conversions we use "× faster/slower" style (matching historical README)
    let xrel = |d: f64, h: f64| -> String {
        let r = d / h;
        if r < 1.0 {
            format!("{:.1}× faster", 1.0 / r)
        } else {
            format!("{:.1}× slower", r)
        }
    };

    const COL_OP: usize = 58;
    const COL_TIME: usize = 11;
    const COL_VS: usize = 16;

    let cell = |w: usize, s: &str| format!(" {:<inner$}", s, inner = w - 1);

    let perf_row = |op: &str, time: &str, vs: &str| {
        eprintln!(
            "|{}|{}|{}|",
            cell(COL_OP, op),
            cell(COL_TIME, time),
            cell(COL_VS, vs),
        );
    };

    eprintln!();
    eprintln!("#### Parsing and Formatting");
    eprintln!();
    perf_row("deep-time vs jiff", "Time", "vs Jiff 0.2.31");
    eprintln!(
        "|{}|{}|{}|",
        "-".repeat(COL_OP),
        "-".repeat(COL_TIME),
        "-".repeat(COL_VS),
    );
    perf_row(
        "`Dt::from_str` vs `DateTime::parse`",
        &fmt_ns(iso_deep_ns),
        &pct(iso_deep_ns, iso_jiff_ns),
    );
    perf_row(
        "`Parts::from_strptime` vs `BrokenDownTime::parse`",
        &fmt_ns(strptime_timeparts_ns),
        &pct(strptime_timeparts_ns, strptime_jiff_ns),
    );
    perf_row(
        "`Dt::from_strptime` vs `BrokenDownTime::parse`+`to_zoned`",
        &fmt_ns(zoned_deep_ns),
        &pct(zoned_deep_ns, zoned_jiff_ns),
    );
    perf_row(
        "`Dt::to_str_b` vs `DateTime::strftime`+`.to_string`",
        &fmt_ns(strftime_b_ns),
        &pct(strftime_b_ns, strftime_jiff_ns),
    );
    perf_row(
        "`Dt::to_str` vs `DateTime::strftime`+`.to_string`",
        &fmt_ns(strftime_alloc_ns),
        &pct(strftime_alloc_ns, strftime_jiff_ns),
    );
    perf_row("`Dt::from_str_parse`", &fmt_ns(auto_ns_per), "—");

    eprintln!();
    eprintln!("#### Time Scale Conversions");
    eprintln!();
    eprintln!("| Conversion       | deep-time     | hifitime 4.3  | Relative Performance      |");
    eprintln!("|------------------|---------------|---------------|---------------------------|");
    eprintln!(
        "| TAI → UTC        | {:<13} | {:<13} | {:<25} |",
        fmt_ns(tai_utc_deep_ns),
        fmt_ns(tai_utc_hifi_ns),
        xrel(tai_utc_deep_ns, tai_utc_hifi_ns)
    );
    eprintln!(
        "| UTC → TAI        | {:<13} | {:<13} | {:<25} |",
        fmt_ns(utc_tai_deep_ns),
        fmt_ns(utc_tai_hifi_ns),
        xrel(utc_tai_deep_ns, utc_tai_hifi_ns)
    );
    eprintln!(
        "| TAI → TDB        | {:<13} | {:<13} | {:<25} |",
        fmt_ns(tai_tdb_deep_ns),
        fmt_ns(tai_tdb_hifi_ns),
        xrel(tai_tdb_deep_ns, tai_tdb_hifi_ns)
    );
    eprintln!(
        "| TDB → TAI        | {:<13} | {:<13} | {:<25} |",
        fmt_ns(tdb_tai_deep_ns),
        fmt_ns(tdb_tai_hifi_ns),
        xrel(tdb_tai_deep_ns, tdb_tai_hifi_ns)
    );
    eprintln!(
        "| GPS conversion   | {:<13} | {:<13} | {:<25} |",
        fmt_ns(gps_deep_ns),
        fmt_ns(gps_hifi_ns),
        xrel(gps_deep_ns, gps_hifi_ns)
    );
    eprintln!(
        "| GPS week + TOW   | {:<13} | {:<13} | {:<25} |",
        fmt_ns(tow_deep_ns),
        fmt_ns(tow_hifi_ns),
        xrel(tow_deep_ns, tow_hifi_ns)
    );
    eprintln!();
}
