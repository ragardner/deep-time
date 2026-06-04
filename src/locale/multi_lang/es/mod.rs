#[cfg(feature = "parse")]
pub mod es;

#[cfg(feature = "parse")]
pub(crate) use es::*;

use crate::FormatNames;

const ES_WEEKDAYS_FULL: [&'static [u8]; 7] = byte_arrays![
    "Domingo",   // Sunday
    "Lunes",     // Monday
    "Martes",    // Tuesday
    "Miércoles", // Wednesday
    "Jueves",    // Thursday
    "Viernes",   // Friday
    "Sábado"     // Saturday
];

const ES_WEEKDAYS_ABBR: [&'static [u8]; 7] =
    byte_arrays!["Dom", "Lun", "Mar", "Mié", "Jue", "Vie", "Sáb"];

const ES_MONTHS_FULL: [&'static [u8]; 12] = byte_arrays![
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre"
];

const ES_MONTHS_ABBR: [&'static [u8]; 12] = byte_arrays![
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"
];

pub(crate) static ES_NAMES: FormatNames = FormatNames {
    weekdays_full: &ES_WEEKDAYS_FULL,
    weekdays_abbr: &ES_WEEKDAYS_ABBR,
    months_full: &ES_MONTHS_FULL,
    months_abbr: &ES_MONTHS_ABBR,
};
