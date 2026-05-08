#[cfg(feature = "perf-tests")]
#[cfg(test)]
mod tests {
    use deep_time::Dt;
    use std::time::Instant;

    #[test]
    fn date_alloc_parser_perf() {
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

        const ITERATIONS: usize = 800_000;

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            for &input in &corpus {
                let _ = Dt::from_str_parse(input, &None);
                // if let Err(x) = x {
                //     eprintln!("{}", x);
                //     return;
                // }
            }
        }
        let elapsed = start.elapsed();

        let total_parses = corpus.len() * ITERATIONS;
        let ns_per_parse = elapsed.as_nanos() as f64 / total_parses as f64;

        println!("\n=== DATE ALLOC PARSER PERF ===");
        println!("Total parses : {}", total_parses);
        println!("Total time   : {:.3} ms", elapsed.as_secs_f64() * 1000.0);
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
            // if let Err(x) = x {
            //     eprintln!("{}", x);
            //     return;
            // }
        }
        let elapsed = start.elapsed();

        let total_parses = ITERATIONS;
        let ns_per_parse = elapsed.as_nanos() as f64 / total_parses as f64;

        println!("\n=== DATE FROM STR PERF ===");
        println!("Total parses : {}", total_parses);
        println!("Total time   : {:.3} ms", elapsed.as_secs_f64() * 1000.0);
        println!("Avg time     : {:.2} ns/parse", ns_per_parse);
        println!(
            "Throughput   : {:.0} k parses/sec",
            1_000_000.0 / ns_per_parse
        );
    }
}
