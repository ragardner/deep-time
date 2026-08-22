//! Earth orientation tables: load, interpolate, apply UT1 offsets.
//!
//! [`EopData`] is the table. Each [`EopDataRow`] is an epoch, an offset in
//! seconds, and optional polar motion. [`EopData::eop_offset`] interpolates to
//! an [`EopOffset`]. Loaders take an [`EopFormat`] and a [`Separator`].
//! There is no [`Scale::UT1`](../enum.Scale.html).
//!
//! ## Formats
//!
//! - [`EopFormat::C04`] — IERS C04 / eopc04. UTC MJD, UT1−UTC.
//! - [`EopFormat::Finals2000A`] — USNO finals. UTC MJD, UT1−UTC; Bulletin B
//!   when filled, else A.
//! - [`EopFormat::JplEop2`] — JPL EOP2. TAI MJD; file TAI−UT1 (ms) stored as
//!   UT1−TAI (seconds).
//! - [`EopFormat::Custom`] — column indices via [`CustomEopCols`], or build
//!   with [`EopData::from_rows`].
//!
//! IERS loaders set [`EopData::epoch_scale`] to UTC and strip ~1 s leap jumps
//! before interpolating. JPL and custom default to TAI and no leap-strip.
//!
//! ## Applying an offset
//!
//! [`Dt::to_ut1`](../struct.Dt.html#method.to_ut1) / [`Dt::from_ut1`](../struct.Dt.html#method.from_ut1)
//! use the table’s epoch scale. [`Dt::to_eop`](../struct.Dt.html#method.to_eop)
//! / [`Dt::from_eop`](../struct.Dt.html#method.from_eop) take an explicit table
//! epoch (custom / other bodies).
//!
//! [`Dt::utc_mjd_to_ut1_mjd`](../struct.Dt.html#method.utc_mjd_to_ut1_mjd) maps
//! a UTC MJD float to UT1 MJD (IAU/ERFA leap-day length), which is a different
//! input from a [`Dt`](../struct.Dt.html).
//!
//! Out of range, interpolation holds the nearest endpoint. [`EopData::merge`]
//! keeps the base table’s leap-strip and epoch-scale; do not merge IERS with JPL.

#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::{Dt, DtErr, DtErrKind, Real, Scale, an_err, floor_f, round};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Delimiter used to split columns in EOP data files.
///
/// Passed as the `separator` argument to the various `data_from_*` /
/// `from_*` loaders. Controls how each line is tokenized before the parser
/// extracts the epoch, offset, and polar-motion values.
#[derive(Debug, Clone, Copy, Default)]
pub enum Separator {
    /// Split on any Unicode whitespace (default).
    #[default]
    Whitespace,
    /// Comma-separated values (`,`).
    Comma,
    /// Tab-separated values (`\t`).
    Tab,
    /// Pipe-separated values (`|`).
    Pipe,
    /// Semicolon-separated values (`;`).
    Semicolon,
}

/// File layout for the orientation-table parser.
///
/// - [`Finals2000A`](#variant.Finals2000A) — e.g.
///   <https://maia.usno.navy.mil/ser7/finals2000A.all>
/// - [`C04`](#variant.C04) — e.g.
///   <https://datacenter.iers.org/data/latestVersion/EOP_20u24_C04_one_file_1962-now.txt>
/// - [`JplEop2`](#variant.JplEop2) — e.g.
///   <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short>
/// - [`Custom`](#variant.Custom) — column indices via [`CustomEopCols`]
///
/// IERS formats set [`EopData::strip_offset_leaps`] to `true`; JplEop2 and Custom
/// to `false`.
#[derive(Debug, Clone, Default)]
pub enum EopFormat {
    /// USNO finals2000A fixed-width lines.
    ///
    /// Uses final (B) columns when filled, otherwise rapid (A). Epoch is UTC MJD.
    #[default]
    Finals2000A,
    /// IERS C04 / eopc04 long-term series. Epoch is UTC MJD.
    C04,
    /// JPL EOP2 comma-separated series (`latest_eop2.short` / `.long`).
    ///
    /// File columns: TAI MJD, PMx (mas), PMy (mas), TAI−UT1 (ms), …
    /// Stored as: `epoch` = TAI MJD, `offset` = **UT1 − TAI** in seconds
    /// (file ms/1000, then negated), `pm_x` / `pm_y` in arcseconds.
    ///
    /// [`to_ut1`](../struct.Dt.html#method.to_ut1) adds `offset`. For this
    /// table that means TAI + (UT1 − TAI) = UT1.
    ///
    /// Always comma-split; the `separator` argument is ignored.
    JplEop2,
    /// User-defined column indices (0-based). Epoch is whatever that column holds.
    Custom(CustomEopCols),
}

