use crate::{
    AsciiStr, ClockType, GregorianTime, STRFTIME_SIZE, TimePoint, TimeSpan, error::DtErrKind,
    tzdb::offset_info_at,
};

impl TimePoint {
    /// High-level alloc version (defaults to UTC label-only formatting).
    #[cfg(feature = "alloc")]
    #[inline(always)]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtErrKind> {
        self.to_str_with_offset_label(fmt, 0)
    }

    /// High-level alloc version with explicit offset (label-only).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_with_offset_label(
        &self,
        fmt: &str,
        label_in_secs: i32,
    ) -> Result<alloc::string::String, DtErrKind> {
        use crate::STRFTIME_SIZE;

        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_offset_label(fmt, &mut buf, label_in_secs)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// High-level alloc version for full IANA timezone formatting (with civil-time adjustment).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_str_adjusted_to_tz(
        &self,
        fmt: &str,
        tz_name: &str,
    ) -> Result<alloc::string::String, DtErrKind> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_adjusted_to_tz(fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// No-alloc label-only formatting.
    pub fn to_str_bin(&self, fmt: &str) -> Result<AsciiStr<STRFTIME_SIZE>, DtErrKind> {
        let mut gp = self.to_gregorian_time();
        gp.set_offset(Some(0)).set_tz_abbrev(None);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gp.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// No-alloc label-only formatting.
    pub fn to_str_bin_with_offset_label(
        &self,
        fmt: &str,
        label_in_secs: i32,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErrKind> {
        let mut gp = self.to_gregorian_time();
        gp.set_offset(Some(label_in_secs)).set_tz_abbrev(None);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gp.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// No-alloc full IANA adjusted formatting (civil time is adjusted to local wall time).
    pub fn to_str_bin_adjusted_to_tz(
        &self,
        fmt: &str,
        tz_name: &str,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErrKind> {
        let gp = self.gregorian_time_adjusted_to_tz(tz_name);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gp.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
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
    pub(crate) fn to_u8_with_offset_label(
        &self,
        fmt: &str,
        dest: &mut [u8],
        label_in_secs: i32,
    ) -> Result<usize, DtErrKind> {
        let mut gt = self.to_gregorian_time();
        gt.set_offset(Some(label_in_secs)).set_tz_abbrev(None);
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
    pub(crate) fn to_u8_adjusted_to_tz(
        &self,
        fmt: &str,
        dest: &mut [u8],
        tz_name: &str,
    ) -> Result<usize, DtErrKind> {
        let gp = self.gregorian_time_adjusted_to_tz(tz_name);
        let mut internal_buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gp.format_to_buffer(fmt.as_bytes(), &mut internal_buf, &mut pos)?;
        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Helper for creating a timezone adjusted GregorianTime.
    pub(crate) fn gregorian_time_adjusted_to_tz(&self, tz_name: &str) -> GregorianTime {
        let unix_ts = self.to_unix_sec();

        let (offset_secs, abbrev) = match offset_info_at(tz_name, unix_ts) {
            Some(info) => (info.offset, Some(info.abbrev)),
            None => (0, None),
        };

        let utc = self.to_clock_type(ClockType::UTC);
        let span = TimeSpan::new(offset_secs as i64, 0);
        let local_tp = utc + span;

        let mut gp = local_tp.to_gregorian_time();
        gp.set_offset(Some(offset_secs));
        gp.set_tz(Some(tz_name));
        gp.set_tz_abbrev(abbrev);
        gp
    }
}
