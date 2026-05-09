use crate::{
    Scale, DateParseMode, JD_2000_2_451_545, MAX_YEAR, MIN_YEAR, NS_PER_DAY,
    PLAUSIBLE_YYYYMM_YEAR_RANGE, SEC_PER_DAYI64, SEC_PER_HALF_DAYI64, Dt, parse_jd,
    parse_mjd, parse_yyddd, parse_yymmdd, parse_yyyyjjj, parse_yyyymm,
};

#[inline]
pub(crate) fn parse_i32_year(input: &str) -> Option<Dt> {
    let year: i32 = input.parse().ok()?;
    if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    let year_i64 = year as i64;

    let jdn = Dt::ymd_to_jdn(year_i64, 1, 1);
    let days_since_j2000 = jdn - JD_2000_2_451_545;
    let sec_utc = days_since_j2000 * SEC_PER_DAYI64 - SEC_PER_HALF_DAYI64; // Jan 1 00:00 = JDN noon - 12 h

    Some(Dt::from(sec_utc, 0, Scale::UTC))
}

#[inline]
pub(crate) fn parse_two_digit_year(input: &str) -> Option<Dt> {
    let y: i32 = input.parse().ok()?;
    let full_year = if y <= 68 { 2000 + y } else { 1900 + y };
    if !(MIN_YEAR..=MAX_YEAR).contains(&full_year) {
        return None;
    }
    let year_i64 = full_year as i64;

    let jdn = Dt::ymd_to_jdn(year_i64, 1, 1);
    let days_since_j2000 = jdn - JD_2000_2_451_545;
    let sec_utc = days_since_j2000 * SEC_PER_DAYI64 - SEC_PER_HALF_DAYI64; // Jan 1 00:00 = JDN noon - 12 h

    Some(Dt::from(sec_utc, 0, Scale::UTC))
}

/// Fractional day string → nanoseconds (exact integer math, max 9 digits)
#[inline]
pub(crate) fn frac_to_nanos(frac_part: &str) -> Option<i128> {
    if frac_part.is_empty() {
        return Some(0);
    }
    let digits = frac_part.len().min(9);
    let frac_int: i128 = frac_part[..digits].parse().ok()?;
    let scale = 10i128.pow(digits as u32);

    let mut result = (frac_int * NS_PER_DAY) / scale;

    // Round nearest, half away from zero (identical to original f64 .round())
    let rem = (frac_int * NS_PER_DAY) % scale;
    if rem * 2 >= scale {
        result += 1;
    }
    Some(result)
}

pub(crate) fn try_pure_numeric(
    input: &str,
    total_digits: u8,
    integer_digits: u8,
    is_decimal: bool,
    mode: DateParseMode,
) -> Option<Dt> {
    // Year-only (1-4 digits)
    if (1..=4).contains(&total_digits) {
        return match mode {
            DateParseMode::Scientific => parse_i32_year(input),
            _ => {
                match total_digits {
                    2 => parse_two_digit_year(input),
                    4 => parse_i32_year(input),
                    _ => None, // 1 or 3 digits are too ambiguous
                }
            }
        };
    }
    // 5-digit handling — now also supports fractional MJD (e.g. "60400.75")
    if total_digits == 5 || (is_decimal && integer_digits == 5) {
        return match mode {
            DateParseMode::Legacy => parse_yyddd(input),
            DateParseMode::Scientific => parse_mjd(input).or_else(|| parse_yyddd(input)),
            DateParseMode::Auto => {
                if is_decimal {
                    parse_mjd(input).or_else(|| parse_yyddd(input))
                } else {
                    parse_yyddd(input).or_else(|| parse_mjd(input))
                }
            }
            _ => None,
        };
    }
    // 6-digit pure numeric — explicit per-mode strategy
    if total_digits == 6 {
        return match mode {
            DateParseMode::Legacy => parse_yymmdd(input).or_else(|| parse_yyyymm(input)),
            DateParseMode::Scientific => parse_yyyymm(input).or_else(|| parse_yymmdd(input)),
            DateParseMode::Auto => {
                let year_start = if input.starts_with('-') { 1 } else { 0 };
                let year: i32 = input[year_start..year_start + 4].parse().unwrap_or(0);
                if PLAUSIBLE_YYYYMM_YEAR_RANGE.contains(&year) {
                    parse_yyyymm(input).or_else(|| parse_yymmdd(input))
                } else {
                    parse_yymmdd(input).or_else(|| parse_yyyymm(input))
                }
            }
            _ => None,
        };
    }
    // 7-digit handling — integer JD and fractional JD
    if integer_digits == 7 {
        return match mode {
            DateParseMode::Legacy => parse_yyyyjjj(input),
            DateParseMode::Scientific => {
                parse_jd(input, !is_decimal).or_else(|| parse_yyyyjjj(input))
            }
            DateParseMode::Auto => {
                if is_decimal {
                    parse_jd(input, false)
                } else {
                    parse_yyyyjjj(input).or_else(|| parse_jd(input, false))
                }
            }
            _ => None,
        };
    }
    None
}
