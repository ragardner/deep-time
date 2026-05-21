use crate::{AsciiStr, Dt, DtErr, STRFTIME_SIZE, Scale, YmdHmsRich, tzdb::offset_info_at_utc};

#[cfg(feature = "alloc")]
use crate::ATTOS_PER_SEC;

#[cfg(feature = "alloc")]
impl Dt {
    /// Converts this `Dt` to an ISO 8601 duration string
    /// (e.g. `"PT1H23M45.6789S"`, `"-PT0.5S"`, `"PT0.000000000000000001S"`, or `"PT0S"`).
    ///
    /// - This method is only available when the **`alloc`** feature is enabled.
    /// - It returns `alloc::string::String` (no_std + alloc compatible).
    pub fn to_iso_duration(&self) -> alloc::string::String {
        if self.is_zero() {
            return alloc::string::String::from("PT0S");
        }

        let total = self.to_attos();
        let negative = total < 0;
        let mut attos = total.unsigned_abs();

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

    /// High-level allocating formatter (defaults to UTC / label-only).
    ///
    /// Equivalent to [`Self::to_str_with_offset`] with `secs = 0`.
    ///
    /// This is the convenient `alloc` version of [`Self::to_str_bin`].
    #[inline]
    pub fn to_str(&self, current: Scale, fmt: &str) -> Result<alloc::string::String, DtErr> {
        self.to_str_with_offset(current, fmt, 0)
    }

    /// High-level allocating formatter with a fixed UTC offset.
    ///
    /// - The civil time is adjusted by the given offset before formatting,
    /// and `%z`/`%:z` directives will reflect that offset.
    /// - IANA name/abbreviation are **not** set.
    ///
    /// This is the convenient `alloc` version of [`Self::to_str_bin_with_offset`].
    #[inline]
    pub fn to_str_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        secs: i32,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_offset(current, fmt, &mut buf, secs)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// High-level allocating formatter with full IANA timezone support
    /// (Jiff-compatible directive behavior).
    ///
    /// Performs a correct UTC-based IANA lookup so that the following
    /// directives produce accurate results:
    ///
    /// - `%Q` / `%:Q` → full IANA timezone name (e.g. `America/New_York`)
    /// - `%Z` → timezone abbreviation (e.g. `EDT`). These **do not** round-trip.
    /// - `%z` / `%:z` → numeric offset
    ///
    /// This is the convenient `alloc` version of [`Self::to_str_bin_with_tz`].
    #[inline]
    pub fn to_str_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        tz_name: &str,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self.to_u8_with_tz(current, fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }
}

impl Dt {
    /// Formats this `Dt` into a fixed-size ASCII string **without** any heap allocation.
    ///
    /// The Dt is first converted to Gregorian civil time on the given
    /// `current` scale, **UTC** (offset = 0, no timezone abbreviation or IANA
    /// name). This is the simplest no-alloc formatter.
    ///
    /// # Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    pub fn to_str_bin(&self, current: Scale, fmt: &str) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let mut gt = self.to_ymdhms_rich_on(current, current.to_utc());
        gt.set_offset(Some(0)).set_tz_abbrev(None);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// Formats this `Dt` into a fixed-size ASCII string **without** any heap allocation,
    /// applying a fixed UTC offset.
    ///
    /// - The civil time is adjusted by the given `secs` offset **before** formatting,
    /// and the offset is stored so that `%z` / `%:z` directives will reflect it.
    /// - No IANA timezone name or abbreviation is set.
    pub fn to_str_bin_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        secs: i32,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.date_time_with_offset(current, secs);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// Formats this `Dt` into a fixed-size ASCII string **without** any heap allocation,
    /// adjusting to the given IANA timezone.
    ///
    /// This performs a correct UTC-based lookup in the IANA transition table,
    /// so the resulting `YmdHmsRich` contains:
    /// - accurate civil time
    /// - correct numeric offset (for `%z` / `%:z`)
    /// - timezone abbreviation (for `%Z`). These **do not** round-trip.
    /// - full IANA timezone name (for `%Q` / `%:Q`)
    ///
    /// Use this method when you want full IANA-aware formatting (`%Q`, `%Z`,
    /// `%z`, etc.).
    pub fn to_str_bin_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        tz_name: &str,
    ) -> Result<AsciiStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.date_time_with_tz(current, tz_name);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(AsciiStr::from_filled_buffer(buf))
    }

    /// Low-level no-alloc formatter that writes into a caller-provided slice,
    /// using a fixed UTC offset.
    ///
    /// Same logic as [`Self::to_str_bin_with_offset`], but writes directly into
    /// `dest` (truncated to `dest.len()`) and returns the number of bytes written.
    pub fn to_u8_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        dest: &mut [u8],
        secs: i32,
    ) -> Result<usize, DtErr> {
        let gt = self.date_time_with_offset(current, secs);
        let mut internal_buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut internal_buf, &mut pos)?;
        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Low-level no-alloc formatter that writes into a caller-provided slice,
    /// using a full IANA timezone.
    ///
    /// Same logic as [`Self::to_str_bin_with_tz`], but writes directly into
    /// `dest` (truncated to `dest.len()`) and returns the number of bytes written.
    pub fn to_u8_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        dest: &mut [u8],
        tz_name: &str,
    ) -> Result<usize, DtErr> {
        let gt = self.date_time_with_tz(current, tz_name);
        let mut internal_buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut internal_buf, &mut pos)?;
        let written = pos.min(dest.len());
        if written > 0 {
            dest[0..written].copy_from_slice(&internal_buf[0..written]);
        }
        Ok(written)
    }

    /// Returns `(is_negative, hours, minutes)`.
    #[inline]
    pub(crate) const fn sec_as_hhmm(seconds: i32) -> (bool, u8, u8) {
        let total = seconds.saturating_abs();
        let hours = (total / 3600) as u8;
        let minutes = ((total % 3600) / 60) as u8;
        (seconds < 0, hours, minutes)
    }

    /// Helper for creating an offset adjusted YmdHmsRich.
    pub(crate) fn date_time_with_offset(&self, current: Scale, secs: i32) -> YmdHmsRich {
        let local_tp = if secs != 0 {
            *self + Dt::new(secs as i64, 0)
        } else {
            *self
        };
        let mut gt = local_tp.to_ymdhms_rich_on(current, current.to_utc());
        gt.set_offset(Some(secs));
        gt
    }

    /// Helper for creating a timezone-adjusted YmdHmsRich.
    ///
    /// Always converts to UTC first, then does a correct UTC-based lookup
    /// in the IANA transition table. This avoids the previous bug where
    /// a non-UTC `unix_ts` was being passed to `offset_info_at_local`.
    pub(crate) fn date_time_with_tz(&self, current: Scale, tz_name: &str) -> YmdHmsRich {
        // 1. Get the true UTC Unix timestamp (this is what we search with)
        let utc_unix = self
            .to(current, current.to_utc())
            .to_diff_raw(Dt::UNIX_EPOCH);

        // 2. Look up offset + abbrev at that exact UTC instant
        let (offset_secs, abbrev) = match offset_info_at_utc(tz_name, utc_unix.sec) {
            Some(info) => (info.offset, info.abbrev),
            None => (0, "UTC"), // fallback for unknown timezone
        };

        // 3. Build local time = UTC + offset
        let span = Dt::new(offset_secs as i64, 0);
        let local_tp = *self + span;

        let mut gt = local_tp.to_ymdhms_rich_on(current, current.to_utc());
        gt.set_offset(Some(offset_secs));
        gt.set_tz(Some(tz_name));
        gt.set_tz_abbrev(Some(abbrev));
        gt
    }
}
