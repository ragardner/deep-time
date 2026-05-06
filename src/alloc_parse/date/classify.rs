use crate::{
    ClassifiedDate, ConnectorType, DateClassification, DateToken, DtErr, DtErrKind, EndsWithExt,
    IndexIn, Lang, LangData, OffsetType, SplitKeepWithPos, TimePoint, TimeType, an_err, lang_map,
    natural_duration_to_span, to_ascii_digit,
};
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use crate::ClockType;

/// Expects s to be lowercase.
pub(crate) fn classify_date(
    s: &str,
    lang: Lang,
    ref_time: &Option<TimePoint>,
) -> Result<ClassifiedDate, DtErr> {
    let Some(LangData {
        map: term_map,
        date_ac: finder,
        ..
    }) = lang_map().get(&lang)
    else {
        return Err(an_err!(DtErrKind::InternalErr, "no langdata for: {}", lang));
    };

    let (s, attach_hyphen) = s.strip_prefix('-').map_or((s, false), |s| (s, true));
    let ascii_len = s.len();

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
    let mut space_before_bracket = false;
    let mut space_before_offset = false;
    let mut year_maybe_on_end = false;
    let mut connector = ConnectorType::None;
    let mut tokens: Vec<DateToken> = Vec::with_capacity(10);

    let mut in_digit_run_date = false;
    let mut curr_date_digit_run_len = 0usize;
    let mut time_digits = 0u8;
    let mut time_colons = 0u8;
    let mut seen_dot = false;
    let mut iana_offset = OffsetType::None;
    let mut currently = IndexIn::PreDate;
    let mut offset_digits = 0u8;
    let mut offset_colons = 0u8;
    let mut bracket_digits = 0u8;
    let mut bracket_colons = 0u8;
    let mut bracket_has_sign = false;
    let mut part_chars: Vec<char> = Vec::with_capacity(24);
    let mut following_digits: usize;

    let mut date_norm = String::with_capacity(ascii_len);
    if attach_hyphen {
        date_norm.push('-');
    }
    let mut splitter = SplitKeepWithPos::new(finder, &s);

    while let Some((part, _)) = splitter.next() {
        if let Some((norm_part, token)) = term_map.get(part) {
            if token.is_relative() {
                // ── Use the reference time (or fall back to real system time) ──
                let now: TimePoint = if let Some(tp) = ref_time {
                    *tp
                } else {
                    #[cfg(feature = "std")]
                    {
                        TimePoint::now(ClockType::UTC)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        return Err(an_err!(
                            DtErrKind::InternalErr,
                            "relative dates need ref time/std"
                        ));
                    }
                };
                now.add(natural_duration_to_span(s, lang, false)?);
                return Ok(ClassifiedDate::Parsed(now));
            }
            match token {
                DateToken::DayShort
                | DateToken::DayLong
                | DateToken::MonthShort
                | DateToken::MonthLong => {
                    if currently == IndexIn::PreDate {
                        currently = IndexIn::Date;
                    }
                    num_named += 1;
                    tokens.push(*token);
                    date_norm.push_str(*norm_part);
                }
                DateToken::Am | DateToken::Pm => {
                    if currently != IndexIn::PreDate {
                        has_ampm = true;
                        currently = IndexIn::PostDate;
                        date_norm.push_str(*norm_part);
                    }
                }
                DateToken::Iana => {
                    if currently != IndexIn::PreDate {
                        if currently == IndexIn::Bracket {
                            iana_offset = OffsetType::InBracketIana;
                        } else {
                            iana_offset = OffsetType::Iana;
                            if date_norm.ends_with_space() {
                                space_before_offset = true;
                            }
                        }
                        date_norm.push_str(*norm_part);
                    }
                }
                DateToken::W => {
                    if currently != IndexIn::PreDate {
                        has_w = true;
                        tokens.push(DateToken::W);
                        date_norm.push_str(*norm_part);
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
                    }
                    num_digits += 1;
                    if currently != IndexIn::Fraction {
                        num_non_decimal_digits += 1;
                    }
                    match currently {
                        IndexIn::Date => {
                            num_date_digits += 1;
                            if !in_digit_run_date {
                                in_digit_run_date = true;
                                num_date_digit_groups += 1;
                            }
                            curr_date_digit_run_len += 1;
                            if match num_date_digits {
                                8.. => idx + 1 < part_len && part_chars[idx + 1].is_numeric(),
                                6 => {
                                    is_pure_numeric
                                        && match ascii_len {
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
                                if curr_date_digit_run_len > 0 {
                                    tokens.push(DateToken::Digits(curr_date_digit_run_len));
                                    in_digit_run_date = false;
                                    curr_date_digit_run_len = 0;
                                }
                                currently = IndexIn::Time;
                            }
                        }
                        IndexIn::Time => time_digits += 1,
                        IndexIn::Offset => offset_digits += 1,
                        IndexIn::Bracket => bracket_digits += 1,
                        IndexIn::PostDate => year_maybe_on_end = true,
                        _ => {}
                    }
                } else if currently != IndexIn::PreDate {
                    if in_digit_run_date && curr_date_digit_run_len > 0 {
                        tokens.push(DateToken::Digits(curr_date_digit_run_len));
                    }
                    in_digit_run_date = false;
                    curr_date_digit_run_len = 0;
                    if is_pure_numeric && ch != &'.' {
                        is_pure_numeric = false;
                    }
                    match ch {
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
                                iana_offset = OffsetType::Zulu;
                                currently = IndexIn::PostDate;
                                date_norm.push('Z');
                            }
                        }
                        'w' => {
                            if !has_w && idx + 1 < part_len && part_chars[idx + 1].is_numeric() {
                                has_w = true;
                                tokens.push(DateToken::W);
                                date_norm.push('W');
                            }
                        }
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
                                            && matches!(part_chars[j], ':' | '+' | '[')
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
                                                    if num_date_digits >= 6 && in_digit_run_date {
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
                                    currently = IndexIn::PostDate;
                                }
                            }
                            if currently == IndexIn::Date {
                                tokens.push(DateToken::Space);
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
                                IndexIn::Bracket => bracket_colons += 1,
                                _ => {}
                            }
                        }
                        '.' => {
                            if date_norm.ends_with_dot() {
                                continue;
                            }
                            date_norm.push('.');
                            num_dot += 1;
                            let is_fractional_trigger = !seen_dot
                                && (currently.after_date()
                                    || (is_pure_numeric && num_non_decimal_digits >= 10));
                            if is_fractional_trigger {
                                seen_dot = true;
                                has_fractional = true;
                                currently = IndexIn::Fraction;
                            } else if currently == IndexIn::Date {
                                tokens.push(DateToken::Dot);
                            }
                        }
                        '/' => {
                            if date_norm.ends_with_slash() {
                                continue;
                            }
                            date_norm.push('/');
                            num_slash += 1;
                            if currently == IndexIn::Date {
                                tokens.push(DateToken::Slash);
                            }
                        }
                        ',' => {
                            if date_norm.ends_with_comma() {
                                continue;
                            }
                            date_norm.push(',');
                            num_comma += 1;
                            if currently == IndexIn::Date {
                                tokens.push(DateToken::Comma);
                            }
                        }
                        '+' => {
                            if date_norm.ends_with_plus() {
                                continue;
                            }
                            date_norm.push('+');
                            match currently {
                                IndexIn::Bracket => {
                                    bracket_has_sign = true;
                                }
                                _ => {
                                    if num_digits >= 8 {
                                        currently = IndexIn::Offset;
                                        if idx != 0 && part_chars[idx - 1] == ' ' {
                                            space_before_offset = true;
                                        }
                                    }
                                }
                            }
                        }
                        '-' => {
                            if date_norm.ends_with_minus() {
                                continue;
                            }
                            date_norm.push('-');
                            match currently {
                                IndexIn::Bracket => {
                                    bracket_has_sign = true;
                                }
                                _ => {
                                    if num_digits >= 8 {
                                        currently = IndexIn::Offset;
                                        if idx != 0 && part_chars[idx - 1] == ' ' {
                                            space_before_offset = true;
                                        }
                                    } else if currently == IndexIn::Date {
                                        num_hyphen += 1;
                                        tokens.push(DateToken::Hyphen);
                                    }
                                }
                            }
                        }
                        '[' => {
                            if date_norm.ends_with_lbracket() {
                                continue;
                            }
                            date_norm.push('[');
                            if idx != 0 && part_chars[idx - 1] == ' ' {
                                space_before_bracket = true;
                            }
                            currently = IndexIn::Bracket;
                            bracket_digits = 0;
                            bracket_colons = 0;
                            bracket_has_sign = false;
                        }
                        ']' => {
                            if date_norm.ends_with_rbracket() {
                                continue;
                            }
                            date_norm.push(']');
                            currently = IndexIn::PostDate;
                        }
                        _ => {
                            // In Date and digits both sides
                            // and an unrecognized character
                            // -> Add a separator
                            if currently == IndexIn::Date
                                && date_norm.ends_with_ascii_digit()
                                && idx + 1 < part_len
                                && part_chars[idx + 1].is_numeric()
                            {
                                date_norm.push('-');
                                num_hyphen += 1;
                                tokens.push(DateToken::Hyphen);
                            }
                        }
                    }
                }
            }

            // push digit runs after
            if curr_date_digit_run_len > 0 && matches!(currently, IndexIn::Date | IndexIn::PostDate)
            {
                tokens.push(DateToken::Digits(curr_date_digit_run_len));
                in_digit_run_date = false;
                curr_date_digit_run_len = 0;
            }
        }
    }

    if num_digits == 0 {
        return Err(an_err!(DtErrKind::InvalidInput, "0 digits"));
    }
    if curr_date_digit_run_len > 0 && matches!(currently, IndexIn::Date | IndexIn::PostDate) {
        tokens.push(DateToken::Digits(curr_date_digit_run_len));
    }
    if num_dot > 1 {
        is_pure_numeric = false;
    }
    let has_time =
        time_digits >= 3 || time_colons > 0 || connector.is_some() || has_ampm || has_fractional;
    let time = if has_time {
        if time_digits > 4 {
            TimeType::HmS {
                colons: time_colons > 0,
            }
        } else {
            TimeType::Hm {
                colons: time_colons > 0,
            }
        }
    } else {
        TimeType::None
    };

    let offset = if iana_offset == OffsetType::Zulu {
        OffsetType::Zulu
    } else if iana_offset == OffsetType::Iana {
        // Plain IANA name (optionally with a leading numeric offset)
        match offset_digits {
            0 => OffsetType::Iana,
            1..=4 => OffsetType::HmAndIana {
                colon: offset_colons > 0,
            },
            _ => OffsetType::HmSAndIana {
                colon: offset_colons > 0,
            },
        }
    } else if iana_offset == OffsetType::InBracketIana {
        // IANA name inside brackets (optionally with a leading numeric offset)
        match offset_digits {
            0 => OffsetType::InBracketIana,
            1..=4 => OffsetType::HmAndInbracketIana {
                colon: offset_colons > 0,
            },
            _ => OffsetType::HmSAndInbracketIana {
                colon: offset_colons > 0,
            },
        }
    } else if bracket_has_sign {
        // Numeric offset inside brackets (e.g. [+01:00] or [+0100])
        match bracket_digits {
            0..=4 => OffsetType::InBracketHm {
                colon: bracket_colons > 0,
            },
            _ => OffsetType::InBracketHmS {
                colon: bracket_colons > 0,
            },
        }
    } else if offset_digits > 0 {
        // Plain numeric offset (e.g. +01:00 or +0100)
        match offset_digits {
            1..=4 => OffsetType::Hm {
                colon: offset_colons > 0,
            },
            _ => OffsetType::HmS {
                colon: offset_colons > 0,
            },
        }
    } else {
        OffsetType::None
    };

    let has_year = matches!(tokens.first(), Some(DateToken::Digits(n)) if *n >= 4)
        || num_date_digit_groups >= 3
        || (num_named >= 1 && num_date_digit_groups >= 2)
        || (num_comma > 0 && num_date_digit_groups >= 2)
        || (num_digits >= 6 && num_named == 0)
        || (num_named >= 1 && num_date_digit_groups == 1 && num_date_digits >= 4)
        || (num_named >= 1 && has_time && num_digits >= 10);

    Ok(ClassifiedDate::Cls(DateClassification {
        ascii_len: ascii_len,
        date: date_norm,
        tokens: tokens,
        is_pure_numeric: is_pure_numeric,
        is_decimal: num_dot == 1 && is_pure_numeric,
        has_year: has_year,
        num_named: num_named,
        time: time,
        connector: connector,
        offset: offset,
        has_ampm: has_ampm,
        has_fractional: has_fractional,
        has_w: has_w,
        num_colon: num_colon,
        num_hyphen: num_hyphen,
        num_dot: num_dot,
        num_digits: num_digits,
        num_date_digits: num_date_digits,
        num_non_decimal_digits: num_non_decimal_digits,
        num_date_digit_groups: num_date_digit_groups,
        space_before_bracket: space_before_bracket,
        space_before_offset: space_before_offset,
        year_maybe_on_end: year_maybe_on_end,
    }))
}
