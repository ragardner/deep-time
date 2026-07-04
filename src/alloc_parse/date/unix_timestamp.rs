use crate::{
    ATTOS_PER_SEC_I128, Dt, NS_PER_SEC, Scale, TAI_SECS_1970_MIDNIGHT_TO_2000_NOON, frac_to_nanos,
};

/// Pure-numeric Unix timestamp fallback with automatic unit detection.
/// - 10–12 digit traditional Unix seconds timestamps
/// - 13-digit millisecond timestamps (the main breakage)
/// - 16-digit microsecond timestamps
/// - Any pure-numeric number with a decimal point (not caught as MJD/JD)
///
/// Unit detection is chosen for maximum real-world compatibility and uses
/// `div_euclid`/`rem_euclid` everywhere for negative-timestamp handling.
///
/// It's purely numeric so the scale is assumed to be UTC so the use of
/// TAI_SECS_1970_MIDNIGHT_TO_2000_NOON seems to be ok here
pub(crate) fn parse_pure_numeric_unix_timestamp(
    trimmed: &str,
    integer_digits: usize,
) -> Option<Dt> {
    // sign handling
    let (s, sign) = if let Some(stripped) = trimmed.strip_prefix('+') {
        (stripped, 1i128)
    } else if let Some(stripped) = trimmed.strip_prefix('-') {
        (stripped, -1i128)
    } else {
        (trimmed, 1i128)
    };

    let (int_part, frac_part) = if let Some(dot) = s.find('.') {
        (&s[..dot], &s[dot + 1..])
    } else {
        (s, "")
    };

    let int_val: i128 = if int_part.is_empty() || int_part == "0" {
        0
    } else {
        int_part.parse().ok()?
    };

    // High-precision path (≥ 19 integer digits)
    if integer_digits >= 19 {
        let frac_nanos = frac_to_nanos(frac_part).unwrap_or(0);
        // FIXED: sign now applies to the whole value (int + frac)
        let total_nanos = (int_val + frac_nanos) * sign;

        let unix_secs_i128 = total_nanos.div_euclid(NS_PER_SEC);
        let secs_i128 = unix_secs_i128 - (TAI_SECS_1970_MIDNIGHT_TO_2000_NOON as i128);
        let rem_nanos = total_nanos.rem_euclid(NS_PER_SEC) as u64;
        let secs: i64 = secs_i128.try_into().ok()?;

        let total_attos = Dt::sec_to_attos(secs as i128) + (rem_nanos * 1_000_000_000) as i128;
        return Some(Dt::from_attos(total_attos, Scale::UTC));
    }

    // Common path (1–18 digits)
    let attos_per_unit = match integer_digits {
        12..=15 => 1_000_000_000_000_000i128, // milliseconds
        16..=18 => 1_000_000_000_000i128,     // microseconds
        _ => ATTOS_PER_SEC_I128,              // seconds
    };

    let frac_attos = if frac_part.is_empty() {
        0i128
    } else {
        let frac_str = if frac_part.len() > 18 {
            &frac_part[..18]
        } else {
            frac_part
        };
        let frac_val: f64 = frac_str.parse().ok()?;
        let divisor = 10f64.powi(frac_str.len() as i32);
        let frac_f64 = frac_val / divisor;
        (frac_f64 * attos_per_unit as f64).round() as i128
    };

    let total_attos_since_unix = (int_val * attos_per_unit + frac_attos) * sign;
    let epoch_offset = (TAI_SECS_1970_MIDNIGHT_TO_2000_NOON as i128) * ATTOS_PER_SEC_I128;
    let total_attos = total_attos_since_unix - epoch_offset;

    Some(Dt::from_attos(total_attos, Scale::UTC))
}
