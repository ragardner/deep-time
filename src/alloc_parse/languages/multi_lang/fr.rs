use crate::{Cat, EN_WORDS, LangData, Token, Word, tz::available_timezones, tz_lowered_keys};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use once_cell::race::OnceBox;

pub const FR_WORDS: &[Word] = &[
    // MONTHS
    Word::new("janv", "Jan", Token::MonthShort, Cat::Month),
    Word::new("févr", "Feb", Token::MonthShort, Cat::Month),
    Word::new("mars", "Mar", Token::MonthShort, Cat::Month),
    Word::new("avr", "Apr", Token::MonthShort, Cat::Month),
    Word::new("mai", "May", Token::MonthShort, Cat::Month),
    Word::new("juin", "Jun", Token::MonthShort, Cat::Month),
    Word::new("juil", "Jul", Token::MonthShort, Cat::Month),
    Word::new("août", "Aug", Token::MonthShort, Cat::Month),
    Word::new("sept", "Sep", Token::MonthShort, Cat::Month),
    Word::new("oct", "Oct", Token::MonthShort, Cat::Month),
    Word::new("nov", "Nov", Token::MonthShort, Cat::Month),
    Word::new("déc", "Dec", Token::MonthShort, Cat::Month),
    Word::new("janvier", "January", Token::MonthLong, Cat::Month),
    Word::new("février", "February", Token::MonthLong, Cat::Month),
    Word::new("mars", "March", Token::MonthLong, Cat::Month),
    Word::new("avril", "April", Token::MonthLong, Cat::Month),
    Word::new("mai", "May", Token::MonthLong, Cat::Month),
    Word::new("juin", "June", Token::MonthLong, Cat::Month),
    Word::new("juillet", "July", Token::MonthLong, Cat::Month),
    Word::new("août", "August", Token::MonthLong, Cat::Month),
    Word::new("septembre", "September", Token::MonthLong, Cat::Month),
    Word::new("octobre", "October", Token::MonthLong, Cat::Month),
    Word::new("novembre", "November", Token::MonthLong, Cat::Month),
    Word::new("décembre", "December", Token::MonthLong, Cat::Month),
    // DAYS
    Word::new("lun", "Mon", Token::DayShort, Cat::Day),
    // Word::new("mar", "Tue", Token::DayShort, Cat::Day), // won't parse
    Word::new("mer", "Wed", Token::DayShort, Cat::Day),
    Word::new("jeu", "Thu", Token::DayShort, Cat::Day),
    Word::new("ven", "Fri", Token::DayShort, Cat::Day),
    Word::new("sam", "Sat", Token::DayShort, Cat::Day),
    Word::new("dim", "Sun", Token::DayShort, Cat::Day),
    Word::new("lundi", "Monday", Token::DayLong, Cat::Day),
    Word::new("mardi", "Tuesday", Token::DayLong, Cat::Day),
    Word::new("mercredi", "Wednesday", Token::DayLong, Cat::Day),
    Word::new("jeudi", "Thursday", Token::DayLong, Cat::Day),
    Word::new("vendredi", "Friday", Token::DayLong, Cat::Day),
    Word::new("samedi", "Saturday", Token::DayLong, Cat::Day),
    Word::new("dimanche", "Sunday", Token::DayLong, Cat::Day),
    // RELATIVES
    Word::new("et", "and", Token::Plus, Cat::UnamRel),
    Word::new("plus", "plus", Token::Plus, Cat::UnamRel),
    Word::new("dans", "in", Token::Future, Cat::UnamRel),
    Word::new("en", "in", Token::Future, Cat::UnamRel),
    Word::new("prochain", "next", Token::Future, Cat::UnamRel),
    Word::new("prochaine", "next", Token::Future, Cat::UnamRel),
    Word::new("après", "after", Token::Future, Cat::UnamRel),
    Word::new("il y a", "ago", Token::Past, Cat::UnamRel),
    Word::new("passé", "last", Token::Past, Cat::UnamRel),
    Word::new("dernier", "last", Token::Past, Cat::UnamRel),
    Word::new("dernière", "last", Token::Past, Cat::UnamRel),
    Word::new("précédent", "last", Token::Past, Cat::UnamRel),
    Word::new("précédente", "last", Token::Past, Cat::UnamRel),
    Word::new("maintenant", "now", Token::Now, Cat::UnamRel),
    Word::new("aujourd'hui", "today", Token::Today, Cat::UnamRel),
    Word::new("demain", "tomorrow", Token::Tomorrow, Cat::UnamRel),
    Word::new("hier", "yesterday", Token::Yesterday, Cat::UnamRel),
    // Sub-second
    Word::new("nanosecondes", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanoseconde", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanos", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("microsecondes", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("microseconde", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("micros", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("millisecondes", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("milliseconde", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millis", "ms", Token::Millisecond, Cat::UnamRel),
    // Seconds
    Word::new("secondes", "s", Token::Second, Cat::UnamRel),
    Word::new("seconde", "s", Token::Second, Cat::UnamRel),
    Word::new("secs", "s", Token::Second, Cat::UnamRel),
    Word::new("sec", "s", Token::Second, Cat::UnamRel),
    // Minutes
    Word::new("minutes", "m", Token::Minute, Cat::UnamRel),
    Word::new("minute", "m", Token::Minute, Cat::UnamRel),
    Word::new("mins", "m", Token::Minute, Cat::UnamRel),
    Word::new("min", "m", Token::Minute, Cat::UnamRel),
    // Hours
    Word::new("heures", "h", Token::Hour, Cat::UnamRel),
    Word::new("heure", "h", Token::Hour, Cat::UnamRel),
    Word::new("hrs", "h", Token::Hour, Cat::UnamRel),
    Word::new("hr", "h", Token::Hour, Cat::UnamRel),
    // Days
    Word::new("jours", "d", Token::Day, Cat::UnamRel),
    Word::new("jour", "d", Token::Day, Cat::UnamRel),
    // Weeks
    Word::new("semaines", "w", Token::Week, Cat::UnamRel),
    Word::new("semaine", "w", Token::Week, Cat::UnamRel),
    Word::new("sem", "w", Token::Week, Cat::UnamRel),
    // Months
    Word::new("mois", "mo", Token::Month, Cat::UnamRel),
    // Years
    Word::new("années", "y", Token::Year, Cat::UnamRel),
    Word::new("année", "y", Token::Year, Cat::UnamRel),
    Word::new("ans", "y", Token::Year, Cat::UnamRel),
    Word::new("an", "y", Token::Year, Cat::UnamRel),
    // Large time units
    Word::new("millénaire", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("millénaires", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("siècle", "cen", Token::Century, Cat::UnamDur),
    Word::new("siècles", "cen", Token::Century, Cat::UnamDur),
    Word::new("décennie", "dec", Token::Decade, Cat::UnamDur),
    Word::new("décennies", "dec", Token::Decade, Cat::UnamDur),
    Word::new("trimestre", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("trimestres", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("quinzaine", "fn", Token::Fortnight, Cat::UnamDur),
    Word::new("quinzaines", "fn", Token::Fortnight, Cat::UnamDur),
    // SI units
    Word::new("kilosecondes", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("kiloseconde", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("mégasecondes", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("mégaseconde", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("gigasecondes", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("gigaseconde", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("térasecondes", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("téraseconde", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("pétasecondes", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("pétaseconde", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("picosecondes", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("picoseconde", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("femtosecondes", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("femtoseconde", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("attosecondes", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("attoseconde", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("zeptosecondes", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("zeptoseconde", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("yoctosecondes", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("yoctoseconde", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("rontosecondes", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("rontoseconde", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("quectosecondes", "qs", Token::Quectosecond, Cat::UnamDur),
    Word::new("quectoseconde", "qs", Token::Quectosecond, Cat::UnamDur),
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

static FR_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn fr_date_ac() -> &'static AhoCorasick {
    FR_DATE_AC.get_or_init(|| {
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| {
                matches!(
                    w.c,
                    Cat::UnamRel | Cat::AmRel | Cat::Month | Cat::Day | Cat::AmPm | Cat::TScl
                )
            })
            .map(|w| w.low)
            .chain(
                FR_WORDS
                    .iter()
                    .filter(|w| matches!(w.c, Cat::UnamRel | Cat::AmRel | Cat::Month | Cat::Day))
                    .map(|w| w.low),
            )
            .chain(tz_lowered_keys().iter().copied())
            .filter(|&s| seen.insert(s))
            .collect();

        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for FR date terms");
        Box::new(ac)
    })
}

static FR_DURATION_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn fr_duration_ac() -> &'static AhoCorasick {
    FR_DURATION_AC.get_or_init(|| {
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| matches!(w.c, Cat::UnamRel | Cat::AmRel | Cat::AmDur | Cat::UnamDur))
            .map(|w| w.low)
            .chain(
                FR_WORDS
                    .iter()
                    .filter(|w| {
                        matches!(w.c, Cat::UnamRel | Cat::AmRel | Cat::AmDur | Cat::UnamDur)
                    })
                    .map(|w| w.low),
            )
            .filter(|&s| seen.insert(s))
            .collect();

        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for FR duration terms");
        Box::new(ac)
    })
}

pub(crate) static FR: OnceBox<HashMap<&'static str, (&'static str, Token)>> = OnceBox::new();
pub(crate) fn fr() -> &'static HashMap<&'static str, (&'static str, Token)> {
    FR.get_or_init(|| {
        let mut m = HashMap::new();

        for word in FR_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for word in EN_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for (&lowered_key, original_name) in tz_lowered_keys().iter().zip(available_timezones()) {
            m.insert(lowered_key, (original_name, Token::Iana));
        }

        Box::new(m)
    })
}

static FR_LANG_DATA: OnceBox<LangData> = OnceBox::new();
pub(crate) fn fr_lang_data() -> &'static LangData {
    FR_LANG_DATA.get_or_init(|| {
        Box::new(LangData {
            map: fr(),
            date_ac: fr_date_ac(),
            duration_ac: fr_duration_ac(),
            decimal_char: ',',
        })
    })
}
