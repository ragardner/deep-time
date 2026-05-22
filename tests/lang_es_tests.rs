#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

// spanish won't parse "mar" as tuesday (collision with marzo)

#[cfg(feature = "es")]
#[cfg(test)]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale};

    fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
        let dt = Dt::from_str_parse(input.trim(), &opts)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let actual = dt.to_str_rfc3339(Scale::TAI).unwrap();

        assert_eq!(actual, expected_rfc3339, "Input: {}", input);
    }

    #[test]
    fn es_dates() {
        let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![
            // === Full dates with long months ===
            ("1 de enero de 2024", "2024-01-01T00:00:00Z", es()),
            ("15 de enero de 2024", "2024-01-15T00:00:00Z", es()),
            ("el 15 de enero de 2024", "2024-01-15T00:00:00Z", es()),
            ("31 de diciembre de 2023", "2023-12-31T00:00:00Z", es()),
            ("1 de diciembre de 2025", "2025-12-01T00:00:00Z", es()),
            // === Short month forms ===
            ("15 ene 2024", "2024-01-15T00:00:00Z", es()),
            ("3 feb 2025", "2025-02-03T00:00:00Z", es()),
            ("20 ago 2024", "2024-08-20T00:00:00Z", es()),
            ("1 dic 2024", "2024-12-01T00:00:00Z", es()),
            // === With weekday ===
            ("lunes 15 de enero de 2024", "2024-01-15T00:00:00Z", es()),
            ("vie 20 de diciembre de 2024", "2024-12-20T00:00:00Z", es()),
            ("sábado 1 de junio de 2024", "2024-06-01T00:00:00Z", es()),
            // === Numeric formats (DD/MM/YYYY) ===
            ("15/01/2024", "2024-01-15T00:00:00Z", es()),
            ("01/06/2025", "2025-06-01T00:00:00Z", es()),
            ("15-01-2024", "2024-01-15T00:00:00Z", es()),
            // === With time ===
            (
                "15 de enero de 2024 a las 14:30",
                "2024-01-15T14:30:00Z",
                es(),
            ),
            (
                "20 de diciembre de 2024 18:00",
                "2024-12-20T18:00:00Z",
                es(),
            ),
            ("15/01/2024 09:45", "2024-01-15T09:45:00Z", es()),
            // === "el" prefix ===
            ("el 1 de enero de 2024", "2024-01-01T00:00:00Z", es()),
            ("El 15 de marzo de 2025", "2025-03-15T00:00:00Z", es()),
        ];

        for (input, expected, opts) in cases {
            assert_date(input, expected, opts);
        }
    }

    fn es() -> Option<ParseCfg> {
        Some(ParseCfg {
            lang: Lang::Es,
            ..Default::default()
        })
    }

    fn generate_relative_date_test_cases_es() -> Vec<String> {
        let mut cases: Vec<String> = Vec::new();

        // Core Spanish relatives
        let core_phrases = ["ahora", "hoy", "mañana", "ayer"];
        cases.extend(core_phrases.iter().map(|&s| s.to_string()));

        // Numbers (Spanish uses comma for decimal)
        let numbers = [
            "1", "2", "3", "5", "10", "42", "0,5", "1,5", "2,5", "3,75", "1_000",
        ];

        // Spanish unit forms
        let units = [
            "segundo", "segundos", "seg", "minuto", "minutos", "min", "hora", "horas", "hr", "hrs",
            "día", "días", "semana", "semanas", "mes", "meses", "año", "años",
        ];

        let past_prefix = "hace ";
        let future_prefix = "en ";

        for num in numbers {
            for unit in units {
                // Past: "hace 5 minutos"
                cases.push(format!("{}{} {}", past_prefix, num, unit));

                // Future: "en 5 minutos"
                cases.push(format!("{}{} {}", future_prefix, num, unit));
            }
        }

        // Multi-unit Spanish cases
        let multi_unit_cases = [
            "hace 2 horas y 30 minutos",
            "en 1 día y 12 horas",
            "hace 3 semanas y 4 días",
            "en 2 días 3 horas",
            "hace 1 día, 2 horas y 30 minutos",
            "en 45 minutos y 15 segundos",
            "hace 2 semanas 3 días 4 horas",
            "en 1 semana y 2 días",
            "hace 3 días 5 horas",
            // Compact forms
            "hace 2 días 3 h",
            "en 1 semana 2 días",
            "hace 45 min 15 seg",
            "en 2 horas 30 minutos",
        ];
        cases.extend(multi_unit_cases.iter().map(|&s| s.to_string()));

        cases
    }

    #[test]
    fn relative_date_parser_comprehensive_es() {
        let cases = generate_relative_date_test_cases_es();
        let opts = Some(ParseCfg {
            lang: Lang::Es,
            ref_time: Some(Dt::new(5_000_000, 0)),
            ..Default::default()
        });

        for input in cases {
            let result = Dt::from_str_parse(input.trim(), &opts);
            assert!(
                result.is_ok(),
                "Failed to parse Spanish relative date: '{}'",
                input
            );
        }
    }

    #[test]
    fn es_durations() {
        fn assert_duration(input: &str, expected_millis: i64) {
            let dur = Dt::from_duration_str(input.trim(), Lang::Es)
                .unwrap_or_else(|e| panic!("Failed '{}': {}", input, e));

            assert_eq!(dur.to_ms() as i64, expected_millis, "Input: '{}'", input);
        }

        let cases: Vec<(&str, i64)> = vec![
            // === Basics ===
            ("5 segundos", 5000),
            ("30 minutos", 1_800_000),
            ("2 horas", 7_200_000),
            ("1 día", 86_400_000),
            ("2 semanas", 1_209_600_000),
            ("6 meses", 15_778_800_000),
            ("1 año", 31_557_600_000),
            // === Abbreviations ===
            ("2sem", 1_209_600_000),
            // === With "y" (and) ===
            ("2 horas y 30 minutos", 9_000_000),
            ("1 día y 12 horas", 129_600_000),
            ("3 semanas y 4 días", 2_160_000_000),
            // === Spanish decimal comma ===
            ("1,5 horas", 5_400_000),
            ("2,5 días", 216_000_000),
            ("0,5 minuto", 30_000),
            ("3,75 horas", 13_500_000),
            // === Mixed / natural Spanish ===
            ("1 día, 2 horas y 30 minutos", 95_400_000),
            // === Negatives ===
            ("-5 minutos", -300_000),
            ("-2 horas y 30 minutos", -9_000_000),
            ("-1,5 días", -129_600_000),
            // === Edge cases ===
            ("  2 horas  ", 7_200_000),
            ("1día 12h", 129_600_000),
            ("2h30min", 9_000_000),
            ("horas 3", 10_800_000),
        ];

        for (input, expected) in cases {
            assert_duration(input, expected);
        }
    }
}
