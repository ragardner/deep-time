use crate::{DateOrder, DateParseMode, Lang, ParseCfg};
use alloc::string::String;
use core::fmt::{Display, Formatter, Result};
use core::panic::Location;

#[derive(Debug)]
pub enum DtStdError {
    Date {
        input: String,
        reason: String,
        mode: DateParseMode,
        order: DateOrder,
        lang: Lang,
        to_lower: bool,
        verbose: bool,
        location: &'static Location<'static>,
    },

    Duration {
        input: String,
        reason: String,
        lang: Lang,
        location: &'static Location<'static>,
    },

    /// Failed to parse `input` according to a strftime-style `fmt` string
    /// (used by the low-level `Parser` in the custom format parser).
    Strftime {
        fmt: String,
        input: String,
        reason: String,
        location: &'static Location<'static>,
    },

    /// Simple general-purpose error (only input + reason + location)
    Simple {
        input: String,
        reason: String,
        location: &'static Location<'static>,
    },

    Reason {
        reason: String,
        location: &'static Location<'static>,
    },
}

impl DtStdError {
    #[inline]
    #[track_caller]
    pub fn date(input: String, reason: String, opts: &ParseCfg, verbose: bool) -> Self {
        Self::Date {
            input,
            reason,
            mode: opts.mode,
            order: opts.order,
            lang: opts.lang,
            to_lower: opts.to_lower,
            verbose,
            location: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn duration(input: String, reason: String, lang: Lang) -> Self {
        Self::Duration {
            input,
            reason,
            lang,
            location: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn strftime(
        fmt: impl Into<String>,
        input: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::Strftime {
            fmt: fmt.into(),
            input: input.into(),
            reason: reason.into(),
            location: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn simple(input: String, reason: String) -> Self {
        Self::Simple {
            input,
            reason,
            location: Location::caller(),
        }
    }

    #[inline]
    #[track_caller]
    pub fn reason(reason: String) -> Self {
        Self::Reason {
            reason,
            location: Location::caller(),
        }
    }
}

impl Display for DtStdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            DtStdError::Date {
                input,
                reason,
                mode,
                order,
                lang,
                to_lower,
                verbose,
                location,
            } => {
                if *verbose {
                    writeln!(f, "--")?;
                    writeln!(f, "Could not parse: \"{}\"", input)?;
                    writeln!(f, "• Reason   : {}", reason)?;
                    writeln!(f, "• Mode     : {:?}", mode)?;
                    writeln!(f, "• Order    : {:?}", order)?;
                    writeln!(f, "• Lang     : {:?}", lang)?;
                    writeln!(f, "• ToLower  : {}", to_lower)?;
                    writeln!(f, "    at {}:{}", location.file(), location.line())
                } else {
                    writeln!(f, "Could not parse: \"{}\"", input)?;
                    writeln!(f, "• Reason   : {}", reason)?;
                    writeln!(f, "    at {}:{}", location.file(), location.line())
                }
            }

            DtStdError::Duration {
                input,
                reason,
                lang,
                location,
            } => {
                writeln!(f, "--")?;
                writeln!(f, "Could not parse: \"{}\"", input)?;
                writeln!(f, "• Reason      : {}", reason)?;
                writeln!(f, "• Lang        : {:?}", lang)?;
                writeln!(f, "    at {}:{}", location.file(), location.line())
            }

            DtStdError::Strftime {
                fmt,
                input,
                reason,
                location,
            } => {
                writeln!(f, "--")?;
                writeln!(f, "Could not parse: \"{}\"", input)?;
                writeln!(f, "• Format   : \"{}\"", fmt)?;
                writeln!(f, "• Reason   : {}", reason)?;
                writeln!(f, "    at {}:{}", location.file(), location.line())
            }

            DtStdError::Simple {
                input,
                reason,
                location,
            } => {
                writeln!(f, "--")?;
                writeln!(f, "Input: \"{}\"", input)?;
                writeln!(f, "• Reason   : {}", reason)?;
                writeln!(f, "    at {}:{}", location.file(), location.line())
            }

            DtStdError::Reason { reason, location } => {
                writeln!(f, "--")?;
                writeln!(f, "• Reason   : {}", reason)?;
                writeln!(f, "    at {}:{}", location.file(), location.line())
            }
        }
    }
}

// Implement Error trait from core
impl core::error::Error for DtStdError {}

// Properly formats a String Err. Supports literal strings, format! syntax,
// AND direct String/&str expressions (or any impl Display).
#[macro_export]
macro_rules! str_err {
    // Single expression case
    ($err:expr) => {
        alloc::format!("Error at {}:{}: {}", file!(), line!(), $err)
    };

    // Format! syntax case
    ($($arg:tt)*) => {
        alloc::format!("Error at {}:{}: {}", file!(), line!(), alloc::format!($($arg)*))
    };
}
