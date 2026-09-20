use crate::math::{powi, round};
use crate::{ATTOS_PER_SEC_I128, Dt, Real, Scale, TAI_SEC_1970_MIDNIGHT_TO_2000_NOON, UnixUnit};

/// Attoseconds per unix unit.
#[inline]
fn attos_per_unit(unit: UnixUnit) -> i128 {
    match unit {
        UnixUnit::Seconds => ATTOS_PER_SEC_I128,
        UnixUnit::Millis => 1_000_000_000_000_000i128,
        UnixUnit::Micros => 1_000_000_000_000i128,
        UnixUnit::Nanos => 1_000_000_000i128,
    }
}

/// Digit-length unit used by the unguided guess and leftover unix path.
#[inline]
pub(crate) fn infer_unix_unit(integer_digits: usize) -> UnixUnit {
    match integer_digits {
        12..=15 => UnixUnit::Millis,
        16..=18 => UnixUnit::Micros,
        19.. => UnixUnit::Nanos,
        _ => UnixUnit::Seconds,
    }
}

/// Pure-numeric Unix timestamp.
///
/// The unit is explicit. Digit length is not used to pick seconds vs millis.
pub(crate) fn parse_unix_timestamp(trimmed: &str, unit: UnixUnit) -> Option<Dt> {
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

    let attos_per = attos_per_unit(unit);

    let frac_attos = if frac_part.is_empty() {
        0i128
    } else {
        let frac_str = if frac_part.len() > 18 {
            &frac_part[..18]
        } else {
            frac_part
        };
        let frac_val: Real = frac_str.parse().ok()?;
        let divisor = powi(10.0, frac_str.len() as i32);
        let frac_real = frac_val / divisor;
        round(frac_real * attos_per as Real) as i128
    };

    let total_attos_since_unix = int_val
        .checked_mul(attos_per)?
        .checked_add(frac_attos)?
        .checked_mul(sign)?;
    let epoch_offset = (TAI_SEC_1970_MIDNIGHT_TO_2000_NOON as i128) * ATTOS_PER_SEC_I128;
    let total_attos = total_attos_since_unix.checked_sub(epoch_offset)?;

    Some(Dt::new(total_attos, Scale::UTC, Scale::UTC).to_tai())
}
