#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "perf-tests")]
mod perf_tests {
    use deep_time::{Dt, Lang, Scale, TimeParts};
    use std::time::Instant;

    #[test]
    fn combined_perf_tests() {
        // ═══════════════════════════════════════════════════════════════════════
        // GPS CONVERSION PERF — deep_time vs hifitime 4.x
        // ═══════════════════════════════════════════════════════════════════════
        {
            use hifitime::{Epoch, TimeScale};
            use std::hint::black_box;

            const ITERATIONS: usize = 10_000_000;

            let deep_tai = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
            let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

            println!("\n=== GPS CONVERSION PERF — deep_time vs hifitime 4.x ===");

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_tai).to_gps();
            }
            let deep_gps = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::GPST));
            }
            let hifi_gps = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_tai).to_gps_wk_and_tow();
            }
            let deep_tow = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_tai).to_gpst_seconds();
            }
            let hifi_tow_equiv = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            println!("Method                    | deep_time      | hifitime       | deep/hifi");
            println!("--------------------------|----------------|----------------|----------");
            println!(
                "to_gps() / to_GPST        | {:7.2} ns/it   | {:7.2} ns/it   | {:.2}x",
                deep_gps,
                hifi_gps,
                deep_gps / hifi_gps
            );
            println!(
                "to_gps_wk_and_tow()       | {:7.2} ns/it   | {:7.2} ns/it * | {:.2}x",
                deep_tow,
                hifi_tow_equiv,
                deep_tow / hifi_tow_equiv
            );

            // ── Summary lines (deep_time vs hifitime) ─────────────────────────────
            let ratio_gps = deep_gps / hifi_gps;
            if ratio_gps < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on to_gps() / to_GPST",
                    (1.0 - ratio_gps) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on to_gps() / to_GPST",
                    (ratio_gps - 1.0) * 100.0
                );
            }

            let ratio_tow = deep_tow / hifi_tow_equiv;
            if ratio_tow < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on to_gps_wk_and_tow()",
                    (1.0 - ratio_tow) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on to_gps_wk_and_tow()",
                    (ratio_tow - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // TAI ↔ UTC PERF — deep_time vs hifitime 4.x
        // ═══════════════════════════════════════════════════════════════════════
        {
            use hifitime::{Epoch, TimeScale};
            use std::hint::black_box;

            const ITERATIONS: usize = 10_000_000;

            let deep_tai = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
            let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 0, 0, 0, 0);

            println!("\n=== TAI ↔ UTC PERF — deep_time vs hifitime 4.x ===");

            // TAI → UTC
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_tai).to(black_box(Scale::UTC));
            }
            let deep_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::UTC));
            }
            let hifi_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // UTC → TAI
            let deep_utc = deep_tai.to(Scale::UTC);
            let hifi_utc = hifi_tai.to_time_scale(TimeScale::UTC);

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_utc).to(black_box(Scale::TAI));
            }
            let deep_bwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_utc).to_time_scale(black_box(TimeScale::TAI));
            }
            let hifi_bwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            println!("Direction     | deep_time      | hifitime       | deep/hifi");
            println!("--------------|----------------|----------------|----------");
            println!(
                "TAI → UTC     | {:7.2} ns/it   | {:7.2} ns/it   | {:.2}x",
                deep_fwd,
                hifi_fwd,
                deep_fwd / hifi_fwd
            );
            println!(
                "UTC → TAI     | {:7.2} ns/it   | {:7.2} ns/it   | {:.2}x",
                deep_bwd,
                hifi_bwd,
                deep_bwd / hifi_bwd
            );

            // ── Summary lines (deep_time vs hifitime) ─────────────────────────────
            let ratio_fwd = deep_fwd / hifi_fwd;
            if ratio_fwd < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on TAI → UTC",
                    (1.0 - ratio_fwd) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on TAI → UTC",
                    (ratio_fwd - 1.0) * 100.0
                );
            }

            let ratio_bwd = deep_bwd / hifi_bwd;
            if ratio_bwd < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on UTC → TAI",
                    (1.0 - ratio_bwd) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on UTC → TAI",
                    (ratio_bwd - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // TAI ↔ TDB PERF — deep_time vs hifitime 4.x
        // ═══════════════════════════════════════════════════════════════════════
        {
            use hifitime::{Epoch, TimeScale};
            use std::hint::black_box;

            const ITERATIONS: usize = 1_000_000;

            // Same reference instant: J2000.0 (2000-01-01 12:00:00 TAI)
            let deep_tai = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
            let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

            println!("\n=== TAI ↔ TDB PERF — deep_time vs hifitime 4.x ===");

            // ── TAI → TDB ─────────────────────────────────────────────────────
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_tai).to(black_box(Scale::TDB));
            }
            let deep_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::TDB));
            }
            let hifi_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── TDB → TAI ─────────────────────────────────────────────────────
            let deep_tdb = deep_tai.to(Scale::TDB);
            let hifi_tdb = hifi_tai.to_time_scale(TimeScale::TDB);

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(deep_tdb).to(black_box(Scale::TAI));
            }
            let deep_bwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = black_box(hifi_tdb).to_time_scale(black_box(TimeScale::TAI));
            }
            let hifi_bwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results table ─────────────────────────────────────────────────
            println!("Direction     | deep_time      | hifitime       | deep/hifi");
            println!("--------------|----------------|----------------|----------");
            println!(
                "TAI → TDB     | {:7.2} ns/it   | {:7.2} ns/it   | {:.2}x",
                deep_fwd,
                hifi_fwd,
                deep_fwd / hifi_fwd
            );
            println!(
                "TDB → TAI     | {:7.2} ns/it   | {:7.2} ns/it   | {:.2}x",
                deep_bwd,
                hifi_bwd,
                deep_bwd / hifi_bwd
            );

            // ── Summary lines (deep_time vs hifitime) ─────────────────────────────
            let ratio_fwd = deep_fwd / hifi_fwd;
            if ratio_fwd < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on TAI → TDB",
                    (1.0 - ratio_fwd) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on TAI → TDB",
                    (ratio_fwd - 1.0) * 100.0
                );
            }

            let ratio_bwd = deep_bwd / hifi_bwd;
            if ratio_bwd < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than hifitime on TDB → TAI",
                    (1.0 - ratio_bwd) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than hifitime on TDB → TAI",
                    (ratio_bwd - 1.0) * 100.0
                );
            }
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
                    let _ = Dt::from_str_parse(input, &None);
                }
            }
            let elapsed = start.elapsed();

            let total_parses = cases.len() * ITERATIONS;
            let ns_per_parse = elapsed.as_nanos() as f64 / total_parses as f64;

            println!("\n=== DATE AUTO PARSER PERF ===");
            println!("Avg time     : {:.2} ns/parse", ns_per_parse);
            println!(
                "Throughput   : {:.0} k parses/sec",
                1_000_000.0 / ns_per_parse
            );
        }

        // ═══════════════════════════════════════════════════════════════════════
        // strptime — TimeParts vs Jiff BrokenDownTime strtime
        // ═══════════════════════════════════════════════════════════════════════
        {
            const ITERATIONS: usize = 10_000_000;
            const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
            const INPUT: &str = "2024-03-14T00:00:00";

            // ── Jiff low-level BrokenDownTime strtime ───────────────────────
            use jiff::fmt::strtime::BrokenDownTime;

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = BrokenDownTime::parse(FORMAT, INPUT).unwrap();
            }
            let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── TimeParts ───────────────────────
            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = TimeParts::from_str(FORMAT, INPUT, true, true, false).unwrap();
            }
            let timeparts_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results ───────────────────────────────────────────────────────────────
            println!("\n=== strptime — TimeParts vs Jiff BrokenDownTime ===");
            println!(
                "TimeParts : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                timeparts_ns,
                1_000_000.0 / timeparts_ns
            );
            println!(
                "Jiff      : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                jiff_ns,
                1_000_000.0 / jiff_ns
            );

            let ratio = timeparts_ns / jiff_ns;
            if ratio < 1.0 {
                println!(
                    "→ TimeParts is {:.1}% **faster** than Jiff strtime on this format",
                    (1.0 - ratio) * 100.0
                );
            } else {
                println!(
                    "→ TimeParts is {:.1}% slower than Jiff strtime on this format",
                    (ratio - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // Zoned strptime — TimeParts vs Jiff BrokenDownTime strtime
        // ═══════════════════════════════════════════════════════════════════════
        {
            const ITERATIONS: usize = 5_000_000;
            const INPUT: &str = "2024-03-14T00:00:00[America/New_York]";
            const FORMAT_WITH_Q: &str = "%Y-%m-%dT%H:%M:%S[%Q]";

            // ── Jiff zoned parsing via low-level strtime ───────────────────────
            use jiff::fmt::strtime::BrokenDownTime;

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let tm = BrokenDownTime::parse(FORMAT_WITH_Q, INPUT).unwrap();
                let x = tm.to_zoned().unwrap();
            }
            let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── deep_time with %Q directive ─────────────────
            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = Dt::from_str(INPUT, FORMAT_WITH_Q, true, true, false).unwrap();
            }
            let deep_time_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results ───────────────────────────────────────────────────────────────
            println!("\n=== Zoned strptime — TimeParts vs Jiff BrokenDownTime ===");
            println!(
                "deep_time : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                deep_time_ns,
                1_000_000.0 / deep_time_ns
            );
            println!(
                "jiff      : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                jiff_ns,
                1_000_000.0 / jiff_ns
            );

            let ratio = deep_time_ns / jiff_ns;
            if ratio < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than jiff on zoned IANA parsing",
                    (1.0 - ratio) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than jiff on zoned IANA parsing",
                    (ratio - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // Datetime strptime — Dt::from_str vs Jiff to_datetime strtime
        // ═══════════════════════════════════════════════════════════════════════
        {
            const ITERATIONS: usize = 5_000_000; // lowered because IANA zone resolution is heavier
            const INPUT: &str = "2024-03-14T00:00:00";
            const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

            // ── Jiff parsing via low-level strtime ───────────────────────
            use jiff::fmt::strtime::BrokenDownTime;

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let tm = BrokenDownTime::parse(FORMAT, INPUT).unwrap();
                let x = tm.to_datetime().unwrap();
            }
            let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── deep time  parsing ───────────────────────

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = Dt::from_str(INPUT, FORMAT, true, true, false).unwrap();
            }
            let deep_time_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results ───────────────────────────────────────────────────────────────
            println!("\n=== DateTime strptime — Dt::from_str vs Jiff strtime -> to_datetime ===");
            println!(
                "deep_time : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                deep_time_ns,
                1_000_000.0 / deep_time_ns
            );
            println!(
                "jiff      : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                jiff_ns,
                1_000_000.0 / jiff_ns
            );

            let ratio = deep_time_ns / jiff_ns;
            if ratio < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than jiff on datetime parsing",
                    (1.0 - ratio) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than jiff on datetime parsing",
                    (ratio - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // Datetime parse — TimeParts::from_str_iso vs Jiff parse::DateTime
        // ═══════════════════════════════════════════════════════════════════════
        {
            const ITERATIONS: usize = 10_000_000;
            const INPUT: &str = "2024-03-14T00:00:00.123456789";

            // ── Jiff high-level DateTime parse ───────────────────────
            use jiff::civil::DateTime;

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = INPUT.parse::<DateTime>().unwrap();
            }
            let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── deep_time CCSDS/ISO dedicated parser ───────────────────────
            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = TimeParts::from_str_iso(INPUT).unwrap();
            }
            let deep_time_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results ───────────────────────────────────────────────────────────────
            println!("\n=== DateTime parse — TimeParts::from_str_iso vs Jiff parse::DateTime ===");
            println!(
                "deep_time : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                deep_time_ns,
                1_000_000.0 / deep_time_ns
            );
            println!(
                "jiff      : {:7.2} ns/parse  |  {:7.0} k parses/sec",
                jiff_ns,
                1_000_000.0 / jiff_ns
            );

            let ratio = deep_time_ns / jiff_ns;
            if ratio < 1.0 {
                println!(
                    "→ deep_time (from_str_iso) is {:.1}% **faster** than Jiff on ISO datetime parsing",
                    (1.0 - ratio) * 100.0
                );
            } else {
                println!(
                    "→ deep_time (from_str_iso) is {:.1}% slower than Jiff on ISO datetime parsing",
                    (ratio - 1.0) * 100.0
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // strftime — Dt::to_str vs Jiff civil::DateTime::strftime
        // ═══════════════════════════════════════════════════════════════════════
        {
            const ITERATIONS: usize = 10_000_000;
            const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

            // ── deep_time ───────────────────────
            let dt = Dt::from_str("2024-03-14T00:00:00", FORMAT, true, true, false).unwrap();

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = dt.to_str_lite(FORMAT, Lang::En);
            }
            let deep_time_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Jiff ───────────────────────
            use jiff::civil::DateTime;

            let jiff_dt: DateTime = "2024-03-14T00:00:00".parse().unwrap();

            let start = std::time::Instant::now();
            for _ in 0..ITERATIONS {
                let x = jiff_dt.strftime(FORMAT).to_string();
            }
            let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

            // ── Results ───────────────────────────────────────────────────────────────
            println!("\n=== strftime — Dt::to_str_lite vs Jiff DateTime::strftime ===");
            println!(
                "deep_time : {:7.2} ns/fmt    |  {:7.0} k fmts/sec",
                deep_time_ns,
                1_000_000.0 / deep_time_ns
            );
            println!(
                "jiff      : {:7.2} ns/fmt    |  {:7.0} k fmts/sec",
                jiff_ns,
                1_000_000.0 / jiff_ns
            );

            let ratio = deep_time_ns / jiff_ns;
            if ratio < 1.0 {
                println!(
                    "→ deep_time is {:.1}% **faster** than jiff on strftime formatting",
                    (1.0 - ratio) * 100.0
                );
            } else {
                println!(
                    "→ deep_time is {:.1}% slower than jiff on strftime formatting",
                    (ratio - 1.0) * 100.0
                );
            }
        }
    }
}
