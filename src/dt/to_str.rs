use crate::{Dt, DtErr, LiteStr, STRFTIME_SIZE, YmdHmsRich, tz::offset_for_utc};

#[cfg(feature = "alloc")]
use crate::ATTOS_PER_SEC;

#[cfg(feature = "alloc")]
impl Dt {
    /// Converts this `Dt` to an ISO 8601 duration string
    /// (e.g. `"PT1H23M45.6789S"`, `"-PT0.5S"`, `"PT0.000000000000000001S"`, or `"PT0S"`).
    ///
    /// - This method is only available when the **`alloc`** feature is enabled.
    /// - It returns `alloc::string::String` (no_std + alloc compatible).
    /// - Performs no time scale conversions prior to output.
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
    /// Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    /// time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    /// let s = x.to_str("%F").unwrap();
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
    #[inline(always)]
    pub fn to_str(&self, fmt: &str) -> Result<alloc::string::String, DtErr> {
        self.to_str_with_offset(fmt, 0)
    }

    /// Formats this [`Dt`] into a String, applying a fixed UTC offset.  Requires the
    /// `"alloc"` feature.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// // offset of minus one hour
    /// let s = x.to_str_with_offset("%F", -3600).unwrap();
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
    #[inline(always)]
    pub fn to_str_with_offset(&self, fmt: &str, secs: i32) -> Result<alloc::string::String, DtErr> {
        self.ymdhms_rich_with_offset(secs).to_str(fmt)
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
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - No IANA timezone name or abbreviation is set.
    ///
    /// ## Examples
    ///
    /// ```
    /// # #[cfg(all(feature = "tz", feature = "parse"))]
    /// # {
    /// use deep_time::{Dt, Scale};
    ///
    /// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
    ///
    /// let s = x.to_str_with_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York").unwrap();
    ///
    /// assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
    /// # }
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
    #[inline(always)]
    pub fn to_str_with_tz(&self, fmt: &str, tz_name: &str) -> Result<alloc::string::String, DtErr> {
        self.ymdhms_rich_with_tz(tz_name).to_str(fmt)
    }

    /// Returns this instant as an **RFC 3339** / ISO 8601 timestamp with a
    /// `Z` suffix.
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Default = 9 digits (nanoseconds) but **automatically trims trailing zeros**.
    /// - If fractional part is zero → no decimal point at all (e.g. `...45Z`).
    /// - Example: `"2024-03-14T15:30:45.123Z"`
    #[inline(always)]
    pub fn to_str_rfc3339(&self) -> Result<String, DtErr> {
        self.to_str_rfc3339_nf(9)
    }

    /// Same as [`Dt::to_str_rfc3339`](../struct.Dt.html#method.to_str_rfc3339) but
    /// with a configurable maximum number of fractional digits (0–18). Trailing zeros are
    /// always trimmed.
    ///
    /// Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    /// time scale before producing the result.
    pub fn to_str_rfc3339_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        // Uses the formatter with the `~` "trim trailing zeros" flag.
        // The formatter already handles:
        //   - correct 4-digit years (with sign) for |yr| < 10000
        //   - full-width years otherwise
        //   - suppressing the decimal point entirely when the trimmed fraction is zero
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_with_offset(&fmt, 0)
    }

    /// **ISO 8601 / RFC 3339** with **actual offset** (modern `+00:00` style).
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Uses colon-separated offset (`%:z`) instead of forcing `Z`.
    /// - Still trims trailing zeros in the fractional part.
    /// - Example: `"2025-04-16T14:30:45.123+00:00"`
    #[inline(always)]
    pub fn to_str_iso8601(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%Y-%m-%dT%H:%M:%S%.~f%:z", 0)
    }

    /// **Compact ISO 8601 basic format** (no separators).
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Useful for filenames, URLs, database keys, etc.
    /// - Example: `"20250416T143045.123456789Z"`
    #[inline(always)]
    pub fn to_str_iso8601_basic(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%Y%m%dT%H%M%S%.~fZ", 0)
    }

    /// **HTTP-date** format (RFC 7231 / RFC 1123) — **always in GMT**.
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - This is the format used in `Date`, `Expires`, `Last-Modified` headers.
    /// - Example: `"Wed, 16 Apr 2025 14:30:45 GMT"`
    #[inline(always)]
    pub fn to_str_http(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%a, %d %b %Y %H:%M:%S GMT", 0)
    }

