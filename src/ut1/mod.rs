use crate::{
    ATTOSEC_PER_SEC_I128, ClockType, DtErrKind, DtError, Real, SEC_PER_DAY, SEC_PER_DAYI128,
    TimePoint, TimeSpan, UNIX_EPOCH_TO_J2000_NOON_UTC, an_err,
};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, Default)]
pub enum Separator {
    #[default]
    Whitespace,
    Comma,
    Tab,
    Pipe,
    Semicolon,
}

#[derive(Debug, Clone, Copy)]
pub struct Ut1Columns {
    pub mjd: usize,
    pub ut1_utc: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Ut1Format {
    /// finals2000A.all / finals.all.iau2000.txt style files
    #[default]
    Finals2000A,
    /// C04 long-term series
    C04,
    /// User-defined column indices (0-based)
    Custom(Ut1Columns),
}

#[derive(Debug, Clone, Copy)]
pub struct Ut1Row {
    pub mjd: Real,
    pub ut1_minus_utc: Real,
}

#[derive(Debug, Clone)]
pub struct Ut1Data {
    rows: Vec<Ut1Row>,
}

impl Ut1Data {
    pub const MAX_LINE_LEN: usize = 8192;

    // ============================================================
    // Small helper — parses ONE row (shared by all paths)
    // ============================================================
    fn try_parse_row(trimmed: &str, format: Ut1Format, separator: Separator) -> Option<Ut1Row> {
        let parts: Vec<&str> = match separator {
            Separator::Whitespace => trimmed.split_whitespace().collect(),
            Separator::Comma => trimmed.split(',').map(|s| s.trim()).collect(),
            Separator::Tab => trimmed.split('\t').map(|s| s.trim()).collect(),
            Separator::Pipe => trimmed.split('|').map(|s| s.trim()).collect(),
            Separator::Semicolon => trimmed.split(';').map(|s| s.trim()).collect(),
        };

        if parts.len() < 2 {
            return None;
        }

        let (mjd, ut1_utc) = match format {
            Ut1Format::Finals2000A => {
                let mjd_idx = parts.iter().position(|p| {
                    p.contains('.') && p.parse::<Real>().map_or(false, |v| v > 30000.0)
                })?;
                let mut flag_count = 0;
                let mut ut1_value: Option<Real> = None;

                for i in (mjd_idx + 1)..parts.len() {
                    let token = parts[i];
                    let is_flag = token == "I"
                        || token == "P"
                        || token.starts_with("I-")
                        || token.starts_with("P-");

                    if is_flag {
                        flag_count += 1;
                        if flag_count == 2 {
                            let value_str = if token.starts_with("I-") || token.starts_with("P-") {
                                &token[1..]
                            } else if i + 1 < parts.len() {
                                parts[i + 1]
                            } else {
                                break;
                            };
                            if let Ok(val) = value_str.parse::<Real>() {
                                ut1_value = Some(val);
                            }
                            break;
                        }
                    }
                }

                let ut1 = ut1_value?;
                let mjd_val = parts[mjd_idx].parse::<Real>().ok()?;
                (Some(mjd_val), Some(ut1))
            }

            Ut1Format::C04 => {
                let (mjd_str, ut1_str) = (parts.get(4)?, parts.get(7)?);
                let mjd = mjd_str.parse::<Real>().ok()?;
                let ut1 = ut1_str.parse::<Real>().ok()?;
                (Some(mjd), Some(ut1))
            }

            Ut1Format::Custom(cols) => {
                let (mjd_str, ut1_str) = (parts.get(cols.mjd)?, parts.get(cols.ut1_utc)?);
                let mjd = mjd_str.parse::<Real>().ok()?;
                let ut1 = ut1_str.parse::<Real>().ok()?;
                (Some(mjd), Some(ut1))
            }
        };

        match (mjd, ut1_utc) {
            (Some(mjd_val), Some(ut1_val)) => Some(Ut1Row {
                mjd: mjd_val,
                ut1_minus_utc: ut1_val,
            }),
            _ => None,
        }
    }

    // ============================================================
    // CORE PARSER (no_std + alloc)
    // ============================================================
    fn parse_lines<'a>(
        lines: impl Iterator<Item = &'a str>,
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Vec<Ut1Row>, DtError> {
        let mut rows = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.len() > Self::MAX_LINE_LEN
            {
                continue;
            }

            if let Some(row) = Self::try_parse_row(trimmed, format, separator) {
                rows.push(row);
            }
        }

