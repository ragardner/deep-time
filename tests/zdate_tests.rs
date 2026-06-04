#![allow(clippy::all, clippy::pedantic, clippy::restriction, warnings)]

#[cfg(feature = "lang")]
mod tests {
    use deep_time::{Dt, Lang, Mode, Order, ParseCfg, Scale, TimeParts};

    #[test]
    fn print_stuff() {}

    #[cfg(feature = "tz")]
    #[test]
    fn roundtrip_gap_boundary_new_york() {
        let our_input = "2023-03-12 02:00:00 America/New_York";
        let expected_snapped = "2023-03-12 03:00:00 America/New_York";

        // Parse the non-existent local time (should succeed via lenient gap handling)
        let our_dt: Dt = Dt::from_str_parse(our_input, &None)
            .expect("parse should succeed (lenient gap handling)");

        // Verify internal representation (the snapped UTC instant)
        assert_eq!(
            our_dt.to_unix().to_sec(),
            1678604400,
            "internal unix timestamp should be the snapped UTC instant"
        );

        // Format back using the IANA zone
        let fmt = "%Y-%m-%d %H:%M:%S %Q";
        let output = our_dt
            .to_str_in_tz(fmt, "America/New_York")
            .expect("to_str_in_tz should succeed");

        // === THE KEY REGRESSION ASSERT ===
        assert_eq!(
            output, expected_snapped,
            "gap time should silently snap forward to the next valid local time (post-DST)"
        );

        // Bonus: verify the round-trip is stable (parse → format → parse → format)
        let our_dt2: Dt =
            Dt::from_str_parse(&output, &None).expect("second parse should also succeed");
        let output2 = our_dt2
            .to_str_in_tz(fmt, "America/New_York")
            .expect("second format should succeed");

        assert_eq!(output2, expected_snapped, "round-trip must be stable");
    }

    #[cfg(feature = "tz")]
    #[test]
    fn tz_output() {
        use deep_time::{Dt, Scale};

        let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
        let s = x
            .to_str_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
            .unwrap();
        let b = x
            .to_str_lite_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York")
            .unwrap();

        assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
        assert_eq!(
            b.as_str().unwrap(),
            "Saturday, January 01, 2000 07:00:00 America/New_York"
        );
    }

