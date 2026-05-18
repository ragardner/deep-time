#![allow(clippy::indexing_slicing)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::approx_constant)]
#![allow(clippy::eq_op)]

use crate::{Dt, DtErr, DtErrKind, Real, an_err};
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

/// Body Orientation Parameters Format.
///
/// Formats to provide to the parser, including a
/// custom one to allow specific column indices.
///
/// - `Finals2000A` such as is available from
///   https://maia.usno.navy.mil/ser7/finals2000A.all
/// - `C04` such as is available from
///   https://datacenter.iers.org/data/latestVersion/EOP_20u24_C04_one_file_1962-now.txt
/// - `Custom` so you can provide your own specific column indices
///   using [`CustomBopCols`].
#[derive(Debug, Clone, Copy, Default)]
pub enum BopFormat {
    /// finals2000A.all / finals.all.iau2000.txt style files
    #[default]
    Finals2000A,
    /// C04 long-term series
    C04,
    /// User-defined column indices (0-based)
    Custom(CustomBopCols),
}

/// For use with [`BopFormat`].
#[derive(Debug, Clone, Copy)]
pub struct CustomBopCols {
    pub mjd: usize,
    pub offset: usize,
    pub pm_x: Option<usize>,
    pub pm_y: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct BopDataRow {
    pub mjd: Real,
    /// e.g. UT1-UTC(s)
    pub offset: Real,
    /// polar motion x (arcsec)
    pub pm_x: Real,
    /// polar motion y (arcsec)
    pub pm_y: Real,
}

/// Container for Body Orientation Parameters data.
///
/// - On Earth this would enable time scale conversions to and from
///   the **UT1 time scale**.
/// - Earth Orientation Parameters data is available from: https://maia.usno.navy.mil/ser7/finals2000A.all
#[derive(Debug, Clone)]
pub struct BopData {
    rows: Vec<BopDataRow>,
}

#[cfg(feature = "std")]
impl BopData {
    pub fn data_from_reader<R: std::io::BufRead>(
        mut reader: R,
        format: BopFormat,
        separator: Separator,
    ) -> Result<Vec<BopDataRow>, DtErr> {
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

    pub fn data_from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: BopFormat,
        separator: Separator,
    ) -> Result<Vec<BopDataRow>, DtErr> {
        use std::fs::File;
        use std::io::BufReader;

        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| an_err!(DtErrKind::IOErr, "open file: '{}': {}", path.display(), e))?;

        let reader = BufReader::new(file);
        Self::data_from_reader(reader, format, separator)
    }

    pub fn from_text_file<P: AsRef<std::path::Path>>(
        path: P,
        format: BopFormat,
        separator: Separator,
    ) -> Result<Self, DtErr> {
        let rows = Self::data_from_text_file(path, format, separator)?;
        Ok(Self { rows })
    }
}

impl BopData {
    pub const MAX_LINE_LEN: usize = 8192;

    // Small helper — parses ONE row (shared by all paths)
    fn try_parse_row(trimmed: &str, format: BopFormat, separator: Separator) -> Option<BopDataRow> {
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

        let (mjd, offset, pm_x, pm_y) = match format {
            BopFormat::Finals2000A => {
                let mjd_idx = parts.iter().position(|p| {
                    p.contains('.') && p.parse::<Real>().is_ok_and(|v| v > 30000.0)
                })?;

                let mut flag_count = 0;
                let mut offset_value: Option<Real> = None;
                let mut pm_x_val: Real = 0.0;
                let mut pm_y_val: Real = 0.0;

                for i in (mjd_idx + 1)..parts.len() {
                    let token = parts[i];

                    let is_flag = token == "I"
                        || token == "P"
                        || token.starts_with("I-")
                        || token.starts_with("P-");

                    if is_flag {
                        flag_count += 1;

                        if flag_count == 1 {
                            // After first flag: x_p and y_p
                            if let (Some(x_str), Some(y_str)) = (parts.get(i + 1), parts.get(i + 3))
                            {
                                pm_x_val = x_str.parse::<Real>().unwrap_or(0.0);
                                pm_y_val = y_str.parse::<Real>().unwrap_or(0.0);
                            }
                        }

                        if flag_count == 2 {
                            let value_str = if token.starts_with("I-") || token.starts_with("P-") {
                                &token[1..]
                            } else if i + 1 < parts.len() {
                                parts[i + 1]
                            } else {
                                break;
                            };
                            if let Ok(val) = value_str.parse::<Real>() {
                                offset_value = Some(val);
                            }
                            break;
                        }
                    }
                }

                let offset = offset_value?;
                let mjd_val = parts[mjd_idx].parse::<Real>().ok()?;

                (mjd_val, offset, pm_x_val, pm_y_val)
            }

            BopFormat::C04 => {
                let mjd = parts.get(4)?.parse::<Real>().ok()?;
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
                (mjd, offset, pm_x, pm_y)
            }

            BopFormat::Custom(cols) => {
                let mjd = parts.get(cols.mjd)?.parse::<Real>().ok()?;
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
                (mjd, offset, pm_x, pm_y)
            }
        };

        Some(BopDataRow {
            mjd,
            offset,
            pm_x,
            pm_y,
        })
    }

