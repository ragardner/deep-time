use crate::{ATTOSEC_PER_SEC, Delta};

#[cfg(feature = "alloc")]
impl Delta {
    /// Converts this `Delta` to an ISO 8601 duration string
    /// (e.g. `"PT1H23M45.6789S"`, `"-PT0.5S"`, `"PT0.000000000000000001S"`, or `"PT0S"`).
    ///
    /// This method is only available when the **`alloc`** feature is enabled.
    /// It returns `alloc::string::String` (no_std + alloc compatible).
    pub fn to_iso_duration(&self) -> alloc::string::String {
        if self.is_zero() {
            return alloc::string::String::from("PT0S");
        }

        let total = self.total_attos();
        let negative = total < 0;
        let mut attos = total.unsigned_abs() as u128;

        let mut s = alloc::string::String::with_capacity(48);
        if negative {
            s.push('-');
        }
        s.push_str("PT");

        const A_PER_S: u128 = ATTOSEC_PER_SEC as u128;
        const A_PER_M: u128 = A_PER_S * 60;
        const A_PER_H: u128 = A_PER_M * 60;

        let hours = attos / A_PER_H;
        attos %= A_PER_H;
        let minutes = attos / A_PER_M;
        attos %= A_PER_M;
        let seconds = attos / A_PER_S;
        let frac_attos = attos % A_PER_S;

        if hours > 0 {
            s.push_str(&alloc::format!("{}", hours));
            s.push('H');
        }
        if minutes > 0 {
            s.push_str(&alloc::format!("{}", minutes));
            s.push('M');
        }

        if seconds > 0 || frac_attos > 0 {
            s.push_str(&alloc::format!("{}", seconds));

            if frac_attos != 0 {
                let frac_str = alloc::format!("{frac_attos:018}");
                let trimmed = frac_str.trim_end_matches('0');
                s.push('.');
                s.push_str(trimmed);
            }

            s.push('S');
        }

        s
    }
}
