use crate::{
    ClassifiedDate, DateClassification, DateOrder, DateParseMode, DetectedDateOrder, DtError,
    DtStdError, MAX_DATE_STRING_LEN, ParseCfg, TimeParts, TimePoint, classify_date,
    default_date_parse_options, generate_ambiguous_day_first_candidates,
    generate_ambiguous_month_first_candidates, generate_ambiguous_year_first_candidates,
    generate_unambiguous_candidates, is_week_date_missing_weekday,
    parse_pure_numeric_unix_timestamp, parse_syslog_no_year, parse_week_date_no_weekday,
    parse_yyyy_mm, smart_detect_date_order, try_pure_numeric,
};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};

impl TimePoint {
    pub fn from_str_parse(
        s: &str,
        opts: &Option<ParseCfg>,
        verbose_err: bool,
    ) -> Result<TimePoint, DtStdError> {
        let opts: &ParseCfg = opts
            .as_ref()
            .unwrap_or_else(|| default_date_parse_options());

        if s.is_empty() || s.len() > MAX_DATE_STRING_LEN {
            return Err(DtStdError::date(
                s.to_string(),
                "Date either empty or longer than max len".to_string(),
                &opts,
                verbose_err,
            ));
        }

        let lang = opts.lang;
        let ref_time = &opts.ref_time;

        let lowered: Cow<str> = if opts.to_lower {
            Cow::Owned(s.to_lowercase())
        } else {
            Cow::Borrowed(s)
        };

        let classification = match classify_date(&lowered, lang, ref_time) {
            Ok(ClassifiedDate::Parsed(time_point)) => return Ok(time_point),
            Ok(ClassifiedDate::Cls(c)) => c,
            Err(e) => {
                // std::eprintln!("{}", e);
                return Err(DtStdError::date(s.to_string(), e, &opts, verbose_err));
            }
        };
        // let xx = &classification.date;
        // if xx != trimmed {
        //     eprintln!("NOT EQUAL: {:?}, {:?}", trimmed, xx);
        // }
        // eprintln!("BEFORE & AFTER: {:?}, {:?}", lowered, &classification.date);

        let normalized = &classification.date;

        let (mode, date_order) = if let Some(formats) = &opts.parse {
            if !formats.is_empty() {
                for fmt in formats {
                    if let Some(value) = Self::from_str(normalized, fmt, true, true, false) {
                        return Ok(value);
                    }
                }
                // None of the provided formats worked and mode is Explicit
                if opts.mode == DateParseMode::Explicit {
                    return Err(DtStdError::date(
                        s.to_string(),
                        "Could not parse using the provided explicit formats".to_string(),
                        &opts,
                        verbose_err,
                    ));
                }
            }
            (opts.mode, opts.order)
        } else {
            (opts.mode, opts.order)
        };

        // std::eprintln!("INPUT CLS: {:?}", classification);

        if classification.is_pure_numeric {
            match mode {
                DateParseMode::UnixTimestamp => {
                    if let Some(dt) = parse_pure_numeric_unix_timestamp(
                        &normalized,
                        classification.num_non_decimal_digits as usize,
                    ) {
                        return Ok(dt);
                    }
                }
                _ => {
                    if let Some(dt) = try_pure_numeric(
                        &normalized,
                        classification.num_digits,
                        classification.num_non_decimal_digits,
                        classification.is_decimal,
                        mode,
                    ) {
                        // std::eprintln!("NUMERIC INPUT SUCCESS: {:?}", s);
                        return Ok(dt);
                    }
                }
            }
        }
        if !classification.has_year {
            if let Some(dt) = parse_syslog_no_year(&normalized, lang, ref_time) {
                return Ok(dt);
            }
        }
        if is_week_date_missing_weekday(&classification) {
            // std::eprintln!("IS WEEK DATE MISSING WEEKDAY: {:?}", s);
            if let Some(dt) = parse_week_date_no_weekday(&classification.date, lang, ref_time) {
                return Ok(dt);
            }
        }
        if let Some(dt) = try_unambiguous(&normalized, &classification) {
            return Ok(dt);
        }
        // std::eprintln!("done trying unambiguous");
        if let Some(dt) = match date_order {
            DateOrder::Smart => {
                let order = smart_detect_date_order(&normalized, &classification);
                let mut result: Option<TimePoint>;

                match order {
                    DetectedDateOrder::DayFirst => {
                        result = try_compatible_formats(
                            &normalized,
                            generate_ambiguous_day_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying day first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_month_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying month first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_year_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying year first: {:?}", result);
                        }
                    }
                    DetectedDateOrder::MonthFirst => {
                        result = try_compatible_formats(
                            &normalized,
                            generate_ambiguous_month_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying month first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_day_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying day first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_year_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying year first: {:?}", result);
                        }
                    }
                    DetectedDateOrder::YearFirst => {
                        result = try_compatible_formats(
                            &normalized,
                            generate_ambiguous_year_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying year first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_day_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying day first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                &normalized,
                                generate_ambiguous_month_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying month first: {:?}", result);
                        }
                    }
                }

                result
            }
            DateOrder::YearFirst => try_compatible_formats(
                &normalized,
                generate_ambiguous_year_first_candidates(&classification),
            ),
            DateOrder::DayFirst => try_compatible_formats(
                &normalized,
                generate_ambiguous_day_first_candidates(&classification),
            ),
            DateOrder::MonthFirst => try_compatible_formats(
                &normalized,
                generate_ambiguous_month_first_candidates(&classification),
            ),
        } {
            return Ok(dt);
        }
        // std::eprintln!("NOW trying numeric timestamp");
        if classification.is_pure_numeric && mode != DateParseMode::UnixTimestamp {
            if let Some(dt) = parse_pure_numeric_unix_timestamp(
                &normalized,
                classification.num_non_decimal_digits as usize,
            ) {
                return Ok(dt);
            }
        }
        Err(DtStdError::date(
            s.to_string(),
            "Could not parse using any method".to_string(),
            &opts,
            verbose_err,
        ))
    }

