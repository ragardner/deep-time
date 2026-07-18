use crate::{DateClassification, Token, get_compatible_time_suffixes};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone)]
struct Builder {
    pieces: Vec<&'static str>,
    seen_weekday: bool,
    seen_month: bool,
    seen_day: bool,
    seen_year: bool,
}

#[inline(always)]
fn append_to_all(builders: &mut Vec<Builder>, s: &'static str) {
    for b in builders {
        b.pieces.push(s);
    }
}

pub(crate) fn generate_unambiguous_candidates(class: &DateClassification) -> Vec<String> {
    if !class.has_year || (class.is_pure_numeric && class.num_dot == 0) {
        return vec![];
    }

    let suffixes = get_compatible_time_suffixes(class);
    let tokens = &class.date_tokens;

    if class.has_w {
        let mut candidates = Vec::with_capacity(4 * suffixes.len() + 2);
        let mut builders: Vec<Builder> = vec![Builder {
            pieces: Vec::with_capacity(12),
            seen_weekday: false,
            seen_month: false,
            seen_day: false,
            seen_year: false,
        }];
        let mut i = 0;

        // Walk only the prefix tokens that come before the week-date core
        while i < tokens.len() {
            // Detect the start of the ISO week part (handles both YYYYW... and YYYY-W...)
            if matches!(tokens[i], Token::Digits(4)) && i + 1 < tokens.len() {
                let next = &tokens[i + 1];
                if matches!(next, Token::W)
                    || (matches!(next, Token::Hyphen)
                        && i + 2 < tokens.len()
                        && matches!(tokens[i + 2], Token::W))
                {
                    break;
                }
            }
            match tokens[i] {
                Token::DayShort => {
                    let mut new_builders = Vec::with_capacity(builders.len());
                    for b in &builders {
                        if !b.seen_weekday {
                            let mut nb = b.clone();
                            nb.pieces.push("%a");
                            nb.seen_weekday = true;
                            new_builders.push(nb);
                        }
                    }
                    builders = new_builders;
                }
                Token::DayLong => {
                    let mut new_builders = Vec::with_capacity(builders.len());
                    for b in &builders {
                        if !b.seen_weekday {
                            let mut nb = b.clone();
                            nb.pieces.push("%A");
                            nb.seen_weekday = true;
                            new_builders.push(nb);
                        }
                    }
                    builders = new_builders;
                }
                Token::MonthShort => {
                    let mut new_builders = Vec::with_capacity(builders.len());
                    for b in &builders {
                        if !b.seen_month {
                            let mut nb = b.clone();
                            nb.pieces.push("%b");
                            nb.seen_month = true;
                            new_builders.push(nb);
                        }
                    }
                    builders = new_builders;
                }
                Token::MonthLong => {
                    let mut new_builders = Vec::with_capacity(builders.len());
                    for b in &builders {
                        if !b.seen_month {
                            let mut nb = b.clone();
                            nb.pieces.push("%B");
                            nb.seen_month = true;
                            new_builders.push(nb);
                        }
                    }
                    builders = new_builders;
                }
                Token::Comma => append_to_all(&mut builders, ","),
                Token::Space => append_to_all(&mut builders, " "),
                Token::Hyphen => append_to_all(&mut builders, "-"),
                Token::Dot => append_to_all(&mut builders, "."),
                Token::Slash => append_to_all(&mut builders, "/"),
                _ => {}
            }
            i += 1;
        }

        for b in builders {
            let mut new_b = b.clone();
            // ISO week-year; calendar %Y leaves iso_wk_yr unset and can "succeed"
            // as a bare year when trailing week digits are left unmatched.
            new_b.pieces.push("%G");

            for token in &tokens[i + 1..] {
                match token {
                    Token::W => new_b.pieces.push("W"),
                    Token::Digits(2) => new_b.pieces.push("%V"),
                    Token::Digits(1) => new_b.pieces.push("%u"),
                    // Compact "2024W114" = week 11 + weekday 4
                    Token::Digits(3) => {
                        new_b.pieces.push("%V");
                        new_b.pieces.push("%u");
                    }
                    Token::Hyphen => new_b.pieces.push("-"),
                    Token::Space => new_b.pieces.push(" "),
                    Token::Dot => new_b.pieces.push("."),
                    Token::Slash => new_b.pieces.push("/"),
                    _ => {}
                }
            }

            let date_part: String = new_b.pieces.concat();

            if !class.has_time {
                candidates.push(date_part.clone());
            }
            for suf in &suffixes {
                let mut s = date_part.clone();
                s.push_str(suf);
                candidates.push(s);
            }
        }
        return candidates;
    }

    let starts_with_4digits = matches!(tokens.first(), Some(Token::Digits(n)) if *n >= 4);
    let year_fmt = if class.num_date_digits == 4 && !starts_with_4digits {
        "%y"
    } else {
        "%Y"
    };

    let mut builders: Vec<Builder> = vec![Builder {
        pieces: Vec::with_capacity(12),
        seen_weekday: false,
        seen_month: false,
        seen_day: false,
        seen_year: false,
    }];
    let no_named = class.num_named == 0;
    // Named "Feb 29 2023": 2-digit group is the day when a 4-digit year exists —
    // do not also try %y (that yields "%b %y %Y" → day defaults to 1).
    let has_4digit_group = tokens
        .iter()
        .any(|t| matches!(t, Token::Digits(n) if *n >= 4));

    for &token in tokens {
        match token {
            Token::Hyphen => append_to_all(&mut builders, "-"),
            Token::Slash => append_to_all(&mut builders, "/"),
            Token::Dot => append_to_all(&mut builders, "."),
            Token::Space => append_to_all(&mut builders, " "),
            Token::Comma => append_to_all(&mut builders, ", "),
            Token::DayShort => {
                let mut new_builders = Vec::with_capacity(builders.len());
                for b in builders {
                    if !b.seen_weekday {
                        let mut nb = b.clone();
                        nb.pieces.push("%a");
                        nb.seen_weekday = true;
                        new_builders.push(nb);
                    }
                }
                builders = new_builders;
            }
            Token::DayLong => {
                let mut new_builders = Vec::with_capacity(builders.len());
                for b in builders {
                    if !b.seen_weekday {
                        let mut nb = b.clone();
                        nb.pieces.push("%A");
                        nb.seen_weekday = true;
                        new_builders.push(nb);
                    }
                }
                builders = new_builders;
            }
            Token::MonthShort => {
                let mut new_builders = Vec::with_capacity(builders.len());
                for b in builders {
                    if !b.seen_month {
                        let mut nb = b.clone();
                        nb.pieces.push("%b");
                        nb.seen_month = true;
                        new_builders.push(nb);
                    }
                }
                builders = new_builders;
            }
            Token::MonthLong => {
                let mut new_builders = Vec::with_capacity(builders.len());
                for b in builders {
                    if !b.seen_month {
                        let mut nb = b.clone();
                        nb.pieces.push("%B");
                        nb.seen_month = true;
                        new_builders.push(nb);
                    }
                }
                builders = new_builders;
            }
            Token::Digits(n) => {
                let mut new_builders = Vec::with_capacity(builders.len() * 4);
                let digit_len = n;
                for b in builders {
                    let all_opts: &[&'static str] = token.to_fmt_year_first();
                    let add_spec = |spec: &'static str, new_builders: &mut Vec<Builder>| {
                        let mut new_b = b.clone();
                        new_b.pieces.push(spec);

                        match spec {
                            "%Y%m%d" | "%y%m%d" => {
                                new_b.seen_year = true;
                                new_b.seen_month = true;
                                new_b.seen_day = true;
                            }
                            "%y%j" => {
                                new_b.seen_year = true;
                                new_b.seen_day = true;
                            }
                            "%d" | "%e" | "%-d" | "%_d" | "%j" => new_b.seen_day = true,
                            "%m" | "%-m" | "%_m" => new_b.seen_month = true,
                            "%Y" | "%G" | "%y" | "%C" => new_b.seen_year = true,
                            "%V" | "%U" | "%W" => new_b.seen_year = true,
                            _ => {}
                        }
                        new_builders.push(new_b);
                    };
                    let mut processed = false;
                    for &spec in all_opts {
                        let should_add = if no_named {
                            matches!(spec, "%Y" | "%G" | "%y" | "%C") && !b.seen_year
                                || matches!(spec, "%m" | "%-m" | "%_m")
                                    && b.seen_year
                                    && !b.seen_month
                                || matches!(spec, "%d" | "%e" | "%-d" | "%_d")
                                    && b.seen_year
                                    && b.seen_month
                                    && !b.seen_day
                                || spec == "%j" && b.seen_year && !b.seen_day && !b.seen_month
                        } else {
                            matches!(spec, "%d" | "%e" | "%-d" | "%_d") && !b.seen_day
                                || matches!(spec, "%m" | "%-m" | "%_m") && !b.seen_month
                                || matches!(spec, "%Y" | "%G" | "%y" | "%C")
                                    && !b.seen_year
                                    // Prefer day for 1–2 digit groups when a 4-digit year exists.
                                    && !(digit_len <= 2 && has_4digit_group)
                                || spec == "%j" && !b.seen_day && !b.seen_year
                                || matches!(spec, "%V" | "%U" | "%W")
                                    && !b.seen_year
                                    && !b.seen_day
                                    && !b.seen_month
                                || (matches!(spec, "%Y%m%d" | "%y%m%d")
                                    && !b.seen_year
                                    && !b.seen_month
                                    && !b.seen_day)
                                || (spec == "%y%j" && !b.seen_year && !b.seen_day)
                        };

                        if should_add {
                            processed = true;
                            add_spec(spec, &mut new_builders);
                        }
                    }
                    if !processed {
                        let should_prune = no_named || (b.seen_year && b.seen_month && b.seen_day);
                        if !should_prune {
                            for &spec in all_opts {
                                add_spec(spec, &mut new_builders);
                            }
                        }
                    }
                }
                builders = new_builders;
            }
            _ => {}
        }
    }

    let mut candidates = Vec::with_capacity(builders.len() * (suffixes.len() + 2));
    for b in builders {
        let date_part: String = b.pieces.concat();
        if !class.has_time {
            let mut s = date_part.clone();
            if class.year_maybe_on_end && !b.seen_year {
                s.push(' ');
                s.push_str(year_fmt);
            }
            candidates.push(s);
        }
        for suffix in &suffixes {
            let mut s = date_part.clone();
            s.push_str(suffix);
            if class.year_maybe_on_end && !b.seen_year {
                s.push(' ');
                s.push_str(year_fmt);
            }
            candidates.push(s);
        }
    }

    candidates
}
