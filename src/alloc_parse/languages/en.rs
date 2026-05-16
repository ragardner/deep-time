use crate::{DateToken, LangData, tzdb::TZ_ENTRIES};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashMap;
use once_cell::race::OnceBox;

pub(crate) static TZ_LOWERED_KEYS: OnceBox<&'static [&'static str]> = OnceBox::new();
pub(crate) fn tz_lowered_keys() -> &'static [&'static str] {
    TZ_LOWERED_KEYS.get_or_init(|| {
        let keys: Vec<&'static str> = TZ_ENTRIES
            .iter()
            .map(|&(name, _, _)| Box::leak(name.to_lowercase().into_boxed_str()) as &'static str)
            .collect();

        let leaked: &'static [&'static str] = Box::leak(keys.into_boxed_slice());
        Box::new(leaked)
    })
}

// pub(crate) const CLOCK_TYPES: &[(&'static str, &'static str, DateToken)] = &[
//     ("tai", "TAI", DateToken::TAI),
//     ("tt", "TT", DateToken::TT),
//     ("et", "ET", DateToken::ET),
//     ("tdb", "TDB", DateToken::TDB),
//     // ("utc", "UTC", DateToken::UTC), // avoid overlap with iana
//     ("gps", "GPS", DateToken::GPS),
//     ("gst", "GST", DateToken::GST),
//     ("bdt", "BDT", DateToken::BDT),
//     ("qzss", "QZSS", DateToken::QZSS),
//     ("tcg", "TCG", DateToken::TCG),
//     ("tcb", "TCB", DateToken::TCB),
//     ("ltc", "LTC", DateToken::LTC),
//     ("custom", "Custom", DateToken::Custom),
// ];

pub(crate) const EN_RELATIVES: &[(&'static str, &'static str, DateToken)] = &[
    ("and", "and", DateToken::Plus),
    ("plus", "plus", DateToken::Plus),
    // Temporal
    ("in", "in", DateToken::Future),
    ("next", "next", DateToken::Future),
    ("after", "after", DateToken::Future),
    ("ago", "ago", DateToken::Past),
    ("last", "last", DateToken::Past),
    ("now", "now", DateToken::Now),
    ("today", "today", DateToken::Today),
    ("tomorrow", "tomorrow", DateToken::Tomorrow),
    ("yesterday", "yesterday", DateToken::Yesterday),
    // Sub-second units
    ("nanoseconds", "ns", DateToken::Nanosecond),
    ("nanosecond", "ns", DateToken::Nanosecond),
    ("nanos", "ns", DateToken::Nanosecond),
    ("microseconds", "us", DateToken::Microsecond),
    ("microsecond", "us", DateToken::Microsecond),
    ("micros", "us", DateToken::Microsecond),
    ("milliseconds", "ms", DateToken::Millisecond),
    ("millisecond", "ms", DateToken::Millisecond),
    ("millis", "ms", DateToken::Millisecond),
    // Seconds
    ("seconds", "s", DateToken::Second),
    ("second", "s", DateToken::Second),
    ("secs", "s", DateToken::Second),
    ("sec", "s", DateToken::Second),
    // Minutes
    ("minutes", "m", DateToken::Minute),
    ("minute", "m", DateToken::Minute),
    ("mins", "m", DateToken::Minute),
    ("min", "m", DateToken::Minute),
    // Hours
    ("hours", "h", DateToken::Hour),
    ("hour", "h", DateToken::Hour),
    ("hrs", "h", DateToken::Hour),
    ("hr", "h", DateToken::Hour),
    // Days
    ("days", "d", DateToken::Day),
    ("day", "d", DateToken::Day),
    // Weeks
    ("weeks", "w", DateToken::Week),
    ("week", "w", DateToken::Week),
    ("wks", "w", DateToken::Week),
    ("wk", "w", DateToken::Week),
    // Months
    ("months", "mo", DateToken::Month),
    ("month", "mo", DateToken::Month),
    ("mos", "mo", DateToken::Month),
    ("mo", "mo", DateToken::Month),
    // Years
    ("years", "y", DateToken::Year),
    ("year", "y", DateToken::Year),
    ("yrs", "y", DateToken::Year),
    ("yr", "y", DateToken::Year),
];