    fn parse_lines<'a>(
        lines: impl Iterator<Item = &'a str>,
        format: BopFormat,
        separator: Separator,
    ) -> Result<Vec<BopDataRow>, DtErr> {
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

    pub fn data_from_str(
        s: &str,
        format: BopFormat,
        separator: Separator,
    ) -> Result<Vec<BopDataRow>, DtErr> {
        Self::parse_lines(s.lines(), format, separator)
    }

    pub fn data_from_bytes(
        bytes: &[u8],
        format: BopFormat,
        separator: Separator,
    ) -> Result<Vec<BopDataRow>, DtErr> {
        let s = core::str::from_utf8(bytes).unwrap_or("");
        Self::data_from_str(s, format, separator)
    }

    pub fn from_str(s: &str, format: BopFormat, separator: Separator) -> Result<Self, DtErr> {
        let rows = Self::data_from_str(s, format, separator)?;
        Ok(Self { rows })
    }

    pub fn from_bytes(
        bytes: &[u8],
        format: BopFormat,
        separator: Separator,
    ) -> Result<Self, DtErr> {
        let rows = Self::data_from_bytes(bytes, format, separator)?;
        Ok(Self { rows })
    }

    /// Returns all interpolated orientation parameters (offset + polar motion)
    /// at the given MJD.
    ///
    /// Returns `None` if the table is empty or the MJD is completely outside
    /// the loaded data.
    pub fn params(&self, mjd: Real) -> Option<BopParams> {
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
                    let row = &self.rows[0];
                    return Some(BopParams {
                        offset: row.offset,
                        pm_x: row.pm_x,
                        pm_y: row.pm_y,
                    });
                }
                if i >= self.rows.len() {
                    let row = &self.rows[self.rows.len() - 1];
                    return Some(BopParams {
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

            let t = (mjd - e0.mjd) / (e1.mjd - e0.mjd);

            let offset = e0.offset + t * (e1.offset - e0.offset);
            let pm_x = e0.pm_x + t * (e1.pm_x - e0.pm_x);
            let pm_y = e0.pm_y + t * (e1.pm_y - e0.pm_y);

            Some(BopParams { offset, pm_x, pm_y })
        } else {
            let row = &self.rows[idx];
            Some(BopParams {
                offset: row.offset,
                pm_x: row.pm_x,
                pm_y: row.pm_y,
            })
        }
    }
}

/// Interpolated Body Orientation Parameters at a specific MJD.
///
/// Contains everything needed for high-precision sidereal time
/// and polar-motion corrections.
#[derive(Debug, Clone, Copy, Default)]
pub struct BopParams {
    /// Value in **seconds** e.g. UT1 − UTC offset
    pub offset: Real,
    /// Polar motion x-coordinate in **arcseconds**
    pub pm_x: Real,
    /// Polar motion y-coordinate in **arcseconds**
    pub pm_y: Real,
}

impl Dt {
    /// Get an orientation parameters offset in seconds for a particular Modified Julian Date.
    ///
    /// - On Earth this would be the UT1 time scale.
    /// - Earth Orientation Parameters data is available from: https://maia.usno.navy.mil/ser7/finals2000A.all
    pub fn orientation_offset(mjd: Real, op_data: &BopData) -> Result<Real, DtErr> {
        let offset = op_data
            .params(mjd)
            .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "mjd: {mjd}"))?
            .offset;
        Ok(offset)
    }

    /// Offsets a [`Dt`] using orientation parameters data.
    ///
    /// - On Earth this would be the UT1 time scale.
    /// - Earth Orientation Parameters data is available from: https://maia.usno.navy.mil/ser7/finals2000A.all
    #[inline]
    pub fn to_offset_by_bop(&self, op_data: &BopData) -> Result<Self, DtErr> {
        Ok(self.add(Dt::from_sec_f(Self::orientation_offset(
            self.to_mjd_f(),
            op_data,
        )?)))
    }

    /// Convert a [`Dt`] already offset using orientation parameters data back to whatever
    /// it was before.
    ///
    /// - On Earth this would be the UT1 time scale.
    /// - Earth Orientation Parameters data is available from: https://maia.usno.navy.mil/ser7/finals2000A.all
    pub fn from_offset_by_bop(&self, op_data: &BopData) -> Result<Self, DtErr> {
        if op_data.rows.is_empty() {
            return Err(an_err!(DtErrKind::InternalErr, "contains no data"));
        }
        let mut guess = *self;

        for _ in 0..8 {
            let mjd = guess.to_mjd_f();
            let offset = op_data
                .params(mjd)
                .ok_or_else(|| an_err!(DtErrKind::OutOfRange, "mjd: {mjd}"))?
                .offset;

            guess = self.sub(Dt::from_sec_f(offset)); // TODO: guess or self?
        }

        Ok(guess)
    }
}
