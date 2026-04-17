use crate::TimePoint;
use core::fmt;

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
