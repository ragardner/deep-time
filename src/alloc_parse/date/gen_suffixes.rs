use crate::{DateClassification, Token};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Returns all compatible strftime suffix strings for the given parsed date/time.
///
/// The zone/offset/iana part is fully token-driven.
/// The time base still uses a few flags (has_ampm, has_fractional, is_pure_numeric)
/// because those control variant generation (12h vs 24h, with/without seconds).
pub(crate) fn get_compatible_time_suffixes(class: &DateClassification) -> Vec<String> {
    if !class.has_time {
        let suff = build_zone_suffix(class);
        return if suff.is_empty() {
            Vec::new()
        } else {
            vec![suff]
        };
    }

    let mut time_bases = build_time_bases(class);
    let zone = build_zone_suffix(class);
    if !zone.is_empty() {
        for base in &mut time_bases {
            base.push_str(&zone);
        }
    }
    // eprintln!("{:?}", time_bases);
    time_bases
}

/// Builds the zone/offset/iana suffix by walking `time_tokens` after the main time token.
///
/// Examples: "%z", "%:z", " [%:z] %Q", "Z"
/// - Never emits literal + or - (the %z family includes the sign)
fn build_zone_suffix(class: &DateClassification) -> String {
    let tokens = &class.time_tokens;
    if tokens.is_empty() {
        return String::new();
    }

    let mut s = String::with_capacity(32);
    let mut i = 0usize;

    // Skip leading main time token
    if i < tokens.len() && is_main_time_token(tokens[i]) {
        i += 1;
    }

    while i < tokens.len() {
        match tokens[i] {
            Token::Pm | Token::Am => {
                s.push_str("%p");
                i += 1;
            }
            Token::Space => {
                s.push(' ');
                i += 1;
            }
            Token::Plus | Token::Minus => {
                i += 1;
            } // sign lives in %z/%:z
            Token::Offset => {
                s.push_str("%z");
                i += 1;
            }
            Token::OffsetColon => {
                s.push_str("%:z");
                i += 1;
            }
            Token::LBracket => {
                s.push('[');
                i += 1;
            }
            Token::RBracket => {
                s.push(']');
                i += 1;
            }
            Token::Iana => {
                s.push_str("%Q");
                i += 1;
            }
            Token::Zulu => {
                s.push('Z');
                i += 1;
            }
            Token::Scale => {
                s.push_str("%L");
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    s
}

#[inline]
fn build_time_bases(class: &DateClassification) -> Vec<String> {
    let tokens = &class.time_tokens;
    if tokens.is_empty() {
        return Vec::new();
    }

    // Derive has_seconds and time_sep directly from the first token
    let first = tokens[0];
    let (has_seconds, time_sep) = match first {
        Token::Hms | Token::HmsColon => (true, if first == Token::HmsColon { ":" } else { "" }),
        Token::Hm | Token::HmColon => (false, if first == Token::HmColon { ":" } else { "" }),
        _ => return Vec::new(),
    };

    let mut suffixes: Vec<String> = Vec::with_capacity(4);
    let connector_str = class.connector.as_str();
    let use_fractional = class.has_fractional;
    let is_12h = class.has_ampm;

    let include_extra = !use_fractional && class.is_pure_numeric;

    let has_seconds_options: Vec<bool> = if use_fractional {
        vec![true]
    } else if include_extra {
        if has_seconds {
            vec![true, false]
        } else {
            vec![false, true]
        }
    } else {
        vec![has_seconds]
    };

    if is_12h {
        for &sec in &has_seconds_options {
            let mut base = String::with_capacity(32);
            base.push_str(connector_str);
            base.push_str("%I");
            base.push_str(time_sep);
            base.push_str("%M");

            if use_fractional {
                base.push_str(time_sep);
                base.push_str("%S%.f");
                suffixes.push(base);
            } else if sec {
                base.push_str(time_sep);
                base.push_str("%S");
                suffixes.push(base);
            }
        }
    } else {
        for hour in ["%H", "%k"] {
            for &sec in &has_seconds_options {
                let mut base = String::with_capacity(32);
                base.push_str(connector_str);
                base.push_str(hour);
                base.push_str(time_sep);
                base.push_str("%M");

                if use_fractional {
                    base.push_str(time_sep);
                    base.push_str("%S%.f");
                    suffixes.push(base);
                } else if sec {
                    base.push_str(time_sep);
                    base.push_str("%S");
                    suffixes.push(base);
                } else {
                    suffixes.push(base);
                }
            }
        }
    }
    suffixes
}

#[inline]
fn is_main_time_token(t: Token) -> bool {
    matches!(t, Token::Hms | Token::HmsColon | Token::Hm | Token::HmColon)
}
