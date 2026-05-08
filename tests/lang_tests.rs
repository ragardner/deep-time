#[cfg(feature = "lang")]
#[cfg(test)]
mod lang_tests {
    use deep_time::{Dt, Lang, ParseCfg};

    fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
        let dt = Dt::from_str_parse(input.trim(), &opts)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let actual = dt.to_str_rfc3339().unwrap();

        assert_eq!(actual, expected_rfc3339, "Input: {}", input);
    }

    #[test]
    fn date_parser_comprehensive() {
        let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![(
            "1er janvier 2024",
            "2024-01-01T00:00:00Z",
            Some(ParseCfg {
                lang: Lang::Fr,
                ..Default::default()
            }),
        )];

        for (input, expected, opts) in cases {
            assert_date(input, expected, opts);
        }
    }
}
