use crate::{Dt, DtErr, DtErrKind, Lang, LiteStr, STRFTIME_SIZE, YmdHms, an_err};

#[cfg(feature = "alloc")]
use {crate::ATTOS_PER_SEC, alloc::string::String};

#[cfg(not(feature = "jiff-tz"))]
use crate::tz::UTC_ALIASES;

#[cfg(feature = "alloc")]
impl Dt {
    /// Converts this `Dt` to an ISO 8601 duration string
    /// (e.g. `"PT1H23M45.6789S"`, `"-PT0.5S"`, `"PT0.000000000000000001S"`, or `"PT0S"`).
    ///
    /// - This method is only available when the **`alloc`** feature is enabled.
    /// - It returns `alloc::string::String` (no_std + alloc compatible).
    /// - Performs no time scale conversions prior to output.
    pub fn to_iso_duration(&self) -> String {
        if self.is_zero() {
            return String::from("PT0S");
        }

        let total = self.to_attos();
        let negative = total < 0;
        let mut attos = total.unsigned_abs();

        let mut s = String::with_capacity(48);
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
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    /// let s = x.to_str("%F", Lang::En).unwrap();
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
    /// - [`Dt::to_str_in_offset`](../struct.Dt.html#method.to_str_in_offset)
    /// - [`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz)
    #[inline(always)]
    pub fn to_str(&self, fmt: &str, lang: Lang) -> Result<String, DtErr> {
        self.to_str_in_offset(fmt, 0, lang)
    }

    /// Formats this [`Dt`] into a String, applying a fixed offset. Requires the
    /// `"alloc"` feature.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - No IANA timezone name or abbreviation is set.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// // offset of minus one hour
    /// let s = x.to_str_in_offset("%F", -3600, Lang::En).unwrap();
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
    /// - [`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz)
    #[inline(always)]
    pub fn to_str_in_offset(&self, fmt: &str, secs: i32, lang: Lang) -> Result<String, DtErr> {
        self.ymd_with_offset(secs)
            .to_str(fmt, Some(secs), None, None, lang)
    }

    /// Formats this [`Dt`] into a string, time adjusted to the given IANA timezone. Requires
    /// the `"alloc"` feature.
    ///
    /// Use this method when you want full IANA-aware formatting (`%Q`, `%Z`, `%z`, etc.).
    ///
    /// - A copy of the [`Dt`] is adjusted by the offset at the [`Dt`]'s time for the given
    ///   IANA timezone. This is so that the formatter will have:
    ///     - Accurate wall time for the timezone.
    ///     - Correct numeric offset (for `%z` / `%:z`).
    ///     - Timezone abbreviation (for `%Z`). These **do not** round-trip (the parser
    ///       does not parse them).
    ///     - Full IANA timezone name (for `%Q` / `%:Q`).
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// You can offset an output that wasn't originally from a zoned input:
    ///
    /// ```rust
    /// # #[cfg(all(feature = "tz", feature = "parse"))]
    /// # {
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x: Dt = "2000-01-01 12:00:00".parse().unwrap();
    /// let s = x.to_str_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En).unwrap();
    /// assert_eq!(s, "Saturday, January 01, 2000 07:00:00 America/New_York");
    /// # }
    /// ```
    ///
    /// You can also return to a zoned output from a zoned input:
    ///
    /// ```rust
    /// # #[cfg(all(feature = "tz", feature = "parse"))]
    /// # {
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x: Dt = "Saturday, January 01, 2000 07:00:00 America/New_York".parse().unwrap();
    /// let s = x.to_str_in_tz("%A, %B %d, %Y %H:%M:%S %Q", "America/New_York", Lang::En).unwrap();
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
    /// - [`Dt::to_str_in_offset`](../struct.Dt.html#method.to_str_in_offset)
    #[inline(always)]
    pub fn to_str_in_tz(&self, fmt: &str, tz_name: &str, lang: Lang) -> Result<String, DtErr> {
        let (ymd, offset, abbrev) = self.ymd_with_tz(tz_name, true)?;
        ymd.to_str(
            fmt,
            Some(offset),
            Some(LiteStr::new(tz_name)),
            Some(abbrev),
            lang,
        )
    }

    /// **RFC 9557** / Temporal format with IANA timezone name in brackets.
    ///
    /// - Automatically trims trailing zeros in the fractional part.
    /// - Example: `"2020-06-15T14:30:00-04:00[America/New_York]"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_rfc9557(&self, tz_name: &str) -> Result<String, DtErr> {
        self.to_str_in_tz("%Y-%m-%dT%H:%M:%S%.~f%:z[%Q]", tz_name, Lang::En)
    }

