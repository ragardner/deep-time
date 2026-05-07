use crate::Dt;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DateClassification {
    pub ascii_len: usize,
    pub date: String,
    pub tokens: Vec<DateToken>,
    pub is_pure_numeric: bool,
    pub is_decimal: bool,
    pub has_year: bool,
    pub num_named: u8,
    pub time: TimeType,
    pub connector: ConnectorType,
    pub offset: OffsetType,
    pub has_ampm: bool,
    pub has_fractional: bool,
    pub has_w: bool,
    pub num_colon: u8,
    pub num_hyphen: u8,
    pub num_dot: u8,
    pub num_digits: u8,
    pub num_date_digits: u8,
    pub num_non_decimal_digits: u8,
    pub num_date_digit_groups: u8,
    pub space_before_bracket: bool,
    pub space_before_offset: bool,
    pub year_maybe_on_end: bool,
}

impl DateClassification {
    #[inline]
    pub(crate) fn has_offset_or_zone(&self) -> bool {
        self.offset.is_some()
    }
}

/// A parsed date
/// Or a DateClassification for use with parsing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClassifiedDate {
    Parsed(Dt),
    Cls(DateClassification),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DateToken {
    DayShort,
    DayLong,
    MonthShort,
    MonthLong,
    Digits(usize),
    Iana,
    Am,
    Pm,
    W,
    Comma,
    Hyphen,
    Dot,
    Slash,
    Space,
    // relative
    Now,
    Today,
    Tomorrow,
    Yesterday,
    Future,
    Past,
    Plus,
    // Duration units (largest → smallest)
    Millennium,
    Century,
    Decade,
    Quarter,
    Month,
    Fortnight,
    Year,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    // Large SI-prefixed seconds
    Kilosecond,
    Megasecond,
    Gigasecond,
    Terasecond,
    Petasecond,
    Exasecond,
    Zettasecond,
    Yottasecond,
    Ronnasecond,
    Quettasecond,
    // Sub-second SI units
    Millisecond,
    Microsecond,
    Nanosecond,
    Picosecond,
    Femtosecond,
    Attosecond,
    Zeptosecond,
    Yoctosecond,
    Rontosecond,
    Quectosecond,
    TAI,
    TT,
    ET,
    TDB,
    // UTC, // avoid overlap with iana
    GPS,
    GST,
    BDT,
    QZSS,
    TCG,
    TCB,
    LTC,
    Proper,
    Custom,
}

impl DateToken {
    #[inline]
    pub(crate) fn is_relative(&self) -> bool {
        matches!(
            self,
            DateToken::Now
                | DateToken::Today
                | DateToken::Tomorrow
                | DateToken::Yesterday
                | DateToken::Future
                | DateToken::Past
                | DateToken::Nanosecond
                | DateToken::Microsecond
                | DateToken::Millisecond
                | DateToken::Second
                | DateToken::Minute
                | DateToken::Hour
                | DateToken::Day
                | DateToken::Week
                | DateToken::Month
                | DateToken::Year
        )
    }

    #[inline]
    pub(crate) fn to_fmt(&self) -> &'static [&'static str] {
        match self {
            DateToken::DayShort => &["%a"],
            DateToken::DayLong => &["%A"],
            DateToken::MonthShort => &["%b"],
            DateToken::MonthLong => &["%B"],
            DateToken::W => &["W"],
            DateToken::Comma => &[","],
            DateToken::Hyphen => &["-"],
            DateToken::Dot => &["."],
            DateToken::Slash => &["/"],
            DateToken::Space => &[" "],
            DateToken::Digits(n) => match n {
                1 => &["%e", "%_d", "%-d", "%_m", "%-m"],
                2 => &["%d", "%m", "%y"],
                3 => &["%j"],
                4 => &["%Y"], // "%G"
                5 => &["%y%j"],
                8 => &["%Y%m%d"],
                _ => &[],
            },
            _ => &[],
        }
    }

    #[inline]
    pub(crate) fn to_fmt_year_first(&self) -> &'static [&'static str] {
        match self {
            DateToken::Digits(n) => match n {
                6 => &["%y%m%d"],
                8 => &["%Y%m%d"],
                _ => self.to_fmt(),
            },
            _ => self.to_fmt(),
        }
    }

    #[inline]
    pub(crate) fn to_fmt_month_first(&self) -> &'static [&'static str] {
        match self {
            DateToken::Digits(n) => match n {
                6 => &["%m%d%y"],
                8 => &["%m%d%Y"],
                _ => self.to_fmt(),
            },
            _ => self.to_fmt(),
        }
    }

    #[inline]
    pub(crate) fn to_fmt_day_first(&self) -> &'static [&'static str] {
        match self {
            DateToken::Digits(n) => match n {
                6 => &["%d%m%y"],
                8 => &["%d%m%Y"],
                _ => self.to_fmt(),
            },
            _ => self.to_fmt(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectorType {
    None,
    UpperT,
    Space,
    Colon,
}

impl ConnectorType {
    #[inline]
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, ConnectorType::None)
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ConnectorType::None => "",
            ConnectorType::UpperT => "T",
            ConnectorType::Space => " ",
            ConnectorType::Colon => ":",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeType {
    None,
    Hm { colons: bool },
    HmS { colons: bool },
}

impl TimeType {
    #[inline]
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, TimeType::None)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) enum OffsetType {
    #[default]
    None, // no offset or zone at all
    Zulu,          // just "Z"
    Iana,          // plain "America/New_York" (usually space-separated)
    InBracketIana, // "[America/New_York]" (no offset)
    Hm {
        colon: bool,
    }, // +05:00 or +0500
    HmS {
        colon: bool,
    }, // +05:00:00 or +050000
    InBracketHm {
        colon: bool,
    }, // [+05:00]
    InBracketHmS {
        colon: bool,
    }, // [+05:00:00]
    HmAndIana {
        colon: bool,
    }, // -04:00 America/New_York   (space-separated, less common)
    HmSAndIana {
        colon: bool,
    }, // -04:00:00 America/New_York
    HmAndInbracketIana {
        colon: bool,
    }, // -04:00[America/New_York]   ← most common today
    HmSAndInbracketIana {
        colon: bool,
    }, // -04:00:00[America/New_York]
}

impl OffsetType {
    #[inline]
    pub(crate) fn is_bracketed(&self) -> bool {
        matches!(
            self,
            OffsetType::InBracketIana
                | OffsetType::InBracketHm { .. }
                | OffsetType::InBracketHmS { .. }
                | OffsetType::HmAndInbracketIana { .. }
                | OffsetType::HmSAndInbracketIana { .. }
        )
    }

    #[inline]
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, OffsetType::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IndexIn {
    PreDate,
    Date,
    PostDate,
    Time,
    Fraction,
    Offset,
    Bracket,
}

impl IndexIn {
    #[inline]
    pub(crate) fn after_date(&self) -> bool {
        !matches!(self, IndexIn::Date)
    }
}
