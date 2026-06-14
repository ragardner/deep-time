#[cfg(feature = "parse")]
pub mod fr_alloc;

use crate::FormatNames;

const FR_WEEKDAYS_FULL: [&[u8]; 7] = byte_arrays![
    "Dimanche", // Sunday
    "Lundi",    // Monday
    "Mardi",    // Tuesday
    "Mercredi", // Wednesday
    "Jeudi",    // Thursday
    "Vendredi", // Friday
    "Samedi"    // Saturday
];

const FR_WEEKDAYS_ABBR: [&[u8]; 7] = byte_arrays!["Dim", "Lun", "Mar", "Mer", "Jeu", "Ven", "Sam"];

const FR_MONTHS_FULL: [&[u8]; 12] = byte_arrays![
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre"
];

const FR_MONTHS_ABBR: [&[u8]; 12] = byte_arrays![
    "janv", "févr", "mars", "avr", "mai", "juin", "juil", "août", "sept", "oct", "nov", "déc"
];

pub(crate) static FR_NAMES: FormatNames = FormatNames {
    weekdays_full: &FR_WEEKDAYS_FULL,
    weekdays_abbr: &FR_WEEKDAYS_ABBR,
    months_full: &FR_MONTHS_FULL,
    months_abbr: &FR_MONTHS_ABBR,
};
