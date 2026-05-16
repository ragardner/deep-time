use crate::{
    DateToken, EN_DAYS, EN_DURATIONS, EN_MONTHS, EN_RELATIVES, EN_SPECIAL, LangData,
    tz_lowered_keys, tzdb::TZ_ENTRIES,
};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashMap;
use once_cell::race::OnceBox;

pub(crate) const DE_RELATIVES: &[(&'static str, &'static str, DateToken)] = &[
    ("und", "and", DateToken::Plus),
    ("plus", "plus", DateToken::Plus),
    // Temporal
    ("in", "in", DateToken::Future),
    ("nach", "after", DateToken::Future),
    ("nächste", "next", DateToken::Future),
    ("nächster", "next", DateToken::Future),
    ("nächsten", "next", DateToken::Future),
    ("nächstes", "next", DateToken::Future),
    ("vor", "ago", DateToken::Past),
    ("letzte", "last", DateToken::Past),
    ("letzter", "last", DateToken::Past),
    ("letzten", "last", DateToken::Past),
    ("letztes", "last", DateToken::Past),
    ("jetzt", "now", DateToken::Now),
    ("heute", "today", DateToken::Today),
    ("morgen", "tomorrow", DateToken::Tomorrow),
    ("gestern", "yesterday", DateToken::Yesterday),
    ("übermorgen", "tomorrow", DateToken::Tomorrow),
    ("vorgestern", "yesterday", DateToken::Yesterday),
    // Sub-second units
    ("nanosekunden", "ns", DateToken::Nanosecond),
    ("nanosekunde", "ns", DateToken::Nanosecond),
    ("nanos", "ns", DateToken::Nanosecond),
    ("mikrosekunden", "us", DateToken::Microsecond),
    ("mikrosekunde", "us", DateToken::Microsecond),
    ("mikros", "us", DateToken::Microsecond),
    ("millisekunden", "ms", DateToken::Millisecond),
    ("millisekunde", "ms", DateToken::Millisecond),
    ("millis", "ms", DateToken::Millisecond),
    // Seconds
    ("sekunden", "s", DateToken::Second),
    ("sekunde", "s", DateToken::Second),
    ("sek", "s", DateToken::Second),
    ("secs", "s", DateToken::Second),
    // Minutes
    ("minuten", "m", DateToken::Minute),
    ("minute", "m", DateToken::Minute),
    ("mins", "m", DateToken::Minute),
    ("min", "m", DateToken::Minute),
    // Hours
    ("stunden", "h", DateToken::Hour),
    ("stunde", "h", DateToken::Hour),
    ("std", "h", DateToken::Hour),
    ("hrs", "h", DateToken::Hour),
    // Days
    ("tage", "d", DateToken::Day),
    ("tag", "d", DateToken::Day),
    // Weeks
    ("wochen", "w", DateToken::Week),
    ("woche", "w", DateToken::Week),
    // Months
    ("monate", "mo", DateToken::Month),
    ("monat", "mo", DateToken::Month),
    // Years
    ("jahre", "y", DateToken::Year),
    ("jahr", "y", DateToken::Year),
    ("j", "y", DateToken::Year),
];

