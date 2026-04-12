use crate::Timestamp;
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

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(12); // ← changed default (picosecond level)

        write!(f, "{}", self.sec())?;
        write_fractional(self.subsec(), precision, f)?;
        write!(f, " [{}]", self.clock_type())
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Timestamp")
            .field("sec", &self.sec())
            .field("subsec", &self.subsec())
            .field("clock_type", &self.clock_type())
            .finish()
    }
}
