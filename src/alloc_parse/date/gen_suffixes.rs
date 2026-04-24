use crate::{DateClassification, OffsetType, TimeType};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[inline]
pub(crate) fn get_compatible_time_suffixes(class: &DateClassification) -> Vec<String> {
    if class.time.is_none() {
        return get_offset_suffix(class).map_or_else(Vec::new, |s| vec![s]);
    }

    let mut time_bases = build_time_bases(class);
    if let Some(suff) = get_offset_suffix(class) {
        for base in &mut time_bases {
            base.push_str(&suff);
        }
    }
    time_bases
}

#[inline]
fn get_offset_suffix(class: &DateClassification) -> Option<String> {
    let offset = class.offset;
    match offset {
        OffsetType::None => None,
        OffsetType::Zulu => Some(String::from("Z")),

        _ => {
            let prefix = if offset.is_bracketed() {
                if class.space_before_bracket { " " } else { "" }
            } else {
                if class.space_before_offset { " " } else { "" }
            };

            let mut s = String::with_capacity(20);
            s.push_str(prefix);

            match offset {
                OffsetType::Iana => s.push_str("%Q"),
                OffsetType::InBracketIana => s.push_str("[%Q]"),
                OffsetType::Hm { colon } => s.push_str(offset_spec(colon, false)),
                OffsetType::HmS { colon } => s.push_str(offset_spec(colon, true)),
                OffsetType::InBracketHm { colon } => {
                    s.push('[');
                    s.push_str(offset_spec(colon, false));
                    s.push(']');
                }
                OffsetType::InBracketHmS { colon } => {
                    s.push('[');
                    s.push_str(offset_spec(colon, true));
                    s.push(']');
                }
                OffsetType::HmAndIana { colon } => {
                    s.push_str(offset_spec(colon, false));
                    s.push_str(" %Q");
                }
                OffsetType::HmSAndIana { colon } => {
                    s.push_str(offset_spec(colon, true));
                    s.push_str(" %Q");
                }
                OffsetType::HmAndInbracketIana { colon } => {
                    s.push_str(offset_spec(colon, false));
                    s.push_str("[%Q]");
                }
                OffsetType::HmSAndInbracketIana { colon } => {
                    s.push_str(offset_spec(colon, true));
                    s.push_str("[%Q]");
                }
                _ => unreachable!(),
            }
            Some(s)
        }
    }
}

#[inline]
fn build_time_bases(class: &DateClassification) -> Vec<String> {
    let mut suffixes: Vec<String> = Vec::with_capacity(4);
    let connector_str = class.connector.as_str();
    let use_fractional = class.has_fractional;
    let (preferred_has_seconds, time_colons) = match class.time {
        TimeType::Hm { colons } => (false, colons),
        TimeType::HmS { colons } => (true, colons),
        TimeType::None => unreachable!(),
    };
    let time_sep = if time_colons { ":" } else { "" };
    let is_12h = class.has_ampm;

    let include_extra = !use_fractional && class.is_pure_numeric;

    let has_seconds_options: Vec<bool> = if use_fractional {
        vec![true]
    } else if include_extra {
        if preferred_has_seconds {
            vec![true, false]
        } else {
            vec![false, true]
        }
    } else {
        vec![preferred_has_seconds]
    };

    if is_12h {
        for &has_seconds in &has_seconds_options {
            let mut base = String::with_capacity(32);
            base.push_str(connector_str);
            base.push_str("%I");
            base.push_str(time_sep);
            base.push_str("%M");

            if use_fractional {
                base.push_str(time_sep);
                base.push_str("%S%.f %p");
                suffixes.push(base);
            } else if has_seconds {
                base.push_str(time_sep);
                base.push_str("%S %p");
                suffixes.push(base);
            } else {
                base.push_str(" %p");
                suffixes.push(base);
            }
        }
    } else {
        for hour in ["%H", "%k"] {
            for &has_seconds in &has_seconds_options {
                let mut base = String::with_capacity(32);
                base.push_str(connector_str);
                base.push_str(hour);
                base.push_str(time_sep);
                base.push_str("%M");

                if use_fractional {
                    base.push_str(time_sep);
                    base.push_str("%S%.f");
                    suffixes.push(base);
                } else if has_seconds {
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

#[inline(always)]
fn offset_spec(colon: bool, has_seconds: bool) -> &'static str {
    match (has_seconds, colon) {
        (true, true) => "%::z",
        (true, false) => "%z",
        (false, true) => "%:z",
        (false, false) => "%z",
    }
}