/// For use with [`EopFormat::Custom`].
///
/// 0-based column indices for a delimited orientation file.
#[derive(Debug, Clone)]
pub struct CustomEopCols {
    /// 0-based column index of the time key (epoch).
    pub epoch: usize,
    /// 0-based column index of the orientation offset in **seconds**.
    pub offset: usize,
    /// Optional 0-based column index of polar motion *x* (arcseconds).
    pub pm_x: Option<usize>,
    /// Optional 0-based column index of polar motion *y* (arcseconds).
    pub pm_y: Option<usize>,
}

/// One sample in an orientation table.
///
/// - `epoch` — time key of this sample (UTC MJD for IERS; TAI MJD for JPL EOP2;
///   arbitrary float for custom / other-body tables)
/// - `offset` — seconds to **add** to reach UT1 (UT1 − UTC in IERS files;
///   UT1 − TAI in JPL EOP2)
/// - `pm_x`, `pm_y` — polar motion in **arcseconds** (0 if unused)
#[derive(Debug, Clone)]
pub struct EopDataRow {
    /// Time key of this sample (same units as queries to [`EopData::eop_offset`]).
    pub epoch: Real,
    /// Orientation offset in seconds (e.g. UT1−UTC).
    pub offset: Real,
    /// Polar motion x (arcsec).
    pub pm_x: Real,
    /// Polar motion y (arcsec).
    pub pm_y: Real,
}

/// Sorted orientation-parameter table for a body.
///
/// Interpolate with [`eop_offset`](Self::eop_offset). On Earth, load IERS data
/// and use [`Dt::to_ut1`](../struct.Dt.html#method.to_ut1). For other bodies,
/// load custom rows and call [`Dt::to_eop`](../struct.Dt.html#method.to_eop)
/// / [`Dt::from_eop`](../struct.Dt.html#method.from_eop) with the epoch you
/// computed for that instant.
#[derive(Debug, Clone)]
pub struct EopData {
    /// Sample rows, sorted by ascending [`EopDataRow::epoch`].
    ///
    /// Loaders and [`from_rows`](Self::from_rows) keep this order.
    /// [`eop_offset`](Self::eop_offset) and [`merge`](Self::merge) assume it.
    pub rows: Vec<EopDataRow>,
    /// When true, a ~1 s jump between neighboring `offset` samples is removed
    /// before interpolation (leap-second days on Earth UT1−UTC tables).
    ///
    /// Finals2000A / C04 loaders set this `true`; JplEop2 and Custom set `false`.
    pub strip_offset_leaps: bool,
    /// Time scale of each row’s [`EopDataRow::epoch`] MJD.
    ///
    /// [`Dt::to_ut1`](../struct.Dt.html#method.to_ut1) converts the `Dt` to this
    /// scale before lookup. C04 / Finals2000A → [`Scale::UTC`]; JplEop2 and
    /// [`from_rows`](Self::from_rows) → [`Scale::TAI`]. Override with
    /// [`with_epoch_scale`](Self::with_epoch_scale).
    pub epoch_scale: Scale,
}

impl EopData {
    fn strip_for_format(format: &EopFormat) -> bool {
        matches!(format, EopFormat::Finals2000A | EopFormat::C04)
    }

