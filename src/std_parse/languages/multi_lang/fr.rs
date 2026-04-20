use crate::{
    DateToken, EN_DAYS, EN_DURATIONS, EN_MONTHS, EN_RELATIVES, EN_SPECIAL, LangData, TZ_ENTRIES,
};
use aho_corasick::{AhoCorasick, MatchKind};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::vec::Vec;

#[allow(dead_code)]
pub(crate) const FR_RELATIVES: &[(&'static str, &'static str, DateToken)] = &[
    ("et", "and", DateToken::Plus),
    ("plus", "plus", DateToken::Plus),
    // Temporal
    ("dans", "in", DateToken::Future),
    ("en", "in", DateToken::Future),
    ("prochain", "next", DateToken::Future),
    ("prochaine", "next", DateToken::Future),
    ("après", "after", DateToken::Future),
    ("il y a", "ago", DateToken::Past),
    ("dernier", "last", DateToken::Past),
    ("dernière", "last", DateToken::Past),
    ("précédent", "last", DateToken::Past),
    ("précédente", "last", DateToken::Past),
    ("maintenant", "now", DateToken::Now),
    ("aujourd'hui", "today", DateToken::Today),
    ("demain", "tomorrow", DateToken::Tomorrow),
    ("hier", "yesterday", DateToken::Yesterday),
    // Sub-second units
    ("nanosecondes", "ns", DateToken::Nanosecond),
    ("nanoseconde", "ns", DateToken::Nanosecond),
    ("nanos", "ns", DateToken::Nanosecond),
    ("microsecondes", "us", DateToken::Microsecond),
    ("microseconde", "us", DateToken::Microsecond),
    ("micros", "us", DateToken::Microsecond),
    ("millisecondes", "ms", DateToken::Millisecond),
    ("milliseconde", "ms", DateToken::Millisecond),
    ("millis", "ms", DateToken::Millisecond),
    // Seconds
    ("secondes", "s", DateToken::Second),
    ("seconde", "s", DateToken::Second),
    ("secs", "s", DateToken::Second),
    ("sec", "s", DateToken::Second),
    // Minutes
    ("minutes", "m", DateToken::Minute),
    ("minute", "m", DateToken::Minute),
    ("mins", "m", DateToken::Minute),
    ("min", "m", DateToken::Minute),
    // Hours
    ("heures", "h", DateToken::Hour),
    ("heure", "h", DateToken::Hour),
    ("hrs", "h", DateToken::Hour),
    ("hr", "h", DateToken::Hour),
    // Days
    ("jours", "d", DateToken::Day),
    ("jour", "d", DateToken::Day),
    // ("j", "d", DateToken::Day), // common French abbreviation for "jour"
    // Weeks
    ("semaines", "w", DateToken::Week),
    ("semaine", "w", DateToken::Week),
    // Months
    ("mois", "mo", DateToken::Month),
    // Years
    ("années", "y", DateToken::Year),
    ("année", "y", DateToken::Year),
    ("ans", "y", DateToken::Year),
    ("an", "y", DateToken::Year),
];