    fn assert_date(input: &str, expected_rfc3339: &str, opts: Option<ParseCfg>) {
        let dt = Dt::from_str_parse(input.trim(), &opts)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));
        let actual = dt.to_str_rfc3339().unwrap();

        assert_eq!(actual, expected_rfc3339, "Input: {}", input);
    }

    fn assert_millis(input: &str, expected_millis: i128, opts: Option<ParseCfg>) {
        let millis = Dt::str_to_unix_ms(input, &opts)
            .unwrap_or_else(|| panic!("Failed millis parse: {}", input));
        assert_eq!(millis, expected_millis, "Input: {}", input);
    }

    fn assert_fails(input: &str, opts: Option<ParseCfg>) {
        assert!(
            Dt::from_str_parse(input, &opts).is_err(),
            "Expected failure: {}",
            input
        );
    }

    fn generate_date_test_cases() -> Vec<(String, String, Option<ParseCfg>)> {
        let mut cases = Vec::new();

        // ================================================================
        // 1. Core components – add new formats here for instant 10x coverage
        // ================================================================

        let date_only_bases = [
            "2024-03-14",
            "2024.03.14",
            "2024/03/14",
            "2024 03 14",
            "20240314",
            "14Mar24",
            "14-Mar-24",
            "14 Mar 2024",
            "Mar 14 2024",
            "March 14, 2024",
            "14 March 2024",
            "14-March-2024",
            "14/Mar/2024",
            "2024-W11-4",
            "2024-074",
            "2024/074",
            "2024.074",
            "240314",
            // "202403",
        ];

        let dt_separators = [" ", "T", "", ":"]; // ← ":" added (per request)

        let time_variants = [
            ("", "T00:00:00Z"), // date-only
            ("15:30", "T15:30:00Z"),
            ("15:30:45", "T15:30:45Z"),
            ("15:30:45.123", "T15:30:45.123Z"),
            ("15:30:45.123456", "T15:30:45.123456Z"),
            ("15:30:45.123456789", "T15:30:45.123456789Z"),
            ("03:30:45 AM", "T03:30:45Z"),
            ("03:30:45 PM", "T15:30:45Z"),
            ("03:30:45.123456789 PM", "T15:30:45.123456789Z"),
        ];

        let tz_variants = ["", "+0000", " +0000", "+00:00", " +00:00", "Z", "-0000"];

        let prefixes = ["", " ", "Thu ", "Thu. ", "Thursday, ", "Thu, "];

        // ================================================================
        // 2. Generate massive combinatorial coverage
        // ================================================================
        for lang in [Lang::En, Lang::Es, Lang::De, Lang::Fr] {
            let opts = ParseCfg {
                order: Order::Year,
                lang: lang,
                ..Default::default()
            };
            for date in date_only_bases {
                for prefix in prefixes {
                    for dt_sep in dt_separators {
                        for (time_in, time_expected) in time_variants {
                            for tz in tz_variants {
                                // === Prevent invalid date-only + timezone (no time) ===
                                if time_in.is_empty()
                                    && !tz.is_empty()
                                    && (dt_sep.is_empty() || dt_sep == " ")
                                {
                                    continue;
                                }

                                // === Prevent malformed date-only with T or : separator ===
                                //     (now covers both "2024-03-14T" and "2024-03-14:")
                                if time_in.is_empty() && (dt_sep == "T" || dt_sep == ":") {
                                    continue;
                                }

                                // === Prevent malformed 12-hour AM/PM + compact timezone suffix ===
                                //     e.g. "03:30:45 PMZ", "PM+0000", "PM-0000", etc.
                                //     (still allows valid spaced variants like "PM +0000")
                                if (time_in.contains("PM") || time_in.contains("AM"))
                                    && !tz.is_empty()
                                    && !tz.starts_with(' ')
                                {
                                    continue;
                                }

                                // === Prevent compact NAMED date formats glued directly to time ===
                                //     e.g. "14Mar2415:30", "14-Mar-2415:30", "March 14, 202415:30"
                                //     (named = contains letters; numeric compacts like "2024031415:30" are untouched)
                                if dt_sep.is_empty()
                                    && !time_in.is_empty()
                                    && date.chars().any(|c| c.is_alphabetic())
                                {
                                    continue;
                                }

                                // === Prevent "Thu 2024 03 14:15:30" and similar bad cases ===
                                //     Day name + purely numeric spaced date (YYYY MM DD) + time glued with ":"
                                //     (or empty separator). These produce only "2 date digit groups" after the
                                //     day name (2024 03) before the time starts, which violates the rule "if
                                //     there's only 2 date digit groups and a named then the
                                //     named should be the month, not day". We keep weekday prefixes only
                                //     with named-month dates like "Thu Mar 14 2024".
                                if !prefix.trim().is_empty()
                                    && (prefix.contains("Thu") || prefix.contains("Thursday"))
                                    && date.contains(' ')
                                    && !date.chars().any(|c| c.is_alphabetic())
                                    && (dt_sep == ":" || dt_sep.is_empty())
                                    && !time_in.is_empty()
                                {
                                    continue;
                                }

                                // === Prevent "2024 03 14:15:30" ===
                                //     Purely numeric spaced date (YYYY MM DD) glued directly to time with ":"
                                //     It is now completely excluded from
                                //     the generated test cases while leaving every other valid combination intact.
                                if date.contains(' ')
                                    && !date.chars().any(|c| c.is_alphabetic())
                                    && dt_sep == ":"
                                    && !time_in.is_empty()
                                {
                                    continue;
                                }

                                // === Prevent julians from being produced with times that dont have a time connector ===
                                //     e.g. "2024-07415:30" - should not be produced.
                                //     Only blocks the empty (glued) case; T, space, and colon are still allowed.
                                if date.len() == 8
                                    && matches!(
                                        date.chars().nth(4),
                                        Some('-') | Some('/') | Some('.')
                                    )
                                    && date[5..].chars().all(|c| c.is_ascii_digit())
                                    && !time_in.is_empty()
                                    && dt_sep.is_empty()
                                {
                                    continue;
                                }

                                // === Prevent all pure numeric YYYYMMDDHHMM* format tests ===
                                //     (ONLY REMOVE THE PURE NUMERIC CASES — separators, connectors, T, :, ., space, etc. are OK!)
                                //     This blocks only fully-glued digit-only cases like "202403141530" or "240314153045".
                                //     Cases with any separator/connector (including the . in millis) are left untouched.
                                if dt_sep.is_empty()
                                    && !time_in.is_empty()
                                    && date.chars().all(|c| c.is_ascii_digit())
                                    && time_in.chars().all(|c| c.is_ascii_digit())
                                {
                                    continue;
                                }

                                // === Prevent 6-digit pure numeric date (e.g. 240314) glued directly to time
                                //     without any connector
                                //     Bad examples that are now blocked:
                                //       "24031415:30:00", "24031415:30:45", "24031415:30:45.123"
                                //     Allowed (they contain a connector):
                                //       "240314T15:30+0000", "240314 15:30", "240314:15:30:00", etc.
                                if date.len() == 6
                                    && date.chars().all(|c| c.is_ascii_digit())
                                    && dt_sep.is_empty()
                                    && !time_in.is_empty()
                                {
                                    continue;
                                }

                                let input =
                                    format!("{}{}{}{}{}", prefix, date, dt_sep, time_in, tz)
                                        .trim()
                                        .to_string();

                                let expected = if time_in.is_empty() {
                                    "2024-03-14T00:00:00Z".to_string()
                                } else {
                                    format!("2024-03-14{}", time_expected)
                                };

                                cases.push((input, expected, Some(opts.clone())));
                            }
                        }
                    }
                }
            }
        }

        // ================================================================
        // 3. Truly special / one-off cases
        // ================================================================

        let special_cases: Vec<(String, String, Option<ParseCfg>)> = vec![
            (
                "2024-W11".to_string(),
                "2024-03-11T00:00:00Z".to_string(),
                None,
            ),
            (
                "2024W11".to_string(),
                "2024-03-11T00:00:00Z".to_string(),
                None,
            ),
            (
                "2024-074T15:30:45.123456789Z".to_string(),
                "2024-03-14T15:30:45.123456789Z".to_string(),
                None,
            ),
            (
                "20240314T153045Z".to_string(),
                "2024-03-14T15:30:45Z".to_string(),
                None,
            ),
            (
                "2024-03-14T15:30:45Z".to_string(),
                "2024-03-14T15:30:45Z".to_string(),
                None,
            ),
            (
                "Thu Mar 14 15:30:45 2024".to_string(),
                "2024-03-14T15:30:45Z".to_string(),
                None,
            ),
            (
                "14/Mar/2024:15:30:45 +0000".to_string(),
                "2024-03-14T15:30:45Z".to_string(),
                None,
            ),
            (
                "20240314153045.123456789".to_string(),
                "2024-03-14T15:30:45.123456789Z".to_string(),
                None,
            ),
            // Parse-mode / Order / explicit format tests
            (
                "60400".to_string(),
                "2024-03-31T00:00:00Z".to_string(),
                Some(ParseCfg {
                    mode: Mode::Scientific,
                    ..Default::default()
                }),
            ),
            (
                "05/06/2024".to_string(),
                "2024-06-05T00:00:00Z".to_string(),
                Some(ParseCfg {
                    order: Order::Day,
                    ..Default::default()
                }),
            ),
            (
                "14/03/2024 15:30".to_string(),
                "2024-03-14T15:30:00Z".to_string(),
                Some(ParseCfg {
                    parse: Some(vec!["%d/%m/%Y %H:%M".to_string()]),
                    ..Default::default()
                }),
            ),
            ("2024".to_string(), "2024-01-01T00:00:00Z".to_string(), None),
            (
                "-2024-03-14".to_string(),
                "-2024-03-14T00:00:00Z".to_string(),
                None,
            ),
            (
                "Dec 31 23:59:59".to_string(),
                "2025-12-31T23:59:59Z".to_string(),
                Some(ParseCfg {
                    ref_time: Some(Dt::from_ymd(2025, 12, 31, 23, 59, 59, 0, Scale::UTC)),
                    ..Default::default()
                }),
            ),
        ];

        cases.extend(special_cases);
        cases
    }

    #[test]
    fn date_parser_roundtrip() {
        let tp1 = Dt::from_sec(5, Scale::LTC);
        let tp2 = Dt::from_sec(5, Scale::GPS);
        let xp1 = tp1
            .target(Scale::UTC)
            .to_str("%Y-%m-%dT%H:%M:%S%.f")
            .unwrap();
        let xp2 = tp2
            .target(Scale::UTC)
            .to_str("%Y-%m-%dT%H:%M:%S%.f")
            .unwrap();
        let res_tp1 = Dt::from_str(&xp1, "%Y-%m-%dT%H:%M:%S%.f", true, true, false).unwrap();
        let res_tp2 = Dt::from_str(&xp2, "%Y-%m-%dT%H:%M:%S%.f", true, true, false).unwrap();
        assert!(tp1 == res_tp1);
        assert!(tp2 == res_tp2);
    }

    #[test]
    fn round_trip_fixed_offsets() {
        for tp in [Dt::from_tai_sec(5), Dt::from_tai_sec(-5)] {
            let xp1 = tp
                .target(Scale::UTC)
                .to_str_in_offset("%Y-%m-%dT%H:%M:%S%.~f %:z", 3600)
                .unwrap();
            let tp2 = Dt::from_str_parse(&xp1, &None).unwrap();
            let xp2 = tp2
                .to_str_in_offset("%Y-%m-%dT%H:%M:%S%.~f %:z", 3600)
                .unwrap();
            let tp3 = Dt::from_str_parse(&xp2, &None).unwrap();
            assert_eq!(tp, tp3);
        }
    }

    #[test]
    fn date_parser_comprehensive() {
        let cases: Vec<(&str, &str, Option<ParseCfg>)> = vec![
            (
                "2440587.5",
                "1970-01-01T00:00:00Z",
                Some(ParseCfg {
                    mode: Mode::Scientific,
                    ..Default::default()
                }),
            ),
            (
                "2024-03-14 03:30:45.123 PM",
                "2024-03-14T15:30:45.123Z",
                None,
            ),
            ("2024.03.14", "2024-03-14T00:00:00Z", None),
            ("The 14th of March, 2024", "2024-03-14T00:00:00Z", None),
            ("2024-03-14", "2024-03-14T00:00:00Z", None),
            ("2024/03/14", "2024-03-14T00:00:00Z", None),
            ("2024 03 14", "2024-03-14T00:00:00Z", None),
            ("2024.03.14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            (
                "2024.03.14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14T15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14T15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14 15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14 15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("14Mar24", "2024-03-14T00:00:00Z", None),
            ("20240314", "2024-03-14T00:00:00Z", None),
            ("2024-W11", "2024-03-11T00:00:00Z", None),
            ("2024-W11", "2024-03-11T00:00:00Z", None),
            ("2024-074", "2024-03-14T00:00:00Z", None),
            ("2024/074", "2024-03-14T00:00:00Z", None),
            ("2024.074", "2024-03-14T00:00:00Z", None),
            ("14Mar2024", "2024-03-14T00:00:00Z", None),
            ("14-Mar-24", "2024-03-14T00:00:00Z", None),
            (" 14-Mar-24", "2024-03-14T00:00:00Z", None),
            ("2024-W11-4", "2024-03-14T00:00:00Z", None),
            ("2024-W11-4", "2024-03-14T00:00:00Z", None),
            ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
            ("14-Mar-2024", "2024-03-14T00:00:00Z", None),
            ("Mar 14 2024", "2024-03-14T00:00:00Z", None),
            ("2024 Mar 14", "2024-03-14T00:00:00Z", None),
            ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
            ("14-Mar-2024", "2024-03-14T00:00:00Z", None),
            ("20240314 15:30", "2024-03-14T15:30:00Z", None),
            ("2024-074 15:30", "2024-03-14T15:30:00Z", None),
            ("2024.074 15:30", "2024-03-14T15:30:00Z", None),
            ("2024/074 15:30", "2024-03-14T15:30:00Z", None),
            ("20240314T153045", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30", "2024-03-14T15:30:00Z", None),
            ("2024-03-14T15:30", "2024-03-14T15:30:00Z", None),
            ("14-03-2024 15:30", "2024-03-14T15:30:00Z", None),
            ("14.03.2024 15:30", "2024-03-14T15:30:00Z", None),
            ("2024 03 14 15:30", "2024-03-14T15:30:00Z", None),
            ("2024-074T15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-074 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024/074T15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024/074 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024.074T15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024.074 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024 Mar 14 15:30", "2024-03-14T15:30:00Z", None),
            ("14 Mar 2024 15:30", "2024-03-14T15:30:00Z", None),
            ("2024 Mar 14 15:30", "2024-03-14T15:30:00Z", None),
            ("20240314 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-03-1415:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024/03/14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024 03 14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14-03-2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14.03.2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-W11-4 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-W11-4 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14/Mar/2024:15:30:45", "2024-03-14T15:30:45Z", None),
            ("14-Mar-2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("Mar 14 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14 Mar 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024 Mar 14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024 Mar 14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14 Mar 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14-Mar-2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14 Mar, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("Mar 14, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024.03.14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            (
                "2024-03-14T15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14T15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14 15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14 15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("2024-03-1415:30:45.123", "2024-03-14T15:30:45.123Z", None),
            ("14 Mar 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("Mar 14 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("Mar 14, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("2024-03-14 15:30:45+0000", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45+0000", "2024-03-14T15:30:45Z", None),
            ("Thu Mar 14 15:30:45 2024", "2024-03-14T15:30:45Z", None),
            ("14 Mar, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45+00:00", "2024-03-14T15:30:45Z", None),
            (
                "2024-03-1415:30:45.123456",
                "2024-03-14T15:30:45.123456Z",
                None,
            ),
            ("14/Mar/2024:15:30:45 +0000", "2024-03-14T15:30:45Z", None),
            (
                "Thu Mar 14 15:30:45 +00:00 2024",
                "2024-03-14T15:30:45Z",
                None,
            ),
            ("Mar 14, 2024 15:30:45 +0000", "2024-03-14T15:30:45Z", None),
            ("Mar 14, 2024 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            (
                "Thu Mar 14 15:30:45 +0000 2024",
                "2024-03-14T15:30:45Z",
                None,
            ),
            (
                "Thu, 14 Mar 2024 15:30:45 +0000",
                "2024-03-14T15:30:45Z",
                None,
            ),
            (
                "Thu Mar 14 15:30:45 +00:00 2024",
                "2024-03-14T15:30:45Z",
                None,
            ),
            ("20240314T153045Z", "2024-03-14T15:30:45Z", None),
            ("2024-03-14T15:30:45Z", "2024-03-14T15:30:45Z", None),
            ("March 14, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("March 14, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            (
                "March 14, 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "March 14, 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("March 14, 2024", "2024-03-14T00:00:00Z", None),
            (
                "March 14, 2024 15:30:45 +00:00",
                "2024-03-14T15:30:45Z",
                None,
            ),
            (
                "March 14, 2024 15:30:45 +0000",
                "2024-03-14T15:30:45Z",
                None,
            ),
            (
                "March 14 2024 15:30:45 +00:00",
                "2024-03-14T15:30:45Z",
                None,
            ),
            (
                "Mar 14, 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Mar 14, 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Mar 14 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Mar 14 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("Thursday, 14 March 2024", "2024-03-14T00:00:00Z", None),
            ("Thursday 14 March 2024", "2024-03-14T00:00:00Z", None),
            ("Thursday, 14 Mar 2024", "2024-03-14T00:00:00Z", None),
            ("Thursday 14 Mar 2024", "2024-03-14T00:00:00Z", None),
            (
                "Thursday, 14 Mar 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Thursday 14 Mar 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Thu Mar 14 15:30:45.123456789 2024",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Thu Mar 14 15:30:45.123456789 +00:00 2024",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Thu Mar 14 15:30:45.123456789 +0000 2024",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("14 March 2024", "2024-03-14T00:00:00Z", None),
            ("14 March, 2024", "2024-03-14T00:00:00Z", None),
            ("14 March 2024", "2024-03-14T00:00:00Z", None),
            ("14 March, 2024", "2024-03-14T00:00:00Z", None),
            ("14-March-2024", "2024-03-14T00:00:00Z", None),
            ("14 March 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14 March, 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14 March 2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("14-March-2024 15:30:45", "2024-03-14T15:30:45Z", None),
            ("Mar 14, 2024 15:30:45 +00:00", "2024-03-14T15:30:45Z", None),
            (
                "Mar 14, 2024 15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "Mar 14, 2024 15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar, 2024 15:30:45.123456789 +00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar, 2024 15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 March 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 March, 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 March 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14-March-2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("14 March 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            ("14 March, 2024 03:30:45 PM", "2024-03-14T15:30:45Z", None),
            (
                "14 March 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 March, 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar, 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar, 2024 03:30:45.123456789 PM",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14 Mar 2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14-Mar-2024 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "14/Mar/2024:15:30:45.123456789 +0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "20240314T153045.123456789Z",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074T15:30:45.123456789Z", // here
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024.074T15:30:45.123456789Z",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/074T15:30:45.123456789Z",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14T15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14T15:30:45.123456789+0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-03-1415:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/03/14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "20240314T153045.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "20240314153045.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "20240314 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024 March 14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024 Mar 14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            ("2024 March 14 15:30:45", "2024-03-14T15:30:45Z", None),
            ("2024 March 14", "2024-03-14T00:00:00Z", None),
            (
                "2024 Mar 14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024 03 14 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074T15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074T15:30:45.123456789+0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074 15:30:45.123456789+0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074T15:30:45.123456789+00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024-074 15:30:45.123456789+00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024.074T15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024.074 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024.074T15:30:45.123456789+0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024.074T15:30:45.123456789+00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/074T15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/074 15:30:45.123456789",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/074T15:30:45.123456789+0000",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            (
                "2024/074T15:30:45.123456789+00:00",
                "2024-03-14T15:30:45.123456789Z",
                None,
            ),
            // Common formats
            ("2024-03-14", "2024-03-14T00:00:00Z", None),
            ("14 Mar 2024", "2024-03-14T00:00:00Z", None),
            ("15 Mar 2024 14:30", "2024-03-15T14:30:00Z", None),
            ("Mar 14, 2024", "2024-03-14T00:00:00Z", None),
            ("2024/03/14", "2024-03-14T00:00:00Z", None),
            ("14.03.2024", "2024-03-14T00:00:00Z", None),
            ("14/03.2024", "2024-03-14T00:00:00Z", None),
            ("14/03.2024T00:00  -1", "2024-03-14T01:00:00Z", None),
            // Pure-numeric special cases (date-only or epoch — kept; only full datetime smashed cases were removed)
            ("240314", "2024-03-14T00:00:00Z", None),
            ("202403", "2024-03-01T00:00:00Z", None),
            ("2024073", "2024-03-13T00:00:00Z", None),
            ("24073", "2024-03-13T00:00:00Z", None),
            ("60400", "2024-03-31T00:00:00Z", None),
            ("2460000", "2023-02-25T00:00:00Z", None),
            ("1700000000", "2023-11-14T22:13:20Z", None),
            ("1700000000000", "2023-11-14T22:13:20Z", None),
            // Parse modes
            (
                "60400",
                "2024-03-31T00:00:00Z",
                Some(ParseCfg {
                    parse: None,
                    mode: Mode::Scientific,
                    order: Order::default(),
                    ..Default::default()
                }),
            ),
            (
                "24073",
                "2024-03-13T00:00:00Z",
                Some(ParseCfg {
                    parse: None,
                    mode: Mode::Legacy,
                    order: Order::default(),
                    ..Default::default()
                }),
            ),
            // Order
            (
                "05/06/2024",
                "2024-06-05T00:00:00Z",
                Some(ParseCfg {
                    parse: None,
                    mode: Mode::Scientific,
                    order: Order::Day,
                    ..Default::default()
                }),
            ),
            (
                "05/06/2024",
                "2024-05-06T00:00:00Z",
                Some(ParseCfg {
                    order: Order::Month,
                    ..Default::default()
                }),
            ),
            // Year-only
            ("2024", "2024-01-01T00:00:00Z", None),
            ("24", "2024-01-01T00:00:00Z", None),
            ("2024.074", "2024-03-14T00:00:00Z", None),
            // Negative years — Jiff pads negative years to 6 digits (Temporal/ISO rule)
            ("-2024-03-14", "-2024-03-14T00:00:00Z", None),
            ("-2025/01/01", "-2025-01-01T00:00:00Z", None),
            ("-0001-01-01", "-0001-01-01T00:00:00Z", None),
            // Syslog no-year
            (
                "Dec 31 23:59:59",
                "2025-12-31T23:59:59Z",
                Some(ParseCfg {
                    ref_time: Some(Dt::from_ymd(2025, 12, 31, 23, 59, 59, 0, Scale::UTC)),
                    ..Default::default()
                }),
            ),
            // Explicit format
            (
                "14/03/2024 15:30",
                "2024-03-14T15:30:00Z",
                Some(ParseCfg {
                    parse: Some(vec!["%d/%m/%Y %H:%M".to_string()]),
                    ..Default::default()
                }),
            ),
            (
                "２０２６年４月５日",
                "2026-04-05T00:00:00Z",
                Some(ParseCfg {
                    order: Order::Year,
                    ..Default::default()
                }),
            ),
            (
                "߂߀߂߄-߀߄-߀߅",
                "2024-04-05T00:00:00Z",
                Some(ParseCfg {
                    order: Order::Year,
                    ..Default::default()
                }),
            ),
        ];

        for (input, expected, opts) in cases {
            assert_date(input, expected, opts);
        }
        let cases = generate_date_test_cases();
        for (input, expected, opts) in cases {
            assert_date(&input, &expected, opts);
        }
    }

    fn generate_relative_date_test_cases() -> Vec<String> {
        let mut cases: Vec<String> = Vec::new();

        let core_phrases = ["now", "today", "tomorrow", "yesterday"];
        cases.extend(core_phrases.iter().map(|&s| s.to_string()));

        let numbers = [
            "1", "2", "3", "5", "10", "42", "0.5", "1.5", "2,5", "3.75", "1_000",
        ];
        let units = [
            "sec", "second", "seconds", "min", "minute", "minutes", "hr", "hour", "hours", "day",
            "days", "wk", "week", "weeks", "mo", "month", "months", "yr", "year", "years",
        ];

        let past_suffixes = [" ago"];
        let future_prefixes = ["in "];

        for num in numbers {
            for unit in units {
                // past forms
                for suffix in past_suffixes {
                    cases.push(format!("{} {}{}", num, unit, suffix));
                }
                for prefix in future_prefixes {
                    if !prefix.is_empty() || num.parse::<f64>().unwrap_or(0.0) != 1.0 {
                        cases.push(format!("{}{} {}", prefix, num, unit).trim().to_string());
                    }
                }
            }
        }

        let multi_unit_cases = [
            "2 hours and 30 minutes ago",
            "1 day 12 hours from now",
            "3 weeks 4 days later",
            "1day ,5hr ago",
            "2hrs30min from now",
            "45min 15sec later",
            "1 day, 2 hours, and 30 minutes ago",
            "the 1.5 days ago",
            "in 2day 3hr 45min",
            "1 week and 2 days from now",
            "2 weeks 3 days 4 hours ago",
            "in 1 week and 2 days",
            "3 days 5 hours later",
        ];
        cases.extend(multi_unit_cases.iter().map(|&s| s.to_string()));

        cases
    }

    #[test]
    fn relative_date_parser_comprehensive() {
        let cases = generate_relative_date_test_cases();
        let opts = Some(ParseCfg {
            ref_time: Some(Dt::from_tai_sec(5_000_000)),
            ..Default::default()
        });

        for input in cases {
            let result = Dt::from_str_parse(input.trim(), &opts);
            // eprintln!("Tried: {}, got result: {:?}", &input, result);
            assert!(result.is_ok(), "Failed to parse relative date: '{}'", input);
        }
    }

    #[test]
    fn date_millis_and_errors() {
        assert_millis("2024-03-14", 1710374400000, None);
        assert_millis("1700000000000", 1700000000000, None);

        assert_fails("not-a-date", None);
        assert_fails("2024-13-01", None);
    }
}