    fn epoch_scale_for_format(format: &EopFormat) -> Scale {
        match format {
            EopFormat::Finals2000A | EopFormat::C04 => Scale::UTC,
            EopFormat::JplEop2 | EopFormat::Custom(_) => Scale::TAI,
        }
    }

    fn from_parsed_rows(rows: Vec<EopDataRow>, format: &EopFormat) -> Self {
        Self {
            rows,
            strip_offset_leaps: Self::strip_for_format(format),
            epoch_scale: Self::epoch_scale_for_format(format),
        }
    }

    /// Build a table from already-parsed rows (sorted by `epoch`).
    ///
    /// Set `strip_offset_leaps` true for Earth UT1−UTC-style leap days.
    /// [`epoch_scale`](Self::epoch_scale) defaults to [`Scale::TAI`]; chain
    /// [`with_epoch_scale`](Self::with_epoch_scale) for UTC-indexed tables.
    #[must_use]
    pub fn from_rows(mut rows: Vec<EopDataRow>, strip_offset_leaps: bool) -> Self {
        rows.sort_by(|a, b| a.epoch.partial_cmp(&b.epoch).unwrap_or(Ordering::Equal));
        Self {
            rows,
            strip_offset_leaps,
            epoch_scale: Scale::TAI,
        }
    }

    /// Set [`strip_offset_leaps`](Self::strip_offset_leaps).
    #[must_use]
    pub fn with_strip_offset_leaps(mut self, on: bool) -> Self {
        self.strip_offset_leaps = on;
        self
    }

    /// Set [`epoch_scale`](Self::epoch_scale).
    #[must_use]
    pub fn with_epoch_scale(mut self, scale: Scale) -> Self {
        self.epoch_scale = scale;
        self
    }
}

#[cfg(feature = "std")]
impl EopData {
    /// Parse EOP data from any `std::io::BufRead` (file, network stream, etc.).
    ///
    /// Lines starting with `#` or longer than [`EopData::MAX_LINE_LEN`] are skipped.
    /// The returned vector is always sorted by epoch.
    pub fn data_from_reader<R: std::io::BufRead>(
        mut reader: R,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopDataRow>, DtErr> {
        let mut line_buf = String::with_capacity(256);
        let mut rows = Vec::new();

        loop {
            line_buf.clear();

            let bytes_read = match reader.read_line(&mut line_buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return Err(an_err!(DtErrKind::IOErr, "{}", e));
                }
            };

            if bytes_read > Self::MAX_LINE_LEN {
                continue;
            }

            // Keep leading layout for fixed-width Finals; strip CR/LF only.
            let line = line_buf.trim_end();
            if Self::skip_eop_line(line, &format) {
                continue;
            }

            if let Some(row) = Self::try_parse_row(line, &format, separator) {
                rows.push(row);
            }
        }

        if rows.is_empty() {
            return Err(an_err!(DtErrKind::Empty));
        }

        rows.sort_by(|a, b| a.epoch.partial_cmp(&b.epoch).unwrap_or(Ordering::Equal));
        Ok(rows)
    }

    /// Returns a [`Vec`] of [`EopDataRow`] from a text file on disk.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std"))]
    /// # {
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    ///
    /// let path = "tests/assets/finals.all.iau2000.txt";
    /// let rows = EopData::data_from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace).unwrap();
    /// # }
    /// ```
    ///
    /// ## See also
    ///
    /// - [`EopData::from_text_file`](#method.from_text_file)
    pub fn data_from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopDataRow>, DtErr> {
        use std::fs::File;
        use std::io::BufReader;

        let path = path.as_ref();
        let file = File::open(path).map_err(|e| an_err!(DtErrKind::IOErr, "{}", e))?;

        let reader = BufReader::new(file);
        Self::data_from_reader(reader, format, separator)
    }