    /// Same parsing logic as `TimePoint::from_str`, but returns milliseconds since
    /// the library epoch: 2000-01-01 12:00:00 UTC (on the UTC clock type).
    ///
    /// Returns `Some(millis)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_ms(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        TimePoint::from_str_parse(s, opts, false)
            .ok()
            .map(|tp| tp.as_ms())
    }

    /// Same parsing logic as `TimePoint::from_str`, but returns milliseconds since
    /// the UNIX epoch: (1970-01-01 00:00:00 UTC).
    ///
    /// Returns `Some(millis)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_unix_ms(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        TimePoint::from_str_parse(s, opts, false)
            .ok()
            .map(|tp| tp.to_unix_ms())
    }

    #[inline(always)]
    pub fn from_str(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Option<TimePoint> {
        // std::eprintln!("TRYING: {}, FOR: {}", fmt, s);

        let components = TimeParts::from_str(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        );

        // std::eprintln!("RESULT from_str: {:?}", components);

        // Convert Result<TimeParts, DtError> -> Result<TimePoint, DtError>
        let time_point_result: Result<TimePoint, DtError> =
            components.and_then(|p| p.to_time_point(None));

        // Print the error if there is one (this is what you asked for)
        if let Err(_) = &time_point_result {
            // std::eprintln!("ERROR in to_time_point: {:?}", e);
        }

        // Finally convert Result -> Option (as your function signature requires)
        time_point_result.ok()
    }
}

/// Core zero-allocation helper (updated to match the new `&str` signature).
///
/// The `fmt` we get from the iterator is still `'static`, but it coerces automatically
/// to `&str`, so everything continues to work.
#[inline]
pub(crate) fn try_compatible_formats<'a, I>(s: &str, formats: I) -> Option<TimePoint>
where
    I: IntoIterator<Item = String>,
{
    formats
        .into_iter()
        .find_map(|fmt| TimePoint::from_str(s, &fmt, true, true, false))
}

#[inline]
pub(crate) fn try_unambiguous(s: &str, classification: &DateClassification) -> Option<TimePoint> {
    if matches!(classification.ascii_len, 6 | 7 | 8) {
        if let Some(dt) = parse_yyyy_mm(&s.as_bytes()) {
            return Some(dt);
        }
    }
    try_compatible_formats(s, generate_unambiguous_candidates(&classification))
}