/// Any missing short and long units from RELATIVES
#[allow(dead_code)]
pub(crate) const FR_DURATIONS: &[(&'static str, &'static str, DateToken)] = &[
    ("y", "y", DateToken::Year),
    ("w", "w", DateToken::Week),
    ("d", "d", DateToken::Day),
    ("h", "h", DateToken::Hour),
    ("m", "m", DateToken::Minute),
    ("s", "s", DateToken::Second),
    ("millénaire", "mil", DateToken::Millennium),
    ("millénaires", "mil", DateToken::Millennium),
    ("siècle", "cen", DateToken::Century),
    ("siècles", "cen", DateToken::Century),
    ("décennie", "dec", DateToken::Decade),
    ("décennies", "dec", DateToken::Decade),
    ("trimestre", "qr", DateToken::Quarter),
    ("trimestres", "qr", DateToken::Quarter),
    ("quinzaine", "fn", DateToken::Fortnight),
    ("quinzaines", "fn", DateToken::Fortnight),
    ("kilosecondes", "ks", DateToken::Kilosecond),
    ("kiloseconde", "ks", DateToken::Kilosecond),
    ("mégasecondes", "mgs", DateToken::Megasecond),
    ("mégaseconde", "mgs", DateToken::Megasecond),
    ("gigasecondes", "gs", DateToken::Gigasecond),
    ("gigaseconde", "gs", DateToken::Gigasecond),
    ("térasecondes", "ts", DateToken::Terasecond),
    ("téraseconde", "ts", DateToken::Terasecond),
    ("pétasecondes", "pes", DateToken::Petasecond),
    ("pétaseconde", "pes", DateToken::Petasecond),
    ("exasecondes", "es", DateToken::Exasecond),
    ("exaseconde", "es", DateToken::Exasecond),
    ("zettasecondes", "zes", DateToken::Zettasecond),
    ("zettaseconde", "zes", DateToken::Zettasecond),
    ("yottasecondes", "yos", DateToken::Yottasecond),
    ("yottaseconde", "yos", DateToken::Yottasecond),
    ("ronnasecondes", "ros", DateToken::Ronnasecond),
    ("ronnaseconde", "ros", DateToken::Ronnasecond),
    ("quettasecondes", "qus", DateToken::Quettasecond),
    ("quettaseconde", "qus", DateToken::Quettasecond),
    ("picosecondes", "ps", DateToken::Picosecond),
    ("picoseconde", "ps", DateToken::Picosecond),
    ("femtosecondes", "fs", DateToken::Femtosecond),
    ("femtoseconde", "fs", DateToken::Femtosecond),
    ("attosecondes", "as", DateToken::Attosecond),
    ("attoseconde", "as", DateToken::Attosecond),
    ("zeptosecondes", "zs", DateToken::Zeptosecond),
    ("zeptoseconde", "zs", DateToken::Zeptosecond),
    ("yoctosecondes", "ys", DateToken::Yoctosecond),
    ("yoctoseconde", "ys", DateToken::Yoctosecond),
    ("rontosecondes", "rs", DateToken::Rontosecond),
    ("rontoseconde", "rs", DateToken::Rontosecond),
    ("quectosecondes", "qs", DateToken::Quectosecond),
    ("quectoseconde", "qs", DateToken::Quectosecond),
    // Short forms (same as EN)
    ("mil", "mil", DateToken::Millennium),
    ("cen", "cen", DateToken::Century),
    ("dec", "dec", DateToken::Decade),
    ("qr", "qr", DateToken::Quarter),
    ("fn", "fn", DateToken::Fortnight),
    ("ks", "ks", DateToken::Kilosecond),
    ("mgs", "mgs", DateToken::Megasecond),
    ("gs", "gs", DateToken::Gigasecond),
    ("ts", "ts", DateToken::Terasecond),
    ("pes", "pes", DateToken::Petasecond),
    ("es", "es", DateToken::Exasecond),
    ("zes", "zes", DateToken::Zettasecond),
    ("yos", "yos", DateToken::Yottasecond),
    ("ros", "ros", DateToken::Ronnasecond),
    ("qus", "qus", DateToken::Quettasecond),
    ("ms", "ms", DateToken::Millisecond),
    ("u", "us", DateToken::Microsecond),
    ("us", "us", DateToken::Microsecond),
    ("ns", "ns", DateToken::Nanosecond),
    ("ps", "ps", DateToken::Picosecond),
    ("fs", "fs", DateToken::Femtosecond),
    ("as", "as", DateToken::Attosecond),
    ("zs", "zs", DateToken::Zeptosecond),
    ("ys", "ys", DateToken::Yoctosecond),
    ("rs", "rs", DateToken::Rontosecond),
    ("qs", "qs", DateToken::Quectosecond),
];

#[allow(dead_code)]
pub(crate) const FR_MONTHS: &[(&'static str, &'static str, DateToken)] = &[
    // Short months (French abbreviations) → English short form
    ("janv", "Jan", DateToken::MonthShort),
    ("févr", "Feb", DateToken::MonthShort),
    ("mars", "Mar", DateToken::MonthShort),
    ("avr", "Apr", DateToken::MonthShort),
    ("mai", "May", DateToken::MonthShort),
    ("juin", "Jun", DateToken::MonthShort),
    ("juil", "Jul", DateToken::MonthShort),
    ("août", "Aug", DateToken::MonthShort),
    ("sept", "Sep", DateToken::MonthShort),
    ("oct", "Oct", DateToken::MonthShort),
    ("nov", "Nov", DateToken::MonthShort),
    ("déc", "Dec", DateToken::MonthShort),
    // Long months (French) → English long form
    ("janvier", "January", DateToken::MonthLong),
    ("février", "February", DateToken::MonthLong),
    ("mars", "March", DateToken::MonthLong),
    ("avril", "April", DateToken::MonthLong),
    ("mai", "May", DateToken::MonthLong),
    ("juin", "June", DateToken::MonthLong),
    ("juillet", "July", DateToken::MonthLong),
    ("août", "August", DateToken::MonthLong),
    ("septembre", "September", DateToken::MonthLong),
    ("octobre", "October", DateToken::MonthLong),
    ("novembre", "November", DateToken::MonthLong),
    ("décembre", "December", DateToken::MonthLong),
];

