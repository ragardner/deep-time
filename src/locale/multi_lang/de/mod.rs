#[cfg(feature = "parse")]
pub mod de;

#[cfg(feature = "parse")]
pub(crate) use de::*;

use crate::FormatNames;

const DE_WEEKDAYS_FULL: [&'static [u8]; 7] = byte_arrays![
    "Sonntag",
    "Montag",
    "Dienstag",
    "Mittwoch",
    "Donnerstag",
    "Freitag",
    "Samstag"
];

const DE_WEEKDAYS_ABBR: [&'static [u8]; 7] = byte_arrays!["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];

const DE_MONTHS_FULL: [&'static [u8]; 12] = byte_arrays![
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember"
];

const DE_MONTHS_ABBR: [&'static [u8]; 12] = byte_arrays![
    "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"
];

pub(crate) static DE_NAMES: FormatNames = FormatNames {
    weekdays_full: &DE_WEEKDAYS_FULL,
    weekdays_abbr: &DE_WEEKDAYS_ABBR,
    months_full: &DE_MONTHS_FULL,
    months_abbr: &DE_MONTHS_ABBR,
};
