use crate::frac_to_nanos;
use crate::{ClockType, TimePoint};

/// Pure-numeric Unix timestamp fallback with automatic unit detection.
/// - 10–12 digit traditional Unix seconds timestamps
/// - 13-digit millisecond timestamps (the main breakage)
/// - 16-digit microsecond timestamps
/// - Any pure-numeric number with a decimal point (not caught as MJD/JD)
///
/// Unit detection is chosen for maximum real-world compatibility and uses
/// `div_euclid`/`rem_euclid` everywhere for correct negative-timestamp handling.
///
/// Returns a `TimePoint` in `ClockType::UTC`.
#[inline]
pub(crate) fn parse_pure_numeric_unix_timestamp(
    trimmed: &str,
    integer_digits: usize,
) -> Option<TimePoint> {
    let ts_f64: f64 = trimmed.parse().ok()?;

    let tp = match integer_digits {
        // 12–15 digits → milliseconds
        12..=15 => {
            let total_millis = ts_f64.trunc() as i64;
            let secs = total_millis.div_euclid(1_000);
            let rem_millis = total_millis.rem_euclid(1_000) as u32;

            let frac_nanos =
                ((ts_f64.fract().abs() * 1_000_000_000.0).round() as u32).min(999_999_999);

            let total_subsec_nanos = (rem_millis as u64) * 1_000_000 + (frac_nanos as u64);
            let subsec_attos = total_subsec_nanos * 1_000_000_000;

            TimePoint::new(secs, subsec_attos, ClockType::UTC)
        }

        // 16–18 digits → microseconds
        16..=18 => {
            let total_micros = ts_f64.trunc() as i64;
            let secs = total_micros.div_euclid(1_000_000);
            let rem_micros = total_micros.rem_euclid(1_000_000) as u32;

            let frac_nanos =
                ((ts_f64.fract().abs() * 1_000_000_000.0).round() as u32).min(999_999_999);

            let total_subsec_nanos = (rem_micros as u64) * 1_000 + (frac_nanos as u64);
            let subsec_attos = total_subsec_nanos * 1_000_000_000;

            TimePoint::new(secs, subsec_attos, ClockType::UTC)
        }

        // 19+ digits → nanoseconds (uses existing `frac_to_nanos` for perfect precision)
        19.. => {
            let (int_part, frac_part) = if let Some(dot) = trimmed.find('.') {
                (&trimmed[..dot], &trimmed[dot + 1..])
            } else {
                (trimmed, "")
            };
            if let Ok(int_nanos) = int_part.parse::<i128>() {
                let frac_nanos = frac_to_nanos(frac_part).unwrap_or(0);
                let total_nanos = int_nanos + frac_nanos;

                let ns_per_sec = 1_000_000_000i128;
                let secs_i128 = total_nanos.div_euclid(ns_per_sec);
                let rem_nanos = total_nanos.rem_euclid(ns_per_sec) as u64;

                let secs: i64 = secs_i128.try_into().ok()?;

                let subsec_attos = rem_nanos * 1_000_000_000;
                TimePoint::new(secs, subsec_attos, ClockType::UTC)
            } else {
                // Extremely rare fallback
                let secs = ts_f64.trunc() as i64;
                let nanos = ((ts_f64.fract().abs() * 1_000_000_000.0).round() as u32)
                    .min(999_999_999) as u64;
                let subsec_attos = nanos * 1_000_000_000;
                TimePoint::new(secs, subsec_attos, ClockType::UTC)
            }
        }

        // Everything else (1–11 digits + huge future seconds) → classic Unix seconds
        _ => {
            let secs = ts_f64.trunc() as i64;
            let nanos =
                ((ts_f64.fract().abs() * 1_000_000_000.0).round() as u32).min(999_999_999) as u64;
            let subsec_attos = nanos * 1_000_000_000;
            TimePoint::new(secs, subsec_attos, ClockType::UTC).to_tai()
        }
    };

    Some(tp)
}
