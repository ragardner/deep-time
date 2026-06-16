use crate::en::{EN_WORDS, tz_lowered_keys};
use crate::{Cat, LangData, Token, Word};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use once_cell::race::OnceBox;

pub const ES_WORDS: &[Word] = &[
    // MONTHS
    Word::new("ene", "Jan", Token::MonthShort, Cat::Month),
    Word::new("feb", "Feb", Token::MonthShort, Cat::Month),
    Word::new("mar", "Mar", Token::MonthShort, Cat::Month),
    Word::new("abr", "Apr", Token::MonthShort, Cat::Month),
    Word::new("may", "May", Token::MonthShort, Cat::Month),
    Word::new("jun", "Jun", Token::MonthShort, Cat::Month),
    Word::new("jul", "Jul", Token::MonthShort, Cat::Month),
    Word::new("ago", "Aug", Token::MonthShort, Cat::Month),
    Word::new("sep", "Sep", Token::MonthShort, Cat::Month),
    Word::new("oct", "Oct", Token::MonthShort, Cat::Month),
    Word::new("nov", "Nov", Token::MonthShort, Cat::Month),
    Word::new("dic", "Dec", Token::MonthShort, Cat::Month),
    Word::new("enero", "January", Token::MonthLong, Cat::Month),
    Word::new("febrero", "February", Token::MonthLong, Cat::Month),
    Word::new("marzo", "March", Token::MonthLong, Cat::Month),
    Word::new("abril", "April", Token::MonthLong, Cat::Month),
    Word::new("mayo", "May", Token::MonthLong, Cat::Month),
    Word::new("junio", "June", Token::MonthLong, Cat::Month),
    Word::new("julio", "July", Token::MonthLong, Cat::Month),
    Word::new("agosto", "August", Token::MonthLong, Cat::Month),
    Word::new("septiembre", "September", Token::MonthLong, Cat::Month),
    Word::new("octubre", "October", Token::MonthLong, Cat::Month),
    Word::new("noviembre", "November", Token::MonthLong, Cat::Month),
    Word::new("diciembre", "December", Token::MonthLong, Cat::Month),
    // DAYS
    Word::new("lun", "Mon", Token::DayShort, Cat::Day),
    // Word::new("mar", "Tue", Token::DayShort, Cat::Day), // collision with marzo
    Word::new("mié", "Wed", Token::DayShort, Cat::Month),
    Word::new("jue", "Thu", Token::DayShort, Cat::Day),
    Word::new("vie", "Fri", Token::DayShort, Cat::Day),
    Word::new("sáb", "Sat", Token::DayShort, Cat::Day),
    Word::new("dom", "Sun", Token::DayShort, Cat::Day),
    Word::new("lunes", "Monday", Token::DayLong, Cat::Day),
    Word::new("martes", "Tuesday", Token::DayLong, Cat::Day),
    Word::new("miércoles", "Wednesday", Token::DayLong, Cat::Day),
    Word::new("jueves", "Thursday", Token::DayLong, Cat::Day),
    Word::new("viernes", "Friday", Token::DayLong, Cat::Day),
    Word::new("sábado", "Saturday", Token::DayLong, Cat::Day),
    Word::new("domingo", "Sunday", Token::DayLong, Cat::Day),
    // am/pm
    Word::new("por la mañana", "AM", Token::Am, Cat::AmPm),
    Word::new("de la mañana", "AM", Token::Am, Cat::AmPm),
    Word::new("tarde", "PM", Token::Pm, Cat::AmPm),
    // RELATIVES
    Word::new("y", "and", Token::Plus, Cat::UnamRel), // clashes with y duration unit
    // Word::new("en", "in", Token::Future, Cat::UnamRel),
    // Word::new("dentro de", "in", Token::Future, Cat::UnamRel),
    Word::new("este", "this", Token::Present, Cat::UnamRel),
    Word::new("esta", "this", Token::Present, Cat::UnamRel),
    Word::new("viene", "next", Token::Future, Cat::UnamRel),
    Word::new("próximo", "next", Token::Future, Cat::UnamRel),
    Word::new("próxima", "next", Token::Future, Cat::UnamRel),
    Word::new("después", "after", Token::Future, Cat::UnamRel),
    Word::new("pasada", "last", Token::Past, Cat::UnamRel),
    Word::new("pasado", "last", Token::Past, Cat::UnamRel),
    Word::new("último", "last", Token::Past, Cat::UnamRel),
    Word::new("última", "last", Token::Past, Cat::UnamRel),
    Word::new("anterior", "last", Token::Past, Cat::UnamRel),
    Word::new("ahora", "now", Token::Now, Cat::UnamRel),
    Word::new("hoy", "today", Token::Today, Cat::UnamRel),
    Word::new("mañana", "tomorrow", Token::Tomorrow, Cat::UnamRel),
    Word::new("ayer", "yesterday", Token::Yesterday, Cat::UnamRel),
    Word::new("hace", "ago", Token::Ago, Cat::Ago),
    // Sub-second
    Word::new("nanosegundos", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanosegundo", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanos", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("microsegundos", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("microsegundo", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("micros", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("milisegundos", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("milisegundo", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millis", "ms", Token::Millisecond, Cat::UnamRel),
    // Seconds
    Word::new("segundos", "s", Token::Second, Cat::UnamRel),
    Word::new("segundo", "s", Token::Second, Cat::UnamRel),
    Word::new("segs", "s", Token::Second, Cat::UnamRel),
    Word::new("seg", "s", Token::Second, Cat::UnamRel),
    // Minutes
    Word::new("minutos", "m", Token::Minute, Cat::UnamRel),
    Word::new("minuto", "m", Token::Minute, Cat::UnamRel),
    Word::new("mins", "m", Token::Minute, Cat::UnamRel),
    Word::new("min", "m", Token::Minute, Cat::UnamRel),
    // Hours
    Word::new("horas", "h", Token::Hour, Cat::UnamRel),
    Word::new("hora", "h", Token::Hour, Cat::UnamRel),
    Word::new("hrs", "h", Token::Hour, Cat::UnamRel),
    Word::new("hr", "h", Token::Hour, Cat::UnamRel),
    // Days
    Word::new("días", "d", Token::Day, Cat::UnamRel),
    Word::new("día", "d", Token::Day, Cat::UnamRel),
    // Weeks
    Word::new("semanas", "w", Token::Week, Cat::UnamRel),
    Word::new("semana", "w", Token::Week, Cat::UnamRel),
    Word::new("sem", "w", Token::Week, Cat::UnamRel),
    // Months
    Word::new("meses", "mo", Token::Month, Cat::UnamRel),
    Word::new("mes", "mo", Token::Month, Cat::UnamRel),
    // Years
    Word::new("años", "y", Token::Year, Cat::UnamRel),
    Word::new("año", "y", Token::Year, Cat::UnamRel),
    Word::new("años", "y", Token::Year, Cat::UnamRel),
    Word::new("año", "y", Token::Year, Cat::UnamRel),
    // Large time units
    Word::new("milenio", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("milenios", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("siglo", "cen", Token::Century, Cat::UnamDur),
    Word::new("siglos", "cen", Token::Century, Cat::UnamDur),
    Word::new("década", "dec", Token::Decade, Cat::UnamDur),
    Word::new("décadas", "dec", Token::Decade, Cat::UnamDur),
    Word::new("trimestre", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("trimestres", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("quincena", "fn", Token::Fortnight, Cat::UnamDur),
    Word::new("quincenas", "fn", Token::Fortnight, Cat::UnamDur),
    // SI units
    Word::new("kilosegundos", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("kilosegundo", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("megasegundos", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("megasegundo", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("gigasegundos", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("gigasegundo", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("terasegundos", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("terasegundo", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("petasegundos", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("petasegundo", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("picosegundos", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("picosegundo", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("femtosegundos", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("femtosegundo", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("attosegundos", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("attosegundo", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("zeptosegundos", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("zeptosegundo", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("yoctosegundos", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("yoctosegundo", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("rontosegundos", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("rontosegundo", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("quectosegundos", "qs", Token::Quectosecond, Cat::UnamDur),
    Word::new("quectosegundo", "qs", Token::Quectosecond, Cat::UnamDur),
    // Short forms
    Word::new("mil", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("cen", "cen", Token::Century, Cat::UnamDur),
    Word::new("qr", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("fn", "fn", Token::Fortnight, Cat::UnamDur),
    Word::new("ks", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("mgs", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("gs", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("ts", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("pes", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("ms", "ms", Token::Millisecond, Cat::UnamDur),
    Word::new("us", "us", Token::Microsecond, Cat::UnamDur),
    Word::new("ns", "ns", Token::Nanosecond, Cat::UnamDur),
    Word::new("ps", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("fs", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("as", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("zs", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("ys", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("rs", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("qs", "qs", Token::Quectosecond, Cat::UnamDur),
];

static ES_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn es_date_ac() -> &'static AhoCorasick {
    ES_DATE_AC.get_or_init(|| {
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| {
                matches!(
                    w.c,
                    Cat::UnamRel | Cat::Month | Cat::Day | Cat::AmPm | Cat::TScl
                )
            })
            .map(|w| w.low)
            .chain(
                ES_WORDS
                    .iter()
                    .filter(|w| {
                        matches!(
                            w.c,
                            Cat::UnamRel | Cat::AmPm | Cat::Ago | Cat::Month | Cat::Day
                        )
                    })
                    .map(|w| w.low),
            )
            .chain(tz_lowered_keys().iter().copied())
            .filter(|&s| seen.insert(s))
            .collect();

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
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| matches!(w.c, Cat::UnamRel | Cat::AmDur | Cat::UnamDur))
            .map(|w| w.low)
            .chain(
                ES_WORDS
                    .iter()
                    .filter(|w| matches!(w.c, Cat::UnamRel | Cat::Ago | Cat::AmDur | Cat::UnamDur))
                    .map(|w| w.low),
            )
            .filter(|&s| seen.insert(s))
            .collect();

        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for ES duration terms");
        Box::new(ac)
    })
}

pub(crate) static ES: OnceBox<HashMap<&'static str, (&'static str, Token)>> = OnceBox::new();

pub(crate) fn es() -> &'static HashMap<&'static str, (&'static str, Token)> {
    ES.get_or_init(|| {
        let mut m = HashMap::new();

        for word in ES_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for word in EN_WORDS
            .iter()
            .filter(|w| !matches!(w.c, Cat::AmRel | Cat::AmDur | Cat::Ago))
        {
            m.insert(word.low, (word.norm, word.t));
        }

        for name in crate::tz::tz_names() {
            let s = name.as_str();

            let lowered = s.to_lowercase();
            let lowered_static = Box::leak(lowered.into_boxed_str()) as &'static str;

            let original_static =
                Box::leak(alloc::string::String::from(s).into_boxed_str()) as &'static str;

            m.insert(lowered_static, (original_static, Token::Iana));
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
            decimal_char: ',',
        })
    })
}
