use crate::{Cat, LangData, Token, Word};
use aho_corasick::{AhoCorasick, MatchKind};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::{HashMap, HashSet};
use once_cell::race::OnceBox;

pub(crate) static TZ_LOWERED_KEYS: OnceBox<&'static [&'static str]> = OnceBox::new();

pub(crate) fn tz_lowered_keys() -> &'static [&'static str] {
    TZ_LOWERED_KEYS.get_or_init(|| {
        let mut keys = Vec::new();

        for name in crate::tz::tz_names() {
            let lowered = name.as_str().unwrap().to_lowercase();
            keys.push(Box::leak(lowered.into_boxed_str()) as &'static str);
        }

        let leaked: &'static [&'static str] = Box::leak(keys.into_boxed_slice());
        Box::new(leaked)
    })
}

pub const EN_WORDS: &[Word] = &[
    // Short months
    Word::new("jan", "Jan", Token::MonthShort, Cat::Month),
    Word::new("feb", "Feb", Token::MonthShort, Cat::Month),
    Word::new("mar", "Mar", Token::MonthShort, Cat::Month),
    Word::new("apr", "Apr", Token::MonthShort, Cat::Month),
    Word::new("may", "May", Token::MonthShort, Cat::Month),
    Word::new("jun", "Jun", Token::MonthShort, Cat::Month),
    Word::new("jul", "Jul", Token::MonthShort, Cat::Month),
    Word::new("aug", "Aug", Token::MonthShort, Cat::Month),
    Word::new("sep", "Sep", Token::MonthShort, Cat::Month),
    Word::new("oct", "Oct", Token::MonthShort, Cat::Month),
    Word::new("nov", "Nov", Token::MonthShort, Cat::Month),
    Word::new("dec", "Dec", Token::MonthShort, Cat::Month),
    // Long months
    Word::new("january", "January", Token::MonthLong, Cat::Month),
    Word::new("february", "February", Token::MonthLong, Cat::Month),
    Word::new("march", "March", Token::MonthLong, Cat::Month),
    Word::new("april", "April", Token::MonthLong, Cat::Month),
    Word::new("may", "May", Token::MonthLong, Cat::Month),
    Word::new("june", "June", Token::MonthLong, Cat::Month),
    Word::new("july", "July", Token::MonthLong, Cat::Month),
    Word::new("august", "August", Token::MonthLong, Cat::Month),
    Word::new("september", "September", Token::MonthLong, Cat::Month),
    Word::new("october", "October", Token::MonthLong, Cat::Month),
    Word::new("november", "November", Token::MonthLong, Cat::Month),
    Word::new("december", "December", Token::MonthLong, Cat::Month),
    // Short days
    Word::new("mon", "Mon", Token::DayShort, Cat::Day),
    Word::new("tue", "Tue", Token::DayShort, Cat::Day),
    Word::new("wed", "Wed", Token::DayShort, Cat::Day),
    Word::new("thu", "Thu", Token::DayShort, Cat::Day),
    Word::new("fri", "Fri", Token::DayShort, Cat::Day),
    Word::new("sat", "Sat", Token::DayShort, Cat::Day),
    Word::new("sun", "Sun", Token::DayShort, Cat::Day),
    // Long days
    Word::new("monday", "Monday", Token::DayLong, Cat::Day),
    Word::new("tuesday", "Tuesday", Token::DayLong, Cat::Day),
    Word::new("wednesday", "Wednesday", Token::DayLong, Cat::Day),
    Word::new("thursday", "Thursday", Token::DayLong, Cat::Day),
    Word::new("friday", "Friday", Token::DayLong, Cat::Day),
    Word::new("saturday", "Saturday", Token::DayLong, Cat::Day),
    Word::new("sunday", "Sunday", Token::DayLong, Cat::Day),
    // am/pm
    Word::new("am", "AM", Token::Am, Cat::AmPm),
    Word::new("pm", "PM", Token::Pm, Cat::AmPm),
    // Connectors
    Word::new("and", "and", Token::Plus, Cat::UnamRel),
    Word::new("plus", "plus", Token::Plus, Cat::UnamRel),
    // Temporal
    Word::new("in", "in", Token::Future, Cat::UnamRel),
    Word::new("next", "next", Token::Future, Cat::UnamRel),
    Word::new("after", "after", Token::Future, Cat::UnamRel),
    Word::new("ago", "ago", Token::Past, Cat::AmRel),
    Word::new("last", "last", Token::Past, Cat::UnamRel),
    Word::new("now", "now", Token::Now, Cat::UnamRel),
    Word::new("today", "today", Token::Today, Cat::UnamRel),
    Word::new("tomorrow", "tomorrow", Token::Tomorrow, Cat::UnamRel),
    Word::new("yesterday", "yesterday", Token::Yesterday, Cat::UnamRel),
    // Seconds
    Word::new("seconds", "s", Token::Second, Cat::UnamRel),
    Word::new("second", "s", Token::Second, Cat::UnamRel),
    Word::new("secs", "s", Token::Second, Cat::UnamRel),
    Word::new("sec", "s", Token::Second, Cat::UnamRel),
    // Minutes
    Word::new("minutes", "m", Token::Minute, Cat::UnamRel),
    Word::new("minute", "m", Token::Minute, Cat::UnamRel),
    Word::new("mins", "m", Token::Minute, Cat::UnamRel),
    Word::new("min", "m", Token::Minute, Cat::UnamRel),
    // Hours
    Word::new("hours", "h", Token::Hour, Cat::UnamRel),
    Word::new("hour", "h", Token::Hour, Cat::UnamRel),
    Word::new("hrs", "h", Token::Hour, Cat::UnamRel),
    Word::new("hr", "h", Token::Hour, Cat::UnamRel),
    // Days
    Word::new("days", "d", Token::Day, Cat::UnamRel),
    Word::new("day", "d", Token::Day, Cat::UnamRel),
    // Weeks
    Word::new("weeks", "w", Token::Week, Cat::UnamRel),
    Word::new("week", "w", Token::Week, Cat::UnamRel),
    Word::new("wks", "w", Token::Week, Cat::UnamRel),
    Word::new("wk", "w", Token::Week, Cat::UnamRel),
    // Months
    Word::new("months", "mo", Token::Month, Cat::UnamRel),
    Word::new("month", "mo", Token::Month, Cat::UnamRel),
    Word::new("mos", "mo", Token::Month, Cat::UnamRel),
    Word::new("mo", "mo", Token::Month, Cat::UnamRel),
    // Years
    Word::new("years", "y", Token::Year, Cat::UnamRel),
    Word::new("year", "y", Token::Year, Cat::UnamRel),
    Word::new("yrs", "y", Token::Year, Cat::UnamRel),
    Word::new("yr", "y", Token::Year, Cat::UnamRel),
    // Sub-second units
    Word::new("nanoseconds", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanosecond", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("nanos", "ns", Token::Nanosecond, Cat::UnamRel),
    Word::new("microseconds", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("microsecond", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("micros", "us", Token::Microsecond, Cat::UnamRel),
    Word::new("milliseconds", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millisecond", "ms", Token::Millisecond, Cat::UnamRel),
    Word::new("millis", "ms", Token::Millisecond, Cat::UnamRel),
    // Short forms
    Word::new("y", "y", Token::Year, Cat::AmDur),
    Word::new("w", "w", Token::Week, Cat::UnamDur),
    Word::new("d", "d", Token::Day, Cat::UnamDur),
    Word::new("h", "h", Token::Hour, Cat::UnamDur),
    Word::new("m", "m", Token::Minute, Cat::UnamDur),
    Word::new("s", "s", Token::Second, Cat::UnamDur),
    // SI units (large)
    Word::new("kiloseconds", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("kilosecond", "ks", Token::Kilosecond, Cat::UnamDur),
    Word::new("megaseconds", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("megasecond", "mgs", Token::Megasecond, Cat::UnamDur),
    Word::new("gigaseconds", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("gigasecond", "gs", Token::Gigasecond, Cat::UnamDur),
    Word::new("teraseconds", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("terasecond", "ts", Token::Terasecond, Cat::UnamDur),
    Word::new("petaseconds", "pes", Token::Petasecond, Cat::UnamDur),
    Word::new("petasecond", "pes", Token::Petasecond, Cat::UnamDur),
    // SI units (small)
    Word::new("picoseconds", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("picosecond", "ps", Token::Picosecond, Cat::UnamDur),
    Word::new("femtoseconds", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("femtosecond", "fs", Token::Femtosecond, Cat::UnamDur),
    Word::new("attoseconds", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("attosecond", "as", Token::Attosecond, Cat::UnamDur),
    Word::new("zeptoseconds", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("zeptosecond", "zs", Token::Zeptosecond, Cat::UnamDur),
    Word::new("yoctoseconds", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("yoctosecond", "ys", Token::Yoctosecond, Cat::UnamDur),
    Word::new("rontoseconds", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("rontosecond", "rs", Token::Rontosecond, Cat::UnamDur),
    Word::new("quectoseconds", "qs", Token::Quectosecond, Cat::UnamDur),
    Word::new("quectosecond", "qs", Token::Quectosecond, Cat::UnamDur),
    // Large time units
    Word::new("millennia", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("millennium", "mil", Token::Millennium, Cat::UnamDur),
    Word::new("centuries", "cen", Token::Century, Cat::UnamDur),
    Word::new("century", "cen", Token::Century, Cat::UnamDur),
    Word::new("decades", "dec", Token::Decade, Cat::UnamDur),
    Word::new("decade", "dec", Token::Decade, Cat::UnamDur),
    Word::new("quarters", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("quarter", "qr", Token::Quarter, Cat::UnamDur),
    Word::new("fortnights", "fn", Token::Fortnight, Cat::UnamDur),
    Word::new("fortnight", "fn", Token::Fortnight, Cat::UnamDur),
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
    // time scales for round tripping
    Word::new("tai", "TAI", Token::Scale, Cat::TScl),
    Word::new("tt", "TT", Token::Scale, Cat::TScl),
    Word::new("et", "ET", Token::Scale, Cat::TScl),
    Word::new("tdb", "TDB", Token::Scale, Cat::TScl),
    // ("utc", "UTC", Token::UTC), // avoid overlap with iana
    Word::new("utcspice", "UtcSpice", Token::Scale, Cat::TScl),
    Word::new("utchist", "UtcHist", Token::Scale, Cat::TScl),
    Word::new("gps", "GPS", Token::Scale, Cat::TScl),
    Word::new("gst", "GST", Token::Scale, Cat::TScl),
    Word::new("bdt", "BDT", Token::Scale, Cat::TScl),
    Word::new("qzss", "QZSS", Token::Scale, Cat::TScl),
    Word::new("tcg", "TCG", Token::Scale, Cat::TScl),
    Word::new("tcb", "TCB", Token::Scale, Cat::TScl),
    Word::new("ltc", "LTC", Token::Scale, Cat::TScl),
    Word::new("tcl", "TCL", Token::Scale, Cat::TScl),
    Word::new("custom", "Custom", Token::Scale, Cat::TScl),
];

static EN_DATE_AC: OnceBox<AhoCorasick> = OnceBox::new();
pub(crate) fn en_date_ac() -> &'static AhoCorasick {
    EN_DATE_AC.get_or_init(|| {
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
            .chain(tz_lowered_keys().iter().copied())
            .filter(|&s| seen.insert(s))
            .collect();

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
        let mut seen = HashSet::new();
        let terms: Vec<&'static str> = EN_WORDS
            .iter()
            .filter(|w| matches!(w.c, Cat::UnamRel | Cat::AmRel | Cat::AmDur | Cat::UnamDur))
            .map(|w| w.low)
            .filter(|&s| seen.insert(s))
            .collect();

        #[allow(clippy::expect_used)]
        let ac = AhoCorasick::builder()
            .match_kind(MatchKind::LeftmostLongest)
            .build(&terms)
            .expect("invalid Aho-Corasick patterns for EN duration terms");
        Box::new(ac)
    })
}

pub(crate) static EN: OnceBox<HashMap<&'static str, (&'static str, Token)>> = OnceBox::new();

pub(crate) fn en() -> &'static HashMap<&'static str, (&'static str, Token)> {
    EN.get_or_init(|| {
        let mut m = HashMap::new();

        // Regular English words
        for word in EN_WORDS {
            m.insert(word.low, (word.norm, word.t));
        }

        for name in crate::tz::tz_names() {
            let s = name.as_str().unwrap();
            let lowered = s.to_lowercase();
            let lowered_static = Box::leak(lowered.into_boxed_str()) as &'static str;

            let original_static =
                Box::leak(alloc::string::String::from(s).into_boxed_str()) as &'static str;

            m.insert(lowered_static, (original_static, Token::Iana));
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
            decimal_char: '.',
        })
    })
}
