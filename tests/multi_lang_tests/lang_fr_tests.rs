#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

// french wont parse "mar" as tuesday

#[cfg(feature = "fr")]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale};

    fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
        let dt = Dt::from_str_parse(input.trim(), &opts)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let actual = dt.to_str_rfc3339().unwrap();

        assert_eq!(actual, expected_rfc3339, "Input: {}", input);
    }

    #[test]
    fn fr_dates() {
        let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![
            // === Full dates with long months ===
            ("1er janvier 2024", "2024-01-01T00:00:00Z", fr()),
            ("15 janvier 2024", "2024-01-15T00:00:00Z", fr()),
            ("le 15 janvier 2024", "2024-01-15T00:00:00Z", fr()),
            ("31 décembre 2023", "2023-12-31T00:00:00Z", fr()),
            ("1er décembre 2025", "2025-12-01T00:00:00Z", fr()),
            // === Short month forms ===
            ("15 janv 2024", "2024-01-15T00:00:00Z", fr()),
            ("3 févr 2025", "2025-02-03T00:00:00Z", fr()),
            ("20 août 2024", "2024-08-20T00:00:00Z", fr()),
            ("1er déc 2024", "2024-12-01T00:00:00Z", fr()),
            // === With weekday ===
            ("lundi 15 janvier 2024", "2024-01-15T00:00:00Z", fr()),
            ("ven 20 décembre 2024", "2024-12-20T00:00:00Z", fr()),
            ("samedi 1er juin 2024", "2024-06-01T00:00:00Z", fr()),
            // === Numeric formats (DD/MM/YYYY) ===
            ("15/01/2024", "2024-01-15T00:00:00Z", fr()),
            ("01/06/2025", "2025-06-01T00:00:00Z", fr()),
            ("15-01-2024", "2024-01-15T00:00:00Z", fr()),
            // === With time (use full minutes for clarity) ===
            ("15 janvier 2024 à 14h30", "2024-01-15T14:30:00Z", fr()),
            ("20 décembre 2024 18:00", "2024-12-20T18:00:00Z", fr()),
            ("15/01/2024 09:45", "2024-01-15T09:45:00Z", fr()),
            // === "le" prefix ===
            ("le 1er janvier 2024", "2024-01-01T00:00:00Z", fr()),
            ("Le 15 mars 2025", "2025-03-15T00:00:00Z", fr()),
            ("Le 15 mars 2025", "2025-03-15T00:00:00Z", fr()),
        ];

        for (input, expected, opts) in cases {
            assert_date(input, expected, opts);
        }
    }

    fn fr() -> Option<ParseCfg> {
        Some(ParseCfg {
            lang: Lang::Fr,
            ..Default::default()
        })
    }

    fn generate_relative_date_test_cases_fr() -> Vec<String> {
        let mut cases: Vec<String> = Vec::new();

        // Core French relatives
        let core_phrases = ["maintenant", "aujourd'hui", "demain", "hier"];
        cases.extend(core_phrases.iter().map(|&s| s.to_string()));

        // Numbers (including French decimal comma)
        let numbers = [
            "1", "2", "3", "5", "10", "42", "0.5", "1.5", "2,5", "3,75", "1_000",
        ];

        // French unit forms (we have good coverage here)
        let units = [
            "seconde", "secondes", "sec", "secs", "minute", "minutes", "min", "mins", "heure",
            "heures", "hr", "hrs", "jour", "jours", "semaine", "semaines", "mois", "an", "ans",
            "année", "années",
        ];

        let past_prefix = "il y a ";
        let future_prefix = "dans ";

        for num in numbers {
            for unit in units {
                // Past: "il y a 5 minutes"
                cases.push(format!("{}{} {}", past_prefix, num, unit));

                // Future: "dans 5 minutes"
                cases.push(format!("{}{} {}", future_prefix, num, unit));
            }
        }

        // Multi-unit French cases (realistic + some messy ones)
        let multi_unit_cases = [
            "il y a 2 heures et 30 minutes",
            "dans 1 jour et 12 heures",
            "il y a 3 semaines et 4 jours",
            "dans 2 jours 3 heures",
            "il y a 1 jour, 2 heures et 30 minutes",
            "dans 45 minutes et 15 secondes",
            "il y a 2 semaines 3 jours 4 heures",
            "dans 1 semaine et 2 jours",
            "il y a 3 jours 5 heures",
            // Messy / compact forms (still worth testing)
            "il y a 2jours 3h",
            "dans 1jour12h",
            "il y a 45min 15sec",
            "dans 2heures30minutes",
        ];
        cases.extend(multi_unit_cases.iter().map(|&s| s.to_string()));

        cases
    }

    #[test]
    fn relative_date_parser_comprehensive_fr() {
        let cases = generate_relative_date_test_cases_fr();
        let opts = Some(ParseCfg {
            lang: Lang::Fr,
            ref_time: Some(Dt::from_tai_sec(5_000_000)),
            ..Default::default()
        });

        for input in cases {
            let result = Dt::from_str_parse(input.trim(), &opts);
            assert!(
                result.is_ok(),
                "Failed to parse French relative date: '{}'",
                input
            );
        }
    }

    #[test]
    fn fr_durations() {
        fn assert_duration(input: &str, expected_millis: i64) {
            let dur = Dt::from_duration_str(input.trim(), Lang::Fr)
                .unwrap_or_else(|e| panic!("Failed '{}': {}", input, e));

            assert_eq!(dur.to_ms() as i64, expected_millis, "Input: '{}'", input);
        }

        let cases: Vec<(&str, i64)> = vec![
            // === Basics ===
            ("5 secondes", 5000),
            ("30 minutes", 1_800_000),
            ("2 heures", 7_200_000),
            ("1 jour", 86_400_000),
            ("2 semaines", 1_209_600_000),
            ("6 mois", 15_778_800_000),
            ("1 an", 31_557_600_000),
            // === Abbreviations ===
            ("2sem", 1_209_600_000),
            // === With "et" (and) ===
            ("2 heures et 30 minutes", 9_000_000),
            ("1 jour et 12 heures", 129_600_000),
            ("3 semaines et 4 jours", 2_160_000_000),
            // === French decimal comma ===
            ("1,5 heures", 5_400_000),
            ("2,5 jours", 216_000_000),
            ("0,5 minute", 30_000),
            ("3,75 heures", 13_500_000),
            // === Mixed / natural French ===
            ("1 jour, 2 heures et 30 minutes", 95_400_000),
            // === Negatives ===
            ("-5 minutes", -300_000),
            ("-2 heures et 30 minutes", -9_000_000),
            ("-1,5 jours", -129_600_000),
            // === Edge cases worth keeping ===
            ("  2 heures  ", 7_200_000), // whitespace
            ("1jour 12h", 129_600_000),  // compact
            ("2h30min", 9_000_000),      // concatenated
            ("heures 3", 10_800_000),    // unit-first
        ];

        for (input, expected) in cases {
            assert_duration(input, expected);
        }
    }

    #[test]
    fn fr_output_formatting() {
        let dt: Dt = "2025-01-01".parse().unwrap();

        let out = dt.to_str_lite("%a, %d %b %Y", Lang::Fr).unwrap();
        assert_eq!(out.as_str().unwrap(), "Mer, 01 janv 2025");

        let out = dt.to_str_lite("%A, %d %B %Y", Lang::Fr).unwrap();
        assert_eq!(out.as_str().unwrap(), "Mercredi, 01 janvier 2025");

        let out = dt.to_str_lite("%A, %d %B %Y %H:%M:%S", Lang::Fr).unwrap();
        assert_eq!(out.as_str().unwrap(), "Mercredi, 01 janvier 2025 00:00:00");
    }
}
