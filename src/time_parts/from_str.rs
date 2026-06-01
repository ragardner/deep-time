use crate::{DtErr, DtErrKind, Offset, Parser, TimeParts, an_err};

impl TimeParts {
    /// Low-level parser equivalent to `strptime` with a provided format string.
    ///
    /// This is the core entry point for format-string based parsing in the library.
    /// It supports a large range of `%` directives (the same as jiff pretty much).
    ///
    /// The parser populates a [`TimeParts`] struct with all fields that can be
    /// extracted from the input. After parsing, [`Self::finish`] is called
    /// automatically to apply defaults and validation.
    ///
    /// ## Parameters
    ///
    /// - `fmt`: The format string containing `%` directives.
    /// - `input`: The string to parse.
    /// - `inp_can_end_before_fmt`: If `true`, the input may end before the format
    ///   string is fully consumed (extra format specifiers are ignored).
    /// - `fmt_can_end_before_inp`: If `true`, the format may end before the input
    ///   is fully consumed (trailing characters in the input are allowed).
    /// - `allow_partial_date`: If `true`, a missing month/day will be defaulted
    ///   to `1` instead of returning an [`Incomplete`] error.
    ///
    /// ## Errors
    ///
    /// Returns [`DtErr`] for:
    /// - Parse failures (`InvalidFormat`, `OutOfRange`, etc.)
    /// - Incomplete data when `allow_partial_date` is `false`
    /// - Trailing characters (when `fmt_can_end_before_inp` is `false`)
    pub fn from_str(
        fmt: &str,
        input: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Result<TimeParts, DtErr> {
        let mut parts = TimeParts::new_utc();
        let mut parser = Parser::new(
            fmt.as_bytes(),
            input.as_bytes(),
            &mut parts,
            inp_can_end_before_fmt,
        );
        parser.parse()?;
        if parser.inp.is_empty() || fmt_can_end_before_inp {
            // All input consumed → finalize
            parts.finish(allow_partial_date)?;
            Ok(parts)
        } else {
            // Trailing characters remain
            Err(an_err!(DtErrKind::TrailingCharacters))
        }
    }

    /// Finalizes a [`TimeParts`] after parsing by applying sensible defaults and
    /// performing validation.
    ///
    /// This is called automatically by the various parsing paths (`from_str`,
    /// CCSDS parsers, etc.). It ensures the struct is in a consistent state
    /// before being turned into a full [`Dt`] or passed to other converters.
    ///
    /// ## Behavior
    ///
    /// - If a Unix timestamp is present, it takes precedence and the time
    ///   components are defaulted to `00:00:00.000000000` with a UTC offset.
    /// - Otherwise:
    ///   - Hour/minute/second/attoseconds/offset are defaulted to `0` / `Utc`.
    ///   - Leap seconds (`second == 60`) are detected and flagged.
    /// - Date completeness is checked in this priority order:
    ///   1. Calendar date (`year`, `month`, `day`)
    ///   2. Ordinal date (`year`, `day_of_year`)
    ///   3. ISO week date (`iso_week_year`, `iso_week`)
    /// - If `allow_partial_date` is `true`, missing month/day are defaulted to `1`.
    ///
    /// ## Errors
    ///
    /// - [`DtErrKind::Incomplete`] if no valid date representation is present.
    /// - [`DtErrKind::OutOfRange`] for seconds outside `0..=60`.
    pub fn finish(&mut self, allow_partial_date: bool) -> Result<&mut Self, DtErr> {
        if self.unix_timestamp_seconds.is_some() {
            if self.hr.is_none() {
                self.hr = Some(0);
            }
            if self.min.is_none() {
                self.min = Some(0);
            }
            if self.sec.is_none() {
                self.sec = Some(0);
            }
            if self.attos.is_none() {
                self.attos = Some(0);
            }
            if self.offset.is_none() {
                self.offset = Some(Offset::Utc);
            }
            return Ok(self);
        }

        // Sensible defaults for time components (most tests expect a full datetime)
        if self.hr.is_none() {
            self.hr = Some(0);
        }
        if self.min.is_none() {
            self.min = Some(0);
        }
        if let Some(sec) = self.sec {
            if sec == 60 {
                self.is_leap_sec = true;
            } else if sec > 60 {
                return Err(an_err!(DtErrKind::OutOfRange, "seconds (0..=60): {}", sec));
            }
        } else {
            self.sec = Some(0);
        }
        if self.attos.is_none() {
            self.attos = Some(0);
        }
        if self.offset.is_none() {
            self.offset = Some(Offset::Utc);
        }

        let has_calendar_date = if allow_partial_date {
            if self.day.is_none() {
                self.day = Some(1);
            }
            if self.mo.is_none() {
                self.mo = Some(1);
            }
            self.yr.is_some()
        } else {
            self.yr.is_some() && self.mo.is_some() && self.day.is_some()
        };
        let has_ordinal_date = self.yr.is_some() && self.day_of_yr.is_some();
        let has_iso_week_date = self.iso_wk_yr.is_some() && self.iso_wk.is_some();

        if !has_calendar_date && !has_ordinal_date && !has_iso_week_date {
            return Err(an_err!(DtErrKind::Incomplete));
        }

        Ok(self)
    }
}
