use crate::{
    DateToken, Dt, DtErr, DtErrKind, Lang, LangData, NS_PER_DAY, NS_PER_HOUR, NS_PER_MINUTE,
    NS_PER_MONTH, NS_PER_SEC, NS_PER_WEEK, NS_PER_YEAR, Scale, SplitKeepWithPos, an_err, lang_map,
    to_ascii_digit,
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

fn extract_number(part: &str, part_chars: &mut Vec<char>) -> Option<ParsedNumber> {
    part_chars.clear();
    part_chars.extend(part.chars());

    let mut integer: i64 = 0;
    let mut decimals: i64 = 0;
    let mut decimal_places: u32 = 0;
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut sign = Sign::None;

    // Scientific notation support (hard-coded 'e')
    let mut seen_e = false;
    let mut exponent: i32 = 0;
    let mut exp_sign = Sign::None;
    let mut exp_seen_digit = false;

    for &ch in part_chars.iter() {
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
            '.' => {
                if seen_e || seen_dot {
                    continue;
                }
                seen_dot = true;
                continue;
            }
            _ => {
                if !ch.is_numeric() {
                    continue;
                }
                let Some(d) = to_ascii_digit(ch).and_then(|c| c.to_digit(10)) else {
                    continue;
                };
                let digit = d as i64;

                if seen_e {
                    exponent = exponent.saturating_mul(10).saturating_add(digit as i32);
                    exp_seen_digit = true;
                } else if seen_dot {
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
    total_nanos: &mut i128,
    num: ParsedNumber,
    unit_nanos: i128,
    overall_multiplier: i128,
) {
    let magnitude_int = num.integer as i128;
    let magnitude_frac = num.decimals as i128;

    let int_contrib = magnitude_int.saturating_mul(unit_nanos);

    let frac_contrib = if num.decimal_places > 0 && magnitude_frac != 0 {
        let numerator = magnitude_frac.saturating_mul(unit_nanos);
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

    *total_nanos = total_nanos.saturating_add(contribution.saturating_mul(effective_multiplier));
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
        ..
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
    let mut pending_unit: Option<i128> = None;
    let mut total_nanos: i128 = 0;
    let splitter = SplitKeepWithPos::new(finder, lower.as_str());

    for (part, _) in splitter {
        if let Some((_, token)) = term_map.get(part) {
            match token {
                DateToken::Future => overall_multiplier = 1,
                DateToken::Past => overall_multiplier = -1,
                DateToken::Now | DateToken::Today => {
                    if !has_duration {
                        return Ok(Dt::ZERO);
                    }
                }
                DateToken::Tomorrow => {
                    if !has_duration {
                        return Ok(Dt::from_ns(NS_PER_DAY, Scale::TAI));
                    }
                }
                DateToken::Yesterday => {
                    if !has_duration {
                        return Ok(Dt::from_ns(-NS_PER_DAY, Scale::TAI));
                    }
                }

                DateToken::Year => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_YEAR, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_YEAR);
                    }
                }
                DateToken::Month => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_MONTH, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_MONTH);
                    }
                }
                DateToken::Week => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_WEEK, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_WEEK);
                    }
                }
                DateToken::Day => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_DAY, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_DAY);
                    }
                }
                DateToken::Hour => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_HOUR, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_HOUR);
                    }
                }
                DateToken::Minute => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_MINUTE, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_MINUTE);
                    }
                }
                DateToken::Second => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, NS_PER_SEC, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(NS_PER_SEC);
                    }
                }
                DateToken::Millisecond => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, 1_000_000, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(1_000_000);
                    }
                }
                DateToken::Microsecond => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, 1_000, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(1_000);
                    }
                }
                DateToken::Nanosecond => {
                    if let Some(num) = pending_num.take() {
                        add_to_total(&mut total_nanos, num, 1, overall_multiplier);
                        has_duration = true;
                    } else if pending_unit.is_none() {
                        pending_unit = Some(1);
                    }
                }

                _ => {}
            }
        } else if let Some(num) = extract_number(part, &mut part_chars) {
            if let Some(unit_nanos) = pending_unit.take() {
                // unit came first → apply it to this number
                add_to_total(&mut total_nanos, num, unit_nanos, overall_multiplier);
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

    // Convert total nanoseconds → attoseconds and build Dt
    // (Dt supports the full representable range, so no size checks are needed)
    let total_attos = total_nanos * 1_000_000_000i128;
    Ok(Dt::from_attos(total_attos, Scale::TAI))
}

pub(crate) fn natural_duration_to_iso(
    input: &str,
    lang: Lang,
    use_dur_finder: bool,
) -> Result<String, DtErr> {
    let span = natural_duration_to_span(input, lang, use_dur_finder)?;
    Ok(span.to_string())
}
