#[cfg(feature = "parse")]
pub mod de_alloc;

use crate::FormatNames;

const DE_WEEKDAYS_FULL: [&[u8]; 7] = byte_arrays![
    "Sonntag",
    "Montag",
    "Dienstag",
    "Mittwoch",
    "Donnerstag",
    "Freitag",
    "Samstag"
];

const DE_WEEKDAYS_ABBR: [&[u8]; 7] = byte_arrays!["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];

const DE_MONTHS_FULL: [&[u8]; 12] = byte_arrays![
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

const DE_MONTHS_ABBR: [&[u8]; 12] = byte_arrays![
    "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"
];

pub(crate) static DE_NAMES: FormatNames = FormatNames {
    weekdays_full: &DE_WEEKDAYS_FULL,
    weekdays_abbr: &DE_WEEKDAYS_ABBR,
    months_full: &DE_MONTHS_FULL,
    months_abbr: &DE_MONTHS_ABBR,
};
