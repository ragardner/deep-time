use crate::Dt;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DateClassification {
    pub bytes_len: usize,
    pub date: String,
    pub date_tokens: Vec<Token>,
    pub time_tokens: Vec<Token>,
    pub is_pure_numeric: bool,
    pub is_decimal: bool,
    pub has_year: bool,
    pub has_time: bool,
    pub num_named: u8,
    pub connector: ConnectorType,
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
    pub year_maybe_on_end: bool,
}

impl DateClassification {
    #[inline(always)]
    pub(crate) fn has_offset_or_zone(&self) -> bool {
        self.time_tokens
            .iter()
            .any(|t| matches!(t, Token::Offset | Token::OffsetColon | Token::Iana))
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
pub(crate) enum Token {
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
    // Colon,
    Hyphen,
    Dot,
    Slash,
    Space,
    Minus,
    LBracket,
    RBracket,
    Zulu,
    Hm,
    HmColon,
    Hms,
    HmsColon,
    Offset,
    OffsetColon,
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
    Scale,
}

impl Token {
    #[inline]
    pub(crate) fn is_relative(&self) -> bool {
        matches!(
            self,
            Token::Now
                | Token::Today
                | Token::Tomorrow
                | Token::Yesterday
                | Token::Future
                | Token::Past
                | Token::Nanosecond
                | Token::Microsecond
                | Token::Millisecond
                | Token::Second
                | Token::Minute
                | Token::Hour
                | Token::Day
                | Token::Week
                | Token::Month
                | Token::Year
        )
    }

    #[inline]
    pub(crate) fn to_fmt(&self) -> &'static [&'static str] {
        match self {
            Token::DayShort => &["%a"],
            Token::DayLong => &["%A"],
            Token::MonthShort => &["%b"],
            Token::MonthLong => &["%B"],
            Token::W => &["W"],
            Token::Comma => &[","],
            Token::Hyphen => &["-"],
            Token::Dot => &["."],
            Token::Slash => &["/"],
            Token::Space => &[" "],
            Token::Digits(n) => match n {
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
            Token::Digits(n) => match n {
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
            Token::Digits(n) => match n {
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
            Token::Digits(n) => match n {
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
pub(crate) enum IndexIn {
    PreDate,
    Date,
    PostDate,
    Time,
    Fraction,
    Offset,
}

impl IndexIn {
    #[inline(always)]
    pub(crate) fn after_date(&self) -> bool {
        !matches!(self, IndexIn::PreDate | IndexIn::Date)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    /// Ambiguous relative
    AmRel,
    /// Unambiguous relative
    UnamRel,
    /// Ambiguous duration
    AmDur,
    UnamDur,
    Month,
    Day,
    AmPm,
    TScl,
}

#[derive(Clone, Copy, Debug)]
pub struct Word {
    pub low: &'static str,
    pub norm: &'static str,
    pub t: Token,
    pub c: Cat,
}

impl Word {
    #[inline]
    pub const fn new(low: &'static str, norm: &'static str, t: Token, c: Cat) -> Word {
        Word { low, norm, t, c }
    }
}

#[derive(Clone)]
pub(crate) struct AmBuilder {
    pub pieces: Vec<&'static str>,
    pub seen_year: bool,
    pub seen_month: bool,
    pub seen_day: bool,
}

#[inline]
pub(crate) fn append_to_all(builders: &mut Vec<AmBuilder>, s: &'static str) {
    for b in builders {
        b.pieces.push(s);
    }
}
