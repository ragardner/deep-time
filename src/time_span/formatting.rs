use crate::TimeSpan;
use core::fmt;

fn write_fractional(subsec: u64, precision: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if precision == 0 {
        return Ok(());
    }
    let prec = precision.min(18); // attosecond precision (10^{-18} s)
    let scale = 10u64.pow(18 - prec as u32);
    let value = subsec / scale;
    write!(f, ".{:0>width$}", value, width = prec)
}

impl fmt::Display for TimeSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(9); // nanosecond precision — most useful default

        if self.is_zero() {
            // Special-case zero for cleanliness (no fractional part), but still respect flags
            let sign = if f.sign_plus() { "+" } else { "" };
            write!(f, "{sign}0")?;

            if f.alternate() {
                // # flag → raw internal representation (excellent for debugging)
                write!(f, " [sec=0 subsec=0]")?;
            }

            f.write_str(" s")?;
            return Ok(());
        }

        // Respect + sign when requested
        if f.sign_plus() && self.sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", self.sec)?;
        write_fractional(self.subsec, precision, f)?;
        f.write_str(" s")?;

        if f.alternate() {
            // # flag → raw internal representation (excellent for debugging)
            write!(f, " [sec={} subsec={}]", self.sec, self.subsec)?;
        }

        Ok(())
    }
}

impl fmt::Debug for TimeSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let approx_sec_f = self.sec as f64 + (self.subsec as f64 / 1e18_f64);

        f.debug_struct("TimeSpan")
            .field("sec", &self.sec)
            .field("subsec", &self.subsec)
            .field("as_sec_f", &approx_sec_f)
            .finish()
    }
}
