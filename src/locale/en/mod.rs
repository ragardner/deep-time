#[cfg(feature = "parse")]
pub mod en_alloc;

#[cfg(feature = "parse")]
pub(crate) use en_alloc::*;

use crate::{DtErr, DtErrKind, FormatNames, an_err};

pub(crate) const EN_WEEKDAYS_FULL: [&[u8]; 7] = byte_arrays![
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday"
];

pub(crate) const EN_WEEKDAYS_ABBR: [&[u8]; 7] =
    byte_arrays!["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub(crate) const EN_MONTHS_FULL: [&[u8]; 12] = byte_arrays![
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December"
];

pub(crate) const EN_MONTHS_ABBR: [&[u8]; 12] = byte_arrays![
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
];

pub(crate) static EN_NAMES: FormatNames = FormatNames {
    weekdays_full: &EN_WEEKDAYS_FULL,
    weekdays_abbr: &EN_WEEKDAYS_ABBR,
    months_full: &EN_MONTHS_FULL,
    months_abbr: &EN_MONTHS_ABBR,
};

/// bytes **must** be len >= 3
#[inline(always)]
pub(crate) fn parse_month_name_abbrev(bytes: &[u8]) -> Result<u8, DtErr> {
    let candidate = [
        bytes[0].to_ascii_lowercase(),
        bytes[1].to_ascii_lowercase(),
        bytes[2].to_ascii_lowercase(),
    ];
    match &candidate {
        b"jan" => Ok(1),
        b"feb" => Ok(2),
        b"mar" => Ok(3),
        b"apr" => Ok(4),
        b"may" => Ok(5),
        b"jun" => Ok(6),
        b"jul" => Ok(7),
        b"aug" => Ok(8),
        b"sep" => Ok(9),
        b"oct" => Ok(10),
        b"nov" => Ok(11),
        b"dec" => Ok(12),
        _ => {
            return Err(an_err!(DtErrKind::InvalidMonthName));
        }
    }
}