#[allow(dead_code)]
pub(crate) const FR_DAYS: &[(&'static str, &'static str, DateToken)] = &[
    // Short days (French) → English short form
    ("lun", "Mon", DateToken::DayShort),
    ("mar", "Tue", DateToken::DayShort),
    ("mer", "Wed", DateToken::DayShort),
    ("jeu", "Thu", DateToken::DayShort),
    ("ven", "Fri", DateToken::DayShort),
    ("sam", "Sat", DateToken::DayShort),
    ("dim", "Sun", DateToken::DayShort),
    // Long days (French) → English long form
    ("lundi", "Monday", DateToken::DayLong),
    ("mardi", "Tuesday", DateToken::DayLong),
    ("mercredi", "Wednesday", DateToken::DayLong),
    ("jeudi", "Thursday", DateToken::DayLong),
    ("vendredi", "Friday", DateToken::DayLong),
    ("samedi", "Saturday", DateToken::DayLong),
    ("dimanche", "Sunday", DateToken::DayLong),
];

#[allow(dead_code)]
pub(crate) const FR_SPECIAL: &[(&'static str, &'static str, DateToken)] =
    &[("am", "AM", DateToken::Am), ("pm", "PM", DateToken::Pm)];

#[allow(dead_code)]
static FR_DATE_AC: LazyLock<AhoCorasick> = LazyLock::new(|| {
    let mut terms: Vec<&'static str> = Vec::with_capacity(
        FR_RELATIVES.len() + FR_MONTHS.len() + FR_DAYS.len() + FR_SPECIAL.len() + TZ_ENTRIES.len(),
    );
    terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
    terms.extend(EN_MONTHS.iter().map(|&(k, _, _)| k));
    terms.extend(EN_DAYS.iter().map(|&(k, _, _)| k));
    terms.extend(EN_SPECIAL.iter().map(|&(k, _, _)| k));
    terms.extend(FR_RELATIVES.iter().map(|&(k, _, _)| k));
    terms.extend(FR_MONTHS.iter().map(|&(k, _, _)| k));
    terms.extend(FR_DAYS.iter().map(|&(k, _, _)| k));
    terms.extend(FR_SPECIAL.iter().map(|&(k, _, _)| k));
    for (name, _, _) in TZ_ENTRIES.iter() {
        let lowered = name.to_lowercase();
        let leaked: &'static str = Box::leak(lowered.into_boxed_str());
        terms.push(leaked);
    }
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&terms)
        .expect("invalid Aho-Corasick patterns for FR date terms")
});

#[allow(dead_code)]
static FR_DURATION_AC: LazyLock<AhoCorasick> = LazyLock::new(|| {
    let mut terms: Vec<&'static str> = Vec::with_capacity(FR_RELATIVES.len() + FR_DURATIONS.len());
    terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
    terms.extend(EN_DURATIONS.iter().map(|&(k, _, _)| k));
    terms.extend(FR_RELATIVES.iter().map(|&(k, _, _)| k));
    terms.extend(FR_DURATIONS.iter().map(|&(k, _, _)| k));
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&terms)
        .expect("invalid Aho-Corasick patterns for FR duration terms")
});

#[allow(dead_code)]
pub(crate) static FR: LazyLock<HashMap<&'static str, (&'static str, DateToken)>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        for &(k, v, token) in EN_RELATIVES {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in EN_DURATIONS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in EN_MONTHS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in EN_DAYS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in EN_SPECIAL {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in FR_RELATIVES {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in FR_DURATIONS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in FR_MONTHS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in FR_DAYS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in FR_SPECIAL {
            m.insert(k, (v, token));
        }
        for (name, _, _) in TZ_ENTRIES.iter() {
            let lowered = name.to_lowercase();
            let key: &'static str = Box::leak(lowered.into_boxed_str());
            m.insert(key, (name, DateToken::Iana));
        }
        m
    });

#[allow(dead_code)]
pub(crate) static FR_LANG_DATA: LangData = LangData {
    map: &FR,
    date_ac: &FR_DATE_AC,
    duration_ac: &FR_DURATION_AC,
};
