use crate::{
    ClassifiedDate, ConnectorType, DateClassification, Dt, DtErr, DtErrKind, EndsWithExt, IndexIn,
    Lang, LangData, SplitKeepWithPos, Token, an_err, lang_map, to_ascii_digit,
};
use alloc::string::String;
use alloc::vec::Vec;

fn send_to_relative_parser(
    s: &str,
    lang: Lang,
    ref_time: &Option<Dt>,
) -> Result<ClassifiedDate, DtErr> {
    let now: Dt = if let Some(tp) = ref_time {
        *tp
    } else {
        #[cfg(feature = "std")]
        {
            Dt::now()
        }
        #[cfg(not(feature = "std"))]
        {
            return Err(an_err!(DtErrKind::MissingRefTimeOrStd));
        }
    };

    let dt = Dt::from_natural_relative_or_duration(s, now, lang, false)?;

    Ok(ClassifiedDate::Parsed(dt))
}

/// Expects s to be lowercase.
pub(crate) fn classify_date(
    s: &str,
    lang: Lang,
    ref_time: &Option<Dt>,
) -> Result<ClassifiedDate, DtErr> {
    let Some(LangData {
        map: term_map,
        date_ac: finder,
        ..
    }) = lang_map().get(&lang)
    else {
        return Err(an_err!(DtErrKind::InternalErr, "{}", lang));
    };

    let bytes_len = s.len();

    let mut has_ampm = false;
    let mut has_fractional = false;
    let mut has_w = false;
    let mut is_pure_numeric = true;
    let mut num_named = 0u8;
    let mut num_colon = 0u8;
    let mut num_comma = 0u8;
    let mut num_hyphen = 0u8;
    let mut num_slash = 0u8;
    let mut num_dot = 0u8;
    let mut num_digits = 0u8;
    let mut num_non_decimal_digits = 0u8;
    let mut num_date_digit_groups = 0u8;
    let mut num_date_digits = 0u8;
    let mut year_maybe_on_end = false;
    let mut connector = ConnectorType::None;
    let mut date_tokens: Vec<Token> = Vec::with_capacity(10);
    let mut time_tokens: Vec<Token> = Vec::with_capacity(10);
    let mut in_digit_run = false;
    let mut digit_run_len = 0usize;
    let mut in_time_digit_run = false;
    let mut time_digit_run_len = 0usize;
    let mut time_digits = 0u8;
    let mut time_colons = 0u8;
    let mut currently = IndexIn::PreDate;
    let mut offset_colons = 0u8;
    let mut has_time = false;
    let mut part_chars: Vec<char> = Vec::with_capacity(24);
    let mut following_digits: usize;
    let mut date_norm = String::with_capacity(bytes_len);

    let splitter = SplitKeepWithPos::new(finder, s);

    for (part, _) in splitter {
        if let Some((norm_part, token)) = term_map.get(part) {
            if (token.is_relative() || token.is_duration()) && !currently.after_date() {
                return send_to_relative_parser(s, lang, ref_time);
            }

            match token {
                Token::DayShort | Token::DayLong | Token::MonthShort | Token::MonthLong => {
                    if currently == IndexIn::PreDate {
                        currently = IndexIn::Date;
                    }
                    num_named += 1;
                    date_tokens.push(*token);
                    date_norm.push_str(norm_part);
                }
                Token::Am | Token::Pm => {
                    if matches!(currently, IndexIn::Time | IndexIn::PostDate) {
                        has_ampm = true;
                        currently = IndexIn::PostDate;
                        date_norm.push_str(norm_part);
                        time_tokens.push(*token);
                    }
                }
                Token::Iana => {
                    if currently != IndexIn::PreDate {
                        date_norm.push_str(norm_part);
                        time_tokens.push(Token::Iana);
                    }
                }
                Token::W => {
                    if currently != IndexIn::PreDate {
                        has_w = true;
                        date_tokens.push(Token::W);
                        date_norm.push_str(norm_part);
                    }
                }
                Token::Scale => {
                    if currently.after_date() {
                        time_tokens.push(Token::Scale);
                        date_norm.push_str(norm_part);
                    }
                }
                _ => {}
            }
            is_pure_numeric = false;
        } else {
            part_chars.clear();
            part_chars.extend(part.chars());
            let part_len = part_chars.len();

            for (idx, ch) in part_chars.iter().enumerate() {
                if ch.is_numeric() {
                    if currently == IndexIn::PreDate {
                        currently = IndexIn::Date;
                    }
                    if let Some(d) = to_ascii_digit(*ch) {
                        date_norm.push(d);
                    } else {
                        continue;
                    }

                    num_digits += 1;
                    if currently != IndexIn::Fraction {
                        num_non_decimal_digits += 1;
                    }

                    match currently {
                        IndexIn::Date => {
                            num_date_digits += 1;
                            if !in_digit_run {
                                in_digit_run = true;
                                num_date_digit_groups += 1;
                            }
                            digit_run_len += 1;

                            if match num_date_digits {
                                8.. => idx + 1 < part_len && part_chars[idx + 1].is_numeric(),
                                6 => {
                                    is_pure_numeric
                                        && match bytes_len {
                                            10 | 12 => {
                                                idx + 1 < part_len
                                                    && part_chars[idx + 1].is_numeric()
                                                    && num_date_digit_groups == 1
                                            }
                                            11 => {
                                                idx + 3 < part_len
                                                    && part_chars[idx + 1].is_numeric()
                                                    && part_chars[idx + 2].is_numeric()
                                                    && part_chars[idx + 3] == ':'
                                            }
                                            _ => false,
                                        }
                                }
                                _ => false,
                            } {
                                if digit_run_len > 0 {
                                    date_tokens.push(Token::Digits(digit_run_len));
                                    in_digit_run = false;
                                    digit_run_len = 0;
                                }
                                currently = IndexIn::Time;
                            }
                        }
                        IndexIn::Time | IndexIn::Fraction | IndexIn::Offset => {
                            if currently == IndexIn::Time {
                                time_digits += 1;
                            }
                            if !in_time_digit_run {
                                in_time_digit_run = true;
                            }
                            time_digit_run_len += 1;
                        }
                        IndexIn::PostDate => {
                            year_maybe_on_end = true;
                            if !in_time_digit_run {
                                in_time_digit_run = true;
                            }
                            time_digit_run_len += 1;
                        }
                        _ => {}
                    }
                } else if currently != IndexIn::PreDate {
                    if in_digit_run && digit_run_len > 0 {
                        date_tokens.push(Token::Digits(digit_run_len));
                        in_digit_run = false;
                        digit_run_len = 0;
                    }

                    if in_time_digit_run && time_digit_run_len > 0 {
                        if *ch != ':' {
                            if currently == IndexIn::Offset {
                                let tok = if offset_colons > 0 {
                                    Token::OffsetColon
                                } else {
                                    Token::Offset
                                };
                                time_tokens.push(tok);
                            } else if currently == IndexIn::Time && time_digits >= 3 {
                                let tok = if time_digits > 4 {
                                    if time_colons > 0 {
                                        Token::HmsColon
                                    } else {
                                        Token::Hms
                                    }
                                } else if time_colons > 0 {
                                    Token::HmColon
                                } else {
                                    Token::Hm
                                };
                                has_time = true;
                                time_tokens.push(tok);
                            }
                        }
                        in_time_digit_run = false;
                        time_digit_run_len = 0;
                    }

                    if is_pure_numeric && ch != &'.' {
                        is_pure_numeric = false;
                    }

                    match ch {
                        ' ' => {
                            if !date_norm.ends_with_space() {
                                date_norm.push(' ');
                            }
                            match currently {
                                IndexIn::Date => {
                                    if num_date_digits >= 8 {
                                        currently = IndexIn::Time;
                                        connector = ConnectorType::Space;
                                    } else if idx + 1 < part_len && part_chars[idx + 1].is_numeric()
                                    {
                                        let start = idx + 1;
                                        following_digits = 1;
                                        for c in &part_chars[(start + 1)..part_len.min(start + 6)] {
                                            if !c.is_numeric() {
                                                break;
                                            }
                                            following_digits += 1;
                                        }
                                        let j = start + following_digits;
                                        if (following_digits == 1 || following_digits == 2)
                                            && j < part_len
                                            && matches!(part_chars[j], ':' | '+' | '[' | 'h')
                                        {
                                            currently = IndexIn::Time;
                                            connector = ConnectorType::Space;
                                        } else if following_digits >= 4 {
                                            match num_date_digit_groups {
                                                3.. => {
                                                    currently = IndexIn::Time;
                                                    connector = ConnectorType::Space;
                                                }
                                                2.. => {
                                                    if num_named >= 1
                                                        && num_comma
                                                            + num_dot
                                                            + num_hyphen
                                                            + num_slash
                                                            == 0
                                                    {
                                                        currently = IndexIn::Time;
                                                        connector = ConnectorType::Space;
                                                    }
                                                }
                                                1.. => {
                                                    if num_date_digits >= 6 && in_digit_run {
                                                        currently = IndexIn::Time;
                                                        connector = ConnectorType::Space;
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    if !time_tokens.ends_with(&[Token::Space]) {
                                        time_tokens.push(Token::Space);
                                    }
                                    currently = IndexIn::PostDate;
                                }
                            }
                            if currently == IndexIn::Date && !date_tokens.ends_with(&[Token::Space])
                            {
                                date_tokens.push(Token::Space);
                            }
                        }
                        ':' => {
                            if date_norm.ends_with_colon() {
                                continue;
                            }
                            date_norm.push(':');
                            num_colon += 1;

                            match currently {
                                IndexIn::Date => {
                                    connector = ConnectorType::Colon;
                                    currently = IndexIn::Time;
                                }
                                IndexIn::Time => time_colons += 1,
                                IndexIn::Offset => offset_colons += 1,
                                _ => {}
                            }
                        }
                        '-' => {
                            if date_norm.ends_with_minus() {
                                continue;
                            }
                            date_norm.push('-');
                            if currently.after_date() && currently != IndexIn::Offset {
                                currently = IndexIn::Offset;
                                offset_colons = 0;
                            }
                            if currently == IndexIn::Date {
                                num_hyphen += 1;
                                date_tokens.push(Token::Hyphen);
                            } else if currently.after_date() {
                                time_tokens.push(Token::Minus);
                            }
                        }
                        '.' => {
                            if date_norm.ends_with_dot() {
                                continue;
                            }
                            date_norm.push('.');
                            num_dot += 1;
                            let is_fractional_trigger = !has_fractional
                                && (currently.after_date() || {
                                    if num_dot > 1 {
                                        is_pure_numeric && num_non_decimal_digits >= 10
                                    } else {
                                        // mjd
                                        is_pure_numeric && num_non_decimal_digits >= 5
                                    }
                                });
                            if is_fractional_trigger {
                                has_fractional = true;
                                currently = IndexIn::Fraction;
                                time_tokens.push(Token::Dot);
                            } else if currently == IndexIn::Date {
                                date_tokens.push(Token::Dot);
                            }
                        }
                        '/' => {
                            if date_norm.ends_with_slash() {
                                continue;
                            }
                            date_norm.push('/');
                            num_slash += 1;
                            if currently == IndexIn::Date {
                                date_tokens.push(Token::Slash);
                            }
                        }
                        ',' => {
                            if date_norm.ends_with_comma() {
                                continue;
                            }
                            date_norm.push(',');
                            num_comma += 1;
                            if currently == IndexIn::Date {
                                date_tokens.push(Token::Comma);
                            }
                        }
                        't' => {
                            if date_norm.ends_with_ascii_digit()
                                && idx + 1 < part_len
                                && part_chars[idx + 1].is_numeric()
                            {
                                connector = ConnectorType::UpperT;
                                currently = IndexIn::Time;
                                date_norm.push('T');
                            }
                        }
                        'z' => {
                            if date_norm.ends_with_ascii_digit() {
                                currently = IndexIn::PostDate;
                                date_norm.push('Z');
                                time_tokens.push(Token::Zulu);
                            }
                        }
                        'w' => {
                            if currently == IndexIn::Date
                                && !has_w
                                && idx + 1 < part_len
                                && part_chars[idx + 1].is_numeric()
                            {
                                has_w = true;
                                date_tokens.push(Token::W);
                                date_norm.push('W');
                            }
                        }
                        '+' => {
                            if date_norm.ends_with_plus() {
                                continue;
                            }
                            date_norm.push('+');
                            if currently.after_date() && currently != IndexIn::Offset {
                                currently = IndexIn::Offset;
                                offset_colons = 0;
                            } else if currently != IndexIn::PreDate {
                                time_tokens.push(Token::Plus);
                            }
                        }
                        '[' => {
                            if date_norm.ends_with_lbracket() {
                                continue;
                            }
                            date_norm.push('[');
                            currently = IndexIn::PostDate;
                            time_tokens.push(Token::LBracket);
                        }
                        ']' => {
                            if date_norm.ends_with_rbracket() {
                                continue;
                            }
                            date_norm.push(']');
                            currently = IndexIn::PostDate;
                            time_tokens.push(Token::RBracket);
                        }
                        _ => {
                            if date_norm.ends_with_ascii_digit()
                                && idx + 1 < part_len
                                && part_chars[idx + 1].is_numeric()
                            {
                                if currently == IndexIn::Date {
                                    date_norm.push('-');
                                    num_hyphen += 1;
                                    date_tokens.push(Token::Hyphen);
                                } else if currently == IndexIn::Time {
                                    date_norm.push(':');
                                    num_colon += 1;
                                    time_colons += 1;
                                }
                            }
                        }
                    }
                } else {
                    if ch == &'-' {
                        if date_norm.ends_with_minus() {
                            continue;
                        }
                        if idx + 1 < part_len && part_chars[idx + 1].is_numeric() {
                            date_norm.push('-');
                        }
                    }
                }
            }

            // remaining digit runs (date)
            if digit_run_len > 0 && matches!(currently, IndexIn::Date | IndexIn::PostDate) {
                date_tokens.push(Token::Digits(digit_run_len));
                in_digit_run = false;
                digit_run_len = 0;
            }

            // remaining time/offset run
            if in_time_digit_run && time_digit_run_len > 0 && currently.after_date() {
                if currently == IndexIn::Offset {
                    let tok = if offset_colons > 0 {
                        Token::OffsetColon
                    } else {
                        Token::Offset
                    };
                    time_tokens.push(tok);
                } else if currently == IndexIn::Time && time_digits >= 3 {
                    let tok = if time_digits > 4 {
                        if time_colons > 0 {
                            Token::HmsColon
                        } else {
                            Token::Hms
                        }
                    } else if time_colons > 0 {
                        Token::HmColon
                    } else {
                        Token::Hm
                    };
                    has_time = true;
                    time_tokens.push(tok);
                }
                in_time_digit_run = false;
                time_digit_run_len = 0;
            }
        }
    }

    if num_digits == 0 {
        return Err(an_err!(DtErrKind::ExpectedDigits));
    }

    // final remaining (end of string)
    if digit_run_len > 0 && matches!(currently, IndexIn::Date | IndexIn::PostDate) {
        date_tokens.push(Token::Digits(digit_run_len));
    }

    // trim trailing space if connector is space
    if date_tokens.ends_with(&[Token::Space]) && connector == ConnectorType::Space {
        date_tokens.pop();
    }

    if tokens_look_like_relative(&date_tokens, has_ampm, num_colon, has_time) {
        return send_to_relative_parser(s, lang, ref_time);
    }

    if in_time_digit_run && time_digit_run_len > 0 && currently.after_date() {
        if currently == IndexIn::Offset {
            let tok = if offset_colons > 0 {
                Token::OffsetColon
            } else {
                Token::Offset
            };
            time_tokens.push(tok);
        } else if currently == IndexIn::Time && time_digits >= 3 {
            let tok = if time_digits > 4 {
                if time_colons > 0 {
                    Token::HmsColon
                } else {
                    Token::Hms
                }
            } else if time_colons > 0 {
                Token::HmColon
            } else {
                Token::Hm
            };
            has_time = true;
            time_tokens.push(tok);
        }
    }

    if num_dot > 1 {
        is_pure_numeric = false;
    }

    let has_year = matches!(date_tokens.first(), Some(Token::Digits(n)) if *n >= 4)
        || num_date_digit_groups >= 3
        || (num_named >= 1 && num_date_digit_groups >= 2)
        || (num_comma > 0 && num_date_digit_groups >= 2)
        || (num_digits >= 6 && num_named == 0)
        || (num_named >= 1 && num_date_digit_groups == 1 && num_date_digits >= 4)
        || (num_named >= 1 && has_time && num_digits >= 10);

    Ok(ClassifiedDate::Cls(DateClassification {
        bytes_len,
        date: date_norm,
        date_tokens,
        time_tokens,
        is_pure_numeric,
        is_decimal: num_dot == 1 && is_pure_numeric,
        has_year,
        has_time,
        num_named,
        connector,
        has_ampm,
        has_fractional,
        has_w,
        num_colon,
        num_hyphen,
        num_dot,
        num_digits,
        num_date_digits,
        num_non_decimal_digits,
        num_date_digit_groups,
        year_maybe_on_end,
    }))
}

#[inline(always)]
fn tokens_look_like_relative(
    date_tokens: &[Token],
    has_ampm: bool,
    num_colon: u8,
    has_time: bool,
) -> bool {
    let mut has_digit_block = false;
    let mut has_named = false;

    for token in date_tokens {
        match token {
            Token::MonthShort | Token::MonthLong => return false,
            Token::Digits(n) => {
                if has_digit_block {
                    return false;
                } else if *n <= 4 {
                    has_digit_block = true;
                } else {
                    return false;
                }
            }
            Token::DayShort | Token::DayLong => {
                if has_named {
                    return false;
                } else {
                    has_named = true;
                }
            }
            Token::W => return false,
            _ => {}
        }
    }

    if has_digit_block && !has_named && !has_ampm {
        if num_colon == 0 {
            return false;
        }
        return !has_time;
    }
    true
}
