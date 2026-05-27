use crate::{
    ClassifiedDate, DateClassification, Dt, DtErr, DtErrKind, MAX_DATE_STRING_LEN, Mode, Order,
    OrderFirst, ParseCfg, Scale, an_err, classify_date, default_date_parse_options,
    generate_ambiguous_day_first_candidates, generate_ambiguous_month_first_candidates,
    generate_ambiguous_year_first_candidates, generate_unambiguous_candidates,
    is_week_date_missing_weekday, parse_pure_numeric_unix_timestamp, parse_syslog_no_year,
    parse_week_date_no_weekday, parse_yyyy_mm, smart_detect_date_order, try_pure_numeric,
};
use alloc::borrow::Cow;
use alloc::string::String;

impl Dt {
    /// Automatically parses datetime [`str`] into a [`Dt`] by guessing and generating the format. Supports the vast
    /// majority of date formats.
    ///
    /// Requires the `"alloc"` feature.
    ///
    /// ## Parameters
    ///
    /// - `s`: The string to parse. Must be non-empty and no longer than 255 bytes. Empty strings or overly
    ///   long inputs return an error.
    /// - `opts`: Optional [`ParseCfg`]. Pass `None` to use the defaults.
    ///
    /// ## Configuration Options (via [`ParseCfg`])
    ///
    /// | Field          | Default     | Effect |
    /// |----------------|-------------|--------|
    /// | `lang`         | `En`        | Language, scroll down to see currently supported languages |
    /// | `order`        | `Smart`     | How to resolve ambiguous numeric dates like `01/02/03` |
    /// | `mode`         | `Auto`      | Special handling for purely numeric inputs |
    /// | `parse`        | `None`      | If provided, these exact `strftime`-style formats are tried **first** (and exclusively if `mode` is `Explicit`) |
    /// | `relative`     | `true`      | Enable phrases like "tomorrow", "next Friday", "in 3 days" |
    /// | `ref_time`     | `None`      | Reference time for relative dates and syslog-style "no-year" dates (uses system time if `std` feature is enabled) |
    /// | `to_lower`     | `true`      | Automatically lowercase the input, set to `false` only if it's already lowercase |
    ///
    /// ## Purely Numeric Inputs
    ///
    /// When the input consists **only** of digits (and optionally a decimal point),
    /// the parser uses a fast, mode-aware path before trying any other strategies.
    /// The exact interpretation depends on the number of digits and the selected `mode`.
    ///
    /// | Digits | Example(s)               | `Mode`          | Interpreted as                          | Notes |
    /// |--------|--------------------------|-----------------|-----------------------------------------|-------|
    /// | 1–4    | `2024`, `24`, `5`        | `Auto`/`Legacy` | Year (2-digit uses 2000/1900 pivot)    | 1- and 3-digit years only work in `Scientific` |
    /// | 5      | `24123`, `60400`         | `Legacy`        | Ordinal date (YYDDD)                    | — |
    /// | 5      | `60400`, `60400.75`      | `Scientific`    | Modified Julian Date (MJD)              | Fractional days supported |
    /// | 5      | `24123`, `60400.75`      | `Auto`          | Ordinal (non-decimal) or MJD (decimal) | Smart default |
    /// | 6      | `240315`, `202403`       | `Auto`          | YYYYMM if plausible year, else YYMMDD   | Most common compact form |
    /// | 6      | `240315`                 | `Legacy`        | YYMMDD preferred                        | — |
    /// | 6      | `202403`                 | `Scientific`    | YYYYMM preferred                        | — |
    /// | 7      | `2024123`                | `Legacy`        | Ordinal date (YYYYDDD)                  | — |
    /// | 7      | `2460123`, `2460123.5`   | `Scientific`    | Julian Day (JD)                         | Fractional days supported |
    /// | 7      | `2024123`                | `Auto`          | Ordinal (integer) or JD (decimal)       | Smart default |
    /// | 10–11  | `1735689600`             | any             | Unix seconds                            | — |
    /// | 12–15  | `1735689600123`          | any             | Unix milliseconds                       | Most common high-precision case |
    /// | 16–18  | `1735689600123456`       | any             | Unix microseconds                       | — |
    /// | 19+    | `1735689600123456789`    | any             | Unix nanoseconds                        | Full precision |
    ///
    /// **Tip**: Use `Mode::UnixTimestamp` when you know the input is always a Unix timestamp.
    ///
    /// ## Ambiguous Numeric Dates
    ///
    /// Dates where the components could map to different orders (e.g. `01/02/03`,
    /// `3-4-5`, `15.03.24`, `2024.03.15`) are resolved via the `order` field:
    ///
    /// - **`Order::Smart`** (default) — Applies the fast heuristic described in [`Order::Smart`].
    ///   It strongly prefers modern/tech conventions (Year-first for compact/ISO-like data)
    ///   while handling the majority of international and US-style dates.
    ///
    /// - **`Order::Year`**, **`Order::Day`**, or **`Order::Month`** force a
    ///   specific interpretation and bypass the heuristic entirely.
    ///
    /// This combination of `Smart` + `Auto` mode gives the best real-world parsing
    /// success rate for mixed data sources.
    ///
    /// ## Other Supported Formats
    ///
    /// - **ISO 8601** and variants: `2024-03-15`, `2024-03-15T14:30:00Z`, `2024-03-15T14:30:00+01:00[Europe/Paris]`
    /// - **Named dates** (in supported languages): `15 March 2024`, `15 mars 2024`, `15. März 2024`, `15 de marzo de 2024`
    /// - **Week dates**: `2024-W15`, `2024-W15-3`, `2024W153` (missing weekday defaults to Monday)
    /// - **Syslog-style** (no year): `Mar  5 10:23:45` (year inferred from `ref_time`)
    /// - **Relative expressions**: `tomorrow`, `next Friday at 09:00`, `in 3 days`, `2 weeks ago`
    /// - **12-hour time**: `2:30 PM`, `14:30:45.123`
    /// - **Offsets and timezones**: `+0100`, `-05:30`, `Z`, IANA names in brackets
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, ParseCfg, Order, Mode, Lang};
    ///
    /// // Default smart parsing
    /// let dt = Dt::from_str_parse("2024-03-15 14:30:00", &None).unwrap();
    ///
    /// // German named date
    /// let cfg = ParseCfg { lang: Lang::De, ..Default::default() };
    /// let dt = Dt::from_str_parse("15. März 2024 um 14:30", &Some(cfg)).unwrap();
    ///
    /// // Force month-first
    /// let cfg = ParseCfg { order: Order::Month, ..Default::default() };
    /// let dt = Dt::from_str_parse("03/15/2024", &Some(cfg)).unwrap();
    ///
    /// // Pure numeric compact form
    /// let dt = Dt::from_str_parse("20240315", &None).unwrap(); // March 15, 2024
    ///
    /// // Unix timestamp (milliseconds)
    /// let cfg = ParseCfg { mode: Mode::UnixTimestamp, ..Default::default() };
    /// let dt = Dt::from_str_parse("1735689600123", &Some(cfg)).unwrap();
    ///
    /// // Explicit formats only (no fallback)
    /// let cfg = ParseCfg {
    ///     parse: Some(vec!["%d/%m/%Y".into(), "%Y-%m-%d".into()]),
    ///     mode: Mode::Explicit,
    ///     ..Default::default()
    /// };
    /// let dt = Dt::from_str_parse("15/03/2024", &Some(cfg)).unwrap();
    ///
    /// // Relative date
    /// let dt = Dt::from_str_parse("2 days from now", &None).unwrap();
    /// ```
    ///
    /// ## Notes
    ///
    /// - The `Smart` + `Auto` combination gives the best real-world success rate for mixed data.
    /// - All successfully parsed [`Dt`] values are stored with attosecond precision on the internal TAI timescale.
    /// - For maximum reproducibility in production code, prefer `ParseCfg` with `parse: Some(...)` and `mode: Explicit`.
    /// - Timezone handling (IANA names and fixed offsets) is fully supported.
    ///
    /// See also: [`ParseCfg`], [`Order`], [`Mode`], [`Lang`], [`Dt`],
    /// [`Dt::str_to_attos`], [`Dt::str_to_ms`], [`Dt::str_to_unix_ms`].
    ///
    /// ## Supported Languages:
    ///
    /// - En
    /// - De
    /// - Es
    /// - Fr
    ///
    pub fn from_str_parse(s: &str, opts: &Option<ParseCfg>) -> Result<Dt, DtErr> {
        let opts: &ParseCfg = opts
            .as_ref()
            .unwrap_or_else(|| default_date_parse_options());

        if s.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "empty"));
        } else if s.len() > MAX_DATE_STRING_LEN {
            return Err(an_err!(DtErrKind::InvalidInput, "too long: {}", s));
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
                return Err(an_err!(
                    DtErrKind::InvalidInput,
                    "{}",
                    s => e
                ));
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
                    if let Ok(value) = Self::from_str(normalized, fmt, true, true, false) {
                        return Ok(value);
                    }
                }
                // None of the provided formats worked and mode is Explicit
                if opts.mode == Mode::Explicit {
                    return Err(an_err!(DtErrKind::InvalidInput, "{}", s));
                }
            }
            (opts.mode, opts.order)
        } else {
            (opts.mode, opts.order)
        };

        // if s == "2006-04-02 02:30-05 America/Indiana/Vevay" {
        //     std::eprintln!("{:?}", classification);
        // }
        // std::eprintln!("{:?}", classification);

        if classification.is_pure_numeric {
            match mode {
                Mode::UnixTimestamp => {
                    if let Some(dt) = parse_pure_numeric_unix_timestamp(
                        normalized,
                        classification.num_non_decimal_digits as usize,
                    ) {
                        return Ok(dt);
                    }
                }
                _ => {
                    if let Some(dt) = try_pure_numeric(
                        normalized,
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
        if !classification.has_year
            && let Some(dt) = parse_syslog_no_year(normalized, lang, ref_time)
        {
            return Ok(dt);
        }

        if is_week_date_missing_weekday(&classification) {
            // std::eprintln!("IS WEEK DATE MISSING WEEKDAY: {:?}", s);
            if let Some(dt) = parse_week_date_no_weekday(&classification, lang, ref_time) {
                return Ok(dt);
            }
        }
        if let Some(dt) = try_unambiguous(normalized, &classification) {
            return Ok(dt);
        }
        // std::eprintln!("done trying unambiguous");
        if let Some(dt) = match date_order {
            Order::Smart => {
                let order = smart_detect_date_order(normalized, &classification);
                let mut result: Option<Dt>;

                match order {
                    OrderFirst::Day => {
                        result = try_compatible_formats(
                            normalized,
                            generate_ambiguous_day_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying day first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_month_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying month first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_year_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying year first: {:?}", result);
                        }
                    }
                    OrderFirst::Month => {
                        result = try_compatible_formats(
                            normalized,
                            generate_ambiguous_month_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying month first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_day_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying day first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_year_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying year first: {:?}", result);
                        }
                    }
                    OrderFirst::Year => {
                        result = try_compatible_formats(
                            normalized,
                            generate_ambiguous_year_first_candidates(&classification),
                        );
                        // std::eprintln!("done trying year first: {:?}", result);

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_day_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying day first: {:?}", result);
                        }

                        if result.is_none() {
                            result = try_compatible_formats(
                                normalized,
                                generate_ambiguous_month_first_candidates(&classification),
                            );
                            // std::eprintln!("done trying month first: {:?}", result);
                        }
                    }
                }

                result
            }
            Order::Year => try_compatible_formats(
                normalized,
                generate_ambiguous_year_first_candidates(&classification),
            ),
            Order::Day => try_compatible_formats(
                normalized,
                generate_ambiguous_day_first_candidates(&classification),
            ),
            Order::Month => try_compatible_formats(
                normalized,
                generate_ambiguous_month_first_candidates(&classification),
            ),
        } {
            return Ok(dt);
        }
        // std::eprintln!("NOW trying numeric timestamp");
        if classification.is_pure_numeric
            && mode != Mode::UnixTimestamp
            && let Some(dt) = parse_pure_numeric_unix_timestamp(
                normalized,
                classification.num_non_decimal_digits as usize,
            )
        {
            return Ok(dt);
        }
        Err(an_err!(DtErrKind::InvalidInput, "{}", s))
    }

    /// Same parsing logic as [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse),
    /// but returns attoseconds since the library epoch: 2000-01-01 12:00:00 UTC
    /// (on the UTC scale).
    ///
    /// Returns `Some(attos)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_attos(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        Dt::from_str_parse(s, opts).ok().map(|tp| tp.to_attos())
    }

    /// Same parsing logic as [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse),
    /// but returns milliseconds since the library epoch: 2000-01-01 12:00:00 UTC
    /// (on the UTC scale).
    ///
    /// Returns `Some(millis)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_ms(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        Dt::from_str_parse(s, opts).ok().map(|tp| tp.to_ms())
    }

    /// Same parsing logic as [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse),
    /// but returns nanoseconds since the library epoch: 2000-01-01 12:00:00 UTC
    /// (on the UTC scale).
    ///
    /// Returns `Some(nanos)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_ns(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        Dt::from_str_parse(s, opts).ok().map(|tp| tp.to_ns())
    }

    /// Same parsing logic as [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse),
    /// but returns milliseconds since the UNIX epoch: (1970-01-01 00:00:00 UTC).
    ///
    /// Returns `Some(millis)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_unix_ms(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        Dt::from_str_parse(s, opts).ok().map(|tp| {
            tp.to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
                .to_ms()
        })
    }

    /// Same parsing logic as [`Dt::from_str_parse`](../struct.Dt.html#method.from_str_parse),
    /// but returns nanoseconds since the UNIX epoch: (1970-01-01 00:00:00 UTC).
    ///
    /// Returns `Some(nanos)` on success (negative for pre-2000 dates) or `None`
    /// on any parse error.
    #[inline]
    pub fn str_to_unix_ns(s: &str, opts: &Option<ParseCfg>) -> Option<i128> {
        Dt::from_str_parse(s, opts).ok().map(|tp| {
            tp.to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
                .to_ns()
        })
    }
}

/// Core zero-allocation helper (updated to match the new `&str` signature).
///
/// The `fmt` we get from the iterator is still `'static`, but it coerces automatically
/// to `&str`, so everything continues to work.
#[inline]
pub(crate) fn try_compatible_formats<I>(s: &str, formats: I) -> Option<Dt>
where
    I: IntoIterator<Item = String>,
{
    // let mut dt = None;

    // for fmt in formats.into_iter() {
    //     // eprintln!("TRYING FMT: {}", fmt);
    //     if let Ok(parsed) = Dt::from_str(s, &fmt, true, true, false) {
    //         dt = Some(parsed);
    //         break;
    //     }
    //     // === DEBUG ===
    //     // eprintln!("Tried format: {:?}", fmt);
    // }

    // dt
    formats
        .into_iter()
        .find_map(|fmt| Dt::from_str(s, &fmt, true, true, false).ok())
}

#[inline]
pub(crate) fn try_unambiguous(s: &str, classification: &DateClassification) -> Option<Dt> {
    if matches!(classification.bytes_len, 6..=8)
        && let Some(dt) = parse_yyyy_mm(s.as_bytes())
    {
        return Some(dt);
    }
    try_compatible_formats(s, generate_unambiguous_candidates(classification))
}
