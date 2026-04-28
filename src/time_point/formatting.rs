use crate::DtError;
use crate::TimePoint;
use alloc::string::String;

impl TimePoint {
    /// Returns this instant as an **RFC 3339** / ISO 8601 timestamp in **UTC**
    /// with the `Z` suffix.
    ///
    /// - Always uses UTC (`Z` = Zulu = UTC).
    /// - Default = 9 digits (nanoseconds) but **automatically trims trailing zeros**.
    /// - If fractional part is zero → no decimal point at all (e.g. `...45Z`).
    /// - Example: `"2024-03-14T15:30:45.123Z"`
    #[inline(always)]
    pub fn to_str_rfc3339(&self) -> Result<String, DtError> {
        self.to_str_rfc3339_nf(9)
    }

    /// Same as [`to_str_rfc3339`] but with a configurable maximum number of fractional digits
    /// (0–18). Trailing zeros are always trimmed.
    pub fn to_str_rfc3339_nf(&self, max_precision: usize) -> Result<String, DtError> {
        let prec = max_precision.min(18);
        // Uses the new formatter with the `~` "trim trailing zeros" flag.
        // The formatter already handles:
        //   - correct 4-digit years (with sign) for |yr| < 10000
        //   - full-width years otherwise
        //   - suppressing the decimal point entirely when the trimmed fraction is zero
        let fmt = alloc::format!("%Y-%m-%dT%H:%M:%S%.{}~fZ", prec);
        self.to_str_with_offset(&fmt, 0)
    }

    /// **ISO 8601 / RFC 3339** with **actual offset** (modern `+00:00` style).
    ///
    /// - Uses colon-separated offset (`%:z`) instead of forcing `Z`.
    /// - Still trims trailing zeros in the fractional part.
    /// - Example: `"2025-04-16T14:30:45.123+00:00"`
    #[inline]
    pub fn to_str_iso8601(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%Y-%m-%dT%H:%M:%S%.~f%:z", 0)
    }

    /// **Compact ISO 8601 basic format** (no separators).
    ///
    /// - Useful for filenames, URLs, database keys, etc.
    /// - Example: `"20250416T143045.123456789Z"`
    #[inline]
    pub fn to_str_iso8601_basic(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%Y%m%dT%H%M%S%.~fZ", 0)
    }

    /// **HTTP-date** format (RFC 7231 / RFC 1123) — **always in GMT**.
    ///
    /// This is the format used in `Date`, `Expires`, `Last-Modified` headers.
    /// Example: `"Wed, 16 Apr 2025 14:30:45 GMT"`
    #[inline]
    pub fn to_str_http(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%a, %d %b %Y %H:%M:%S GMT", 0)
    }

    /// **RFC 2822** date format (used in email `Date` headers).
    ///
    /// Example: `"Wed, 16 Apr 2025 14:30:45 +0000"`
    #[inline]
    pub fn to_str_rfc2822(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%a, %d %b %Y %H:%M:%S %z", 0)
    }

    /// **ISO 8601 week date**.
    ///
    /// Example: `"2025-W16-3"` (year-week-day)
    #[inline]
    pub fn to_str_iso_week_date(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%G-W%V-%u", 0)
    }

    /// Just the **ISO date** part (no time).
    ///
    /// Example: `"2025-04-16"`
    #[inline]
    pub fn to_str_iso_date(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%Y-%m-%d", 0)
    }

    /// Just the **time** part with fractional seconds (trimmed).
    ///
    /// Example: `"14:30:45.123456789"`
    #[inline]
    pub fn to_str_iso_time(&self) -> Result<String, DtError> {
        self.to_str_with_offset("%H:%M:%S%.~f", 0)
    }
}
