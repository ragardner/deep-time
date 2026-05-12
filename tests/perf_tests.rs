#[cfg(feature = "perf-tests")]
#[cfg(test)]
mod tests {
    use deep_time::{Dt, Scale};
    use std::time::Instant;

    #[test]
    fn date_auto_parser_perf() {
        let corpus: Vec<&str> = vec![
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
            for &input in &corpus {
                let _ = Dt::from_str_parse(input, &None);
            }
        }
        let elapsed = start.elapsed();

        let total_parses = corpus.len() * ITERATIONS;
        let ns_per_parse = elapsed.as_nanos() as f64 / total_parses as f64;

        println!("\n=== DATE AUTO PARSER PERF ===");
        println!("Avg time     : {:.2} ns/parse", ns_per_parse);
        println!(
            "Throughput   : {:.0} k parses/sec",
            1_000_000.0 / ns_per_parse
        );
    }

    #[test]
    fn date_from_str_perf() {
        const ITERATIONS: usize = 10_000_000;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = Dt::from_str(
                "2024-03-14T00:00:00",
                "%Y-%m-%dT%H:%M:%S",
                true,
                true,
                false,
            );
        }
        let elapsed = start.elapsed();

        let total_parses = ITERATIONS;
        let ns_per_parse = elapsed.as_nanos() as f64 / total_parses as f64;

        println!("\n=== DATE FROM STR PERF ===");
        println!("Avg time     : {:.2} ns/parse", ns_per_parse);
        println!(
            "Throughput   : {:.0} k parses/sec",
            1_000_000.0 / ns_per_parse
        );
    }

    #[cfg(feature = "jiff-tz")]
    #[test]
    fn zoned_from_str_perf_jiff_comparison() {
        const ITERATIONS: usize = 5_000_000; // lowered because IANA zone resolution is heavier
        const INPUT: &str = "2024-03-14T00:00:00[America/New_York]";
        const FORMAT_WITH_Q: &str = "%Y-%m-%dT%H:%M:%S[%Q]";

        // ── deep_time with %Q directive (your auto-handling path) ─────────────────
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = Dt::from_str(INPUT, FORMAT_WITH_Q, true, true, false).unwrap();
        }
        let deep_time_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── Jiff native zoned parsing (idiomatic IANA path) ───────────────────────
        use jiff::Zoned;

        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = INPUT.parse::<Zoned>().unwrap();
        }
        let jiff_ns = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── Results ───────────────────────────────────────────────────────────────
        println!("\n=== ZONED FROM STR PERF — deep_time vs jiff ===");
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

    #[test]
    fn tai_tdb_perf() {
        const ITERATIONS: usize = 1_000_000;

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::TAI, Scale::TDB);
        }
        let elapsed = start.elapsed();

        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== TAI -> TDB PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1).to(Scale::TAI, Scale::TDB);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::TDB, Scale::TAI);
        }
        let elapsed = start.elapsed();
        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== TDB -> TAI PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);
    }

    #[test]
    fn tai_tt_perf() {
        const ITERATIONS: usize = 1_000_000;

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::TAI, Scale::TT);
        }
        let elapsed = start.elapsed();

        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== TAI -> TT PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1).to(Scale::TAI, Scale::TT);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::TT, Scale::TAI);
        }
        let elapsed = start.elapsed();
        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== TT -> TAI PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);
    }

    #[cfg(feature = "hifitime")]
    #[test]
    fn tai_utc_hifitime_comparison() {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;
        use std::time::Instant;

        const ITERATIONS: usize = 10_000_000;

        let deep_tai = Dt::from_ymd(2000, 1, 1);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 0, 0, 0, 0);

        println!("\n=== TAI ↔ UTC PERF — deep_time vs hifitime 4.x ===");

        // TAI → UTC
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to(black_box(Scale::TAI), black_box(Scale::UTC));
        }
        let deep_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::UTC));
        }
        let hifi_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // UTC → TAI
        let deep_utc = deep_tai.to(Scale::TAI, Scale::UTC);
        let hifi_utc = hifi_tai.to_time_scale(TimeScale::UTC);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_utc).to(black_box(Scale::UTC), black_box(Scale::TAI));
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
    }

    #[cfg(feature = "hifitime")]
    #[test]
    fn tai_tdb_hifitime_comparison() {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;
        use std::time::Instant;

        const ITERATIONS: usize = 1_000_000;

        // Same reference instant: J2000.0 (2000-01-01 12:00:00 TAI)
        let deep_tai = Dt::from_ymd(2000, 1, 1);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

        println!("\n=== TAI ↔ TDB PERF — deep_time vs hifitime 4.x ===");

        // ── TAI → TDB ─────────────────────────────────────────────────────
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to(black_box(Scale::TAI), black_box(Scale::TDB));
        }
        let deep_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::TDB));
        }
        let hifi_fwd = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        // ── TDB → TAI ─────────────────────────────────────────────────────
        let deep_tdb = deep_tai.to(Scale::TAI, Scale::TDB);
        let hifi_tdb = hifi_tai.to_time_scale(TimeScale::TDB);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tdb).to(black_box(Scale::TDB), black_box(Scale::TAI));
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
    }

    #[cfg(feature = "hifitime")]
    #[test]
    fn gps_conversion_hifitime_comparison() {
        use hifitime::{Epoch, TimeScale};
        use std::hint::black_box;
        use std::time::Instant;

        const ITERATIONS: usize = 10_000_000;

        let deep_tai = Dt::from_ymd(2000, 1, 1);
        let hifi_tai = Epoch::from_gregorian_tai(2000, 1, 1, 12, 0, 0, 0);

        println!("\n=== GPS CONVERSION PERF — deep_time vs hifitime 4.x ===");

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to_gps(black_box(Scale::TAI));
        }
        let deep_gps = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(hifi_tai).to_time_scale(black_box(TimeScale::GPST));
        }
        let hifi_gps = start.elapsed().as_nanos() as f64 / ITERATIONS as f64;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = black_box(deep_tai).to_gps_wk_and_tow(black_box(Scale::TAI));
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
    }

    #[test]
    fn tai_utc_perf() {
        const ITERATIONS: usize = 1_000_000;

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::TAI, Scale::UTC);
        }
        let elapsed = start.elapsed();

        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== TAI -> UTC PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1).to(Scale::TAI, Scale::UTC);
        for _ in 0..ITERATIONS {
            let _ = x.to(Scale::UTC, Scale::TAI);
        }
        let elapsed = start.elapsed();
        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== UTC -> TAI PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);
    }

    #[test]
    fn gps_perf() {
        const ITERATIONS: usize = 1_000_000;

        let start = Instant::now();
        let x = Dt::from_ymd(2000, 1, 1);
        for _ in 0..ITERATIONS {
            let _ = x.to_gps(Scale::TAI);
        }
        let elapsed = start.elapsed();

        let ns_per_it = elapsed.as_nanos() as f64 / ITERATIONS as f64;

        println!("\n=== GPS OUTPUT PERF ===");
        println!("Avg time     : {:.2} ns/it", ns_per_it);
        println!("Throughput   : {:.0} k its/sec", 1_000_000.0 / ns_per_it);
    }
}
