#[cfg(feature = "parse")]
pub mod fr;

#[cfg(feature = "parse")]
pub(crate) use fr::*;

use crate::FormatNames;

const FR_WEEKDAYS_FULL: [&'static [u8]; 7] = byte_arrays![
    "Dimanche", // Sunday
    "Lundi",    // Monday
    "Mardi",    // Tuesday
    "Mercredi", // Wednesday
    "Jeudi",    // Thursday
    "Vendredi", // Friday
    "Samedi"    // Saturday
];

const FR_WEEKDAYS_ABBR: [&'static [u8]; 7] =
    byte_arrays!["Dim", "Lun", "Mar", "Mer", "Jeu", "Ven", "Sam"];

const FR_MONTHS_FULL: [&'static [u8]; 12] = byte_arrays![
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

const FR_MONTHS_ABBR: [&'static [u8]; 12] = byte_arrays![
    "janv", "févr", "mars", "avr", "mai", "juin", "juil", "août", "sept", "oct", "nov", "déc"
];

pub(crate) static FR_NAMES: FormatNames = FormatNames {
    weekdays_full: &FR_WEEKDAYS_FULL,
    weekdays_abbr: &FR_WEEKDAYS_ABBR,
    months_full: &FR_MONTHS_FULL,
    months_abbr: &FR_MONTHS_ABBR,
};