    /// **RFC 2822** date format (used in email `Date` headers).
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Example: `"Wed, 16 Apr 2025 14:30:45 +0000"`
    #[inline(always)]
    pub fn to_str_rfc2822(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%a, %d %b %Y %H:%M:%S %z", 0)
    }

    /// **ISO 8601 week date**.
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Example: `"2025-W16-3"` (year-week-day)
    #[inline(always)]
    pub fn to_str_iso_week_date(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%G-W%V-%u", 0)
    }

    /// Just the **ISO date** part (no time).
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Example: `"2025-04-16"`
    #[inline(always)]
    pub fn to_str_iso_date(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%Y-%m-%d", 0)
    }

    /// Just the **time** part with fractional seconds (trimmed).
    ///
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    /// - Example: `"14:30:45.123456789"`
    #[inline(always)]
    pub fn to_str_iso_time(&self) -> Result<String, DtErr> {
        self.to_str_with_offset("%H:%M:%S%.~f", 0)
    }
}

impl Dt {
    /// Formats this [`Dt`] into a fixed-size binary string.
    ///
    /// Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    /// time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    /// let b = x.to_str_lite("%F").unwrap();
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
    /// - [`Dt::to_str_lite_with_offset`](../struct.Dt.html#method.to_str_lite_with_offset)
    /// - [`Dt::to_str_lite_with_tz`](../struct.Dt.html#method.to_str_lite_with_tz)
    #[inline(always)]
    pub fn to_str_lite(&self, fmt: &str) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.to_ymd_rich().to_str_lite(fmt)
    }

    /// Formats this [`Dt`] into a fixed-size binary string, applying a fixed UTC offset.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - No IANA timezone name or abbreviation is set.
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// // offset of minus one hour
    /// let b = x.to_str_lite_with_offset("%F", -3600).unwrap();
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
    /// - [`Dt::to_str_lite`](../struct.Dt.html#method.to_str_lite)
    /// - [`Dt::to_str_lite_with_tz`](../struct.Dt.html#method.to_str_lite_with_tz)
    #[inline(always)]
    pub fn to_str_lite_with_offset(
        &self,
        fmt: &str,
        secs: i32,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.ymdhms_rich_with_offset(secs).to_str_lite(fmt)
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
    /// - No IANA timezone name or abbreviation is set.
    /// - Converts from the [`Dt`]s current time `scale` to the [`Dt`]s `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// let b = x.to_str_lite_with_tz("%F", "America/New_York").unwrap();
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
    /// - [`Dt::to_str_lite`](../struct.Dt.html#method.to_str_lite)
    /// - [`Dt::to_str_lite_with_offset`](../struct.Dt.html#method.to_str_lite_with_offset)
    #[inline(always)]
    pub fn to_str_lite_with_tz(
        &self,
        fmt: &str,
        tz_name: &str,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.ymdhms_rich_with_tz(tz_name).to_str_lite(fmt)
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
    pub(crate) fn ymdhms_rich_with_offset(&self, secs: i32) -> YmdHmsRich {
        let local_tp = if secs != 0 {
            self.add_sec(secs as i128)
        } else {
            *self
        };
        let mut ymdhms = local_tp.to_ymd_rich();
        ymdhms.set_offset(Some(secs));
        ymdhms
    }

    /// Helper for creating a timezone-adjusted YmdHmsRich.
    pub(crate) fn ymdhms_rich_with_tz(&self, tz_name: &str) -> YmdHmsRich {
        // Look up offset + abbrev at that exact UTC instant
        let unix_sec = self.to_unix().to_sec64();
        let (offset_secs, abbrev) = match offset_for_utc(tz_name, unix_sec) {
            Some(info) => (info.offset, info.abbrev),
            None => (0, "UTC"),
        };
        let local_tp = self.add_sec(offset_secs as i128);

        let mut ymdhms = local_tp.to_ymd_rich();
        ymdhms.set_offset(Some(offset_secs));
        ymdhms.set_tz(Some(tz_name));
        ymdhms.set_tz_abbrev(Some(abbrev));
        ymdhms
    }
}
