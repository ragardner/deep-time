use crate::{
    Dt, DtErr, DtErrKind, LiteStr, STRFTIME_SIZE, Scale, YmdHmsRich, an_err,
    tzdb::offset_info_at_utc,
};

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

    /// Formats this [`Dt`] into a String. Requires the `"alloc"` feature.
    ///
    /// - It is first converted from the `current` [`Scale`] into
    ///   the `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice`
    ///   are preserved. See [`Scale`] for more info on time scales.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    /// let s = x.to_str(Scale::TAI, "%F").unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_with_offset`](../struct.Dt.html#method.to_str_with_offset)
    /// - [`Dt::to_str_with_tz`](../struct.Dt.html#method.to_str_with_tz)
    #[inline]
    pub fn to_str(&self, current: Scale, fmt: &str) -> Result<alloc::string::String, DtErr> {
        self.to_str_with_offset(current, fmt, 0)
    }

    /// Formats this [`Dt`] into a String, applying a fixed UTC offset.  Requires the
    /// `"alloc"` feature.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - Then it's converted from the `current` [`Scale`] into the
    ///   `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice` are preserved.
    ///   See [`Scale`] for more info on time scales.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    ///
    /// // offset of minus one hour
    /// let s = x.to_str_with_offset(Scale::TAI, "%F", -3600).unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str`](../struct.Dt.html#method.to_str)
    /// - [`Dt::to_str_with_tz`](../struct.Dt.html#method.to_str_with_tz)
    #[inline]
    pub fn to_str_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        secs: i32,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self._to_u8_with_offset(current, fmt, &mut buf, secs)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }

    /// Formats this [`Dt`] into a string, time adjusted to the given IANA timezone. Requires
    /// the `"alloc"` feature.
    ///
    /// Use this method when you want full IANA-aware formatting (`%Q`, `%Z`, `%z`, etc.).
    ///
    /// - A copy of the [`Dt`] is adjusted by the offset at the [`Dt`]s time for the given
    ///   IANA timezone. This is so that the formatter will have:
    ///     - Accurate wall time for the timezone.
    ///     - Correct numeric offset (for `%z` / `%:z`).
    ///     - Timezone abbreviation (for `%Z`). These **do not** round-trip.
    ///     - Full IANA timezone name (for `%Q` / `%:Q`).
    /// - Then it's converted from the `current` [`Scale`] into the
    ///   `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice` are preserved.
    ///   See [`Scale`] for more info on time scales.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    /// let s = x.to_str_with_tz(Scale::TAI, "%F", "America/New_York").unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str`](../struct.Dt.html#method.to_str)
    /// - [`Dt::to_str_with_offset`](../struct.Dt.html#method.to_str_with_offset)
    #[inline]
    pub fn to_str_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        tz_name: &str,
    ) -> Result<alloc::string::String, DtErr> {
        let mut buf = [0u8; STRFTIME_SIZE];
        let n = self._to_u8_with_tz(current, fmt, &mut buf, tz_name)?;
        Ok(alloc::string::String::from_utf8_lossy(&buf[0..n]).into_owned())
    }
}

