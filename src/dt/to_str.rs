use crate::{AsciiStr, Dt, DtErr, GregorianTime, STRFTIME_SIZE, Scale, tzdb::offset_info_at_utc};

#[cfg(feature = "alloc")]
use crate::ATTOS_PER_SEC;

#[cfg(feature = "alloc")]
impl Dt {
    /// High-level alloc version (defaults to UTC label-only formatting).
    #[inline]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtErr> {
        self.to_str_with_offset(fmt, 0)
    }

    /// High-level alloc version with explicit offset (label-only).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_offset(&self, fmt: &str, secs: i32) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_offset(fmt, &mut buf, secs)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// High-level alloc version for full IANA timezone formatting (with civil-time adjustment).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_tz(&self, fmt: &str, tz_name: &str) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_tz(fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// Converts this `Dt` to an ISO 8601 duration string
    /// (e.g. `"PT1H23M45.6789S"`, `"-PT0.5S"`, `"PT0.000000000000000001S"`, or `"PT0S"`).
    ///
    /// This method is only available when the **`alloc`** feature is enabled.
    /// It returns `alloc::string::String` (no_std + alloc compatible).
    #[cfg(feature = "alloc")]
    pub fn to_iso_duration(&self) -> alloc::string::String {
        if self.is_zero() {
            return alloc::string::String::from("PT0S");
        }

        let total = self.to_attos();
        let negative = total < 0;
        let mut attos = total.unsigned_abs() as u128;

        let mut s = alloc::string::String::with_capacity(48);
        if negative {
            s.push('-');
        }
        s.push_str("PT");

        const A_PER_S: u128 = ATTOS_PER_SEC as u128;
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

impl Dt {
    /// No-alloc label-only formatting.
    pub fn to_str_bin(&self, fmt: &str) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let mut gt = self.to_gregorian_time();
        gt.set_offset(Some(0)).set_tz_abbrev(None);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// No-alloc label-only formatting.
    pub fn to_str_bin_with_offset(
        &self,
        fmt: &str,
        secs: i32,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.gregorian_time_with_offset(secs);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// No-alloc full IANA adjusted formatting (civil time is adjusted to local wall time).
    pub fn to_str_bin_with_tz(
        &self,
        fmt: &str,
        tz_name: &str,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.gregorian_time_with_tz(tz_name);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// Returns `(is_negative, hours, minutes)`.
    #[inline]
    pub const fn sec_as_hhmm(seconds: i32) -> (bool, u8, u8) {
        let total = seconds.abs();
        let hours = (total / 3600) as u8;
        let minutes = ((total % 3600) / 60) as u8;
        (seconds < 0, hours, minutes)
    }

    /// Helper for to_str.
    pub fn to_u8_with_offset(&self, fmt: &str, dest: &mut [u8], secs: i32) -> Result<usize, DtErr> {
        let gt = self.gregorian_time_with_offset(secs);
        let mut internal_buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut internal_buf, &mut pos)?;
        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Helper for to_str.
    pub fn to_u8_with_tz(&self, fmt: &str, dest: &mut [u8], tz_name: &str) -> Result<usize, DtErr> {
        let gt = self.gregorian_time_with_tz(tz_name);
        let mut internal_buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut internal_buf, &mut pos)?;
        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Helper for creating an offset adjusted GregorianTime.
    pub(crate) fn gregorian_time_with_offset(&self, secs: i32) -> GregorianTime {
        let local_tp = if secs != 0 {
            *self + Dt::new(secs as i64, 0)
        } else {
            *self
        };
        let mut gt = local_tp.to_gregorian_time();
        gt.set_offset(Some(secs));
        gt
    }

    /// Helper for creating a timezone-adjusted GregorianTime.
    ///
    /// Always converts to UTC first, then does a correct UTC-based lookup
    /// in the IANA transition table. This avoids the previous bug where
    /// a non-UTC `unix_ts` was being passed to `offset_info_at_local`.
    pub(crate) fn gregorian_time_with_tz(&self, tz_name: &str) -> GregorianTime {
        // 1. Get the true UTC Unix timestamp (this is what we search with)
        let utc_unix = self
            .to_scale_and_then_diff(Scale::UTC, Dt::UNIX_EPOCH)
            .to_sec();

        // 2. Look up offset + abbrev at that exact UTC instant
        let (offset_secs, abbrev) = match offset_info_at_utc(tz_name, utc_unix) {
            Some(info) => (info.offset, info.abbrev),
            None => (0, "UTC"), // fallback for unknown timezone
        };

        // 3. Build local time = UTC + offset
        let span = Dt::new(offset_secs as i64, 0);
        let local_tp = *self + span;

        let mut gt = local_tp.to_gregorian_time();
        gt.set_offset(Some(offset_secs));
        gt.set_tz(Some(tz_name));
        gt.set_tz_abbrev(Some(abbrev));
        gt
    }
}
