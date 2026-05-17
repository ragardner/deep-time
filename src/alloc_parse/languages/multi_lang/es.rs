use crate::{
    DateToken, EN_DAYS, EN_DURATIONS, EN_MONTHS, EN_RELATIVES, EN_SPECIAL, LangData,
    tz_lowered_keys, tzdb::TZ_ENTRIES,
};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashMap;
use once_cell::race::OnceBox;

pub(crate) const ES_RELATIVES: &[(&str, &str, DateToken)] = &[
    ("y", "and", DateToken::Plus),
    ("e", "and", DateToken::Plus),
    ("más", "plus", DateToken::Plus),
    ("plus", "plus", DateToken::Plus),
    // Temporal
    ("en", "in", DateToken::Future),
    ("dentro de", "in", DateToken::Future),
    ("próximo", "next", DateToken::Future),
    ("próxima", "next", DateToken::Future),
    ("próximos", "next", DateToken::Future),
    ("que viene", "next", DateToken::Future),
    ("después", "after", DateToken::Future),
    ("hace", "ago", DateToken::Past),
    ("pasado", "last", DateToken::Past),
    ("pasada", "last", DateToken::Past),
    ("último", "last", DateToken::Past),
    ("última", "last", DateToken::Past),
    ("anterior", "last", DateToken::Past),
    ("ahora", "now", DateToken::Now),
    ("hoy", "today", DateToken::Today),
    ("mañana", "tomorrow", DateToken::Tomorrow),
    ("ayer", "yesterday", DateToken::Yesterday),
    ("pasado mañana", "tomorrow", DateToken::Tomorrow),
    ("anteayer", "yesterday", DateToken::Yesterday),
    ("antier", "yesterday", DateToken::Yesterday),
    // Sub-second units
    ("nanosegundos", "ns", DateToken::Nanosecond),
    ("nanosegundo", "ns", DateToken::Nanosecond),
    ("nanos", "ns", DateToken::Nanosecond),
    ("microsegundos", "us", DateToken::Microsecond),
    ("microsegundo", "us", DateToken::Microsecond),
    ("micros", "us", DateToken::Microsecond),
    ("milisegundos", "ms", DateToken::Millisecond),
    ("milisegundo", "ms", DateToken::Millisecond),
    ("millis", "ms", DateToken::Millisecond),
    // Seconds
    ("segundos", "s", DateToken::Second),
    ("segundo", "s", DateToken::Second),
    ("seg", "s", DateToken::Second),
    ("secs", "s", DateToken::Second),
    // Minutes
    ("minutos", "m", DateToken::Minute),
    ("minuto", "m", DateToken::Minute),
    ("mins", "m", DateToken::Minute),
    ("min", "m", DateToken::Minute),
    // Hours
    ("horas", "h", DateToken::Hour),
    ("hora", "h", DateToken::Hour),
    ("hrs", "h", DateToken::Hour),
    ("hr", "h", DateToken::Hour),
    // Days
    ("días", "d", DateToken::Day),
    ("día", "d", DateToken::Day),
    // Weeks
    ("semanas", "w", DateToken::Week),
    ("semana", "w", DateToken::Week),
    // Months
    ("meses", "mo", DateToken::Month),
    ("mes", "mo", DateToken::Month),
    // Years
    ("años", "y", DateToken::Year),
    ("año", "y", DateToken::Year),
    ("a", "y", DateToken::Year),
];

