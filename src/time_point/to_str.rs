use crate::{
    AsciiStr, ClockType, DtError, GregorianTime, STRFTIME_SIZE, TimePoint, TimeSpan,
    tzdb::offset_info_at,
};

impl TimePoint {
    /// High-level alloc version (defaults to UTC label-only formatting).
    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtError> {
        self.to_str_with_offset(fmt, 0)
    }

    /// High-level alloc version with explicit offset (label-only).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_offset(
        &self,
        fmt: &str,
        secs: i32,
    ) -> Result<alloc::string::String, DtError> {
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
    ) -> Result<alloc::string::String, DtError> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_tz(fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// No-alloc label-only formatting.
    pub fn to_str_bin(&self, fmt: &str) -> Result<AsciiStr<STRFTIME_SIZE>, DtError> {
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
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtError> {
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
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtError> {
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

    /// Helper for strftime.
    pub(crate) fn to_u8_with_offset(
        &self,
        fmt: &str,
        dest: &mut [u8],
        secs: i32,
    ) -> Result<usize, DtError> {
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

    /// Helper for strftime.
    pub(crate) fn to_u8_with_tz(
        &self,
        fmt: &str,
        dest: &mut [u8],
        tz_name: &str,
    ) -> Result<usize, DtError> {
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

    /// Helper for creating a timezone adjusted GregorianTime.
    pub(crate) fn gregorian_time_with_tz(&self, tz_name: &str) -> GregorianTime {
        let orig_clock_type = self.clock_type;
        let unix_ts = self.to_unix_sec();

        let (offset_secs, abbrev) = match offset_info_at(tz_name, unix_ts) {
            Some(info) => (info.offset, Some(info.abbrev)),
            None => (0, None),
        };

        let utc = self.to_clock_type(ClockType::UTC);
        let span = TimeSpan::new(offset_secs as i64, 0);
        let local_tp = utc + span;

        let mut gt = local_tp.to_gregorian_time();
        gt.set_offset(Some(offset_secs));
        gt.set_tz(Some(tz_name));
        gt.set_tz_abbrev(abbrev);
        gt.set_clock_type(orig_clock_type);
        gt
    }
}
