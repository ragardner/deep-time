use crate::math::floor_f;
use crate::{
    DateBase, Dt, JD_RANGE, MAX_YEAR, MIN_YEAR, MJD_RANGE, Numeric, PLAUSIBLE_YYYYMM_YEAR_RANGE,
    Real, Scale, UnixUnit, infer_unix_unit, parse_unix_timestamp, parse_yyddd, parse_yymmdd,
    parse_yyyyjjj, parse_yyyymm, parse_yyyymmdd,
};

/// Modified Julian Date (MJD) interpreted as UTC
pub(crate) fn parse_mjd(s: &str) -> Option<Dt> {
    let mjd: Real = s.parse().ok()?;
    let days = mjd as i64;
    if !MJD_RANGE.contains(&days) {
        return None;
    }
    Some(Dt::from_mjd_f(mjd, Scale::UTC))
}

/// Julian Day (JD) interpreted as UTC
pub(crate) fn parse_jd(s: &str, astronomical_noon: bool) -> Option<Dt> {
    let mut jd: Real = s.parse().ok()?;
    let days = jd as i64;
    if !JD_RANGE.contains(&days) {
        return None;
    }
    if astronomical_noon {
        jd += f!(0.5);
    }
    Some(Dt::from_jd_f(jd, Scale::UTC))
}

#[inline]
pub(crate) fn parse_i32_year(input: &str) -> Option<Dt> {
    let year: i32 = input.parse().ok()?;
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    Some(Dt::from_ymd(year as i64, 1, 1, Scale::UTC, 0, 0, 0, 0))
}

#[inline]
pub(crate) fn parse_two_digit_year(input: &str) -> Option<Dt> {
    let y: i32 = input.parse().ok()?;
    let year = if y <= 68 { 2000 + y } else { 1900 + y };
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    Some(Dt::from_ymd(year as i64, 1, 1, Scale::UTC, 0, 0, 0, 0))
}

fn compact_ymd_6(input: &str) -> Option<Dt> {
    let year_start = if input.starts_with('-') { 1 } else { 0 };
    let year: i32 = input.get(year_start..year_start + 4)?.parse().ok()?;
    if PLAUSIBLE_YYYYMM_YEAR_RANGE.contains(&year) {
        parse_yyyymm(input).or_else(|| parse_yymmdd(input))
    } else {
        parse_yymmdd(input).or_else(|| parse_yyyymm(input))
    }
}

fn year_digits(input: &str, total_digits: u8, two_digit_pivot: bool) -> Option<Dt> {
    if !(1..=4).contains(&total_digits) {
        return None;
    }
    if two_digit_pivot && total_digits == 2 {
        parse_two_digit_year(input)
    } else {
        parse_i32_year(input)
    }
}

/// Pin: the input is this numeric type. No fallback to another type.
fn try_pinned_numeric(
    input: &str,
    total_digits: u8,
    integer_digits: u8,
    is_decimal: bool,
    numeric: Numeric,
) -> Option<Dt> {
    match numeric {
        Numeric::Unix(unit) => parse_unix_timestamp(input, unit),
        Numeric::Mjd => parse_mjd(input),
        Numeric::Jd => parse_jd(input, !is_decimal),
        Numeric::Ordinal => {
            if integer_digits == 5 && !is_decimal {
                parse_yyddd(input)
            } else if integer_digits == 7 && !is_decimal {
                parse_yyyyjjj(input)
            } else {
                None
            }
        }
        Numeric::Year { two_digit_pivot } => {
            if is_decimal {
                None
            } else {
                year_digits(input, total_digits, two_digit_pivot)
            }
        }
        Numeric::CompactYmd => {
            if is_decimal {
                return None;
            }
            match total_digits {
                6 => compact_ymd_6(input),
                8 => parse_yyyymmdd(input),
                _ => None,
            }
        }
        Numeric::DateSerial(base) => parse_date_serial(input, base),
    }
}

/// Spreadsheet date-time serial: integer day + fraction of a day as time (UTC).
fn parse_date_serial(input: &str, base: DateBase) -> Option<Dt> {
    let serial: Real = input.parse().ok()?;
    if !serial.is_finite() {
        return None;
    }
    let day = floor_f(serial) as i64;

    let (epoch, days) = match base {
        DateBase::Epoch1899 => {
            if day > 2_958_465 {
                return None;
            }
            (Dt::SERIAL_EPOCH_1899, serial)
        }
        DateBase::Epoch1900 => {
            if day == 60 || day > 2_958_465 {
                return None;
            }
            if day < 60 {
                (Dt::SERIAL_EPOCH_1900, serial - f!(1.0))
            } else {
                (Dt::SERIAL_EPOCH_1899, serial)
            }
        }
        DateBase::Epoch1904 => {
            if day > 2_957_003 {
                return None;
            }
            (Dt::SERIAL_EPOCH_1904, serial)
        }
    };

    Some(Dt::from_days_since(days, epoch, Scale::UTC))
}

/// Unguided guess: valid civil form for this length first, else MJD/JD/unix
/// when that is the remaining reading.
fn try_unguided_numeric(
    input: &str,
    total_digits: u8,
    integer_digits: u8,
    is_decimal: bool,
) -> Option<Dt> {
    if !is_decimal && (1..=4).contains(&total_digits) {
        return year_digits(input, total_digits, true);
    }
    if total_digits == 5 || (is_decimal && integer_digits == 5) {
        return if is_decimal {
            parse_mjd(input).or_else(|| parse_yyddd(input))
        } else {
            parse_yyddd(input).or_else(|| parse_mjd(input))
        };
    }
    if !is_decimal && total_digits == 6 {
        return compact_ymd_6(input);
    }
    if integer_digits == 7 {
        return if is_decimal {
            parse_jd(input, false)
        } else {
            parse_yyyyjjj(input).or_else(|| parse_jd(input, true))
        };
    }
    if !is_decimal && total_digits == 8 {
        return parse_yyyymmdd(input).or_else(|| parse_unix_timestamp(input, UnixUnit::Seconds));
    }
    // 10 / 12 / 14: compact datetime (optional fractional seconds) —
    // leave for format generation.
    if matches!(integer_digits, 10 | 12 | 14) {
        return None;
    }
    if integer_digits >= 9 {
        return parse_unix_timestamp(input, infer_unix_unit(integer_digits as usize));
    }
    None
}

pub(crate) fn try_pure_numeric(
    input: &str,
    total_digits: u8,
    integer_digits: u8,
    is_decimal: bool,
    numeric: Option<Numeric>,
) -> Option<Dt> {
    match numeric {
        Some(n) => try_pinned_numeric(input, total_digits, integer_digits, is_decimal, n),
        None => try_unguided_numeric(input, total_digits, integer_digits, is_decimal),
    }
}
