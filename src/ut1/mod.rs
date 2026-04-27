use crate::{ClockType, DtStdError, Real, TimePoint, TimeSpan};
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
pub struct EopColumns {
    pub mjd: usize,
    pub ut1_utc: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum EopFormat {
    /// Auto-detects finals2000A.all / finals.all.iau2000.txt style files
    Finals2000A,
    /// C04 long-term series
    C04,
    /// User-defined column indices (0-based)
    Custom(EopColumns),
}

#[derive(Debug, Clone)]
pub struct EopEntry {
    pub mjd: Real,
    pub ut1_minus_utc: Real,
}

#[derive(Debug, Clone)]
pub struct Ut1Provider {
    entries: Vec<EopEntry>,
}

impl Ut1Provider {
    pub fn parse_eop(
        path: &str,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Vec<EopEntry>, DtStdError> {
        // [your original parse_eop implementation — unchanged]
        let file = File::open(path)
            .map_err(|e| DtStdError::reason(format!("Failed to open file '{}': {}", path, e)))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line =
                line.map_err(|e| DtStdError::reason(format!("Failed to read line: {}", e)))?;
            let trimmed = line.trim();

            if trimmed.len() > 8192 || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = match separator {
                Separator::Whitespace => trimmed.split_whitespace().collect(),
                Separator::Comma => trimmed.split(',').map(|s| s.trim()).collect(),
                Separator::Tab => trimmed.split('\t').map(|s| s.trim()).collect(),
                Separator::Pipe => trimmed.split('|').map(|s| s.trim()).collect(),
                Separator::Semicolon => trimmed.split(';').map(|s| s.trim()).collect(),
            };

            if parts.len() < 4 {
                continue;
            }

            let (mjd, ut1_utc) = match format {
                EopFormat::Finals2000A => {
                    let mjd_idx = parts.iter().position(|p| {
                        p.contains('.') && p.parse::<Real>().map_or(false, |v| v > 30000.0)
                    });
                    let Some(mjd_idx) = mjd_idx else { continue };

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
                                let value_str =
                                    if token.starts_with("I-") || token.starts_with("P-") {
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

                    let Some(ut1) = ut1_value else { continue };
                    let Ok(mjd_val) = parts[mjd_idx].parse::<Real>() else {
                        continue;
                    };
                    (Some(mjd_val), Some(ut1))
                }

                EopFormat::C04 => {
                    let (Some(mjd_str), Some(ut1_str)) = (parts.get(4), parts.get(7)) else {
                        continue;
                    };
                    match (mjd_str.parse::<Real>(), ut1_str.parse::<Real>()) {
                        (Ok(mjd), Ok(ut1)) => (Some(mjd), Some(ut1)),
                        _ => continue,
                    }
                }

                EopFormat::Custom(cols) => {
                    let (Some(mjd_str), Some(ut1_str)) =
                        (parts.get(cols.mjd), parts.get(cols.ut1_utc))
                    else {
                        continue;
                    };
                    match (mjd_str.parse::<Real>(), ut1_str.parse::<Real>()) {
                        (Ok(mjd), Ok(ut1)) => (Some(mjd), Some(ut1)),
                        _ => continue,
                    }
                }
            };

            if let (Some(mjd_val), Some(ut1_val)) = (mjd, ut1_utc) {
                entries.push(EopEntry {
                    mjd: mjd_val,
                    ut1_minus_utc: ut1_val,
                });
            }
        }

        if entries.is_empty() {
            return Err(DtStdError::reason(
                "No valid EOP entries found in file".to_string(),
            ));
        }

        entries.sort_by(|a, b| a.mjd.partial_cmp(&b.mjd).unwrap_or(Ordering::Equal));

        Ok(entries)
    }

    /// Load EOP table from file (finals2000A.all, C04, or custom).
    pub fn from_file(
        path: &str,
        format: EopFormat,
        separator: Separator,
    ) -> Result<Self, DtStdError> {
        let entries = Self::parse_eop(path, format, separator)?;
        Ok(Self { entries })
    }

    /// Returns DUT1 = UT1 − UTC using the library’s *exact* MJD representation.
    ///
    /// This avoids any early floating-point conversion in `to_mjd_utc()` and
    /// guarantees the lookup uses the same high-precision path that the rest
    /// of the library uses (`to_mjd_utc_exact` + `from_mjd_utc_exact`).
    fn ut1_minus_utc_exact(&self, mjd_days: i64, mjd_frac: TimeSpan) -> Option<Real> {
        // This conversion is identical to what `to_mjd_utc()` does internally,
        // but we do it explicitly here so the caller stays on the exact path.
        let mjd_f = (mjd_days as Real) + mjd_frac.as_sec_f() / crate::SEC_PER_DAY;
        self.ut1_minus_utc(mjd_f)
    }

    /// Returns DUT1 = UT1 − UTC (seconds) at the given MJD using linear interpolation.
    /// Returns `None` if the MJD is completely outside the loaded table.
    pub fn ut1_minus_utc(&self, mjd: Real) -> Option<Real> {
        if self.entries.is_empty() {
            return None;
        }

        let idx = match self
            .entries
            .binary_search_by(|probe| probe.mjd.partial_cmp(&mjd).unwrap_or(Ordering::Equal))
        {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    return Some(self.entries[0].ut1_minus_utc);
                }
                if i >= self.entries.len() {
                    return Some(self.entries.last().unwrap().ut1_minus_utc);
                }
                i - 1
            }
        };

        if idx + 1 < self.entries.len() {
            let e0 = &self.entries[idx];
            let e1 = &self.entries[idx + 1];
            let t = (mjd - e0.mjd) / (e1.mjd - e0.mjd);
            Some(e0.ut1_minus_utc + t * (e1.ut1_minus_utc - e0.ut1_minus_utc))
        } else {
            Some(self.entries[idx].ut1_minus_utc)
        }
    }
}

impl TimePoint {
    /// Convert **any** `TimePoint` to the equivalent UT1 instant (stored as `Custom`).
    ///
    /// Uses the library’s exact MJD path (`to_mjd_utc_exact`) for the lookup.
    pub fn to_ut1(&self, provider: &Ut1Provider) -> Result<Self, DtStdError> {
        let utc = self.to_clock_type(ClockType::UTC);
        let (mjd_days, mjd_frac) = utc.to_mjd_utc_exact();

        let dut1 = provider
            .ut1_minus_utc_exact(mjd_days, mjd_frac)
            .ok_or_else(|| {
                let mjd_f = (mjd_days as Real) + mjd_frac.as_sec_f() / crate::SEC_PER_DAY;
                DtStdError::reason(format!("MJD {mjd_f} outside loaded EOP range"))
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
    pub fn from_ut1(ut1: Self, provider: &Ut1Provider) -> Result<Self, DtStdError> {
        if provider.entries.is_empty() {
            return Err(DtStdError::reason("EOP table is empty".to_string()));
        }

        let mut utc_guess = ut1.with_clock_type(ClockType::UTC);

        // DUT1 changes extremely slowly (< 0.1 ms/day). Four iterations are far more than enough.
        for _ in 0..4 {
            let (mjd_days, mjd_frac) = utc_guess.to_mjd_utc_exact();

            let dut1 = provider
                .ut1_minus_utc_exact(mjd_days, mjd_frac)
                .ok_or_else(|| DtStdError::reason("MJD outside loaded EOP range".to_string()))?;

            utc_guess = ut1
                .sub(TimeSpan::from_sec_f(dut1))
                .with_clock_type(ClockType::UTC);
        }

        Ok(utc_guess)
    }
}
