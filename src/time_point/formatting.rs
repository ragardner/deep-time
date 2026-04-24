use crate::TimePoint;
use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "alloc")]
impl TimePoint {
    /// Returns this instant as an **RFC 3339** / ISO 8601 timestamp in **UTC**
    /// with the `Z` suffix.
    ///
    /// - Always uses UTC (`Z` = Zulu = UTC).
    /// - Default = 9 digits (nanoseconds) but **automatically trims trailing zeros**.
    /// - If fractional part is zero → no decimal point at all (e.g. `...45Z`).
    /// - Example: `"2024-03-14T15:30:45.123Z"`
    #[inline(always)]
    pub fn to_str_rfc3339(&self) -> String {
        self.to_str_rfc3339_nf(9)
    }

    /// Same as [`to_str_rfc3339`] but with a configurable maximum number of fractional digits
    /// (0–18). Trailing zeros are always trimmed.
    pub fn to_str_rfc3339_nf(&self, max_precision: usize) -> String {
        let prec = max_precision.min(18);
        // Uses the new formatter with the `~` "trim trailing zeros" flag.
        // The formatter already handles:
        //   - correct 4-digit years (with sign) for |yr| < 10000
        //   - full-width years otherwise
        //   - suppressing the decimal point entirely when the trimmed fraction is zero
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_with_offset_label(&fmt, 0)
            .expect("RFC 3339 formatting should never fail")
    }

    /// **ISO 8601 / RFC 3339** with **actual offset** (modern `+00:00` style).
    ///
    /// - Uses colon-separated offset (`%:z`) instead of forcing `Z`.
    /// - Still trims trailing zeros in the fractional part.
    /// - Example: `"2025-04-16T14:30:45.123+00:00"`
    #[inline]
    pub fn to_str_iso8601(&self) -> String {
        self.to_str_with_offset_label("%Y-%m-%dT%H:%M:%S%.~f%:z", 0)
            .expect("ISO 8601 formatting should never fail")
    }

    /// **Compact ISO 8601 basic format** (no separators).
    ///
    /// - Useful for filenames, URLs, database keys, etc.
    /// - Example: `"20250416T143045.123456789Z"`
    #[inline]
    pub fn to_str_iso8601_basic(&self) -> String {
        self.to_str_with_offset_label("%Y%m%dT%H%M%S%.~fZ", 0)
            .expect("ISO 8601 basic formatting should never fail")
    }

    /// **HTTP-date** format (RFC 7231 / RFC 1123) — **always in GMT**.
    ///
    /// This is the format used in `Date`, `Expires`, `Last-Modified` headers.
    /// Example: `"Wed, 16 Apr 2025 14:30:45 GMT"`
    #[inline]
    pub fn to_str_http(&self) -> String {
        self.to_str_with_offset_label("%a, %d %b %Y %H:%M:%S GMT", 0)
            .expect("HTTP date formatting should never fail")
    }

    /// **RFC 2822** date format (used in email `Date` headers).
    ///
    /// Example: `"Wed, 16 Apr 2025 14:30:45 +0000"`
    #[inline]
    pub fn to_str_rfc2822(&self) -> String {
        self.to_str_with_offset_label("%a, %d %b %Y %H:%M:%S %z", 0)
            .expect("RFC 2822 formatting should never fail")
    }

    /// **ISO 8601 week date**.
    ///
    /// Example: `"2025-W16-3"` (year-week-day)
    #[inline]
    pub fn to_str_iso_week_date(&self) -> String {
        self.to_str_with_offset_label("%G-W%V-%u", 0)
            .expect("ISO week date formatting should never fail")
    }

    /// Just the **ISO date** part (no time).
    ///
    /// Example: `"2025-04-16"`
    #[inline]
    pub fn to_str_iso_date(&self) -> String {
        self.to_str_with_offset_label("%Y-%m-%d", 0)
            .expect("ISO date formatting should never fail")
    }

    /// Just the **time** part with fractional seconds (trimmed).
    ///
    /// Example: `"14:30:45.123456789"`
    #[inline]
    pub fn to_str_iso_time(&self) -> String {
        self.to_str_with_offset_label("%H:%M:%S%.~f", 0)
            .expect("ISO time formatting should never fail")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Display / Debug (unchanged – they show internal representation, not RFC 3339)
// ─────────────────────────────────────────────────────────────────────────────

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sec = self.sec();
        let subsec = self.subsec();

        // Default to nanosecond precision (9 digits) — most useful for everyday use
        let precision = f.precision().unwrap_or(9);

        // Respect the `+` sign when the user writes {:+}
        if f.sign_plus() && sec >= 0 {
            write!(f, "+")?;
        }

        write!(f, "{}", sec)?;
        // (the old write_fractional helper is no longer needed – Display keeps its
        // original zero-padded behaviour for debugging)
        if precision > 0 {
            let prec = precision.min(18);
            let scale = 10u64.pow(18 - prec as u32);
            let value = subsec / scale;
            write!(f, ".{:0>width$}", value, width = prec)?;
        }

        if f.alternate() {
            write!(
                f,
                " [{} | sec={} subsec={}]",
                self.clock_type(),
                sec,
                subsec
            )?;
        } else {
            write!(f, " [{}]", self.clock_type())?;
        }

        Ok(())
    }
}

impl fmt::Debug for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let approx_sec = self.sec() as f64 + (self.subsec() as f64 / 1_000_000_000_000_000_000.0);

        f.debug_struct("TimePoint")
            .field("sec", &self.sec())
            .field("subsec", &self.subsec())
            .field("clock_type", &self.clock_type())
            .field("approx_sec", &approx_sec)
            .finish()
    }
}
