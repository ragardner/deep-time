use crate::Delta;
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

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(12); // ← same default

        if self.is_zero() {
            return f.write_str("0 s");
        }

        write!(f, "{}", self.sec)?;
        write_fractional(self.subsec, precision, f)?;
        f.write_str(" s")
    }
}

impl fmt::Debug for Delta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Delta")
            .field("sec", &self.sec)
            .field("subsec", &self.subsec)
            .finish()
    }
}