    /// Create an [`EopData`] by loading from a text file on disk.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std"))]
    /// # {
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    ///
    /// let path = "tests/assets/finals.all.iau2000.txt";
    /// let provider = EopData::from_text_file(path, EopFormat::Finals2000A, Separator::Whitespace).unwrap();
    /// # }
    /// ```
    pub fn from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Self, DtErr> {
        let rows = Self::data_from_text_file(path, format.clone(), separator)?;
        Ok(Self::from_parsed_rows(rows, &format))
    }
}

impl EopData {
    /// Maximum accepted length of a single input line when parsing EOP text.
    pub const MAX_LINE_LEN: usize = 8192;

    /// Parse a single EOP row.
    ///
    /// For Finals, `line` should keep leading layout (only trailing newline
    /// stripped by the caller). C04/Custom trim when tokenizing.
    fn try_parse_row(line: &str, format: &EopFormat, separator: Separator) -> Option<EopDataRow> {
        match format {
            // USNO finals2000A fixed-width (CDS ReadMe). Prefer B when filled, else A.
            EopFormat::Finals2000A => {
                // Need at least through Bulletin A UT1-UTC (bytes 59–68, 1-based).
                if line.len() < 68 {
                    return None;
                }

                let field = |s: &str| -> Option<Real> {
                    let t = s.trim();
                    if t.is_empty() { None } else { t.parse().ok() }
                };

                // 1-based CDS columns → 0-based half-open slices
                let epoch = field(line.get(7..15)?)?;
                let pm_x_a = line.get(18..27).and_then(field);
                let pm_y_a = line.get(37..46).and_then(field);
                let ut1_a = field(line.get(58..68)?)?;

                let ut1_b = if line.len() >= 165 {
                    line.get(154..165).and_then(field)
                } else {
                    None
                };
                let pm_x_b = if line.len() >= 144 {
                    line.get(134..144).and_then(field)
                } else {
                    None
                };
                let pm_y_b = if line.len() >= 154 {
                    line.get(144..154).and_then(field)
                } else {
                    None
                };

                let offset = ut1_b.unwrap_or(ut1_a);
                let (pm_x, pm_y) = match (pm_x_b, pm_y_b) {
                    (Some(x), Some(y)) => (x, y),
                    _ => (pm_x_a.unwrap_or(0.0), pm_y_a.unwrap_or(0.0)),
                };

                Some(EopDataRow {
                    epoch,
                    offset,
                    pm_x,
                    pm_y,
                })
            }

            EopFormat::C04 => {
                let parts = Self::split_eop_line(line, separator);
                if parts.len() < 2 {
                    return None;
                }
                let epoch = parts.get(4)?.parse::<Real>().ok()?;
                let pm_x = parts
                    .get(5)
                    .unwrap_or(&"0.0")
                    .parse::<Real>()
                    .unwrap_or(0.0);
                let pm_y = parts
                    .get(6)
                    .unwrap_or(&"0.0")
                    .parse::<Real>()
                    .unwrap_or(0.0);
                let offset = parts.get(7)?.parse::<Real>().ok()?;
                Some(EopDataRow {
                    epoch,
                    offset,
                    pm_x,
                    pm_y,
                })
            }

            EopFormat::JplEop2 => {
                let parts = Self::split_eop_line(line, Separator::Comma);
                if parts.len() < 4 {
                    return None;
                }
                let epoch = parts.first()?.parse::<Real>().ok()?;
                let pm_x = parts.get(1)?.parse::<Real>().ok()? / 1000.0;
                let pm_y = parts.get(2)?.parse::<Real>().ok()? / 1000.0;
                // File is TAI−UT1 in ms. Store UT1−TAI in seconds so `to_ut1` adds it.
                let offset = -(parts.get(3)?.parse::<Real>().ok()? / 1000.0);
                Some(EopDataRow {
                    epoch,
                    offset,
                    pm_x,
                    pm_y,
                })
            }

            EopFormat::Custom(cols) => {
                let parts = Self::split_eop_line(line, separator);
                if parts.len() < 2 {
                    return None;
                }
                let epoch = parts.get(cols.epoch)?.parse::<Real>().ok()?;
                let offset = parts.get(cols.offset)?.parse::<Real>().ok()?;
                let pm_x = if let Some(pm_x_col) = cols.pm_x {
                    parts
                        .get(pm_x_col)
                        .unwrap_or(&"0.0")
                        .parse::<Real>()
                        .ok()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let pm_y = if let Some(pm_y_col) = cols.pm_y {
                    parts
                        .get(pm_y_col)
                        .unwrap_or(&"0.0")
                        .parse::<Real>()
                        .ok()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                Some(EopDataRow {
                    epoch,
                    offset,
                    pm_x,
                    pm_y,
                })
            }
        }
    }

    fn split_eop_line(line: &str, separator: Separator) -> Vec<&str> {
        let trimmed = line.trim();
        match separator {
            Separator::Whitespace => trimmed.split_whitespace().collect(),
            Separator::Comma => trimmed.split(',').map(|s| s.trim()).collect(),
            Separator::Tab => trimmed.split('\t').map(|s| s.trim()).collect(),
            Separator::Pipe => trimmed.split('|').map(|s| s.trim()).collect(),
            Separator::Semicolon => trimmed.split(';').map(|s| s.trim()).collect(),
        }
    }

    fn skip_eop_line(line: &str, format: &EopFormat) -> bool {
        if line.is_empty() || line.len() > Self::MAX_LINE_LEN {
            return true;
        }
        if line.starts_with('#') {
            return true;
        }
        match format {
            EopFormat::C04 | EopFormat::Custom(_) => line.trim_start().starts_with('#'),
            EopFormat::JplEop2 => {
                let t = line.trim_start();
                t.starts_with('$') || t.starts_with("EOP2=") || t.starts_with("EOP2L")
            }
            EopFormat::Finals2000A => false,
        }
    }

    fn parse_lines<'a>(
        lines: impl Iterator<Item = &'a str>,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopDataRow>, DtErr> {
        let mut rows = Vec::new();

        for line in lines {
            // Keep leading columns for fixed-width Finals; only strip ends.
            let line = line.trim_end();
            if Self::skip_eop_line(line, &format) {
                continue;
            }

            if let Some(row) = Self::try_parse_row(line, &format, separator) {
                rows.push(row);
            }
        }

        if rows.is_empty() {
            return Err(an_err!(DtErrKind::Empty));
        }

        rows.sort_by(|a, b| a.epoch.partial_cmp(&b.epoch).unwrap_or(Ordering::Equal));
        Ok(rows)
    }

    /// Parse EOP data from a `&str`.
    ///
    /// Useful when the data is already in memory (embedded resource,
    /// downloaded string, etc.).
    pub fn data_from_str(
        s: &str,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopDataRow>, DtErr> {
        Self::parse_lines(s.lines(), format, separator)
    }

    /// Parse EOP data from raw bytes.
    ///
    /// The bytes are interpreted as UTF-8. Invalid UTF-8 sequences
    /// result in an empty string (and therefore an error).
    pub fn data_from_bytes(
        bytes: &[u8],
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopDataRow>, DtErr> {
        let s = core::str::from_utf8(bytes).unwrap_or("");
        Self::data_from_str(s, format, separator)
    }

    /// Create an [`EopData`] from a string slice.
    pub fn from_str(s: &str, format: EopFormat, separator: Separator) -> Result<Self, DtErr> {
        let rows = Self::data_from_str(s, format.clone(), separator)?;
        Ok(Self::from_parsed_rows(rows, &format))
    }

    /// Create an [`EopData`] from raw bytes.
    pub fn from_bytes(
        bytes: &[u8],
        format: EopFormat,
        separator: Separator,
    ) -> Result<Self, DtErr> {
        let rows = Self::data_from_bytes(bytes, format.clone(), separator)?;
        Ok(Self::from_parsed_rows(rows, &format))
    }

    /// Merge rows from `other` into `self` by epoch.
    ///
    /// For each row in `other`:
    ///
    /// - **Same epoch already in `self`:** if `overwrite_rows`, replace
    ///   `offset` / `pm_x` / `pm_y`; otherwise leave `self`’s row as-is.
    /// - **Epoch not in `self`:** if `add_rows`, insert the row (table stays
    ///   sorted by epoch); otherwise skip it.
    ///
    /// Both flags may be true or only one. If both are false, this is a no-op.
    /// [`strip_offset_leaps`](Self::strip_offset_leaps) and
    /// [`epoch_scale`](Self::epoch_scale) on `self` are unchanged.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # #[cfg(all(feature = "eop", feature = "std"))]
    /// # {
    /// use deep_time::eop::{EopData, EopFormat, Separator};
    ///
    /// let mut eop = EopData::from_text_file(
    ///     "tests/assets/EOP_20u24_C04_one_file_1962-now.txt",
    ///     EopFormat::C04,
    ///     Separator::Whitespace,
    /// ).unwrap();
    /// let finals = EopData::from_text_file(
    ///     "tests/assets/finals.all.iau2000.txt",
    ///     EopFormat::Finals2000A,
    ///     Separator::Whitespace,
    /// ).unwrap();
    /// // Keep C04 on overlap; append Finals prediction days only.
    /// eop.merge(&finals, true, false);
    /// # }
    /// ```
    pub fn merge(&mut self, other: &EopData, add_rows: bool, overwrite_rows: bool) {
        if !add_rows && !overwrite_rows {
            return;
        }
        for src in &other.rows {
            match self.rows.binary_search_by(|probe| {
                probe
                    .epoch
                    .partial_cmp(&src.epoch)
                    .unwrap_or(Ordering::Equal)
            }) {
                Ok(i) => {
                    if overwrite_rows {
                        let dst = &mut self.rows[i];
                        dst.offset = src.offset;
                        dst.pm_x = src.pm_x;
                        dst.pm_y = src.pm_y;
                    }
                }
                Err(i) => {
                    if add_rows {
                        self.rows.insert(i, src.clone());
                    }
                }
            }
        }
    }

    /// Convenience: [`merge`](Self::merge) consuming `self`.
    #[must_use]
    pub fn with_merge(mut self, other: &EopData, add_rows: bool, overwrite_rows: bool) -> Self {
        self.merge(other, add_rows, overwrite_rows);
        self
    }

    /// Interpolated orientation parameters at `epoch`.
    ///
    /// Linear blend between neighboring samples. If
    /// [`strip_offset_leaps`](Self::strip_offset_leaps) is set, a ~1 s jump in
    /// `offset` between those samples is removed before blending. Outside the
    /// table range the nearest endpoint is held. Returns `None` if empty.
    pub fn eop_offset(&self, epoch: Real) -> Option<EopOffset> {
        if self.rows.is_empty() {
            return None;
        }

        // Match Astropy `searchsorted(..., side="right") - 1`: left sample is
        // the last row with epoch_row <= query (except before the first row).
        let idx = match self
            .rows
            .binary_search_by(|probe| probe.epoch.partial_cmp(&epoch).unwrap_or(Ordering::Equal))
        {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    let row = &self.rows[0];
                    return Some(EopOffset {
                        offset: row.offset,
                        pm_x: row.pm_x,
                        pm_y: row.pm_y,
                    });
                }
                if i >= self.rows.len() {
                    let row = &self.rows[self.rows.len() - 1];
                    return Some(EopOffset {
                        offset: row.offset,
                        pm_x: row.pm_x,
                        pm_y: row.pm_y,
                    });
                }
                i - 1
            }
        };

        if idx + 1 < self.rows.len() {
            let e0 = &self.rows[idx];
            let e1 = &self.rows[idx + 1];

            let span = e1.epoch - e0.epoch;
            // Exact table epoch (or degenerate span): no blend needed.
            if span == 0.0 || epoch == e0.epoch {
                return Some(EopOffset {
                    offset: e0.offset,
                    pm_x: e0.pm_x,
                    pm_y: e0.pm_y,
                });
            }

            let t = (epoch - e0.epoch) / span;

            let mut d_offset = e1.offset - e0.offset;
            if self.strip_offset_leaps {
                d_offset -= round(d_offset);
            }

            let offset = e0.offset + t * d_offset;
            let pm_x = e0.pm_x + t * (e1.pm_x - e0.pm_x);
            let pm_y = e0.pm_y + t * (e1.pm_y - e0.pm_y);

            Some(EopOffset { offset, pm_x, pm_y })
        } else {
            let row = &self.rows[idx];
            Some(EopOffset {
                offset: row.offset,
                pm_x: row.pm_x,
                pm_y: row.pm_y,
            })
        }
    }
}

/// Interpolated orientation parameters at one epoch.
#[derive(Debug, Clone, Default)]
pub struct EopOffset {
    /// Offset in **seconds** (e.g. UT1 − UTC on Earth).
    pub offset: Real,
    /// Polar motion x-coordinate in **arcseconds**.
    pub pm_x: Real,
    /// Polar motion y-coordinate in **arcseconds**.
    pub pm_y: Real,
}

impl Dt {
    /// Full [`EopOffset`] at a table epoch.
    ///
    /// Outside the table the nearest endpoint is held. Errors only when the
    /// table is empty.
    pub fn eop_offset_at(epoch: Real, op_data: &EopData) -> Result<EopOffset, DtErr> {
        op_data
            .eop_offset(epoch)
            .ok_or_else(|| an_err!(DtErrKind::Empty, "{epoch}"))
    }