/// Any missing short and long units from RELATIVES
pub(crate) const EN_DURATIONS: &[(&'static str, &'static str, DateToken)] = &[
    ("y", "y", DateToken::Year),
    ("w", "w", DateToken::Week),
    ("d", "d", DateToken::Day),
    ("h", "h", DateToken::Hour),
    ("m", "m", DateToken::Minute),
    ("s", "s", DateToken::Second),
    ("millennia", "mil", DateToken::Millennium),
    ("millennium", "mil", DateToken::Millennium),
    ("centuries", "cen", DateToken::Century),
    ("century", "cen", DateToken::Century),
    ("decades", "dec", DateToken::Decade),
    ("decade", "dec", DateToken::Decade),
    ("quarters", "qr", DateToken::Quarter),
    ("quarter", "qr", DateToken::Quarter),
    ("fortnights", "fn", DateToken::Fortnight),
    ("fortnight", "fn", DateToken::Fortnight),
    ("kiloseconds", "ks", DateToken::Kilosecond),
    ("kilosecond", "ks", DateToken::Kilosecond),
    ("megaseconds", "mgs", DateToken::Megasecond),
    ("megasecond", "mgs", DateToken::Megasecond),
    ("gigaseconds", "gs", DateToken::Gigasecond),
    ("gigasecond", "gs", DateToken::Gigasecond),
    ("teraseconds", "ts", DateToken::Terasecond),
    ("terasecond", "ts", DateToken::Terasecond),
    ("petaseconds", "pes", DateToken::Petasecond),
    ("petasecond", "pes", DateToken::Petasecond),
    ("exaseconds", "es", DateToken::Exasecond),
    ("exasecond", "es", DateToken::Exasecond),
    ("zettaseconds", "zes", DateToken::Zettasecond),
    ("zettasecond", "zes", DateToken::Zettasecond),
    ("yottaseconds", "yos", DateToken::Yottasecond),
    ("yottasecond", "yos", DateToken::Yottasecond),
    ("ronnaseconds", "ros", DateToken::Ronnasecond),
    ("ronnasecond", "ros", DateToken::Ronnasecond),
    ("quettaseconds", "qus", DateToken::Quettasecond),
    ("quettasecond", "qus", DateToken::Quettasecond),
    ("picoseconds", "ps", DateToken::Picosecond),
    ("picosecond", "ps", DateToken::Picosecond),
    ("femtoseconds", "fs", DateToken::Femtosecond),
    ("femtosecond", "fs", DateToken::Femtosecond),
    ("attoseconds", "as", DateToken::Attosecond),
    ("attosecond", "as", DateToken::Attosecond),
    ("zeptoseconds", "zs", DateToken::Zeptosecond),
    ("zeptosecond", "zs", DateToken::Zeptosecond),
    ("yoctoseconds", "ys", DateToken::Yoctosecond),
    ("yoctosecond", "ys", DateToken::Yoctosecond),
    ("rontoseconds", "rs", DateToken::Rontosecond),
    ("rontosecond", "rs", DateToken::Rontosecond),
    ("quectoseconds", "qs", DateToken::Quectosecond),
    ("quectosecond", "qs", DateToken::Quectosecond),
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

pub(crate) const EN_MONTHS: &[(&'static str, &'static str, DateToken)] = &[
    // Short months
    ("jan", "Jan", DateToken::MonthShort),
    ("feb", "Feb", DateToken::MonthShort),
    ("mar", "Mar", DateToken::MonthShort),
    ("apr", "Apr", DateToken::MonthShort),
    ("may", "May", DateToken::MonthShort),
    ("jun", "Jun", DateToken::MonthShort),
    ("jul", "Jul", DateToken::MonthShort),
    ("aug", "Aug", DateToken::MonthShort),
    ("sep", "Sep", DateToken::MonthShort),
    ("oct", "Oct", DateToken::MonthShort),
    ("nov", "Nov", DateToken::MonthShort),
    ("dec", "Dec", DateToken::MonthShort),
    // Long months
    ("january", "January", DateToken::MonthLong),
    ("february", "February", DateToken::MonthLong),
    ("march", "March", DateToken::MonthLong),
    ("april", "April", DateToken::MonthLong),
    ("may", "May", DateToken::MonthLong),
    ("june", "June", DateToken::MonthLong),
    ("july", "July", DateToken::MonthLong),
    ("august", "August", DateToken::MonthLong),
    ("september", "September", DateToken::MonthLong),
    ("october", "October", DateToken::MonthLong),
    ("november", "November", DateToken::MonthLong),
    ("december", "December", DateToken::MonthLong),
];

pub(crate) const EN_DAYS: &[(&'static str, &'static str, DateToken)] = &[
    // Short days
    ("mon", "Mon", DateToken::DayShort),
    ("tue", "Tue", DateToken::DayShort),
    ("wed", "Wed", DateToken::DayShort),
    ("thu", "Thu", DateToken::DayShort),
    ("fri", "Fri", DateToken::DayShort),
    ("sat", "Sat", DateToken::DayShort),
    ("sun", "Sun", DateToken::DayShort),
    // Long days
    ("monday", "Monday", DateToken::DayLong),
    ("tuesday", "Tuesday", DateToken::DayLong),
    ("wednesday", "Wednesday", DateToken::DayLong),
    ("thursday", "Thursday", DateToken::DayLong),
    ("friday", "Friday", DateToken::DayLong),
    ("saturday", "Saturday", DateToken::DayLong),
    ("sunday", "Sunday", DateToken::DayLong),
];

pub(crate) const EN_SPECIAL: &[(&'static str, &'static str, DateToken)] =
    &[("am", "AM", DateToken::Am), ("pm", "PM", DateToken::Pm)];

static EN_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn en_date_ac() -> &'static AhoCorasick {
    EN_DATE_AC.get_or_init(|| {
        let mut terms: Vec<&'static str> = Vec::with_capacity(
            EN_RELATIVES.len()
                + EN_MONTHS.len()
                + EN_DAYS.len()
                + EN_SPECIAL.len()
                // + CLOCK_TYPES.len()
                + tz_lowered_keys().len(),
        );
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_MONTHS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DAYS.iter().map(|&(k, _, _)| k));
        terms.extend(EN_SPECIAL.iter().map(|&(k, _, _)| k));
        // terms.extend(CLOCK_TYPES.iter().map(|&(k, _, _)| k));
        terms.extend(tz_lowered_keys());
        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for EN date terms");
        Box::new(ac)
    })
}