pub(crate) const DE_DURATIONS: &[(&'static str, &'static str, DateToken)] = &[
    ("y", "y", DateToken::Year),
    ("w", "w", DateToken::Week),
    ("d", "d", DateToken::Day),
    ("h", "h", DateToken::Hour),
    ("m", "m", DateToken::Minute),
    ("s", "s", DateToken::Second),
    ("jahrtausend", "mil", DateToken::Millennium),
    ("jahrtausende", "mil", DateToken::Millennium),
    ("jahrhundert", "cen", DateToken::Century),
    ("jahrhunderte", "cen", DateToken::Century),
    ("jahrzehnt", "dec", DateToken::Decade),
    ("jahrzehnte", "dec", DateToken::Decade),
    ("quartal", "qr", DateToken::Quarter),
    ("quartale", "qr", DateToken::Quarter),
    ("vierzehntage", "fn", DateToken::Fortnight),
    ("zweiwochen", "fn", DateToken::Fortnight),
    ("kilosekunden", "ks", DateToken::Kilosecond),
    ("kilosekunde", "ks", DateToken::Kilosecond),
    ("megasekunden", "mgs", DateToken::Megasecond),
    ("megasekunde", "mgs", DateToken::Megasecond),
    ("gigasekunden", "gs", DateToken::Gigasecond),
    ("gigasekunde", "gs", DateToken::Gigasecond),
    ("terasekunden", "ts", DateToken::Terasecond),
    ("terasekunde", "ts", DateToken::Terasecond),
    ("petasekunden", "pes", DateToken::Petasecond),
    ("petasekunde", "pes", DateToken::Petasecond),
    ("exasekunden", "es", DateToken::Exasecond),
    ("exasekunde", "es", DateToken::Exasecond),
    ("zettasekunden", "zes", DateToken::Zettasecond),
    ("zettasekunde", "zes", DateToken::Zettasecond),
    ("yottasekunden", "yos", DateToken::Yottasecond),
    ("yottasekunde", "yos", DateToken::Yottasecond),
    ("ronnasekunden", "ros", DateToken::Ronnasecond),
    ("ronnasekunde", "ros", DateToken::Ronnasecond),
    ("quettasekunden", "qus", DateToken::Quettasecond),
    ("quettasekunde", "qus", DateToken::Quettasecond),
    ("pikosekunden", "ps", DateToken::Picosecond),
    ("pikosekunde", "ps", DateToken::Picosecond),
    ("femtosekunden", "fs", DateToken::Femtosecond),
    ("femtosekunde", "fs", DateToken::Femtosecond),
    ("attosekunden", "as", DateToken::Attosecond),
    ("attosekunde", "as", DateToken::Attosecond),
    ("zeptosekunden", "zs", DateToken::Zeptosecond),
    ("zeptosekunde", "zs", DateToken::Zeptosecond),
    ("yoctosekunden", "ys", DateToken::Yoctosecond),
    ("yoctosekunde", "ys", DateToken::Yoctosecond),
    ("rontosekunden", "rs", DateToken::Rontosecond),
    ("rontosekunde", "rs", DateToken::Rontosecond),
    ("quectosekunden", "qs", DateToken::Quectosecond),
    ("quectosekunde", "qs", DateToken::Quectosecond),
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

pub(crate) const DE_MONTHS: &[(&'static str, &'static str, DateToken)] = &[
    ("jan", "Jan", DateToken::MonthShort),
    ("feb", "Feb", DateToken::MonthShort),
    ("mär", "Mar", DateToken::MonthShort),
    ("apr", "Apr", DateToken::MonthShort),
    ("mai", "May", DateToken::MonthShort),
    ("jun", "Jun", DateToken::MonthShort),
    ("jul", "Jul", DateToken::MonthShort),
    ("aug", "Aug", DateToken::MonthShort),
    ("sep", "Sep", DateToken::MonthShort),
    ("okt", "Oct", DateToken::MonthShort),
    ("nov", "Nov", DateToken::MonthShort),
    ("dez", "Dec", DateToken::MonthShort),
    ("januar", "January", DateToken::MonthLong),
    ("februar", "February", DateToken::MonthLong),
    ("märz", "March", DateToken::MonthLong),
    ("april", "April", DateToken::MonthLong),
    ("mai", "May", DateToken::MonthLong),
    ("juni", "June", DateToken::MonthLong),
    ("juli", "July", DateToken::MonthLong),
    ("august", "August", DateToken::MonthLong),
    ("september", "September", DateToken::MonthLong),
    ("oktober", "October", DateToken::MonthLong),
    ("november", "November", DateToken::MonthLong),
    ("dezember", "December", DateToken::MonthLong),
];

pub(crate) const DE_DAYS: &[(&'static str, &'static str, DateToken)] = &[
    ("mo", "Mon", DateToken::DayShort),
    ("di", "Tue", DateToken::DayShort),
    ("mi", "Wed", DateToken::DayShort),
    ("do", "Thu", DateToken::DayShort),
    ("fr", "Fri", DateToken::DayShort),
    ("sa", "Sat", DateToken::DayShort),
    ("so", "Sun", DateToken::DayShort),
    ("montag", "Monday", DateToken::DayLong),
    ("dienstag", "Tuesday", DateToken::DayLong),
    ("mittwoch", "Wednesday", DateToken::DayLong),
    ("donnerstag", "Thursday", DateToken::DayLong),
    ("freitag", "Friday", DateToken::DayLong),
    ("samstag", "Saturday", DateToken::DayLong),
    ("sonntag", "Sunday", DateToken::DayLong),
];

pub(crate) const DE_SPECIAL: &[(&'static str, &'static str, DateToken)] =
    &[("am", "AM", DateToken::Am), ("pm", "PM", DateToken::Pm)];

static DE_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn de_date_ac() -> &'static AhoCorasick {
    DE_DATE_AC.get_or_init(|| {
        let mut terms: Vec<&'static str> = Vec::with_capacity(
            DE_RELATIVES.len()
                + DE_MONTHS.len()
                + DE_DAYS.len()
                + DE_SPECIAL.len()
                + tz_lowered_keys().len(),
        );
        // English base terms (German reuses many)
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_MONTHS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DAYS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_SPECIAL.iter().map(|&(k, _, _)| k));
        // German-specific terms
        terms.extend(DE_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(DE_MONTHS.iter().map(|&(k, _, _)| k));
        terms.extend(DE_DAYS.iter().map(|&(k, _, _)| k));
        terms.extend(DE_SPECIAL.iter().map(|&(k, _, _)| k));
        terms.extend(tz_lowered_keys());
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
        let mut terms: Vec<&'static str> =
            Vec::with_capacity(DE_RELATIVES.len() + DE_DURATIONS.len());
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DURATIONS.iter().map(|&(k, _, _)| k));
        terms.extend(DE_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(DE_DURATIONS.iter().map(|&(k, _, _)| k));
        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for DE duration terms");
        Box::new(ac)
    })
}

pub(crate) static DE: OnceBox<HashMap<&'static str, (&'static str, DateToken)>> = OnceBox::new();
pub(crate) fn de() -> &'static HashMap<&'static str, (&'static str, DateToken)> {
    DE.get_or_init(|| {
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
        // German-specific
        for &(k, v, token) in DE_RELATIVES {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in DE_DURATIONS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in DE_MONTHS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in DE_DAYS {
            m.insert(k, (v, token));
        }
        for &(k, v, token) in DE_SPECIAL {
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

static DE_LANG_DATA: OnceBox<LangData> = OnceBox::new();
pub(crate) fn de_lang_data() -> &'static LangData {
    DE_LANG_DATA.get_or_init(|| {
        Box::new(LangData {
            map: de(),
            date_ac: de_date_ac(),
            duration_ac: de_duration_ac(),
        })
    })
}