    /// Returns this instant as an **RFC 3339** / ISO 8601 timestamp with a
    /// `Z` suffix.
    ///
    /// - Default = 9 digits (nanoseconds) but **automatically trims trailing zeros**.
    /// - If fractional part is zero → no decimal point at all (e.g. `...45Z`).
    /// - Example: `"2024-03-14T15:30:45.123Z"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_rfc3339(&self) -> Result<String, DtErr> {
        self.to_str_rfc3339_nf(9)
    }

    /// Same as [`Dt::to_str_rfc3339`](../struct.Dt.html#method.to_str_rfc3339) but
    /// with a configurable maximum number of fractional digits (0–18). Trailing zeros are
    /// always trimmed.
    ///
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    pub fn to_str_rfc3339_nf(&self, max_precision: usize) -> Result<String, DtErr> {
        let prec = max_precision.min(18);
        // Uses the formatter with the `~` "trim trailing zeros" flag.
        // The formatter already handles:
        //   - correct 4-digit years (with sign) for |yr| < 10000
        //   - full-width years otherwise
        //   - suppressing the decimal point entirely when the trimmed fraction is zero
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_in_offset(&fmt, 0, Lang::En)
    }

    /// **ISO 8601 / RFC 3339** with **actual offset** (modern `+00:00` style).
    ///
    /// - Uses colon-separated offset (`%:z`) instead of forcing `Z`.
    /// - Still trims trailing zeros in the fractional part.
    /// - Example: `"2025-04-16T14:30:45.123+00:00"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_iso8601(&self) -> Result<String, DtErr> {
        self.to_str_in_offset("%Y-%m-%dT%H:%M:%S%.~f%:z", 0, Lang::En)
    }

    /// **Compact ISO 8601 basic format** (no separators).
    ///
    /// - Example: `"20250416T143045.123456789Z"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_iso8601_basic(&self) -> Result<String, DtErr> {
        self.to_str_in_offset("%Y%m%dT%H%M%S%.~fZ", 0, Lang::En)
    }

    /// **ISO 8601 week date**.
    ///
    /// - Example: `"2025-W16-3"` (year-week-day)
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_iso_week_date(&self) -> Result<String, DtErr> {
        self.to_str_in_offset("%G-W%V-%u", 0, Lang::En)
    }

    /// Just the **ISO date** part (no time).
    ///
    /// - Example: `"2025-04-16"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_iso_date(&self) -> Result<String, DtErr> {
        self.to_str_in_offset("%Y-%m-%d", 0, Lang::En)
    }

    /// Just the **time** part with fractional seconds (trimmed).
    ///
    /// - Example: `"14:30:45.123456789"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_iso_time(&self) -> Result<String, DtErr> {
        self.to_str_in_offset("%H:%M:%S%.~f", 0, Lang::En)
    }

    /// **HTTP-date** format (RFC 7231 / RFC 1123) — **always in GMT**.
    ///
    /// - Example: `"Wed, 16 Apr 2025 14:30:45 GMT"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_http(&self, lang: Lang) -> Result<String, DtErr> {
        self.to_str_in_offset("%a, %d %b %Y %H:%M:%S GMT", 0, lang)
    }

    /// **RFC 2822** date format (used in email `Date` headers).
    ///
    /// - Example: `"Wed, 16 Apr 2025 14:30:45 +0000"`
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    #[inline(always)]
    pub fn to_str_rfc2822(&self, lang: Lang) -> Result<String, DtErr> {
        self.to_str_in_offset("%a, %d %b %Y %H:%M:%S %z", 0, lang)
    }

    /// Formats this [`Dt`] into a `String`, attaching an offset **as a label only**.
    ///
    /// - The actual datetime components are **not** shifted or adjusted.
    /// - The given `offset` is used **only** for `%z` / `%:z` format directives.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_in_offset`](../struct.Dt.html#method.to_str_in_offset) —
    ///   shifts the datetime by the offset
    #[inline(always)]
    pub fn to_str_with_offset_label(
        &self,
        fmt: &str,
        offset: i32,
        lang: Lang,
    ) -> Result<String, DtErr> {
        self.to_ymd().to_str(fmt, Some(offset), None, None, lang)
    }

    /// Formats this [`Dt`] into a `String`, attaching a timezone **as a label only**.
    ///
    /// - The actual datetime components are **not** shifted or adjusted.
    /// - The timezone is used to provide correct values for `%z`, `%:z`, `%Z`, `%Q`, and `%:Q`.
    /// - The timezone abbreviation is automatically looked up from tzdata.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers,
    /// if the timezone name is invalid, or if the internal formatting buffer
    /// overflows (extremely unlikely with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_in_tz`](../struct.Dt.html#method.to_str_in_tz) —
    ///   shifts the datetime into the given timezone
    #[inline(always)]
    pub fn to_str_with_tz_label(
        &self,
        fmt: &str,
        tz_name: &str,
        lang: Lang,
    ) -> Result<String, DtErr> {
        let (ymd, offset, abbrev) = self.ymd_with_tz(tz_name, false)?;
        ymd.to_str(
            fmt,
            Some(offset),
            Some(LiteStr::new(tz_name)),
            Some(abbrev),
            lang,
        )
    }
}

