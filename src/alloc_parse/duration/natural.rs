use crate::{
    AS_PER_DAY, AS_PER_HOUR, AS_PER_MINUTE, AS_PER_MONTH, AS_PER_WEEK, AS_PER_YEAR,
    ATTOS_PER_MS_I128, ATTOS_PER_NS_I128, ATTOS_PER_SEC_I128, ATTOS_PER_US_I128, Dt, DtErr,
    DtErrKind, Lang, LangData, SplitKeepWithPos, Token, an_err, lang_map, to_ascii_digit,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Sign {
    None,
    Positive,
    Negative,
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

pub(crate) fn natural_duration_to_span(
    input: &str,
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

    let lower = input.to_lowercase();
    let overall_negative = if let Some(rest) = lower.strip_prefix('-') {
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

    let splitter = SplitKeepWithPos::new(finder, lower.as_str());

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
                        add_to_total(&mut total_attos, num, ATTOS_PER_MS_I128, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit_attos.is_none() {
                        pending_unit_attos = Some(ATTOS_PER_MS_I128);
                    }
                }
                Token::Microsecond => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_attos, num, ATTOS_PER_US_I128, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit_attos.is_none() {
                        pending_unit_attos = Some(ATTOS_PER_US_I128);
                    }
                }
                Token::Nanosecond => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_attos, num, ATTOS_PER_NS_I128, overall_multiplier);
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
                        add_to_total(&mut total_attos, num, 100 * AS_PER_YEAR, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit_attos.is_none() {
                        pending_unit_attos = Some(100 * AS_PER_YEAR);
                    }
                }
                Token::Decade => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_attos, num, 10 * AS_PER_YEAR, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit_attos.is_none() {
                        pending_unit_attos = Some(10 * AS_PER_YEAR);
                    }
                }
                Token::Quarter => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_attos, num, 3 * AS_PER_MONTH, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit_attos.is_none() {
                        pending_unit_attos = Some(3 * AS_PER_MONTH);
                    }
                }
                Token::Fortnight => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_attos, num, 14 * AS_PER_DAY, overall_multiplier);
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
        return Err(an_err!(DtErrKind::InvalidInput, "{}", input));
    }

    Ok(Dt::span(total_attos))
}

pub(crate) fn natural_duration_to_iso(
    input: &str,
    lang: Lang,
    use_dur_finder: bool,
) -> Result<String, DtErr> {
    let span = natural_duration_to_span(input, lang, use_dur_finder)?;
    Ok(span.to_string())
}