    /// Offset in seconds at a table epoch.
    #[inline]
    pub fn eop_offset_at_f(epoch: Real, op_data: &EopData) -> Result<Real, DtErr> {
        Self::eop_offset_at(epoch, op_data).map(|res| res.offset)
    }

    /// Same as [`Dt::eop_offset_at`](../struct.Dt.html#method.eop_offset_at).
    #[inline]
    pub fn mjd_to_eop_offset(mjd: Real, op_data: &EopData) -> Result<EopOffset, DtErr> {
        Self::eop_offset_at(mjd, op_data)
    }

    /// Same as [`Dt::eop_offset_at_f`](../struct.Dt.html#method.eop_offset_at_f).
    #[inline]
    pub fn mjd_to_eop_offset_f(mjd: Real, op_data: &EopData) -> Result<Real, DtErr> {
        Self::eop_offset_at_f(mjd, op_data)
    }

    /// UT1 MJD from a UTC MJD (IAU / ERFA / Astropy convention).
    ///
    /// The fractional part of `mjd_utc` is a fraction of the UTC day, which
    /// is 86 401 SI seconds on a leap-second insertion day (see
    /// [`Dt::utc_day_length_sec`](../struct.Dt.html#method.utc_day_length_sec)). Then:
    ///
    /// ```text
    /// UT1 MJD = day + (frac × UTC_day_length + DUT1) / 86400
    /// ```
    ///
    /// On a normal day this is `mjd_utc + DUT1/86400`. On a leap-insertion
    /// interior it is **not** — Astropy's `Time(mjd, scale="utc").ut1.mjd`
    /// follows this same stretch.
    ///
    /// DUT1 is interpolated at `mjd_utc` (IERS table epoch).
    pub fn utc_mjd_to_ut1_mjd(mjd_utc: Real, op_data: &EopData) -> Result<Real, DtErr> {
        let dut1 = Self::mjd_to_eop_offset_f(mjd_utc, op_data)?;
        let day = floor_f(mjd_utc);
        let frac = mjd_utc - day;
        let day_len = Self::utc_day_length_sec(mjd_utc);
        Ok(day + (frac * day_len + dut1) / 86_400.0)
    }

