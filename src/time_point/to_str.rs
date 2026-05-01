use crate::{
    AsciiStr, ClockType, DtErr, GregorianTime, STRFTIME_SIZE, TimePoint, TimeSpan,
    tzdb::offset_info_at_utc,
};

impl TimePoint {
    /// High-level alloc version (defaults to UTC label-only formatting).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtErr> {
        self.to_str_with_offset(fmt, 0)
    }

    /// High-level alloc version with explicit offset (label-only).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_offset(
        &self,
        fmt: &str,
        secs: i32,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_offset(fmt, &mut buf, secs)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// High-level alloc version for full IANA timezone formatting (with civil-time adjustment).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_tz(
        &self,
        fmt: &str,
        tz_name: &str,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_tz(fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

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
    pub fn to_u8_with_offset(
        &self,
        fmt: &str,
        dest: &mut [u8],
        secs: i32,
    ) -> Result<usize, DtErr> {
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
    pub fn to_u8_with_tz(
        &self,
        fmt: &str,
        dest: &mut [u8],
        tz_name: &str,
    ) -> Result<usize, DtErr> {
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
        let orig_clock_type = self.clock_type;
        let utc = self.to_clock_type(ClockType::UTC);
        let local_tp = if secs != 0 {
            utc + TimeSpan::new(secs as i64, 0)
        } else {
            utc
        };
        let mut gt = local_tp.to_gregorian_time();
        gt.set_offset(Some(secs));
        gt.set_clock_type(orig_clock_type);
        gt
    }

    /// Helper for creating a timezone-adjusted GregorianTime.
    ///
    /// Always converts to UTC first, then does a correct UTC-based lookup
    /// in the IANA transition table. This avoids the previous bug where
    /// a non-UTC `unix_ts` was being passed to `offset_info_at_local`.
    pub(crate) fn gregorian_time_with_tz(&self, tz_name: &str) -> GregorianTime {
        let orig_clock_type = self.clock_type;

        // 1. Get the true UTC Unix timestamp (this is what we search with)
        let utc_unix = self.to_clock_type(ClockType::UTC).to_unix_sec();

        // 2. Look up offset + abbrev at that exact UTC instant
        let (offset_secs, abbrev) = match offset_info_at_utc(tz_name, utc_unix) {
            Some(info) => (info.offset, info.abbrev),
            None => (0, "UTC"), // fallback for unknown timezone
        };

        // 3. Build local time = UTC + offset
        let span = TimeSpan::new(offset_secs as i64, 0);
        let local_tp = self.to_clock_type(ClockType::UTC) + span;

        let mut gt = local_tp.to_gregorian_time();
        gt.set_offset(Some(offset_secs));
        gt.set_tz(Some(tz_name));
        gt.set_tz_abbrev(Some(abbrev));
        gt.set_clock_type(orig_clock_type);
        gt
    }
}