        if rows.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "no valid rows"));
        }

        rows.sort_by(|a, b| a.mjd.partial_cmp(&b.mjd).unwrap_or(Ordering::Equal));
        Ok(rows)
    }

    // ============================================================
    // no_std constructors
    // ============================================================
    pub fn data_from_str(
        s: &str,
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Vec<Ut1Row>, DtError> {
        Self::parse_lines(s.lines(), format, separator)
    }

    pub fn data_from_bytes(
        bytes: &[u8],
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Vec<Ut1Row>, DtError> {
        let s = core::str::from_utf8(bytes).unwrap_or("");
        Self::data_from_str(s, format, separator)
    }

    pub fn from_str(s: &str, format: Ut1Format, separator: Separator) -> Result<Self, DtError> {
        let rows = Self::data_from_str(s, format, separator)?;
        Ok(Self { rows })
    }

    pub fn from_bytes(
        bytes: &[u8],
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Self, DtError> {
        let rows = Self::data_from_bytes(bytes, format, separator)?;
        Ok(Self { rows })
    }

    // ============================================================
    // std-only constructors (streaming)
    // ============================================================
    #[cfg(feature = "std")]
    pub fn data_from_reader<R: std::io::BufRead>(
        mut reader: R,
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Vec<Ut1Row>, DtError> {
        let mut line_buf = String::with_capacity(256);
        let mut rows = Vec::new();

        loop {
            line_buf.clear();

            let bytes_read = match reader.read_line(&mut line_buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return Err(an_err!(DtErrKind::IOErr, "read line: {}", e));
                }
            };

            if bytes_read > Self::MAX_LINE_LEN {
                continue;
            }

            let trimmed = line_buf.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(row) = Self::try_parse_row(trimmed, format, separator) {
                rows.push(row);
            }
        }

        if rows.is_empty() {
            return Err(an_err!(DtErrKind::Incomplete, "no valid rows"));
        }

        rows.sort_by(|a, b| a.mjd.partial_cmp(&b.mjd).unwrap_or(Ordering::Equal));
        Ok(rows)
    }

    #[cfg(feature = "std")]
    pub fn data_from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Vec<Ut1Row>, DtError> {
        use std::fs::File;
        use std::io::BufReader;

        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| an_err!(DtErrKind::IOErr, "open file: '{}': {}", path.display(), e))?;

        let reader = BufReader::new(file);
        Self::data_from_reader(reader, format, separator)
    }

    #[cfg(feature = "std")]
    pub fn from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: Ut1Format,
        separator: Separator,
    ) -> Result<Self, DtError> {
        let rows = Self::data_from_text_file(path, format, separator)?;
        Ok(Self { rows })
    }

    /// Returns DUT1 = UT1 − UTC using the library’s *exact* MJD representation.
    ///
    /// This avoids any early floating-point conversion in `to_mjd_utc()` and
    /// guarantees the lookup uses the same high-precision path that the rest
    /// of the library uses (`to_mjd_utc_exact` + `from_mjd_utc_exact`).
    fn ut1_minus_utc_exact(&self, mjd_days: i64, mjd_frac: TimeSpan) -> Option<Real> {
        // This conversion is identical to what `to_mjd_utc()` does internally,
        // but we do it explicitly here so the caller stays on the exact path.
        let mjd_f = (mjd_days as Real) + mjd_frac.as_sec_f() / SEC_PER_DAY;
        self.ut1_minus_utc(mjd_f)
    }

    /// Returns DUT1 = UT1 − UTC (seconds) at the given MJD using linear interpolation.
    /// Returns `None` if the MJD is completely outside the loaded table.
    pub fn ut1_minus_utc(&self, mjd: Real) -> Option<Real> {
        if self.rows.is_empty() {
            return None;
        }

        let idx = match self
            .rows
            .binary_search_by(|probe| probe.mjd.partial_cmp(&mjd).unwrap_or(Ordering::Equal))
        {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    return Some(self.rows[0].ut1_minus_utc);
                }
                if i >= self.rows.len() {
                    return Some(self.rows[self.rows.len() - 1].ut1_minus_utc);
                }
                i - 1
            }
        };

        if idx + 1 < self.rows.len() {
            let e0 = &self.rows[idx];
            let e1 = &self.rows[idx + 1];
            let t = (mjd - e0.mjd) / (e1.mjd - e0.mjd);
            Some(e0.ut1_minus_utc + t * (e1.ut1_minus_utc - e0.ut1_minus_utc))
        } else {
            Some(self.rows[idx].ut1_minus_utc)
        }
    }
}

impl TimePoint {
    /// Convert **any** `TimePoint` to the equivalent UT1 instant (stored as `Custom`).
    ///
    /// Uses the library’s exact MJD path (`to_mjd_utc_exact`) for the lookup.
    pub fn to_ut1(&self, ut1_data: &Ut1Data) -> Result<Self, DtError> {
        let utc = self.to_clock_type(ClockType::UTC);
        let (mjd_days, mjd_frac) = utc.to_mjd_utc_exact();

        let dut1 = ut1_data
            .ut1_minus_utc_exact(mjd_days, mjd_frac)
            .ok_or_else(|| {
                let mjd_f = (mjd_days as Real) + mjd_frac.as_sec_f() / SEC_PER_DAY;
                an_err!(DtErrKind::OutOfRange, "mjd: {mjd_f}")
            })?;

        Ok(utc
            .add(TimeSpan::from_sec_f(dut1))
            .with_clock_type(ClockType::Custom))
    }