static EN_DURATION_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn en_duration_ac() -> &'static AhoCorasick {
    EN_DURATION_AC.get_or_init(|| {
        let mut terms: Vec<&'static str> =
            Vec::with_capacity(EN_RELATIVES.len() + EN_DURATIONS.len());
        terms.extend(EN_RELATIVES.iter().map(|&(k, _, _)| k));
        terms.extend(EN_DURATIONS.iter().map(|&(k, _, _)| k));
        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for EN duration terms");
        Box::new(ac)
    })
}

pub(crate) static EN: OnceBox<HashMap<&'static str, (&'static str, DateToken)>> = OnceBox::new();
pub(crate) fn en() -> &'static HashMap<&'static str, (&'static str, DateToken)> {
    EN.get_or_init(|| {
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
        // for &(k, v, token) in CLOCK_TYPES {
        //     m.insert(k, (v, token));
        // }
        for (&lowered_key, &(original_name, _, _)) in
            tz_lowered_keys().iter().zip(TZ_ENTRIES.iter())
        {
            m.insert(lowered_key, (original_name, DateToken::Iana));
        }
        Box::new(m)
    })
}

static EN_LANG_DATA: OnceBox<LangData> = OnceBox::new();
pub(crate) fn en_lang_data() -> &'static LangData {
    EN_LANG_DATA.get_or_init(|| {
        Box::new(LangData {
            map: en(),
            date_ac: en_date_ac(),
            duration_ac: en_duration_ac(),
        })
    })
}
