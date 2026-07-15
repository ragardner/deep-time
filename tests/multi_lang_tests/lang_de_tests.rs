#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(all(feature = "parse", feature = "de"))]
mod tests {
    use deep_time::macros::from_sec;
    use deep_time::{Dt, Lang, Order, ParseCfg, Scale};

    fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
        let d = ParseCfg::DEFAULT;
        let o = opts.as_ref().unwrap_or(&d);
        let dt = Dt::from_str_parse(input.trim(), o)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let actual = dt.to_str_rfc3339();

        assert_eq!(actual, expected_rfc3339, "Input: {}", input);
    }

    #[test]
    fn de_dates() {
        let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![
            // === Full dates with long months ===
            ("1. Januar 2024", "2024-01-01T00:00:00Z", de()),
            ("15. Januar 2024", "2024-01-15T00:00:00Z", de()),
            ("am 15. Januar 2024", "2024-01-15T00:00:00Z", de()),
            ("31. Dezember 2023", "2023-12-31T00:00:00Z", de()),
            ("1. Dezember 2025", "2025-12-01T00:00:00Z", de()),
            // === Short month forms ===
            ("15. Jan 2024", "2024-01-15T00:00:00Z", de()),
            ("3. Feb 2025", "2025-02-03T00:00:00Z", de()),
            ("20. Aug 2024", "2024-08-20T00:00:00Z", de()),
            ("1. Dez 2024", "2024-12-01T00:00:00Z", de()),
            // === With weekday ===
            ("Montag, 15. Januar 2024", "2024-01-15T00:00:00Z", de()),
            ("Freitag, 20. Dezember 2024", "2024-12-20T00:00:00Z", de()),
            ("Samstag, 1. Juni 2024", "2024-06-01T00:00:00Z", de()),
            // === Numeric formats (DD.MM.YYYY) ===
            ("15.01.2024", "2024-01-15T00:00:00Z", de()),
            ("01.06.2025", "2025-06-01T00:00:00Z", de()),
            ("15-01-2024", "2024-01-15T00:00:00Z", de()),
            ("15. März 2024 14:30 Uhr", "2024-03-15T14:30:00Z", de()),
            // === With time ===
            ("15. Januar 2024 um 14:30 Uhr", "2024-01-15T14:30:00Z", de()),
            ("20. Dezember 2024 18:00", "2024-12-20T18:00:00Z", de()),
            ("15.01.2024 09:45", "2024-01-15T09:45:00Z", de()),
            // === "am" prefix ===
            ("am 1. Januar 2024", "2024-01-01T00:00:00Z", de()),
            ("am 15. März 2025", "2025-03-15T00:00:00Z", de()),
        ];

        for (input, expected, opts) in cases {
            assert_date(input, expected, opts);
        }
    }

    fn de() -> Option<ParseCfg> {
        Some(ParseCfg {
            lang: Lang::De,
            order: Order::Day,
            ..Default::default()
        })
    }

    fn generate_relative_date_test_cases_de() -> Vec<String> {
        let mut cases: Vec<String> = Vec::new();

        // Core German relatives
        let core_phrases = ["jetzt", "heute", "morgen", "gestern"];
        cases.extend(core_phrases.iter().map(|&s| s.to_string()));

        // Numbers (German uses comma for decimal)
        let numbers = [
            "1", "2", "3", "5", "10", "42", "0,5", "1,5", "2,5", "3,75", "1_000",
        ];

        // German unit forms
        let units = [
            "sekunde", "sekunden", "sek", "minute", "minuten", "min", "stunde", "stunden", "std",
            "tag", "tage", "woche", "wochen", "monat", "monate", "jahr", "jahre",
        ];

        let past_prefix = "vor ";
        let future_prefix = "in ";

        for num in numbers {
            for unit in units {
                cases.push(format!("{}{} {}", past_prefix, num, unit));
                cases.push(format!("{}{} {}", future_prefix, num, unit));
            }
        }

        // Multi-unit German cases (realistic)
        let multi_unit_cases = [
            "vor 2 Stunden und 30 Minuten",
            "in 1 Tag und 12 Stunden",
            "vor 3 Wochen und 4 Tagen",
            "in 2 Tagen 3 Stunden",
            "vor 1 Tag, 2 Stunden und 30 Minuten",
            "in 45 Minuten und 15 Sekunden",
            "vor 2 Wochen 3 Tagen 4 Stunden",
            "in 1 Woche und 2 Tagen",
            "vor 3 Tagen 5 Stunden",
            // More natural compact forms
            "vor 2 Std 30 Min",
            "in 1 Woche 2 Tage",
            "vor 45 Min",
            "in 2h 30min",
            "vor 3 Tagen 5 Std",
        ];
        cases.extend(multi_unit_cases.iter().map(|&s| s.to_string()));

        cases
    }

    #[test]
    fn relative_date_parser_comprehensive_de() {
        let cases = generate_relative_date_test_cases_de();
        let opts = ParseCfg {
            lang: Lang::De,
            ref_time: Some(from_sec!(5_000_000)),
            ..Default::default()
        };

        for input in cases {
            let result = Dt::from_str_parse(input.trim(), &opts);
            assert!(
                result.is_ok(),
                "Failed to parse German relative date: '{}'",
                input
            );
        }
    }

    #[test]
    fn de_durations() {
        fn assert_duration(input: &str, expected_millis: i64) {
            let dur = Dt::from_str_duration(input.trim(), Lang::De)
                .unwrap_or_else(|e| panic!("Failed '{}': {}", input, e));

            assert_eq!(dur.to_ms().0 as i64, expected_millis, "Input: '{}'", input);
        }

        let cases: Vec<(&str, i64)> = vec![
            // === Basics ===
            ("5 Sekunden", 5000),
            ("30 Minuten", 1_800_000),
            ("2 Stunden", 7_200_000),
            ("1 Tag", 86_400_000),
            ("2 Wochen", 1_209_600_000),
            ("6 Monate", 15_778_800_000),
            ("1 Jahr", 31_557_600_000),
            // === Abbreviations ===
            ("5s", 5000),
            ("30min", 1_800_000),
            ("2h", 7_200_000),
            ("1 Tag", 86_400_000),
            ("2 Wochen", 1_209_600_000),
            // === With "und" (and) ===
            ("2 Stunden und 30 Minuten", 9_000_000),
            ("1 Tag und 12 Stunden", 129_600_000),
            ("3 Wochen und 4 Tage", 2_160_000_000),
            // === German decimal comma ===
            ("1,5 Stunden", 5_400_000),
            ("2,5 Tage", 216_000_000),
            ("0,5 Minute", 30_000),
            ("3,75 Stunden", 13_500_000),
            // === Mixed / natural German ===
            ("1 Tag, 2 Stunden und 30 Minuten", 95_400_000),
            // === Negatives ===
            ("-5 Minuten", -300_000),
            ("-2 Stunden und 30 Minuten", -9_000_000),
            ("-1,5 Tage", -129_600_000),
            // === Edge cases ===
            ("  2 Stunden  ", 7_200_000),
            ("2h30min", 9_000_000),
        ];

        for (input, expected) in cases {
            assert_duration(input, expected);
        }
    }

    #[test]
    fn de_output_formatting() {
        let dt: Dt = "2025-01-01".parse().unwrap();

        let out = dt.to_str_b("%a, %d %b %Y", Lang::De).unwrap();
        assert_eq!(out.as_str(), "Mi, 01 Jan 2025");

        let out = dt.to_str_b("%A, %d %B %Y", Lang::De).unwrap();
        assert_eq!(out.as_str(), "Mittwoch, 01 Januar 2025");

        let out = dt.to_str_b("%A, %d. %B %Y %H:%M:%S", Lang::De).unwrap();
        assert_eq!(out.as_str(), "Mittwoch, 01. Januar 2025 00:00:00");
    }
}