    /// Convert a UT1 `TimePoint` (normally `ClockType::Custom`) back to UTC.
    ///
    /// Uses fixed-point iteration (exactly like the library’s TDB ↔ TAI,
    /// TCG ↔ TT, etc.) to solve the implicit equation  
    /// `UTC = UT1 − DUT1(MJD_UTC)` to machine precision.
    pub fn from_ut1(ut1: Self, ut1_data: &Ut1Data) -> Result<Self, DtError> {
        if ut1_data.rows.is_empty() {
            return Err(an_err!(DtErrKind::InternalErr, "contains no data"));
        }

        let mut utc_guess = ut1.with_clock_type(ClockType::UTC);

        // DUT1 changes extremely slowly (< 0.1 ms/day). Four iterations are far more than enough.
        for _ in 0..4 {
            let (mjd_days, mjd_frac) = utc_guess.to_mjd_utc_exact();

            let dut1 = ut1_data
                .ut1_minus_utc_exact(mjd_days, mjd_frac)
                .ok_or_else(|| {
                    let mjd_f = (mjd_days as Real) + mjd_frac.as_sec_f() / SEC_PER_DAY;
                    an_err!(DtErrKind::OutOfRange, "mjd: {mjd_f}")
                })?;

            utc_guess = ut1
                .sub(TimeSpan::from_sec_f(dut1))
                .with_clock_type(ClockType::UTC);
        }

        Ok(utc_guess)
    }

    /// Returns the exact **Julian Date in UT1** with full attosecond precision.
    ///
    /// Uses the **Unix epoch (1970-01-01)** — exactly the same convention
    /// as [`Self::to_jd_utc_exact`]. This is the expected form for civil,
    /// GNSS, Earth orientation (IERS/VLBI), and operational use cases.
    #[inline]
    pub const fn to_jd_ut1_exact(self) -> (i64, TimeSpan) {
        // Mirror the UTC logic exactly:
        // Apply the same UNIX_EPOCH_TO_J2000_NOON_UTC offset that
        // to_canonical() uses for UTC.
        const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;

        let canon_attos = ((self.sec as i128) + (UNIX_EPOCH_TO_J2000_NOON_UTC as i128))
            * ATTOSEC_PER_SEC_I128
            + (self.subsec as i128);

        let days_since_1970 = canon_attos.div_euclid(ATTOS_PER_DAY);
        let frac_attos = canon_attos.rem_euclid(ATTOS_PER_DAY);

        let jd_int = 2_440_587i64 + (days_since_1970 as i64);

        (jd_int, TimeSpan::from_total_attos(frac_attos))
    }

    /// Returns the exact **Modified Julian Date in UT1** (Unix epoch convention).
    #[inline]
    pub const fn to_mjd_ut1_exact(self) -> (i64, TimeSpan) {
        let (jd, frac) = self.to_jd_ut1_exact();
        (jd - 2_400_000, frac)
    }

    /// Creates a `TimePoint` (as `Custom`) from an exact Julian Date in **UT1**.
    ///
    /// Inverse of [`Self::to_jd_ut1_exact`]. Uses the Unix epoch (1970),
    /// exactly like `from_jd_utc_exact`.
    #[inline]
    pub const fn from_jd_ut1_exact(jd_days: i64, frac: TimeSpan) -> Self {
        let days_since_1970 = jd_days - 2_440_587i64;
        const ATTOS_PER_DAY: i128 = SEC_PER_DAYI128 * ATTOSEC_PER_SEC_I128;

        let total_attos = (days_since_1970 as i128) * ATTOS_PER_DAY + frac.total_attos();

        // Apply the inverse offset (mirror what from_canonical does for UTC)
        let internal_sec =
            (total_attos / ATTOSEC_PER_SEC_I128) as i64 - UNIX_EPOCH_TO_J2000_NOON_UTC;
        let subsec = (total_attos % ATTOSEC_PER_SEC_I128) as u64;

        TimePoint::new(internal_sec, subsec, ClockType::Custom)
    }

    /// Creates a `TimePoint` (as `Custom`) from an exact Modified Julian Date in **UT1**.
    #[inline]
    pub const fn from_mjd_ut1_exact(mjd_days: i64, frac: TimeSpan) -> Self {
        Self::from_jd_ut1_exact(mjd_days + 2_400_000, frac)
    }

    /// Returns the **Julian Date in UT1** as a floating-point value (`f64`).
    ///
    /// Uses the Unix epoch (1970) convention, consistent with [`Self::to_jd_utc`].
    #[inline]
    pub const fn to_jd_ut1(self) -> Real {
        let (jd_days, frac) = self.to_jd_ut1_exact();
        (jd_days as Real) + (frac.as_sec_f() / SEC_PER_DAY)
    }

    /// Returns the **Modified Julian Date in UT1** as a floating-point value (`f64`).
    #[inline]
    pub const fn to_mjd_ut1(self) -> Real {
        self.to_jd_ut1() - f!(2_400_000.5)
    }
}