    /// Convert this [`Dt`](../struct.Dt.html) to UT1 using the table’s
    /// [`EopData::epoch_scale`].
    ///
    /// The `Dt` is converted to that scale, the table is interpolated at the
    /// resulting MJD, and the offset is added to that converted instant.
    /// IERS tables (UTC, UT1−UTC) and JPL EOP2 (TAI, UT1−TAI) both work
    /// without placing the `Dt` on the table scale first.
    ///
    /// When the table’s
    /// [`EopData::strip_offset_leaps`]
    /// field is `true`, a ~1 s jump between neighboring offset samples is
    /// removed before interpolating. C04 / Finals loaders set it `true`; JPL
    /// EOP2 and custom tables leave it `false`. Change the field on the table,
    /// or use
    /// [`EopData::with_strip_offset_leaps`],
    /// if you want the other behaviour.
    ///
    /// Returned attoseconds are the UT1 clock. There is no `Scale::UT1`.
    /// Leap seconds are not smeared through the UTC day; for a UTC MJD *float*
    /// in the IAU/ERFA sense see [`Dt::utc_mjd_to_ut1_mjd`](../struct.Dt.html#method.utc_mjd_to_ut1_mjd).
    ///
    /// For an explicit table epoch use [`Dt::to_eop`](../struct.Dt.html#method.to_eop).
    pub fn to_ut1(&self, op_data: &EopData) -> Result<Self, DtErr> {
        let on_epoch = self.to(op_data.epoch_scale);
        on_epoch.to_eop(op_data, on_epoch.to_mjd_f_raw())
    }

