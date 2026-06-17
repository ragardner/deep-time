#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "parse")]
mod tests {
    use deep_time::{Dt, Lang, ParseCfg, Scale, YmdHms};

    fn assert_rel(input: &str, output: Dt, exp: Dt) {
        assert_eq!(output.to_ymd(), exp.to_ymd(), "input was: {}", input)
    }

    #[test]
    fn en_relative_parsing() {
        // Reference time: Tuesday 16 June 2026, 12:00 UTC
        let ref_time = Dt::from_ymd(2026, 6, 16, Scale::UTC, 12, 0, 0, 0);

        let en_cfg = Some(ParseCfg {
            ref_time: Some(ref_time),
            ..Default::default()
        });

        let phrases = [
            (
                "next week wednesday",
                &en_cfg,
                Dt::from_ymd(2026, 6, 24, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "in 3 days",
                &en_cfg,
                Dt::from_ymd(2026, 6, 19, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "5 hours ago",
                &en_cfg,
                Dt::from_ymd(2026, 6, 16, Scale::UTC, 7, 0, 0, 0),
            ),
            (
                "in 2 weeks",
                &en_cfg,
                Dt::from_ymd(2026, 6, 30, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "3 days ago at 10:00",
                &en_cfg,
                Dt::from_ymd(2026, 6, 13, Scale::UTC, 10, 0, 0, 0),
            ),
            (
                "tomorrow",
                &en_cfg,
                Dt::from_ymd(2026, 6, 17, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "yesterday at 9:00",
                &en_cfg,
                Dt::from_ymd(2026, 6, 15, Scale::UTC, 9, 0, 0, 0),
            ),
            (
                "today at 15:30",
                &en_cfg,
                Dt::from_ymd(2026, 6, 16, Scale::UTC, 15, 30, 0, 0),
            ),
            (
                "tomorrow at 9:45",
                &en_cfg,
                Dt::from_ymd(2026, 6, 17, Scale::UTC, 9, 45, 0, 0),
            ),
            (
                "next week",
                &en_cfg,
                Dt::from_ymd(2026, 6, 23, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "last week",
                &en_cfg,
                Dt::from_ymd(2026, 6, 9, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "next Wednesday",
                &en_cfg,
                Dt::from_ymd(2026, 6, 17, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "last Friday",
                &en_cfg,
                Dt::from_ymd(2026, 6, 12, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "next week Wednesday",
                &en_cfg,
                Dt::from_ymd(2026, 6, 24, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "next week wed at 900",
                &en_cfg,
                Dt::from_ymd(2026, 6, 24, Scale::UTC, 9, 0, 0, 0),
            ),
            (
                "next Monday at 14:00",
                &en_cfg,
                Dt::from_ymd(2026, 6, 22, Scale::UTC, 14, 0, 0, 0),
            ),
            (
                "next month",
                &en_cfg,
                Dt::from_ymd(2026, 7, 16, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "PT tomorrow at 0900",
                &en_cfg,
                Dt::from_ymd(2026, 6, 17, Scale::UTC, 9, 0, 0, 0),
            ),
            (
                "week last monday",
                &en_cfg,
                Dt::from_ymd(2026, 6, 8, Scale::UTC, 12, 0, 0, 0),
            ),
            (
                "2 years ago",
                &en_cfg,
                Dt::from_ymd(2024, 6, 16, Scale::UTC, 12, 0, 0, 0),
            ),
        ];

        for (phrase, cfg, expected) in phrases {
            match Dt::from_str_parse(phrase, cfg) {
                Ok(dt) => {
                    assert_rel(phrase, dt, expected);
                }
                Err(e) => {
                    panic!("Failed to parse '{}': {}", phrase, e);
                }
            }
        }
    }
}