pub(crate) const ES_DURATIONS: &[(&str, &str, DateToken)] = &[
    ("y", "y", DateToken::Year),
    ("w", "w", DateToken::Week),
    ("d", "d", DateToken::Day),
    ("h", "h", DateToken::Hour),
    ("m", "m", DateToken::Minute),
    ("s", "s", DateToken::Second),
    ("milenio", "mil", DateToken::Millennium),
    ("milenios", "mil", DateToken::Millennium),
    ("siglo", "cen", DateToken::Century),
    ("siglos", "cen", DateToken::Century),
    ("década", "dec", DateToken::Decade),
    ("décadas", "dec", DateToken::Decade),
    ("trimestre", "qr", DateToken::Quarter),
    ("trimestres", "qr", DateToken::Quarter),
    ("quincena", "fn", DateToken::Fortnight),
    ("quincenas", "fn", DateToken::Fortnight),
    ("kilosegundos", "ks", DateToken::Kilosecond),
    ("kilosegundo", "ks", DateToken::Kilosecond),
    ("megasegundos", "mgs", DateToken::Megasecond),
    ("megasegundo", "mgs", DateToken::Megasecond),
    ("gigasegundos", "gs", DateToken::Gigasecond),
    ("gigasegundo", "gs", DateToken::Gigasecond),
    ("terasegundos", "ts", DateToken::Terasecond),
    ("terasegundo", "ts", DateToken::Terasecond),
    ("petasegundos", "pes", DateToken::Petasecond),
    ("petasegundo", "pes", DateToken::Petasecond),
    ("exasegundos", "es", DateToken::Exasecond),
    ("exasegundo", "es", DateToken::Exasecond),
    ("zettasegundos", "zes", DateToken::Zettasecond),
    ("zettasegundo", "zes", DateToken::Zettasecond),
    ("yottasegundos", "yos", DateToken::Yottasecond),
    ("yottasegundo", "yos", DateToken::Yottasecond),
    ("ronnasegundos", "ros", DateToken::Ronnasecond),
    ("ronnasegundo", "ros", DateToken::Ronnasecond),
    ("quettasegundos", "qus", DateToken::Quettasecond),
    ("quettasegundo", "qus", DateToken::Quettasecond),
    ("picosegundos", "ps", DateToken::Picosecond),
    ("picosegundo", "ps", DateToken::Picosecond),
    ("femtosegundos", "fs", DateToken::Femtosecond),
    ("femtosegundo", "fs", DateToken::Femtosecond),
    ("attosegundos", "as", DateToken::Attosecond),
    ("attosegundo", "as", DateToken::Attosecond),
    ("zeptosegundos", "zs", DateToken::Zeptosecond),
    ("zeptosegundo", "zs", DateToken::Zeptosecond),
    ("yoctosegundos", "ys", DateToken::Yoctosecond),
    ("yoctosegundo", "ys", DateToken::Yoctosecond),
    ("rontosegundos", "rs", DateToken::Rontosecond),
    ("rontosegundo", "rs", DateToken::Rontosecond),
    ("quectosegundos", "qs", DateToken::Quectosecond),
    ("quectosegundo", "qs", DateToken::Quectosecond),
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

pub(crate) const ES_MONTHS: &[(&str, &str, DateToken)] = &[
    ("ene", "Jan", DateToken::MonthShort),
    ("feb", "Feb", DateToken::MonthShort),
    ("mar", "Mar", DateToken::MonthShort),
    ("abr", "Apr", DateToken::MonthShort),
    ("may", "May", DateToken::MonthShort),
    ("jun", "Jun", DateToken::MonthShort),
    ("jul", "Jul", DateToken::MonthShort),
    ("ago", "Aug", DateToken::MonthShort),
    ("sep", "Sep", DateToken::MonthShort),
    ("oct", "Oct", DateToken::MonthShort),
    ("nov", "Nov", DateToken::MonthShort),
    ("dic", "Dec", DateToken::MonthShort),
    ("enero", "January", DateToken::MonthLong),
    ("febrero", "February", DateToken::MonthLong),
    ("marzo", "March", DateToken::MonthLong),
    ("abril", "April", DateToken::MonthLong),
    ("mayo", "May", DateToken::MonthLong),
    ("junio", "June", DateToken::MonthLong),
    ("julio", "July", DateToken::MonthLong),
    ("agosto", "August", DateToken::MonthLong),
    ("septiembre", "September", DateToken::MonthLong),
    ("octubre", "October", DateToken::MonthLong),
    ("noviembre", "November", DateToken::MonthLong),
    ("diciembre", "December", DateToken::MonthLong),
];

pub(crate) const ES_DAYS: &[(&str, &str, DateToken)] = &[
    ("lun", "Mon", DateToken::DayShort),
    ("mar", "Tue", DateToken::DayShort),
    ("mié", "Wed", DateToken::DayShort),
    ("jue", "Thu", DateToken::DayShort),
    ("vie", "Fri", DateToken::DayShort),
    ("sáb", "Sat", DateToken::DayShort),
    ("dom", "Sun", DateToken::DayShort),
    ("lunes", "Monday", DateToken::DayLong),
    ("martes", "Tuesday", DateToken::DayLong),
    ("miércoles", "Wednesday", DateToken::DayLong),
    ("jueves", "Thursday", DateToken::DayLong),
    ("viernes", "Friday", DateToken::DayLong),
    ("sábado", "Saturday", DateToken::DayLong),
    ("domingo", "Sunday", DateToken::DayLong),
];

pub(crate) const ES_SPECIAL: &[(&str, &str, DateToken)] =
    &[("am", "AM", DateToken::Am), ("pm", "PM", DateToken::Pm)];

static ES_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn es_date_ac() -> &'static AhoCorasick {
    ES_DATE_AC.get_or_init(|| {
        let mut terms: Vec<&'static str> = Vec::with_capacity(
            ES_RELATIVES.len()
                + ES_MONTHS.len()
                + ES_DAYS.len()
                + ES_SPECIAL.len()
                + tz_lowered_keys().len(),
        );
        // English base terms (Spanish reuses many)
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_MONTHS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DAYS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_SPECIAL.iter().map(|&(k, _, _)| k));
        // Spanish-specific terms
        terms.extend(ES_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(ES_MONTHS.iter().map(|&(k, _, _)| k));
        terms.extend(ES_DAYS.iter().map(|&(k, _, _)| k));
        terms.extend(ES_SPECIAL.iter().map(|&(k, _, _)| k));
        terms.extend(tz_lowered_keys());
        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for ES date terms");
        Box::new(ac)
    })
}