    /// Inverse of [`Dt::to_ut1`](../struct.Dt.html#method.to_ut1): subtract the table offset looked
    /// up at the table’s epoch scale (fixed-point).
    ///
    /// Uses the table’s
    /// [`EopData::strip_offset_leaps`]
    /// field the same way as [`Dt::to_ut1`](../struct.Dt.html#method.to_ut1) (`true` for C04 / Finals,
    /// `false` for JPL EOP2). Change the field on the table if you want the
    /// other behaviour.
    ///
    /// For an explicit table epoch use [`Dt::from_eop`](../struct.Dt.html#method.from_eop).
    pub fn from_ut1(&self, op_data: &EopData) -> Result<Self, DtErr> {
        if op_data.rows.is_empty() {
            return Err(an_err!(DtErrKind::Empty));
        }
        let ut1 = self.to(op_data.epoch_scale);
        let mut without = ut1;

        for _ in 0..8 {
            let epoch = without.to_mjd_f_raw();
            let offset = op_data
                .eop_offset(epoch)
                .ok_or_else(|| an_err!(DtErrKind::Empty, "{epoch}"))?
                .offset;

            without = ut1.sub(Dt::from_sec_f(offset, Scale::TAI, Scale::TAI));
        }

        Ok(without)
    }

    /// Add the table offset at the given `epoch` (seconds).
    ///
    /// `epoch` is the file’s time index (UTC MJD for IERS, TAI MJD for JPL
    /// EOP2, whatever was stored for [`EopFormat::Custom`]).
    #[inline]
    pub fn to_eop(&self, op_data: &EopData, epoch: Real) -> Result<Self, DtErr> {
        Ok(self.add(Dt::from_sec_f(
            Self::eop_offset_at_f(epoch, op_data)?,
            Scale::TAI,
            Scale::TAI,
        )))
    }

    /// Subtract the table offset at the given `epoch` (seconds).
    ///
    /// `epoch` is the file’s time index (UTC MJD for IERS, TAI MJD for JPL
    /// EOP2, whatever was stored for [`EopFormat::Custom`]).
    ///
    /// If you do not have that table epoch and the table is keyed on a
    /// [`Scale`] MJD, use [`Dt::from_ut1`](../struct.Dt.html#method.from_ut1) instead.
    #[inline]
    pub fn from_eop(&self, op_data: &EopData, epoch: Real) -> Result<Self, DtErr> {
        Ok(self.sub(Dt::from_sec_f(
            Self::eop_offset_at_f(epoch, op_data)?,
            Scale::TAI,
            Scale::TAI,
        )))
    }
}
