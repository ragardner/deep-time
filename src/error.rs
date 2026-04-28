use crate::EzError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtErrKind {
    ParserErr,
    FormatterErr,
    OutOfRange,
    TrailingCharacters,
    Incomplete,
    InvalidDuration,
    InvalidDate,
    CCSDSInputErr,
    CCSDSOutputErr,
    InternalErr,
    IOErr,
    JiffConversion,
    ChronoConversion,
}

pub type DtError = EzError<DtErrKind, 3>;