static ES_DURATION_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn es_duration_ac() -> &'static AhoCorasick {
    ES_DURATION_AC.get_or_init(|| {
        let mut terms: Vec<&'static str> =
            Vec::with_capacity(ES_RELATIVES.len() + ES_DURATIONS.len());
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DURATIONS.iter().map(|&(k, _, _)| k));
        terms.extend(ES_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(ES_DURATIONS.iter().map(|&(k, _, _)| k));
        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for ES duration terms");
        Box::new(ac)
    })
}

pub(crate) static ES: OnceBox<HashMap<&'static str, (&'static str, DateToken)>> = OnceBox::new();
pub(crate) fn es() -> &'static HashMap<&'static str, (&'static str, DateToken)> {
    ES.get_or_init(|| {
        let mut m = HashMap::new();
        // English base
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
        // Spanish-specific
        for &(k, v, token) in ES_RELATIVES {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in ES_DURATIONS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in ES_MONTHS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in ES_DAYS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in ES_SPECIAL {
            m.insert(k, (v, token));
        }
        for (&lowered_key, &(original_name, _, _)) in
            tz_lowered_keys().iter().zip(TZ_ENTRIES.iter())
        {
            m.insert(lowered_key, (original_name, DateToken::Iana));
        }
        Box::new(m)
    })
}

static ES_LANG_DATA: OnceBox<LangData> = OnceBox::new();
pub(crate) fn es_lang_data() -> &'static LangData {
    ES_LANG_DATA.get_or_init(|| {
        Box::new(LangData {
            map: es(),
            date_ac: es_date_ac(),
            duration_ac: es_duration_ac(),
        })
    })
}
