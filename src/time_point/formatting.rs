use crate::TimePoint;
use core::fmt;

fn write_fractional(subsec: u128, precision: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if precision == 0 {
        return Ok(());
    }
    let prec = precision.min(36);
    let scale = 10u128.pow(36 - prec as u32);
    let value = subsec / scale;
    write!(f, ".{:0>width$}", value, width = prec)
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sec = self.sec();
        let subsec = self.subsec();

        // Default to nanosecond precision (9 digits) — most useful default
        let precision = f.precision().unwrap_or(9);

        // Respect + sign when requested
        if f.sign_plus() && sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", sec)?;
        write_fractional(subsec, precision, f)?;

        if f.alternate() {
            // # flag → raw internal representation (excellent for debugging)
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
        let approx_sec_f = self.sec() as f64 + (self.subsec() as f64 / 1e36_f64);

        f.debug_struct("TimePoint")
            .field("sec", &self.sec())
            .field("subsec", &self.subsec())
            .field("clock_type", &self.clock_type())
            .field("as_sec_f", &approx_sec_f) // fixed
            .finish()
    }
}
