use crate::{
    AS_PER_DAY, AS_PER_HOUR, AS_PER_MINUTE, AS_PER_MONTH, AS_PER_WEEK, AS_PER_YEAR,
    ATTOS_PER_MS_I128, ATTOS_PER_NS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_US_I128, Dt, DtErr,
    DtErrKind, Lang, LangData, Scale, SplitKeepWithPos, Token, an_err, lang_map, to_ascii_digit,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Sign {
    None,
    Positive,
    Negative,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Direction {
    #[default]
    None,
    Present,
    Future,
    Past,
}

#[derive(Debug, Clone, Copy)]
struct ParsedNumber {
    integer: i64,
    decimals: i64,
    decimal_places: u32,
    sign: Sign,
}

fn extract_number(part: &str, part_chars: &mut Vec<char>, d: char) -> Option<ParsedNumber> {
    part_chars.clear();
    part_chars.extend(part.chars());

    let decimal_pos: Option<usize> = {
        let dec_seps: Vec<(usize, char)> = {
            let mut ds = Vec::new();
            for (i, c) in part_chars.iter().copied().enumerate() {
                if i + 1 < part_chars.len()
                    && part_chars[i + 1].is_numeric()
                    && (c == '.' || c == d)
                {
                    ds.push((i, c));
                }
            }
            ds
        };
        match dec_seps.len() {
            0 => None,
            1 => {
                let (pos, sep_char) = dec_seps[0];
                if sep_char == d {
                    Some(pos) // language preference matches
                } else {
                    // other separator used → decide based on digit count after
                    let digits_after = part_chars[pos + 1..]
                        .iter()
                        .filter(|&&c| c.is_numeric())
                        .count();
                    match digits_after {
                        3 => None,
                        _ => Some(pos),
                    }
                }
            }
            _ => {
                // 1,000,000 last one is not a decimal point
                if dec_seps.iter().all(|&(_, c)| c == d) {
                    None
                // e.g. 1,000.55
                } else {
                    Some(dec_seps.last()?.0)
                }
            }
        }
    };

    let mut integer: i64 = 0;
    let mut decimals: i64 = 0;
    let mut decimal_places: u32 = 0;
    let mut seen_digit = false;
    let mut seen_decimal = false;
    let mut sign = Sign::None;

    // Scientific notation support (hard-coded 'e')
    let mut seen_e = false;
    let mut exponent: i32 = 0;
    let mut exp_sign = Sign::None;
    let mut exp_seen_digit = false;

    for (i, &ch) in part_chars.iter().enumerate() {
        match ch {
            'e' => {
                if seen_e || !seen_digit {
                    continue;
                }
                seen_e = true;
                continue;
            }
            '-' => {
                if seen_e {
                    if exp_seen_digit || exp_sign != Sign::None {
                        continue;
                    }
                    exp_sign = Sign::Negative;
                } else {
                    if seen_digit || sign != Sign::None {
                        continue;
                    }
                    sign = Sign::Negative;
                }
            }
            '+' => {
                if seen_e {
                    if exp_seen_digit || exp_sign != Sign::None {
                        continue;
                    }
                    exp_sign = Sign::Positive;
                } else {
                    if seen_digit || sign != Sign::None {
                        continue;
                    }
                    sign = Sign::Positive;
                }
            }
            ch if ch == '.' || ch == d => {
                if Some(i) == decimal_pos {
                    if seen_e || seen_decimal {
                        continue;
                    }
                    seen_decimal = true;
                    continue;
                }
                // otherwise it's a thousands separator → skip
                continue;
            }
            _ => {
                if !ch.is_numeric() {
                    continue;
                }
                let Some(digit_val) = to_ascii_digit(ch).and_then(|c| c.to_digit(10)) else {
                    continue;
                };
                let digit = digit_val as i64;

                if seen_e {
                    exponent = exponent.saturating_mul(10).saturating_add(digit as i32);
                    exp_seen_digit = true;
                } else if seen_decimal {
                    decimals = decimals.saturating_mul(10).saturating_add(digit);
                    decimal_places = decimal_places.saturating_add(1);
                } else {
                    integer = integer.saturating_mul(10).saturating_add(digit);
                }
                seen_digit = true;
            }
        }
    }

    if !seen_digit {
        return None;
    }

    // Apply scientific notation if 'e' was present
    if seen_e && exp_seen_digit {
        let exp = match exp_sign {
            Sign::Negative => -exponent,
            _ => exponent,
        };

        let mantissa_scale = if decimal_places > 0 {
            10i64.pow(decimal_places)
        } else {
            1
        };
        let significand = integer
            .saturating_mul(mantissa_scale)
            .saturating_add(decimals);

        let final_decimal_places = decimal_places as i32 - exp;

        if final_decimal_places > 0 {
            // Result is fractional → everything goes into the decimal part
            let dp = (final_decimal_places as u32).min(18);
            integer = 0;
            decimals = significand;
            decimal_places = dp;
        } else {
            // Result is integer (or larger) → shift decimal point right
            let shifts = (-final_decimal_places).max(0) as u32;
            let multiplier = 10i64.pow(shifts.min(18));
            integer = significand.saturating_mul(multiplier);
            decimals = 0;
            decimal_places = 0;
        }
    }

    Some(ParsedNumber {
        integer,
        decimals,
        decimal_places,
        sign,
    })
}

fn add_to_total(
    total_attos: &mut i128,
    num: ParsedNumber,
    unit_attos: i128,
    overall_multiplier: i128,
) {
    let magnitude_int = num.integer as i128;
    let magnitude_frac = num.decimals as i128;

    let int_contrib = magnitude_int.saturating_mul(unit_attos);

    let frac_contrib = if num.decimal_places > 0 && magnitude_frac != 0 {
        let numerator = magnitude_frac.saturating_mul(unit_attos);
        let divisor = 10i128.pow(num.decimal_places.min(18));
        numerator / divisor
    } else {
        0
    };

    let contribution = int_contrib.saturating_add(frac_contrib);

    let effective_multiplier = match num.sign {
        Sign::Positive => 1i128,
        Sign::Negative => -1i128,
        Sign::None => overall_multiplier,
    };

    *total_attos = total_attos.saturating_add(contribution.saturating_mul(effective_multiplier));
}

/// Parses time like "9:45", "14:30", "945", "1245" into (hour, minute)
fn parse_hms(s: &str) -> Option<(u8, u8, u8, u64)> {
    let mut values: Vec<u32> = Vec::new();
    let mut current: u32 = 0;
    let mut has_digit = false;
    let mut frac_str = String::new();
    let mut in_fraction = false;

    for ch in s.chars() {
        if let Some(digit_val) = to_ascii_digit(ch).and_then(|c| c.to_digit(10)) {
            has_digit = true;
            if in_fraction {
                frac_str.push(ch);
            } else {
                current = current * 10 + digit_val;
            }
        } else if ch == ':' {
            if in_fraction {
                return None;
            }
            values.push(current);
            current = 0;
        } else if ch == '.' {
            if in_fraction {
                return None;
            }
            values.push(current);
            current = 0;
            in_fraction = true;
        }
    }

    if has_digit && !in_fraction {
        values.push(current);
    }

    if values.is_empty() {
        return None;
    }

    let hour: u8;
    let minute: u8;
    let mut second: u8 = 0;

    match values.len() {
        1 => {
            let n = values[0];

            if n <= 23 {
                // Bare hour (casual): "9", "14", "23"
                hour = n as u8;
                minute = 0;
            } else {
                // Compact military time: "945", "1245", "2359", "0045", etc.
                hour = (n / 100) as u8;
                minute = (n % 100) as u8;
            }
        }
        2 => {
            hour = values[0] as u8;
            minute = values[1] as u8;
        }
        3 => {
            hour = values[0] as u8;
            minute = values[1] as u8;
            second = values[2] as u8;
        }
        _ => return None,
    }

    // Final validation (allow second = 60 for leap seconds)
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    // Fractional seconds → attoseconds
    let attos: u64 = if !frac_str.is_empty() {
        let frac: u64 = frac_str.parse().unwrap_or(0);
        let digits = frac_str.len() as u32;
        const MAX_DIGITS: u32 = 18;

        if digits > MAX_DIGITS {
            let truncated: u64 = frac_str[..MAX_DIGITS as usize].parse().unwrap_or(0);
            truncated
        } else {
            frac * 10u64.pow(MAX_DIGITS - digits)
        }
    } else {
        0
    };

    Some((hour, minute, second, attos))
}

impl Dt {
    /// expects input to be lowercase
    pub(crate) fn from_natural_relative_or_duration(
        s: &str,
        ref_date: Dt,
        lang: Lang,
        use_dur_finder: bool,
    ) -> Result<Dt, DtErr> {
        let Some(LangData {
            map: term_map,
            duration_ac,
            date_ac,
            decimal_char: _,
        }) = lang_map().get(&lang)
        else {
            return Err(an_err!(DtErrKind::InternalErr, "no langdata for: {}", lang));
        };
        let finder = if use_dur_finder { duration_ac } else { date_ac };
        let splitter = SplitKeepWithPos::new(finder, s);
        let mut is_relative = false;

        for (part, _) in splitter {
            if let Some((_, token)) = term_map.get(part) {
                if token.is_relative() {
                    is_relative = true;
                    break;
                }
            }
        }

        if is_relative {
            Dt::from_natural_relative_date(s, ref_date, lang, use_dur_finder)
        } else {
            // let result = Dt::from_natural_duration(s, lang, use_dur_finder)?;
            // Ok(ref_date.add(result))
            Dt::from_natural_relative_date(s, ref_date, lang, use_dur_finder)
        }
    }

    /// Expects s to be lowercase
    pub(crate) fn from_natural_relative_date(
        s: &str,
        ref_date: Dt,
        lang: Lang,
        use_dur_finder: bool,
    ) -> Result<Dt, DtErr> {
        let Some(LangData {
            map: term_map,
            duration_ac,
            date_ac,
            decimal_char: d,
        }) = lang_map().get(&lang)
        else {
            return Err(an_err!(DtErrKind::InternalErr, "no langdata for: {}", lang));
        };
        let finder = if use_dur_finder { duration_ac } else { date_ac };
        let names = Lang::En.names();

        let overall_negative = if let Some(rest) = s.strip_prefix('-') {
            rest.trim_start()
                .chars()
                .next()
                .is_some_and(|c| c.is_numeric() || c == '.' || c.is_whitespace())
        } else {
            false
        };

        let mut overall_multiplier: i128 = if overall_negative { -1 } else { 1 };
        let mut part_chars: Vec<char> = Vec::with_capacity(50);
        let mut pending_num: Option<ParsedNumber> = None;
        let mut pending_unit_attos: Option<i128> = None;
        let mut total_attos: i128 = 0;

        let mut target_weekday: Option<u8> = None;
        let mut target_month: Option<u8> = None;
        let mut month_offset: i32 = 0;
        let mut year_offset: i32 = 0;
        let mut week_offset: i32 = 0;
        let mut day_offset: i32 = 0;

        let mut target_hour: Option<u8> = None;
        let mut target_minute: Option<u8> = None;
        let mut target_second: Option<u8> = None;
        let mut target_attos: u64 = 0;
        let mut is_pm = false;
        let mut scale: Scale = Scale::UTC;
        let mut pending_bare_unit: Option<Token> = None;
        let mut direction: Direction = Direction::None;

        let mut splitter = SplitKeepWithPos::new(finder, s);

        while let Some((part, _)) = splitter.next() {
            if let Some((norm, token)) = term_map.get(part) {
                match token {
                    Token::Future => {
                        overall_multiplier = 1;
                        if let Some(unit) = pending_bare_unit.take() {
                            match unit {
                                Token::Week => week_offset = 1,
                                Token::Month => month_offset = 1,
                                Token::Year => year_offset = 1,
                                _ => {}
                            }
                        }
                        direction = Direction::Future;
                    }
                    Token::Past => {
                        overall_multiplier = -1;
                        if let Some(unit) = pending_bare_unit.take() {
                            match unit {
                                Token::Week => week_offset = -1,
                                Token::Month => month_offset = -1,
                                Token::Year => year_offset = -1,
                                _ => {}
                            }
                        }
                        direction = Direction::Past;
                    }
                    Token::Present => {
                        if let Some(unit) = pending_bare_unit.take() {
                            match unit {
                                Token::Week => week_offset = 0,
                                Token::Month => month_offset = 0,
                                Token::Year => year_offset = 0,
                                _ => {}
                            }
                        }
                        direction = Direction::Present;
                    }
                    Token::Tomorrow => day_offset = 1,
                    Token::TwoDaysLater => day_offset = 2,
                    Token::Yesterday => day_offset = -1,
                    Token::TwoDaysBefore => day_offset = -2,
                    Token::Ago => {
                        total_attos = -total_attos;
                        overall_multiplier = -1;
                        direction = Direction::Past;
                    }

                    Token::DayShort => {
                        if let Some(pos) = names
                            .weekdays_abbr
                            .iter()
                            .position(|&day| day.eq_ignore_ascii_case(norm.as_bytes()))
                        {
                            target_weekday = Some(pos as u8);
                        }
                    }
                    Token::DayLong => {
                        if let Some(pos) = names
                            .weekdays_full
                            .iter()
                            .position(|&day| day.eq_ignore_ascii_case(norm.as_bytes()))
                        {
                            target_weekday = Some(pos as u8);
                        }
                    }

                    Token::MonthShort => {
                        if let Some(pos) = names
                            .months_abbr
                            .iter()
                            .position(|&m| m.eq_ignore_ascii_case(norm.as_bytes()))
                        {
                            target_month = Some(pos as u8);
                        }
                    }
                    Token::MonthLong => {
                        if let Some(pos) = names
                            .months_full
                            .iter()
                            .position(|&m| m.eq_ignore_ascii_case(norm.as_bytes()))
                        {
                            target_month = Some(pos as u8);
                        }
                    }

                    Token::Year => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_YEAR, overall_multiplier);
                        } else {
                            if direction == Direction::None {
                                pending_bare_unit = Some(Token::Year);
                            } else {
                                match direction {
                                    Direction::Future => year_offset = 1,
                                    Direction::Past => year_offset = -1,
                                    Direction::Present => year_offset = 0,
                                    _ => {}
                                }
                            }
                        }
                    }
                    Token::Month => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_MONTH, overall_multiplier);
                        } else {
                            if direction == Direction::None {
                                pending_bare_unit = Some(Token::Month);
                            } else {
                                match direction {
                                    Direction::Future => month_offset = 1,
                                    Direction::Past => month_offset = -1,
                                    Direction::Present => month_offset = 0,
                                    _ => {}
                                }
                            }
                        }
                    }
                    Token::Week => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_WEEK, overall_multiplier);
                        } else {
                            if direction == Direction::None {
                                pending_bare_unit = Some(Token::Week);
                            } else {
                                match direction {
                                    Direction::Future => week_offset = 1,
                                    Direction::Past => week_offset = -1,
                                    Direction::Present => week_offset = 0,
                                    _ => {}
                                }
                            }
                        }
                    }

                    Token::Day => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_DAY, overall_multiplier);
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_DAY);
                        }
                    }
                    Token::Hour => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_HOUR, overall_multiplier);
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_HOUR);
                        }
                    }
                    Token::Minute => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_MINUTE, overall_multiplier);
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_MINUTE);
                        }
                    }
                    Token::Second => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Millisecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_MS_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_MS_I128);
                        }
                    }
                    Token::Microsecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_US_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_US_I128);
                        }
                    }
                    Token::Nanosecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_NS_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_NS_I128);
                        }
                    }
                    Token::Millennium => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1000 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1000 * AS_PER_YEAR);
                        }
                    }
                    Token::Century => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                100 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(100 * AS_PER_YEAR);
                        }
                    }
                    Token::Decade => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                10 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(10 * AS_PER_YEAR);
                        }
                    }
                    Token::Quarter => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                3 * AS_PER_MONTH,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(3 * AS_PER_MONTH);
                        }
                    }
                    Token::Fortnight => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                14 * AS_PER_DAY,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(14 * AS_PER_DAY);
                        }
                    }
                    Token::Kilosecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Megasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Gigasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Terasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Petasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Am => is_pm = false,
                    Token::Pm => is_pm = true,

                    Token::Scale => {
                        if let Some(scl) = Scale::from_abbrev(norm) {
                            scale = scl;
                        }
                    }

                    _ => {}
                }
            } else {
                // peek at next token to see if it's a duration unit
                let is_time = if let Some(next) = splitter.peek() {
                    term_map
                        .get(next.0)
                        .map(|(_, token)| !token.is_duration())
                        .unwrap_or(true)
                } else {
                    true
                };

                if is_time && let Some((h, m, sec, attos)) = parse_hms(part) {
                    target_hour = Some(h);
                    target_minute = Some(m);
                    target_second = Some(sec);
                    target_attos = attos;
                } else {
                    if let Some(num) = extract_number(part, &mut part_chars, *d) {
                        if let Some(unit_attos) = pending_unit_attos.take() {
                            add_to_total(&mut total_attos, num, unit_attos, overall_multiplier);
                        } else {
                            pending_num = Some(num);
                        }
                    }
                }
            }
        }

        // Apply leftover numeric duration
        if let Some(num) = pending_num.take() {
            add_to_total(&mut total_attos, num, AS_PER_DAY, overall_multiplier);
        }

        if let Some(unit) = pending_bare_unit.take() {
            match unit {
                Token::Week => week_offset = 0,
                Token::Month => month_offset = 0,
                Token::Year => year_offset = 0,
                _ => {}
            }
        }

        let mut result = ref_date.target(scale);

        if year_offset != 0 {
            result = result.add_yr(year_offset as i64);
        }
        if month_offset != 0 {
            result = result.add_mo(month_offset as i64);
        }
        if week_offset != 0 {
            result = result.add_wk(week_offset as i64);
        }
        if day_offset != 0 {
            result = result.add_days(day_offset as i64);
        }

        // Target month name
        if let Some(target_mo) = target_month {
            let current = result.to_ymd();
            let cur_mo = (current.mo() - 1) as i32;
            let raw_delta = (target_mo as i32 - cur_mo + 12) % 12;

            let month_delta = if month_offset != 0 || year_offset != 0 || week_offset != 0 {
                // Explicit unit shift already applied (e.g. from "mes pasado")
                // → just move to the target month in the shifted period
                if raw_delta == 0 { 0 } else { raw_delta }
            } else {
                match direction {
                    Direction::Present => {
                        if raw_delta <= 6 {
                            raw_delta
                        } else {
                            raw_delta - 12
                        }
                    }
                    Direction::Future => {
                        if raw_delta == 0 {
                            12
                        } else {
                            raw_delta
                        }
                    }
                    Direction::Past => {
                        if raw_delta == 0 {
                            -12
                        } else {
                            raw_delta - 12
                        }
                    }
                    Direction::None => raw_delta,
                }
            };

            if month_delta != 0 {
                result = result.add_mo(month_delta as i64);
            }
        }

        // Target weekday
        if let Some(target_wd) = target_weekday {
            let current_wd = result.to_ymd().wkday() as i32;
            let raw_delta = (target_wd as i32 - current_wd + 7) % 7;

            let day_delta = if week_offset != 0 || month_offset != 0 || year_offset != 0 {
                // Explicit unit shift already applied (bare unit like "week last" / "semana pasada")
                // → use Present-style logic so we pick the closest day in the shifted period
                if raw_delta == 0 {
                    0
                } else if raw_delta <= 3 {
                    raw_delta
                } else {
                    raw_delta - 7
                }
            } else {
                // No explicit unit shift → original nearest-occurrence logic
                match direction {
                    Direction::Present => {
                        if raw_delta <= 3 {
                            raw_delta
                        } else {
                            raw_delta - 7
                        }
                    }
                    Direction::Future => {
                        if raw_delta == 0 {
                            7
                        } else {
                            raw_delta
                        }
                    }
                    Direction::Past => {
                        if raw_delta == 0 {
                            -7
                        } else {
                            raw_delta - 7
                        }
                    }
                    Direction::None => raw_delta,
                }
            };

            if day_delta != 0 {
                result = result.add(Dt::span(day_delta as i128 * AS_PER_DAY));
            }
        }

        if total_attos != 0 {
            result = result.add(Dt::span(total_attos));
        }

        // Time of day
        if target_hour.is_some()
            || is_pm
            || target_minute.is_some()
            || target_second.is_some()
            || target_attos != 0
        {
            let mut h = target_hour.unwrap_or(0);
            if is_pm && h < 12 {
                h += 12;
            }
            if !is_pm && h == 12 {
                h = 0;
            }

            let ymd = result.to_ymd();
            result = Dt::from_ymd(
                ymd.yr(),
                ymd.mo(),
                ymd.day(),
                ymd.scale(),
                h,
                target_minute.unwrap_or(0),
                target_second.unwrap_or(0),
                target_attos,
            );
        }

        Ok(result)
    }

    /// Expects s to be lowercase
    pub(crate) fn from_natural_duration(
        s: &str,
        lang: Lang,
        use_dur_finder: bool,
    ) -> Result<Dt, DtErr> {
        let Some(LangData {
            map: term_map,
            duration_ac,
            date_ac,
            decimal_char: d,
        }) = lang_map().get(&lang)
        else {
            return Err(an_err!(DtErrKind::InternalErr, "no langdata for: {}", lang));
        };
        let finder = if use_dur_finder { duration_ac } else { date_ac };

        let overall_negative = if let Some(rest) = s.strip_prefix('-') {
            rest.trim_start()
                .chars()
                .next()
                .is_some_and(|c| c.is_numeric() || c == '.' || c.is_whitespace())
        } else {
            false
        };

        let mut overall_multiplier: i128 = if overall_negative { -1 } else { 1 };
        let mut part_chars: Vec<char> = Vec::with_capacity(50);
        let mut has_duration = false;
        let mut pending_num: Option<ParsedNumber> = None;
        let mut pending_unit_attos: Option<i128> = None;
        let mut total_attos: i128 = 0;

        let splitter = SplitKeepWithPos::new(finder, s);

        for (part, _) in splitter {
            if let Some((_, token)) = term_map.get(part) {
                match token {
                    Token::Future => overall_multiplier = 1,
                    Token::Past => overall_multiplier = -1,
                    Token::Now | Token::Today => {
                        if !has_duration {
                            return Ok(Dt::ZERO);
                        }
                    }
                    Token::Tomorrow => {
                        if !has_duration {
                            return Ok(Dt::span(AS_PER_DAY));
                        }
                    }
                    Token::Yesterday => {
                        if !has_duration {
                            return Ok(Dt::span(-AS_PER_DAY));
                        }
                    }
                    Token::Ago => {
                        total_attos = -total_attos;
                    }
                    Token::Year => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_YEAR, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_YEAR);
                        }
                    }
                    Token::Month => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_MONTH, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_MONTH);
                        }
                    }
                    Token::Week => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_WEEK, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_WEEK);
                        }
                    }
                    Token::Day => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_DAY, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_DAY);
                        }
                    }
                    Token::Hour => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_HOUR, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_HOUR);
                        }
                    }
                    Token::Minute => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(&mut total_attos, num, AS_PER_MINUTE, overall_multiplier);
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(AS_PER_MINUTE);
                        }
                    }
                    Token::Second => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Millisecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_MS_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_MS_I128);
                        }
                    }
                    Token::Microsecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_US_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_US_I128);
                        }
                    }
                    Token::Nanosecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                ATTOS_PER_NS_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(ATTOS_PER_NS_I128);
                        }
                    }
                    Token::Millennium => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1000 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1000 * AS_PER_YEAR);
                        }
                    }
                    Token::Century => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                100 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(100 * AS_PER_YEAR);
                        }
                    }
                    Token::Decade => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                10 * AS_PER_YEAR,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(10 * AS_PER_YEAR);
                        }
                    }
                    Token::Quarter => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                3 * AS_PER_MONTH,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(3 * AS_PER_MONTH);
                        }
                    }
                    Token::Fortnight => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                14 * AS_PER_DAY,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(14 * AS_PER_DAY);
                        }
                    }
                    // SI large units
                    Token::Kilosecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Megasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Gigasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Terasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }
                    Token::Petasecond => {
                        if let Some(num) = pending_num.take() {
                            add_to_total(
                                &mut total_attos,
                                num,
                                1_000_000_000_000_000 * ATTOS_PER_SEC_I128,
                                overall_multiplier,
                            );
                            has_duration = true;
                        } else if pending_unit_attos.is_none() {
                            pending_unit_attos = Some(1_000_000_000_000_000 * ATTOS_PER_SEC_I128);
                        }
                    }

                    _ => {}
                }
            } else if let Some(num) = extract_number(part, &mut part_chars, *d) {
                if let Some(unit_attos) = pending_unit_attos.take() {
                    // unit came first → apply it to this number
                    add_to_total(&mut total_attos, num, unit_attos, overall_multiplier);
                    has_duration = true;
                } else {
                    // normal number-first case
                    pending_num = Some(num);
                }
            }
        }

        if !has_duration {
            return Err(an_err!(DtErrKind::InvalidInput, "{}", s));
        }

        Ok(Dt::span(total_attos))
    }
}

// Expects s to be lowercase
pub(crate) fn natural_duration_to_iso(
    s: &str,
    lang: Lang,
    use_dur_finder: bool,
) -> Result<String, DtErr> {
    let span = Dt::from_natural_duration(s, lang, use_dur_finder)?;
    Ok(span.to_string())
}