impl Dt {
    /// Formats this [`Dt`] into a fixed-size binary string.
    ///
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    /// let b = x.to_str_lite("%F", Lang::En).unwrap();
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
    /// - [`Dt::to_str_lite_in_offset`](../struct.Dt.html#method.to_str_lite_in_offset)
    /// - [`Dt::to_str_lite_in_tz`](../struct.Dt.html#method.to_str_lite_in_tz)
    #[inline(always)]
    pub fn to_str_lite(&self, fmt: &str, lang: Lang) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.to_ymd().to_str_lite(fmt, None, None, None, lang)
    }

    /// Formats this [`Dt`] into a fixed-size binary string, applying a fixed UTC offset.
    ///
    /// - A copy of the [`Dt`] is adjusted by the given `secs` offset **before**
    ///   formatting, and the offset is stored so that `%z` / `%:z` format directives
    ///   will reflect it.
    /// - No IANA timezone name or abbreviation is set.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// // offset of minus one hour
    /// let b = x.to_str_lite_in_offset("%F", -3600, Lang::En).unwrap();
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
    /// - [`Dt::to_str_lite_in_tz`](../struct.Dt.html#method.to_str_lite_in_tz)
    #[inline(always)]
    pub fn to_str_lite_in_offset(
        &self,
        fmt: &str,
        secs: i32,
        lang: Lang,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.ymd_with_offset(secs)
            .to_str_lite(fmt, Some(secs), None, None, lang)
    }

    /// Formats this [`Dt`] into a fixed-size binary string, time adjusted to the given
    /// IANA timezone.
    ///
    /// Use this method when you want full IANA-aware formatting (`%Q`, `%Z`, `%z`, etc.).
    ///
    /// - A copy of the [`Dt`] is adjusted by the offset at the [`Dt`]'s time for the given
    ///   IANA timezone. This is so that the formatter will have:
    ///     - Accurate wall time for the timezone.
    ///     - Correct numeric offset (for `%z` / `%:z`).
    ///     - Timezone abbreviation (for `%Z`). These **do not** round-trip.
    ///     - Full IANA timezone name (for `%Q` / `%:Q`).
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "tz")]
    /// # {
    /// use deep_time::{Dt, Lang, Scale};
    ///
    /// let x = Dt::from_ymd(2000, 1, 1, 0, 0, 0, 0, Scale::UTC);
    ///
    /// let b = x.to_str_lite_in_tz("%F", "America/New_York", Lang::En).unwrap();
    /// let s = b.as_str().unwrap();
    ///
    /// println!("{}", s);
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
    /// - [`Dt::to_str_lite`](../struct.Dt.html#method.to_str_lite)
    /// - [`Dt::to_str_lite_in_offset`](../struct.Dt.html#method.to_str_lite_in_offset)
    #[inline(always)]
    pub fn to_str_lite_in_tz(
        &self,
        fmt: &str,
        tz_name: &str,
        lang: Lang,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let (ymd, offset, abbrev) = self.ymd_with_tz(tz_name, true)?;
        ymd.to_str_lite(
            fmt,
            Some(offset),
            Some(LiteStr::new(tz_name)),
            Some(abbrev),
            lang,
        )
    }

    /// Formats this [`Dt`] into a `LiteStr`, attaching an offset **as a label only**.
    ///
    /// - The actual datetime components are **not** shifted or adjusted.
    /// - The given `offset` is used **only** for `%z` / `%:z` format directives.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers
    /// or if the internal formatting buffer overflows (extremely unlikely
    /// with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_lite_in_offset`](../struct.Dt.html#method.to_str_lite_in_offset) —
    ///   shifts the datetime by the offset
    #[inline(always)]
    pub fn to_str_lite_with_offset_label(
        &self,
        fmt: &str,
        offset: i32,
        lang: Lang,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        self.to_ymd()
            .to_str_lite(fmt, Some(offset), None, None, lang)
    }

    /// Formats this [`Dt`] into a `LiteStr`, attaching a timezone **as a label only**.
    ///
    /// - The actual datetime components are **not** shifted or adjusted.
    /// - The timezone is used to provide correct values for `%z`, `%:z`, `%Z`, `%Q`, and `%:Q`.
    /// - The timezone abbreviation is automatically looked up from tzdata.
    /// - Converts from this [`Dt`]'s current time `scale` to its `target`
    ///   time scale before producing the result.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] if the format string contains invalid specifiers,
    /// if the timezone name is invalid, or if the internal formatting buffer
    /// overflows (extremely unlikely with [`STRFTIME_SIZE`]).
    ///
    /// ## See also
    ///
    /// - [`Dt::to_str_lite_in_tz`](../struct.Dt.html#method.to_str_lite_in_tz) —
    ///   shifts the datetime into the given timezone
    #[inline(always)]
    pub fn to_str_lite_with_tz_label(
        &self,
        fmt: &str,
        tz_name: &str,
        lang: Lang,
    ) -> Result<LiteStr<STRFTIME_SIZE>, DtErr> {
        let (ymd, offset, abbrev) = self.ymd_with_tz(tz_name, false)?;
        ymd.to_str_lite(
            fmt,
            Some(offset),
            Some(LiteStr::new(tz_name)),
            Some(abbrev),
            lang,
        )
    }

    /// Returns `(is_negative, hours, minutes)`.
    #[inline]
    pub(crate) const fn sec_as_hhmm(seconds: i32) -> (bool, u8, u8) {
        let total = seconds.saturating_abs();
        let hours = (total / 3600) as u8;
        let minutes = ((total % 3600) / 60) as u8;
        (seconds < 0, hours, minutes)
    }

    pub(crate) fn ymd_with_offset(&self, secs: i32) -> YmdHms {
        if secs != 0 {
            self.add_sec(secs as i128).to_ymd()
        } else {
            self.to_ymd()
        }
    }

    pub(crate) fn ymd_with_tz(
        &self,
        tz_name: &str,
        apply_offset: bool,
    ) -> Result<(YmdHms, i32, LiteStr<49>), DtErr> {
        #[cfg(feature = "jiff-tz")]
        let (offset_secs, abbrev): (i32, LiteStr<49>) = {
            use jiff::{Timestamp, tz::TimeZone};

            let unix_sec = self.to_unix().to_sec64();

            let ts = Timestamp::from_second(unix_sec).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidNumber,
                    "invalid unix {:?} for jiff Timestamp: {}",
                    unix_sec,
                    e
                )
            })?;

            let tz = TimeZone::get(tz_name).map_err(|e| {
                an_err!(
                    DtErrKind::InvalidTimezoneOffset,
                    "invalid tz {:?}: {}",
                    tz_name,
                    e
                )
            })?;

            let info = tz.to_offset_info(ts);
            let offset_secs = info.offset().seconds();
            let abbrev: LiteStr<49> = LiteStr::new(info.abbreviation());

            (offset_secs, abbrev)
        };

        #[cfg(not(feature = "jiff-tz"))]
        let (offset_secs, abbrev): (i32, LiteStr<49>) = {
            if !UTC_ALIASES.contains(&tz_name) {
                return Err(an_err!(
                    DtErrKind::InvalidBytes,
                    "non-utc tz: {} requires jiff-tz feature",
                    tz_name,
                ));
            }
            // UTC → offset 0, canonical abbrev "UTC"
            let abbrev: LiteStr<49> = LiteStr::new("UTC");
            (0i32, abbrev)
        };

        let ymd = if offset_secs != 0 && apply_offset {
            self.add_sec(offset_secs as i128).to_ymd()
        } else {
            self.to_ymd()
        };

        Ok((ymd, offset_secs, abbrev))
    }
}