impl Dt {
    /// Formats this [`Dt`] into a fixed-size binary string.
    ///
    /// - It is first converted from the `current` [`Scale`] into
    ///   the `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice`
    ///   are preserved. See [`Scale`] for more info on time scales.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    /// let b = x.to_str_bin(Scale::TAI, "%F").unwrap();
    /// let s = b.as_str().unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_bin_with_offset`](../struct.Dt.html#method.to_str_bin_with_offset)
    /// - [`Dt::to_str_bin_with_tz`](../struct.Dt.html#method.to_str_bin_with_tz)
    pub fn to_str_bin(&self, current: Scale, fmt: &str) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let mut gt = self.to_ymdhms_rich_on(current, current.to_utc());
        gt.set_offset(Some(0)).set_tz_abbrev(None);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(LiteStr::from_bytes(&buf).map_err(|_| an_err!(DtErrKind::InvalidBytes))?)
    }

    /// Formats this [`Dt`] into a fixed-size binary string, applying a fixed UTC offset.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - Then it's converted from the `current` [`Scale`] into the
    ///   `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice` are preserved.
    ///   See [`Scale`] for more info on time scales.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    ///
    /// // offset of minus one hour
    /// let b = x.to_str_bin_with_offset(Scale::TAI, "%F", -3600).unwrap();
    /// let s = b.as_str().unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_bin`](../struct.Dt.html#method.to_str_bin)
    /// - [`Dt::to_str_bin_with_tz`](../struct.Dt.html#method.to_str_bin_with_tz)
    pub fn to_str_bin_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        secs: i32,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.ymdhms_rich_with_offset(current, secs);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(LiteStr::from_bytes(&buf).map_err(|_| an_err!(DtErrKind::InvalidBytes))?)
    }

    /// Formats this [`Dt`] into a fixed-size binary string, time adjusted to the given
    /// IANA timezone.
    ///
    /// Use this method when you want full IANA-aware formatting (`%Q`, `%Z`, `%z`, etc.).
    ///
    /// - A copy of the [`Dt`] is adjusted by the offset at the [`Dt`]s time for the given
    ///   IANA timezone. This is so that the formatter will have:
    ///     - Accurate wall time for the timezone.
    ///     - Correct numeric offset (for `%z` / `%:z`).
    ///     - Timezone abbreviation (for `%Z`). These **do not** round-trip.
    ///     - Full IANA timezone name (for `%Q` / `%:Q`).
    /// - Then it's converted from the `current` [`Scale`] into the
    ///   `UTC` time scale.
    /// - Historical UTC time scales such as `UTCSofa` and `UTCSpice` are preserved.
    ///   See [`Scale`] for more info on time scales.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1);
    ///
    /// let b = x.to_str_bin_with_tz(Scale::TAI, "%F", "America/New_York").unwrap();
    /// let s = b.as_str().unwrap();
    ///
    /// println!("{}", s);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_bin`](../struct.Dt.html#method.to_str_bin)
    /// - [`Dt::to_str_bin_with_offset`](../struct.Dt.html#method.to_str_bin_with_offset)
    pub fn to_str_bin_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        tz_name: &str,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let gt = self.ymdhms_rich_with_tz(current, tz_name);
        let mut buf = [0u8; STRFTIME_SIZE];
        let mut pos = 0usize;
        gt.format_to_buffer(fmt.as_bytes(), &mut buf, &mut pos)?;
        Ok(LiteStr::from_bytes(&buf).map_err(|_| an_err!(DtErrKind::InvalidBytes))?)
    }

    /// Low-level no-alloc formatter that writes into a caller-provided slice,
    /// using a fixed UTC offset.
    ///
    /// Same logic as [`Self::to_str_bin_with_offset`], but writes directly into
    /// `dest` (truncated to `dest.len()`) and returns the number of bytes written.
    pub fn _to_u8_with_offset(
        &self,
        current: Scale,
        fmt: &str,
        dest: &mut [u8],
        secs: i32,
    ) -> Result<usize, DtErr> {
        let gt = self.ymdhms_rich_with_offset(current, secs);
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
    pub fn _to_u8_with_tz(
        &self,
        current: Scale,
        fmt: &str,
        dest: &mut [u8],
        tz_name: &str,
    ) -> Result<usize, DtErr> {
        let gt = self.ymdhms_rich_with_tz(current, tz_name);
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
    pub(crate) fn ymdhms_rich_with_offset(&self, current: Scale, secs: i32) -> YmdHmsRich {
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
    pub(crate) fn ymdhms_rich_with_tz(&self, current: Scale, tz_name: &str) -> YmdHmsRich {
        // 1. Get the true UTC Unix timestamp
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
