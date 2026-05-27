use crate::{Cat, EN_WORDS, LangData, Token, Word, tz_lowered_keys, tzdb::TZ_ENTRIES};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use once_cell::race::OnceBox;

pub const DE_WORDS: &[Word] = &[
    // MONTHS
    Word::new("jan", "Jan", Token::MonthShort, Cat::Month),
    Word::new("feb", "Feb", Token::MonthShort, Cat::Month),
    Word::new("mär", "Mar", Token::MonthShort, Cat::Month),
    Word::new("apr", "Apr", Token::MonthShort, Cat::Month),
    Word::new("mai", "May", Token::MonthShort, Cat::Month),
    Word::new("jun", "Jun", Token::MonthShort, Cat::Month),
    Word::new("jul", "Jul", Token::MonthShort, Cat::Month),
    Word::new("aug", "Aug", Token::MonthShort, Cat::Month),
    Word::new("sep", "Sep", Token::MonthShort, Cat::Month),
    Word::new("okt", "Oct", Token::MonthShort, Cat::Month),
    Word::new("nov", "Nov", Token::MonthShort, Cat::Month),
    Word::new("dez", "Dec", Token::MonthShort, Cat::Month),
    Word::new("januar", "January", Token::MonthLong, Cat::Month),
    Word::new("februar", "February", Token::MonthLong, Cat::Month),
    Word::new("märz", "March", Token::MonthLong, Cat::Month),
    Word::new("april", "April", Token::MonthLong, Cat::Month),
    Word::new("mai", "May", Token::MonthLong, Cat::Month),
    Word::new("juni", "June", Token::MonthLong, Cat::Month),
    Word::new("juli", "July", Token::MonthLong, Cat::Month),
    Word::new("august", "August", Token::MonthLong, Cat::Month),
    Word::new("september", "September", Token::MonthLong, Cat::Month),
    Word::new("oktober", "October", Token::MonthLong, Cat::Month),
    Word::new("november", "November", Token::MonthLong, Cat::Month),
    Word::new("dezember", "December", Token::MonthLong, Cat::Month),
    // DAYS
    Word::new("mo", "Mon", Token::DayShort, Cat::Day),
    Word::new("di", "Tue", Token::DayShort, Cat::Day),
    Word::new("mi", "Wed", Token::DayShort, Cat::Day),
    Word::new("do", "Thu", Token::DayShort, Cat::Day),
    Word::new("fr", "Fri", Token::DayShort, Cat::Day),
    Word::new("sa", "Sat", Token::DayShort, Cat::Day),
    Word::new("so", "Sun", Token::DayShort, Cat::Day),
    Word::new("montag", "Monday", Token::DayLong, Cat::Day),
    Word::new("dienstag", "Tuesday", Token::DayLong, Cat::Day),
    Word::new("mittwoch", "Wednesday", Token::DayLong, Cat::Day),
    Word::new("donnerstag", "Thursday", Token::DayLong, Cat::Day),
    Word::new("freitag", "Friday", Token::DayLong, Cat::Day),
    Word::new("samstag", "Saturday", Token::DayLong, Cat::Day),
    Word::new("sonntag", "Sunday", Token::DayLong, Cat::Day),
    // RELATIVES
    Word::new("und", "and", Token::Plus, Cat::UnamRel),
    Word::new("plus", "plus", Token::Plus, Cat::UnamRel),
    Word::new("in", "in", Token::Future, Cat::UnamRel),
    Word::new("nächster", "next", Token::Future, Cat::UnamRel),
    Word::new("nächste", "next", Token::Future, Cat::UnamRel),
    Word::new("nächstes", "next", Token::Future, Cat::UnamRel),
    Word::new("nach", "after", Token::Future, Cat::UnamRel),
    Word::new("vor", "ago", Token::Past, Cat::UnamRel),
    Word::new("letzter", "last", Token::Past, Cat::UnamRel),
    Word::new("letzte", "last", Token::Past, Cat::UnamRel),
    Word::new("letztes", "last", Token::Past, Cat::UnamRel),
    Word::new("vorheriger", "last", Token::Past, Cat::UnamRel),
    Word::new("vorherige", "last", Token::Past, Cat::UnamRel),
    Word::new("jetzt", "now", Token::Now, Cat::UnamRel),
    Word::new("heute", "today", Token::Today, Cat::UnamRel),
    Word::new("morgen", "tomorrow", Token::Tomorrow, Cat::UnamRel),
    Word::new("gestern", "yesterday", Token::Yesterday, Cat::UnamRel),
    // Sub-second
    Word::new("nanosekunden", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanosekunde", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanos", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("mikrosekunden", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("mikrosekunde", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("mikros", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("millisekunden", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millisekunde", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millis", "ms", Token::Millisecond, Cat::UnamRel),
    // Seconds
    Word::new("sekunden", "s", Token::Second, Cat::UnamRel),
    Word::new("sekunde", "s", Token::Second, Cat::UnamRel),
    Word::new("sek", "s", Token::Second, Cat::UnamRel),
    Word::new("secs", "s", Token::Second, Cat::UnamRel),
    Word::new("sec", "s", Token::Second, Cat::UnamRel),
    // Minutes
    Word::new("minuten", "m", Token::Minute, Cat::UnamRel),
    Word::new("minute", "m", Token::Minute, Cat::UnamRel),
    Word::new("mins", "m", Token::Minute, Cat::UnamRel),
    Word::new("min", "m", Token::Minute, Cat::UnamRel),
    // Hours
    Word::new("stunden", "h", Token::Hour, Cat::UnamRel),
    Word::new("stunde", "h", Token::Hour, Cat::UnamRel),
    Word::new("std", "h", Token::Hour, Cat::UnamRel),
    Word::new("hrs", "h", Token::Hour, Cat::UnamRel),
    Word::new("hr", "h", Token::Hour, Cat::UnamRel),
    // Days
    Word::new("tage", "d", Token::Day, Cat::UnamRel),
    Word::new("tag", "d", Token::Day, Cat::UnamRel),
    // Word::new("t", "d", Token::Day, Cat::UnamRel),
    // Weeks
    Word::new("wochen", "w", Token::Week, Cat::UnamRel),
    Word::new("woche", "w", Token::Week, Cat::UnamRel),
    // Months
    Word::new("monate", "mo", Token::Month, Cat::UnamRel),
    Word::new("monat", "mo", Token::Month, Cat::UnamRel),
    // Years
    Word::new("jahre", "y", Token::Year, Cat::UnamRel),
    Word::new("jahr", "y", Token::Year, Cat::UnamRel),
    Word::new("yrs", "y", Token::Year, Cat::UnamRel),
    Word::new("yr", "y", Token::Year, Cat::UnamRel),
    // Large time units
    Word::new("jahrtausend", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("jahrtausende", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("jahrhundert", "cen", Token::Century, Cat::UnamDur),
    Word::new("jahrhunderte", "cen", Token::Century, Cat::UnamDur),
    Word::new("jahrzehnt", "dec", Token::Decade, Cat::UnamDur),
    Word::new("jahrzehnte", "dec", Token::Decade, Cat::UnamDur),
    Word::new("quartal", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("quartale", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("zwei wochen", "fn", Token::Fortnight, Cat::UnamDur),
    // SI units
    Word::new("kilosekunden", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("kilosekunde", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("megasekunden", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("megasekunde", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("gigasekunden", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("gigasekunde", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("terasekunden", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("terasekunde", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("petasekunden", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("petasekunde", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("pikosekunden", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("pikosekunde", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("femtosekunden", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("femtosekunde", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("attosekunden", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("attosekunde", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("zeptosekunden", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("zeptosekunde", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("yoctosekunden", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("yoctosekunde", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("rontosekunden", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("rontosekunde", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("quectosekunden", "qs", Token::Quectosecond, Cat::UnamDur),
    Word::new("quectosekunde", "qs", Token::Quectosecond, Cat::UnamDur),
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

static DE_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn de_date_ac() -> &'static AhoCorasick {
    DE_DATE_AC.get_or_init(|| {
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
                DE_WORDS
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
            .expect("invalid Aho-Corasick patterns for DE date terms");
        Box::new(ac)
    })
}

static DE_DURATION_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn de_duration_ac() -> &'static AhoCorasick {
    DE_DURATION_AC.get_or_init(|| {
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| matches!(w.c, Cat::UnamRel | Cat::AmRel | Cat::AmDur | Cat::UnamDur))
            .map(|w| w.low)
            .chain(
                DE_WORDS
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
            .expect("invalid Aho-Corasick patterns for DE duration terms");
        Box::new(ac)
    })
}

pub(crate) static DE: OnceBox<HashMap<&'static str, (&'static str, Token)>> = OnceBox::new();
pub(crate) fn de() -> &'static HashMap<&'static str, (&'static str, Token)> {
    DE.get_or_init(|| {
        let mut m = HashMap::new();

        for word in DE_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for word in EN_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for (&lowered_key, &(original_name, _, _)) in
            tz_lowered_keys().iter().zip(TZ_ENTRIES.iter())
        {
            m.insert(lowered_key, (original_name, Token::Iana));
        }

        Box::new(m)
    })
}

static DE_LANG_DATA: OnceBox<LangData> = OnceBox::new();
pub(crate) fn de_lang_data() -> &'static LangData {
    DE_LANG_DATA.get_or_init(|| {
        Box::new(LangData {
            map: de(),
            date_ac: de_date_ac(),
            duration_ac: de_duration_ac(),
            decimal_char: ',',
        })
    })
}
