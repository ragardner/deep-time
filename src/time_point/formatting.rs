use crate::{ClockType, TimePoint};
use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "alloc")]
impl TimePoint {
    /// Returns this instant as an **RFC 3339** / ISO 8601 timestamp in **UTC**
    /// with the `Z` suffix.
    ///
    /// - Always uses UTC (`Z` = Zulu = UTC).
    /// - Default = 9 digits (nanoseconds) but **automatically trims trailing zeros**.
    /// - If fractional part is zero → no decimal point at all (e.g. `...45Z`).
    /// - Example: `"2024-03-14T15:30:45.123Z"`
    pub fn to_rfc3339(&self) -> String {
        self.to_rfc3339_precision(9)
    }

    /// Same as [`to_rfc3339`] but with a configurable maximum number of fractional digits
    /// (0–18). Trailing zeros are always trimmed.
    pub fn to_rfc3339_precision(&self, max_precision: usize) -> String {
        let utc = self.to_clock_type(ClockType::UTC);

        let (year, month, day) = utc.to_gregorian_date(None);
        let (hour, minute, second, subsec_attos) = utc.to_hms_subsec();

        // RFC 3339 / ISO 8601 requires exactly 4 digits for years |year| <= 9999.
        // Larger years use full width (no padding). Negative years always have the sign.
        let year_str = if year.abs() < 10_000 {
            if year < 0 {
                alloc::format!("-{:04}", -year)
            } else {
                alloc::format!("{:04}", year)
            }
        } else {
            alloc::format!("{}", year)
        };

        let mut prec = max_precision.min(18);

        if prec == 0 {
            return alloc::format!(
                "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year_str,
                month,
                day,
                hour,
                minute,
                second
            );
        }

        let scale = 10u64.pow(18 - prec as u32);
        let mut frac_value = subsec_attos / scale;

        // Trim trailing zeros (no string ops)
        while prec > 0 && frac_value % 10 == 0 {
            frac_value /= 10;
            prec -= 1;
        }

        if prec == 0 {
            alloc::format!(
                "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year_str,
                month,
                day,
                hour,
                minute,
                second
            )
        } else {
            alloc::format!(
                "{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:0>width$}Z",
                year_str,
                month,
                day,
                hour,
                minute,
                second,
                frac_value,
                width = prec
            )
        }
    }
}

/// Writes the fractional part of a second when `subsec` is in **attoseconds** (10⁻¹⁸ s).
///
/// - `precision` = number of decimal digits after the point (capped at 18 because a u64 can't hold more).
/// - This is the **correct** version after switching the internal representation to attoseconds.
fn write_fractional(subsec: u64, precision: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if precision == 0 {
        return Ok(());
    }

    // Cap at 18 digits (attosecond precision limit)
    let prec = precision.min(18);
    let scale = 10u64.pow(18 - prec as u32);
    let value = subsec / scale;

    write!(f, ".{:0>width$}", value, width = prec)
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sec = self.sec();
        let subsec = self.subsec();

        // Default to nanosecond precision (9 digits) — most useful for everyday use
        let precision = f.precision().unwrap_or(9);

        // Respect the `+` sign when the user writes {:+}
        if f.sign_plus() && sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", sec)?;
        write_fractional(subsec, precision, f)?;

        if f.alternate() {
            // # flag → raw internal representation (great for debugging)
            write!(
                f,
                " [{} | sec={} subsec={}]",
                self.clock_type(),
                sec,
                subsec
            )?;
        } else {
            write!(f, " [{}]", self.clock_type())?;
        }

        Ok(())
    }
}

impl fmt::Debug for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let approx_sec = self.sec() as f64 + (self.subsec() as f64 / 1_000_000_000_000_000_000.0);

        f.debug_struct("TimePoint")
            .field("sec", &self.sec())
            .field("subsec", &self.subsec())
            .field("clock_type", &self.clock_type())
            .field("approx_sec", &approx_sec)
            .finish()
    }
}
